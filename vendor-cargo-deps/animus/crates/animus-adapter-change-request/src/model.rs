use alloc::{string::String, vec::Vec};

use crate::{
    DEFAULT_MAX_CI_FAILURES, DEFAULT_MAX_REPAIR_ATTEMPTS, DEFAULT_MAX_REVIEW_FILES,
    DEFAULT_MAX_REVIEW_FINDINGS, DEFAULT_MAX_REVIEW_HUNKS, DEFAULT_MAX_REVIEW_LINES,
    DEFAULT_MINIMUM_COMMENT_CONFIDENCE, MAX_BODY_BYTES, MAX_IDENTITY_BYTES, MAX_REVIEW_CONFIDENCE,
    MAX_TITLE_BYTES,
};

/// Change-request adapter error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required field is empty, malformed, or over its bound.
    InvalidField(&'static str),
    /// A numeric bound is zero.
    InvalidBound(&'static str),
    /// A collection or count exceeds its bound.
    BoundExceeded {
        /// Bounded field name.
        field: &'static str,
        /// Observed count.
        actual: u32,
        /// Allowed count.
        maximum: u32,
    },
    /// A service request and observation have different bindings.
    BindingMismatch,
    /// A requested inline review location was not observed.
    MissingInlineAnchor,
    /// A required service is absent.
    MissingService(ServiceKind),
    /// A declared service is not currently available.
    ServiceBlocked {
        /// Blocked service family.
        service: ServiceKind,
        /// Exact declared availability.
        availability: ServiceAvailability,
    },
    /// A receipt requested raw content disclosure.
    UnsafeReceipt,
    /// A receipt requested a claim outside this adapter contract.
    UnsupportedClaim,
}

/// Supported forge families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForgeKind {
    /// GitHub-compatible repository.
    GitHub,
    /// GitLab-compatible repository.
    GitLab,
    /// Forgejo-compatible repository.
    Forgejo,
    /// Gitea-compatible repository.
    Gitea,
    /// Aspen Patch repository.
    Aspen,
    /// Git transport without a change-request API.
    GitOnly,
    /// Unknown or unsupported forge.
    Unknown,
}

/// Change-request publication mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationMode {
    /// Prepare a draft change request.
    Draft,
    /// Prepare a review-ready change request.
    Ready,
    /// Prepare a patch series without a forge request.
    PatchSeries,
}

/// Side of a reviewed diff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewSide {
    /// Old-file or deletion side.
    Left,
    /// New-file or addition side.
    Right,
}

/// Review finding severity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewSeverity {
    /// Informational finding.
    Info,
    /// Suggested improvement.
    Suggestion,
    /// Warning that requires attention.
    Warning,
    /// Blocking finding.
    Blocking,
}

/// Review finding source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewFindingSource {
    /// Deterministic host analyzer.
    Deterministic,
    /// Injected advisory review service.
    Advisory,
    /// Human escalation record.
    HumanEscalation,
}

/// Service families for change-request operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind {
    /// Change-request metadata and diff source.
    ChangeRequestSource,
    /// Advisory review analysis.
    ReviewAnalysis,
    /// Change-request publication.
    ChangeRequestPublication,
    /// Review comment or summary publication.
    ReviewPublication,
}

/// Service availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceAvailability {
    /// Service is available for admitted requests.
    Available,
    /// Service exists but current policy denied it.
    Denied,
    /// Service does not support the operation.
    Unsupported,
    /// Service is not present.
    Unavailable,
    /// Service evidence is stale.
    Stale,
}

/// Bounded service failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceFailureClass {
    /// Policy denied the request.
    Denied,
    /// Service is unavailable.
    Unavailable,
    /// Service does not support the request.
    Unsupported,
    /// Request was cancelled.
    Cancelled,
    /// Request timed out.
    TimedOut,
    /// Response was malformed.
    Malformed,
    /// Response was stale.
    Stale,
    /// Response binding did not match.
    Mismatch,
    /// Response exceeded a declared bound.
    Overflow,
    /// Host infrastructure failed.
    Infrastructure,
}

/// Exact binding for one change-request adapter call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRequestBinding {
    repository_identity: String,
    policy_identity: String,
    plan_identity: String,
    generation: u64,
    implementation_identity: String,
    correlation_identity: String,
}

impl ChangeRequestBinding {
    /// Creates and validates one exact binding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when an identity is empty or invalid.
    pub fn new(
        repository_identity: String,
        policy_identity: String,
        plan_identity: String,
        generation: u64,
        implementation_identity: String,
        correlation_identity: String,
    ) -> Result<Self, Error> {
        validate_identity("binding.repository_identity", &repository_identity)?;
        validate_identity("binding.policy_identity", &policy_identity)?;
        validate_identity("binding.plan_identity", &plan_identity)?;
        validate_identity("binding.implementation_identity", &implementation_identity)?;
        validate_identity("binding.correlation_identity", &correlation_identity)?;
        Ok(Self {
            repository_identity,
            policy_identity,
            plan_identity,
            generation,
            implementation_identity,
            correlation_identity,
        })
    }

    /// Returns the repository identity.
    #[must_use]
    pub fn repository_identity(&self) -> &str {
        &self.repository_identity
    }

    /// Returns the policy identity.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        &self.policy_identity
    }

    /// Returns the plan identity.
    #[must_use]
    pub fn plan_identity(&self) -> &str {
        &self.plan_identity
    }

    /// Returns the caller-supplied generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the implementation identity.
    #[must_use]
    pub fn implementation_identity(&self) -> &str {
        &self.implementation_identity
    }

    /// Returns the correlation identity.
    #[must_use]
    pub fn correlation_identity(&self) -> &str {
        &self.correlation_identity
    }
}

/// Forge-neutral repository reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryRef {
    /// Stable repository identity.
    pub identity: String,
    /// Selected forge family.
    pub forge: ForgeKind,
    /// Default branch label.
    pub default_branch: String,
}

impl RepositoryRef {
    /// Validates repository metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when repository metadata is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity("repository.identity", &self.identity)?;
        validate_identity("repository.default_branch", &self.default_branch)?;
        if self.forge == ForgeKind::Unknown {
            return Err(Error::InvalidField("repository.forge"));
        }
        Ok(())
    }
}

/// Capabilities advertised by a concrete forge adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForgeCapabilities {
    /// Draft change-request support.
    pub draft_change_requests: bool,
    /// Status or check observation support.
    pub status_checks: bool,
    /// Issue-thread comment support.
    pub comments: bool,
    /// Inline review-comment support.
    pub inline_review_comments: bool,
    /// Review summary support.
    pub review_submission: bool,
    /// Patch-series support without a forge request.
    pub patch_series: bool,
}

impl ForgeCapabilities {
    /// Returns capabilities for a Git-only adapter.
    #[must_use]
    pub const fn git_only() -> Self {
        Self {
            draft_change_requests: false,
            status_checks: false,
            comments: false,
            inline_review_comments: false,
            review_submission: false,
            patch_series: true,
        }
    }

    /// Returns capabilities for a full review forge.
    #[must_use]
    pub const fn full_review() -> Self {
        Self {
            draft_change_requests: true,
            status_checks: true,
            comments: true,
            inline_review_comments: true,
            review_submission: true,
            patch_series: false,
        }
    }
}

/// Forge-neutral change-request reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRequestRef {
    /// Provider-local change-request identity.
    pub identity: String,
    /// Source branch identity.
    pub source_branch: String,
    /// Target branch identity.
    pub target_branch: String,
    /// Exact reviewed commit identity.
    pub commit_identity: String,
}

impl ChangeRequestRef {
    /// Validates the reference.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when an identity is empty or invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity("change_request.identity", &self.identity)?;
        validate_identity("change_request.source_branch", &self.source_branch)?;
        validate_identity("change_request.target_branch", &self.target_branch)?;
        validate_identity("change_request.commit_identity", &self.commit_identity)
    }
}

/// Prepared change-request draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRequestDraft {
    /// Bounded title.
    pub title: String,
    /// Bounded description body.
    pub body: String,
    /// Prepared source branch.
    pub source_branch: String,
    /// Requested publication mode.
    pub mode: PublicationMode,
}

impl ChangeRequestDraft {
    /// Validates the bounded draft.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when draft text or identities are invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_text("change_request_draft.title", &self.title, MAX_TITLE_BYTES)?;
        validate_text("change_request_draft.body", &self.body, MAX_BODY_BYTES)?;
        validate_identity("change_request_draft.source_branch", &self.source_branch)
    }
}

/// One-based inclusive line range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineRange {
    /// First line.
    pub start: u32,
    /// Last line.
    pub end: u32,
}

impl LineRange {
    /// Validates the range.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when the range is zero or reversed.
    pub const fn validate(self) -> Result<(), Error> {
        if self.start == 0 || self.end < self.start {
            return Err(Error::InvalidField("line_range"));
        }
        Ok(())
    }
}

/// Provider-neutral observed inline anchor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewInlineAnchor {
    /// Exact commit identity.
    pub commit_identity: String,
    /// Repository-relative file identity.
    pub file_identity: String,
    /// One-based terminal line.
    pub line: u32,
    /// Diff side.
    pub side: ReviewSide,
    /// Optional one-based range start.
    pub start_line: Option<u32>,
}

impl ReviewInlineAnchor {
    /// Validates the anchor.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when an identity or line is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity("inline_anchor.commit_identity", &self.commit_identity)?;
        validate_identity("inline_anchor.file_identity", &self.file_identity)?;
        if self.line == 0
            || self
                .start_line
                .is_some_and(|start| start == 0 || start > self.line)
        {
            return Err(Error::InvalidField("inline_anchor.line"));
        }
        Ok(())
    }
}

/// Bounded per-file diff observation without source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffFileObservation {
    /// Safe repository-relative file identity.
    pub file_identity: String,
    /// Number of hunks.
    pub hunk_count: u32,
    /// Number of added lines.
    pub added_line_count: u32,
    /// Number of deleted lines.
    pub deleted_line_count: u32,
    /// Observed inline anchors.
    pub inline_anchors: Vec<ReviewInlineAnchor>,
}

/// Bounded diff observation without raw diff data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffObservation {
    /// Exact reviewed commit identity.
    pub commit_identity: String,
    /// File observations.
    pub files: Vec<DiffFileObservation>,
    /// Total hunk count.
    pub total_hunk_count: u32,
    /// Total added-line count.
    pub total_added_line_count: u32,
    /// Total deleted-line count.
    pub total_deleted_line_count: u32,
}

/// Bounded review finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewFinding {
    /// Finding source.
    pub source: ReviewFindingSource,
    /// Finding severity.
    pub severity: ReviewSeverity,
    /// Safe category label.
    pub category: String,
    /// Bounded model-visible or human-visible message.
    pub message: String,
    /// Optional file identity.
    pub file_identity: Option<String>,
    /// Optional line range.
    pub line_range: Option<LineRange>,
    /// Optional exact diff side for the location.
    pub side: Option<ReviewSide>,
    /// Confidence from zero through one hundred.
    pub confidence: u8,
}

impl ReviewFinding {
    /// Validates finding bounds and location shape.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when content, confidence, or location is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity("review_finding.category", &self.category)?;
        validate_text("review_finding.message", &self.message, MAX_BODY_BYTES)?;
        if self.confidence > MAX_REVIEW_CONFIDENCE {
            return Err(Error::InvalidField("review_finding.confidence"));
        }
        if let Some(file_identity) = &self.file_identity {
            validate_identity("review_finding.file_identity", file_identity)?;
        }
        if let Some(range) = self.line_range {
            range.validate()?;
        }
        let has_file = self.file_identity.is_some();
        if self.line_range.is_some() != has_file || self.side.is_some() != has_file {
            return Err(Error::InvalidField("review_finding.location"));
        }
        Ok(())
    }
}

/// Failed validation observation for CI triage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckFailureObservation {
    /// Trusted validation profile identity.
    pub profile_identity: String,
    /// BLAKE3 identity of bounded output evidence.
    pub output_identity: String,
    /// Completed repair attempts.
    pub repair_attempts: u32,
}

impl CheckFailureObservation {
    /// Validates safe CI-failure metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when an identity is malformed.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity("check_failure.profile_identity", &self.profile_identity)?;
        validate_blake3("check_failure.output_identity", &self.output_identity)
    }
}

/// Bounds for one code-review plan.
#[allow(
    clippy::struct_field_names,
    reason = "the max_ prefix makes each public bound explicit at call sites"
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewBounds {
    /// Maximum files.
    pub max_files: u32,
    /// Maximum hunks.
    pub max_hunks: u32,
    /// Maximum changed lines.
    pub max_changed_lines: u32,
    /// Maximum findings.
    pub max_findings: u32,
    /// Maximum CI failures.
    pub max_ci_failures: u32,
}

impl ReviewBounds {
    /// Returns conservative bounds.
    #[must_use]
    pub const fn conservative() -> Self {
        Self {
            max_files: DEFAULT_MAX_REVIEW_FILES,
            max_hunks: DEFAULT_MAX_REVIEW_HUNKS,
            max_changed_lines: DEFAULT_MAX_REVIEW_LINES,
            max_findings: DEFAULT_MAX_REVIEW_FINDINGS,
            max_ci_failures: DEFAULT_MAX_CI_FAILURES,
        }
    }

    /// Validates all bounds.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBound`] when any bound is zero.
    pub const fn validate(self) -> Result<(), Error> {
        if self.max_files == 0 {
            return Err(Error::InvalidBound("review_bounds.max_files"));
        }
        if self.max_hunks == 0 {
            return Err(Error::InvalidBound("review_bounds.max_hunks"));
        }
        if self.max_changed_lines == 0 {
            return Err(Error::InvalidBound("review_bounds.max_changed_lines"));
        }
        if self.max_findings == 0 {
            return Err(Error::InvalidBound("review_bounds.max_findings"));
        }
        if self.max_ci_failures == 0 {
            return Err(Error::InvalidBound("review_bounds.max_ci_failures"));
        }
        Ok(())
    }
}

/// Policy for one code-review plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReviewPolicy {
    /// Request inline publication for located findings.
    pub publish_inline_comments: bool,
    /// Permit summary fallback when inline publication is unavailable.
    pub allow_summary_fallback: bool,
    /// Request injected analysis for complex diffs.
    pub request_analysis_for_complex_diff: bool,
    /// Minimum confidence for automated publication.
    pub minimum_comment_confidence: u8,
    /// Maximum repair attempts before escalation.
    pub max_repair_attempts: u32,
}

impl ReviewPolicy {
    /// Returns conservative review policy.
    #[must_use]
    pub const fn conservative() -> Self {
        Self {
            publish_inline_comments: true,
            allow_summary_fallback: true,
            request_analysis_for_complex_diff: true,
            minimum_comment_confidence: DEFAULT_MINIMUM_COMMENT_CONFIDENCE,
            max_repair_attempts: DEFAULT_MAX_REPAIR_ATTEMPTS,
        }
    }

    /// Validates review policy and forge capability compatibility.
    ///
    /// # Errors
    ///
    /// Returns a typed error when a bound, confidence, or capability is invalid.
    pub const fn validate(self, capabilities: ForgeCapabilities) -> Result<(), Error> {
        if self.minimum_comment_confidence > MAX_REVIEW_CONFIDENCE {
            return Err(Error::InvalidField(
                "review_policy.minimum_comment_confidence",
            ));
        }
        if self.max_repair_attempts == 0 {
            return Err(Error::InvalidBound("review_policy.max_repair_attempts"));
        }
        if self.publish_inline_comments
            && !capabilities.inline_review_comments
            && (!self.allow_summary_fallback || !capabilities.review_submission)
        {
            return Err(Error::MissingInlineAnchor);
        }
        Ok(())
    }
}

/// Validates that an observation binding exactly matches its request.
///
/// # Errors
///
/// Returns [`Error::BindingMismatch`] when any bound identity differs.
pub fn validate_binding(
    expected: &ChangeRequestBinding,
    actual: &ChangeRequestBinding,
) -> Result<(), Error> {
    if expected != actual {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}

pub(crate) fn validate_identity(field: &'static str, value: &str) -> Result<(), Error> {
    validate_text(field, value, MAX_IDENTITY_BYTES)
}

pub(crate) fn validate_text(
    field: &'static str,
    value: &str,
    maximum_bytes: usize,
) -> Result<(), Error> {
    if value.is_empty() || value.trim() != value || value.len() > maximum_bytes {
        return Err(Error::InvalidField(field));
    }
    Ok(())
}

pub(crate) fn validate_blake3(field: &'static str, value: &str) -> Result<(), Error> {
    const BLAKE3_HEX_BYTES: usize = 64;
    let digest = value.strip_prefix("b3:").unwrap_or(value);
    if digest.len() != BLAKE3_HEX_BYTES
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(Error::InvalidField(field));
    }
    Ok(())
}

pub(crate) fn bounded_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
