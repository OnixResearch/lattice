// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven
// validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

use super::*;

const BOUNDARY_ID: &str = "boundary.outer";
const TOOL_CARGO: &str = "cargo-test";
const TOOL_RIPGREP: &str = "rg";
const TOOL_SHELL: &str = "shell";
const WITHHELD_ORACLE: &str = "oracle-hidden";
const VERIFIER_LABEL: &str = "verifier-main";
const SANDBOX_STRICT: &str = "sandbox-strict";
const SANDBOX_WEAK: &str = "sandbox-weak";
const EVIDENCE_TEST: &str = "test-evidence";
const UNKNOWN_EVIDENCE: &str = "unknown-evidence";
const HASH_LABEL: &str = "boundary";
const VERIFIER_HASH_LABEL: &str = "verifier";
const RECEIPT_LABEL: &str = "receipt";
const ACTION_ID: &str = "action.one";

#[test]
fn valid_boundary_passes_and_invalid_descriptor_fails_closed() {
    // r[verify agent_harness_runtime.ornith_trust_boundary.outer_boundary.invalid]
    let valid = boundary();
    assert!(validate_trust_boundary_descriptor(&valid).is_ok());

    let invalid = TrustBoundaryDescriptor {
        sanctioned_tools: Vec::new(),
        withheld_path_labels: vec![
            WITHHELD_ORACLE.to_string(),
            WITHHELD_ORACLE.to_string(),
            "/raw/secret".to_string(),
        ],
        verifier: TrustBoundaryVerifierArtifact {
            label: VERIFIER_LABEL.to_string(),
            hash: TrustBoundaryFingerprint::new(BLAKE3_ALGORITHM, String::new()),
        },
        mutable_boundary_request: true,
        ..valid
    };
    let diagnostics = validate_trust_boundary_descriptor(&invalid).expect_err("invalid boundary");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == TrustBoundaryDiagnosticKind::EmptyCollection && diagnostic.field == FIELD_SANCTIONED_TOOLS
    }));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == TrustBoundaryDiagnosticKind::DuplicateLabel })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == TrustBoundaryDiagnosticKind::UnsafeRawPath })
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == TrustBoundaryDiagnosticKind::InvalidHash && diagnostic.field == FIELD_VERIFIER_HASH
    }));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == TrustBoundaryDiagnosticKind::MutableBoundaryRequest })
    );
}

#[test]
fn monitor_allows_safe_actions_and_denies_exact_boundary_violations() {
    // r[verify agent_harness_runtime.ornith_trust_boundary.monitor.violations]
    let boundary = boundary();
    let allowed = action(TrustBoundaryActionKind::ToolCall, Some(TOOL_CARGO), None, None, None, true);
    let allowed_decision = monitor_trust_boundary_action(&boundary, &allowed);
    assert!(allowed_decision.accepted);
    assert!(allowed_decision.issue_codes.is_empty());

    let cases = [
        (
            action(TrustBoundaryActionKind::PathRead, None, Some(WITHHELD_ORACLE), None, None, true),
            TrustBoundaryIssueCode::WithheldPathRead,
        ),
        (
            action(TrustBoundaryActionKind::VerifierModification, None, Some(VERIFIER_LABEL), None, None, true),
            TrustBoundaryIssueCode::VerifierModification,
        ),
        (
            action(TrustBoundaryActionKind::ToolCall, Some(TOOL_SHELL), None, None, None, true),
            TrustBoundaryIssueCode::UnsanctionedTool,
        ),
        (
            action(TrustBoundaryActionKind::SandboxProfileChange, None, None, Some(SANDBOX_WEAK), None, true),
            TrustBoundaryIssueCode::SandboxWeakening,
        ),
        (
            action(TrustBoundaryActionKind::ApprovalSuppression, None, None, None, None, false),
            TrustBoundaryIssueCode::ApprovalSuppression,
        ),
        (
            action(TrustBoundaryActionKind::EvidenceAcceptance, None, None, None, Some(UNKNOWN_EVIDENCE), true),
            TrustBoundaryIssueCode::EvidencePolicyBypass,
        ),
    ];

    for (candidate, expected_issue) in cases {
        let decision = monitor_trust_boundary_action(&boundary, &candidate);
        assert!(!decision.accepted);
        assert!(decision.issue_codes.contains(&expected_issue));
    }
}

#[test]
fn judge_approval_cannot_override_monitor_denial_or_grant_authority() {
    // r[verify agent_harness_runtime.ornith_trust_boundary.judge_veto.no_override]
    let boundary = boundary();
    let denied = monitor_trust_boundary_action(
        &boundary,
        &action(TrustBoundaryActionKind::ToolCall, Some(TOOL_SHELL), None, None, None, true),
    );
    let binding = bind_trust_boundary_judge(&denied, &TrustBoundaryJudgeVeto {
        outcome: TrustBoundaryJudgeOutcome::Approve,
        reason_label: Some("judge-ok".to_string()),
    });
    assert!(!binding.accepted);
    assert!(binding.monitor_denial_final);
    assert!(binding.judge_grant_ignored);

    let allowed = monitor_trust_boundary_action(
        &boundary,
        &action(TrustBoundaryActionKind::ToolCall, Some(TOOL_CARGO), None, None, None, true),
    );
    let veto = bind_trust_boundary_judge(&allowed, &TrustBoundaryJudgeVeto {
        outcome: TrustBoundaryJudgeOutcome::Veto,
        reason_label: Some("suspicious-intent".to_string()),
    });
    assert!(!veto.accepted);
    assert!(!veto.monitor_denial_final);
}

#[test]
fn anti_gaming_fixtures_are_reproducible_without_provider_calls() {
    // r[verify agent_harness_runtime.ornith_trust_boundary.anti_gaming.fixtures]
    let boundary = boundary();
    let fixtures = [
        (TrustBoundaryActionKind::OracleCopy, TrustBoundaryIssueCode::OracleCopy),
        (TrustBoundaryActionKind::HardcodedExpectedOutput, TrustBoundaryIssueCode::HardcodedExpectedOutput),
        (TrustBoundaryActionKind::VisibleTestOverfit, TrustBoundaryIssueCode::VisibleTestOverfit),
        (TrustBoundaryActionKind::AllowedToolIntentGaming, TrustBoundaryIssueCode::AllowedToolIntentGaming),
    ];

    for (kind, expected_issue) in fixtures {
        let decision =
            monitor_trust_boundary_action(&boundary, &action(kind, Some(TOOL_RIPGREP), None, None, None, true));
        assert!(!decision.accepted);
        assert!(decision.issue_codes.contains(&expected_issue));
    }
}

#[test]
fn violation_receipts_accept_redacted_metadata_and_reject_leaks() {
    // r[verify agent_harness_runtime.ornith_trust_boundary.receipts.redacted]
    let safe = receipt(Vec::new(), TrustBoundaryRedactionStatus::Redacted);
    assert!(validate_trust_boundary_receipt(&safe).is_ok());

    let leaking = receipt(
        vec![
            TrustBoundaryReceiptLeak::HiddenPath,
            TrustBoundaryReceiptLeak::RawVerifierBody,
            TrustBoundaryReceiptLeak::Prompt,
            TrustBoundaryReceiptLeak::ToolBody,
            TrustBoundaryReceiptLeak::ProviderPayload,
            TrustBoundaryReceiptLeak::Credential,
            TrustBoundaryReceiptLeak::FullPathDump,
        ],
        TrustBoundaryRedactionStatus::Unsafe,
    );
    let diagnostics = validate_trust_boundary_receipt(&leaking).expect_err("receipt leaks");

    assert!(diagnostics.iter().any(|diagnostic| { diagnostic.kind == TrustBoundaryDiagnosticKind::ReceiptLeak }));
    assert!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.kind == TrustBoundaryDiagnosticKind::ReceiptLeak)
            .count()
            > leaking.leaks.len()
    );
}

#[allow(clippy::too_many_arguments)]
fn action(
    kind: TrustBoundaryActionKind,
    tool_name: Option<&str>,
    path_label: Option<&str>,
    sandbox_profile_id: Option<&str>,
    evidence_label: Option<&str>,
    approval_present: bool,
) -> TrustBoundaryAction {
    TrustBoundaryAction {
        action_id: ACTION_ID.to_string(),
        kind,
        tool_name: tool_name.map(str::to_string),
        path_label: path_label.map(str::to_string),
        sandbox_profile_id: sandbox_profile_id.map(str::to_string),
        evidence_label: evidence_label.map(str::to_string),
        approval_present,
    }
}

fn boundary() -> TrustBoundaryDescriptor {
    TrustBoundaryDescriptor {
        boundary_id: BOUNDARY_ID.to_string(),
        sanctioned_tools: vec![TOOL_CARGO.to_string(), TOOL_RIPGREP.to_string()],
        withheld_path_labels: vec![WITHHELD_ORACLE.to_string()],
        verifier: TrustBoundaryVerifierArtifact {
            label: VERIFIER_LABEL.to_string(),
            hash: fp(VERIFIER_HASH_LABEL),
        },
        sandbox_profile_id: SANDBOX_STRICT.to_string(),
        approval_required_actions: vec![TrustBoundaryActionKind::EvidenceAcceptance],
        evidence_policy: TrustBoundaryEvidencePolicy {
            accepted_evidence_labels: vec![EVIDENCE_TEST.to_string()],
            require_monitor_clean: true,
        },
        policy_hash: fp(HASH_LABEL),
        mutable_boundary_request: false,
    }
}

fn fp(label: &str) -> TrustBoundaryFingerprint {
    blake3_trust_boundary_fingerprint(label, label.as_bytes())
}

fn receipt(
    leaks: Vec<TrustBoundaryReceiptLeak>,
    redaction_status: TrustBoundaryRedactionStatus,
) -> TrustBoundaryViolationReceipt {
    TrustBoundaryViolationReceipt {
        policy_hash: fp(RECEIPT_LABEL),
        issue_code: TrustBoundaryIssueCode::UnsanctionedTool,
        labels: vec!["tool-label".to_string()],
        count: 1,
        redaction_status,
        leaks,
    }
}
