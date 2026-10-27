// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

use super::*;

const PROFILE_ID: &str = "scaffold.default";
const FALLBACK_PROFILE_ID: &str = "scaffold.general";
const REVISION_ID: &str = "revision.one";
const NEXT_REVISION_ID: &str = "revision.two";
const PROPOSAL_ID: &str = "proposal.one";
const TASK_CLASS: &str = "coding";
const FALLBACK_TASK_CLASS: &str = "general";
const ROUTE_FAST: &str = "route.fast";
const ROUTE_LOCAL: &str = "route.local";
const ROUTE_REMOTE: &str = "route.remote";
const SOURCE: &str = "ornith-reference-adaptation";
const REVIEWER: &str = "runtime-review";
const RATIONALE: &str = "route-neutral reviewed strategy data";
const PROFILE_LABEL: &str = "profile";
const REVISION_LABEL: &str = "revision";
const PROPOSED_LABEL: &str = "proposed";

#[test]
fn reviewed_profile_validates_and_invalid_fixtures_fail_closed() {
    // r[verify agent_harness_runtime.ornith_scaffold.profile_contract.invalid]
    let valid = profile(PROFILE_ID, TASK_CLASS, ROUTE_FAST, true);
    assert!(validate_scaffold_profile(&valid).is_ok());

    let invalid = ScaffoldProfile {
        profile_id: "bad".to_string(),
        active_revision_id: REVISION_ID.to_string(),
        provenance: ScaffoldProfileProvenance {
            source: String::new(),
            reviewed_by: Some(REVIEWER.to_string()),
            rationale: Some(RATIONALE.to_string()),
        },
        revisions: vec![
            revision(REVISION_ID, ScaffoldReviewStatus::Proposed),
            ScaffoldRevision {
                revision_id: REVISION_ID.to_string(),
                strategy_knobs: vec![ScaffoldStrategyKnob::Unsupported("shell".to_string())],
                hash_matches_content: false,
                executable_harness: true,
                ..revision(REVISION_ID, ScaffoldReviewStatus::Reviewed)
            },
        ],
        ..valid
    };
    let diagnostics = validate_scaffold_profile(&invalid).expect_err("invalid scaffold");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ScaffoldDiagnosticKind::MalformedId
            && diagnostic.field == FIELD_PROFILE_ID
    }));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::DuplicateRevision })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::UnsupportedKnob })
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ScaffoldDiagnosticKind::MissingField
            && diagnostic.field == FIELD_PROFILE_PROVENANCE
    }));
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == ScaffoldDiagnosticKind::UnreviewedActiveRevision
        })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::ContentHashInvalid })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::ExecutableHarness })
    );
}

#[test]
fn selection_prefers_exact_then_fallback_and_reports_policy_rejections() {
    // r[verify agent_harness_runtime.ornith_scaffold.selection.policy]
    let exact = profile(PROFILE_ID, TASK_CLASS, ROUTE_FAST, true);
    let fallback = profile(FALLBACK_PROFILE_ID, FALLBACK_TASK_CLASS, ROUTE_FAST, true);
    let request = selection_request(TASK_CLASS, &[ROUTE_FAST], &[]);

    let selected = select_scaffold_profile(&[fallback.clone(), exact.clone()], &request);
    assert_eq!(selected.outcome, ScaffoldSelectionOutcome::ExactMatch);
    assert_eq!(selected.profile_id.as_deref(), Some(PROFILE_ID));

    let fallback_selected = select_scaffold_profile(
        &[
            fallback.clone(),
            profile(PROFILE_ID, TASK_CLASS, ROUTE_FAST, false),
        ],
        &request,
    );
    assert_eq!(
        fallback_selected.outcome,
        ScaffoldSelectionOutcome::Fallback
    );
    assert_eq!(
        fallback_selected.profile_id.as_deref(),
        Some(FALLBACK_PROFILE_ID)
    );
    assert!(
        fallback_selected
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::DisabledProfile })
    );

    let denied = select_scaffold_profile(
        &[fallback, exact],
        &selection_request(TASK_CLASS, &[ROUTE_REMOTE], &[FALLBACK_PROFILE_ID]),
    );
    assert_eq!(
        denied.outcome,
        ScaffoldSelectionOutcome::NoCompatibleProfile
    );
    assert!(
        denied
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::RouteMismatch })
    );
    assert!(
        denied
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::NoCompatibleProfile })
    );
}

#[test]
fn admission_denied_unavailable_or_stale_never_influences_future_turns() {
    // r[verify agent_harness_runtime.ornith_scaffold.admission.unreviewed]
    let profile = profile(PROFILE_ID, TASK_CLASS, ROUTE_FAST, true);
    let request = admission_request(PROFILE_ID, REVISION_ID);
    let admitted = apply_scaffold_admission(&profile, &request, ScaffoldAdmissionOutcome::Admitted);
    assert_eq!(admitted.effect, ScaffoldAdmissionEffect::PersistAndActivate);
    assert!(admitted.persist);
    assert!(admitted.activate);
    assert!(admitted.future_influence);

    for outcome in [
        ScaffoldAdmissionOutcome::Denied,
        ScaffoldAdmissionOutcome::Unavailable,
        ScaffoldAdmissionOutcome::StaleBase,
    ] {
        let decision = apply_scaffold_admission(&profile, &request, outcome);
        assert_eq!(decision.effect, ScaffoldAdmissionEffect::RecordOnly);
        assert!(!decision.persist);
        assert!(!decision.activate);
        assert!(!decision.future_influence);
        assert!(
            decision
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::AdmissionRejected })
        );
    }

    let stale = admission_request(PROFILE_ID, NEXT_REVISION_ID);
    let decision = apply_scaffold_admission(&profile, &stale, ScaffoldAdmissionOutcome::Admitted);
    assert!(!decision.future_influence);
    assert!(
        decision
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::ActiveRevisionInvalid })
    );
}

#[test]
fn receipts_accept_safe_metadata_and_reject_sensitive_bodies() {
    // r[verify agent_harness_runtime.ornith_scaffold.receipts.redacted]
    let safe = receipt(Vec::new());
    assert!(validate_scaffold_influence_receipt(&safe).is_ok());

    let leaking = receipt(vec![
        ScaffoldReceiptSensitiveEvidence::RawPrompt,
        ScaffoldReceiptSensitiveEvidence::ScaffoldBody,
        ScaffoldReceiptSensitiveEvidence::ProviderPayload,
        ScaffoldReceiptSensitiveEvidence::Credential,
        ScaffoldReceiptSensitiveEvidence::FullPath,
        ScaffoldReceiptSensitiveEvidence::UnboundedToolOutput,
    ]);
    let diagnostics = validate_scaffold_influence_receipt(&leaking).expect_err("receipt leak");

    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == ScaffoldDiagnosticKind::ReceiptLeak)
            .count(),
        leaking.sensitive_evidence.len()
    );
}

#[test]
fn executable_scaffold_harness_is_rejected_as_out_of_scope() {
    // r[verify agent_harness_runtime.ornith_scaffold.scope.executable_rejected]
    let profile = ScaffoldProfile {
        revisions: vec![ScaffoldRevision {
            executable_harness: true,
            ..revision(REVISION_ID, ScaffoldReviewStatus::Reviewed)
        }],
        ..profile(PROFILE_ID, TASK_CLASS, ROUTE_FAST, true)
    };

    let diagnostics = validate_scaffold_profile(&profile).expect_err("executable rejected");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ScaffoldDiagnosticKind::ExecutableHarness })
    );
}

fn admission_request(profile_id: &str, base_revision_id: &str) -> ScaffoldAdmissionRequest {
    ScaffoldAdmissionRequest {
        proposal: ScaffoldRevisionProposal {
            proposal_id: PROPOSAL_ID.to_string(),
            profile_id: profile_id.to_string(),
            base_revision_id: base_revision_id.to_string(),
            proposed_revision_id: NEXT_REVISION_ID.to_string(),
            proposed_content_hash: fp(PROPOSED_LABEL),
            proposed_strategy_knobs: vec![ScaffoldStrategyKnob::ValidationChecklist],
        },
    }
}

fn fp(label: &str) -> ScaffoldFingerprint {
    blake3_scaffold_fingerprint(label, label.as_bytes())
}

fn profile(profile_id: &str, task_class: &str, route: &str, enabled: bool) -> ScaffoldProfile {
    ScaffoldProfile {
        profile_id: profile_id.to_string(),
        task_classes: vec![task_class.to_string()],
        active_revision_id: REVISION_ID.to_string(),
        revisions: vec![revision(REVISION_ID, ScaffoldReviewStatus::Reviewed)],
        route_constraints: vec![route.to_string()],
        enabled,
        provenance: ScaffoldProfileProvenance {
            source: SOURCE.to_string(),
            reviewed_by: Some(REVIEWER.to_string()),
            rationale: Some(RATIONALE.to_string()),
        },
    }
}

fn receipt(sensitive_evidence: Vec<ScaffoldReceiptSensitiveEvidence>) -> ScaffoldInfluenceReceipt {
    ScaffoldInfluenceReceipt {
        profile_id: PROFILE_ID.to_string(),
        revision_id: REVISION_ID.to_string(),
        task_class: TASK_CLASS.to_string(),
        review_status: ScaffoldReviewStatus::Reviewed,
        route_labels: vec![ROUTE_FAST.to_string(), ROUTE_LOCAL.to_string()],
        validation_outcome: ScaffoldInfluenceValidationOutcome::Accepted,
        profile_hash: fp(PROFILE_LABEL),
        revision_hash: fp(REVISION_LABEL),
        sensitive_evidence,
    }
}

fn revision(revision_id: &str, review_status: ScaffoldReviewStatus) -> ScaffoldRevision {
    ScaffoldRevision {
        revision_id: revision_id.to_string(),
        strategy_knobs: vec![
            ScaffoldStrategyKnob::RetryPosture,
            ScaffoldStrategyKnob::ValidationChecklist,
        ],
        review_status,
        content_hash: fp(REVISION_LABEL),
        hash_matches_content: true,
        executable_harness: false,
    }
}

fn selection_request(
    task_class: &str,
    route_capability_labels: &[&str],
    disabled_profile_ids: &[&str],
) -> ScaffoldSelectionRequest {
    ScaffoldSelectionRequest {
        task_class: task_class.to_string(),
        route_capability_labels: route_capability_labels
            .iter()
            .map(|label| (*label).to_string())
            .collect(),
        disabled_profile_ids: disabled_profile_ids
            .iter()
            .map(|profile_id| (*profile_id).to_string())
            .collect(),
        fallback_task_class: Some(FALLBACK_TASK_CLASS.to_string()),
    }
}
