//! Repair records forged or changed after a run, through the real run,
//! export and inspect shell: export refuses them and inventory never
//! reports an unproven or contradicted outcome as a settled worker.

// r[verify lattice.implementation_attempt.repair_export.scenario.refused]
// r[verify lattice.implementation_attempt.repair_export.scenario.inventory]

use super::Fixture;
use super::FixtureChecks;
use super::Outcome;
use super::StepReport;
use super::WorkerOutcome;
use super::export::Seen;
use super::export::export;
use super::export::inventory;
use super::export::is_empty;
use super::export::output_parent;
use super::export::refused;

type Reason = super::export::Reason;

/// Rewrite one retained JSON record in place.
fn rewrite(path: &std::path::Path, change: impl FnOnce(&mut serde_json::Value)) {
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).expect("record")).expect("record json");
    change(&mut value);
    std::fs::write(path, serde_json::to_vec(&value).expect("bytes")).expect("rewrite the record");
}

/// One fresh attempt run with `script`: the fixture, request path, attempt
/// directory and retained disposition.
async fn ran(
    attempt: &str,
    policy: crate::implementation_attempt::repair::Policy,
    script: Vec<super::Script<'_>>,
    checks: FixtureChecks,
) -> (Fixture, std::path::PathBuf, std::path::PathBuf, serde_json::Value) {
    let fixture = Fixture::new();
    let request = fixture.request(attempt, Some(policy), None);
    let (observation, _, _) = super::scenario(&fixture, &request, script, checks).await;
    let path = fixture.write(&request);
    let directory = fixture.directory(&request);
    (fixture, path, directory, observation)
}

fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("chmod");
}

#[tokio::test]
async fn a_failed_or_drifted_check_relabelled_as_a_pass_is_refused() {
    let lint = FixtureChecks {
        forced: Some(Outcome::Failed { class: "lint".into() }),
        ..FixtureChecks::admitted()
    };
    let rewriting = FixtureChecks {
        is_rewriting: true,
        ..FixtureChecks::admitted()
    };
    for (attempt, policy, checks, kind) in [
        ("attempt-forged-failed", super::policy(), lint, "not_repairable"),
        ("attempt-forged-drifted", super::formatted_policy(), rewriting, "checks_rejected"),
    ] {
        let script = vec![super::write("still broken")];
        let (fixture, path, directory, observation) = ran(attempt, policy, script, checks).await;
        assert_eq!(observation["disposition"]["kind"], kind);
        Seen::Terminal.assert(inventory(&path, &fixture.state), attempt);
        let revision = observation["revisions"][0]["revision"].clone();
        rewrite(&directory.join("repair/disposition.json"), |value| {
            value["disposition"] = serde_json::json!({"kind": "correct_candidate", "revision": revision});
        });
        rewrite(&directory.join("repair/step-1.json"), |value| {
            value["verdict"] = serde_json::json!({"kind": "passed"});
            for copy in value["copies"].as_array_mut().expect("copies") {
                copy["is_intact"] = true.into();
            }
        });
        let parent = output_parent(&fixture.root, "exports");
        assert_eq!(export(&path, &fixture.state, &parent), refused(Reason::RepairDispositionInvalid), "{attempt}");
        assert!(is_empty(&parent), "nothing is exported");
        Seen::Error.assert(inventory(&path, &fixture.state), attempt);
    }
}

#[tokio::test]
async fn execute_bits_are_exported_only_as_they_were_checked() {
    let executable: super::Script<'_> = Box::new(|_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), "fixed").expect("edit");
        set_mode(&workspace.join("src/lib.txt"), 0o755);
        super::exited()
    });
    for (attempt, script, checked, changed) in [
        ("attempt-modes-executable", vec![executable], 0o755, 0o644),
        ("attempt-modes-plain", vec![super::write("fixed")], 0o644, 0o755),
    ] {
        let (fixture, path, _, observation) = ran(attempt, super::policy(), script, FixtureChecks::admitted()).await;
        let expected = if checked & 0o111 == 0 {
            serde_json::json!({})
        } else {
            serde_json::json!({"src/lib.txt": 0o111})
        };
        assert_eq!(observation["revisions"][0]["executables"], expected, "{attempt}");
        let parent = output_parent(&fixture.root, "exports");
        let source = fixture.workspace.join("src/lib.txt");
        set_mode(&source, changed);
        let error = export(&path, &fixture.state, &parent).expect_err("a mode changed after the checks");
        assert!(error.contains("workspace file modes differ from the checked candidate revision"), "{error}");
        assert!(is_empty(&parent), "nothing is exported");
        set_mode(&source, checked);
        export(&path, &fixture.state, &parent).expect("the checked modes export");
        let exported = std::fs::metadata(parent.join(format!("{attempt}-7/src/lib.txt"))).expect("exported");
        let mode = std::os::unix::fs::PermissionsExt::mode(&exported.permissions());
        assert_eq!(mode & 0o111, checked & 0o111, "{attempt}");
    }
}

#[tokio::test]
async fn an_unknown_worker_or_an_unrecorded_step_stays_unknown() {
    let unknown: super::Script<'_> = Box::new(|_, _, _| StepReport {
        outcome: WorkerOutcome::Unknown,
        ..super::exited()
    });
    let (fixture, path, _, observation) =
        ran("attempt-unknown-worker", super::policy(), vec![unknown], FixtureChecks::admitted()).await;
    assert_eq!(observation["disposition"], serde_json::json!({"kind": "uncertain", "cause": "worker_outcome"}));
    Seen::Unknown.assert(inventory(&path, &fixture.state), "an unknown worker outcome");
    let parent = output_parent(&fixture.root, "exports");
    assert_eq!(export(&path, &fixture.state, &parent), refused(Reason::RepairNotCandidate));
    let script = vec![
        super::write("broken 1"),
        super::write("broken 2"),
        super::write("broken 3"),
    ];
    let (fixture, path, directory, _) =
        ran("attempt-unrecorded", super::policy(), script, FixtureChecks::admitted()).await;
    Seen::Terminal.assert(inventory(&path, &fixture.state), "every started worker exited");
    let intent = directory.join("repair/step-3.intent");
    let next = directory.join("repair/step-4.intent");
    std::fs::copy(&intent, &next).expect("an intent");
    rewrite(&next, |value| value["iteration"] = 4.into());
    Seen::Unknown.assert(inventory(&path, &fixture.state), "a step intent without a record");
}

#[tokio::test]
async fn a_receipt_claiming_other_changed_paths_is_refused() {
    let script = vec![super::write("fixed")];
    let (fixture, path, directory, observation) =
        ran("attempt-manifest", super::policy(), script, FixtureChecks::admitted()).await;
    let parent = output_parent(&fixture.root, "exports");
    let records = [
        directory.join("repair/disposition.json"),
        directory.join("repair/step-1.json"),
    ];
    let originals: Vec<Vec<u8>> = records.iter().map(|record| std::fs::read(record).expect("record")).collect();
    let source = observation["revisions"][0]["base_files"]["src/lib.txt"].clone();
    let digest = |bytes: &[u8]| serde_json::json!(crate::candidate::digests::FileDigest::of(bytes));
    let unchanged = serde_json::json!({ "checks/unit.conf": digest(super::UNIT_CONFIG) });
    let forged = serde_json::json!({ "checks/unit.conf": digest(b"another base"), "src/lib.txt": source });
    let cases = [
        ("a claimed path alone", serde_json::json!(["checks/unit.conf"]), None, true),
        ("a claimed unchanged path", serde_json::json!(["checks/unit.conf"]), Some(unchanged), false),
        ("a forged base digest", serde_json::json!(["checks/unit.conf", "src/lib.txt"]), Some(forged), false),
    ];
    for (name, changed_paths, base_files, is_typed) in cases {
        for ((record, original), pointer) in records.iter().zip(&originals).zip(["/revisions/0", "/candidate"]) {
            std::fs::write(record, original).expect("restore");
            rewrite(record, |value| {
                let revision = value.pointer_mut(pointer).expect("revision");
                revision["receipt"]["changed_paths"] = changed_paths.clone();
                if let Some(base_files) = &base_files {
                    revision["base_files"] = base_files.clone();
                }
            });
        }
        let exported = export(&path, &fixture.state, &parent);
        if is_typed {
            assert_eq!(exported, refused(Reason::RepairDispositionInvalid), "{name}");
        } else {
            let error = exported.expect_err(name);
            assert!(error.contains("workspace differs from the exact retained candidate revision"), "{name}: {error}");
        }
        assert!(is_empty(&parent), "{name}: nothing is exported");
    }
    for (record, original) in records.iter().zip(&originals) {
        std::fs::write(record, original).expect("restore");
    }
    export(&path, &fixture.state, &parent).expect("the retained receipt re-derives");
}

#[tokio::test]
async fn a_first_schema_disposition_is_printed_but_never_acted_on() {
    let script = vec![super::write("fixed")];
    let (fixture, path, directory, _) = ran("attempt-legacy", super::policy(), script, FixtureChecks::admitted()).await;
    rewrite(&directory.join("repair/disposition.json"), |value| {
        value["schema"] = crate::implementation_attempt::repair::retained::LEGACY_OBSERVATION_SCHEMA.into();
    });
    super::recover(&path, &fixture.state).await.expect("recovery prints the retained record");
    Seen::Unknown.assert(inventory(&path, &fixture.state), "a first-schema disposition");
    let parent = output_parent(&fixture.root, "exports");
    assert_eq!(export(&path, &fixture.state, &parent), refused(Reason::RepairDispositionInvalid));
}
