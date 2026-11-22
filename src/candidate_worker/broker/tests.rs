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
        forwarded: None,
    }
}

/// A forwarding broker profile of `binding` at `vsock_port`, for `model`.
pub(crate) fn forwarding_profile(
    model: &str,
    max_requests: u32,
    binding: String,
    vsock_port: crate::vm_profile::VsockPort,
) -> Profile {
    let mut forwarding = profile();
    forwarding.model = model.into();
    forwarding.max_requests = max_requests;
    forwarding.forwarded = Some(Forwarded {
        schema: FORWARDED_SCHEMA.into(),
        vsock_port,
        binding,
    });
    forwarding
}

/// One worker request through a real forwarding broker handler whose first
/// forwarded index is `first_request_index`, against the gateway (or its
/// stand-in) at `gateway`; the raw HTTP response the worker reads.
#[cfg(unix)]
pub(crate) fn forward_one(profile: &Profile, gateway: &std::path::Path, first_request_index: u32) -> String {
    use std::io::Read;
    use std::io::Write;
    let credentials = crate::credentials::EmptyCredentialProvider;
    let mut gate = DirectRoute;
    let cancellation = std::sync::atomic::AtomicBool::new(false);
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let mut handler = wire::Handler {
        profile,
        credentials: &credentials,
        gate: &mut gate,
        cancellation: &cancellation,
        stopped: &stopped,
        deadline: std::time::Instant::now() + std::time::Duration::from_secs(5),
        requests: 0,
        failure: None,
        forwarding: Some(forwarded::Target::Unix(gateway.to_path_buf())),
        first_request_index,
    };
    let body = serde_json::json!({"model": profile.model}).to_string();
    let request = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let (server, mut client) = std::os::unix::net::UnixStream::pair().expect("private channel");
    client.set_read_timeout(Some(std::time::Duration::from_secs(10))).expect("bounded read");
    std::thread::scope(|scope| {
        let host = scope.spawn(|| handler.exchange(server));
        client.write_all(request.as_bytes()).expect("write request");
        client.shutdown(std::net::Shutdown::Write).expect("complete request");
        let mut response = String::new();
        client.read_to_string(&mut response).expect("bounded response");
        host.join().expect("host join").expect("response delivery");
        response
    })
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
            forwarding: None,
            first_request_index: 0,
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
            forwarding: None,
            first_request_index: 0,
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
            forwarding: None,
            first_request_index: 0,
        };
        assert!(
            exchange(&mut handler, request("/v1/chat/completions", r#"{"model":"admitted-model"}"#).as_bytes())
                .starts_with("HTTP/1.1 403")
        );
        assert_eq!(credentials.0.load(Ordering::SeqCst), 0);
    }

    /// Guest side of C4: the real `wire::Handler` forwarding over a Unix
    /// socket standing in for the Cloud Hypervisor vsock endpoint.
    // r[verify lattice.vm_profile.forwarded_upstream]
    mod forwarding {
        use std::io::Write;
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::Ordering;
        use std::time::Duration;
        use std::time::Instant;

        use crate::model_gateway::tests as harness;

        const GATEWAY_PORT: crate::vm_profile::VsockPort = crate::vm_profile::VsockPort(10243);

        fn forwarding_profile() -> super::super::Profile {
            let mut profile = super::super::profile();
            profile.model = harness::MODEL.into();
            profile.max_requests = 4;
            profile.forwarded = Some(super::super::Forwarded {
                schema: super::super::FORWARDED_SCHEMA.into(),
                vsock_port: GATEWAY_PORT,
                binding: harness::binding_identity(),
            });
            profile
        }

        fn worker_request() -> String {
            super::request("/v1/chat/completions", &serde_json::json!({"model": harness::MODEL}).to_string())
        }

        /// Records how each admitted call settled.
        #[derive(Default)]
        struct Recorder(Vec<String>);

        impl super::super::ModelRequestGate for Recorder {
            fn admit(
                &mut self,
                checks: crate::execution_binding::serving::gate::Checks,
            ) -> Option<super::super::GateTicket> {
                (checks.is_exact_model && checks.is_within_budget && checks.is_current)
                    .then(super::super::GateTicket::unserved)
            }

            fn settle(
                &mut self,
                _: super::super::GateTicket,
                upstream: crate::execution_binding::serving::gate::Upstream,
            ) -> Result<bool, String> {
                self.0.push(format!("{upstream:?}"));
                Ok(true)
            }

            fn exhausted(&mut self) -> Result<(), String> {
                Ok(())
            }
        }

        struct Flags {
            cancellation: AtomicBool,
            stopped: AtomicBool,
        }

        fn flags() -> Flags {
            Flags {
                cancellation: AtomicBool::new(false),
                stopped: AtomicBool::new(false),
            }
        }

        fn handler<'a>(
            profile: &'a super::super::Profile,
            credentials: &'a super::Credentials,
            gate: &'a mut dyn super::super::ModelRequestGate,
            flags: &'a Flags,
            gateway: &std::path::Path,
            within: Duration,
        ) -> super::super::wire::Handler<'a> {
            super::super::wire::Handler {
                profile,
                credentials,
                gate,
                cancellation: &flags.cancellation,
                stopped: &flags.stopped,
                deadline: Instant::now() + within,
                requests: 0,
                failure: None,
                forwarding: Some(super::super::forwarded::Target::Unix(gateway.to_path_buf())),
                first_request_index: 0,
            }
        }

        // r[verify lattice.vm_profile.forwarded_upstream.scenario.credential]
        #[test]
        fn guest_broker_reaches_the_upstream_only_through_the_host_gateway_without_a_credential() {
            let directory = tempfile::tempdir().expect("scratch");
            let authority =
                harness::Authority::start(directory.path(), "authority.sock", harness::admitting("host-a", 0, 4));
            let upstream = harness::Upstream::start(harness::response_body());
            let gateway_stopped = AtomicBool::new(false);
            let mut gateway =
                harness::gateway(&authority, &upstream, &gateway_stopped, &harness::fixed_clock, std::process::id());
            let socket = directory.path().join("vsock.sock_10243");
            let listener = crate::model_gateway::shell::listen(&socket).expect("gateway listener");
            let profile = forwarding_profile();
            let credentials = super::Credentials(std::sync::atomic::AtomicUsize::new(0));
            let mut recorder = Recorder::default();
            let flags = flags();
            let (admitted, denied) = std::thread::scope(|scope| {
                let served = scope.spawn(|| gateway.serve(&listener));
                let mut guest = handler(&profile, &credentials, &mut recorder, &flags, &socket, Duration::from_secs(5));
                let admitted = super::exchange(&mut guest, worker_request().as_bytes());
                let other = super::request("/v1/chat/completions", r#"{"model":"other"}"#);
                let denied = super::exchange(&mut guest, other.as_bytes());
                assert_eq!(guest.requests, 1);
                gateway_stopped.store(true, Ordering::Release);
                served.join().expect("gateway join").expect("gateway serve");
                (admitted, denied)
            });
            assert!(admitted.starts_with("HTTP/1.1 200"), "{admitted}");
            assert!(admitted.contains("Content-Type: application/json"), "{admitted}");
            assert!(!admitted.contains(harness::CANARY));
            assert!(denied.starts_with("HTTP/1.1 403"), "{denied}");
            assert_eq!(credentials.0.load(Ordering::SeqCst), 0, "the guest resolves no credential");
            assert_eq!(upstream.calls(), 1);
            let headers = upstream.headers.lock().expect("headers").join("");
            assert!(headers.contains(&format!("Bearer {}", harness::CANARY)));
            assert!(!headers.contains("worker-controlled"));
            assert_eq!(recorder.0.len(), 1);
            assert!(recorder.0[0].starts_with("Responded"), "{:?}", recorder.0);
        }

        enum Behavior {
            Reply(crate::model_gateway::core::ForwardedResponse),
            Close,
            Hold(Duration),
        }

        /// A stand-in gateway that answers one forwarded frame as scripted.
        fn stand_in(socket: &std::path::Path, behavior: Behavior) -> std::thread::JoinHandle<()> {
            let listener = std::os::unix::net::UnixListener::bind(socket).expect("stand-in socket");
            std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().expect("one forwarded call");
                let frame = lattice_workflow_transport::read_frame(&mut stream, 1 << 24).expect("C4 frame");
                let request: crate::model_gateway::core::ForwardedRequest =
                    serde_json::from_slice(&frame).expect("C4 request");
                assert_eq!(request.request_index, 0);
                assert_eq!(request.binding, harness::binding_identity());
                match behavior {
                    Behavior::Reply(reply) => {
                        let frame = lattice_workflow_transport::encode_frame(&reply, 1 << 24).expect("reply");
                        stream.write_all(&frame).expect("reply written");
                    }
                    Behavior::Close => drop(stream),
                    Behavior::Hold(duration) => std::thread::sleep(duration),
                }
            })
        }

        // r[verify lattice.vm_profile.forwarded_upstream.scenario.settlement]
        #[test]
        fn forwarded_replies_keep_the_direct_route_settlement_semantics() {
            let refused = crate::model_gateway::core::ForwardedResponse::Refused(crate::model_gateway::core::Refused {
                request_index: 0,
                refused: crate::model_gateway::core::ForwardedRefusal::Deadline,
            });
            let uncertain =
                crate::model_gateway::core::ForwardedResponse::Uncertain(crate::model_gateway::core::Uncertain {
                    request_index: 0,
                    uncertain: true,
                });
            let crossed =
                crate::model_gateway::core::ForwardedResponse::Responded(crate::model_gateway::core::Responded {
                    request_index: 5,
                    status: 200,
                    body: "{}".into(),
                });
            let cases = [
                (Behavior::Reply(refused), Duration::from_secs(5), false, "TransportFailed"),
                (Behavior::Reply(uncertain), Duration::from_secs(5), false, "TransportFailed"),
                (Behavior::Reply(crossed), Duration::from_secs(5), false, "TransportFailed"),
                (Behavior::Close, Duration::from_secs(5), false, "TransportFailed"),
                (Behavior::Hold(Duration::from_secs(3)), Duration::from_millis(1_500), false, "TimedOut"),
                (Behavior::Hold(Duration::from_secs(3)), Duration::from_secs(5), true, "Cancelled"),
            ];
            for (case, (behavior, within, is_cancelled, settled)) in cases.into_iter().enumerate() {
                let directory = tempfile::tempdir().expect("scratch");
                let socket = directory.path().join("vsock.sock_10243");
                let gateway = stand_in(&socket, behavior);
                let profile = forwarding_profile();
                let credentials = super::Credentials(std::sync::atomic::AtomicUsize::new(0));
                let mut recorder = Recorder::default();
                let flags = flags();
                let response = std::thread::scope(|scope| {
                    if is_cancelled {
                        scope.spawn(|| {
                            std::thread::sleep(Duration::from_millis(300));
                            flags.stopped.store(true, Ordering::Release);
                        });
                    }
                    let mut guest = handler(&profile, &credentials, &mut recorder, &flags, &socket, within);
                    super::exchange(&mut guest, worker_request().as_bytes())
                });
                gateway.join().expect("stand-in");
                assert!(response.starts_with("HTTP/1.1 403"), "case {case}: {response}");
                assert_eq!(recorder.0, [settled], "case {case}");
                assert_eq!(credentials.0.load(Ordering::SeqCst), 0);
            }
        }

        #[test]
        fn a_forwarding_profile_never_holds_a_credential() {
            let profile = forwarding_profile();
            super::super::admission::validate(&profile).expect("forwarding profile");
            let mut credentialed = profile.clone();
            credentialed.credential = Some(crate::credentials::SecretRef::new("env", "MODEL_TOKEN", None));
            assert!(super::super::admission::validate(&credentialed).is_err());
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/vm-profile/forwarded-upstream");
            let read = |name: &str| -> super::super::Profile {
                serde_json::from_slice(&std::fs::read(root.join(name)).expect(name)).expect(name)
            };
            super::super::admission::validate(&read("broker-profile.json")).expect("published forwarding profile");
            for name in ["refused-broker-credential.json", "refused-broker-vsock-port.json"] {
                assert!(super::super::admission::validate(&read(name)).is_err(), "{name}");
            }
            // Absent forwarding serializes exactly as before.
            let direct = serde_json::to_value(super::super::profile()).expect("profile");
            assert!(direct.get("forwarded").is_none());
        }

        // r[verify lattice.vm_profile.forwarded_upstream.scenario.no_fallback]
        #[test]
        fn a_vm_bound_broker_without_a_host_gateway_never_starts_the_worker() {
            let root = tempfile::tempdir().expect("workspace");
            let request = crate::candidate_worker::tests::request(root.path(), "true");
            let profile = forwarding_profile();
            let credentials = super::Credentials(std::sync::atomic::AtomicUsize::new(0));
            let mut gate = super::super::DirectRoute;
            let cancellation = AtomicBool::new(false);
            let started = AtomicBool::new(false);
            let services = super::super::Services {
                credentials: &credentials,
                gate: &mut gate,
                first_request_index: 0,
            };
            let refusal = super::super::with_route(request, &profile, services, &cancellation, |_| {
                started.store(true, Ordering::Release);
            })
            .expect_err("no AF_VSOCK host gateway on this host");
            assert!(refusal.contains("no host model gateway"), "{refusal}");
            assert!(!started.load(Ordering::Acquire), "the worker never started");
            assert_eq!(credentials.0.load(Ordering::SeqCst), 0);
        }
    }
}
