use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::sync::atomic::AtomicBool;

use super::*;

fn executable(path: impl AsRef<std::path::Path>) -> Executable {
    let path = std::fs::canonicalize(path).expect("reviewed executable");
    let blake3 = super::super::compatibility::executable_identity(&path).expect("executable bytes");
    Executable { path, blake3 }
}

// r[verify lattice.candidate_worker.model_broker.scenario.enforcement]
#[test]
#[ignore = "requires LATTICE_BWRAP, LATTICE_SOCAT and enabled Linux user/network namespaces"]
fn bubblewrap_broker_allows_only_the_model_route_and_hides_host_capabilities() {
    let directory = tempfile::tempdir().expect("workspace");
    let root = directory.path();
    std::fs::create_dir(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("base source");
    let secrets = tempfile::tempdir().expect("host secrets");
    let token = secrets.path().join("publisher-token");
    std::fs::write(&token, b"publisher-secret").expect("host secret");
    let host_socket = secrets.path().join("privileged.sock");
    let _privileged = std::os::unix::net::UnixListener::bind(&host_socket).expect("host socket");
    std::fs::write(root.join(".git"), format!("gitdir: {}", secrets.path().display())).expect("linked metadata");
    let endpoint = std::net::TcpListener::bind("127.0.0.1:0").expect("local endpoint");
    let address = endpoint.local_addr().expect("endpoint address");
    let relay = executable(std::env::var("LATTICE_SOCAT").expect("reviewed socat"));
    let shell = executable(which::which("sh").expect("shell"));
    let sandbox = executable(std::env::var("LATTICE_BWRAP").expect("reviewed bubblewrap"));
    let body = r#"{"model":"admitted-model","messages":[]}"#;
    let wire = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let script = format!(
        "test ! -e '{}' && test ! -e '{}' && test ! -e /workspace/.git && test -z \"$LATTICE_TEST_PUBLISHER_SECRET\" || exit 7; if '{}' -u OPEN:/dev/null TCP4:{address},connect-timeout=0.1 2>/dev/null; then exit 8; fi; response=$(printf '%s' '{}' | '{}' - TCP4:127.0.0.1:{PORT},connect-timeout=1); case \"$response\" in 'HTTP/1.1 200'*) printf new > src/main.rs ;; *) printf '%s' \"$response\" >&2; exit 9 ;; esac",
        token.display(),
        host_socket.display(),
        relay.path.display(),
        wire,
        relay.path.display(),
    );
    let mut request = super::super::tests::request(root, &script);
    request.program = shell.path.to_string_lossy().into_owned();
    request.require_isolation = true;
    request.sandbox_program = Some(sandbox.path.to_string_lossy().into_owned());
    request.timeout_secs = 10;
    let profile = Profile {
        schema: SCHEMA.into(),
        endpoint: format!("http://{address}/v1/chat/completions"),
        model: "admitted-model".into(),
        credential: Some(crate::credentials::SecretRef::new("env", "LATTICE_TEST_INFERENCE", None)),
        relay,
        shell,
        sandbox_blake3: sandbox.blake3,
        expires_at_ms: admission::now_ms().expect("clock") + 30_000,
        max_requests: 1,
        forwarded: None,
    };
    let credentials = crate::credentials::EnvironmentCredentialProvider::from_map(
        [("LATTICE_TEST_INFERENCE".into(), "host-inference-token".into())].into(),
    );
    let cancellation = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let observed = scope.spawn(|| {
            endpoint.set_nonblocking(true).expect("bounded accept");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let (mut stream, _) = loop {
                match endpoint.accept() {
                    Ok(value) => break value,
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock && std::time::Instant::now() < deadline =>
                    {
                        std::thread::sleep(std::time::Duration::from_millis(5))
                    }
                    Err(error) => panic!("model endpoint was not reached: {error}"),
                }
            };
            stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).expect("bounded read");
            let mut reader = std::io::BufReader::new(&mut stream);
            let mut headers = String::new();
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).expect("request");
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse::<usize>().expect("length");
                }
                headers.push_str(&line);
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).expect("body");
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
                .expect("response");
            assert!(headers.contains("Bearer host-inference-token"));
            assert!(!headers.contains("publisher-secret"));
        });
        let services = super::Services {
            credentials: &credentials,
            gate: &mut super::DirectRoute,
            first_request_index: 0,
        };
        let (outcome, clean) = with_route(request, &profile, services, &cancellation, |selected| {
            let mut crossed = selected.clone();
            crossed.attempt_id.push_str("-crossed");
            assert!(super::super::sandbox::command(&crossed).is_err());
            crossed = selected.clone();
            crossed.args.push("extra-authority".into());
            assert!(super::super::sandbox::command(&crossed).is_err());
            super::super::run(&selected, &crate::process::core::RuntimeConfig::default(), &cancellation)
        })
        .expect("admitted broker");
        assert!(clean);
        let outcome = outcome.expect("isolated worker");
        assert!(matches!(outcome, super::super::Outcome::Produced { .. }), "{outcome:?}");
        observed.join().expect("exact host inference call");
    });
    assert_eq!(std::fs::read(root.join("src/main.rs")).expect("candidate"), b"new");
    assert_eq!(std::fs::read(token).expect("host secret retained"), b"publisher-secret");
}
