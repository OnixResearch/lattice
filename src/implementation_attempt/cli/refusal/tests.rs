// r[verify lattice.implementation_attempt.consumer.scenario.artifact_source]
use crate::implementation_attempt::SourceKind;

fn envelope(kind: SourceKind, source_id: &str) -> crate::implementation_attempt::ImplementationAttemptEnvelope {
    let mut envelope = super::super::tests::host_local().envelope;
    envelope.source_kind = kind;
    envelope.source_id = source_id.to_owned();
    envelope
}

#[test]
fn artifact_sources_name_their_exact_stage_and_keep_a_distinct_tag() {
    let mut request = super::super::tests::host_local();
    request.envelope = envelope(SourceKind::CairnArtifact, "change/specs/spec");
    super::validate_coordinator(&request).expect("exact artifact stage");
    for relabeled in [
        "change/task",
        "task:1",
        "change/specs/spec/extra",
        "change//spec",
        "change/../spec",
    ] {
        request.envelope.source_id = relabeled.to_owned();
        assert!(super::validate_coordinator(&request).expect_err(relabeled).contains("change/stage/artifact"));
    }
    let mut task = super::super::tests::host_local();
    task.envelope = envelope(SourceKind::CairnTask, "change/task");
    super::validate_coordinator(&task).expect("task sources keep their admitted shapes");
    assert_eq!(serde_json::to_string(&SourceKind::CairnArtifact).expect("kind"), "\"cairn_artifact\"");
    assert_ne!(
        envelope(SourceKind::CairnArtifact, "change/specs/spec").identity(),
        envelope(SourceKind::CairnTask, "change/specs/spec").identity()
    );
}

#[test]
fn existing_issue_and_task_identity_bytes_are_unchanged() {
    // Independently computed over the published v1 byte layout (tags 1-3).
    let mut fixed = envelope(SourceKind::CairnTask, "change/task");
    fixed.work_id = "onix/kiln#45".into();
    fixed.work_version = 3;
    fixed.repository = "onix/kiln".into();
    fixed.base = "a".repeat(40);
    fixed.readiness_snapshot = "b".repeat(64);
    fixed.task_spec_digest = "c".repeat(64);
    fixed.verification_profile = "d".repeat(64);
    fixed.attempt_id = "attempt-1".into();
    fixed.fence = 7;
    fixed.workspace_id = "workspace-1".into();
    fixed.worker_profile = "e".repeat(64);
    fixed.budget = crate::implementation_attempt::AttemptBudget {
        max_turns: 8,
        max_seconds: 600,
        max_cost_micros: 10_000,
    };
    fixed.policy_digest = "f".repeat(64);
    assert_eq!(fixed.identity(), "908f0cb128a38577e4e62c6c2e95e403b0a73598bb3956ee8eb2d8b7089229b5");
    fixed.source_kind = SourceKind::GitHubIssue;
    assert_eq!(fixed.identity(), "181a1086fb08db4a61c18685f5a19fe813e9e4d5a144318d74f6de1c13bc723d");
    fixed.source_kind = SourceKind::CairnArtifact;
    fixed.source_id = "change/specs/spec".into();
    assert_eq!(fixed.identity(), "687cc5aa5317d208a966ec9764d399a3352afdc18214b597dd17b988ea3890eb");
    assert_eq!(serde_json::to_string(&SourceKind::GitHubIssue).expect("issue"), "\"github_issue\"");
    assert!(serde_json::from_str::<SourceKind>("\"git_hub_issue\"").is_err(), "no derived alias");
    assert_eq!(serde_json::to_string(&SourceKind::CairnTask).expect("task"), "\"cairn_task\"");
}
