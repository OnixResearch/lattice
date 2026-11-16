#[test]
fn intent_is_one_shot_and_crossed_requests_are_refused() {
    let root = tempfile::tempdir().expect("scratch");
    let path = root.path().join("intent");
    assert!(super::artifacts::write_once(&path, b"exact").expect("first"));
    assert!(!super::artifacts::write_once(&path, b"exact").expect("replay"));
    assert!(super::artifacts::write_once(&path, b"other").is_err());
}
#[test]
fn unsafe_state_and_unbounded_requests_are_refused() {
    assert!(super::artifacts::private_directory(std::path::Path::new("relative")).is_err());
    let root = tempfile::tempdir().expect("scratch");
    let path = root.path().join("request");
    std::fs::write(&path, b"not-json").expect("write");
    assert!(super::artifacts::load_request(&path).is_err());
}

#[test]
fn recovery_requires_exact_bounded_raw_and_normalized_event_artifacts() {
    let root = tempfile::tempdir().expect("artifact directory");
    let raw = concat!(
        "{\"type\":\"session\",\"version\":3,\"id\":\"owned-session\",\"cwd\":\"/workspace\"}\n",
        "{\"type\":\"agent_start\"}\n",
        "{\"type\":\"message_end\",\"message\":{\"role\":\"assistant\",\"stopReason\":\"stop\"}}\n",
        "{\"type\":\"agent_end\",\"messages\":[{\"role\":\"assistant\",\"stopReason\":\"stop\"}]}\n",
    )
    .as_bytes();
    let digest = blake3::hash(raw).to_hex().to_string();
    let raw_path = root.path().join("raw-events.jsonl");
    let observation_path = root.path().join("omp-observation.json");
    assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    super::artifacts::write_once(&raw_path, raw).expect("durable raw events");
    assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    let observation = crate::candidate_worker::omp::normalize_jsonl(raw, 4096, "/workspace");
    assert!(observation.is_ok());
    let encoded = serde_json::to_vec(&observation).expect("encode normalized observation");
    super::artifacts::write_once(&observation_path, &encoded).expect("durable normalized observation");
    super::artifacts::verify_events(root.path(), 4096, &digest).expect("exact retained evidence");
    assert!(super::artifacts::verify_events(root.path(), raw.len() - 1, &digest).is_err());
    std::fs::write(&raw_path, b"substituted events\n").expect("substitute raw artifact");
    assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    std::fs::write(&raw_path, raw).expect("restore original raw artifact");
    std::fs::write(&observation_path, b"{\"Err\":\"forged completion\"}").expect("substitute normalized artifact");
    assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    #[cfg(unix)]
    {
        std::fs::remove_file(&raw_path).expect("remove raw artifact");
        std::os::unix::fs::symlink(&observation_path, &raw_path).expect("substitute artifact link");
        assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    }
}

#[test]
fn bounded_artifact_reads_accept_exact_bounds_and_reject_excess() {
    let root = tempfile::tempdir().expect("artifact directory");
    let path = root.path().join("bounded");
    std::fs::write(&path, b"exact").expect("write bounded artifact");
    assert_eq!(super::artifacts::read_bounded(&path, 5).expect("exact bound"), b"exact");
    assert!(super::artifacts::read_bounded(&path, 4).is_err());
    std::fs::write(&path, b"").expect("write empty artifact");
    assert_eq!(super::artifacts::read_bounded(&path, 0).expect("empty bound"), b"");
}

#[test]
fn concurrent_intent_writers_have_exactly_one_execution_owner() {
    let root = tempfile::tempdir().expect("state directory");
    let path = root.path().join("intent");
    let barrier = std::sync::Barrier::new(2);
    let outcomes = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            barrier.wait();
            super::artifacts::write_once(&path, b"exact")
        });
        let second = scope.spawn(|| {
            barrier.wait();
            super::artifacts::write_once(&path, b"exact")
        });
        [
            first.join().expect("first writer"),
            second.join().expect("second writer"),
        ]
    });
    assert_eq!(outcomes.iter().filter(|outcome| matches!(outcome, Ok(true))).count(), 1);
    assert_eq!(super::artifacts::read_bounded(&path, 5).expect("exact intent"), b"exact");
}

#[test]
fn namespace_paths_preserve_exact_admission() {
    assert!(super::validate_namespace_path(std::path::Path::new("/run/netns/provider-1_a.b")).is_ok());
    assert!(super::validate_namespace_path(std::path::Path::new("/run/netns")).is_err());
    assert!(super::validate_namespace_path(std::path::Path::new("/tmp/provider")).is_err());
    assert!(super::validate_namespace_path(std::path::Path::new("/run/netns/provider name")).is_err());
}
