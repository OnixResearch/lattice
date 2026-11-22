//! Q1 (in-process): the real gateway shell and `KilnAuthorityGate` over real
//! Unix sockets, a scripted C3 authority and a scripted OpenAI-compatible
//! upstream with a host-side request counter. No VM, no vsock, no live model.

// r[verify lattice.vm_profile.gateway]
// r[verify lattice.vm_profile.effect_authority]

use std::io::BufRead;
use std::io::Read;
use std::io::Write;

use super::core::AuthorityReply;
use super::core::AuthorityRequest;
use super::core::ForwardedRefusal;
use super::core::ForwardedResponse;

pub(crate) const CANARY: &str = "host-canary-credential";
pub(crate) const MODEL: &str = "admitted-model";
pub(crate) const DEADLINE_MS: u64 = 1_000_000;
const MAX_REQUESTS: u32 = 4;
const NOW_MS: u64 = 1_000;

type Script = Box<dyn FnMut(&AuthorityRequest) -> Option<AuthorityReply> + Send>;

/// A scripted Kiln effect authority. A script returning `None` closes the
/// connection without a reply: a lost reply.
pub(crate) struct Authority {
    pub socket: std::path::PathBuf,
    pub requests: std::sync::Arc<std::sync::Mutex<Vec<AuthorityRequest>>>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Authority {
    pub(crate) fn start(directory: &std::path::Path, name: &str, mut script: Script) -> Self {
        let socket = directory.join(name);
        let listener = std::os::unix::net::UnixListener::bind(&socket).expect("authority socket");
        listener.set_nonblocking(true).expect("bounded accept");
        let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (seen, stopped) = (requests.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            while !stopped.load(std::sync::atomic::Ordering::Acquire) {
                let Ok((mut stream, _)) = listener.accept() else {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                    continue;
                };
                stream.set_nonblocking(false).expect("blocking stream");
                let frame = lattice_workflow_transport::read_frame(&mut stream, super::core::MAX_AUTHORITY_FRAME_BYTES)
                    .expect("one C3 frame");
                let request: AuthorityRequest = serde_json::from_slice(&frame).expect("C3 request");
                seen.lock().expect("requests").push(request.clone());
                if let Some(reply) = script(&request) {
                    let frame =
                        lattice_workflow_transport::encode_frame(&reply, super::core::MAX_AUTHORITY_FRAME_BYTES)
                            .expect("C3 reply");
                    lattice_workflow_transport::write_frame(&mut stream, &frame).expect("reply written");
                }
            }
        });
        Self {
            socket,
            requests,
            stop,
            thread: Some(thread),
        }
    }

    pub(crate) fn seen(&self) -> Vec<AuthorityRequest> {
        self.requests.lock().expect("requests").clone()
    }
}

impl Drop for Authority {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// An authority that admits every request with the carried consumption
/// starting at `consumed` and records every settlement.
pub(crate) fn admitting(host: &'static str, consumed: u32, maximum: u32) -> Script {
    let mut consumed_requests = consumed;
    Box::new(move |request| match request {
        AuthorityRequest::Admit { .. } => {
            if consumed_requests >= maximum {
                return Some(AuthorityReply::Refused {
                    code: super::core::AuthorityRefusal::Budget,
                });
            }
            consumed_requests += 1;
            Some(AuthorityReply::Admitted {
                placement_host: host.into(),
                placement_generation: 1,
                consumed_requests,
                remaining_requests: maximum - consumed_requests,
                deadline_unix_ms: DEADLINE_MS,
            })
        }
        AuthorityRequest::Settle { .. } => Some(AuthorityReply::Recorded { consumed_requests }),
    })
}

/// A scripted OpenAI-compatible upstream; `calls` is the host-side oracle.
pub(crate) struct Upstream {
    pub endpoint: String,
    pub calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    pub headers: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Upstream {
    pub(crate) fn start(body: String) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("upstream");
        let endpoint = format!("http://{}/v1/chat/completions", listener.local_addr().expect("address"));
        listener.set_nonblocking(true).expect("bounded accept");
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let headers = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (counted, recorded, stopped) = (calls.clone(), headers.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            while !stopped.load(std::sync::atomic::Ordering::Acquire) {
                let Ok((mut stream, _)) = listener.accept() else {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                    continue;
                };
                stream.set_nonblocking(false).expect("blocking stream");
                stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).expect("bounded read");
                let mut reader = std::io::BufReader::new(&mut stream);
                let mut head = String::new();
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).expect("request line");
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse::<usize>().expect("length");
                    }
                    head.push_str(&line);
                }
                let mut request = vec![0; length];
                reader.read_exact(&mut request).expect("request body");
                counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                recorded.lock().expect("headers").push(head);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).expect("response");
            }
        });
        Self {
            endpoint,
            calls,
            headers,
            stop,
            thread: Some(thread),
        }
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Drop for Upstream {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(crate) fn response_body() -> String {
    serde_json::json!({"id": "resp-1", "model": MODEL, "choices": [], "usage": {"prompt_tokens": 12, "completion_tokens": 3}})
        .to_string()
}

pub(crate) fn binding_identity() -> String {
    let vectors: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/vm-profile/vectors.json"))
            .expect("vectors"),
    )
    .expect("vectors JSON");
    vectors["binding"]["identity"].as_str().expect("binding identity").to_owned()
}

pub(crate) fn bound() -> super::admission::Bound {
    super::admission::Bound {
        binding: binding_identity(),
        deadline_unix_ms: DEADLINE_MS,
        model: MODEL.into(),
        max_requests: MAX_REQUESTS,
    }
}

pub(crate) fn fixed_clock() -> Result<u64, String> {
    Ok(NOW_MS)
}

pub(crate) fn gateway<'a>(
    authority: &Authority,
    upstream: &Upstream,
    stopped: &'a std::sync::atomic::AtomicBool,
    clock: &'a (dyn Fn() -> Result<u64, String> + Sync),
    vmm_pid: u32,
) -> super::shell::Server<'a, super::authority::UnixSocket> {
    super::shell::Server {
        session: super::admission::Session::new(bound()),
        gate: super::authority::KilnAuthorityGate::new(
            super::authority::UnixSocket {
                socket: authority.socket.clone(),
            },
            bound(),
        ),
        endpoint: upstream.endpoint.clone(),
        credential: Some(crate::credentials::SecretValue::new(CANARY)),
        vmm_pid,
        stopped,
        clock,
        dropped_connections: 0,
        undelivered_replies: 0,
    }
}

pub(crate) fn forwarded(request_index: u32) -> super::core::ForwardedRequest {
    super::core::ForwardedRequest {
        binding: binding_identity(),
        request_index,
        body: serde_json::json!({"model": MODEL, "messages": []}).to_string(),
    }
}

fn refused(reply: &ForwardedResponse) -> Option<ForwardedRefusal> {
    match reply {
        ForwardedResponse::Refused(value) => Some(value.refused),
        ForwardedResponse::Responded(_) | ForwardedResponse::Uncertain(_) => None,
    }
}

fn kinds(requests: &[AuthorityRequest]) -> Vec<&'static str> {
    requests
        .iter()
        .map(|request| match request {
            AuthorityRequest::Admit { .. } => "admit",
            AuthorityRequest::Settle { .. } => "settle",
        })
        .collect()
}

#[test]
fn admit_precedes_one_upstream_call_with_the_host_credential_and_settle_follows() {
    let directory = tempfile::tempdir().expect("scratch");
    let authority = Authority::start(directory.path(), "authority.sock", admitting("host-a", 0, MAX_REQUESTS));
    let upstream = Upstream::start(response_body());
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let mut gateway = gateway(&authority, &upstream, &stopped, &fixed_clock, std::process::id());
    let reply = gateway.decide(forwarded(0));
    let ForwardedResponse::Responded(responded) = reply else {
        panic!("admitted request responds: {reply:?}");
    };
    assert_eq!((responded.request_index, responded.status), (0, 200));
    assert!(!responded.body.contains(CANARY));
    assert_eq!(upstream.calls(), 1);
    assert!(upstream.headers.lock().expect("headers")[0].contains(&format!("Bearer {CANARY}")));
    let seen = authority.seen();
    assert_eq!(kinds(&seen), ["admit", "settle"]);
    assert_eq!(seen[0], AuthorityRequest::Admit {
        binding: binding_identity(),
        request_index: 0,
        model: MODEL.into(),
        route: super::core::ROUTE.into(),
    });
    assert_eq!(seen[1], AuthorityRequest::Settle {
        binding: binding_identity(),
        request_index: 0,
        outcome: super::core::SettleOutcome::Completed,
        usage: Some(super::core::Usage {
            input_tokens: Some(12),
            output_tokens: Some(3),
        }),
    });
}

// r[verify lattice.vm_profile.gateway.scenario.authority]
#[test]
fn every_authority_refusal_reaches_the_guest_and_never_the_upstream() {
    let directory = tempfile::tempdir().expect("scratch");
    let upstream = Upstream::start(response_body());
    for code in [
        super::core::AuthorityRefusal::NotCurrent,
        super::core::AuthorityRefusal::Deadline,
        super::core::AuthorityRefusal::Budget,
        super::core::AuthorityRefusal::Revoked,
        super::core::AuthorityRefusal::UnknownBinding,
    ] {
        let name = format!("{code:?}.sock");
        let authority =
            Authority::start(directory.path(), &name, Box::new(move |_| Some(AuthorityReply::Refused { code })));
        let stopped = std::sync::atomic::AtomicBool::new(false);
        let mut gateway = gateway(&authority, &upstream, &stopped, &fixed_clock, std::process::id());
        assert_eq!(refused(&gateway.decide(forwarded(0))), Some(code.into()), "{code:?}");
        assert_eq!(kinds(&authority.seen()), ["admit"], "{code:?}");
    }
    assert_eq!(upstream.calls(), 0);
}

// r[verify lattice.vm_profile.gateway.scenario.lost_settle]
#[test]
fn lost_settle_leaves_the_request_uncertain_and_never_calls_the_upstream_again() {
    let directory = tempfile::tempdir().expect("scratch");
    let mut admit = admitting("host-a", 0, MAX_REQUESTS);
    let authority = Authority::start(
        directory.path(),
        "authority.sock",
        Box::new(move |request| match request {
            AuthorityRequest::Settle { .. } => None,
            AuthorityRequest::Admit { .. } => admit(request),
        }),
    );
    let upstream = Upstream::start(response_body());
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let mut gateway = gateway(&authority, &upstream, &stopped, &fixed_clock, std::process::id());
    assert_eq!(
        gateway.decide(forwarded(0)),
        ForwardedResponse::Uncertain(super::core::Uncertain {
            request_index: 0,
            uncertain: true,
        })
    );
    assert_eq!(upstream.calls(), 1);
    // A replay of the uncertain request is refused locally: no Admit, no upstream.
    assert_eq!(refused(&gateway.decide(forwarded(0))), Some(ForwardedRefusal::NotCurrent));
    assert_eq!(upstream.calls(), 1);
    assert_eq!(kinds(&authority.seen()), ["admit", "settle"]);
}

#[test]
fn lost_admit_reply_refuses_without_upstream_contact() {
    let directory = tempfile::tempdir().expect("scratch");
    let authority = Authority::start(directory.path(), "authority.sock", Box::new(|_| None));
    let upstream = Upstream::start(response_body());
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let mut gateway = gateway(&authority, &upstream, &stopped, &fixed_clock, std::process::id());
    assert_eq!(refused(&gateway.decide(forwarded(0))), Some(ForwardedRefusal::NotCurrent));
    assert_eq!(upstream.calls(), 0);
}

// r[verify lattice.vm_profile.gateway.scenario.deadline]
#[test]
fn the_host_clock_decides_the_deadline_and_no_authority_may_extend_it() {
    let directory = tempfile::tempdir().expect("scratch");
    let upstream = Upstream::start(response_body());
    let authority = Authority::start(directory.path(), "late.sock", admitting("host-a", 0, MAX_REQUESTS));
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let past_deadline = || Ok(DEADLINE_MS);
    let mut late = gateway(&authority, &upstream, &stopped, &past_deadline, std::process::id());
    assert_eq!(refused(&late.decide(forwarded(0))), Some(ForwardedRefusal::Deadline));
    assert!(authority.seen().is_empty(), "a request past the host deadline never reaches the authority");
    let extending = Authority::start(
        directory.path(),
        "extending.sock",
        Box::new(|_| {
            Some(AuthorityReply::Admitted {
                placement_host: "host-a".into(),
                placement_generation: 1,
                consumed_requests: 1,
                remaining_requests: 1,
                deadline_unix_ms: DEADLINE_MS + 60_000,
            })
        }),
    );
    let mut extended = gateway(&extending, &upstream, &stopped, &fixed_clock, std::process::id());
    assert_eq!(refused(&extended.decide(forwarded(0))), Some(ForwardedRefusal::Deadline));
    assert_eq!(upstream.calls(), 0);
}

// r[verify lattice.vm_profile.gateway.scenario.budget_continuity]
#[test]
fn budget_continues_from_the_consumption_a_new_host_authority_carries() {
    let directory = tempfile::tempdir().expect("scratch");
    let upstream = Upstream::start(response_body());
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let host_a = Authority::start(directory.path(), "a.sock", admitting("host-a", 0, MAX_REQUESTS));
    let mut before = gateway(&host_a, &upstream, &stopped, &fixed_clock, std::process::id());
    for index in 0..3 {
        assert!(matches!(before.decide(forwarded(index)), ForwardedResponse::Responded(_)));
    }
    // The move: a new gateway incarnation and a new host authority that carries
    // the three consumed requests. The guest's request index continues.
    let host_b = Authority::start(directory.path(), "b.sock", admitting("host-b", 3, MAX_REQUESTS));
    let mut after = gateway(&host_b, &upstream, &stopped, &fixed_clock, std::process::id());
    assert!(matches!(after.decide(forwarded(3)), ForwardedResponse::Responded(_)));
    assert_eq!(after.gate.admission(), None, "settled admissions are released");
    assert_eq!(refused(&after.decide(forwarded(4))), Some(ForwardedRefusal::Budget));
    assert_eq!(upstream.calls(), 4, "the new host served only the carried remainder");
    // A broken authority that reports more consumption than the ceiling is caught.
    let broken = Authority::start(
        directory.path(),
        "broken.sock",
        Box::new(|_| {
            Some(AuthorityReply::Admitted {
                placement_host: "host-c".into(),
                placement_generation: 2,
                consumed_requests: MAX_REQUESTS + 1,
                remaining_requests: 0,
                deadline_unix_ms: DEADLINE_MS,
            })
        }),
    );
    let mut overrun = gateway(&broken, &upstream, &stopped, &fixed_clock, std::process::id());
    assert_eq!(refused(&overrun.decide(forwarded(5))), Some(ForwardedRefusal::Budget));
    assert_eq!(upstream.calls(), 4);
}

#[test]
fn crossed_binding_malformed_body_and_echoed_credential_never_reach_the_guest() {
    let directory = tempfile::tempdir().expect("scratch");
    let authority = Authority::start(directory.path(), "authority.sock", admitting("host-a", 0, MAX_REQUESTS));
    let echo = Upstream::start(serde_json::json!({"model": MODEL, "echo": CANARY}).to_string());
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let mut gateway = gateway(&authority, &echo, &stopped, &fixed_clock, std::process::id());
    let mut crossed = forwarded(0);
    crossed.binding = format!("b3:{}", "6".repeat(64));
    assert_eq!(refused(&gateway.decide(crossed)), Some(ForwardedRefusal::UnknownBinding));
    let mut other_model = forwarded(0);
    other_model.body = serde_json::json!({"model": "other"}).to_string();
    assert_eq!(refused(&gateway.decide(other_model)), Some(ForwardedRefusal::Malformed));
    assert!(authority.seen().is_empty());
    assert!(matches!(gateway.decide(forwarded(0)), ForwardedResponse::Uncertain(_)));
    assert_eq!(echo.calls(), 1);
    assert!(matches!(
        authority.seen().last(),
        Some(AuthorityRequest::Settle {
            outcome: super::core::SettleOutcome::Failed,
            ..
        })
    ));
}

/// Round trip through the real listener, then a peer that is not the VMM.
// r[verify lattice.vm_profile.gateway.scenario.peer]
#[test]
fn only_the_vmm_peer_is_served_on_the_listener() {
    let directory = tempfile::tempdir().expect("scratch");
    let authority = Authority::start(directory.path(), "authority.sock", admitting("host-a", 0, MAX_REQUESTS));
    let upstream = Upstream::start(response_body());
    for (vmm_pid, is_served) in [(std::process::id(), true), (1, false)] {
        let socket = directory.path().join(format!("vsock.sock_10243-{vmm_pid}"));
        let listener = super::shell::listen(&socket).expect("listener");
        let stopped = std::sync::atomic::AtomicBool::new(false);
        let mut gateway = gateway(&authority, &upstream, &stopped, &fixed_clock, vmm_pid);
        let index = if is_served { 0 } else { 1 };
        std::thread::scope(|scope| {
            let served = scope.spawn(|| gateway.serve(&listener));
            let mut client = std::os::unix::net::UnixStream::connect(&socket).expect("connect");
            client.set_read_timeout(Some(std::time::Duration::from_secs(10))).expect("bounded read");
            let frame = lattice_workflow_transport::encode_frame(&forwarded(index), 1 << 20).expect("frame");
            // A dropped peer may close before the frame is written.
            let written = lattice_workflow_transport::write_frame(&mut client, &frame);
            assert!(written.is_ok() || !is_served, "pid {vmm_pid}");
            let reply = lattice_workflow_transport::read_frame(&mut client, 1 << 24);
            stopped.store(true, std::sync::atomic::Ordering::Release);
            served.join().expect("serve join").expect("serve");
            match reply {
                Ok(bytes) => {
                    assert!(is_served, "pid {vmm_pid}");
                    let reply: ForwardedResponse = serde_json::from_slice(&bytes).expect("C4 reply");
                    assert!(matches!(reply, ForwardedResponse::Responded(_)));
                }
                Err(_) => assert!(!is_served, "pid {vmm_pid}"),
            }
        });
        assert_eq!(gateway.dropped_connections, u64::from(!is_served));
    }
    assert_eq!(upstream.calls(), 1);
    assert_eq!(kinds(&authority.seen()), ["admit", "settle"]);
}

#[test]
fn gate_refuses_locally_before_any_authority_call() {
    let directory = tempfile::tempdir().expect("scratch");
    let authority = Authority::start(directory.path(), "authority.sock", admitting("host-a", 0, MAX_REQUESTS));
    let mut gate = super::authority::KilnAuthorityGate::new(
        super::authority::UnixSocket {
            socket: authority.socket.clone(),
        },
        bound(),
    );
    for (checks, refusal) in [
        (
            crate::execution_binding::serving::gate::Checks {
                is_exact_model: true,
                is_within_budget: true,
                is_current: false,
            },
            ForwardedRefusal::NotCurrent,
        ),
        (
            crate::execution_binding::serving::gate::Checks {
                is_exact_model: true,
                is_within_budget: false,
                is_current: true,
            },
            ForwardedRefusal::Budget,
        ),
    ] {
        gate.prepare(0);
        assert!(crate::candidate_worker::broker::ModelRequestGate::admit(&mut gate, checks).is_none());
        assert_eq!(gate.refusal(), Some(refusal));
    }
    assert!(authority.seen().is_empty());
}

#[test]
fn published_authority_and_forwarded_fixtures_decode_exactly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/vm-profile");
    let read = |name: &str| std::fs::read(root.join(name)).unwrap_or_else(|error| panic!("{name}: {error}"));
    let round_trip = |name: &str, value: serde_json::Value| {
        let canonical: serde_json::Value = serde_json::from_slice(&read(name)).expect(name);
        assert_eq!(value, canonical, "{name}");
    };
    for name in ["admit.json", "settle.json", "settle-uncertain.json"] {
        let path = format!("effect-authority/{name}");
        let request: AuthorityRequest = serde_json::from_slice(&read(&path)).expect(&path);
        round_trip(&path, serde_json::to_value(request).expect("encode"));
    }
    let mut replies = vec!["reply-admitted.json".to_owned(), "reply-recorded.json".to_owned()];
    replies.extend(
        ["not_current", "deadline", "budget", "revoked", "unknown_binding"]
            .map(|code| format!("reply-refused-{code}.json")),
    );
    for name in replies {
        let path = format!("effect-authority/{name}");
        let reply: AuthorityReply = serde_json::from_slice(&read(&path)).expect(&path);
        round_trip(&path, serde_json::to_value(reply).expect("encode"));
    }
    for name in [
        "invalid-unknown-field",
        "invalid-unknown-code",
        "invalid-unknown-outcome",
        "invalid-missing-kind",
    ] {
        let bytes = read(&format!("effect-authority/{name}.json"));
        let is_request = serde_json::from_slice::<AuthorityRequest>(&bytes).is_ok();
        let is_reply = serde_json::from_slice::<AuthorityReply>(&bytes).is_ok();
        assert!(!is_request && !is_reply, "{name}");
    }
    let request: super::core::ForwardedRequest =
        serde_json::from_slice(&read("forwarded-upstream/request.json")).expect("request");
    assert_eq!(request.binding, binding_identity());
    for name in ["response-status", "response-refused", "response-uncertain"] {
        let path = format!("forwarded-upstream/{name}.json");
        let response: ForwardedResponse = serde_json::from_slice(&read(&path)).expect(&path);
        assert!(response.answers(0), "{name}");
        round_trip(&path, serde_json::to_value(response).expect("encode"));
    }
    let uncertain_false: ForwardedResponse =
        serde_json::from_slice(&read("forwarded-upstream/invalid-uncertain-false.json")).expect("decodes");
    assert!(!uncertain_false.answers(0), "`uncertain: false` is never a valid reply");
    assert!(serde_json::from_slice::<ForwardedResponse>(&read("forwarded-upstream/invalid-mixed.json")).is_err());
    assert!(
        serde_json::from_slice::<super::core::ForwardedRequest>(&read(
            "forwarded-upstream/invalid-request-unknown-field.json"
        ))
        .is_err()
    );
    let upstream: super::core::Upstream =
        serde_json::from_slice(&read("forwarded-upstream/upstream.json")).expect("upstream");
    super::admission::validate_upstream(&upstream).expect("admitted upstream");
    let file_provider: super::core::Upstream =
        serde_json::from_slice(&read("forwarded-upstream/refused-upstream-credential-provider.json")).expect("decodes");
    assert!(super::admission::validate_upstream(&file_provider).is_err());
}

/// A slow authority: the host clock passes the deadline while `Admit` is in
/// flight. The gateway rereads the clock after `Admit`, never calls the
/// upstream and settles the request `cancelled`.
// r[verify lattice.vm_profile.gateway.scenario.admit_latency]
#[test]
fn a_deadline_that_passes_during_a_slow_admit_settles_cancelled_without_upstream_contact() {
    let directory = tempfile::tempdir().expect("scratch");
    let now = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(NOW_MS));
    let elapsed = now.clone();
    let mut admit = admitting("host-a", 0, MAX_REQUESTS);
    let authority = Authority::start(
        directory.path(),
        "slow.sock",
        Box::new(move |request| {
            if matches!(request, AuthorityRequest::Admit { .. }) {
                // The admission took until the binding deadline.
                elapsed.store(DEADLINE_MS, std::sync::atomic::Ordering::SeqCst);
            }
            admit(request)
        }),
    );
    let upstream = Upstream::start(response_body());
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let clock = move || -> Result<u64, String> { Ok(now.load(std::sync::atomic::Ordering::SeqCst)) };
    let mut gateway = gateway(&authority, &upstream, &stopped, &clock, std::process::id());
    assert_eq!(refused(&gateway.decide(forwarded(0))), Some(ForwardedRefusal::Deadline));
    assert_eq!(upstream.calls(), 0, "nothing reaches the upstream after the deadline");
    let seen = authority.seen();
    assert_eq!(kinds(&seen), ["admit", "settle"]);
    assert!(matches!(seen[1], AuthorityRequest::Settle {
        outcome: super::core::SettleOutcome::Cancelled,
        usage: None,
        ..
    }));
}
