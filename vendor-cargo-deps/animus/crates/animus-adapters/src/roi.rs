//! Optional ROI recommendation report contracts.
//!
//! Hosts can collect evidence, score candidate work, and present ROI actions.
//! This module validates structured report data and reduces safe action
//! descriptors only; it never scans repositories, reads diffs, calls models,
//! opens UI, creates lifecycle changes, delegates work, or starts implementation.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units,
        tigerstyle::path_segment_repetition
    )
)]

use crate::artifact::Hash;
use crate::validation::{self, AdapterContractError, FieldName};

const REQUEST_ID_FIELD: FieldName = FieldName::new("roi_request_id");
const FOCUS_FIELD: FieldName = FieldName::new("roi_focus");
const BEST_NEXT_STEP_FIELD: FieldName = FieldName::new("roi_best_next_step");
const ACTION_FIELD: FieldName = FieldName::new("roi_recommended_action");
const REASON_FIELD: FieldName = FieldName::new("roi_reason");
const LIFECYCLE_STATUS_FIELD: FieldName = FieldName::new("roi_lifecycle_status");
const RUNNER_UP_FIELD: FieldName = FieldName::new("roi_runner_up");
const MISSING_CONTEXT_FIELD: FieldName = FieldName::new("roi_missing_context");
const EVIDENCE_LABEL_FIELD: FieldName = FieldName::new("roi_evidence_label");
const HASH_FIELD: FieldName = FieldName::new("roi_hash");
const RECEIPT_STATUS_FIELD: FieldName = FieldName::new("roi_receipt_status");
const RECEIPT_ISSUE_FIELD: FieldName = FieldName::new("roi_receipt_issue");

const MIN_SCORE: u8 = 0;
const MAX_SCORE: u8 = 100;
const MAX_TEXT_CHARS: usize = 512;
const MAX_REASON_CHARS: usize = 1_024;
const MAX_FIRST_ACTIONS: usize = 8;
const MAX_RUNNER_UPS: usize = 8;
const MAX_MISSING_CONTEXT: usize = 8;
const MAX_EVIDENCE_REFS: usize = 16;
const BEGIN_NOW_LABEL: &str = "begin-now";
const LIFECYCLE_FIRST_LABEL: &str = "lifecycle-first";
const INVESTIGATE_LABEL: &str = "investigate";
const DELEGATE_LABEL: &str = "delegate";
const DISMISS_LABEL: &str = "dismiss";
const NO_OP_LABEL: &str = "no-op";
const UNSAFE_ROI_MARKERS: &[&str] = &[
    "raw prompt",
    "provider payload",
    "provider json",
    "credential",
    "secret",
    "token",
    "/home/",
    "/tmp/",
    "c:\\",
    "\\users\\",
    "raw diff",
    "raw file body",
    "raw tool output",
    "hidden reasoning",
    "chain of thought",
    "unbounded transcript",
];

const _: () = assert!(MAX_TEXT_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_REASON_CHARS >= MAX_TEXT_CHARS);
const _: () = assert!(MAX_FIRST_ACTIONS > 0);
const _: () = assert!(MAX_RUNNER_UPS > 0);
const _: () = assert!(MAX_MISSING_CONTEXT > 0);
const _: () = assert!(MAX_EVIDENCE_REFS > 0);

/// Product behaviors that ROI contracts explicitly do not perform.
pub const PROHIBITED_ROI_EFFECTS: &[&str] = &[
    "repository scan",
    "diff read",
    "model scoring",
    "UI action selection",
    "lifecycle mutation",
    "Cairn creation",
    "task execution",
    "delegation",
    "default command registration",
];

/// ROI follow-up action kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoiRecommendedAction {
    /// Begin host-admitted work now.
    BeginNow,
    /// Route through a lifecycle change first.
    LifecycleFirst,
    /// Investigate before selecting implementation.
    Investigate,
    /// Delegate through a host-owned worker system.
    Delegate,
    /// Dismiss the recommendation.
    Dismiss,
    /// No action.
    NoOp,
}

/// Lifecycle route hint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoiLifecycleRouteKind {
    /// No lifecycle route is needed.
    None,
    /// Host should create or update lifecycle/spec work first.
    LifecycleFirst,
    /// Host can attach to existing lifecycle work.
    ExistingLifecycle,
}

/// ROI host service availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoiHostServiceStatus {
    /// Host injected an ROI evidence/scoring service.
    Available,
    /// No ROI service is injected.
    Missing,
}

/// ROI workflow planning status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoiWorkflowStatus {
    /// Host may run ROI evidence/scoring workflow.
    Ready,
    /// Host service is unavailable.
    ServiceUnavailable,
}

/// Action admission status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoiActionAdmissionStatus {
    /// Host admitted the action descriptor.
    Admitted,
    /// Host service is unavailable.
    ServiceUnavailable,
    /// Host policy declined the action.
    PolicyDenied,
}

/// ROI validation diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoiDiagnosticCode {
    /// Required field is empty.
    EmptyField,
    /// Field is too long.
    FieldTooLong,
    /// Field contains a NUL byte.
    NulCharacter,
    /// Collection is empty.
    EmptyCollection,
    /// Collection is too large.
    TooManyItems,
    /// Score is outside the accepted range.
    InvalidScore,
    /// Recommended action label is unsupported.
    UnsupportedAction,
    /// Metadata includes unsafe prompt/provider/path/reasoning content.
    UnsafeEvidence,
    /// Evidence hash is malformed.
    InvalidHash,
    /// ROI host service is missing.
    ServiceUnavailable,
}

/// Safe diagnostic emitted by ROI contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoiDiagnostic {
    code: RoiDiagnosticCode,
    subject_id: Option<Text>,
    message: Text,
}

/// Score dimensions for a recommendation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoiScoreInputDraft {
    /// Expected impact score.
    pub impact: u8,
    /// Expected effort score.
    pub effort: u8,
    /// Confidence score.
    pub confidence: u8,
    /// Unblock value score.
    pub unblock_value: u8,
    /// Risk reduction score.
    pub risk_reduction: u8,
}

/// Validated score dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoiScoreInput {
    impact: u8,
    effort: u8,
    confidence: u8,
    unblock_value: u8,
    risk_reduction: u8,
}

/// Runner-up recommendation draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoiRunnerUpDraft<'a> {
    /// Runner-up label.
    pub label: &'a str,
    /// Reason the runner-up was not selected.
    pub reason: &'a str,
    /// Runner-up score.
    pub score: RoiScoreInputDraft,
}

/// Validated runner-up recommendation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoiRunnerUp {
    label: Text,
    reason: Text,
    score: RoiScoreInput,
}

/// Missing-context draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoiMissingContextDraft<'a> {
    /// Missing context label.
    pub label: &'a str,
    /// Whether the missing context blocks confidence.
    pub blocking: bool,
}

/// Validated missing-context item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoiMissingContext {
    label: Text,
    blocking: bool,
}

/// Structured ROI report draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoiRecommendationReportDraft<'a> {
    /// Request id.
    pub request_id: &'a str,
    /// Recommendation focus.
    pub focus: &'a str,
    /// Best next step.
    pub best_next_step: &'a str,
    /// Score dimensions.
    pub score: RoiScoreInputDraft,
    /// Safe first action labels.
    pub first_actions: &'a [&'a str],
    /// Runner-up recommendations.
    pub runner_ups: &'a [RoiRunnerUpDraft<'a>],
    /// Lifecycle status label.
    pub lifecycle_status: &'a str,
    /// Recommended action label.
    pub recommended_action: &'a str,
    /// Safe rationale summary.
    pub reason: &'a str,
    /// Missing context items.
    pub missing_context: &'a [RoiMissingContextDraft<'a>],
    /// Safe evidence labels.
    pub evidence_labels: &'a [&'a str],
    /// Evidence hashes.
    pub evidence_hashes: &'a [&'a str],
}

/// Validated ROI recommendation report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoiRecommendationReport {
    request_id: Text,
    focus: Text,
    best_next_step: Text,
    score: RoiScoreInput,
    first_actions: Collection<Text>,
    runner_ups: Collection<RoiRunnerUp>,
    lifecycle_status: Text,
    recommended_action: RoiRecommendedAction,
    reason: Text,
    missing_context: Collection<RoiMissingContext>,
    evidence_labels: Collection<Text>,
    evidence_hashes: Collection<Text>,
}

/// Action descriptor reduced from a report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoiActionDescriptor {
    action: RoiRecommendedAction,
    lifecycle_route: RoiLifecycleRouteKind,
    objective: Text,
    reason: Text,
    evidence_hashes: Collection<Text>,
}

/// Action admission result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoiActionAdmission {
    status: RoiActionAdmissionStatus,
    descriptor: Option<RoiActionDescriptor>,
    diagnostic: Option<RoiDiagnostic>,
}

/// ROI workflow service plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoiWorkflowPlan {
    status: RoiWorkflowStatus,
    installed_commands: Collection<Text>,
    diagnostic: Option<RoiDiagnostic>,
}

/// ROI receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoiReceiptDraft<'a> {
    /// Request id.
    pub request_id: &'a str,
    /// Receipt status label.
    pub status: &'a str,
    /// Report hash.
    pub report_hash: &'a str,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
}

/// Validated ROI receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoiReceipt {
    request_id: Text,
    status: Text,
    report_hash: Text,
    issue_code: Option<Text>,
}

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;
type RoiResult<T> = Result<T, RoiDiagnostic>;

impl RoiDiagnostic {
    /// Diagnostic code.
    #[must_use]
    pub const fn code(&self) -> RoiDiagnosticCode {
        self.code
    }

    /// Subject id when available.
    #[must_use]
    pub fn subject_id(&self) -> Option<&str> {
        self.subject_id.as_deref()
    }

    /// Safe message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl RoiScoreInput {
    /// Impact score.
    #[must_use]
    pub const fn impact(self) -> u8 {
        self.impact
    }

    /// Effort score.
    #[must_use]
    pub const fn effort(self) -> u8 {
        self.effort
    }

    /// Confidence score.
    #[must_use]
    pub const fn confidence(self) -> u8 {
        self.confidence
    }

    /// Unblock value score.
    #[must_use]
    pub const fn unblock_value(self) -> u8 {
        self.unblock_value
    }

    /// Risk reduction score.
    #[must_use]
    pub const fn risk_reduction(self) -> u8 {
        self.risk_reduction
    }
}

impl RoiRunnerUp {
    /// Runner-up label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Runner-up reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Runner-up score.
    #[must_use]
    pub const fn score(&self) -> RoiScoreInput {
        self.score
    }
}

impl RoiMissingContext {
    /// Missing context label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Whether the context blocks confidence.
    #[must_use]
    pub const fn blocking(&self) -> bool {
        self.blocking
    }
}

impl RoiRecommendationReport {
    /// Request id.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Recommendation focus.
    #[must_use]
    pub fn focus(&self) -> &str {
        &self.focus
    }

    /// Best next step.
    #[must_use]
    pub fn best_next_step(&self) -> &str {
        &self.best_next_step
    }

    /// Score dimensions.
    #[must_use]
    pub const fn score(&self) -> RoiScoreInput {
        self.score
    }

    /// First action labels.
    #[must_use]
    pub fn first_actions(&self) -> &[Text] {
        &self.first_actions
    }

    /// Runner-up recommendations.
    #[must_use]
    pub fn runner_ups(&self) -> &[RoiRunnerUp] {
        &self.runner_ups
    }

    /// Lifecycle status.
    #[must_use]
    pub fn lifecycle_status(&self) -> &str {
        &self.lifecycle_status
    }

    /// Recommended action.
    #[must_use]
    pub const fn recommended_action(&self) -> RoiRecommendedAction {
        self.recommended_action
    }

    /// Reason summary.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Missing context.
    #[must_use]
    pub fn missing_context(&self) -> &[RoiMissingContext] {
        &self.missing_context
    }

    /// Evidence labels.
    #[must_use]
    pub fn evidence_labels(&self) -> &[Text] {
        &self.evidence_labels
    }

    /// Evidence hashes.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[Text] {
        &self.evidence_hashes
    }
}

impl RoiActionDescriptor {
    /// Action kind.
    #[must_use]
    pub const fn action(&self) -> RoiRecommendedAction {
        self.action
    }

    /// Lifecycle route hint.
    #[must_use]
    pub const fn lifecycle_route(&self) -> RoiLifecycleRouteKind {
        self.lifecycle_route
    }

    /// Objective label.
    #[must_use]
    pub fn objective(&self) -> &str {
        &self.objective
    }

    /// Safe reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Evidence hashes.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[Text] {
        &self.evidence_hashes
    }
}

impl RoiActionAdmission {
    /// Admission status.
    #[must_use]
    pub const fn status(&self) -> RoiActionAdmissionStatus {
        self.status
    }

    /// Admitted descriptor.
    #[must_use]
    pub fn descriptor(&self) -> Option<&RoiActionDescriptor> {
        self.descriptor.as_ref()
    }

    /// Diagnostic when rejected.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&RoiDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl RoiWorkflowPlan {
    /// Workflow status.
    #[must_use]
    pub const fn status(&self) -> RoiWorkflowStatus {
        self.status
    }

    /// Commands installed by reusable logic. This is always empty.
    #[must_use]
    pub fn installed_commands(&self) -> &[Text] {
        &self.installed_commands
    }

    /// Diagnostic when unavailable.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&RoiDiagnostic> {
        self.diagnostic.as_ref()
    }
}

impl RoiReceipt {
    /// Request id.
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Receipt status.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Report hash.
    #[must_use]
    pub fn report_hash(&self) -> &str {
        &self.report_hash
    }

    /// Issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }
}

/// Parses a recommended action label.
///
/// # Errors
///
/// Returns [`RoiDiagnostic`] when the action label is unsafe or unsupported.
pub fn parse_roi_action_label(label: &str) -> RoiResult<RoiRecommendedAction> {
    let label = safe_text(ACTION_FIELD, label, None)?;
    match label.as_str() {
        BEGIN_NOW_LABEL => Ok(RoiRecommendedAction::BeginNow),
        LIFECYCLE_FIRST_LABEL => Ok(RoiRecommendedAction::LifecycleFirst),
        INVESTIGATE_LABEL => Ok(RoiRecommendedAction::Investigate),
        DELEGATE_LABEL => Ok(RoiRecommendedAction::Delegate),
        DISMISS_LABEL => Ok(RoiRecommendedAction::Dismiss),
        NO_OP_LABEL => Ok(RoiRecommendedAction::NoOp),
        _ => Err(diagnostic(
            RoiDiagnosticCode::UnsupportedAction,
            None,
            "unsupported ROI action",
        )),
    }
}

/// Validates a structured ROI recommendation report.
///
/// # Errors
///
/// Returns [`RoiDiagnostic`] when required fields are missing, text/evidence is
/// unsafe, score dimensions are invalid, or action labels are unsupported.
pub fn validate_roi_report(
    draft: &RoiRecommendationReportDraft<'_>,
) -> RoiResult<RoiRecommendationReport> {
    let request_id = safe_text(REQUEST_ID_FIELD, draft.request_id, None)?;
    let focus = safe_bounded_text(
        FOCUS_FIELD,
        draft.focus,
        MAX_TEXT_CHARS,
        Some(request_id.as_str()),
    )?;
    let best_next_step = safe_bounded_text(
        BEST_NEXT_STEP_FIELD,
        draft.best_next_step,
        MAX_TEXT_CHARS,
        Some(request_id.as_str()),
    )?;
    let score = validate_score(draft.score)?;
    let first_actions = validate_text_list(
        "roi_first_actions",
        draft.first_actions,
        MAX_FIRST_ACTIONS,
        Some(request_id.as_str()),
    )?;
    validate_collection_limit("roi_runner_ups", draft.runner_ups.len(), MAX_RUNNER_UPS)?;
    let mut runner_ups = Collection::with_capacity(draft.runner_ups.len());
    for runner in draft.runner_ups {
        runner_ups.push(validate_runner_up(runner, request_id.as_str())?);
    }
    let lifecycle_status = safe_bounded_text(
        LIFECYCLE_STATUS_FIELD,
        draft.lifecycle_status,
        MAX_TEXT_CHARS,
        Some(request_id.as_str()),
    )?;
    let recommended_action = parse_roi_action_label(draft.recommended_action)?;
    let reason = safe_bounded_text(
        REASON_FIELD,
        draft.reason,
        MAX_REASON_CHARS,
        Some(request_id.as_str()),
    )?;
    validate_collection_limit(
        "roi_missing_context",
        draft.missing_context.len(),
        MAX_MISSING_CONTEXT,
    )?;
    let mut missing_context = Collection::with_capacity(draft.missing_context.len());
    for item in draft.missing_context {
        missing_context.push(validate_missing_context(item, request_id.as_str())?);
    }
    let evidence_labels = validate_text_list(
        "roi_evidence_labels",
        draft.evidence_labels,
        MAX_EVIDENCE_REFS,
        Some(request_id.as_str()),
    )?;
    let evidence_hashes = validate_hashes(draft.evidence_hashes)?;

    Ok(RoiRecommendationReport {
        request_id,
        focus,
        best_next_step,
        score,
        first_actions,
        runner_ups,
        lifecycle_status,
        recommended_action,
        reason,
        missing_context,
        evidence_labels,
        evidence_hashes,
    })
}

/// Reduces a report to an action descriptor without starting work.
#[must_use]
pub fn reduce_roi_action_descriptor(report: &RoiRecommendationReport) -> RoiActionDescriptor {
    let lifecycle_route = match report.recommended_action() {
        RoiRecommendedAction::LifecycleFirst => RoiLifecycleRouteKind::LifecycleFirst,
        RoiRecommendedAction::BeginNow
        | RoiRecommendedAction::Investigate
        | RoiRecommendedAction::Delegate
        | RoiRecommendedAction::Dismiss
        | RoiRecommendedAction::NoOp => RoiLifecycleRouteKind::None,
    };
    RoiActionDescriptor {
        action: report.recommended_action(),
        lifecycle_route,
        objective: Text::from(report.best_next_step()),
        reason: Text::from(report.reason()),
        evidence_hashes: report.evidence_hashes().to_vec(),
    }
}

/// Admits an ROI action descriptor through host policy and service availability.
#[must_use]
pub fn admit_roi_action(
    descriptor: &RoiActionDescriptor,
    service_status: RoiHostServiceStatus,
    host_policy_allows: bool,
) -> RoiActionAdmission {
    if service_status == RoiHostServiceStatus::Missing {
        return RoiActionAdmission {
            status: RoiActionAdmissionStatus::ServiceUnavailable,
            descriptor: None,
            diagnostic: Some(diagnostic(
                RoiDiagnosticCode::ServiceUnavailable,
                None,
                "ROI host service unavailable",
            )),
        };
    }
    if !host_policy_allows {
        return RoiActionAdmission {
            status: RoiActionAdmissionStatus::PolicyDenied,
            descriptor: None,
            diagnostic: Some(diagnostic(
                RoiDiagnosticCode::UnsupportedAction,
                None,
                "ROI action denied by host policy",
            )),
        };
    }
    RoiActionAdmission {
        status: RoiActionAdmissionStatus::Admitted,
        descriptor: Some(descriptor.clone()),
        diagnostic: None,
    }
}

/// Plans ROI workflow availability without registering a default command.
#[must_use]
pub fn plan_roi_workflow(service_status: RoiHostServiceStatus) -> RoiWorkflowPlan {
    if service_status == RoiHostServiceStatus::Missing {
        return RoiWorkflowPlan {
            status: RoiWorkflowStatus::ServiceUnavailable,
            installed_commands: Collection::new(),
            diagnostic: Some(diagnostic(
                RoiDiagnosticCode::ServiceUnavailable,
                None,
                "ROI evidence/scoring service unavailable",
            )),
        };
    }
    RoiWorkflowPlan {
        status: RoiWorkflowStatus::Ready,
        installed_commands: Collection::new(),
        diagnostic: None,
    }
}

/// Validates a safe ROI receipt.
///
/// # Errors
///
/// Returns [`RoiDiagnostic`] when metadata leaks unsafe content or the report
/// hash is malformed.
pub fn validate_roi_receipt(draft: &RoiReceiptDraft<'_>) -> RoiResult<RoiReceipt> {
    let request_id = safe_text(REQUEST_ID_FIELD, draft.request_id, None)?;
    let status = safe_text(
        RECEIPT_STATUS_FIELD,
        draft.status,
        Some(request_id.as_str()),
    )?;
    let report_hash = safe_hash(draft.report_hash)?;
    let issue_code = draft
        .issue_code
        .map(|issue| safe_text(RECEIPT_ISSUE_FIELD, issue, Some(request_id.as_str())))
        .transpose()?;
    Ok(RoiReceipt {
        request_id,
        status,
        report_hash,
        issue_code,
    })
}

fn validate_score(score: RoiScoreInputDraft) -> RoiResult<RoiScoreInput> {
    validate_score_value(score.impact)?;
    validate_score_value(score.effort)?;
    validate_score_value(score.confidence)?;
    validate_score_value(score.unblock_value)?;
    validate_score_value(score.risk_reduction)?;
    Ok(RoiScoreInput {
        impact: score.impact,
        effort: score.effort,
        confidence: score.confidence,
        unblock_value: score.unblock_value,
        risk_reduction: score.risk_reduction,
    })
}

fn validate_runner_up(draft: &RoiRunnerUpDraft<'_>, request_id: &str) -> RoiResult<RoiRunnerUp> {
    Ok(RoiRunnerUp {
        label: safe_bounded_text(
            RUNNER_UP_FIELD,
            draft.label,
            MAX_TEXT_CHARS,
            Some(request_id),
        )?,
        reason: safe_bounded_text(REASON_FIELD, draft.reason, MAX_TEXT_CHARS, Some(request_id))?,
        score: validate_score(draft.score)?,
    })
}

fn validate_missing_context(
    draft: &RoiMissingContextDraft<'_>,
    request_id: &str,
) -> RoiResult<RoiMissingContext> {
    Ok(RoiMissingContext {
        label: safe_bounded_text(
            MISSING_CONTEXT_FIELD,
            draft.label,
            MAX_TEXT_CHARS,
            Some(request_id),
        )?,
        blocking: draft.blocking,
    })
}

fn validate_score_value(value: u8) -> RoiResult<()> {
    if !(MIN_SCORE..=MAX_SCORE).contains(&value) {
        return Err(diagnostic(
            RoiDiagnosticCode::InvalidScore,
            None,
            "ROI score outside range",
        ));
    }
    Ok(())
}

fn validate_text_list(
    field: &'static str,
    values: &[&str],
    max_len: usize,
    request_id: Option<&str>,
) -> RoiResult<Collection<Text>> {
    validate_non_empty_bound(field, values.len(), max_len)?;
    let mut result = Collection::with_capacity(values.len());
    for value in values {
        result.push(safe_bounded_text(
            EVIDENCE_LABEL_FIELD,
            value,
            MAX_TEXT_CHARS,
            request_id,
        )?);
    }
    Ok(result)
}

fn validate_hashes(hashes: &[&str]) -> RoiResult<Collection<Text>> {
    validate_non_empty_bound("roi_evidence_hashes", hashes.len(), MAX_EVIDENCE_REFS)?;
    let mut result = Collection::with_capacity(hashes.len());
    for hash in hashes {
        result.push(safe_hash(hash)?);
    }
    Ok(result)
}

fn validate_non_empty_bound(field: &'static str, len: usize, max: usize) -> RoiResult<()> {
    if len == 0 {
        return Err(RoiDiagnostic {
            code: RoiDiagnosticCode::EmptyCollection,
            subject_id: None,
            message: Text::from(field),
        });
    }
    validate_collection_limit(field, len, max)
}

fn validate_collection_limit(field: &'static str, len: usize, max: usize) -> RoiResult<()> {
    if len > max {
        return Err(RoiDiagnostic {
            code: RoiDiagnosticCode::TooManyItems,
            subject_id: None,
            message: Text::from(field),
        });
    }
    Ok(())
}

fn safe_bounded_text(
    field: FieldName,
    value: &str,
    max_chars: usize,
    subject_id: Option<&str>,
) -> RoiResult<Text> {
    let text = safe_text(field, value, subject_id)?;
    if text.chars().count() > max_chars {
        return Err(diagnostic(
            RoiDiagnosticCode::FieldTooLong,
            subject_id,
            field.as_str(),
        ));
    }
    Ok(text)
}

fn safe_text(field: FieldName, value: &str, subject_id: Option<&str>) -> RoiResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value)
        .map_err(|error| map_adapter_error(&error))?;
    let lower = normalized.to_ascii_lowercase();
    if UNSAFE_ROI_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(diagnostic(
            RoiDiagnosticCode::UnsafeEvidence,
            subject_id,
            "ROI metadata contains unsafe evidence marker",
        ));
    }
    Ok(normalized)
}

fn safe_hash(value: &str) -> RoiResult<Text> {
    let normalized = validation::normalized_adapter_field(HASH_FIELD, value)
        .map_err(|error| map_adapter_error(&error))?;
    if Hash::new(&normalized).is_ok() {
        Ok(normalized)
    } else {
        Err(diagnostic(
            RoiDiagnosticCode::InvalidHash,
            None,
            HASH_FIELD.as_str(),
        ))
    }
}

fn map_adapter_error(error: &AdapterContractError) -> RoiDiagnostic {
    match error {
        AdapterContractError::EmptyField { field } => {
            diagnostic(RoiDiagnosticCode::EmptyField, None, field)
        }
        AdapterContractError::FieldTooLong { field, .. } => {
            diagnostic(RoiDiagnosticCode::FieldTooLong, None, field)
        }
        AdapterContractError::NulCharacter { field, .. } => {
            diagnostic(RoiDiagnosticCode::NulCharacter, None, field)
        }
        AdapterContractError::InvalidEvidenceHash { field, .. } => {
            diagnostic(RoiDiagnosticCode::InvalidHash, None, field)
        }
        AdapterContractError::EmptyCollection { field } => {
            diagnostic(RoiDiagnosticCode::EmptyCollection, None, field)
        }
        AdapterContractError::UnsupportedStatusState { field, .. }
        | AdapterContractError::UnsupportedStatusObservationKind { field, .. }
        | AdapterContractError::UnsupportedSessionReferenceKind { field, .. }
        | AdapterContractError::UnsupportedWorkspaceValue { field, .. }
        | AdapterContractError::UnsupportedDataspaceValue { field, .. }
        | AdapterContractError::UnsupportedRouterValue { field, .. } => {
            diagnostic(RoiDiagnosticCode::UnsafeEvidence, None, field)
        }
    }
}

fn diagnostic(code: RoiDiagnosticCode, subject_id: Option<&str>, message: &str) -> RoiDiagnostic {
    RoiDiagnostic {
        code,
        subject_id: subject_id.map(Text::from),
        message: Text::from(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUEST_ID: &str = "roi-alpha";
    const FOCUS: &str = "drain active lifecycle changes";
    const BEST_NEXT_STEP: &str = "implement the unblocked lifecycle adapter";
    const FIRST_ACTION: &str = "run focused validation";
    const RUNNER_UP: &str = "investigate adjacent adapter";
    const RUNNER_REASON: &str = "lower unblock value";
    const LIFECYCLE_STATUS: &str = "active-change-present";
    const REASON: &str = "high unblock value with bounded evidence";
    const MISSING_CONTEXT: &str = "latest validation receipt";
    const EVIDENCE_LABEL: &str = "cairn-validation-summary";
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const UNSAFE_EVIDENCE: &str = "provider JSON /home/user/private";
    const SCORE_HIGH: u8 = 90;
    const SCORE_MEDIUM: u8 = 50;
    const SCORE_LOW: u8 = 10;
    const SCORE_INVALID: u8 = 101;
    const SCORE: RoiScoreInputDraft = RoiScoreInputDraft {
        impact: SCORE_HIGH,
        effort: SCORE_MEDIUM,
        confidence: SCORE_HIGH,
        unblock_value: SCORE_HIGH,
        risk_reduction: SCORE_LOW,
    };
    static FIRST_ACTIONS: [&str; 1] = [FIRST_ACTION];
    static RUNNER_UPS: [RoiRunnerUpDraft<'static>; 1] = [RoiRunnerUpDraft {
        label: RUNNER_UP,
        reason: RUNNER_REASON,
        score: SCORE,
    }];
    static MISSING_CONTEXT_ITEMS: [RoiMissingContextDraft<'static>; 1] = [RoiMissingContextDraft {
        label: MISSING_CONTEXT,
        blocking: false,
    }];
    static EVIDENCE_LABELS: [&str; 1] = [EVIDENCE_LABEL];
    static EVIDENCE_HASHES: [&str; 1] = [HASH_A];

    fn report_draft(action: &'static str) -> RoiRecommendationReportDraft<'static> {
        RoiRecommendationReportDraft {
            request_id: REQUEST_ID,
            focus: FOCUS,
            best_next_step: BEST_NEXT_STEP,
            score: SCORE,
            first_actions: &FIRST_ACTIONS,
            runner_ups: &RUNNER_UPS,
            lifecycle_status: LIFECYCLE_STATUS,
            recommended_action: action,
            reason: REASON,
            missing_context: &MISSING_CONTEXT_ITEMS,
            evidence_labels: &EVIDENCE_LABELS,
            evidence_hashes: &EVIDENCE_HASHES,
        }
    }

    fn report(action: &'static str) -> RoiRecommendationReport {
        validate_roi_report(&report_draft(action)).unwrap()
    }

    #[test]
    fn valid_roi_report_names_best_next_step_without_side_effects() {
        // r[verify agent_harness_runtime.hermes_roi.recommendation]
        // r[verify agent_harness_runtime.hermes_roi.report_validation]
        let report = report(LIFECYCLE_FIRST_LABEL);
        let workflow = plan_roi_workflow(RoiHostServiceStatus::Missing);

        assert_eq!(report.request_id(), REQUEST_ID);
        assert_eq!(report.best_next_step(), BEST_NEXT_STEP);
        assert_eq!(
            report.recommended_action(),
            RoiRecommendedAction::LifecycleFirst
        );
        assert_eq!(report.runner_ups().len(), 1);
        assert_eq!(report.missing_context().len(), 1);
        assert_eq!(workflow.status(), RoiWorkflowStatus::ServiceUnavailable);
        assert!(workflow.installed_commands().is_empty());
        assert!(PROHIBITED_ROI_EFFECTS.contains(&"repository scan"));
    }

    #[test]
    fn unsafe_evidence_missing_best_step_and_unknown_actions_fail() {
        // r[verify agent_harness_runtime.hermes_roi.report_validation]
        let unsafe_error = validate_roi_report(&RoiRecommendationReportDraft {
            evidence_labels: &[UNSAFE_EVIDENCE],
            ..report_draft(BEGIN_NOW_LABEL)
        })
        .expect_err("unsafe evidence labels must fail");
        let missing_error = validate_roi_report(&RoiRecommendationReportDraft {
            best_next_step: "",
            ..report_draft(BEGIN_NOW_LABEL)
        })
        .expect_err("missing best next step must fail");
        let bad_score = validate_roi_report(&RoiRecommendationReportDraft {
            score: RoiScoreInputDraft {
                confidence: SCORE_INVALID,
                ..SCORE
            },
            ..report_draft(BEGIN_NOW_LABEL)
        })
        .expect_err("out-of-range score must fail");
        let unknown_action =
            parse_roi_action_label("auto-implement").expect_err("unknown action labels must fail");

        assert_eq!(unsafe_error.code(), RoiDiagnosticCode::UnsafeEvidence);
        assert_eq!(missing_error.code(), RoiDiagnosticCode::EmptyField);
        assert_eq!(bad_score.code(), RoiDiagnosticCode::InvalidScore);
        assert_eq!(unknown_action.code(), RoiDiagnosticCode::UnsupportedAction);
    }

    #[test]
    fn action_descriptors_require_host_admission_and_do_not_create_lifecycle() {
        // r[verify agent_harness_runtime.hermes_roi.action_descriptors]
        // r[verify agent_harness_runtime.hermes_roi.host_owned]
        let report = report(LIFECYCLE_FIRST_LABEL);
        let descriptor = reduce_roi_action_descriptor(&report);
        let missing = admit_roi_action(&descriptor, RoiHostServiceStatus::Missing, true);
        let denied = admit_roi_action(&descriptor, RoiHostServiceStatus::Available, false);
        let admitted = admit_roi_action(&descriptor, RoiHostServiceStatus::Available, true);

        assert_eq!(descriptor.action(), RoiRecommendedAction::LifecycleFirst);
        assert_eq!(
            descriptor.lifecycle_route(),
            RoiLifecycleRouteKind::LifecycleFirst
        );
        assert_eq!(
            missing.status(),
            RoiActionAdmissionStatus::ServiceUnavailable
        );
        assert_eq!(denied.status(), RoiActionAdmissionStatus::PolicyDenied);
        assert_eq!(admitted.status(), RoiActionAdmissionStatus::Admitted);
    }

    #[test]
    fn non_lifecycle_actions_reduce_to_neutral_routes() {
        // r[verify agent_harness_runtime.hermes_roi.action_descriptors]
        let begin = reduce_roi_action_descriptor(&report(BEGIN_NOW_LABEL));
        let investigate = reduce_roi_action_descriptor(&report(INVESTIGATE_LABEL));
        let no_op = reduce_roi_action_descriptor(&report(NO_OP_LABEL));

        assert_eq!(begin.lifecycle_route(), RoiLifecycleRouteKind::None);
        assert_eq!(investigate.action(), RoiRecommendedAction::Investigate);
        assert_eq!(no_op.action(), RoiRecommendedAction::NoOp);
    }

    #[test]
    fn receipts_are_hash_backed_and_reject_leaks() {
        // r[verify agent_harness_runtime.hermes_roi.verification]
        let receipt = validate_roi_receipt(&RoiReceiptDraft {
            request_id: REQUEST_ID,
            status: "rendered",
            report_hash: HASH_A,
            issue_code: Some("roi-rendered"),
        })
        .unwrap();
        let leaking = validate_roi_receipt(&RoiReceiptDraft {
            request_id: REQUEST_ID,
            status: "failed",
            report_hash: HASH_A,
            issue_code: Some("hidden reasoning leaked"),
        })
        .expect_err("receipt issue code must be safe");

        assert_eq!(receipt.report_hash(), HASH_A);
        assert_eq!(leaking.code(), RoiDiagnosticCode::UnsafeEvidence);
    }
}
