use super::*;

#[test]
#[ignore = "requires an operator-admitted pinned OMP build, bubblewrap and authorized model endpoint"]
fn omp_worker_produces_a_sandboxed_candidate() {
    let sandbox = std::env::var("LATTICE_BWRAP").expect("reviewed bubblewrap path");
    let omp = std::env::var("LATTICE_OMP").expect("pinned OMP path");
    let base_url = std::env::var("LATTICE_OMP_BASE_URL").expect("authorized qualification endpoint");
    let model = std::env::var("LATTICE_OMP_MODEL").expect("exact qualification model");
    let executable_blake3 = std::env::var("LATTICE_OMP_BLAKE3").expect("reviewed executable bytes");
    let revision = std::env::var("LATTICE_OMP_REVISION").expect("reviewed executable source revision");
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join(".omp/agent")).expect("OMP config directory");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"pub fn answer() -> i32 { 41 }\n").expect("base source");
    std::fs::write(
        root.join(".omp/agent/models.yml"),
        serde_json::json!({
            "providers": {
                "qualification": {
                    "api": "openai-completions",
                    "apiKey": "local",
                    "baseUrl": base_url,
                    "models": [{
                        "id": model,
                        "name": "Lattice qualification",
                        "api": "openai-completions"
                    }]
                }
            }
        })
        .to_string(),
    )
    .expect("OMP model profile");
    let boundary =
        admit_workspace_boundary(root, &[".omp".to_string(), "src".to_string()], &["src".to_string()], true, false)
            .expect("workspace boundary");
    let isolated = Request {
        work_id: "work-omp".to_string(),
        attempt_id: "attempt-omp".to_string(),
        fence: "9".to_string(),
        workspace_id: "workspace-omp".to_string(),
        program: omp,
        args: Vec::new(),
        boundary,
        require_isolation: true,
        sandbox_program: Some(sandbox),
        timeout_secs: 120,
        output_limit_bytes: 1024 * 1024,
    };

    let profile = omp::Profile {
        revision,
        executable_blake3,
        base_identity: crate::candidate::snapshot_identity(
            &crate::candidate_capture::workspace(root).expect("base capture"),
        )
        .expect("base identity"),
        model: format!("qualification/{model}"),
        prompt: "Use the edit tool to replace 41 with 42 in src/main.rs. Do not run commands. Reply done.".to_string(),
    };
    let observed = omp::run(
        &isolated,
        &profile,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("selected OMP route");
    assert!(!observed.raw_events().is_empty());
    assert_eq!(observed.observation.expect("complete JSONL").terminal, omp::Terminal::Completed);
    let outcome = observed.outcome;

    let result = AttemptResult::from_outcome(&outcome).with_compatibility(&isolated);
    result.validate().expect("compatibility validates");
    assert!(result.compatibility.is_some());
    let Outcome::Produced { receipt, .. } = outcome else {
        panic!("expected OMP candidate, got {outcome:?}");
    };
    assert_eq!(receipt.changed_paths, vec!["src/main.rs"]);
    assert_eq!(std::fs::read(root.join("src/main.rs")).expect("candidate"), b"pub fn answer() -> i32 { 42 }\n");
}

#[test]
#[ignore = "requires a reviewed bubblewrap executable and working user namespaces"]
fn bubblewrap_isolation_confines_workspace_and_hides_host_paths() {
    let sandbox = std::env::var("LATTICE_BWRAP").expect("reviewed bubblewrap path");
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("base source");
    let shared = tempfile::tempdir().expect("unrelated repository and secrets");
    std::fs::write(shared.path().join("refs"), b"unrelated ref").expect("shared ref");
    std::fs::write(shared.path().join("publisher-token"), b"host-secret").expect("host secret");
    std::fs::write(root.join(".git"), format!("gitdir: {}", shared.path().display())).expect("linked metadata pointer");
    let mut isolated = request(
        root,
        &format!(
            "if [ -e /etc/passwd ] || [ -e /workspace/.git ] || [ -e '{}' ]; then exit 7; fi; printf new > src/main.rs",
            shared.path().join("publisher-token").display(),
        ),
    );
    isolated.require_isolation = true;
    isolated.sandbox_program = Some(sandbox);

    let outcome = run(
        &isolated,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    let result = AttemptResult::from_outcome(&outcome).with_compatibility(&isolated);
    result.validate().expect("compatibility validates");
    assert!(result.compatibility.is_some());
    let Outcome::Produced { receipt, .. } = outcome else {
        panic!("expected isolated candidate, got {outcome:?}");
    };
    assert_eq!(receipt.changed_paths, vec!["src/main.rs"]);
    assert_eq!(std::fs::read(root.join("src/main.rs")).expect("candidate"), b"new");
    assert_eq!(std::fs::read(shared.path().join("refs")).expect("unrelated ref retained"), b"unrelated ref");
    assert_eq!(std::fs::read(shared.path().join("publisher-token")).expect("host secret retained"), b"host-secret");
}

#[test]
#[ignore = "requires a reviewed bubblewrap executable and working user namespaces"]
fn bubblewrap_isolation_propagates_cancellation() {
    use std::sync::atomic::Ordering;

    let sandbox = std::env::var("LATTICE_BWRAP").expect("reviewed bubblewrap path");
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("base source");
    let mut isolated = request(root, "sleep 1; printf leaked > src/leak");
    isolated.require_isolation = true;
    isolated.sandbox_program = Some(sandbox);
    let signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let canceller = {
        let signal = std::sync::Arc::clone(&signal);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            signal.store(true, Ordering::Release);
        })
    };

    let outcome =
        run(&isolated, &crate::process::core::RuntimeConfig::default(), signal.as_ref()).expect("worker runs");
    canceller.join().expect("canceller joins");

    assert!(matches!(outcome, Outcome::Cancelled { .. }), "{outcome:?}");
    std::thread::sleep(std::time::Duration::from_millis(1_200));
    assert!(!root.join("src/leak").exists());
}

#[test]
#[ignore = "requires a reviewed bubblewrap executable and working user namespaces"]
fn bubblewrap_isolation_rejects_writes_outside_the_workspace() {
    let sandbox = std::env::var("LATTICE_BWRAP").expect("reviewed bubblewrap path");
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"old").expect("base source");
    let mut isolated = request(root, "printf nope > /workspace/not-admitted && printf new > src/main.rs");
    isolated.require_isolation = true;
    isolated.sandbox_program = Some(sandbox);

    let outcome = run(
        &isolated,
        &crate::process::core::RuntimeConfig::default(),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("worker runs");

    assert!(matches!(outcome, Outcome::Failed { .. }), "{outcome:?}");
    assert_eq!(std::fs::read(root.join("src/main.rs")).expect("source remains"), b"old");
    assert!(!root.join("not-admitted").exists());
}
