#![cfg_attr(animus_octet, allow(tigerstyle::non_trait_imports))]

use std::collections::BTreeMap;

/// BLAKE3 fingerprint used by the portfolio-search core.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PortfolioFingerprint {
    /// Hash algorithm label.
    pub algorithm: String,
    /// Lowercase hexadecimal digest.
    pub value_hex: String,
}

impl PortfolioFingerprint {
    /// Builds a fingerprint value for later validation.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Finite portfolio limits supplied by the host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioBudget {
    /// Maximum dispatch rounds.
    pub rounds: u32,
    /// Maximum registered approaches.
    pub approach_capacity: u32,
    /// Maximum aggregate tool calls requested by dispatches.
    pub tool_calls: u32,
    /// Maximum aggregate model tokens requested by dispatches.
    pub tokens: u64,
}

/// Deterministic counters consumed by portfolio transitions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PortfolioUsage {
    /// Dispatch rounds recorded.
    pub rounds: u32,
    /// Approaches registered.
    pub approaches: u32,
    /// Tool calls requested.
    pub tool_calls: u32,
    /// Model tokens requested.
    pub tokens: u64,
}

/// Proof and reasoning risks that adversarial review can cover.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PortfolioAuditRisk {
    /// Preconditions are unsatisfiable or materially narrower than claimed.
    Preconditions,
    /// The result is vacuous.
    Vacuity,
    /// Quantifiers range over the wrong domain.
    QuantifierDomain,
    /// Arithmetic bounds or overflow assumptions are unsound.
    ArithmeticBounds,
    /// Index bounds or finite-domain assumptions are unsound.
    IndexBounds,
    /// Recursive or iterative progress is not justified.
    Decreases,
    /// Executable behavior does not correspond to the specification.
    ExecSpecCorrespondence,
    /// The proposal expands trust or authority boundaries.
    TrustBoundary,
    /// Failure modes or edge cases are omitted.
    FailureModes,
    /// Evidence does not support the stated claim.
    EvidenceLinkage,
    /// A bounded domain-specific risk label.
    Named(String),
}

/// Immutable objective and review contract for one portfolio.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioContract {
    /// Safe portfolio contract identifier.
    pub contract_id: String,
    /// Content identity of the exact objective.
    pub objective: PortfolioFingerprint,
    /// Evidence classes that count as a successful bounded result.
    pub completion_evidence: Vec<String>,
    /// Known false-completion cases that must not count as success.
    pub false_completion_cases: Vec<String>,
    /// Risk classes required during adversarial audit.
    pub required_audit_risks: Vec<PortfolioAuditRisk>,
    /// Fingerprint of admitted source, authority, and retrieval policy.
    pub source_policy: PortfolioFingerprint,
    /// Fingerprint of the admitted external validator policy.
    pub validator_policy: PortfolioFingerprint,
    /// Finite work budget.
    pub budget: PortfolioBudget,
}

/// Portfolio orchestration phases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortfolioPhase {
    /// Objective and policy framing.
    Framing,
    /// Independent route exploration before cross-pollination.
    IndependentExploration,
    /// Cross-route synthesis and comparison.
    Synthesis,
    /// Evidence-driven work on underexplored or promising routes.
    TargetedRedirection,
    /// Adversarial review of one selected candidate.
    AdversarialAudit,
    /// External validation receipt is awaited.
    Validation,
    /// One bounded portfolio result has a matching pass receipt.
    Validated,
    /// Explicit portfolio budget or viable routes are exhausted.
    Exhausted,
    /// A named user, authority, policy, or tool barrier stopped progress.
    Blocked,
}

impl PortfolioPhase {
    /// Returns whether this phase is terminal for the portfolio core.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Validated | Self::Exhausted | Self::Blocked)
    }
}

/// Lifecycle state of one approach record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApproachState {
    /// Independently proposed and not yet cross-reviewed.
    Independent,
    /// Available for synthesis or targeted work.
    Active,
    /// Stopped by an exact unresolved barrier.
    Blocked,
    /// Rejected by concrete evidence.
    Falsified,
    /// Selected for adversarial audit.
    Audit,
    /// Supported by the matching bounded validation receipt.
    Validated,
}

/// Strength of a remaining route gap relative to the objective.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GapStrength {
    /// The remaining obligation is materially smaller than the objective.
    Simpler,
    /// The remaining obligation restates the objective at equivalent strength.
    Equivalent,
    /// The remaining obligation is stronger than the objective.
    Stronger,
    /// Available evidence cannot establish relative strength.
    Unknown,
}

/// Stable classes for exact route and portfolio blockers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortfolioBlockerClass {
    /// The missing step is equivalent to the objective.
    EquivalentStrength,
    /// The missing step is stronger than the objective.
    StrongerThanObjective,
    /// Relative gap strength is unknown.
    UnknownGapStrength,
    /// Required evidence is absent.
    MissingEvidence,
    /// Concrete evidence contradicts the route.
    Contradiction,
    /// The host denied required authority.
    AuthorityDenied,
    /// Policy denied the requested operation.
    PolicyDenied,
    /// A required tool is unavailable.
    ToolUnavailable,
    /// A decision must be made by the user.
    UserDecisionRequired,
}

/// Exact blocker metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioBlocker {
    /// Stable blocker class.
    pub class: PortfolioBlockerClass,
    /// Safe bounded explanation.
    pub detail: String,
    /// Optional safe evidence reference.
    pub evidence_ref: Option<String>,
}

/// One typed approach-family registry entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApproachRecord {
    /// Stable approach identifier.
    pub approach_id: String,
    /// Safe mechanism-family label.
    pub family: String,
    /// BLAKE3 identity of the canonical mechanism description.
    pub mechanism: PortfolioFingerprint,
    /// BLAKE3 identity of the route's bounded claim.
    pub claim: PortfolioFingerprint,
    /// Safe concrete artifact references.
    pub artifact_refs: Vec<String>,
    /// Safe validation and counterexample references.
    pub evidence_refs: Vec<String>,
    /// Remaining-gap strength relative to the objective.
    pub gap_strength: GapStrength,
    /// Current route state.
    pub state: ApproachState,
    /// Exact blocker for blocked or falsified routes.
    pub blocker: Option<PortfolioBlocker>,
    /// Smallest next discriminating check.
    pub next_check: String,
}

/// Host-supplied dispatch accounting and disclosed context metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioDispatch {
    /// Stable dispatch identifier.
    pub dispatch_id: String,
    /// Requested mechanism-family hint.
    pub family_hint: String,
    /// Prior approaches deliberately disclosed to the worker.
    pub disclosed_approach_ids: Vec<String>,
    /// Finite tool-call request.
    pub requested_tool_calls: u32,
    /// Finite token request.
    pub requested_tokens: u64,
}

/// Novel information required to reopen a blocked route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReopenNovelty {
    /// Optional replacement mechanism identity.
    pub mechanism: Option<PortfolioFingerprint>,
    /// Optional new safe evidence reference.
    pub evidence_ref: Option<String>,
    /// Reclassified gap strength.
    pub gap_strength: GapStrength,
    /// Updated next discriminating check.
    pub next_check: String,
}

/// Result of one required adversarial risk check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditOutcome {
    /// The candidate survived this bounded check.
    Pass,
    /// The check found a concrete contradiction or defect.
    Fail,
    /// The check could not resolve the risk.
    Unresolved,
}

/// One bounded adversarial finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioAuditFinding {
    /// Risk class under review.
    pub risk: PortfolioAuditRisk,
    /// Review outcome.
    pub outcome: AuditOutcome,
    /// Safe evidence reference supporting the outcome.
    pub evidence_ref: String,
    /// Safe bounded residual uncertainty label.
    pub residual_uncertainty: String,
}

/// Complete audit packet for one selected approach.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioAudit {
    /// Selected approach identifier.
    pub approach_id: String,
    /// Findings indexed by required risk class.
    pub findings: BTreeMap<PortfolioAuditRisk, PortfolioAuditFinding>,
}

/// External validation outcome supplied to the pure core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortfolioValidationStatus {
    /// Validator passed the bounded candidate.
    Passed,
    /// Validator rejected the bounded candidate.
    Failed,
}

/// Safe external-validation receipt consumed by the core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioValidationReceipt {
    /// Fingerprint of the portfolio contract.
    pub contract: PortfolioFingerprint,
    /// Selected approach identifier.
    pub approach_id: String,
    /// Selected mechanism identity.
    pub mechanism: PortfolioFingerprint,
    /// Fingerprint of candidate and audit evidence.
    pub evidence: PortfolioFingerprint,
    /// Identity of the admitted validator policy.
    pub validator_policy: PortfolioFingerprint,
    /// External validation outcome.
    pub status: PortfolioValidationStatus,
}

/// Immutable state passed through pure portfolio transitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioState {
    /// Immutable portfolio contract.
    pub contract: PortfolioContract,
    /// Current orchestration phase.
    pub phase: PortfolioPhase,
    /// Registry keyed by approach identifier.
    pub approaches: BTreeMap<String, ApproachRecord>,
    /// Consumed budget counters.
    pub usage: PortfolioUsage,
    /// Candidate selected for audit and validation.
    pub selected_approach: Option<String>,
    /// Recorded adversarial audit.
    pub audit: Option<PortfolioAudit>,
    /// Exact terminal portfolio blocker.
    pub terminal_blocker: Option<PortfolioBlocker>,
    /// Matching pass receipt, when validated.
    pub validation_receipt: Option<PortfolioValidationReceipt>,
}

/// Redacted terminal portfolio receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioReceipt {
    /// Domain-separated identity of this receipt.
    pub receipt_id: PortfolioFingerprint,
    /// Contract identity.
    pub contract: PortfolioFingerprint,
    /// Honest terminal phase.
    pub phase: PortfolioPhase,
    /// Final bounded usage counters.
    pub usage: PortfolioUsage,
    /// Selected approach, if any.
    pub selected_approach: Option<String>,
    /// Selected mechanism, if any.
    pub selected_mechanism: Option<PortfolioFingerprint>,
    /// Safe evidence references retained by the selected route.
    pub evidence_refs: Vec<String>,
    /// Exact safe terminal blocker, if blocked.
    pub blocker: Option<PortfolioBlocker>,
}

/// Typed portfolio failure classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortfolioErrorKind {
    /// Contract fields or bounds are invalid.
    InvalidContract,
    /// A fingerprint is malformed or uses the wrong algorithm.
    InvalidFingerprint,
    /// Metadata is blank, oversized, or contains a raw-material marker.
    UnsafeMetadata,
    /// A requested or consumed budget is invalid.
    BudgetInvalid,
    /// A finite portfolio limit was exceeded.
    BudgetExceeded,
    /// The requested event is not admitted from the current phase.
    IllegalPhase,
    /// A referenced approach does not exist.
    MissingApproach,
    /// An approach identifier already exists.
    DuplicateApproachId,
    /// A mechanism fingerprint already exists in the registry.
    DuplicateMechanism,
    /// Independent exploration disclosed prior route context.
    CrossPollination,
    /// Approach metadata violates registry invariants.
    InvalidApproach,
    /// Gap strength is not eligible for the requested transition.
    GapNotSimpler,
    /// Blocker class does not match the recorded gap.
    BlockerMismatch,
    /// A blocked route was reopened without new evidence or mechanism.
    MissingNovelty,
    /// A required adversarial risk is missing.
    AuditIncomplete,
    /// A required adversarial risk failed or remained unresolved.
    AuditFailed,
    /// A validation or terminal receipt does not match current state.
    ReceiptMismatch,
    /// A nonterminal operation targeted terminal state.
    TerminalState,
}

/// Deterministic portfolio error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioError {
    /// Stable error class.
    pub kind: PortfolioErrorKind,
    /// Field or object that failed.
    pub field: String,
    /// Safe deterministic detail.
    pub detail: String,
}

impl PortfolioError {
    /// Builds a typed error.
    #[must_use]
    pub fn new(kind: PortfolioErrorKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Result returned by pure portfolio-search operations.
pub type PortfolioResult<T> = Result<T, PortfolioError>;
