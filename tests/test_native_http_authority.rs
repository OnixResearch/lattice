//! Standalone CLI proof uses the stock client and actual operator authority path.

use std::io::Read as _;
use std::io::Write as _;
use std::net::TcpListener;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use lattice::engine::effect::standalone;
use lattice::engine::effect::standalone::core::Profile;
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowTimestamp;

// r[verify lattice.native_http.authority]
// r[verify lattice.native_http.authority.scenario.allowed]
// r[verify lattice.native_http.authority.scenario.denied]
#[test]
fn actual_cli_authorized_http_reaches_server_and_missing_authority_does_not() {
    let directory = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/observed", listener.local_addr().unwrap());
    let mut workflow = Workflow::new("native-http-cli", WorkflowTimestamp::epoch());
    workflow.add_http_node("fetch", "GET", &url);
    let state = directory.path().join("state");
    std::fs::create_dir(&state).unwrap();
    let store = lattice::store::Store::new(state.join("lattice.redb")).unwrap();
    store.save_workflow(&workflow).unwrap();
    drop(store);
    let inspection = standalone::inspect_workflow(&workflow).unwrap();
    let profile = Profile {
        schema: inspection.schema,
        workflow_revision: inspection.workflow_revision,
        machine_identity: inspection.machine_identity,
        policy_ref: inspection.policy_ref,
        expires_at_unix_ms: 4_102_444_800_000,
        grants: inspection.grants,
        roots: Default::default(),
    };
    std::fs::write(directory.path().join("authority.json"), serde_json::to_vec(&profile).unwrap()).unwrap();
    let authority = directory.path().join("authority.ncl");
    std::fs::write(&authority, "import \"authority.json\"\n").unwrap();
    let config = directory.path().join("config.ncl");
    let base =
        format!("data_dir = {}, default_timeout_secs = 5, shell = \"sh\",", serde_json::to_string(&state).unwrap());
    std::fs::write(&config, format!("{{ {base} }}")).unwrap();
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_lattice"))
            .arg("--config")
            .arg(&config)
            .args(["run", "native-http-cli", "--output", "json"])
            .output()
            .unwrap()
    };
    let denied = invoke();
    assert!(!denied.status.success(), "missing authority cannot succeed");
    assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
    std::fs::write(&config, format!("{{ {base} effect_authority = {}, }}", serde_json::to_string(&authority).unwrap()))
        .unwrap();
    std::thread::scope(|scope| {
        let server = scope.spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(15);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    result => panic!("authorized CLI did not reach bounded server: {result:?}"),
                }
            };
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut bytes = Vec::new();
            let mut byte = [0];
            while !bytes.ends_with(b"\r\n\r\n") {
                stream.read_exact(&mut byte).unwrap();
                bytes.push(byte[0]);
                assert!(bytes.len() <= lattice::http::MAX_HEADER_BYTES);
            }
            assert!(bytes.starts_with(b"GET /observed HTTP/1.1\r\n"));
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nobserved")
                .unwrap();
            drop(stream);
            assert_eq!(listener.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
        });
        let allowed = invoke();
        assert!(
            allowed.status.success(),
            "authorized CLI: stdout={} stderr={}",
            String::from_utf8_lossy(&allowed.stdout),
            String::from_utf8_lossy(&allowed.stderr)
        );
        let summary: serde_json::Value = serde_json::from_slice(&allowed.stdout).unwrap();
        assert_eq!(summary["status"], "succeeded");
        server.join().unwrap();
    });
}
