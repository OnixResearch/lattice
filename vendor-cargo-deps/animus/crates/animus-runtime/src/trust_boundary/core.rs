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
use std::collections::BTreeSet;

use super::{
    BLAKE3_ALGORITHM, FIELD_ACTION_ID, FIELD_ACTION_KIND, FIELD_APPROVAL, FIELD_BOUNDARY_ID,
    FIELD_EVIDENCE, FIELD_MUTABLE_REQUEST, FIELD_POLICY_HASH, FIELD_RECEIPT_LEAK,
    FIELD_RECEIPT_POLICY_HASH, FIELD_SANCTIONED_TOOLS, FIELD_SANDBOX, FIELD_VERIFIER_HASH,
    FIELD_VERIFIER_LABEL, FIELD_WITHHELD_LABELS, HASH_PART_SEPARATOR, TRUST_BOUNDARY_HASH_DOMAIN,
    data::{
        TrustBoundaryAction, TrustBoundaryActionKind, TrustBoundaryDescriptor,
        TrustBoundaryDiagnostic, TrustBoundaryDiagnosticKind, TrustBoundaryFingerprint,
        TrustBoundaryIssueCode, TrustBoundaryJudgeBinding, TrustBoundaryJudgeOutcome,
        TrustBoundaryJudgeVeto, TrustBoundaryMonitorDecision, TrustBoundaryReceiptLeak,
        TrustBoundaryRedactionStatus, TrustBoundaryValidationResult, TrustBoundaryViolationReceipt,
    },
};

const PATH_SEPARATOR: char = '/';
const WINDOWS_PATH_SEPARATOR: char = '\\';
const HOME_PATH_PREFIX: &str = "~";
const RAW_PATH_MARKER: &str = "://";
const MIN_RECEIPT_COUNT: u32 = 1;

/// Computes a domain-separated BLAKE3 fingerprint for trust-boundary evidence.
#[must_use]
pub fn blake3_trust_boundary_fingerprint(label: &str, bytes: &[u8]) -> TrustBoundaryFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TRUST_BOUNDARY_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    TrustBoundaryFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Validates an immutable trust-boundary descriptor.
///
/// # Errors
///
/// Returns all deterministic descriptor diagnostics, including missing verifier
/// hashes, duplicate labels, empty sanctioned tools, raw path serialization, and
/// mutable-boundary requests.
pub fn validate_trust_boundary_descriptor(
    boundary: &TrustBoundaryDescriptor,
) -> TrustBoundaryValidationResult {
    let mut diagnostics = Vec::new();
    require_text(&boundary.boundary_id, FIELD_BOUNDARY_ID, &mut diagnostics);
    require_collection(
        &boundary.sanctioned_tools,
        FIELD_SANCTIONED_TOOLS,
        &mut diagnostics,
    );
    require_collection(
        &boundary.withheld_path_labels,
        FIELD_WITHHELD_LABELS,
        &mut diagnostics,
    );
    detect_duplicate_or_raw_labels(
        &boundary.withheld_path_labels,
        FIELD_WITHHELD_LABELS,
        &mut diagnostics,
    );
    require_text(
        &boundary.verifier.label,
        FIELD_VERIFIER_LABEL,
        &mut diagnostics,
    );
    detect_raw_label(
        &boundary.verifier.label,
        FIELD_VERIFIER_LABEL,
        &mut diagnostics,
    );
    require_blake3(
        &boundary.verifier.hash,
        FIELD_VERIFIER_HASH,
        &mut diagnostics,
    );
    require_text(
        &boundary.sandbox_profile_id,
        FIELD_SANDBOX,
        &mut diagnostics,
    );
    require_collection(
        &boundary.evidence_policy.accepted_evidence_labels,
        FIELD_EVIDENCE,
        &mut diagnostics,
    );
    detect_duplicate_or_raw_labels(
        &boundary.evidence_policy.accepted_evidence_labels,
        FIELD_EVIDENCE,
        &mut diagnostics,
    );
    require_blake3(&boundary.policy_hash, FIELD_POLICY_HASH, &mut diagnostics);
    if boundary.mutable_boundary_request {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::MutableBoundaryRequest,
            FIELD_MUTABLE_REQUEST,
            "inner runtime requested mutation of immutable trust boundary",
        ));
    }
    to_validation_result(diagnostics)
}

/// Evaluates one planned or observed action against the trust boundary.
#[must_use]
pub fn monitor_trust_boundary_action(
    boundary: &TrustBoundaryDescriptor,
    action: &TrustBoundaryAction,
) -> TrustBoundaryMonitorDecision {
    let mut diagnostics = Vec::new();
    let mut issue_codes = Vec::new();
    append_descriptor_diagnostics(boundary, &mut diagnostics);
    require_text(&action.action_id, FIELD_ACTION_ID, &mut diagnostics);
    collect_action_issues(boundary, action, &mut issue_codes, &mut diagnostics);
    TrustBoundaryMonitorDecision {
        accepted: diagnostics.is_empty() && issue_codes.is_empty(),
        issue_codes,
        diagnostics,
    }
}

/// Binds optional judge output as reject-only with deterministic monitor output.
#[must_use]
pub fn bind_trust_boundary_judge(
    monitor: &TrustBoundaryMonitorDecision,
    judge: &TrustBoundaryJudgeVeto,
) -> TrustBoundaryJudgeBinding {
    let mut diagnostics = Vec::new();
    let monitor_denied = !monitor.issue_codes.is_empty() || !monitor.accepted;
    if monitor_denied && judge.outcome == TrustBoundaryJudgeOutcome::Approve {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::JudgeRejectOnly,
            FIELD_ACTION_KIND,
            "judge approval cannot override deterministic monitor denial",
        ));
    }
    if judge.outcome == TrustBoundaryJudgeOutcome::Veto {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::JudgeRejectOnly,
            FIELD_ACTION_KIND,
            "judge veto rejects suspicious trajectory without granting authority",
        ));
    }
    TrustBoundaryJudgeBinding {
        accepted: !monitor_denied && judge.outcome != TrustBoundaryJudgeOutcome::Veto,
        monitor_denial_final: monitor_denied,
        judge_grant_ignored: monitor_denied && judge.outcome == TrustBoundaryJudgeOutcome::Approve,
        diagnostics,
    }
}

/// Validates a redacted trust-boundary violation receipt.
///
/// # Errors
///
/// Returns diagnostics when receipt metadata is missing, hashes are not BLAKE3,
/// counts are empty, or sensitive leak classes appear.
pub fn validate_trust_boundary_receipt(
    receipt: &TrustBoundaryViolationReceipt,
) -> TrustBoundaryValidationResult {
    let mut diagnostics = Vec::new();
    require_blake3(
        &receipt.policy_hash,
        FIELD_RECEIPT_POLICY_HASH,
        &mut diagnostics,
    );
    require_collection(&receipt.labels, FIELD_EVIDENCE, &mut diagnostics);
    detect_duplicate_or_raw_labels(&receipt.labels, FIELD_EVIDENCE, &mut diagnostics);
    if receipt.count < MIN_RECEIPT_COUNT {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::MissingField,
            FIELD_EVIDENCE,
            "violation receipt count must be positive",
        ));
    }
    if receipt.redaction_status == TrustBoundaryRedactionStatus::Unsafe {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::ReceiptLeak,
            FIELD_RECEIPT_LEAK,
            "receipt redaction status is unsafe",
        ));
    }
    for leak in &receipt.leaks {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::ReceiptLeak,
            FIELD_RECEIPT_LEAK,
            leak_detail(leak),
        ));
    }
    to_validation_result(diagnostics)
}

fn append_descriptor_diagnostics(
    boundary: &TrustBoundaryDescriptor,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    if let Err(mut descriptor_diagnostics) = validate_trust_boundary_descriptor(boundary) {
        diagnostics.append(&mut descriptor_diagnostics);
    }
}

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn collect_action_issues(
    boundary: &TrustBoundaryDescriptor,
    action: &TrustBoundaryAction,
    issue_codes: &mut Vec<TrustBoundaryIssueCode>,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    match action.kind {
        TrustBoundaryActionKind::ToolCall => {
            monitor_tool(boundary, action, issue_codes, diagnostics);
        }
        TrustBoundaryActionKind::PathRead => {
            monitor_path_read(boundary, action, issue_codes, diagnostics);
        }
        TrustBoundaryActionKind::VerifierModification => push_issue(
            TrustBoundaryIssueCode::VerifierModification,
            FIELD_VERIFIER_LABEL,
            "action attempted verifier modification",
            issue_codes,
            diagnostics,
        ),
        TrustBoundaryActionKind::SandboxProfileChange => {
            monitor_sandbox(boundary, action, issue_codes, diagnostics);
        }
        TrustBoundaryActionKind::ApprovalSuppression => push_issue(
            TrustBoundaryIssueCode::ApprovalSuppression,
            FIELD_APPROVAL,
            "action attempted approval suppression",
            issue_codes,
            diagnostics,
        ),
        TrustBoundaryActionKind::EvidenceAcceptance => {
            monitor_evidence(boundary, action, issue_codes, diagnostics);
        }
        TrustBoundaryActionKind::OracleCopy => push_issue(
            TrustBoundaryIssueCode::OracleCopy,
            FIELD_ACTION_KIND,
            "action attempted oracle copying",
            issue_codes,
            diagnostics,
        ),
        TrustBoundaryActionKind::HardcodedExpectedOutput => push_issue(
            TrustBoundaryIssueCode::HardcodedExpectedOutput,
            FIELD_ACTION_KIND,
            "action hardcoded expected output or artifact",
            issue_codes,
            diagnostics,
        ),
        TrustBoundaryActionKind::VisibleTestOverfit => push_issue(
            TrustBoundaryIssueCode::VisibleTestOverfit,
            FIELD_ACTION_KIND,
            "action overfit visible tests",
            issue_codes,
            diagnostics,
        ),
        TrustBoundaryActionKind::AllowedToolIntentGaming => push_issue(
            TrustBoundaryIssueCode::AllowedToolIntentGaming,
            FIELD_ACTION_KIND,
            "action used an allowed tool for gaming intent",
            issue_codes,
            diagnostics,
        ),
    }
}

fn detect_duplicate_or_raw_labels(
    labels: &[String],
    field: &str,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    let mut seen = BTreeSet::new();
    for label in labels {
        if !seen.insert(label.as_str()) {
            diagnostics.push(TrustBoundaryDiagnostic::new(
                TrustBoundaryDiagnosticKind::DuplicateLabel,
                field,
                "safe labels must be unique",
            ));
        }
        detect_raw_label(label, field, diagnostics);
    }
}

fn detect_raw_label(label: &str, field: &str, diagnostics: &mut Vec<TrustBoundaryDiagnostic>) {
    if label.contains(PATH_SEPARATOR)
        || label.contains(WINDOWS_PATH_SEPARATOR)
        || label.starts_with(HOME_PATH_PREFIX)
        || label.contains(RAW_PATH_MARKER)
    {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::UnsafeRawPath,
            field,
            "trust-boundary descriptors must use safe labels instead of raw paths",
        ));
    }
}

fn labels_contain(labels: &[String], needle: &str) -> bool {
    labels.iter().any(|label| label == needle)
}

fn leak_detail(leak: &TrustBoundaryReceiptLeak) -> &'static str {
    match leak {
        TrustBoundaryReceiptLeak::HiddenPath => "receipt contains hidden path material",
        TrustBoundaryReceiptLeak::RawVerifierBody => "receipt contains raw verifier body",
        TrustBoundaryReceiptLeak::Prompt => "receipt contains prompt text",
        TrustBoundaryReceiptLeak::ToolBody => "receipt contains tool input or output body",
        TrustBoundaryReceiptLeak::ProviderPayload => "receipt contains provider payload",
        TrustBoundaryReceiptLeak::Credential => "receipt contains credential-looking value",
        TrustBoundaryReceiptLeak::FullPathDump => "receipt contains full path dump",
    }
}

fn monitor_evidence(
    boundary: &TrustBoundaryDescriptor,
    action: &TrustBoundaryAction,
    issue_codes: &mut Vec<TrustBoundaryIssueCode>,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    let Some(evidence_label) = action.evidence_label.as_deref() else {
        push_issue(
            TrustBoundaryIssueCode::EvidencePolicyBypass,
            FIELD_EVIDENCE,
            "evidence acceptance action omitted evidence label",
            issue_codes,
            diagnostics,
        );
        return;
    };
    if !labels_contain(
        &boundary.evidence_policy.accepted_evidence_labels,
        evidence_label,
    ) || (boundary.evidence_policy.require_monitor_clean && !action.approval_present)
    {
        push_issue(
            TrustBoundaryIssueCode::EvidencePolicyBypass,
            FIELD_EVIDENCE,
            "action attempted evidence-policy bypass",
            issue_codes,
            diagnostics,
        );
    }
}

fn monitor_path_read(
    boundary: &TrustBoundaryDescriptor,
    action: &TrustBoundaryAction,
    issue_codes: &mut Vec<TrustBoundaryIssueCode>,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    if action
        .path_label
        .as_deref()
        .is_some_and(|label| labels_contain(&boundary.withheld_path_labels, label))
    {
        push_issue(
            TrustBoundaryIssueCode::WithheldPathRead,
            FIELD_WITHHELD_LABELS,
            "action attempted withheld path read",
            issue_codes,
            diagnostics,
        );
    }
}

fn monitor_sandbox(
    boundary: &TrustBoundaryDescriptor,
    action: &TrustBoundaryAction,
    issue_codes: &mut Vec<TrustBoundaryIssueCode>,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    if action
        .sandbox_profile_id
        .as_deref()
        .is_some_and(|profile_id| profile_id != boundary.sandbox_profile_id)
    {
        push_issue(
            TrustBoundaryIssueCode::SandboxWeakening,
            FIELD_SANDBOX,
            "action attempted sandbox profile change",
            issue_codes,
            diagnostics,
        );
    }
}

fn monitor_tool(
    boundary: &TrustBoundaryDescriptor,
    action: &TrustBoundaryAction,
    issue_codes: &mut Vec<TrustBoundaryIssueCode>,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    let Some(tool_name) = action.tool_name.as_deref() else {
        push_issue(
            TrustBoundaryIssueCode::UnsanctionedTool,
            FIELD_SANCTIONED_TOOLS,
            "tool action omitted tool name",
            issue_codes,
            diagnostics,
        );
        return;
    };
    if !labels_contain(&boundary.sanctioned_tools, tool_name) {
        push_issue(
            TrustBoundaryIssueCode::UnsanctionedTool,
            FIELD_SANCTIONED_TOOLS,
            "action attempted unsanctioned tool call",
            issue_codes,
            diagnostics,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_issue(
    issue_code: TrustBoundaryIssueCode,
    field: &str,
    detail: &str,
    issue_codes: &mut Vec<TrustBoundaryIssueCode>,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    issue_codes.push(issue_code);
    diagnostics.push(TrustBoundaryDiagnostic::new(
        TrustBoundaryDiagnosticKind::MonitorDenied,
        field,
        detail,
    ));
}

fn require_blake3(
    fingerprint: &TrustBoundaryFingerprint,
    field: &str,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    if fingerprint.algorithm != BLAKE3_ALGORITHM || blank(&fingerprint.value_hex) {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::InvalidHash,
            field,
            "trust-boundary hashes must use non-empty BLAKE3 fingerprints",
        ));
    }
}

fn require_collection(
    values: &[String],
    field: &str,
    diagnostics: &mut Vec<TrustBoundaryDiagnostic>,
) {
    if values.is_empty() || values.iter().any(|value| blank(value)) {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::EmptyCollection,
            field,
            "required collection is empty or contains a blank entry",
        ));
    }
}

fn require_text(text: &str, field: &str, diagnostics: &mut Vec<TrustBoundaryDiagnostic>) {
    if blank(text) {
        diagnostics.push(TrustBoundaryDiagnostic::new(
            TrustBoundaryDiagnosticKind::MissingField,
            field,
            "required field is missing or blank",
        ));
    }
}

fn to_validation_result(
    diagnostics: Vec<TrustBoundaryDiagnostic>,
) -> TrustBoundaryValidationResult {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
