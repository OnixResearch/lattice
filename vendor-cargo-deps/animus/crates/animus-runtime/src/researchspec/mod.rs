//! Pure ResearchSpec-inspired optimization lifecycle contracts.
//!
//! Hosts own benchmark execution, clocks, filesystem access, process control,
//! metric extraction, and archival storage. This module only validates already
//! loaded lifecycle DTOs and evaluates deterministic keep/promotion/archive
//! decisions.

// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while
// drained changes are archived.
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
#![allow(clippy::similar_names)]

mod core;
mod data;

pub use core::blake3_researchspec_fingerprint;
pub use core::evaluate_research_keep_gate;
pub use core::evaluate_research_promotion_groups;
pub use core::validate_research_archive;
pub use core::validate_research_baseline;
pub use core::validate_research_hypothesis;
pub use core::validate_research_run;
pub use core::validate_research_study_contract;

pub use data::ResearchSpecArchiveConclusion;
pub use data::ResearchSpecBaselineEvidence;
pub use data::ResearchSpecCheckResult;
pub use data::ResearchSpecCheckStatus;
pub use data::ResearchSpecDiagnostic;
pub use data::ResearchSpecDiagnosticKind;
pub use data::ResearchSpecFingerprint;
pub use data::ResearchSpecGuardedFingerprint;
pub use data::ResearchSpecHypothesis;
pub use data::ResearchSpecHypothesisDecision;
pub use data::ResearchSpecKeepDecision;
pub use data::ResearchSpecKeepOutcome;
pub use data::ResearchSpecMeasurementRisk;
pub use data::ResearchSpecMetricDirection;
pub use data::ResearchSpecMetricSample;
pub use data::ResearchSpecMetricValue;
pub use data::ResearchSpecPromotionDecision;
pub use data::ResearchSpecPromotionGroup;
pub use data::ResearchSpecPromotionOutcome;
pub use data::ResearchSpecProvenance;
pub use data::ResearchSpecRunEvidence;
pub use data::ResearchSpecRunStatus;
pub use data::ResearchSpecStudyContract;
pub use data::ResearchSpecThresholdPolicy;
pub use data::ResearchSpecValidationResult;
pub use data::ResearchSpecWorkloadIdentity;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const RESEARCHSPEC_HASH_DOMAIN: &str = "animus.researchspec.v1";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const FIELD_ARCHIVE_BEST_RUN: &str = "archive.best_run_id";
pub(crate) const FIELD_ARCHIVE_IDEAS: &str = "archive.unresolved_idea_ids";
pub(crate) const FIELD_ARCHIVE_KEPT_RUNS: &str = "archive.kept_run_ids";
pub(crate) const FIELD_BASELINE_CONFIG: &str = "baseline.config_fingerprint";
pub(crate) const FIELD_BASELINE_EVIDENCE: &str = "baseline.evidence_fingerprint";
pub(crate) const FIELD_BASELINE_GUARDED: &str = "baseline.guarded_fingerprints";
pub(crate) const FIELD_BASELINE_PRIMARY_METRIC: &str = "baseline.primary_metric";
pub(crate) const FIELD_BASELINE_STUDY: &str = "baseline.study_id";
pub(crate) const FIELD_BASELINE_WORKLOAD: &str = "baseline.workload_fingerprint";
pub(crate) const FIELD_CHECKS: &str = "correctness_checks";
pub(crate) const FIELD_CLAIM: &str = "hypothesis.claim";
pub(crate) const FIELD_DIFF: &str = "run.diff_fingerprint";
pub(crate) const FIELD_EXPECTED_SIGNAL: &str = "hypothesis.expected_signal";
pub(crate) const FIELD_FALSIFICATION: &str = "hypothesis.falsification_criteria";
pub(crate) const FIELD_GUARDED_FILES: &str = "guarded_measurement_files";
pub(crate) const FIELD_HYPOTHESIS_ID: &str = "hypothesis.id";
pub(crate) const FIELD_HYPOTHESIS_LINKS: &str = "hypothesis.linked_run_ids";
pub(crate) const FIELD_HYPOTHESIS_REFERENCE: &str = "run.hypothesis_id";
pub(crate) const FIELD_INSTRUMENTATION: &str = "run.guarded_fingerprints";
pub(crate) const FIELD_INTERVENTION: &str = "hypothesis.intervention";
pub(crate) const FIELD_METRIC_DIRECTION: &str = "metric_direction";
pub(crate) const FIELD_OBJECTIVE: &str = "objective";
pub(crate) const FIELD_OFF_LIMITS: &str = "off_limits_files";
pub(crate) const FIELD_PRIMARY_METRIC: &str = "primary_metric";
pub(crate) const FIELD_PROMOTION_DEPENDENCY: &str = "promotion_group.depends_on_group_ids";
pub(crate) const FIELD_PROMOTION_FILE: &str = "promotion_group.touched_files";
pub(crate) const FIELD_PROMOTION_GROUP: &str = "promotion_group.group_id";
pub(crate) const FIELD_PROMOTION_RUNS: &str = "promotion_group.run_ids";
pub(crate) const FIELD_PROVENANCE: &str = "provenance.source";
pub(crate) const FIELD_RISK_NOTES: &str = "hypothesis.risk_notes";
pub(crate) const FIELD_ROLLBACK: &str = "run.rollback_reason";
pub(crate) const FIELD_RUN_CHECKS: &str = "run.checks";
pub(crate) const FIELD_RUN_CHANGED_FILES: &str = "run.changed_files";
pub(crate) const FIELD_RUN_ID: &str = "run.run_id";
pub(crate) const FIELD_RUN_MEASUREMENT_RISKS: &str = "run.measurement_integrity_risks";
pub(crate) const FIELD_RUN_NEXT_ACTION: &str = "run.next_action";
pub(crate) const FIELD_RUN_PRIMARY_METRIC: &str = "run.primary_metric";
pub(crate) const FIELD_RUN_SEQUENCE: &str = "run.timestamp_or_sequence";
pub(crate) const FIELD_RUN_STUDY: &str = "run.study_id";
pub(crate) const FIELD_SCOPE: &str = "scope_files";
pub(crate) const FIELD_STUDY_ID: &str = "study_id";
pub(crate) const FIELD_WORKLOAD: &str = "workload.command_identity";

#[cfg(test)]
mod tests;
