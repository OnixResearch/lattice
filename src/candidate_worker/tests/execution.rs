use super::*;

#[test]
fn worker_produces_host_derived_candidate_receipt() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("base source");

    let outcome = run(
        &request(root, "printf new > src/main.rs"),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    let Outcome::Produced { receipt, .. } = outcome else {
        panic!("expected produced candidate, got {outcome:?}");
    };
    assert_eq!(receipt.disposition, crate::candidate::CandidateDisposition::Produced);
    assert_eq!(receipt.changed_paths, vec!["src/main.rs"]);
    assert!(receipt.is_usable_candidate());
}

#[test]
fn exit_zero_without_change_is_not_a_candidate() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("source");

    let outcome = run(
        &request(root, "true"),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    assert!(matches!(&outcome, Outcome::Unchanged { .. }));
    let generic = AttemptResult::from_outcome(&outcome);
    assert_eq!(generic.disposition, crate::candidate::CandidateDisposition::Failed);
    assert!(generic.receipt.is_none());
}
#[test]
fn changes_outside_the_admitted_write_boundary_are_rejected() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("source");

    let outcome = run(
        &request(root, "printf changed > README.md"),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    assert!(matches!(
        outcome,
        Outcome::Failed { ref blocker, .. }
            if blocker == "worker changed a path outside its admitted write boundary"
    ));
}

#[test]
fn non_zero_exit_is_a_failure() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("source");

    let outcome = run(
        &request(root, "exit 3"),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Failed { .. }));
}

#[test]
fn output_overflow_and_timeout_are_failures() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("source");

    let mut overflow = request(root, "printf 1234567890");
    overflow.output_limit_bytes = 8;
    let outcome = run(
        &overflow,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");
    assert!(matches!(outcome, Outcome::Failed { .. }));

    let mut timed = request(root, "sleep 2");
    timed.timeout_secs = 1;
    let outcome =
        run(&timed, &crate::process::core::RuntimeConfig::default(), &std::sync::atomic::AtomicBool::new(false))
            .expect("worker runs");
    assert!(matches!(outcome, Outcome::Failed { .. }));
}

#[cfg(unix)]
#[test]
fn interrupted_candidate_capture_is_unknown() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("source");

    let outcome = run(
        &request(root, &format!("{} 000 src/main.rs", host_tool("chmod"))),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Unknown { .. }), "{outcome:?}");
    std::fs::set_permissions(root.join("src/main.rs"), std::fs::Permissions::from_mode(0o644)).expect("restore source");
}
#[test]
fn worker_environment_is_clear() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"same").expect("source");

    let outcome = run(
        &request(root, "printf %s \"$HOME\" > src/home"),
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Produced { .. }), "{outcome:?}");
    assert_eq!(std::fs::read(root.join("src/home")).expect("environment probe"), b"");
}
