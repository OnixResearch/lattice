//! Q1: the real `lattice model-gateway serve` process over a Unix socket
//! standing in for Cloud Hypervisor's guest-initiated vsock socket, with a
//! scripted C3 authority and a scripted OpenAI-compatible upstream in this
//! test process. No VM, no vsock, no live model.
#![cfg(target_os = "linux")]

// r[verify lattice.vm_profile.gateway]
// r[verify lattice.vm_profile.gateway.scenario.peer]
// r[verify lattice.vm_profile.gateway.scenario.deadline]

use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

use lattice::model_gateway::core::AuthorityReply;
use lattice::model_gateway::core::AuthorityRequest;
use lattice::model_gateway::core::ForwardedRefusal;
use lattice::model_gateway::core::ForwardedResponse;

const CANARY: &str = "host-canary-credential";
const TOKEN_VARIABLE: &str = "LATTICE_TEST_MODEL_GATEWAY_TOKEN";
const MODEL: &str = "admitted-model";

fn fixtures() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/vm-profile")
}

fn now_ms() -> u64 {
    u64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("clock").as_millis())
        .expect("epoch millis")
}

struct Threads {
    stop: Arc<AtomicBool>,
    handles: Vec<std::thread::JoinHandle<()>>,
}

impl Drop for Threads {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }
}

/// Scripted C3 authority: admits up to four requests and records settlements.
fn authority(
    socket: &std::path::Path,
    deadline_unix_ms: u64,
    threads: &mut Threads,
) -> Arc<Mutex<Vec<AuthorityRequest>>> {
    let listener = std::os::unix::net::UnixListener::bind(socket).expect("authority socket");
    listener.set_nonblocking(true).expect("bounded accept");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let (recorded, stop) = (seen.clone(), threads.stop.clone());
    threads.handles.push(std::thread::spawn(move || {
        let mut consumed_requests = 0_u32;
        while !stop.load(Ordering::Acquire) {
            let Ok((mut stream, _)) = listener.accept() else {
                std::thread::sleep(Duration::from_millis(2));
                continue;
            };
            stream.set_nonblocking(false).expect("blocking");
            let frame = lattice_workflow_transport::read_frame(&mut stream, 65_536).expect("C3 frame");
            let request: AuthorityRequest = serde_json::from_slice(&frame).expect("C3 request");
            recorded.lock().expect("seen").push(request.clone());
            let reply = match request {
                AuthorityRequest::Admit { .. } => {
                    consumed_requests += 1;
                    AuthorityReply::Admitted {
                        placement_host: "host-a".into(),
                        placement_generation: 1,
                        consumed_requests,
                        remaining_requests: 4 - consumed_requests,
                        deadline_unix_ms,
                    }
                }
                AuthorityRequest::Settle { .. } => AuthorityReply::Recorded { consumed_requests },
            };
            let frame = lattice_workflow_transport::encode_frame(&reply, 65_536).expect("C3 reply");
            lattice_workflow_transport::write_frame(&mut stream, &frame).expect("reply");
        }
    }));
    seen
}

/// Scripted OpenAI-compatible upstream; returns its endpoint, call counter and headers.
fn upstream(threads: &mut Threads) -> (String, Arc<AtomicUsize>, Arc<Mutex<String>>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("upstream");
    let endpoint = format!("http://{}/v1/chat/completions", listener.local_addr().expect("address"));
    listener.set_nonblocking(true).expect("bounded accept");
    let calls = Arc::new(AtomicUsize::new(0));
    let headers = Arc::new(Mutex::new(String::new()));
    let (counted, recorded, stop) = (calls.clone(), headers.clone(), threads.stop.clone());
    threads.handles.push(std::thread::spawn(move || {
        while !stop.load(Ordering::Acquire) {
            let Ok((mut stream, _)) = listener.accept() else {
                std::thread::sleep(Duration::from_millis(2));
                continue;
            };
            stream.set_nonblocking(false).expect("blocking");
            let mut reader = std::io::BufReader::new(&mut stream);
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).expect("header");
                if line == "\r\n" || line.is_empty() {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse::<usize>().expect("length");
                }
                recorded.lock().expect("headers").push_str(&line);
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).expect("body");
            counted.fetch_add(1, Ordering::SeqCst);
            let body = serde_json::json!({"id": "resp-1", "model": MODEL, "choices": []}).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).expect("response");
        }
    }));
    (endpoint, calls, headers)
}

struct Gateway {
    child: std::process::Child,
    socket: std::path::PathBuf,
    binding: String,
}

impl Drop for Gateway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Write the profile, a binding with `deadline_unix_ms` and the upstream,
/// then start the real gateway binary.
fn command(
    directory: &std::path::Path,
    endpoint: &str,
    deadline_unix_ms: u64,
    vmm_pid: u32,
) -> (std::process::Command, std::path::PathBuf, String) {
    let profile_path = fixtures().join("execution-profile/admitted.json");
    let mut binding: lattice::vm_profile::AttemptBinding =
        serde_json::from_slice(&std::fs::read(fixtures().join("attempt-binding/admitted.json")).expect("binding"))
            .expect("binding JSON");
    binding.deadline_unix_ms = deadline_unix_ms;
    let identity = lattice::vm_profile::binding_identity(&binding).expect("binding identity");
    let binding_path = directory.join("binding.json");
    std::fs::write(&binding_path, serde_json::to_vec(&binding).expect("binding bytes")).expect("binding file");
    let upstream_path = directory.join("upstream.json");
    let upstream = serde_json::json!({
        "schema": "lattice.model-gateway-upstream.v1",
        "endpoint": endpoint,
        "model": MODEL,
        "credential": {"provider": "env", "key": TOKEN_VARIABLE, "field": null},
        "max_requests": 4,
    });
    std::fs::write(&upstream_path, upstream.to_string()).expect("upstream file");
    let socket = directory.join("vsock.sock_10243");
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_lattice"));
    command
        .args(["model-gateway", "serve", "--profile"])
        .arg(profile_path)
        .arg("--binding")
        .arg(&binding_path)
        .arg("--upstream")
        .arg(&upstream_path)
        .arg("--listen-uds")
        .arg(&socket)
        .arg("--vmm-pid")
        .arg(vmm_pid.to_string())
        .arg("--authority-socket")
        .arg(directory.join("authority.sock"))
        .env_clear()
        .env(TOKEN_VARIABLE, CANARY)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    (command, socket, identity)
}

fn start(directory: &std::path::Path, endpoint: &str, deadline_unix_ms: u64, vmm_pid: u32) -> Gateway {
    let (mut command, socket, binding) = command(directory, endpoint, deadline_unix_ms, vmm_pid);
    let mut child = command.spawn().expect("gateway process");
    let mut ready = String::new();
    std::io::BufReader::new(child.stdout.take().expect("stdout"))
        .read_line(&mut ready)
        .expect("readiness");
    let ready: serde_json::Value = serde_json::from_str(&ready).expect("readiness JSON");
    assert_eq!(ready["schema"], "lattice.model-gateway-serve.v1");
    assert_eq!(ready["state"], "listening");
    assert_eq!(ready["binding"], binding.as_str());
    Gateway { child, socket, binding }
}

fn forward(gateway: &Gateway, request_index: u32) -> Option<ForwardedResponse> {
    let mut client = std::os::unix::net::UnixStream::connect(&gateway.socket).expect("connect");
    client.set_read_timeout(Some(Duration::from_secs(10))).expect("bounded read");
    let request = lattice::model_gateway::core::ForwardedRequest {
        binding: gateway.binding.clone(),
        request_index,
        body: serde_json::json!({"model": MODEL, "messages": []}).to_string(),
    };
    let frame = lattice_workflow_transport::encode_frame(&request, 1 << 20).expect("frame");
    let _ = lattice_workflow_transport::write_frame(&mut client, &frame);
    let reply = lattice_workflow_transport::read_frame(&mut client, 1 << 24).ok()?;
    Some(serde_json::from_slice(&reply).expect("C4 reply"))
}

fn threads() -> Threads {
    Threads {
        stop: Arc::new(AtomicBool::new(false)),
        handles: Vec::new(),
    }
}

#[test]
fn gateway_process_admits_calls_once_with_the_host_credential_and_settles() {
    let directory = tempfile::tempdir().expect("scratch");
    let deadline_unix_ms = now_ms() + 120_000;
    let mut threads = threads();
    let seen = authority(&directory.path().join("authority.sock"), deadline_unix_ms, &mut threads);
    let (endpoint, calls, headers) = upstream(&mut threads);
    let gateway = start(directory.path(), &endpoint, deadline_unix_ms, std::process::id());
    let Some(ForwardedResponse::Responded(responded)) = forward(&gateway, 0) else {
        panic!("the VMM peer is served");
    };
    assert_eq!(responded.status, 200);
    assert!(!responded.body.contains(CANARY));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(headers.lock().expect("headers").contains(&format!("Bearer {CANARY}")));
    // A replayed index is refused by the process without a second upstream call.
    assert!(matches!(
        forward(&gateway, 0),
        Some(ForwardedResponse::Refused(lattice::model_gateway::core::Refused {
            refused: ForwardedRefusal::NotCurrent,
            ..
        }))
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let kinds: Vec<bool> = seen
        .lock()
        .expect("seen")
        .iter()
        .map(|request| matches!(request, AuthorityRequest::Admit { .. }))
        .collect();
    assert_eq!(kinds, [true, false], "admit before the upstream call, settle after it");
}

#[test]
fn gateway_process_drops_a_peer_that_is_not_the_vmm() {
    let directory = tempfile::tempdir().expect("scratch");
    let deadline_unix_ms = now_ms() + 120_000;
    let mut threads = threads();
    let seen = authority(&directory.path().join("authority.sock"), deadline_unix_ms, &mut threads);
    let (endpoint, calls, _) = upstream(&mut threads);
    let gateway = start(directory.path(), &endpoint, deadline_unix_ms, 1);
    assert!(forward(&gateway, 0).is_none(), "a non-VMM peer gets no reply");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(seen.lock().expect("seen").is_empty());
}

#[test]
fn gateway_process_enforces_the_binding_deadline_on_the_host_clock() {
    let directory = tempfile::tempdir().expect("scratch");
    let mut threads = threads();
    let (endpoint, calls, _) = upstream(&mut threads);
    // Refused at start once the host clock has reached the deadline.
    let (mut late, _, _) = command(directory.path(), &endpoint, now_ms() - 1, std::process::id());
    let output = late.output().expect("late gateway");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("deadline"), "{output:?}");
    // A deadline that passes while serving (for example across a guest pause)
    // refuses without any authority or upstream call.
    let deadline_unix_ms = now_ms() + 1_500;
    let seen = authority(&directory.path().join("authority.sock"), deadline_unix_ms, &mut threads);
    let gateway = start(directory.path(), &endpoint, deadline_unix_ms, std::process::id());
    std::thread::sleep(Duration::from_millis(2_000));
    assert!(matches!(
        forward(&gateway, 0),
        Some(ForwardedResponse::Refused(lattice::model_gateway::core::Refused {
            refused: ForwardedRefusal::Deadline,
            ..
        }))
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(seen.lock().expect("seen").is_empty());
}

#[test]
fn gateway_process_refuses_a_crossed_binding_before_listening() {
    let directory = tempfile::tempdir().expect("scratch");
    let (mut command, socket, _) =
        command(directory.path(), "http://127.0.0.1:1/v1/chat/completions", now_ms() + 60_000, 1);
    let mut binding: serde_json::Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("binding.json")).expect("binding")).expect("JSON");
    binding["profile"] = serde_json::Value::from(format!("b3:{}", "6".repeat(64)));
    std::fs::write(directory.path().join("binding.json"), binding.to_string()).expect("crossed binding");
    let output = command.output().expect("gateway");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("crossed_profile"), "{output:?}");
    assert!(!socket.exists(), "nothing listens for a crossed binding");
}
