//! Real native transport regressions over bounded loopback servers.

use std::collections::HashMap;
use std::io::Read as _;
use std::io::Write as _;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

use lattice::http::HttpClient as _;
use lattice::http::HttpError;
use lattice::http::HttpLimit;
use lattice::http::HttpRequestPlan;
use lattice::http::NativeHttpClient;

const IO_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT_SECS: u64 = 3;
const SERVER_POLL: Duration = Duration::from_millis(5);
const SECRET: &str = "never-print-this-secret";

fn listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0").expect("isolated loopback listener")
}

fn plan(listener: &TcpListener) -> HttpRequestPlan {
    HttpRequestPlan {
        method: "GET".to_owned(),
        url: format!("http://{}/data?token={SECRET}", listener.local_addr().unwrap()),
        headers: HashMap::new(),
        body: None,
        timeout_secs: REQUEST_TIMEOUT_SECS,
    }
}

fn accept(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + IO_TIMEOUT;
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_read_timeout(Some(IO_TIMEOUT)).unwrap();
                stream.set_write_timeout(Some(IO_TIMEOUT)).unwrap();
                return stream;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(SERVER_POLL);
            }
            result => panic!("bounded server accept failed: {result:?}"),
        }
    }
}

fn read_head(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut byte = [0];
    while !bytes.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
        assert!(bytes.len() <= lattice::http::MAX_HEADER_BYTES + lattice::http::MAX_URL_BYTES);
    }
    String::from_utf8(bytes).unwrap()
}

fn reply(wire: Vec<u8>) -> Result<lattice::http::HttpResponse, HttpError> {
    reply_bounded(wire, &lattice::http::HttpBounds::default())
}

fn reply_bounded(wire: Vec<u8>, bounds: &lattice::http::HttpBounds) -> Result<lattice::http::HttpResponse, HttpError> {
    let listener = listener();
    let request = plan(&listener);
    std::thread::scope(|scope| {
        let server = scope.spawn(move || {
            let mut stream = accept(&listener);
            read_head(&mut stream);
            // Oversized responses are deliberately cut off by the client.
            let _ = stream.write_all(&wire);
        });
        let response = NativeHttpClient.execute_bounded(&request, bounds, None);
        server.join().unwrap();
        response
    })
}

// r[verify lattice.native_http.bounds]
// r[verify lattice.native_http.security]
#[test]
fn native_request_preserves_method_headers_body_and_response() {
    let listener = listener();
    let mut request = plan(&listener);
    request.method = "post".to_owned();
    request.body = Some("payload".to_owned());
    request.headers.insert("Authorization".to_owned(), format!("Bearer {SECRET}"));
    std::thread::scope(|scope| {
        let server = scope.spawn(move || {
            let mut stream = accept(&listener);
            let head = read_head(&mut stream);
            assert!(head.starts_with(&format!("POST /data?token={SECRET} HTTP/1.1\r\n")));
            assert!(head.contains(&format!("authorization: Bearer {SECRET}\r\n")));
            let mut body = [0; 7];
            stream.read_exact(&mut body).unwrap();
            assert_eq!(&body, b"payload");
            stream.write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 5\r\nX-Item: first\r\nX-Item: second\r\nConnection: close\r\n\r\nhello").unwrap();
        });
        let response = NativeHttpClient.execute(&request).unwrap();
        assert_eq!(response.status, 201);
        assert_eq!(response.body, "hello");
        assert_eq!(response.headers.get("x-item").unwrap(), "first\nsecond");
        server.join().unwrap();
    });
}

// r[verify lattice.native_http.bounds.scenario.request]
#[test]
fn malformed_and_oversized_requests_never_connect_or_disclose_content() {
    let listener = listener();
    listener.set_nonblocking(true).unwrap();
    let base = plan(&listener);
    let mut cases = Vec::new();
    let mut request = base.clone();
    request.url = format!("http://user:{SECRET}@{}/", listener.local_addr().unwrap());
    cases.push(request);
    let mut request = base.clone();
    request.headers.insert("x-secret".to_owned(), format!("{SECRET}\r\nInjected: true"));
    cases.push(request);
    for name in ["host", "content-length", "transfer-encoding", "proxy-authorization"] {
        let mut request = base.clone();
        request.headers.insert(name.to_owned(), SECRET.to_owned());
        cases.push(request);
    }
    let mut request = base.clone();
    request.headers.insert("X-Duplicate".to_owned(), SECRET.to_owned());
    request.headers.insert("x-duplicate".to_owned(), SECRET.to_owned());
    cases.push(request);
    for timeout in [0, lattice::http::MAX_TIMEOUT_SECS + 1] {
        let mut request = base.clone();
        request.timeout_secs = timeout;
        cases.push(request);
    }
    let mut request = base.clone();
    request.body = Some("x".repeat(lattice::http::MAX_REQUEST_BODY_BYTES + 1));
    cases.push(request);
    let mut request = base.clone();
    request.url.push_str(&"x".repeat(lattice::http::MAX_URL_BYTES));
    cases.push(request);
    let mut request = base.clone();
    request.headers.insert("x-big".to_owned(), "x".repeat(lattice::http::MAX_HEADER_BYTES));
    cases.push(request);
    let mut request = base;
    request.headers = (0..=lattice::http::MAX_HEADER_COUNT)
        .map(|index| (format!("x-{index}"), SECRET.to_owned()))
        .collect();
    cases.push(request);
    for request in cases {
        let error = NativeHttpClient.execute(&request).expect_err("request rejected before I/O");
        assert!(matches!(error, HttpError::InvalidRequest(_) | HttpError::LimitExceeded(_)));
        assert!(!format!("{error:?} {error}").contains(SECRET));
        assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    }
}

// r[verify lattice.native_http.bounds.scenario.response]
#[test]
fn response_body_and_header_limits_have_positive_and_negative_boundaries() {
    let allowed = "x".repeat(lattice::http::MAX_RESPONSE_BODY_BYTES);
    let response = reply(
        format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{allowed}", allowed.len())
            .into_bytes(),
    )
    .unwrap();
    assert_eq!(response.body, allowed);
    let oversized =
        format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", lattice::http::MAX_RESPONSE_BODY_BYTES + 1);
    assert_eq!(reply(oversized.into_bytes()).unwrap_err(), HttpError::LimitExceeded(HttpLimit::ResponseBody));
    // No Content-Length: the accumulation path must independently enforce the ceiling.
    let wire = format!(
        "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{}",
        "x".repeat(lattice::http::MAX_RESPONSE_BODY_BYTES + 1)
    );
    assert_eq!(reply(wire.into_bytes()).unwrap_err(), HttpError::LimitExceeded(HttpLimit::ResponseBody));
    // X: + value + CRLF contributes 1 + value + 4; Content-Length contributes 19.
    let exact = "x".repeat(lattice::http::MAX_HEADER_BYTES - 24);
    let wire = format!("HTTP/1.1 200 OK\r\nContent-Length: 0\r\nX: {exact}\r\n\r\n");
    assert_eq!(reply(wire.into_bytes()).unwrap().headers.get("x").unwrap(), &exact);
    let wire = format!("HTTP/1.1 200 OK\r\nContent-Length: 0\r\nX: {exact}x\r\n\r\n");
    assert_eq!(reply(wire.into_bytes()).unwrap_err(), HttpError::LimitExceeded(HttpLimit::ResponseHeaderBytes));
    for (count, allowed) in [
        (lattice::http::MAX_HEADER_COUNT, true),
        (lattice::http::MAX_HEADER_COUNT + 1, false),
    ] {
        let mut wire = "HTTP/1.1 200 OK\r\n".to_owned();
        for index in 0..count {
            wire.push_str(&format!("X-{index}: value\r\n"));
        }
        wire.push_str("\r\n");
        let result = reply(wire.into_bytes());
        if allowed {
            assert_eq!(result.unwrap().status, 200);
        } else {
            assert_eq!(result.unwrap_err(), HttpError::LimitExceeded(HttpLimit::ResponseHeaderCount));
        }
    }
}

// r[verify lattice.native_http.security.scenario.redirect]
#[test]
fn redirects_and_server_failures_are_returned_without_followup_requests() {
    let destination = listener();
    destination.set_nonblocking(true).unwrap();
    let wire = format!(
        "HTTP/1.1 302 Found\r\nLocation: http://{}/must-not-receive\r\nContent-Length: 0\r\n\r\n",
        destination.local_addr().unwrap()
    );
    assert_eq!(reply(wire.into_bytes()).unwrap().status, 302);
    assert_eq!(destination.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    assert_eq!(reply(b"HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\n\r\n".to_vec()).unwrap().status, 503);
}

// r[verify lattice.native_http.cancellation.scenario.timeout]
#[test]
fn timeout_and_truncated_responses_are_redacted_failures() {
    let listener = listener();
    let mut request = plan(&listener);
    request.timeout_secs = 1;
    let (release, wait) = mpsc::channel();
    std::thread::scope(|scope| {
        let server = scope.spawn(move || {
            let mut stream = accept(&listener);
            read_head(&mut stream);
            wait.recv_timeout(IO_TIMEOUT).unwrap();
        });
        let result = NativeHttpClient.execute(&request);
        release.send(()).unwrap();
        assert!(matches!(result.unwrap_err(), HttpError::Timeout(_)));
        server.join().unwrap();
    });
    let error = reply(format!("HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n{SECRET}").into_bytes()).unwrap_err();
    assert!(matches!(error, HttpError::Transport(_)));
    assert!(!format!("{error:?} {error}").contains(SECRET));
}

// r[verify lattice.native_http.cancellation.scenario.inflight]
#[test]
fn pre_and_inflight_cancellation_stop_local_http_work() {
    let listener = listener();
    let request = plan(&listener);
    listener.set_nonblocking(true).unwrap();
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        NativeHttpClient.execute_with_cancellation(&request, Some(&cancelled)).unwrap_err(),
        HttpError::Cancelled
    );
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    let cancelled = Arc::new(AtomicBool::new(false));
    let server_flag = Arc::clone(&cancelled);
    let (release, wait) = mpsc::channel();
    std::thread::scope(|scope| {
        let server = scope.spawn(move || {
            let mut stream = accept(&listener);
            read_head(&mut stream);
            server_flag.store(true, Ordering::Release);
            wait.recv_timeout(IO_TIMEOUT).unwrap();
        });
        let start = Instant::now();
        let result = NativeHttpClient.execute_with_cancellation(&request, Some(&cancelled));
        release.send(()).unwrap();
        assert_eq!(result.unwrap_err(), HttpError::Cancelled);
        assert!(start.elapsed() < Duration::from_secs(2), "cancel must not wait for request deadline");
        server.join().unwrap();
    });
}

// r[verify lattice.native_http.security]
#[test]
fn https_never_downgrades_to_plaintext() {
    let listener = listener();
    let mut request = plan(&listener);
    request.url = request.url.replacen("http:", "https:", 1);
    std::thread::scope(|scope| {
        let server = scope.spawn(move || {
            let mut stream = accept(&listener);
            let mut first = [0];
            stream.read_exact(&mut first).unwrap();
            assert_eq!(first[0], 0x16, "TLS handshake, not plaintext HTTP");
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").unwrap();
        });
        let error = NativeHttpClient.execute(&request).unwrap_err();
        assert!(matches!(error, HttpError::Transport(_)));
        assert!(!format!("{error:?} {error}").contains(SECRET));
        server.join().unwrap();
    });
}

// r[verify lattice.native_http.security.scenario.tls]
#[test]
fn untrusted_localhost_certificate_is_rejected_before_http() {
    use rustls::pki_types::pem::PemObject as _;

    // This key is public, checked-in test material, never a deployed credential.
    let certificate =
        rustls::pki_types::CertificateDer::from_pem_slice(include_bytes!("fixtures/native-http/localhost-cert.pem"))
            .unwrap();
    let key =
        rustls::pki_types::PrivateKeyDer::from_pem_slice(include_bytes!("fixtures/native-http/localhost-key.pem"))
            .unwrap();
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate], key)
        .unwrap();
    let listener = listener();
    let mut request = plan(&listener);
    request.url = format!("https://localhost:{}/private?token={SECRET}", listener.local_addr().unwrap().port());
    std::thread::scope(|scope| {
        let server = scope.spawn(move || {
            let mut stream = accept(&listener);
            let mut tls = rustls::ServerConnection::new(Arc::new(config)).unwrap();
            let deadline = Instant::now() + IO_TIMEOUT;
            loop {
                assert!(Instant::now() < deadline, "TLS teardown is finite");
                match tls.complete_io(&mut stream) {
                    Err(_) => return,
                    Ok(_) if !tls.is_handshaking() => {
                        panic!("untrusted certificate must not complete TLS or receive HTTP")
                    }
                    Ok(_) => {}
                }
            }
        });
        let error = NativeHttpClient.execute(&request).unwrap_err();
        assert!(matches!(error, HttpError::Transport(_)));
        assert!(!format!("{error:?} {error}").contains(SECRET));
        server.join().unwrap();
    });
}

// r[verify lattice.native_http.bounds]
#[test]
fn caller_can_narrow_streaming_bounds_but_cannot_enlarge_or_disable_them() {
    let bounds = lattice::http::HttpBounds {
        response_body_bytes: 32,
        header_bytes: 128,
        connect_timeout_secs: 1,
    };
    for size in [bounds.response_body_bytes, bounds.response_body_bytes + 1] {
        let wire = format!("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{}", "x".repeat(size));
        let response = reply_bounded(wire.into_bytes(), &bounds);
        if size == bounds.response_body_bytes {
            assert_eq!(response.unwrap().body, "x".repeat(size));
        } else {
            assert_eq!(response.unwrap_err(), HttpError::LimitExceeded(HttpLimit::ResponseBody));
        }
    }
    let exact = "x".repeat(bounds.header_bytes - 24);
    let wire = format!("HTTP/1.1 200 OK\r\nContent-Length: 0\r\nX: {exact}\r\n\r\n");
    assert_eq!(reply_bounded(wire.into_bytes(), &bounds).unwrap().headers.get("x").unwrap(), &exact);
    let wire = format!("HTTP/1.1 200 OK\r\nContent-Length: 0\r\nX: {exact}x\r\n\r\n");
    assert_eq!(
        reply_bounded(wire.into_bytes(), &bounds).unwrap_err(),
        HttpError::LimitExceeded(HttpLimit::ResponseHeaderBytes)
    );

    let listener = listener();
    listener.set_nonblocking(true).unwrap();
    let request = plan(&listener);
    let defaults = lattice::http::HttpBounds::default();
    for invalid in [
        lattice::http::HttpBounds {
            response_body_bytes: 0,
            ..defaults
        },
        lattice::http::HttpBounds {
            response_body_bytes: lattice::http::MAX_RESPONSE_BODY_BYTES + 1,
            ..defaults
        },
        lattice::http::HttpBounds {
            header_bytes: 0,
            ..defaults
        },
        lattice::http::HttpBounds {
            header_bytes: lattice::http::MAX_HEADER_BYTES + 1,
            ..defaults
        },
        lattice::http::HttpBounds {
            connect_timeout_secs: 0,
            ..defaults
        },
        lattice::http::HttpBounds {
            connect_timeout_secs: lattice::http::MAX_CONNECT_TIMEOUT_SECS + 1,
            ..defaults
        },
    ] {
        assert!(matches!(
            NativeHttpClient.execute_bounded(&request, &invalid, None),
            Err(HttpError::InvalidRequest(_))
        ));
        assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    }
}
