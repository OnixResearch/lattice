use super::*;

pub(crate) fn profile() -> Profile {
    let executable = Executable {
        path: "/nix/store/fixture".into(),
        blake3: "a".repeat(64),
    };
    Profile {
        schema: SCHEMA.into(),
        endpoint: "http://127.0.0.1:1/v1/chat/completions".into(),
        model: "admitted-model".into(),
        credential: None,
        relay: executable.clone(),
        shell: executable,
        sandbox_blake3: "b".repeat(64),
        expires_at_ms: u64::MAX,
        max_requests: 1,
    }
}

#[test]
fn endpoint_policy_rejects_url_credentials_redirect_surfaces_and_unbounded_calls() {
    let accepted = profile();
    admission::validate(&accepted).expect("literal loopback route");
    let mut value = accepted.clone();
    value.endpoint = "https://model.example/v1/chat/completions".into();
    admission::validate(&value).expect("explicit HTTPS route");
    for endpoint in [
        "http://localhost/v1/chat/completions",
        "http://192.0.2.1/v1/chat/completions",
        "https://user:secret@model.example/v1/chat/completions",
        "https://model.example/v1/chat/completions?url=other",
        "https://model.example/v1/chat/completions#other",
        "https://model.example/v1/models",
    ] {
        value.endpoint = endpoint.into();
        assert!(admission::validate(&value).is_err(), "{endpoint}");
    }
    for max_requests in [0, admission::MAX_REQUESTS + 1] {
        value = accepted.clone();
        value.max_requests = max_requests;
        assert!(admission::validate(&value).is_err());
    }
}

#[cfg(unix)]
mod transport {
    use std::io::BufRead;
    use std::io::Read;
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use std::time::Instant;

    use super::*;

    struct Credentials(AtomicUsize);
    impl crate::credentials::CredentialProvider for Credentials {
        fn resolve(
            &self,
            _: &crate::credentials::SecretRef,
        ) -> Result<crate::credentials::SecretValue, crate::credentials::CredentialError> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(crate::credentials::SecretValue::new("host-only-secret"))
        }
    }

    fn exchange(handler: &mut wire::Handler<'_>, request: &[u8]) -> String {
        let (server, mut client) = UnixStream::pair().expect("private channel");
        client.set_read_timeout(Some(Duration::from_secs(5))).expect("bounded read");
        std::thread::scope(|scope| {
            let host = scope.spawn(|| handler.exchange(server));
            client.write_all(request).expect("write request");
            client.shutdown(std::net::Shutdown::Write).expect("complete request");
            let mut response = String::new();
            client.read_to_string(&mut response).expect("bounded response");
            host.join().expect("host join").expect("response delivery");
            response
        })
    }

    pub(super) fn request(path: &str, body: &str) -> String {
        format!(
            "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAuthorization: Bearer worker-controlled\r\n\r\n{body}",
            body.len()
        )
    }

    #[test]
    fn actual_socket_route_confines_destination_model_credentials_and_request_count() {
        let endpoint = std::net::TcpListener::bind("127.0.0.1:0").expect("local endpoint");
        let address = endpoint.local_addr().expect("endpoint address");
        let upstream = std::thread::spawn(move || {
            let (mut stream, _) = endpoint.accept().expect("one admitted request");
            stream.set_read_timeout(Some(Duration::from_secs(5))).expect("bounded read");
            let mut reader = std::io::BufReader::new(&mut stream);
            let mut headers = String::new();
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).expect("request headers");
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse::<usize>().expect("body length");
                }
                headers.push_str(&line);
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).expect("request body");
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
                .expect("response");
            (headers, body)
        });
        let mut profile = profile();
        profile.endpoint = format!("http://{address}/v1/chat/completions");
        profile.credential = Some(crate::credentials::SecretRef::new("fixture", "key", None));
        let credentials = Credentials(AtomicUsize::new(0));
        let cancellation = AtomicBool::new(false);
        let stopped = AtomicBool::new(false);
        let mut direct = crate::candidate_worker::broker::DirectRoute;
        let mut handler = wire::Handler {
            profile: &profile,
            credentials: &credentials,
            gate: &mut direct,
            cancellation: &cancellation,
            stopped: &stopped,
            deadline: Instant::now() + Duration::from_secs(5),
            requests: 0,
            failure: None,
        };
        let denied = request("/v1/chat/completions", r#"{"model":"ungranted-model"}"#);
        assert!(exchange(&mut handler, denied.as_bytes()).starts_with("HTTP/1.1 403"));
        assert_eq!(credentials.0.load(Ordering::SeqCst), 0);
        let admitted = request("/v1/chat/completions", r#"{"model":"other","model":"admitted-model"}"#);
        let response = exchange(&mut handler, admitted.as_bytes());
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(!response.contains("host-only-secret"));
        assert!(exchange(&mut handler, admitted.as_bytes()).starts_with("HTTP/1.1 403"));
        assert_eq!(credentials.0.load(Ordering::SeqCst), 1);
        let (headers, body) = upstream.join().expect("one endpoint invocation");
        assert!(headers.contains("Bearer host-only-secret"));
        assert!(!headers.contains("worker-controlled"));
        assert_eq!(std::str::from_utf8(&body).expect("canonical body").matches("\"model\"").count(), 1);
        assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).expect("JSON")["model"], "admitted-model");
    }

    // r[verify lattice.candidate_worker.model_broker.scenario.denial]
    #[test]
    fn denied_and_cancelled_calls_never_resolve_credentials() {
        let mut profile = profile();
        profile.credential = Some(crate::credentials::SecretRef::new("fixture", "key", None));
        let credentials = Credentials(AtomicUsize::new(0));
        let cancellation = AtomicBool::new(false);
        let stopped = AtomicBool::new(false);
        let (mut direct, mut replacement) =
            (crate::candidate_worker::broker::DirectRoute, crate::candidate_worker::broker::DirectRoute);
        let mut handler = wire::Handler {
            profile: &profile,
            credentials: &credentials,
            gate: &mut direct,
            cancellation: &cancellation,
            stopped: &stopped,
            deadline: Instant::now() + Duration::from_secs(5),
            requests: 0,
            failure: None,
        };
        for invalid in [
            request("/v1/models", r#"{"model":"admitted-model"}"#),
            request("/v1/chat/completions", r#"{"model":"other"}"#),
            "CONNECT other:443 HTTP/1.1\r\n\r\n".into(),
            format!(
                "POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nContent-Type: application/json\r\nContent-Length: 2\r\nTransfer-Encoding: chunked\r\n\r\n{{}}"
            ),
            format!(
                "POST /v1/chat/completions HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
                crate::http::MAX_REQUEST_BODY_BYTES + 1
            ),
        ] {
            assert!(exchange(&mut handler, invalid.as_bytes()).starts_with("HTTP/1.1 403"));
        }
        cancellation.store(true, Ordering::Release);
        assert!(
            exchange(&mut handler, request("/v1/chat/completions", r#"{"model":"admitted-model"}"#).as_bytes())
                .starts_with("HTTP/1.1 403")
        );
        assert_eq!(credentials.0.load(Ordering::SeqCst), 0);
        assert_eq!(handler.requests, 0);
        cancellation.store(false, Ordering::Release);
        profile.expires_at_ms = 0;
        handler = wire::Handler {
            profile: &profile,
            credentials: &credentials,
            gate: &mut replacement,
            cancellation: &cancellation,
            stopped: &stopped,
            deadline: Instant::now() + Duration::from_secs(5),
            requests: 0,
            failure: None,
        };
        assert!(
            exchange(&mut handler, request("/v1/chat/completions", r#"{"model":"admitted-model"}"#).as_bytes())
                .starts_with("HTTP/1.1 403")
        );
        assert_eq!(credentials.0.load(Ordering::SeqCst), 0);
    }
}
