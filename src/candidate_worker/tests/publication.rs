use super::*;

#[test]
fn produced_candidate_is_published_to_store() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path().join("workspace");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("base source");
    let store = crate::store::Store::new(directory.path().join("store.redb")).expect("store opens");

    let outcome = run_and_publish(
        &request(&root, "printf new > src/main.rs"),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
        &store,
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Produced { .. }), "{outcome:?}");
    let receipt = store.load_candidate_receipt("attempt-1").expect("receipt loads").expect("receipt exists");
    assert_eq!(receipt.changed_paths, vec!["src/main.rs"]);
    assert_eq!(
        store
            .load_candidate_artifact(crate::store::ArtifactKey {
                attempt_id: "attempt-1",
                relative_path: "src/main.rs"
            })
            .expect("artifact loads"),
        Some(b"new".to_vec())
    );
    assert_eq!(store.list_candidate_artifacts("attempt-1").expect("artifacts list").len(), 1);
    let result = store.load_candidate_attempt_result("attempt-1").expect("result loads").expect("result exists");
    assert_eq!(result.disposition, crate::candidate::CandidateDisposition::Produced);
    assert!(matches!(reconcile_attempt(&store, "attempt-1"), Ok(Recovery::Published(_))));
}

#[test]
fn failed_worker_outcome_is_persisted_without_a_candidate() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path().join("workspace");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("base source");
    let store = crate::store::Store::new(directory.path().join("store.redb")).expect("store opens");

    let outcome = run_and_publish(
        &request(&root, "exit 3"),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
        &store,
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Failed { .. }), "{outcome:?}");
    let result = store.load_candidate_attempt_result("attempt-1").expect("result loads").expect("result exists");
    assert_eq!(result.disposition, crate::candidate::CandidateDisposition::Failed);
    assert!(result.blocker.is_some());
    assert!(result.receipt.is_none());
    assert!(result.published_artifacts.is_empty());
    assert!(store.load_candidate_receipt("attempt-1").expect("receipt lookup").is_none());
}

#[test]
fn blocked_worker_outcome_is_persisted_without_a_candidate() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path().join("workspace");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("base source");
    let store = crate::store::Store::new(directory.path().join("store.redb")).expect("store opens");
    let mut request = request(&root, "printf new > src/main.rs");
    request.boundary = admit_workspace_boundary(&root, &["src".to_string()], &["src".to_string()], false, true)
        .expect("credential boundary admits structurally");

    let outcome = run_and_publish(
        &request,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
        &store,
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Blocked { .. }), "{outcome:?}");
    let result = store.load_candidate_attempt_result("attempt-1").expect("result loads").expect("result exists");
    assert_eq!(result.disposition, crate::candidate::CandidateDisposition::Blocked);
    assert!(result.blocker.is_some());
    assert!(result.receipt.is_none());
    assert!(store.load_candidate_receipt("attempt-1").expect("receipt lookup").is_none());
}

#[cfg(unix)]
#[test]
fn unknown_worker_outcome_is_persisted_without_a_candidate() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path().join("workspace");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("base source");
    let store = crate::store::Store::new(directory.path().join("store.redb")).expect("store opens");

    let outcome = run_and_publish(
        &request(&root, &format!("{} 000 src/main.rs", host_tool("chmod"))),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
        &store,
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Unknown { .. }), "{outcome:?}");
    let result = store.load_candidate_attempt_result("attempt-1").expect("result loads").expect("result exists");
    assert_eq!(result.disposition, crate::candidate::CandidateDisposition::Unknown);
    assert!(result.blocker.is_some());
    assert!(result.receipt.is_none());
    assert!(store.load_candidate_receipt("attempt-1").expect("receipt lookup").is_none());
    std::fs::set_permissions(root.join("src/main.rs"), std::fs::Permissions::from_mode(0o644)).expect("restore source");
}

#[test]
fn cancelled_worker_outcome_is_persisted_without_a_candidate() {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path().join("workspace");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("base source");
    let store = crate::store::Store::new(directory.path().join("store.redb")).expect("store opens");
    let cancellation = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let signal = Arc::clone(&cancellation);
    let canceller = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        signal.store(true, Ordering::Release);
    });

    let outcome = run_and_publish(
        &request(&root, &format!("{} 1; printf leaked > src/leak", host_tool("sleep"))),
        &crate::process::core::RuntimeConfig::default(),
        cancellation.as_ref(),
        &store,
    )
    .expect("worker runs");
    canceller.join().expect("canceller joins");

    assert!(matches!(outcome, Outcome::Cancelled { .. }), "{outcome:?}");
    let result = store.load_candidate_attempt_result("attempt-1").expect("result loads").expect("result exists");
    assert_eq!(result.disposition, crate::candidate::CandidateDisposition::Cancelled);
    assert!(result.receipt.is_none());
    assert!(store.load_candidate_receipt("attempt-1").expect("receipt lookup").is_none());
    assert!(!root.join("src/leak").exists());
}

#[test]
fn reconciliation_reports_missing_and_incomplete_publication() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let store = crate::store::Store::new(directory.path().join("reconcile.redb")).expect("store opens");
    assert!(matches!(reconcile_attempt(&store, "attempt-missing"), Ok(Recovery::Missing)));

    let usage = crate::orchestration::CommandWorkerObservation {
        schema: crate::orchestration::USAGE_OBSERVATION_SCHEMA.to_string(),
        scope: crate::orchestration::Scope::Run,
        workflow_revision: "revision-1".to_string(),
        instance_id: "instance-1".to_string(),
        dispatch_id: "dispatch-1".to_string(),
        run_id: 7,
        node_id: 0,
        invocation_id: "invocation-1".to_string(),
        worker_profile: "profile-1".to_string(),
        attempt_id: "attempt-usage-only".to_string(),
        agent_invocations: Some(1),
        model_requests: None,
        subagent_calls: None,
        tokens: None,
        cost_microunits: None,
        duration_millis: Some(1),
        provenance: crate::orchestration::UsageProvenance::HostMeasured,
        complete: true,
        accounting: None,
        execution_binding: None,
    };
    store.save_command_worker_usage(&usage).expect("usage saves");
    assert!(matches!(reconcile_attempt(&store, "attempt-usage-only"), Ok(Recovery::Incomplete { .. })));

    let receipt = crate::candidate::CandidateReceipt {
        schema: crate::candidate::CANDIDATE_RECEIPT_SCHEMA.to_string(),
        work_id: "work-1".to_string(),
        attempt_id: "attempt-incomplete".to_string(),
        fence: "7".to_string(),
        workspace_id: "workspace-1".to_string(),
        base_identity: "a".repeat(64),
        candidate_identity: Some("b".repeat(64)),
        diff_identity: Some("c".repeat(64)),
        changed_paths: vec!["src/main.rs".to_string()],
        disposition: crate::candidate::CandidateDisposition::Produced,
        blockers: Vec::new(),
    };
    let result = AttemptResult {
        schema: CANDIDATE_ATTEMPT_RESULT_SCHEMA.to_string(),
        disposition: crate::candidate::CandidateDisposition::Produced,
        receipt: Some(receipt),
        published_artifacts: Vec::new(),
        stdout_blake3: crate::pure_inet_profile::blake3_identity(b""),
        stderr_blake3: crate::pure_inet_profile::blake3_identity(b""),
        blocker: None,
        compatibility: None,
        process_failure: None,
        observed_artifacts: std::collections::BTreeMap::new(),
    };
    store.save_candidate_attempt_result("attempt-incomplete", &result).expect("result saves");
    assert!(matches!(reconcile_attempt(&store, "attempt-incomplete"), Ok(Recovery::Incomplete { .. })));
}
