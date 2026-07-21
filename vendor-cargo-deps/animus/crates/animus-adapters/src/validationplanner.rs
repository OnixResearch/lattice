//! Optional scoped validation planner contracts.
//!
//! Hosts supply changed-artifact summaries and trusted validation profile labels.
//! This module selects safe plan items and reduces result summaries only; it never
//! walks files, queries VCS, discovers package managers, executes commands,
//! schedules retries, calls models, or starts auto-fix loops.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::non_trait_imports,
        tigerstyle::raw_arithmetic_overflow,
        tigerstyle::unbounded_collection_growth
    )
)]

use crate::artifact::Hash;
use crate::validation::{self, AdapterContractError, FieldName};

const ARTIFACT_ID_FIELD: FieldName = FieldName::new("validation_artifact_id");
const ARTIFACT_LABEL_FIELD: FieldName = FieldName::new("validation_artifact_label");
const WORKSPACE_LABEL_FIELD: FieldName = FieldName::new("validation_workspace_label");
const PACKAGE_LABEL_FIELD: FieldName = FieldName::new("validation_package_label");
const PROFILE_LABEL_FIELD: FieldName = FieldName::new("validation_profile_label");
const GATE_LABEL_FIELD: FieldName = FieldName::new("validation_gate_label");
const REASON_FIELD: FieldName = FieldName::new("validation_reason");
const RESULT_ID_FIELD: FieldName = FieldName::new("validation_result_id");
const FAILURE_SUMMARY_FIELD: FieldName = FieldName::new("validation_failure_summary");
const RECEIPT_STATUS_FIELD: FieldName = FieldName::new("validation_receipt_status");
const RECEIPT_ISSUE_FIELD: FieldName = FieldName::new("validation_receipt_issue");
const HASH_FIELD: FieldName = FieldName::new("validation_hash");

const SINGLE_ITEM_CAPACITY: usize = 1;
const MAX_ARTIFACTS: usize = 32;
const MAX_PROFILES: usize = 32;
const MAX_PLAN_ITEMS: usize = 32;
const MAX_TEXT_CHARS: usize = 512;
const MAX_FAILURE_SUMMARY_CHARS: usize = 1_024;
const MAX_BUDGET_UNITS: u16 = 16;
const MIN_BUDGET_UNITS: u16 = 1;
const STALE_ARTIFACT_EPOCH_DELTA: u32 = 3;
const WORKSPACE_WILDCARD: &str = "*";
const RAW_COMMAND_MARKERS: &[&str] = &[
    "bash -c",
    "sh -c",
    "cmd /c",
    "powershell",
    "cargo ",
    "npm ",
    "pytest",
    "nextest",
];
const UNSAFE_VALIDATION_MARKERS: &[&str] = &[
    "raw prompt",
    "provider payload",
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
    "unbounded transcript",
    "auto-fix",
];

const _: () = assert!(MAX_ARTIFACTS > 0);
const _: () = assert!(MAX_PROFILES > 0);
const _: () = assert!(MAX_PLAN_ITEMS > 0);
const _: () = assert!(MAX_TEXT_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_FAILURE_SUMMARY_CHARS >= MAX_TEXT_CHARS);
const _: () = assert!(MAX_BUDGET_UNITS >= MIN_BUDGET_UNITS);
const _: () = assert!(STALE_ARTIFACT_EPOCH_DELTA > 0);

/// Product behaviors that scoped validation planning explicitly does not perform.
pub const PROHIBITED_VALIDATION_PLANNER_EFFECTS: &[&str] = &[
    "filesystem walk",
    "VCS diff query",
    "manifest discovery",
    "process execution",
    "network access",
    "model call",
    "timer scheduling",
    "auto-fix loop",
];

/// Changed artifact kind supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ValidationArtifactKind {
    /// Rust-like source or manifest.
    Rust,
    /// Node/TypeScript-like source or manifest.
    Node,
    /// Python-like source or manifest.
    Python,
    /// Documentation artifact.
    Documentation,
    /// Configuration artifact.
    Configuration,
    /// Other host-supported artifact.
    Other,
}

/// Validation service availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationServiceStatus {
    /// Host injected validation planning/execution services.
    Available,
    /// Host did not inject validation services.
    Missing,
}

/// Plan status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationPlanStatus {
    /// Plan contains profile-label items.
    Ready,
    /// Service is missing.
    ServiceUnavailable,
    /// Planning failed closed.
    Rejected,
    /// No checks were selected.
    Empty,
}

/// Result summary status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationResultStatus {
    /// Validation passed.
    Passed,
    /// Validation failed.
    Failed,
    /// Validation was skipped.
    Skipped,
}

/// Follow-up intent status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationFollowUpStatus {
    /// No follow-up is needed.
    None,
    /// Advisory follow-up should be shown or queued by host policy.
    Advisory,
}

/// Validation diagnostic code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationPlannerDiagnosticCode {
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
    /// Metadata is unsafe.
    UnsafeMetadata,
    /// Raw command payload is not allowed.
    UnsafeCommand,
    /// Profile label is missing or unmatched.
    MissingProfile,
    /// Artifact summary is stale.
    StaleArtifact,
    /// Validation service is unavailable.
    ServiceUnavailable,
    /// Artifact kind is not supported by policy.
    UnsupportedArtifactKind,
    /// Budget is outside bounds.
    InvalidBudget,
    /// Hash is malformed.
    InvalidHash,
}

/// Safe diagnostic emitted by validation planner contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationPlannerDiagnostic {
    code: ValidationPlannerDiagnosticCode,
    subject_id: Option<Text>,
    message: Text,
}

/// Changed-artifact summary draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChangedArtifactSummaryDraft<'a> {
    /// Stable artifact id.
    pub artifact_id: &'a str,
    /// Safe artifact label.
    pub label: &'a str,
    /// Artifact kind.
    pub kind: ValidationArtifactKind,
    /// Workspace label.
    pub workspace_label: &'a str,
    /// Package label.
    pub package_label: &'a str,
    /// Content identity hash.
    pub content_hash: &'a str,
    /// Host observation epoch.
    pub observed_epoch: u32,
}

/// Validated changed-artifact summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedArtifactSummary {
    artifact_id: Text,
    label: Text,
    kind: ValidationArtifactKind,
    workspace_label: Text,
    package_label: Text,
    content_hash: Text,
    observed_epoch: u32,
}

/// Validation profile candidate draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationProfileCandidateDraft<'a> {
    /// Profile label or process-job profile id.
    pub profile_label: &'a str,
    /// Gate label.
    pub gate_label: &'a str,
    /// Artifact kind supported by this profile.
    pub artifact_kind: ValidationArtifactKind,
    /// Workspace label or wildcard.
    pub workspace_label: &'a str,
    /// Reason for selecting this profile.
    pub reason: &'a str,
    /// Budget units for the host execution profile.
    pub budget_units: u16,
    /// Whether the host trusts this profile.
    pub trusted: bool,
    /// Raw command payload, rejected by reusable logic when present.
    pub raw_command_payload: Option<&'a str>,
}

/// Validated validation profile candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationProfileCandidate {
    profile_label: Text,
    gate_label: Text,
    artifact_kind: ValidationArtifactKind,
    workspace_label: Text,
    reason: Text,
    budget_units: u16,
}

/// Validation policy draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationPlannerPolicyDraft<'a> {
    /// Current host epoch for freshness checks.
    pub current_epoch: u32,
    /// Supported artifact kinds.
    pub supported_kinds: &'a [ValidationArtifactKind],
    /// Optional trusted fallback profile label.
    pub fallback_profile_label: Option<&'a str>,
}

/// Validation plan item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationPlanItem {
    item_id: Text,
    artifact_id: Text,
    profile_label: Text,
    gate_label: Text,
    reason: Text,
    budget_units: u16,
}

/// Scoped validation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedValidationPlan {
    status: ValidationPlanStatus,
    items: Collection<ValidationPlanItem>,
    diagnostics: Collection<ValidationPlannerDiagnostic>,
}

/// Validation result summary draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationResultSummaryDraft<'a> {
    /// Plan item id.
    pub item_id: &'a str,
    /// Profile label.
    pub profile_label: &'a str,
    /// Result status.
    pub status: ValidationResultStatus,
    /// Bounded failure summary.
    pub failure_summary: Option<&'a str>,
    /// Output hash.
    pub output_hash: &'a str,
    /// Whether output was truncated by host.
    pub truncated: bool,
}

/// Validated result summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResultSummary {
    item_id: Text,
    profile_label: Text,
    status: ValidationResultStatus,
    failure_summary: Option<Text>,
    output_hash: Text,
    truncated: bool,
}

/// Follow-up intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationFollowUpIntent {
    status: ValidationFollowUpStatus,
    profile_label: Text,
    reason: Option<Text>,
    output_hash: Text,
    truncated: bool,
}

/// Validation planner receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationPlannerReceiptDraft<'a> {
    /// Plan id.
    pub plan_id: &'a str,
    /// Receipt status label.
    pub status: &'a str,
    /// Plan hash.
    pub plan_hash: &'a str,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
}

/// Validated receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationPlannerReceipt {
    plan_id: Text,
    status: Text,
    plan_hash: Text,
    issue_code: Option<Text>,
}

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;
type PlannerResult<T> = Result<T, ValidationPlannerDiagnostic>;

impl ValidationPlannerDiagnostic {
    /// Diagnostic code.
    #[must_use]
    pub const fn code(&self) -> ValidationPlannerDiagnosticCode {
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

impl ChangedArtifactSummary {
    /// Artifact id.
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    /// Artifact label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Artifact kind.
    #[must_use]
    pub const fn kind(&self) -> ValidationArtifactKind {
        self.kind
    }

    /// Workspace label.
    #[must_use]
    pub fn workspace_label(&self) -> &str {
        &self.workspace_label
    }

    /// Package label.
    #[must_use]
    pub fn package_label(&self) -> &str {
        &self.package_label
    }

    /// Content hash.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        &self.content_hash
    }

    /// Observation epoch.
    #[must_use]
    pub const fn observed_epoch(&self) -> u32 {
        self.observed_epoch
    }
}

impl ValidationProfileCandidate {
    /// Profile label.
    #[must_use]
    pub fn profile_label(&self) -> &str {
        &self.profile_label
    }

    /// Gate label.
    #[must_use]
    pub fn gate_label(&self) -> &str {
        &self.gate_label
    }

    /// Artifact kind.
    #[must_use]
    pub const fn artifact_kind(&self) -> ValidationArtifactKind {
        self.artifact_kind
    }

    /// Workspace label.
    #[must_use]
    pub fn workspace_label(&self) -> &str {
        &self.workspace_label
    }

    /// Reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Budget units.
    #[must_use]
    pub const fn budget_units(&self) -> u16 {
        self.budget_units
    }
}

impl ValidationPlanItem {
    /// Plan item id.
    #[must_use]
    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    /// Artifact id.
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    /// Profile label.
    #[must_use]
    pub fn profile_label(&self) -> &str {
        &self.profile_label
    }

    /// Gate label.
    #[must_use]
    pub fn gate_label(&self) -> &str {
        &self.gate_label
    }

    /// Reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Budget units.
    #[must_use]
    pub const fn budget_units(&self) -> u16 {
        self.budget_units
    }
}

impl ScopedValidationPlan {
    /// Plan status.
    #[must_use]
    pub const fn status(&self) -> ValidationPlanStatus {
        self.status
    }

    /// Plan items.
    #[must_use]
    pub fn items(&self) -> &[ValidationPlanItem] {
        &self.items
    }

    /// Diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[ValidationPlannerDiagnostic] {
        &self.diagnostics
    }
}

impl ValidationResultSummary {
    /// Plan item id.
    #[must_use]
    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    /// Profile label.
    #[must_use]
    pub fn profile_label(&self) -> &str {
        &self.profile_label
    }

    /// Result status.
    #[must_use]
    pub const fn status(&self) -> ValidationResultStatus {
        self.status
    }

    /// Failure summary.
    #[must_use]
    pub fn failure_summary(&self) -> Option<&str> {
        self.failure_summary.as_deref()
    }

    /// Output hash.
    #[must_use]
    pub fn output_hash(&self) -> &str {
        &self.output_hash
    }

    /// Whether output was truncated.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

impl ValidationFollowUpIntent {
    /// Follow-up status.
    #[must_use]
    pub const fn status(&self) -> ValidationFollowUpStatus {
        self.status
    }

    /// Profile label.
    #[must_use]
    pub fn profile_label(&self) -> &str {
        &self.profile_label
    }

    /// Reason.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// Output hash.
    #[must_use]
    pub fn output_hash(&self) -> &str {
        &self.output_hash
    }

    /// Whether output was truncated.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

impl ValidationPlannerReceipt {
    /// Plan id.
    #[must_use]
    pub fn plan_id(&self) -> &str {
        &self.plan_id
    }

    /// Receipt status.
    #[must_use]
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Plan hash.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        &self.plan_hash
    }

    /// Issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }
}

/// Builds a scoped validation plan from host-supplied facts.
#[must_use]
pub fn plan_scoped_validation(
    artifacts: &[ChangedArtifactSummaryDraft<'_>],
    profiles: &[ValidationProfileCandidateDraft<'_>],
    policy: &ValidationPlannerPolicyDraft<'_>,
    service_status: ValidationServiceStatus,
) -> ScopedValidationPlan {
    if service_status == ValidationServiceStatus::Missing {
        return rejected_plan(
            ValidationPlannerDiagnosticCode::ServiceUnavailable,
            "validation service unavailable",
        );
    }
    let prepared = prepare_inputs(artifacts, profiles, policy);
    let (mut artifacts, profiles) = match prepared {
        Ok(value) => value,
        Err(error) => {
            return ScopedValidationPlan {
                status: ValidationPlanStatus::Rejected,
                items: Collection::new(),
                diagnostics: collection_with_one(error),
            };
        }
    };
    artifacts.sort_by(|left, right| left.artifact_id().cmp(right.artifact_id()));
    let mut items = Collection::new();
    let mut diagnostics = Collection::new();
    for artifact in &artifacts {
        let selected = profiles
            .iter()
            .find(|profile| profile_matches(profile, artifact));
        let selected = selected.or_else(|| fallback_profile(policy, profiles.as_slice()));
        match selected {
            Some(profile) => items.push(plan_item(artifact, profile)),
            None => diagnostics.push(diagnostic(
                ValidationPlannerDiagnosticCode::MissingProfile,
                Some(artifact.artifact_id()),
                "no validation profile matched artifact",
            )),
        }
    }
    let status = if !diagnostics.is_empty() {
        ValidationPlanStatus::Rejected
    } else if items.is_empty() {
        ValidationPlanStatus::Empty
    } else {
        ValidationPlanStatus::Ready
    };
    ScopedValidationPlan {
        status,
        items,
        diagnostics,
    }
}

/// Validates a host-supplied validation result summary.
///
/// # Errors
///
/// Returns [`ValidationPlannerDiagnostic`] for unsafe summary text or malformed hashes.
pub fn validate_validation_result_summary(
    draft: &ValidationResultSummaryDraft<'_>,
) -> PlannerResult<ValidationResultSummary> {
    let item_id = safe_text(RESULT_ID_FIELD, draft.item_id, None)?;
    let profile_label = safe_text(
        PROFILE_LABEL_FIELD,
        draft.profile_label,
        Some(item_id.as_str()),
    )?;
    let failure_summary = draft
        .failure_summary
        .map(|summary| {
            safe_bounded_text(
                FAILURE_SUMMARY_FIELD,
                summary,
                MAX_FAILURE_SUMMARY_CHARS,
                Some(item_id.as_str()),
            )
        })
        .transpose()?;
    let output_hash = safe_hash(draft.output_hash)?;
    Ok(ValidationResultSummary {
        item_id,
        profile_label,
        status: draft.status,
        failure_summary,
        output_hash,
        truncated: draft.truncated,
    })
}

/// Reduces validation result summaries to advisory follow-up intents only.
#[must_use]
pub fn reduce_validation_follow_up(result: &ValidationResultSummary) -> ValidationFollowUpIntent {
    match result.status() {
        ValidationResultStatus::Passed | ValidationResultStatus::Skipped => {
            ValidationFollowUpIntent {
                status: ValidationFollowUpStatus::None,
                profile_label: Text::from(result.profile_label()),
                reason: None,
                output_hash: Text::from(result.output_hash()),
                truncated: result.truncated(),
            }
        }
        ValidationResultStatus::Failed => ValidationFollowUpIntent {
            status: ValidationFollowUpStatus::Advisory,
            profile_label: Text::from(result.profile_label()),
            reason: result.failure_summary().map(Text::from),
            output_hash: Text::from(result.output_hash()),
            truncated: result.truncated(),
        },
    }
}

/// Validates a scoped validation planner receipt.
///
/// # Errors
///
/// Returns [`ValidationPlannerDiagnostic`] when metadata leaks unsafe content or
/// the plan hash is malformed.
pub fn validate_validation_planner_receipt(
    draft: &ValidationPlannerReceiptDraft<'_>,
) -> PlannerResult<ValidationPlannerReceipt> {
    let plan_id = safe_text(RESULT_ID_FIELD, draft.plan_id, None)?;
    let status = safe_text(RECEIPT_STATUS_FIELD, draft.status, Some(plan_id.as_str()))?;
    let plan_hash = safe_hash(draft.plan_hash)?;
    let issue_code = draft
        .issue_code
        .map(|issue| safe_text(RECEIPT_ISSUE_FIELD, issue, Some(plan_id.as_str())))
        .transpose()?;
    Ok(ValidationPlannerReceipt {
        plan_id,
        status,
        plan_hash,
        issue_code,
    })
}

fn prepare_inputs(
    artifacts: &[ChangedArtifactSummaryDraft<'_>],
    profiles: &[ValidationProfileCandidateDraft<'_>],
    policy: &ValidationPlannerPolicyDraft<'_>,
) -> PlannerResult<(
    Collection<ChangedArtifactSummary>,
    Collection<ValidationProfileCandidate>,
)> {
    validate_non_empty_bound("validation_artifacts", artifacts.len(), MAX_ARTIFACTS)?;
    validate_non_empty_bound("validation_profiles", profiles.len(), MAX_PROFILES)?;
    let mut validated_artifacts = Collection::with_capacity(artifacts.len());
    for artifact in artifacts {
        validated_artifacts.push(validate_artifact(artifact, policy)?);
    }
    let mut validated_profiles = Collection::with_capacity(profiles.len());
    for profile in profiles {
        validated_profiles.push(validate_profile(profile)?);
    }
    Ok((validated_artifacts, validated_profiles))
}

fn validate_artifact(
    draft: &ChangedArtifactSummaryDraft<'_>,
    policy: &ValidationPlannerPolicyDraft<'_>,
) -> PlannerResult<ChangedArtifactSummary> {
    if !policy.supported_kinds.contains(&draft.kind) {
        return Err(diagnostic(
            ValidationPlannerDiagnosticCode::UnsupportedArtifactKind,
            None,
            "artifact kind is not supported by policy",
        ));
    }
    let artifact_id = safe_text(ARTIFACT_ID_FIELD, draft.artifact_id, None)?;
    if is_stale(draft.observed_epoch, policy.current_epoch) {
        return Err(diagnostic(
            ValidationPlannerDiagnosticCode::StaleArtifact,
            Some(artifact_id.as_str()),
            "changed-artifact summary is stale",
        ));
    }
    Ok(ChangedArtifactSummary {
        artifact_id,
        label: safe_text(ARTIFACT_LABEL_FIELD, draft.label, None)?,
        kind: draft.kind,
        workspace_label: safe_text(WORKSPACE_LABEL_FIELD, draft.workspace_label, None)?,
        package_label: safe_text(PACKAGE_LABEL_FIELD, draft.package_label, None)?,
        content_hash: safe_hash(draft.content_hash)?,
        observed_epoch: draft.observed_epoch,
    })
}

fn validate_profile(
    draft: &ValidationProfileCandidateDraft<'_>,
) -> PlannerResult<ValidationProfileCandidate> {
    if !draft.trusted {
        return Err(diagnostic(
            ValidationPlannerDiagnosticCode::MissingProfile,
            None,
            "validation profile is not trusted",
        ));
    }
    if let Some(command) = draft.raw_command_payload {
        reject_raw_command(command)?;
    }
    validate_budget(draft.budget_units)?;
    Ok(ValidationProfileCandidate {
        profile_label: safe_text(PROFILE_LABEL_FIELD, draft.profile_label, None)?,
        gate_label: safe_text(GATE_LABEL_FIELD, draft.gate_label, None)?,
        artifact_kind: draft.artifact_kind,
        workspace_label: safe_text(WORKSPACE_LABEL_FIELD, draft.workspace_label, None)?,
        reason: safe_bounded_text(REASON_FIELD, draft.reason, MAX_TEXT_CHARS, None)?,
        budget_units: draft.budget_units,
    })
}

fn reject_raw_command(command: &str) -> PlannerResult<()> {
    let normalized = validation::normalized_adapter_field(PROFILE_LABEL_FIELD, command)
        .map_err(|error| map_adapter_error(&error))?;
    let lower = normalized.to_ascii_lowercase();
    if RAW_COMMAND_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(diagnostic(
            ValidationPlannerDiagnosticCode::UnsafeCommand,
            None,
            "raw validation command payload is not allowed",
        ));
    }
    Err(diagnostic(
        ValidationPlannerDiagnosticCode::UnsafeCommand,
        None,
        "raw validation command payload is not allowed",
    ))
}

fn plan_item(
    artifact: &ChangedArtifactSummary,
    profile: &ValidationProfileCandidate,
) -> ValidationPlanItem {
    let mut item_id =
        Text::with_capacity(artifact.artifact_id().len() + profile.profile_label().len() + 1);
    item_id.push_str(artifact.artifact_id());
    item_id.push(':');
    item_id.push_str(profile.profile_label());
    ValidationPlanItem {
        item_id,
        artifact_id: Text::from(artifact.artifact_id()),
        profile_label: Text::from(profile.profile_label()),
        gate_label: Text::from(profile.gate_label()),
        reason: Text::from(profile.reason()),
        budget_units: profile.budget_units(),
    }
}

fn profile_matches(
    profile: &ValidationProfileCandidate,
    artifact: &ChangedArtifactSummary,
) -> bool {
    profile.artifact_kind() == artifact.kind()
        && (profile.workspace_label() == artifact.workspace_label()
            || profile.workspace_label() == WORKSPACE_WILDCARD)
}

fn fallback_profile<'a>(
    policy: &ValidationPlannerPolicyDraft<'_>,
    profiles: &'a [ValidationProfileCandidate],
) -> Option<&'a ValidationProfileCandidate> {
    let label = policy.fallback_profile_label?;
    profiles
        .iter()
        .find(|profile| profile.profile_label() == label)
}

fn rejected_plan(code: ValidationPlannerDiagnosticCode, message: &str) -> ScopedValidationPlan {
    ScopedValidationPlan {
        status: ValidationPlanStatus::ServiceUnavailable,
        items: Collection::new(),
        diagnostics: collection_with_one(diagnostic(code, None, message)),
    }
}

fn collection_with_one<T>(item: T) -> Collection<T> {
    let mut values = Collection::with_capacity(SINGLE_ITEM_CAPACITY);
    values.push(item);
    values
}

fn validate_budget(value: u16) -> PlannerResult<()> {
    if !(MIN_BUDGET_UNITS..=MAX_BUDGET_UNITS).contains(&value) {
        return Err(diagnostic(
            ValidationPlannerDiagnosticCode::InvalidBudget,
            None,
            "validation budget outside bounds",
        ));
    }
    Ok(())
}

fn is_stale(observed_epoch: u32, current_epoch: u32) -> bool {
    current_epoch.saturating_sub(observed_epoch) > STALE_ARTIFACT_EPOCH_DELTA
}

fn validate_non_empty_bound(field: &'static str, len: usize, max: usize) -> PlannerResult<()> {
    if len == 0 {
        return Err(ValidationPlannerDiagnostic {
            code: ValidationPlannerDiagnosticCode::EmptyCollection,
            subject_id: None,
            message: Text::from(field),
        });
    }
    if len > max {
        return Err(ValidationPlannerDiagnostic {
            code: ValidationPlannerDiagnosticCode::TooManyItems,
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
) -> PlannerResult<Text> {
    let text = safe_text(field, value, subject_id)?;
    if text.chars().count() > max_chars {
        return Err(diagnostic(
            ValidationPlannerDiagnosticCode::FieldTooLong,
            subject_id,
            field.as_str(),
        ));
    }
    Ok(text)
}

fn safe_text(field: FieldName, value: &str, subject_id: Option<&str>) -> PlannerResult<Text> {
    let normalized = validation::normalized_adapter_field(field, value)
        .map_err(|error| map_adapter_error(&error))?;
    let lower = normalized.to_ascii_lowercase();
    if UNSAFE_VALIDATION_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Err(diagnostic(
            ValidationPlannerDiagnosticCode::UnsafeMetadata,
            subject_id,
            "validation metadata contains unsafe marker",
        ));
    }
    Ok(normalized)
}

fn safe_hash(value: &str) -> PlannerResult<Text> {
    let normalized = validation::normalized_adapter_field(HASH_FIELD, value)
        .map_err(|error| map_adapter_error(&error))?;
    if Hash::new(&normalized).is_ok() {
        Ok(normalized)
    } else {
        Err(diagnostic(
            ValidationPlannerDiagnosticCode::InvalidHash,
            None,
            HASH_FIELD.as_str(),
        ))
    }
}

fn map_adapter_error(error: &AdapterContractError) -> ValidationPlannerDiagnostic {
    match error {
        AdapterContractError::EmptyField { field } => {
            diagnostic(ValidationPlannerDiagnosticCode::EmptyField, None, field)
        }
        AdapterContractError::FieldTooLong { field, .. } => {
            diagnostic(ValidationPlannerDiagnosticCode::FieldTooLong, None, field)
        }
        AdapterContractError::NulCharacter { field, .. } => {
            diagnostic(ValidationPlannerDiagnosticCode::NulCharacter, None, field)
        }
        AdapterContractError::InvalidEvidenceHash { field, .. } => {
            diagnostic(ValidationPlannerDiagnosticCode::InvalidHash, None, field)
        }
        AdapterContractError::EmptyCollection { field } => diagnostic(
            ValidationPlannerDiagnosticCode::EmptyCollection,
            None,
            field,
        ),
        AdapterContractError::UnsupportedStatusState { field, .. }
        | AdapterContractError::UnsupportedStatusObservationKind { field, .. }
        | AdapterContractError::UnsupportedSessionReferenceKind { field, .. }
        | AdapterContractError::UnsupportedWorkspaceValue { field, .. }
        | AdapterContractError::UnsupportedDataspaceValue { field, .. }
        | AdapterContractError::UnsupportedRouterValue { field, .. } => {
            diagnostic(ValidationPlannerDiagnosticCode::UnsafeMetadata, None, field)
        }
    }
}

fn diagnostic(
    code: ValidationPlannerDiagnosticCode,
    subject_id: Option<&str>,
    message: &str,
) -> ValidationPlannerDiagnostic {
    ValidationPlannerDiagnostic {
        code,
        subject_id: subject_id.map(Text::from),
        message: Text::from(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTIFACT_RUST: &str = "artifact-rust";
    const ARTIFACT_DOC: &str = "artifact-doc";
    const LABEL_RUST: &str = "crates/adapter/lib.rs";
    const LABEL_DOC: &str = "docs/contracts.md";
    const WORKSPACE: &str = "animus";
    const PACKAGE: &str = "animus-adapters";
    const PROFILE_RUST: &str = "cargo-test-adapters";
    const PROFILE_DOC: &str = "docs-lint";
    const PROFILE_FALLBACK: &str = "fallback-smoke";
    const GATE_RUST: &str = "rust-focused";
    const GATE_DOC: &str = "docs-focused";
    const REASON_RUST: &str = "rust-like artifact changed";
    const REASON_DOC: &str = "documentation artifact changed";
    const FAILURE_SUMMARY: &str = "adapter validation failed with bounded summary";
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const CURRENT_EPOCH: u32 = 7;
    const OBSERVED_EPOCH: u32 = 7;
    const STALE_EPOCH: u32 = 1;
    const BUDGET: u16 = 4;

    fn artifact(
        artifact_id: &'static str,
        label: &'static str,
        kind: ValidationArtifactKind,
    ) -> ChangedArtifactSummaryDraft<'static> {
        ChangedArtifactSummaryDraft {
            artifact_id,
            label,
            kind,
            workspace_label: WORKSPACE,
            package_label: PACKAGE,
            content_hash: HASH_A,
            observed_epoch: OBSERVED_EPOCH,
        }
    }

    fn profile(
        profile_label: &'static str,
        gate_label: &'static str,
        kind: ValidationArtifactKind,
        reason: &'static str,
    ) -> ValidationProfileCandidateDraft<'static> {
        ValidationProfileCandidateDraft {
            profile_label,
            gate_label,
            artifact_kind: kind,
            workspace_label: WORKSPACE,
            reason,
            budget_units: BUDGET,
            trusted: true,
            raw_command_payload: None,
        }
    }

    fn policy() -> ValidationPlannerPolicyDraft<'static> {
        static KINDS: [ValidationArtifactKind; 5] = [
            ValidationArtifactKind::Rust,
            ValidationArtifactKind::Node,
            ValidationArtifactKind::Python,
            ValidationArtifactKind::Documentation,
            ValidationArtifactKind::Configuration,
        ];
        ValidationPlannerPolicyDraft {
            current_epoch: CURRENT_EPOCH,
            supported_kinds: &KINDS,
            fallback_profile_label: Some(PROFILE_FALLBACK),
        }
    }

    #[test]
    fn scoped_planning_selects_profile_labels_without_commands() {
        // r[verify agent_harness_runtime.hermes_validation_planner.plan]
        // r[verify agent_harness_runtime.hermes_validation_planner.pure_planner]
        let artifacts = [
            artifact(
                ARTIFACT_DOC,
                LABEL_DOC,
                ValidationArtifactKind::Documentation,
            ),
            artifact(ARTIFACT_RUST, LABEL_RUST, ValidationArtifactKind::Rust),
        ];
        let profiles = [
            profile(
                PROFILE_RUST,
                GATE_RUST,
                ValidationArtifactKind::Rust,
                REASON_RUST,
            ),
            profile(
                PROFILE_DOC,
                GATE_DOC,
                ValidationArtifactKind::Documentation,
                REASON_DOC,
            ),
        ];
        let plan = plan_scoped_validation(
            &artifacts,
            &profiles,
            &policy(),
            ValidationServiceStatus::Available,
        );

        assert_eq!(plan.status(), ValidationPlanStatus::Ready);
        assert_eq!(plan.items().len(), 2);
        assert_eq!(plan.items()[0].artifact_id(), ARTIFACT_DOC);
        assert_eq!(plan.items()[1].artifact_id(), ARTIFACT_RUST);
        assert!(
            plan.items()
                .iter()
                .any(|item| item.profile_label() == PROFILE_RUST)
        );
        assert!(PROHIBITED_VALIDATION_PLANNER_EFFECTS.contains(&"process execution"));
    }

    #[test]
    fn fallback_profile_is_deterministic_for_unmatched_supported_artifacts() {
        // r[verify agent_harness_runtime.hermes_validation_planner.pure_planner]
        let artifacts = [artifact(
            ARTIFACT_RUST,
            LABEL_RUST,
            ValidationArtifactKind::Rust,
        )];
        let profiles = [ValidationProfileCandidateDraft {
            profile_label: PROFILE_FALLBACK,
            gate_label: "fallback-gate",
            artifact_kind: ValidationArtifactKind::Documentation,
            workspace_label: WORKSPACE_WILDCARD,
            reason: "conservative fallback",
            budget_units: BUDGET,
            trusted: true,
            raw_command_payload: None,
        }];
        let first = plan_scoped_validation(
            &artifacts,
            &profiles,
            &policy(),
            ValidationServiceStatus::Available,
        );
        let second = plan_scoped_validation(
            &artifacts,
            &profiles,
            &policy(),
            ValidationServiceStatus::Available,
        );

        assert_eq!(first, second);
        assert_eq!(first.items()[0].profile_label(), PROFILE_FALLBACK);
    }

    #[test]
    fn missing_service_raw_command_and_stale_artifact_fail_closed() {
        // r[verify agent_harness_runtime.hermes_validation_planner.fail_closed]
        let artifacts = [artifact(
            ARTIFACT_RUST,
            LABEL_RUST,
            ValidationArtifactKind::Rust,
        )];
        let profiles = [profile(
            PROFILE_RUST,
            GATE_RUST,
            ValidationArtifactKind::Rust,
            REASON_RUST,
        )];
        let missing = plan_scoped_validation(
            &artifacts,
            &profiles,
            &policy(),
            ValidationServiceStatus::Missing,
        );
        let raw_profile = [ValidationProfileCandidateDraft {
            raw_command_payload: Some("bash -c cargo test"),
            ..profile(
                PROFILE_RUST,
                GATE_RUST,
                ValidationArtifactKind::Rust,
                REASON_RUST,
            )
        }];
        let raw = plan_scoped_validation(
            &artifacts,
            &raw_profile,
            &policy(),
            ValidationServiceStatus::Available,
        );
        let stale_artifacts = [ChangedArtifactSummaryDraft {
            observed_epoch: STALE_EPOCH,
            ..artifact(ARTIFACT_RUST, LABEL_RUST, ValidationArtifactKind::Rust)
        }];
        let stale = plan_scoped_validation(
            &stale_artifacts,
            &profiles,
            &policy(),
            ValidationServiceStatus::Available,
        );

        assert_eq!(missing.status(), ValidationPlanStatus::ServiceUnavailable);
        assert_eq!(raw.status(), ValidationPlanStatus::Rejected);
        assert_eq!(
            raw.diagnostics()[0].code(),
            ValidationPlannerDiagnosticCode::UnsafeCommand
        );
        assert_eq!(
            stale.diagnostics()[0].code(),
            ValidationPlannerDiagnosticCode::StaleArtifact
        );
    }

    #[test]
    fn failed_results_reduce_to_advisory_follow_up_only() {
        // r[verify agent_harness_runtime.hermes_validation_planner.followup]
        let result = validate_validation_result_summary(&ValidationResultSummaryDraft {
            item_id: "artifact-rust:cargo-test-adapters",
            profile_label: PROFILE_RUST,
            status: ValidationResultStatus::Failed,
            failure_summary: Some(FAILURE_SUMMARY),
            output_hash: HASH_A,
            truncated: true,
        })
        .unwrap();
        let follow_up = reduce_validation_follow_up(&result);
        let passed = validate_validation_result_summary(&ValidationResultSummaryDraft {
            status: ValidationResultStatus::Passed,
            failure_summary: None,
            output_hash: HASH_B,
            truncated: false,
            ..ValidationResultSummaryDraft {
                item_id: "artifact-rust:cargo-test-adapters",
                profile_label: PROFILE_RUST,
                status: ValidationResultStatus::Failed,
                failure_summary: Some(FAILURE_SUMMARY),
                output_hash: HASH_A,
                truncated: true,
            }
        })
        .unwrap();
        let no_follow_up = reduce_validation_follow_up(&passed);

        assert_eq!(follow_up.status(), ValidationFollowUpStatus::Advisory);
        assert_eq!(follow_up.profile_label(), PROFILE_RUST);
        assert_eq!(follow_up.output_hash(), HASH_A);
        assert_eq!(no_follow_up.status(), ValidationFollowUpStatus::None);
    }

    #[test]
    fn unsafe_result_summary_and_receipt_leaks_are_rejected() {
        // r[verify agent_harness_runtime.hermes_validation_planner.verification]
        let unsafe_result = validate_validation_result_summary(&ValidationResultSummaryDraft {
            item_id: "artifact-rust:cargo-test-adapters",
            profile_label: PROFILE_RUST,
            status: ValidationResultStatus::Failed,
            failure_summary: Some("raw tool output leaked"),
            output_hash: HASH_A,
            truncated: true,
        })
        .expect_err("unsafe result summaries must fail");
        let receipt = validate_validation_planner_receipt(&ValidationPlannerReceiptDraft {
            plan_id: "plan-alpha",
            status: "planned",
            plan_hash: HASH_A,
            issue_code: Some("validation-planned"),
        })
        .unwrap();
        let leaking_receipt = validate_validation_planner_receipt(&ValidationPlannerReceiptDraft {
            plan_id: "plan-alpha",
            status: "failed",
            plan_hash: HASH_A,
            issue_code: Some("raw prompt leaked"),
        })
        .expect_err("receipt issue code must be safe");

        assert_eq!(
            unsafe_result.code(),
            ValidationPlannerDiagnosticCode::UnsafeMetadata
        );
        assert_eq!(receipt.plan_hash(), HASH_A);
        assert_eq!(
            leaking_receipt.code(),
            ValidationPlannerDiagnosticCode::UnsafeMetadata
        );
    }
}
