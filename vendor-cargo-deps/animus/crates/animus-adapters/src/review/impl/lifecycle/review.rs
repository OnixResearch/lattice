const REVIEW_LIFECYCLE_HASH_DOMAIN: &str = "animus.review-lifecycle.v1";
const REVIEW_LIFECYCLE_RECEIPT_HASH_DOMAIN: &str = "animus.review-lifecycle.receipt.v1";
const MAX_REVIEW_EVIDENCE_ITEMS: usize = 16;
const MAX_REVIEW_RAILS: usize = 16;
const MAX_REVIEW_ATTEMPTS: usize = 8;
const MIN_REVIEW_ATTEMPT_BUDGET_UNITS: u16 = 1;

/// Kind of subject under independent done-review.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewLifecycleSubjectKind {
    /// A lifecycle change package.
    Change,
    /// A source commit or patch set.
    Commit,
    /// A release or release candidate.
    Release,
    /// A bounded work item.
    WorkItem,
}

impl ReviewLifecycleSubjectKind {
    /// Returns the stable subject-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Change => "change",
            Self::Commit => "commit",
            Self::Release => "release",
            Self::WorkItem => "work-item",
        }
    }
}

/// Borrowed review subject metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewLifecycleSubjectDraft<'a> {
    /// Safe subject identity.
    pub subject_id: &'a str,
    /// Subject kind.
    pub kind: ReviewLifecycleSubjectKind,
    /// Safe owner or area label.
    pub owner_label: &'a str,
    /// Safe lifecycle stage label.
    pub stage_label: &'a str,
}

/// Validated review subject metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewLifecycleSubject {
    subject_id: String,
    kind: ReviewLifecycleSubjectKind,
    owner_label: String,
    stage_label: String,
}

impl ReviewLifecycleSubject {
    /// Returns the safe subject identity.
    #[must_use]
    pub fn subject_id(&self) -> &str {
        self.subject_id.as_str()
    }

    /// Returns the subject kind.
    #[must_use]
    pub const fn kind(&self) -> ReviewLifecycleSubjectKind {
        self.kind
    }

    /// Returns the safe owner label.
    #[must_use]
    pub fn owner_label(&self) -> &str {
        self.owner_label.as_str()
    }

    /// Returns the safe lifecycle stage label.
    #[must_use]
    pub fn stage_label(&self) -> &str {
        self.stage_label.as_str()
    }
}

/// Kind of bounded evidence supplied for done-review.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewEvidenceKind {
    /// Lifecycle validation or gate evidence.
    LifecycleGate,
    /// Test or check evidence.
    FocusedCheck,
    /// Completion rail result evidence.
    CompletionRail,
    /// Documentation or architecture note evidence.
    Documentation,
    /// Safe receipt evidence.
    Receipt,
}

impl ReviewEvidenceKind {
    /// Returns the stable evidence-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LifecycleGate => "lifecycle-gate",
            Self::FocusedCheck => "focused-check",
            Self::CompletionRail => "completion-rail",
            Self::Documentation => "documentation",
            Self::Receipt => "receipt",
        }
    }
}

/// Redaction state for review evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewEvidenceRedaction {
    /// Evidence is a safe summary.
    SafeSummary,
    /// Evidence is represented by hashes only.
    HashOnly,
}

impl ReviewEvidenceRedaction {
    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SafeSummary => "safe-summary",
            Self::HashOnly => "hash-only",
        }
    }
}

/// Borrowed bounded review evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewEvidenceDraft<'a> {
    /// Safe evidence identity.
    pub evidence_id: &'a str,
    /// Evidence kind.
    pub kind: ReviewEvidenceKind,
    /// Safe evidence label.
    pub label: &'a str,
    /// BLAKE3 identity of the bounded evidence artifact or summary.
    pub evidence_hash: &'a str,
    /// Bounded item count represented by this evidence.
    pub item_count: usize,
    /// Redaction state.
    pub redaction: ReviewEvidenceRedaction,
}

/// Validated bounded review evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewEvidence {
    evidence_id: String,
    kind: ReviewEvidenceKind,
    label: String,
    evidence_hash: String,
    item_count: usize,
    redaction: ReviewEvidenceRedaction,
}

impl ReviewEvidence {
    /// Returns the safe evidence identity.
    #[must_use]
    pub fn evidence_id(&self) -> &str {
        self.evidence_id.as_str()
    }

    /// Returns the evidence kind.
    #[must_use]
    pub const fn kind(&self) -> ReviewEvidenceKind {
        self.kind
    }

    /// Returns the safe evidence label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the BLAKE3 evidence identity.
    #[must_use]
    pub fn evidence_hash(&self) -> &str {
        self.evidence_hash.as_str()
    }

    /// Returns the bounded item count.
    #[must_use]
    pub const fn item_count(&self) -> usize {
        self.item_count
    }

    /// Returns the redaction state.
    #[must_use]
    pub const fn redaction(&self) -> ReviewEvidenceRedaction {
        self.redaction
    }
}

/// Completion rail requirement level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewRailRequirement {
    /// Rail must pass for a completion claim.
    Required,
    /// Rail is advisory evidence.
    Advisory,
}

impl ReviewRailRequirement {
    /// Returns the stable requirement label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Advisory => "advisory",
        }
    }
}

/// Borrowed completion rail descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewRailDescriptorDraft<'a> {
    /// Safe rail identity.
    pub rail_id: &'a str,
    /// Safe profile or command label; not a command body.
    pub profile_label: &'a str,
    /// Requirement level.
    pub requirement: ReviewRailRequirement,
}

/// Validated completion rail descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewRailDescriptor {
    rail_id: String,
    profile_label: String,
    requirement: ReviewRailRequirement,
}

impl ReviewRailDescriptor {
    /// Returns the safe rail identity.
    #[must_use]
    pub fn rail_id(&self) -> &str {
        self.rail_id.as_str()
    }

    /// Returns the safe profile label.
    #[must_use]
    pub fn profile_label(&self) -> &str {
        self.profile_label.as_str()
    }

    /// Returns the requirement level.
    #[must_use]
    pub const fn requirement(&self) -> ReviewRailRequirement {
        self.requirement
    }
}

/// Host-supplied rail result status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewRailStatus {
    /// Rail passed.
    Passed,
    /// Rail warned but did not fail.
    Warned,
    /// Rail failed.
    Failed,
    /// Rail was skipped with a safe reason.
    Skipped,
}

impl ReviewRailStatus {
    /// Returns the stable rail status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warned => "warned",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

/// Borrowed host-supplied completion rail result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewRailResultDraft<'a> {
    /// Safe rail identity.
    pub rail_id: &'a str,
    /// Host-supplied status.
    pub status: ReviewRailStatus,
    /// BLAKE3 summary hash for rail output.
    pub summary_hash: &'a str,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
}

/// Validated host-supplied completion rail result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewRailResult {
    rail_id: String,
    status: ReviewRailStatus,
    summary_hash: String,
    issue_code: Option<String>,
}

impl ReviewRailResult {
    /// Returns the safe rail identity.
    #[must_use]
    pub fn rail_id(&self) -> &str {
        self.rail_id.as_str()
    }

    /// Returns the rail status.
    #[must_use]
    pub const fn status(&self) -> ReviewRailStatus {
        self.status
    }

    /// Returns the BLAKE3 summary hash.
    #[must_use]
    pub fn summary_hash(&self) -> &str {
        self.summary_hash.as_str()
    }

    /// Returns the optional safe issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }
}

/// Borrowed reviewer attempt descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewAttemptPlanDraft<'a> {
    /// Safe attempt identity.
    pub attempt_id: &'a str,
    /// Safe reviewer role label.
    pub reviewer_role: &'a str,
    /// Safe reasoning profile label.
    pub reasoning_profile: &'a str,
    /// Safe output contract label.
    pub output_contract: &'a str,
    /// Safe timeout class label.
    pub timeout_class: &'a str,
    /// Attempt budget units.
    pub budget_units: u16,
}

/// Validated reviewer attempt descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAttemptPlan {
    attempt_id: String,
    reviewer_role: String,
    reasoning_profile: String,
    output_contract: String,
    timeout_class: String,
    budget_units: u16,
}

impl ReviewAttemptPlan {
    /// Returns the safe attempt identity.
    #[must_use]
    pub fn attempt_id(&self) -> &str {
        self.attempt_id.as_str()
    }

    /// Returns the safe reviewer role label.
    #[must_use]
    pub fn reviewer_role(&self) -> &str {
        self.reviewer_role.as_str()
    }

    /// Returns the safe output contract label.
    #[must_use]
    pub fn output_contract(&self) -> &str {
        self.output_contract.as_str()
    }

    /// Returns the safe timeout class label.
    #[must_use]
    pub fn timeout_class(&self) -> &str {
        self.timeout_class.as_str()
    }

    /// Returns attempt budget units.
    #[must_use]
    pub const fn budget_units(&self) -> u16 {
        self.budget_units
    }
}

/// Borrowed review lifecycle request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewLifecycleRequestDraft<'a> {
    /// Safe request identity.
    pub request_id: &'a str,
    /// Review subject metadata.
    pub subject: ReviewLifecycleSubjectDraft<'a>,
    /// Bounded evidence items.
    pub evidence: &'a [ReviewEvidenceDraft<'a>],
    /// Completion rail descriptors.
    pub rails: &'a [ReviewRailDescriptorDraft<'a>],
    /// Host-supplied completion rail results.
    pub rail_results: &'a [ReviewRailResultDraft<'a>],
    /// Reviewer attempt descriptors.
    pub attempts: &'a [ReviewAttemptPlanDraft<'a>],
    /// Whether an independent reviewer service is required.
    pub requires_independent_reviewer: bool,
    /// Safe output contract label.
    pub output_contract: &'a str,
}

/// Validated review lifecycle request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewLifecycleRequest {
    request_id: String,
    subject: ReviewLifecycleSubject,
    evidence: Vec<ReviewEvidence>,
    rails: Vec<ReviewRailDescriptor>,
    rail_results: Vec<ReviewRailResult>,
    attempts: Vec<ReviewAttemptPlan>,
    requires_independent_reviewer: bool,
    output_contract: String,
    request_hash: String,
}

impl ReviewLifecycleRequest {
    /// Returns the safe request identity.
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.request_id.as_str()
    }

    /// Returns review subject metadata.
    #[must_use]
    pub const fn subject(&self) -> &ReviewLifecycleSubject {
        &self.subject
    }

    /// Returns bounded evidence.
    #[must_use]
    pub fn evidence(&self) -> &[ReviewEvidence] {
        self.evidence.as_slice()
    }

    /// Returns completion rail descriptors.
    #[must_use]
    pub fn rails(&self) -> &[ReviewRailDescriptor] {
        self.rails.as_slice()
    }

    /// Returns host-supplied rail results.
    #[must_use]
    pub fn rail_results(&self) -> &[ReviewRailResult] {
        self.rail_results.as_slice()
    }

    /// Returns reviewer attempt descriptors.
    #[must_use]
    pub fn attempts(&self) -> &[ReviewAttemptPlan] {
        self.attempts.as_slice()
    }

    /// Returns whether an independent reviewer is required.
    #[must_use]
    pub const fn requires_independent_reviewer(&self) -> bool {
        self.requires_independent_reviewer
    }

    /// Returns the safe output contract label.
    #[must_use]
    pub fn output_contract(&self) -> &str {
        self.output_contract.as_str()
    }

    /// Returns the deterministic request hash.
    #[must_use]
    pub fn request_hash(&self) -> &str {
        self.request_hash.as_str()
    }
}

/// Availability state for a host-owned reviewer service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewServiceStatus {
    /// Reviewer service is available.
    Available,
    /// Reviewer service is required but missing.
    Missing,
    /// Reviewer service exists but is unavailable.
    Unavailable,
}

/// Review dispatch plan status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewDispatchStatus {
    /// Host may dispatch an attempt through its reviewer service.
    Ready,
    /// Missing reviewer service blocks dispatch.
    ReviewerServiceMissing,
    /// Unavailable reviewer service blocks dispatch.
    ReviewerServiceUnavailable,
}

/// Host-effect-free reviewer dispatch plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewDispatchPlan {
    request_hash: String,
    status: ReviewDispatchStatus,
}

impl ReviewDispatchPlan {
    /// Returns the safe request hash.
    #[must_use]
    pub fn request_hash(&self) -> &str {
        self.request_hash.as_str()
    }

    /// Returns the dispatch status.
    #[must_use]
    pub const fn status(&self) -> ReviewDispatchStatus {
        self.status
    }
}

/// Terminal lifecycle status after reviewer output or service failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewLifecycleStatus {
    /// Reviewer passed the completion claim.
    Passed,
    /// Reviewer warned and needs host triage.
    Warning,
    /// Reviewer failed the completion claim.
    Failed,
    /// Reviewer output was missing or malformed.
    MalformedOutput,
    /// Required reviewer service was missing or unavailable.
    MissingService,
}

impl ReviewLifecycleStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
            Self::MalformedOutput => "malformed-output",
            Self::MissingService => "missing-service",
        }
    }
}

/// Safe continuation intent after review reduction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewContinuationIntent {
    /// Completion may be acknowledged by host policy.
    AcknowledgeCompletion,
    /// Host should inspect or schedule manual follow-up.
    ManualFollowUp,
    /// Completion claim remains blocked.
    BlockCompletion,
    /// No continuation action is requested.
    None,
}

impl ReviewContinuationIntent {
    /// Returns the stable continuation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AcknowledgeCompletion => "acknowledge-completion",
            Self::ManualFollowUp => "manual-follow-up",
            Self::BlockCompletion => "block-completion",
            Self::None => "none",
        }
    }
}

/// Reduced done-review lifecycle outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewLifecycleOutcome {
    request_hash: String,
    status: ReviewLifecycleStatus,
    continuation: ReviewContinuationIntent,
    report_hash: Option<String>,
}

impl ReviewLifecycleOutcome {
    /// Returns the safe request hash.
    #[must_use]
    pub fn request_hash(&self) -> &str {
        self.request_hash.as_str()
    }

    /// Returns the terminal lifecycle status.
    #[must_use]
    pub const fn status(&self) -> ReviewLifecycleStatus {
        self.status
    }

    /// Returns the safe continuation intent.
    #[must_use]
    pub const fn continuation(&self) -> ReviewContinuationIntent {
        self.continuation
    }

    /// Returns the optional parsed report hash.
    #[must_use]
    pub fn report_hash(&self) -> Option<&str> {
        self.report_hash.as_deref()
    }
}

/// Borrowed done-review lifecycle receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewLifecycleReceiptDraft<'a> {
    /// BLAKE3 request hash.
    pub request_hash: &'a str,
    /// Terminal lifecycle status.
    pub status: ReviewLifecycleStatus,
    /// Continuation intent.
    pub continuation: ReviewContinuationIntent,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
    /// Safe BLAKE3 evidence hashes.
    pub evidence_hashes: &'a [&'a str],
}

/// Safe done-review lifecycle receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewLifecycleReceipt {
    receipt_hash: String,
    request_hash: String,
    status: ReviewLifecycleStatus,
    continuation: ReviewContinuationIntent,
    issue_code: Option<String>,
    evidence_hashes: Vec<String>,
}

impl ReviewLifecycleReceipt {
    /// Returns the deterministic receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }

    /// Returns the BLAKE3 request hash.
    #[must_use]
    pub fn request_hash(&self) -> &str {
        self.request_hash.as_str()
    }

    /// Returns the terminal lifecycle status.
    #[must_use]
    pub const fn status(&self) -> ReviewLifecycleStatus {
        self.status
    }

    /// Returns the continuation intent.
    #[must_use]
    pub const fn continuation(&self) -> ReviewContinuationIntent {
        self.continuation
    }

    /// Returns optional safe issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }

    /// Returns safe evidence hashes.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[String] {
        self.evidence_hashes.as_slice()
    }
}

/// Validates a done-review lifecycle request.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when metadata, evidence, rail, or attempt
/// descriptors are unsafe or over configured bounds.
pub fn validate_review_lifecycle_request(
    draft: &ReviewLifecycleRequestDraft<'_>,
) -> Result<ReviewLifecycleRequest, ReviewReportError> {
    let request_id = validate_safe_metadata("request_id", draft.request_id)?;
    let subject = validate_review_subject(&draft.subject)?;
    let evidence = validate_review_evidence_items(draft.evidence)?;
    let rails = validate_review_rails(draft.rails)?;
    let rail_results = validate_review_rail_results(draft.rail_results)?;
    let attempts = validate_review_attempts(draft.attempts)?;
    let output_contract = validate_safe_metadata("output_contract", draft.output_contract)?;
    let mut request = ReviewLifecycleRequest {
        request_id,
        subject,
        evidence,
        rails,
        rail_results,
        attempts,
        requires_independent_reviewer: draft.requires_independent_reviewer,
        output_contract,
        request_hash: String::new(),
    };
    request.request_hash = review_lifecycle_request_hash(&request);
    Ok(request)
}

/// Plans reviewer dispatch without calling providers or running rails.
#[must_use]
pub fn plan_review_lifecycle_dispatch(
    request: &ReviewLifecycleRequest,
    reviewer_status: ReviewServiceStatus,
) -> ReviewDispatchPlan {
    let status = if request.requires_independent_reviewer() {
        match reviewer_status {
            ReviewServiceStatus::Available => ReviewDispatchStatus::Ready,
            ReviewServiceStatus::Missing => ReviewDispatchStatus::ReviewerServiceMissing,
            ReviewServiceStatus::Unavailable => ReviewDispatchStatus::ReviewerServiceUnavailable,
        }
    } else {
        ReviewDispatchStatus::Ready
    };
    ReviewDispatchPlan {
        request_hash: request.request_hash().to_owned(),
        status,
    }
}

/// Reduces reviewer output or dispatch failure into a safe lifecycle outcome.
#[must_use]
pub fn reduce_review_lifecycle_outcome(
    request: &ReviewLifecycleRequest,
    dispatch: &ReviewDispatchPlan,
    report: Option<&ReviewReport>,
) -> ReviewLifecycleOutcome {
    if dispatch.status() != ReviewDispatchStatus::Ready {
        return ReviewLifecycleOutcome {
            request_hash: request.request_hash().to_owned(),
            status: ReviewLifecycleStatus::MissingService,
            continuation: ReviewContinuationIntent::BlockCompletion,
            report_hash: None,
        };
    }
    let Some(report) = report else {
        return malformed_review_outcome(request);
    };
    if !report.diagnostics().is_empty() {
        return malformed_review_outcome(request);
    }
    let (status, continuation) = match report.verdict() {
        ReviewVerdict::Pass => (
            ReviewLifecycleStatus::Passed,
            ReviewContinuationIntent::AcknowledgeCompletion,
        ),
        ReviewVerdict::Warn => (
            ReviewLifecycleStatus::Warning,
            ReviewContinuationIntent::ManualFollowUp,
        ),
        ReviewVerdict::Fail => (
            ReviewLifecycleStatus::Failed,
            ReviewContinuationIntent::BlockCompletion,
        ),
    };
    ReviewLifecycleOutcome {
        request_hash: request.request_hash().to_owned(),
        status,
        continuation,
        report_hash: Some(report.report_hash().to_owned()),
    }
}

/// Validates a done-review lifecycle receipt.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when receipt metadata leaks unsafe content or
/// hash fields are malformed.
pub fn validate_review_lifecycle_receipt(
    draft: &ReviewLifecycleReceiptDraft<'_>,
) -> Result<ReviewLifecycleReceipt, ReviewReportError> {
    let request_hash = validate_blake3_hash("request_hash", draft.request_hash)?;
    let issue_code = draft
        .issue_code
        .map(|issue| validate_safe_metadata("issue_code", issue))
        .transpose()?;
    let mut evidence_hashes = Vec::with_capacity(draft.evidence_hashes.len());
    for evidence_hash in draft.evidence_hashes {
        evidence_hashes.push(validate_blake3_hash("evidence_hash", evidence_hash)?);
    }
    let receipt_hash = review_lifecycle_receipt_hash(ReviewLifecycleReceiptHashInput {
        request_hash: request_hash.as_str(),
        status: draft.status,
        continuation: draft.continuation,
        issue_code: issue_code.as_deref(),
        evidence_hashes: evidence_hashes.as_slice(),
    });
    Ok(ReviewLifecycleReceipt {
        receipt_hash,
        request_hash,
        status: draft.status,
        continuation: draft.continuation,
        issue_code,
        evidence_hashes,
    })
}

fn malformed_review_outcome(request: &ReviewLifecycleRequest) -> ReviewLifecycleOutcome {
    ReviewLifecycleOutcome {
        request_hash: request.request_hash().to_owned(),
        status: ReviewLifecycleStatus::MalformedOutput,
        continuation: ReviewContinuationIntent::ManualFollowUp,
        report_hash: None,
    }
}

fn validate_review_subject(
    draft: &ReviewLifecycleSubjectDraft<'_>,
) -> Result<ReviewLifecycleSubject, ReviewReportError> {
    Ok(ReviewLifecycleSubject {
        subject_id: validate_safe_metadata("subject_id", draft.subject_id)?,
        kind: draft.kind,
        owner_label: validate_safe_metadata("owner_label", draft.owner_label)?,
        stage_label: validate_safe_metadata("stage_label", draft.stage_label)?,
    })
}

fn validate_review_evidence_items(
    drafts: &[ReviewEvidenceDraft<'_>],
) -> Result<Vec<ReviewEvidence>, ReviewReportError> {
    if drafts.len() > MAX_REVIEW_EVIDENCE_ITEMS {
        return Err(ReviewReportError::new(
            ReviewDiagnosticCode::TooManyItems,
            "evidence",
        ));
    }
    let mut evidence = Vec::with_capacity(drafts.len());
    for draft in drafts {
        evidence.push(ReviewEvidence {
            evidence_id: validate_safe_metadata("evidence_id", draft.evidence_id)?,
            kind: draft.kind,
            label: validate_safe_metadata("evidence_label", draft.label)?,
            evidence_hash: validate_blake3_hash("evidence_hash", draft.evidence_hash)?,
            item_count: draft.item_count,
            redaction: draft.redaction,
        });
    }
    Ok(evidence)
}

fn validate_review_rails(
    drafts: &[ReviewRailDescriptorDraft<'_>],
) -> Result<Vec<ReviewRailDescriptor>, ReviewReportError> {
    if drafts.len() > MAX_REVIEW_RAILS {
        return Err(ReviewReportError::new(
            ReviewDiagnosticCode::TooManyItems,
            "rails",
        ));
    }
    let mut rails = Vec::with_capacity(drafts.len());
    for draft in drafts {
        rails.push(ReviewRailDescriptor {
            rail_id: validate_safe_metadata("rail_id", draft.rail_id)?,
            profile_label: validate_safe_metadata("profile_label", draft.profile_label)?,
            requirement: draft.requirement,
        });
    }
    Ok(rails)
}

fn validate_review_rail_results(
    drafts: &[ReviewRailResultDraft<'_>],
) -> Result<Vec<ReviewRailResult>, ReviewReportError> {
    if drafts.len() > MAX_REVIEW_RAILS {
        return Err(ReviewReportError::new(
            ReviewDiagnosticCode::TooManyItems,
            "rail_results",
        ));
    }
    let mut results = Vec::with_capacity(drafts.len());
    for draft in drafts {
        results.push(ReviewRailResult {
            rail_id: validate_safe_metadata("rail_id", draft.rail_id)?,
            status: draft.status,
            summary_hash: validate_blake3_hash("summary_hash", draft.summary_hash)?,
            issue_code: draft
                .issue_code
                .map(|issue| validate_safe_metadata("issue_code", issue))
                .transpose()?,
        });
    }
    Ok(results)
}

fn validate_review_attempts(
    drafts: &[ReviewAttemptPlanDraft<'_>],
) -> Result<Vec<ReviewAttemptPlan>, ReviewReportError> {
    if drafts.len() > MAX_REVIEW_ATTEMPTS {
        return Err(ReviewReportError::new(
            ReviewDiagnosticCode::TooManyItems,
            "attempts",
        ));
    }
    let mut attempts = Vec::with_capacity(drafts.len());
    for draft in drafts {
        if draft.budget_units < MIN_REVIEW_ATTEMPT_BUDGET_UNITS {
            return Err(ReviewReportError::new(
                ReviewDiagnosticCode::PromptAdmissionDenied,
                "budget_units",
            ));
        }
        attempts.push(ReviewAttemptPlan {
            attempt_id: validate_safe_metadata("attempt_id", draft.attempt_id)?,
            reviewer_role: validate_safe_metadata("reviewer_role", draft.reviewer_role)?,
            reasoning_profile: validate_safe_metadata("reasoning_profile", draft.reasoning_profile)?,
            output_contract: validate_safe_metadata("output_contract", draft.output_contract)?,
            timeout_class: validate_safe_metadata("timeout_class", draft.timeout_class)?,
            budget_units: draft.budget_units,
        });
    }
    Ok(attempts)
}

fn review_lifecycle_request_hash(request: &ReviewLifecycleRequest) -> String {
    let mut canonical = String::from(REVIEW_LIFECYCLE_HASH_DOMAIN);
    push_corrective_hash_part(&mut canonical, request.request_id());
    push_corrective_hash_part(&mut canonical, request.subject().subject_id());
    push_corrective_hash_part(&mut canonical, request.subject().kind().as_str());
    push_corrective_hash_part(&mut canonical, request.output_contract());
    for evidence in request.evidence() {
        push_corrective_hash_part(&mut canonical, evidence.evidence_id());
        push_corrective_hash_part(&mut canonical, evidence.kind().as_str());
        push_corrective_hash_part(&mut canonical, evidence.evidence_hash());
    }
    for rail in request.rails() {
        push_corrective_hash_part(&mut canonical, rail.rail_id());
        push_corrective_hash_part(&mut canonical, rail.requirement().as_str());
    }
    for result in request.rail_results() {
        push_corrective_hash_part(&mut canonical, result.rail_id());
        push_corrective_hash_part(&mut canonical, result.status().as_str());
        push_corrective_hash_part(&mut canonical, result.summary_hash());
    }
    for attempt in request.attempts() {
        push_corrective_hash_part(&mut canonical, attempt.attempt_id());
        push_corrective_hash_part(&mut canonical, attempt.reviewer_role());
        push_corrective_hash_part(&mut canonical, attempt.output_contract());
    }
    hash_text(canonical.as_str())
}

#[derive(Clone, Copy)]
struct ReviewLifecycleReceiptHashInput<'a> {
    request_hash: &'a str,
    status: ReviewLifecycleStatus,
    continuation: ReviewContinuationIntent,
    issue_code: Option<&'a str>,
    evidence_hashes: &'a [String],
}

fn review_lifecycle_receipt_hash(input: ReviewLifecycleReceiptHashInput<'_>) -> String {
    let mut canonical = String::from(REVIEW_LIFECYCLE_RECEIPT_HASH_DOMAIN);
    push_corrective_hash_part(&mut canonical, input.request_hash);
    push_corrective_hash_part(&mut canonical, input.status.as_str());
    push_corrective_hash_part(&mut canonical, input.continuation.as_str());
    push_corrective_hash_part(&mut canonical, input.issue_code.unwrap_or_default());
    for evidence_hash in input.evidence_hashes {
        push_corrective_hash_part(&mut canonical, evidence_hash.as_str());
    }
    hash_text(canonical.as_str())
}
