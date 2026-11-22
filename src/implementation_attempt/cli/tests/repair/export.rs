//! `export-candidate` and inventory v3 over retained repair dispositions,
//! driven through the real run, export and inspect shell.

// r[verify lattice.implementation_attempt.repair_export]
// r[verify lattice.implementation_attempt.repair_export.scenario.passing]
// r[verify lattice.implementation_attempt.repair_export.scenario.refused]
// r[verify lattice.implementation_attempt.repair_export.scenario.inventory]

use super::super::super::session::revision::Store;
use super::Fixture;
use super::FixtureChecks;
use super::FixtureWorker;
use super::StepReport;
use super::WorkerOutcome;

pub(super) type Reason = super::super::super::refusal::Reason;

pub(super) fn export(path: &std::path::Path, state: &std::path::Path, parent: &std::path::Path) -> Result<(), String> {
    super::super::super::export::run(path, state, parent, Store(None))
}

pub(super) fn refused(reason: Reason) -> Result<(), String> {
    Err(reason.message().to_owned())
}

pub(super) fn output_parent(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    let parent = root.join(name);
    std::fs::create_dir(&parent).expect("output parent");
    parent
}

pub(super) fn is_empty(directory: &std::path::Path) -> bool {
    std::fs::read_dir(directory).expect("output parent").next().is_none()
}

pub(super) fn inventory(path: &std::path::Path, state: &std::path::Path) -> Result<serde_json::Value, String> {
    super::super::super::inspect::project(
        path,
        state,
        &mut crate::execution_binding::serving::UnavailableOwner,
        Store(None),
    )
}

/// What inventory shows for an attempt.
#[derive(Debug, Clone, Copy)]
pub(super) enum Seen {
    /// `dispatch_recorded_outcome_unknown` with `dispatch_outcome_unknown`.
    Unknown,
    /// An error, never a guessed state.
    Error,
    /// `terminal_observed` with `exited` liveness and no wait reasons.
    Terminal,
}

impl Seen {
    pub(super) fn assert(self, inspected: Result<serde_json::Value, String>, name: &str) {
        match self {
            Self::Error => assert!(inspected.is_err(), "{name}: {inspected:?}"),
            Self::Unknown => {
                let instance = &inspected.expect("inventory")["instance"];
                assert_eq!(instance["lifecycle"]["state"], "dispatch_recorded_outcome_unknown", "{name}");
                assert_eq!(instance["liveness"]["state"], "unknown", "{name}");
                assert!(instance["wait_reasons"].to_string().contains("dispatch_outcome_unknown"), "{name}");
            }
            Self::Terminal => {
                let instance = &inspected.expect("inventory")["instance"];
                assert_eq!(instance["lifecycle"]["state"], "terminal_observed", "{name}");
                assert_eq!(instance["liveness"]["state"], "exited", "{name}");
                assert_eq!(instance["wait_reasons"], serde_json::json!([]), "{name}");
            }
        }
    }
}

fn json(path: &std::path::Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(path).expect("retained file")).expect("json")
}

/// The exported candidate tree without its two evidence files.
fn exported_identity(root: &std::path::Path) -> String {
    let mut snapshot = crate::candidate_capture::workspace(root).expect("capture the export");
    assert!(snapshot.remove("host-receipt.json").is_some() && snapshot.remove("native-observation.json").is_some());
    crate::candidate::snapshot_identity(&snapshot).expect("identity")
}

#[tokio::test]
async fn the_exact_correct_and_repaired_candidate_revisions_are_exported() {
    for (attempt, texts, kind) in [
        ("attempt-export-correct", vec!["fixed"], "correct_candidate"),
        ("attempt-export-repaired", vec!["still broken", "fixed"], "repaired_candidate"),
    ] {
        let fixture = Fixture::new();
        let request = fixture.request(attempt, Some(super::policy()), None);
        let script = texts.into_iter().map(super::write).collect();
        let (observation, _, _) = super::scenario(&fixture, &request, script, FixtureChecks::admitted()).await;
        assert_eq!(observation["disposition"]["kind"], kind);
        let path = fixture.write(&request);
        let parent = output_parent(&fixture.root, "exports");
        export(&path, &fixture.state, &parent).expect("the passing revision is exported");
        let root = parent.join(format!("{attempt}-7"));
        let revisions = observation["revisions"].as_array().expect("revisions");
        let last = revisions.last().expect("the final revision");
        assert_eq!(observation["disposition"]["revision"], last["revision"]);
        assert_eq!(json(&root.join("host-receipt.json")), last["receipt"], "the exact final revision's receipt");
        assert_eq!(json(&root.join("native-observation.json")), observation, "the retained disposition");
        assert_eq!(exported_identity(&root), last["receipt"]["candidate_identity"].as_str().expect("identity"));
        assert_eq!(std::fs::read(root.join("src/lib.txt")).expect("source"), b"fixed");
        if revisions.len() > 1 {
            assert_ne!(revisions[0]["receipt"], last["receipt"], "never an earlier revision");
        }
        export(&path, &fixture.state, &parent).expect("an exact replay");
    }
}

#[tokio::test]
async fn a_non_passing_disposition_is_refused_and_inventory_reports_it() {
    let cancelled: super::Script<'_> = Box::new(|_, workspace, _| {
        std::fs::write(workspace.join("src/lib.txt"), "half done").expect("edit");
        StepReport {
            outcome: WorkerOutcome::Cancelled,
            ..super::exited()
        }
    });
    let exhausted = vec![
        super::write("broken 1"),
        super::write("broken 2"),
        super::write("broken 3"),
    ];
    for (attempt, script, kind, cancellation) in [
        ("attempt-export-exhausted", exhausted, "budget_exhausted", "not_requested"),
        ("attempt-export-cancelled", vec![cancelled], "cancelled", "observed_terminated"),
    ] {
        let fixture = Fixture::new();
        let request = fixture.request(attempt, Some(super::policy()), None);
        let (observation, _, _) = super::scenario(&fixture, &request, script, FixtureChecks::admitted()).await;
        assert_eq!(observation["disposition"]["kind"], kind);
        let path = fixture.write(&request);
        let parent = output_parent(&fixture.root, "exports");
        assert_eq!(export(&path, &fixture.state, &parent), refused(Reason::RepairNotCandidate));
        assert!(is_empty(&parent), "nothing is exported");
        let record = super::super::super::refusal::record(&request.envelope, Reason::RepairNotCandidate);
        assert_eq!(
            (record["reason"].clone(), record["dispatched"].clone()),
            ("repair_not_candidate".into(), true.into())
        );
        let instance = &inventory(&path, &fixture.state).expect("inventory")["instance"];
        assert_eq!(instance["lifecycle"], serde_json::json!({"state": "terminal_observed", "disposition": kind}));
        assert_eq!(instance["cancellation"], cancellation);
        assert_eq!(instance["liveness"]["state"], "exited");
        assert_eq!(instance["wait_reasons"], serde_json::json!([]));
    }
}

#[tokio::test]
async fn missing_malformed_crossed_and_contradicted_dispositions_are_refused() {
    let fixture = Fixture::new();
    let request = fixture.request("attempt-export-records", Some(super::policy()), None);
    super::scenario(&fixture, &request, vec![super::write("fixed")], FixtureChecks::admitted()).await;
    // The same attempt at another fence, run in another state root.
    let other_fixture = Fixture::new();
    let mut other = other_fixture.request("attempt-export-records", Some(super::policy()), None);
    other.envelope.fence = 8;
    let other = super::readmit(other, &other_fixture.workspace, Some(super::policy()), None);
    let script = vec![super::write("fixed")];
    let (crossed, _, _) = super::scenario(&other_fixture, &other, script, FixtureChecks::admitted()).await;
    assert_eq!(crossed["disposition"]["kind"], "correct_candidate");

    let path = fixture.write(&request);
    let parent = output_parent(&fixture.root, "exports");
    let directory = fixture.directory(&request);
    let retained = directory.join("repair/disposition.json");
    let step = directory.join("repair/step-1.json");
    let original = std::fs::read(&retained).expect("disposition");
    let original_step = std::fs::read(&step).expect("step record");
    let observation: serde_json::Value = serde_json::from_slice(&original).expect("json");
    let mut forged = crossed.clone();
    for field in ["envelope", "request_identity", "worker_request", "execution_binding"] {
        forged[field] = observation[field].clone();
    }
    let mut unschema = observation.clone();
    unschema["schema"] = "lattice.implementation-attempt-observation.v1".into();
    let mut failed_step: serde_json::Value = serde_json::from_slice(&original_step).expect("step json");
    failed_step["verdict"] = serde_json::json!({"kind": "failed", "classes": ["test"]});
    // Only the four fields earlier reads checked.
    let mut minimal = serde_json::json!({"disposition": {"kind": "cancelled"}});
    for field in ["schema", "worker_request", "request_identity"] {
        minimal[field] = observation[field].clone();
    }
    // Another fence's disposition under this request's two identities.
    let mut renamed = crossed.clone();
    for field in ["request_identity", "worker_request"] {
        renamed[field] = observation[field].clone();
    }
    let bytes = |value: &serde_json::Value| Some(serde_json::to_vec(value).expect("bytes"));
    let step_bytes = Some(original_step.clone());
    let invalid = Reason::RepairDispositionInvalid;
    let cases = [
        (
            "a crash before the disposition",
            None,
            step_bytes.clone(),
            Reason::RepairOutcomeUnknown,
            Seen::Unknown,
        ),
        ("malformed", Some(b"{not json".to_vec()), step_bytes.clone(), invalid, Seen::Error),
        ("not a repair observation", bytes(&unschema), step_bytes.clone(), invalid, Seen::Error),
        ("only the identities and a kind", bytes(&minimal), step_bytes.clone(), invalid, Seen::Error),
        ("another fence's disposition", bytes(&crossed), step_bytes.clone(), invalid, Seen::Error),
        ("another fence's envelope", bytes(&renamed), step_bytes.clone(), invalid, Seen::Error),
        ("another fence's revisions", bytes(&forged), step_bytes.clone(), invalid, Seen::Error),
        ("no step record", Some(original.clone()), None, invalid, Seen::Error),
        ("a relabelled step verdict", Some(original.clone()), bytes(&failed_step), invalid, Seen::Error),
    ];
    for (name, disposition, step_record, reason, seen) in cases {
        for (file, content) in [(&retained, disposition), (&step, step_record)] {
            if file.exists() {
                std::fs::remove_file(file).expect("remove the retained record");
            }
            if let Some(content) = content {
                std::fs::write(file, content).expect("replace the retained record");
            }
        }
        assert_eq!(export(&path, &fixture.state, &parent), refused(reason), "{name}");
        seen.assert(inventory(&path, &fixture.state), name);
    }
    assert!(is_empty(&parent), "nothing is exported");
    std::fs::write(&retained, &original).expect("restore the disposition");
    std::fs::write(&step, &original_step).expect("restore the step record");
    std::fs::write(fixture.workspace.join("src/lib.txt"), "changed after the disposition").expect("edit");
    let drifted = export(&path, &fixture.state, &parent).expect_err("the workspace is not the candidate");
    assert!(drifted.contains("workspace differs from the exact retained candidate revision"), "{drifted}");
    assert!(is_empty(&parent), "nothing is exported");
    std::fs::write(fixture.workspace.join("src/lib.txt"), "fixed").expect("restore");
    export(&path, &fixture.state, &parent).expect("the restored records export");
}

#[tokio::test]
async fn a_revoked_revision_refuses_the_export_of_a_repaired_candidate() {
    let fixture = super::super::revisions::Fixture::new();
    let workspace = fixture.root.join("workspace");
    super::seed(&workspace);
    let request = fixture.request("attempt-managed-export", 7);
    let request = super::readmit(request, &workspace, Some(super::policy()), None);
    let (path, _) = fixture.write("request.json", &request);
    let (definition, revision) = fixture.prepared(&request, &path);
    fixture.promoted(&definition, &revision);
    let mut worker = FixtureWorker::new(vec![super::write("still broken"), super::write("fixed")]);
    let mut checks = FixtureChecks::admitted();
    let host = super::Host {
        clock: super::FixtureClock::new(),
        compute: &mut crate::execution_binding::UnavailableComputePort,
        revisions: Store(Some(&fixture.store)),
    };
    super::run_with(&path, &fixture.state, &mut worker, &mut checks, host).await.expect("a disposition");
    let observation = super::disposition(&fixture.directory(&request));
    assert_eq!(observation["disposition"]["kind"], "repaired_candidate");
    let managed = |parent: &std::path::Path| {
        super::super::super::export::run(&path, &fixture.state, parent, Store(Some(&fixture.store)))
    };
    managed(&output_parent(&fixture.root, "exports")).expect("a permitted revision exports");
    fixture.revision(&["revoke", "--definition", &definition, "--revision", &revision]).expect("revoke");
    let later = output_parent(&fixture.root, "exports-after-revocation");
    assert_eq!(managed(&later), refused(Reason::ExportRevoked));
    assert!(is_empty(&later), "nothing is exported after revocation");
    let instance = &fixture.inventory(&path, true).expect("inventory")["instance"];
    assert_eq!(instance["pinned"]["value"]["is_blocked"], true);
    assert_eq!(instance["lifecycle"]["disposition"], "repaired_candidate");
}
