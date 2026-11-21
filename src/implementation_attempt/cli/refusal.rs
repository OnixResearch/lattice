//! Coordinator request checks and typed refusals.
//!
//! A refusal is printed once as `lattice.implementation-attempt-refusal.v1` on
//! standard output and the command exits non-zero. It names the attempt it
//! refused. `dispatched` is `false` only when no worker effect of that attempt
//! can have started; whenever the attempt directory holds an execution intent
//! (a substituted or superseded request, or an export refused after the worker
//! ran) it is `true`, so a coordinator never settles a possibly executed
//! attempt as refused before any effect.

// r[impl lattice.implementation_attempt.consumer.scenario.refusal]

pub(super) const SCHEMA: &str = "lattice.implementation-attempt-refusal.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Reason {
    /// A higher fence of the same work is already recorded in this state root.
    StaleFence,
    /// Another attempt is already recorded at this work and fence.
    CrossedAttempt,
    /// The attempt directory holds the intent of different request bytes,
    /// which may have executed.
    SubstitutedRequest,
    /// The envelope's profile revision is not this request's current framing.
    ProfileRevisionSuperseded,
    /// The lifecycle-managed definition admits no new session of this revision.
    RevisionNotAdmitted,
    /// The session's pinned revision, qualification or session record permits
    /// no further effect; refused before spawn.
    RevisionBlocked,
    /// The managed attempt executed, but its session, pinned revision or
    /// qualification has since been revoked, so its retained candidate is not
    /// exported.
    ExportRevoked,
    /// Kiln's budget lineage reports a blocker or serving backpressure;
    /// refused before any write.
    LineageBlocked,
    /// Kiln's budget lineage cannot cover the first step; refused before any
    /// write.
    LineageExhausted,
    /// A repair policy needs embedder-supplied worker and check ports.
    RepairPortsUnavailable,
}

impl Reason {
    pub(super) const fn message(self) -> &'static str {
        match self {
            Self::StaleFence => "implementation request is stale: a higher fence of this work is already recorded",
            Self::CrossedAttempt => "implementation request is crossed: another attempt is recorded at this fence",
            Self::SubstitutedRequest => {
                "request differs from persisted execution intent; this request dispatched nothing, but the \
                 persisted intent may have executed"
            }
            Self::ProfileRevisionSuperseded => {
                "implementation request is stale, crossed or unsupported: its profile revision is not the \
                 lattice-worker-profile-revision-v2 framing; it is never executed or recovered"
            }
            Self::RevisionNotAdmitted => {
                "implementation request is refused: its agent revision is not admitted for new sessions; \
                 nothing was dispatched"
            }
            Self::RevisionBlocked => {
                "implementation attempt is blocked: its pinned agent revision permits no further effect; \
                 nothing was dispatched"
            }
            Self::ExportRevoked => {
                "implementation attempt executed, but its agent revision session, revision or qualification \
                 is revoked; its retained candidate is not exported"
            }
            Self::LineageBlocked => {
                "implementation request is refused: its Kiln budget lineage is blocked or under backpressure; \
                 nothing was dispatched"
            }
            Self::LineageExhausted => {
                "implementation request is refused: its Kiln budget lineage cannot cover the first step; \
                 nothing was dispatched"
            }
            Self::RepairPortsUnavailable => {
                "implementation request is refused: its repair policy needs embedder-supplied worker and check \
                 ports; nothing was dispatched"
            }
        }
    }

    /// Whether a worker effect of the refused attempt can have started.
    const fn could_have_dispatched(self) -> bool {
        match self {
            Self::SubstitutedRequest | Self::ExportRevoked => true,
            Self::StaleFence
            | Self::CrossedAttempt
            | Self::ProfileRevisionSuperseded
            | Self::RevisionNotAdmitted
            | Self::RevisionBlocked
            | Self::LineageBlocked
            | Self::LineageExhausted
            | Self::RepairPortsUnavailable => false,
        }
    }
}

/// The refusal record of one request.
pub(super) fn record(
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
    reason: Reason,
) -> serde_json::Value {
    record_with(envelope, reason, reason.could_have_dispatched())
}

fn record_with(
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
    reason: Reason,
    dispatched: bool,
) -> serde_json::Value {
    serde_json::json!({
        "schema": SCHEMA,
        "reason": reason,
        "work_id": envelope.work_id,
        "attempt_id": envelope.attempt_id,
        "fence": envelope.fence,
        "dispatched": dispatched,
    })
}

/// Print the typed refusal and return its diagnostic.
pub(super) fn refuse(
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
    reason: Reason,
) -> String {
    println!("{}", record(envelope, reason));
    reason.message().to_owned()
}

/// Refuse a request retained under a superseded profile framing. Its attempt
/// may have executed under that framing when its directory holds an intent.
pub(super) fn refuse_superseded(
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
    has_retained_intent: bool,
) -> String {
    let reason = Reason::ProfileRevisionSuperseded;
    println!("{}", record_with(envelope, reason, has_retained_intent));
    reason.message().to_owned()
}

/// Print the typed refusal with the agent revision lifecycle's own refusal
/// and return its diagnostic.
// r[impl lattice.agent_revisions.seam.scenario.attempt]
pub(super) fn refuse_revision(
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
    reason: Reason,
    refusal: &lattice_core::agent_revision::Refusal,
) -> String {
    debug_assert!(matches!(reason, Reason::RevisionNotAdmitted | Reason::RevisionBlocked | Reason::ExportRevoked));
    let mut printed = record(envelope, reason);
    if let Some(fields) = printed.as_object_mut() {
        fields.insert("revision_refusal".to_owned(), serde_json::json!(refusal));
    }
    println!("{printed}");
    reason.message().to_owned()
}

/// Print the typed lineage refusal with Kiln's lineage reason and return
/// its diagnostic.
// r[impl lattice.implementation_attempt.lineage.scenario.exhausted]
pub(super) fn refuse_lineage(
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
    reason: Reason,
    refusal: &crate::implementation_attempt::lineage::Refusal,
) -> String {
    debug_assert!(matches!(reason, Reason::LineageBlocked | Reason::LineageExhausted));
    let mut printed = record(envelope, reason);
    if let Some(fields) = printed.as_object_mut() {
        fields.insert("lineage_refusal".to_owned(), serde_json::json!(refusal));
    }
    println!("{printed}");
    reason.message().to_owned()
}

/// Refuse a shared-network declaration, a malformed dispatch-plan identity
/// and a Cairn artifact source that does not name its exact native stage.
// r[impl lattice.implementation_attempt.consumer.scenario.artifact_source]
pub(super) fn validate_coordinator(request: &super::Request) -> Result<(), String> {
    if request.network == Some(true) {
        return Err(
            "shared-network attempt requests are unsupported; the OMP worker has only its brokered model route".into(),
        );
    }
    if request.dispatch_plan.as_deref().is_some_and(|plan| !crate::implementation_attempt::is_blake3(plan)) {
        return Err("dispatch-plan identity is not 64 lowercase hexadecimal digits".into());
    }
    let is_artifact = request.envelope.source_kind == crate::implementation_attempt::SourceKind::CairnArtifact;
    if is_artifact && !artifact_source(&request.envelope.source_id) {
        return Err("Cairn artifact source must name its exact change/stage/artifact".into());
    }
    debug_assert!(request.network != Some(true));
    Ok(())
}

/// A Cairn artifact source names `change/stage/artifact`; it is never a
/// relabeled coding-task identifier.
fn artifact_source(value: &str) -> bool {
    let mut parts = value.split('/');
    let segments = [parts.next(), parts.next(), parts.next()];
    parts.next().is_none() && segments.iter().all(|part| part.is_some_and(|part| !matches!(part, "" | "." | "..")))
}

#[cfg(test)]
mod tests {
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
}
