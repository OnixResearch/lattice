use super::*;

#[test]
fn invalid_limits_are_blocked_before_execution() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("source");

    let mut no_deadline = request(root, "printf new > src/main.rs");
    no_deadline.timeout_secs = 0;
    let outcome = run(
        &no_deadline,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");
    assert!(matches!(outcome, Outcome::Blocked { .. }));

    let mut no_output = request(root, "printf new > src/main.rs");
    no_output.output_limit_bytes = 0;
    let outcome = run(
        &no_output,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");
    assert!(matches!(outcome, Outcome::Blocked { .. }));
    assert_eq!(std::fs::read(root.join("src/main.rs")).expect("source remains"), b"old");
}

#[test]
fn credential_bearing_workspace_is_blocked_before_execution() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("source");

    let mut request = request(root, "printf new > src/main.rs");
    request.boundary = admit_workspace_boundary(root, &["src".to_string()], &["src".to_string()], false, true)
        .expect("credential boundary admits structurally");
    let outcome = run(
        &request,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Blocked { .. }), "{outcome:?}");
    assert_eq!(std::fs::read(root.join("src/main.rs")).expect("source remains"), b"old");
}

#[test]
fn required_isolation_without_provider_is_blocked() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("source");

    let mut request = request(root, "printf new > src/main.rs");
    request.require_isolation = true;
    let outcome = run(
        &request,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Blocked { .. }), "{outcome:?}");
    assert_eq!(std::fs::read(root.join("src/main.rs")).expect("source remains"), b"old");
}
