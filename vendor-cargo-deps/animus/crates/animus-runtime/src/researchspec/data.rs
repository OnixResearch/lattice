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
/// Fixed-point metric value expressed in metric-owned microunits.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ResearchSpecMetricValue {
    microunits: i64,
}

impl ResearchSpecMetricValue {
    /// Builds a deterministic fixed-point value from metric microunits.
    #[must_use]
    pub const fn from_microunits(microunits: i64) -> Self {
        Self { microunits }
    }

    /// Returns the raw fixed-point microunit value.
    #[must_use]
    pub const fn microunits(self) -> i64 {
        self.microunits
    }
}

/// Direction that defines whether a larger or smaller primary metric is better.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchSpecMetricDirection {
    /// Larger primary metric values are better.
    Maximize,
    /// Smaller primary metric values are better.
    Minimize,
}

/// Threshold or noise policy for deciding whether a metric delta is meaningful.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchSpecThresholdPolicy {
    /// Require at least this absolute primary-metric delta in microunits.
    MinimumDeltaMicrounits(u64),
    /// Treat this noise floor as the minimum meaningful delta in microunits.
    NoiseFloorMicrounits(u64),
}

impl ResearchSpecThresholdPolicy {
    /// Returns the configured minimum improvement in microunits.
    #[must_use]
    pub const fn required_delta_microunits(self) -> u64 {
        match self {
            Self::MinimumDeltaMicrounits(delta) | Self::NoiseFloorMicrounits(delta) => delta,
        }
    }
}

/// Stable BLAKE3 or interoperability fingerprint for lifecycle evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecFingerprint {
    /// Name of the hash algorithm used for the value.
    pub algorithm: String,
    /// Lowercase hex digest emitted by the hashing algorithm.
    pub value_hex: String,
}

impl ResearchSpecFingerprint {
    /// Builds a fingerprint from already-computed parts.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Host-supplied provenance for a study contract or lifecycle record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecProvenance {
    /// Human-meaningful origin of the contract or evidence.
    pub source: String,
    /// Optional owner, author, or agent label.
    pub author: Option<String>,
    /// Optional rationale explaining why the lifecycle exists.
    pub rationale: Option<String>,
}

/// Backend-neutral identity for a host-owned workload command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecWorkloadIdentity {
    /// Stable command identity, not a process invocation owned by runtime core.
    pub command_identity: String,
    /// Optional BLAKE3 fingerprint for command arguments or workload inputs.
    pub arguments_fingerprint: Option<ResearchSpecFingerprint>,
}

/// Typed ResearchSpec-inspired study contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecStudyContract {
    /// Stable study identifier.
    pub study_id: String,
    /// Optimization objective under study.
    pub objective: String,
    /// Primary metric name that all baselines and runs must report.
    pub primary_metric: String,
    /// Direction that determines whether metric deltas improve.
    pub metric_direction: ResearchSpecMetricDirection,
    /// Threshold or noise policy for keep decisions.
    pub threshold_policy: ResearchSpecThresholdPolicy,
    /// Host-owned workload command identity.
    pub workload: ResearchSpecWorkloadIdentity,
    /// Correctness checks required before a run can be kept.
    pub correctness_checks: Vec<String>,
    /// Files or surfaces explicitly in optimization scope.
    pub scope_files: Vec<String>,
    /// Files or surfaces the study must not modify.
    pub off_limits_files: Vec<String>,
    /// Guarded measurement files or inputs used to detect benchmark drift.
    pub guarded_measurement_files: Vec<String>,
    /// Promotion threshold for reporting a kept run as promotion-eligible.
    pub promotion_threshold: ResearchSpecThresholdPolicy,
    /// Human and machine provenance for the contract.
    pub provenance: ResearchSpecProvenance,
}

/// Named metric sample using deterministic fixed-point units.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecMetricSample {
    /// Metric name.
    pub name: String,
    /// Metric value in metric-owned microunits.
    pub value: ResearchSpecMetricValue,
}

/// Result of a host-owned correctness check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchSpecCheckStatus {
    /// The check passed.
    Passed,
    /// The check failed.
    Failed,
}

/// Named host check result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecCheckResult {
    /// Check name.
    pub name: String,
    /// Check status.
    pub status: ResearchSpecCheckStatus,
}

/// Fingerprint for a guarded file or measurement input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecGuardedFingerprint {
    /// Guarded path or input identity.
    pub identity: String,
    /// Fingerprint of the guarded content or input descriptor.
    pub fingerprint: ResearchSpecFingerprint,
}

/// Immutable baseline evidence for a study.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecBaselineEvidence {
    /// Study identifier the baseline belongs to.
    pub study_id: String,
    /// Required primary metric sample.
    pub primary_metric: ResearchSpecMetricSample,
    /// Optional secondary metric samples.
    pub secondary_metrics: Vec<ResearchSpecMetricSample>,
    /// Host-owned correctness check results.
    pub checks: Vec<ResearchSpecCheckResult>,
    /// Fingerprint for the study or benchmark configuration snapshot.
    pub config_fingerprint: ResearchSpecFingerprint,
    /// Fingerprint for the workload command or input descriptor.
    pub workload_fingerprint: ResearchSpecFingerprint,
    /// Fingerprints for guarded measurement files or inputs.
    pub guarded_fingerprints: Vec<ResearchSpecGuardedFingerprint>,
    /// Fingerprint for the complete baseline evidence record.
    pub evidence_fingerprint: ResearchSpecFingerprint,
}

/// Final decision attached to a hypothesis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchSpecHypothesisDecision {
    /// No final decision has been recorded yet.
    Undecided,
    /// The hypothesis produced a kept change.
    Kept,
    /// The hypothesis was rejected after evaluation.
    Discarded,
    /// The hypothesis was superseded by another hypothesis.
    Superseded,
    /// The hypothesis is explicitly deferred.
    Deferred,
    /// The study owner waived the hypothesis with rationale outside this DTO.
    Waived,
}

/// First-class optimization hypothesis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecHypothesis {
    /// Stable hypothesis identifier.
    pub hypothesis_id: String,
    /// Concrete claim being tested.
    pub claim: String,
    /// Planned intervention.
    pub intervention: String,
    /// Expected measurable signal.
    pub expected_signal: String,
    /// Concrete discard or falsification condition.
    pub falsification_criteria: String,
    /// Risks or caveats that reviewers should inspect.
    pub risk_notes: String,
    /// Run identifiers linked to this hypothesis.
    pub linked_run_ids: Vec<String>,
    /// Current or final hypothesis decision.
    pub decision: ResearchSpecHypothesisDecision,
}

/// Measurement-integrity risk labels used by keep gates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResearchSpecMeasurementRisk {
    /// Metric extraction changed during the candidate run.
    MetricExtractorChanged,
    /// Benchmark or workload input coverage was reduced.
    BenchmarkInputExcluded,
    /// Workload inputs were hardcoded by the candidate.
    WorkloadInputHardcoded,
    /// Measured work was deferred outside the measured region.
    MeasuredWorkDeferred,
    /// A guarded file changed without approved instrumentation metadata.
    GuardedFileEdited,
    /// Correctness checks were weakened.
    CheckWeakened,
    /// A repo-specific risk label not covered by the standard taxonomy.
    Other(String),
}

/// Candidate run status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchSpecRunStatus {
    /// The run completed and awaits keep or discard decision.
    Completed,
    /// The run was kept.
    Kept,
    /// The run was discarded.
    Discarded,
    /// The run crashed.
    Crashed,
    /// The run completed but failed required checks.
    CheckFailed,
}

/// Host-reported run evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecRunEvidence {
    /// Stable run identifier.
    pub run_id: String,
    /// Study identifier the run belongs to.
    pub study_id: String,
    /// Hypothesis identifier the run evaluates.
    pub hypothesis_id: String,
    /// Host-provided timestamp or monotonic sequence label.
    pub timestamp_or_sequence: String,
    /// Fingerprint of the candidate revision or diff.
    pub diff_fingerprint: ResearchSpecFingerprint,
    /// Required primary metric sample.
    pub primary_metric: ResearchSpecMetricSample,
    /// Optional secondary metrics.
    pub secondary_metrics: Vec<ResearchSpecMetricSample>,
    /// Candidate run status.
    pub status: ResearchSpecRunStatus,
    /// Correctness check results for the candidate.
    pub checks: Vec<ResearchSpecCheckResult>,
    /// Human-readable changed-file summary.
    pub changed_files: Vec<String>,
    /// Side information safe for durable lifecycle records.
    pub side_information: Vec<String>,
    /// Guarded file/input fingerprints observed during the run.
    pub guarded_fingerprints: Vec<ResearchSpecGuardedFingerprint>,
    /// Whether guarded drift has approved instrumentation-change metadata.
    pub approved_guarded_drift: bool,
    /// Measurement-integrity risks found by the host or reviewer.
    pub measurement_integrity_risks: Vec<ResearchSpecMeasurementRisk>,
    /// Required learning when the run is discarded, crashed, or check-failed.
    pub rollback_reason: Option<String>,
    /// Required next action when the run is discarded, crashed, or check-failed.
    pub next_action: Option<String>,
}

/// Keep-gate outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchSpecKeepOutcome {
    /// The run satisfies the deterministic keep gate.
    Eligible,
    /// The run is blocked by one or more diagnostics.
    Blocked,
}

/// Deterministic keep-gate decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecKeepDecision {
    /// Gate outcome.
    pub outcome: ResearchSpecKeepOutcome,
    /// Diagnostics that explain blocked or notable decisions.
    pub diagnostics: Vec<ResearchSpecDiagnostic>,
    /// Improvement delta in primary-metric microunits when computable.
    pub metric_delta_microunits: Option<i64>,
}

/// Promotion-group candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecPromotionGroup {
    /// Stable promotion group identifier.
    pub group_id: String,
    /// Kept run identifiers in the group.
    pub run_ids: Vec<String>,
    /// Files touched by the proposed promotion group.
    pub touched_files: Vec<String>,
    /// Other promotion groups this group depends on.
    pub depends_on_group_ids: Vec<String>,
}

/// Promotion-group evaluation outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchSpecPromotionOutcome {
    /// Groups are independent and may be promoted separately.
    Independent,
    /// Groups are non-overlapping but dependency ordering is required.
    Dependent,
    /// Groups overlap or contain invalid references.
    Blocked,
}

/// Deterministic promotion-group decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecPromotionDecision {
    /// Promotion evaluation outcome.
    pub outcome: ResearchSpecPromotionOutcome,
    /// Diagnostics describing overlaps, dependencies, or invalid groups.
    pub diagnostics: Vec<ResearchSpecDiagnostic>,
}

/// Completed study archive conclusion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecArchiveConclusion {
    /// Study identifier being archived.
    pub study_id: String,
    /// Archived objective text.
    pub objective: String,
    /// Baseline evidence fingerprint.
    pub baseline_fingerprint: ResearchSpecFingerprint,
    /// Optional best kept run identifier.
    pub best_run_id: Option<String>,
    /// Kept run identifiers.
    pub kept_run_ids: Vec<String>,
    /// Discarded hypothesis identifiers.
    pub discarded_hypothesis_ids: Vec<String>,
    /// Final primary-metric delta in microunits.
    pub final_metric_delta_microunits: Option<i64>,
    /// In-scope ideas still without keep, discard, supersede, defer, or waive decisions.
    pub unresolved_idea_ids: Vec<String>,
    /// Downstream lifecycle links.
    pub downstream_links: Vec<String>,
    /// Run evidence identifiers retained for audit.
    pub run_evidence_ids: Vec<String>,
}

/// Machine-readable diagnostic class for `ResearchSpec` validation and gates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResearchSpecDiagnosticKind {
    /// A required scalar field is missing or blank.
    MissingField,
    /// A required collection is empty or contains a blank entry.
    EmptyCollection,
    /// Evidence uses an unsupported hash algorithm.
    UnexpectedAlgorithm,
    /// A baseline or run does not report the configured primary metric.
    MissingPrimaryMetric,
    /// A hypothesis is not actionable enough to run.
    VagueHypothesis,
    /// A discarded, crashed, or check-failed run lacks learning context.
    FailedRunMissingContext,
    /// A required correctness check failed.
    CheckFailed,
    /// The primary metric did not move in the configured direction.
    MetricDidNotImprove,
    /// Improvement did not satisfy the configured threshold or noise policy.
    ThresholdNotMet,
    /// A run is not traceable to its hypothesis.
    TraceabilityMissing,
    /// A guarded measurement file or input drifted without approval.
    GuardedDrift,
    /// A measurement-integrity risk blocks keep or promotion.
    MeasurementIntegrityRisk,
    /// Promotion groups overlap and must merge or be marked dependent.
    PromotionGroupOverlap,
    /// Promotion groups are non-overlapping but explicitly dependent.
    PromotionGroupDependency,
    /// An archive still has unresolved hypotheses, runs, or ideas.
    UnresolvedArchiveDecision,
    /// A lifecycle reference points to an unknown study, run, or group.
    UnknownReference,
}

/// Validation or gate diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResearchSpecDiagnostic {
    /// Diagnostic class.
    pub kind: ResearchSpecDiagnosticKind,
    /// Field or relation that triggered the diagnostic.
    pub field: String,
    /// Human-readable safe detail.
    pub detail: String,
}

impl ResearchSpecDiagnostic {
    /// Builds a diagnostic with a stable field name.
    #[must_use]
    pub fn new(
        kind: ResearchSpecDiagnosticKind,
        field: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Validation result that returns all deterministic diagnostics found.
pub type ResearchSpecValidationResult = Result<(), Vec<ResearchSpecDiagnostic>>;
