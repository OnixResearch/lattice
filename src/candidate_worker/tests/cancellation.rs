use super::*;

#[test]
fn in_flight_cancellation_stops_the_worker_before_late_write() {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("source");

    let cancellation = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let signal = Arc::clone(&cancellation);
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        signal.store(true, Ordering::Release);
    });
    let outcome = run(
        &request(root, &format!("{} 1; printf leaked > src/leak", host_tool("sleep"))),
        &crate::process::core::RuntimeConfig::default(),
        cancellation.as_ref(),
    )
    .expect("worker runs");
    canceller.join().expect("canceller joins");

    assert!(matches!(outcome, Outcome::Cancelled { .. }), "{outcome:?}");
    assert!(!root.join("src/leak").exists());
}
#[test]
fn observed_cancellation_retains_bounded_partial_output_and_workspace() {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use std::time::Instant;
    let directory = tempfile::tempdir().expect("workspace");
    let root = directory.path();
    std::fs::create_dir(root.join("src")).expect("source directory");
    let sleeper = which::which("sleep").expect("sleep tool");
    let request = Request {
        work_id: "work".into(),
        attempt_id: "attempt".into(),
        fence: "1".into(),
        workspace_id: "workspace".into(),
        program: "sh".into(),
        args: vec![
            "-c".into(),
            format!("printf partial-event; printf candidate > src/marker.rs; '{}' 5", sleeper.display()),
        ],
        boundary: crate::candidate_capture::admit_workspace_boundary(
            root,
            &["src".into()],
            &["src".into()],
            false,
            false,
        )
        .expect("boundary"),
        require_isolation: false,
        sandbox_program: None,
        broker: None,
        timeout_secs: 10,
        output_limit_bytes: 4096,
    };
    let signal = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let canceller_signal = Arc::clone(&signal);
    let marker = root.join("src/marker.rs");
    let canceller = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        while std::fs::read(&marker).ok().as_deref() != Some(b"candidate".as_slice()) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(std::fs::read(&marker).expect("child marker must precede cancellation"), b"candidate");
        canceller_signal.store(true, Ordering::Release);
    });
    let outcome = run(&request, &crate::process::core::RuntimeConfig::default(), &signal).expect("worker");
    canceller.join().expect("observed marker");
    let Outcome::Cancelled { stdout, .. } = &outcome else {
        panic!("expected observed cancellation: {outcome:?}");
    };
    assert_eq!(stdout, b"partial-event");
    assert_eq!(std::fs::read(root.join("src/marker.rs")).expect("retained partial candidate"), b"candidate");
    let result = AttemptResult::from_outcome(&outcome);
    assert_eq!(result.disposition, crate::candidate::CandidateDisposition::Cancelled);
    assert_eq!(result.stdout_blake3, crate::pure_inet_profile::blake3_identity(b"partial-event"));
    assert!(result.receipt.is_none());
}
