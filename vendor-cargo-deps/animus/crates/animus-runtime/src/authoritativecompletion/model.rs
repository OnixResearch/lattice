// Public completion DTO names intentionally retain their domain prefix and standard owned aliases as an embedder compatibility surface.
#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]
use std::{string::String, vec::Vec};

/// Host capability class required by a repository-owned check.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CheckCapability {
    /// Exact-pinned Cairn validation is available.
    Cairn,
    /// A bounded local process service is available.
    LocalProcess,
    /// The repository-pinned Nix service is available.
    Nix,
}

impl CheckCapability {
    /// Returns the stable registry label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cairn => "cairn",
            Self::LocalProcess => "local-process",
            Self::Nix => "nix",
        }
    }
}

/// Maximum check tier admitted by host policy.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CheckTier {
    /// Focused, low-latency checks.
    Quick,
    /// Repository-standard checks.
    Standard,
    /// Broad or host-specific checks.
    Full,
}

impl CheckTier {
    /// Returns the stable registry label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Quick => "quick",
            Self::Standard => "standard",
            Self::Full => "full",
        }
    }
}

/// Exact result status retained for one required check.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CheckStatus {
    /// Check execution passed.
    Passed,
    /// Check execution failed.
    Failed,
    /// Host policy blocked the check.
    Blocked,
    /// Host skipped the check.
    Skipped,
    /// Host deferred the check.
    Deferred,
    /// Check exceeded its finite deadline.
    TimedOut,
    /// Check was cancelled.
    Cancelled,
    /// Only a dry-run plan was produced.
    DryRun,
    /// Process-group cleanup failed.
    CleanupFailed,
    /// Host infrastructure failed.
    InfrastructureError,
    /// Registry selection did not select the row.
    NotSelected,
    /// Producer supplied an unknown status.
    Unknown,
}

impl CheckStatus {
    /// Returns the stable Cairn-compatible status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Skipped => "skipped",
            Self::Deferred => "deferred",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::DryRun => "dry_run",
            Self::CleanupFailed => "cleanup_failed",
            Self::InfrastructureError => "infrastructure_error",
            Self::NotSelected => "not_selected",
            Self::Unknown => "unknown",
        }
    }

    /// Returns true only for completion-eligible passing evidence.
    #[must_use]
    pub const fn is_passing(self) -> bool {
        matches!(self, Self::Passed)
    }
}

/// Borrowed repository-owned check row before validation.
#[derive(Clone, Copy, Debug)]
pub struct AgentCheckRowDraft<'a> {
    /// Stable check identifier.
    pub check_id: &'a str,
    /// Host execution profile identifier.
    pub profile: &'a str,
    /// Exact process argv.
    pub argv: &'a [&'a str],
    /// Repository-relative affected path prefixes.
    pub affected_paths: &'a [&'a str],
    /// Affected crate identifiers.
    pub affected_crates: &'a [&'a str],
    /// Affected surface identifiers.
    pub affected_surfaces: &'a [&'a str],
    /// Required prerequisite check identifiers.
    pub prerequisites: &'a [&'a str],
    /// Host capability required to execute the check.
    pub capability: CheckCapability,
    /// Check tier.
    pub tier: CheckTier,
    /// Reviewed risk label.
    pub risk: &'a str,
    /// Named timeout budget class.
    pub timeout_class: &'a str,
    /// Named output budget class.
    pub output_budget_class: &'a str,
    /// Expected safe artifact paths.
    pub expected_artifacts: &'a [&'a str],
    /// Exact reproduction argv.
    pub reproduction_argv: &'a [&'a str],
    /// Safe reproduction working-root label.
    pub working_root: &'a str,
    /// Repository-relative documentation references.
    pub documentation_refs: &'a [&'a str],
    /// Repository-relative positive contract fixture.
    pub positive_contract: &'a str,
    /// Repository-relative negative contract fixture.
    pub negative_contract: &'a str,
}

/// Borrowed agent-check registry before validation.
#[derive(Clone, Copy, Debug)]
pub struct AgentCheckRegistryDraft<'a> {
    /// Animus registry schema.
    pub schema: &'a str,
    /// Repository identity.
    pub repository_id: &'a str,
    /// Registry identity.
    pub registry_id: &'a str,
    /// Supported Cairn authoritative schema.
    pub supported_cairn_schema: &'a str,
    /// Pinned Cairn generated-policy hash.
    pub cairn_policy_hash: &'a str,
    /// Conservative fallback check identifiers.
    pub fallback_check_ids: &'a [&'a str],
    /// Registry rows.
    pub checks: &'a [AgentCheckRowDraft<'a>],
}

/// Validated repository-owned check row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCheckRow {
    pub(super) check_id: String,
    pub(super) profile: String,
    pub(super) argv: Vec<String>,
    pub(super) affected_paths: Vec<String>,
    pub(super) affected_crates: Vec<String>,
    pub(super) affected_surfaces: Vec<String>,
    pub(super) prerequisites: Vec<String>,
    pub(super) capability: CheckCapability,
    pub(super) tier: CheckTier,
    pub(super) risk: String,
    pub(super) timeout_class: String,
    pub(super) output_budget_class: String,
    pub(super) expected_artifacts: Vec<String>,
    pub(super) documentation_refs: Vec<String>,
    pub(super) positive_contract: String,
    pub(super) negative_contract: String,
}

impl AgentCheckRow {
    /// Returns the stable check identifier.
    #[must_use]
    pub fn check_id(&self) -> &str {
        self.check_id.as_str()
    }

    /// Returns the host execution profile.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.profile.as_str()
    }

    /// Returns the exact argv.
    #[must_use]
    pub fn argv(&self) -> &[String] {
        self.argv.as_slice()
    }

    /// Returns affected path prefixes.
    #[must_use]
    pub fn affected_paths(&self) -> &[String] {
        self.affected_paths.as_slice()
    }

    /// Returns affected crate identifiers.
    #[must_use]
    pub fn affected_crates(&self) -> &[String] {
        self.affected_crates.as_slice()
    }

    /// Returns affected surface identifiers.
    #[must_use]
    pub fn affected_surfaces(&self) -> &[String] {
        self.affected_surfaces.as_slice()
    }

    /// Returns prerequisite identifiers.
    #[must_use]
    pub fn prerequisites(&self) -> &[String] {
        self.prerequisites.as_slice()
    }

    /// Returns the required host capability.
    #[must_use]
    pub const fn capability(&self) -> CheckCapability {
        self.capability
    }

    /// Returns the check tier.
    #[must_use]
    pub const fn tier(&self) -> CheckTier {
        self.tier
    }

    /// Returns the reviewed risk label.
    #[must_use]
    pub fn risk(&self) -> &str {
        self.risk.as_str()
    }

    /// Returns the timeout budget class.
    #[must_use]
    pub fn timeout_class(&self) -> &str {
        self.timeout_class.as_str()
    }

    /// Returns the output budget class.
    #[must_use]
    pub fn output_budget_class(&self) -> &str {
        self.output_budget_class.as_str()
    }

    /// Returns expected artifact paths.
    #[must_use]
    pub fn expected_artifacts(&self) -> &[String] {
        self.expected_artifacts.as_slice()
    }

    /// Returns documentation references.
    #[must_use]
    pub fn documentation_refs(&self) -> &[String] {
        self.documentation_refs.as_slice()
    }

    /// Returns the positive contract path.
    #[must_use]
    pub fn positive_contract(&self) -> &str {
        self.positive_contract.as_str()
    }

    /// Returns the negative contract path.
    #[must_use]
    pub fn negative_contract(&self) -> &str {
        self.negative_contract.as_str()
    }
}

/// Validated repository-owned check registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCheckRegistry {
    pub(super) repository_id: String,
    pub(super) registry_id: String,
    pub(super) cairn_policy_hash: String,
    pub(super) fallback_check_ids: Vec<String>,
    pub(super) checks: Vec<AgentCheckRow>,
    pub(super) registry_hash: String,
}

impl AgentCheckRegistry {
    /// Returns the repository identity.
    #[must_use]
    pub fn repository_id(&self) -> &str {
        self.repository_id.as_str()
    }

    /// Returns the registry identity.
    #[must_use]
    pub fn registry_id(&self) -> &str {
        self.registry_id.as_str()
    }

    /// Returns the pinned Cairn policy hash.
    #[must_use]
    pub fn cairn_policy_hash(&self) -> &str {
        self.cairn_policy_hash.as_str()
    }

    /// Returns conservative fallback check identifiers.
    #[must_use]
    pub fn fallback_check_ids(&self) -> &[String] {
        self.fallback_check_ids.as_slice()
    }

    /// Returns validated check rows.
    #[must_use]
    pub fn checks(&self) -> &[AgentCheckRow] {
        self.checks.as_slice()
    }

    /// Returns the deterministic registry identity hash.
    #[must_use]
    pub fn registry_hash(&self) -> &str {
        self.registry_hash.as_str()
    }
}

/// Host-supplied changed artifact before selection.
#[derive(Clone, Copy, Debug)]
pub struct ChangedArtifactDraft<'a> {
    /// Stable repository-relative path.
    pub path: &'a str,
    /// Stable surface identifier.
    pub surface: &'a str,
    /// BLAKE3 content identity.
    pub content_hash: &'a str,
}

/// Cairn-compatible required check selected from the registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CairnRequiredCheck {
    pub(super) check_id: String,
    pub(super) command: Vec<String>,
    pub(super) command_hash: String,
    pub(super) prerequisites: Vec<String>,
}

impl CairnRequiredCheck {
    /// Returns the selected check identifier.
    #[must_use]
    pub fn check_id(&self) -> &str {
        self.check_id.as_str()
    }

    /// Returns the exact command argv.
    #[must_use]
    pub fn command(&self) -> &[String] {
        self.command.as_slice()
    }

    /// Returns the Cairn-compatible exact command hash.
    #[must_use]
    pub fn command_hash(&self) -> &str {
        self.command_hash.as_str()
    }

    /// Returns selected prerequisite identifiers.
    #[must_use]
    pub fn prerequisites(&self) -> &[String] {
        self.prerequisites.as_slice()
    }
}

/// Diagnostic class for authoritative completion validation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AgentCheckDiagnosticCode {
    /// Registry shape or identity is invalid.
    RegistryInvalid,
    /// Changed artifact input is unsafe or malformed.
    ChangedArtifactInvalid,
    /// A selected check requires an unavailable capability.
    CapabilityUnavailable,
    /// A selected check exceeds the admitted tier.
    TierUnavailable,
    /// A required result row is missing.
    MissingResult,
    /// More than one result names the same required check.
    DuplicateResult,
    /// A result names a check outside the selected set.
    UnknownResult,
    /// A required status is not passing.
    NonPassingResult,
    /// Result identities or execution facts do not match selection.
    ResultMismatch,
    /// A selected prerequisite did not pass.
    PrerequisiteNotPassing,
    /// Completion candidate shape or binding is invalid.
    CandidateInvalid,
    /// Cairn summary is absent, stale, malformed, or failing.
    CairnSummaryInvalid,
    /// Diagnostic or artifact metadata is unsafe.
    UnsafeMetadata,
}

impl AgentCheckDiagnosticCode {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryInvalid => "registry-invalid",
            Self::ChangedArtifactInvalid => "changed-artifact-invalid",
            Self::CapabilityUnavailable => "capability-unavailable",
            Self::TierUnavailable => "tier-unavailable",
            Self::MissingResult => "missing-result",
            Self::DuplicateResult => "duplicate-result",
            Self::UnknownResult => "unknown-result",
            Self::NonPassingResult => "non-passing-result",
            Self::ResultMismatch => "result-mismatch",
            Self::PrerequisiteNotPassing => "prerequisite-not-passing",
            Self::CandidateInvalid => "candidate-invalid",
            Self::CairnSummaryInvalid => "cairn-summary-invalid",
            Self::UnsafeMetadata => "unsafe-metadata",
        }
    }
}

/// Safe bounded authoritative-completion diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCheckDiagnostic {
    pub(super) diagnostic_id: String,
    pub(super) code: AgentCheckDiagnosticCode,
    pub(super) check_id: Option<String>,
    pub(super) status: Option<CheckStatus>,
    pub(super) remediation_label: String,
    pub(super) summary: String,
    pub(super) subject_hash: Option<String>,
    pub(super) registry_hash: Option<String>,
    pub(super) selection_hash: Option<String>,
    pub(super) evidence_hash: Option<String>,
    pub(super) receipt_hash: Option<String>,
    pub(super) reproduction_argv: Vec<String>,
    pub(super) working_root: String,
    pub(super) source_path: Option<String>,
    pub(super) source_line: Option<u32>,
    pub(super) source_column: Option<u32>,
    pub(super) artifact_refs: Vec<String>,
    pub(super) non_claims: Vec<String>,
}

impl AgentCheckDiagnostic {
    /// Returns the deterministic diagnostic identity.
    #[must_use]
    pub fn diagnostic_id(&self) -> &str {
        self.diagnostic_id.as_str()
    }

    /// Returns the diagnostic class.
    #[must_use]
    pub const fn code(&self) -> AgentCheckDiagnosticCode {
        self.code
    }

    /// Returns the applicable check identifier.
    #[must_use]
    pub fn check_id(&self) -> Option<&str> {
        self.check_id.as_deref()
    }

    /// Returns the applicable check status.
    #[must_use]
    pub const fn status(&self) -> Option<CheckStatus> {
        self.status
    }

    /// Returns the stable remediation label.
    #[must_use]
    pub fn remediation_label(&self) -> &str {
        self.remediation_label.as_str()
    }

    /// Returns the bounded remediation summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the applicable final-subject hash.
    #[must_use]
    pub fn subject_hash(&self) -> Option<&str> {
        self.subject_hash.as_deref()
    }

    /// Returns the applicable registry hash.
    #[must_use]
    pub fn registry_hash(&self) -> Option<&str> {
        self.registry_hash.as_deref()
    }

    /// Returns the applicable selection hash.
    #[must_use]
    pub fn selection_hash(&self) -> Option<&str> {
        self.selection_hash.as_deref()
    }

    /// Returns the applicable evidence hash.
    #[must_use]
    pub fn evidence_hash(&self) -> Option<&str> {
        self.evidence_hash.as_deref()
    }

    /// Returns the applicable producer receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> Option<&str> {
        self.receipt_hash.as_deref()
    }

    /// Returns exact safe reproduction argv.
    #[must_use]
    pub fn reproduction_argv(&self) -> &[String] {
        self.reproduction_argv.as_slice()
    }

    /// Returns the safe working-root label.
    #[must_use]
    pub fn working_root(&self) -> &str {
        self.working_root.as_str()
    }

    /// Returns the optional repository-relative source path.
    #[must_use]
    pub fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }

    /// Returns the optional source line.
    #[must_use]
    pub const fn source_line(&self) -> Option<u32> {
        self.source_line
    }

    /// Returns the optional source column.
    #[must_use]
    pub const fn source_column(&self) -> Option<u32> {
        self.source_column
    }

    /// Returns safe artifact references.
    #[must_use]
    pub fn artifact_refs(&self) -> &[String] {
        self.artifact_refs.as_slice()
    }

    /// Returns bounded claim limitations attached to the diagnostic.
    #[must_use]
    pub fn non_claims(&self) -> &[String] {
        self.non_claims.as_slice()
    }
}

/// Deterministic change-aware check selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCheckSelection {
    pub(super) repository_id: String,
    pub(super) registry_id: String,
    pub(super) registry_hash: String,
    pub(super) selection_hash: String,
    pub(super) policy_hash: String,
    pub(super) changed_artifact_hash: String,
    pub(super) final_subject_hash: String,
    pub(super) required_checks: Vec<CairnRequiredCheck>,
    pub(super) not_selected: Vec<String>,
    pub(super) blockers: Vec<AgentCheckDiagnostic>,
}

impl AgentCheckSelection {
    /// Returns the repository identity.
    #[must_use]
    pub fn repository_id(&self) -> &str {
        self.repository_id.as_str()
    }

    /// Returns the registry identity.
    #[must_use]
    pub fn registry_id(&self) -> &str {
        self.registry_id.as_str()
    }

    /// Returns the registry hash.
    #[must_use]
    pub fn registry_hash(&self) -> &str {
        self.registry_hash.as_str()
    }

    /// Returns the Cairn-compatible selection hash.
    #[must_use]
    pub fn selection_hash(&self) -> &str {
        self.selection_hash.as_str()
    }

    /// Returns the pinned Cairn policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the changed-artifact snapshot hash.
    #[must_use]
    pub fn changed_artifact_hash(&self) -> &str {
        self.changed_artifact_hash.as_str()
    }

    /// Returns the final subject identity.
    #[must_use]
    pub fn final_subject_hash(&self) -> &str {
        self.final_subject_hash.as_str()
    }

    /// Returns selected required checks in prerequisite order.
    #[must_use]
    pub fn required_checks(&self) -> &[CairnRequiredCheck] {
        self.required_checks.as_slice()
    }

    /// Returns checks not selected for the snapshot.
    #[must_use]
    pub fn not_selected(&self) -> &[String] {
        self.not_selected.as_slice()
    }

    /// Returns selection blockers.
    #[must_use]
    pub fn blockers(&self) -> &[AgentCheckDiagnostic] {
        self.blockers.as_slice()
    }

    /// Returns true when selection is non-empty and unblocked.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        !self.required_checks.is_empty() && self.blockers.is_empty()
    }
}

/// Safe artifact reference supplied with one execution result.
#[derive(Clone, Copy, Debug)]
pub struct AgentCheckArtifactDraft<'a> {
    /// Repository-relative artifact path.
    pub path: &'a str,
    /// BLAKE3 artifact identity.
    pub hash: &'a str,
}

/// Optional repository-relative diagnostic source span.
#[derive(Clone, Copy, Debug)]
pub struct SourceSpanDraft<'a> {
    /// Repository-relative source path.
    pub path: &'a str,
    /// One-based source line.
    pub line: u32,
    /// One-based source column.
    pub column: u32,
}

/// Producer diagnostic metadata before safety validation.
#[derive(Clone, Copy, Debug)]
pub struct AgentCheckDiagnosticDraft<'a> {
    /// Safe remediation label.
    pub remediation_label: &'a str,
    /// Bounded summary.
    pub summary: &'a str,
    /// Exact reproduction argv.
    pub reproduction_argv: &'a [&'a str],
    /// Safe working-root label.
    pub working_root: &'a str,
    /// Optional source span.
    pub source: Option<SourceSpanDraft<'a>>,
}

/// Host-produced execution result before reusable admission.
#[derive(Clone, Copy, Debug)]
pub struct AgentCheckExecutionDraft<'a> {
    /// Required check identifier.
    pub check_id: &'a str,
    /// Exact result status.
    pub status: CheckStatus,
    /// Executed exact argv.
    pub command: &'a [&'a str],
    /// Exact command hash.
    pub command_hash: &'a str,
    /// Final subject hash observed by execution.
    pub subject_hash: &'a str,
    /// Registry hash observed by execution.
    pub registry_hash: &'a str,
    /// Selection hash observed by execution.
    pub selection_hash: &'a str,
    /// Cairn policy hash observed by execution.
    pub policy_hash: &'a str,
    /// BLAKE3 identity of bounded execution evidence.
    pub evidence_hash: &'a str,
    /// BLAKE3 identity of the host execution receipt.
    pub execution_receipt_hash: &'a str,
    /// Optional process exit code.
    pub exit_code: Option<i32>,
    /// Optional terminating signal.
    pub signal: Option<i32>,
    /// Whether execution exceeded its finite deadline.
    pub timed_out: bool,
    /// Whether process-group cleanup completed.
    pub cleanup_complete: bool,
    /// Bounded stdout byte count.
    pub stdout_bytes: u64,
    /// Bounded stderr byte count.
    pub stderr_bytes: u64,
    /// BLAKE3 stdout identity.
    pub stdout_hash: &'a str,
    /// BLAKE3 stderr identity.
    pub stderr_hash: &'a str,
    /// Safe artifact references.
    pub artifacts: &'a [AgentCheckArtifactDraft<'a>],
    /// Optional bounded producer diagnostic.
    pub diagnostic: Option<AgentCheckDiagnosticDraft<'a>>,
}

/// Admitted exact-coverage execution evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEvidenceReport {
    pub(super) passed_checks: Vec<String>,
    pub(super) non_passing_checks: Vec<(String, CheckStatus)>,
    pub(super) diagnostics: Vec<AgentCheckDiagnostic>,
    pub(super) evidence_hash: String,
}

impl ExecutionEvidenceReport {
    /// Returns passing required checks.
    #[must_use]
    pub fn passed_checks(&self) -> &[String] {
        self.passed_checks.as_slice()
    }

    /// Returns non-passing required checks and exact statuses.
    #[must_use]
    pub fn non_passing_checks(&self) -> &[(String, CheckStatus)] {
        self.non_passing_checks.as_slice()
    }

    /// Returns bounded diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[AgentCheckDiagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns aggregate execution-evidence identity.
    #[must_use]
    pub fn evidence_hash(&self) -> &str {
        self.evidence_hash.as_str()
    }

    /// Returns true only for exact fresh pass-only coverage.
    #[must_use]
    pub fn is_passing(&self) -> bool {
        self.non_passing_checks.is_empty() && self.diagnostics.is_empty()
    }
}

/// Origin of a completion request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionCandidateSource {
    /// Typed host request.
    Typed,
    /// Exact legacy Steel compatibility marker.
    SteelCompatibility,
}

/// Completion request before binding validation.
#[derive(Clone, Copy, Debug)]
pub struct AuthoritativeCompletionCandidateDraft<'a> {
    /// Candidate origin.
    pub source: CompletionCandidateSource,
    /// Exact request spelling.
    pub request: &'a str,
    /// Opaque current work identifier.
    pub work_id: &'a str,
    /// Final subject identity at request time.
    pub subject_hash: &'a str,
}

/// Validated completion candidate that carries no passing authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeCompletionCandidate {
    pub(super) source: CompletionCandidateSource,
    pub(super) request: String,
    pub(super) work_id_hash: String,
    pub(super) subject_hash: String,
    pub(super) candidate_hash: String,
}

impl AuthoritativeCompletionCandidate {
    /// Returns candidate origin.
    #[must_use]
    pub const fn source(&self) -> CompletionCandidateSource {
        self.source
    }

    /// Returns the exact admitted request spelling.
    #[must_use]
    pub fn request(&self) -> &str {
        self.request.as_str()
    }

    /// Returns the opaque work identifier hash.
    #[must_use]
    pub fn work_id_hash(&self) -> &str {
        self.work_id_hash.as_str()
    }

    /// Returns the candidate subject identity.
    #[must_use]
    pub fn subject_hash(&self) -> &str {
        self.subject_hash.as_str()
    }

    /// Returns the deterministic candidate identity.
    #[must_use]
    pub fn candidate_hash(&self) -> &str {
        self.candidate_hash.as_str()
    }
}

/// Normalized Cairn completion-validation summary before local admission.
#[derive(Clone, Copy, Debug)]
pub struct CairnCompletionSummaryDraft<'a> {
    /// Cairn completion schema.
    pub schema: &'a str,
    /// Expected authoritative classification.
    pub classification: &'a str,
    /// Repository identity.
    pub repository_id: &'a str,
    /// Registry identity.
    pub registry_id: &'a str,
    /// Registry hash.
    pub registry_hash: &'a str,
    /// Selection hash.
    pub selection_hash: &'a str,
    /// Cairn policy hash.
    pub policy_hash: &'a str,
    /// Final subject hash.
    pub final_subject_hash: &'a str,
    /// Required check identifiers.
    pub required_checks: &'a [&'a str],
    /// Passing check identifiers.
    pub passed_checks: &'a [&'a str],
    /// Number of non-passing rows retained by Cairn.
    pub non_passing_count: u32,
    /// Number of Cairn diagnostics.
    pub diagnostic_count: u32,
    /// Cairn validity bit.
    pub valid: bool,
    /// Cairn verdict label.
    pub verdict: &'a str,
    /// Cairn input identity.
    pub input_hash: &'a str,
    /// Cairn completion receipt identity.
    pub receipt_hash: &'a str,
}

/// Safe final authoritative completion receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritativeCompletionReceipt {
    pub(super) schema: String,
    pub(super) candidate_hash: String,
    pub(super) repository_id: String,
    pub(super) registry_id: String,
    pub(super) registry_hash: String,
    pub(super) selection_hash: String,
    pub(super) policy_hash: String,
    pub(super) final_subject_hash: String,
    pub(super) execution_evidence_hash: String,
    pub(super) cairn_receipt_hash: String,
    pub(super) required_checks: Vec<String>,
    pub(super) passed_checks: Vec<String>,
    pub(super) non_claims: Vec<String>,
    pub(super) authoritative_completion: bool,
    pub(super) receipt_hash: String,
}

impl AuthoritativeCompletionReceipt {
    /// Returns the Animus completion receipt schema.
    #[must_use]
    pub fn schema(&self) -> &str {
        self.schema.as_str()
    }

    /// Returns the completion candidate hash.
    #[must_use]
    pub fn candidate_hash(&self) -> &str {
        self.candidate_hash.as_str()
    }

    /// Returns the repository identity.
    #[must_use]
    pub fn repository_id(&self) -> &str {
        self.repository_id.as_str()
    }

    /// Returns the registry identity.
    #[must_use]
    pub fn registry_id(&self) -> &str {
        self.registry_id.as_str()
    }

    /// Returns the registry hash.
    #[must_use]
    pub fn registry_hash(&self) -> &str {
        self.registry_hash.as_str()
    }

    /// Returns the selection hash.
    #[must_use]
    pub fn selection_hash(&self) -> &str {
        self.selection_hash.as_str()
    }

    /// Returns the policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the final subject hash.
    #[must_use]
    pub fn final_subject_hash(&self) -> &str {
        self.final_subject_hash.as_str()
    }

    /// Returns the aggregate execution-evidence hash.
    #[must_use]
    pub fn execution_evidence_hash(&self) -> &str {
        self.execution_evidence_hash.as_str()
    }

    /// Returns the authoritative Cairn receipt hash.
    #[must_use]
    pub fn cairn_receipt_hash(&self) -> &str {
        self.cairn_receipt_hash.as_str()
    }

    /// Returns required checks.
    #[must_use]
    pub fn required_checks(&self) -> &[String] {
        self.required_checks.as_slice()
    }

    /// Returns passed checks.
    #[must_use]
    pub fn passed_checks(&self) -> &[String] {
        self.passed_checks.as_slice()
    }

    /// Returns bounded non-claims.
    #[must_use]
    pub fn non_claims(&self) -> &[String] {
        self.non_claims.as_slice()
    }

    /// Returns true only for this Cairn-gated receipt type.
    #[must_use]
    pub const fn authoritative_completion(&self) -> bool {
        self.authoritative_completion
    }

    /// Returns the deterministic receipt identity.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Pure final completion reduction outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthoritativeCompletionOutcome {
    /// Fresh pass-only host evidence and matching Cairn authority permit completion.
    Verified(Box<AuthoritativeCompletionReceipt>),
    /// Completion remains blocked with bounded diagnostics.
    Blocked(Vec<AgentCheckDiagnostic>),
}

impl AuthoritativeCompletionOutcome {
    /// Returns true only for a verified receipt-bearing outcome.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        matches!(self, Self::Verified(_))
    }

    /// Returns blockers for a blocked outcome.
    #[must_use]
    pub fn blockers(&self) -> &[AgentCheckDiagnostic] {
        match self {
            Self::Verified(_) => &[],
            Self::Blocked(diagnostics) => diagnostics.as_slice(),
        }
    }
}
