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
use std::collections::{BTreeMap, BTreeSet};

use super::{
    BLAKE3_ALGORITHM, FIELD_ARCHIVE_BEST_RUN, FIELD_ARCHIVE_IDEAS, FIELD_ARCHIVE_KEPT_RUNS,
    FIELD_BASELINE_CONFIG, FIELD_BASELINE_EVIDENCE, FIELD_BASELINE_GUARDED,
    FIELD_BASELINE_PRIMARY_METRIC, FIELD_BASELINE_STUDY, FIELD_BASELINE_WORKLOAD, FIELD_CHECKS,
    FIELD_CLAIM, FIELD_DIFF, FIELD_EXPECTED_SIGNAL, FIELD_FALSIFICATION, FIELD_GUARDED_FILES,
    FIELD_HYPOTHESIS_ID, FIELD_HYPOTHESIS_LINKS, FIELD_HYPOTHESIS_REFERENCE, FIELD_INSTRUMENTATION,
    FIELD_INTERVENTION, FIELD_METRIC_DIRECTION, FIELD_OBJECTIVE, FIELD_OFF_LIMITS,
    FIELD_PRIMARY_METRIC, FIELD_PROMOTION_DEPENDENCY, FIELD_PROMOTION_FILE, FIELD_PROMOTION_GROUP,
    FIELD_PROMOTION_RUNS, FIELD_PROVENANCE, FIELD_RISK_NOTES, FIELD_ROLLBACK,
    FIELD_RUN_CHANGED_FILES, FIELD_RUN_CHECKS, FIELD_RUN_ID, FIELD_RUN_MEASUREMENT_RISKS,
    FIELD_RUN_NEXT_ACTION, FIELD_RUN_PRIMARY_METRIC, FIELD_RUN_SEQUENCE, FIELD_RUN_STUDY,
    FIELD_SCOPE, FIELD_STUDY_ID, FIELD_WORKLOAD, HASH_PART_SEPARATOR, RESEARCHSPEC_HASH_DOMAIN,
    data::{
        ResearchSpecArchiveConclusion, ResearchSpecBaselineEvidence, ResearchSpecCheckResult,
        ResearchSpecCheckStatus, ResearchSpecDiagnostic, ResearchSpecDiagnosticKind,
        ResearchSpecFingerprint, ResearchSpecGuardedFingerprint, ResearchSpecHypothesis,
        ResearchSpecHypothesisDecision, ResearchSpecKeepDecision, ResearchSpecKeepOutcome,
        ResearchSpecMeasurementRisk, ResearchSpecMetricDirection, ResearchSpecMetricSample,
        ResearchSpecPromotionDecision, ResearchSpecPromotionGroup, ResearchSpecPromotionOutcome,
        ResearchSpecRunEvidence, ResearchSpecRunStatus, ResearchSpecStudyContract,
        ResearchSpecValidationResult,
    },
};

const FIELD_PROMOTION_GROUP_DUPLICATE: &str = "promotion_group.duplicate_group_id";
const FIELD_RUN_UNRESOLVED: &str = "run.status";

/// Computes a domain-separated BLAKE3 fingerprint for lifecycle evidence.
#[must_use]
pub fn blake3_researchspec_fingerprint(label: &str, bytes: &[u8]) -> ResearchSpecFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RESEARCHSPEC_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    ResearchSpecFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Validates a study contract before baseline or candidate evaluation.
///
/// # Errors
///
/// Returns all missing-field and empty-collection diagnostics for required
/// study contract fields.
pub fn validate_research_study_contract(
    contract: &ResearchSpecStudyContract,
) -> ResearchSpecValidationResult {
    let mut diagnostics = Vec::new();
    require_text(&contract.study_id, FIELD_STUDY_ID, &mut diagnostics);
    require_text(&contract.objective, FIELD_OBJECTIVE, &mut diagnostics);
    require_text(
        &contract.primary_metric,
        FIELD_PRIMARY_METRIC,
        &mut diagnostics,
    );
    require_text(
        &contract.workload.command_identity,
        FIELD_WORKLOAD,
        &mut diagnostics,
    );
    require_collection(&contract.correctness_checks, FIELD_CHECKS, &mut diagnostics);
    require_collection(&contract.scope_files, FIELD_SCOPE, &mut diagnostics);
    require_collection(
        &contract.off_limits_files,
        FIELD_OFF_LIMITS,
        &mut diagnostics,
    );
    require_collection(
        &contract.guarded_measurement_files,
        FIELD_GUARDED_FILES,
        &mut diagnostics,
    );
    require_text(
        &contract.provenance.source,
        FIELD_PROVENANCE,
        &mut diagnostics,
    );
    to_validation_result(diagnostics)
}

/// Validates immutable baseline evidence against a study contract.
///
/// # Errors
///
/// Returns diagnostics when the baseline lacks the configured primary metric,
/// required guarded fingerprints, passing checks, or BLAKE3 evidence hashes.
pub fn validate_research_baseline(
    study: &ResearchSpecStudyContract,
    baseline: &ResearchSpecBaselineEvidence,
) -> ResearchSpecValidationResult {
    let mut diagnostics = Vec::new();
    if baseline.study_id != study.study_id {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::UnknownReference,
            FIELD_BASELINE_STUDY,
            "baseline study_id does not match study contract",
        ));
    }
    require_metric_name(
        &baseline.primary_metric,
        &study.primary_metric,
        FIELD_BASELINE_PRIMARY_METRIC,
        &mut diagnostics,
    );
    require_collection_of_checks(&baseline.checks, FIELD_CHECKS, &mut diagnostics);
    collect_failed_checks(&baseline.checks, FIELD_CHECKS, &mut diagnostics);
    require_blake3(
        &baseline.config_fingerprint,
        FIELD_BASELINE_CONFIG,
        &mut diagnostics,
    );
    require_blake3(
        &baseline.workload_fingerprint,
        FIELD_BASELINE_WORKLOAD,
        &mut diagnostics,
    );
    require_blake3(
        &baseline.evidence_fingerprint,
        FIELD_BASELINE_EVIDENCE,
        &mut diagnostics,
    );
    require_guarded_fingerprints(
        &baseline.guarded_fingerprints,
        FIELD_BASELINE_GUARDED,
        &mut diagnostics,
    );
    to_validation_result(diagnostics)
}

/// Validates an optimization hypothesis.
///
/// # Errors
///
/// Returns diagnostics when a hypothesis lacks an actionable claim,
/// intervention, expected signal, falsification criteria, or risk notes.
pub fn validate_research_hypothesis(
    hypothesis: &ResearchSpecHypothesis,
) -> ResearchSpecValidationResult {
    let mut diagnostics = Vec::new();
    require_text(
        &hypothesis.hypothesis_id,
        FIELD_HYPOTHESIS_ID,
        &mut diagnostics,
    );
    require_hypothesis_text(&hypothesis.claim, FIELD_CLAIM, &mut diagnostics);
    require_hypothesis_text(
        &hypothesis.intervention,
        FIELD_INTERVENTION,
        &mut diagnostics,
    );
    require_hypothesis_text(
        &hypothesis.expected_signal,
        FIELD_EXPECTED_SIGNAL,
        &mut diagnostics,
    );
    require_hypothesis_text(
        &hypothesis.falsification_criteria,
        FIELD_FALSIFICATION,
        &mut diagnostics,
    );
    require_hypothesis_text(&hypothesis.risk_notes, FIELD_RISK_NOTES, &mut diagnostics);
    to_validation_result(diagnostics)
}

/// Validates host-reported run evidence.
///
/// # Errors
///
/// Returns diagnostics for missing identifiers, primary metric, BLAKE3 diff
/// fingerprint, checks, changed-file summary, guarded fingerprints, or failure
/// learning context.
pub fn validate_research_run(
    study: &ResearchSpecStudyContract,
    run: &ResearchSpecRunEvidence,
) -> ResearchSpecValidationResult {
    let mut diagnostics = Vec::new();
    require_text(&run.run_id, FIELD_RUN_ID, &mut diagnostics);
    if run.study_id != study.study_id {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::UnknownReference,
            FIELD_RUN_STUDY,
            "run study_id does not match study contract",
        ));
    }
    require_text(
        &run.hypothesis_id,
        FIELD_HYPOTHESIS_REFERENCE,
        &mut diagnostics,
    );
    require_text(
        &run.timestamp_or_sequence,
        FIELD_RUN_SEQUENCE,
        &mut diagnostics,
    );
    require_blake3(&run.diff_fingerprint, FIELD_DIFF, &mut diagnostics);
    require_metric_name(
        &run.primary_metric,
        &study.primary_metric,
        FIELD_RUN_PRIMARY_METRIC,
        &mut diagnostics,
    );
    require_collection_of_checks(&run.checks, FIELD_RUN_CHECKS, &mut diagnostics);
    require_collection(
        &run.changed_files,
        FIELD_RUN_CHANGED_FILES,
        &mut diagnostics,
    );
    require_guarded_fingerprints(
        &run.guarded_fingerprints,
        FIELD_INSTRUMENTATION,
        &mut diagnostics,
    );
    if run_requires_learning_context(run.status) {
        require_failure_context(run, &mut diagnostics);
    }
    to_validation_result(diagnostics)
}

/// Evaluates whether a candidate run can be kept against a baseline.
#[must_use]
pub fn evaluate_research_keep_gate(
    study: &ResearchSpecStudyContract,
    baseline: &ResearchSpecBaselineEvidence,
    hypothesis: &ResearchSpecHypothesis,
    run: &ResearchSpecRunEvidence,
) -> ResearchSpecKeepDecision {
    let mut diagnostics = Vec::new();
    append_result(validate_research_study_contract(study), &mut diagnostics);
    append_result(
        validate_research_baseline(study, baseline),
        &mut diagnostics,
    );
    append_result(validate_research_hypothesis(hypothesis), &mut diagnostics);
    append_result(validate_research_run(study, run), &mut diagnostics);
    validate_run_hypothesis_link(hypothesis, run, &mut diagnostics);
    collect_failed_checks(&run.checks, FIELD_RUN_CHECKS, &mut diagnostics);
    collect_measurement_risks(&run.measurement_integrity_risks, &mut diagnostics);
    collect_guarded_drift(baseline, run, &mut diagnostics);
    let metric_delta_microunits = metric_delta(study, baseline, run);
    if let Some(delta) = metric_delta_microunits {
        collect_metric_gate(study, delta, &mut diagnostics);
    }
    let outcome = if diagnostics.is_empty() {
        ResearchSpecKeepOutcome::Eligible
    } else {
        ResearchSpecKeepOutcome::Blocked
    };
    ResearchSpecKeepDecision {
        outcome,
        diagnostics,
        metric_delta_microunits,
    }
}

/// Evaluates whether kept-run promotion groups are independent or dependent.
#[must_use]
pub fn evaluate_research_promotion_groups(
    groups: &[ResearchSpecPromotionGroup],
) -> ResearchSpecPromotionDecision {
    let mut diagnostics = Vec::new();
    let group_ids = collect_group_ids(groups, &mut diagnostics);
    collect_group_shape_diagnostics(groups, &group_ids, &mut diagnostics);
    collect_group_file_overlaps(groups, &mut diagnostics);
    let has_blocking = diagnostics.iter().any(is_blocking_promotion_diagnostic);
    let has_dependency = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == ResearchSpecDiagnosticKind::PromotionGroupDependency);
    let outcome = if has_blocking {
        ResearchSpecPromotionOutcome::Blocked
    } else if has_dependency {
        ResearchSpecPromotionOutcome::Dependent
    } else {
        ResearchSpecPromotionOutcome::Independent
    };
    ResearchSpecPromotionDecision {
        outcome,
        diagnostics,
    }
}

/// Validates whether a study archive can close out all lifecycle decisions.
///
/// # Errors
///
/// Returns diagnostics when hypotheses, runs, or ideas remain unresolved or
/// archive references point to unknown run evidence.
pub fn validate_research_archive(
    study: &ResearchSpecStudyContract,
    hypotheses: &[ResearchSpecHypothesis],
    runs: &[ResearchSpecRunEvidence],
    archive: &ResearchSpecArchiveConclusion,
) -> ResearchSpecValidationResult {
    let mut diagnostics = Vec::new();
    if archive.study_id != study.study_id {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::UnknownReference,
            FIELD_STUDY_ID,
            "archive study_id does not match study contract",
        ));
    }
    require_text(&archive.objective, FIELD_OBJECTIVE, &mut diagnostics);
    require_blake3(
        &archive.baseline_fingerprint,
        FIELD_BASELINE_EVIDENCE,
        &mut diagnostics,
    );
    collect_unresolved_hypotheses(hypotheses, &mut diagnostics);
    collect_unresolved_runs(runs, &mut diagnostics);
    collect_archive_reference_diagnostics(archive, runs, &mut diagnostics);
    if !archive.unresolved_idea_ids.is_empty() {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::UnresolvedArchiveDecision,
            FIELD_ARCHIVE_IDEAS,
            "archive contains unresolved in-scope ideas",
        ));
    }
    to_validation_result(diagnostics)
}

fn append_result(
    result: ResearchSpecValidationResult,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if let Err(mut new_diagnostics) = result {
        diagnostics.append(&mut new_diagnostics);
    }
}

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn collect_archive_reference_diagnostics(
    archive: &ResearchSpecArchiveConclusion,
    runs: &[ResearchSpecRunEvidence],
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    let run_ids = runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<BTreeSet<_>>();
    if let Some(best_run_id) = archive.best_run_id.as_deref() {
        if !run_ids.contains(best_run_id) {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::UnknownReference,
                FIELD_ARCHIVE_BEST_RUN,
                "archive best_run_id does not refer to retained run evidence",
            ));
        }
    }
    for kept_run_id in &archive.kept_run_ids {
        if !run_ids.contains(kept_run_id.as_str()) {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::UnknownReference,
                FIELD_ARCHIVE_KEPT_RUNS,
                "archive kept_run_ids contains an unknown run",
            ));
        }
    }
}

fn collect_failed_checks(
    checks: &[ResearchSpecCheckResult],
    field: &str,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    for check in checks {
        if check.status == ResearchSpecCheckStatus::Failed {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::CheckFailed,
                field,
                format!("required check '{}' failed", check.name),
            ));
        }
    }
}

fn collect_group_file_overlaps(
    groups: &[ResearchSpecPromotionGroup],
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    let mut file_owner = BTreeMap::<&str, &str>::new();
    for group in groups {
        for file in &group.touched_files {
            if let Some(existing_group_id) =
                file_owner.insert(file.as_str(), group.group_id.as_str())
            {
                if existing_group_id != group.group_id {
                    diagnostics.push(ResearchSpecDiagnostic::new(
                        ResearchSpecDiagnosticKind::PromotionGroupOverlap,
                        FIELD_PROMOTION_FILE,
                        format!(
                            "promotion groups '{}' and '{}' both touch '{}'",
                            existing_group_id, group.group_id, file
                        ),
                    ));
                }
            }
        }
    }
}

fn collect_group_ids<'a>(
    groups: &'a [ResearchSpecPromotionGroup],
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) -> BTreeSet<&'a str> {
    let mut group_ids = BTreeSet::new();
    for group in groups {
        if blank(&group.group_id) {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::MissingField,
                FIELD_PROMOTION_GROUP,
                "promotion group id is required",
            ));
        } else if !group_ids.insert(group.group_id.as_str()) {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::UnknownReference,
                FIELD_PROMOTION_GROUP_DUPLICATE,
                "promotion group ids must be unique",
            ));
        }
    }
    group_ids
}

fn collect_group_shape_diagnostics(
    groups: &[ResearchSpecPromotionGroup],
    group_ids: &BTreeSet<&str>,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    for group in groups {
        require_collection(&group.run_ids, FIELD_PROMOTION_RUNS, diagnostics);
        require_collection(&group.touched_files, FIELD_PROMOTION_FILE, diagnostics);
        for dependency in &group.depends_on_group_ids {
            if group_ids.contains(dependency.as_str()) {
                diagnostics.push(ResearchSpecDiagnostic::new(
                    ResearchSpecDiagnosticKind::PromotionGroupDependency,
                    FIELD_PROMOTION_DEPENDENCY,
                    "promotion group requires dependency ordering",
                ));
            } else {
                diagnostics.push(ResearchSpecDiagnostic::new(
                    ResearchSpecDiagnosticKind::UnknownReference,
                    FIELD_PROMOTION_DEPENDENCY,
                    "promotion group depends on an unknown group",
                ));
            }
        }
    }
}

fn collect_guarded_drift(
    baseline: &ResearchSpecBaselineEvidence,
    run: &ResearchSpecRunEvidence,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    let run_guarded = run
        .guarded_fingerprints
        .iter()
        .map(|guarded| (guarded.identity.as_str(), &guarded.fingerprint))
        .collect::<BTreeMap<_, _>>();
    for baseline_guarded in &baseline.guarded_fingerprints {
        let Some(run_fingerprint) = run_guarded.get(baseline_guarded.identity.as_str()) else {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::MissingField,
                FIELD_INSTRUMENTATION,
                "candidate run lacks a baseline guarded fingerprint",
            ));
            continue;
        };
        if *run_fingerprint != &baseline_guarded.fingerprint && !run.approved_guarded_drift {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::GuardedDrift,
                FIELD_INSTRUMENTATION,
                format!("guarded input '{}' drifted", baseline_guarded.identity),
            ));
        }
    }
}

fn collect_measurement_risks(
    risks: &[ResearchSpecMeasurementRisk],
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    for risk in risks {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::MeasurementIntegrityRisk,
            FIELD_RUN_MEASUREMENT_RISKS,
            format!("measurement-integrity risk: {risk:?}"),
        ));
    }
}

fn collect_metric_gate(
    study: &ResearchSpecStudyContract,
    delta: i64,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if delta <= 0 {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::MetricDidNotImprove,
            FIELD_METRIC_DIRECTION,
            "primary metric did not improve in the configured direction",
        ));
        return;
    }
    let Ok(positive_delta) = u64::try_from(delta) else {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::MetricDidNotImprove,
            FIELD_METRIC_DIRECTION,
            "primary metric delta could not be represented as positive microunits",
        ));
        return;
    };
    let required_delta = study.threshold_policy.required_delta_microunits();
    if positive_delta < required_delta {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::ThresholdNotMet,
            FIELD_METRIC_DIRECTION,
            "primary metric improvement is below threshold or noise policy",
        ));
    }
}

fn collect_unresolved_hypotheses(
    hypotheses: &[ResearchSpecHypothesis],
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    for hypothesis in hypotheses {
        if hypothesis.decision == ResearchSpecHypothesisDecision::Undecided {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::UnresolvedArchiveDecision,
                FIELD_HYPOTHESIS_ID,
                "hypothesis has no final decision",
            ));
        }
    }
}

fn collect_unresolved_runs(
    runs: &[ResearchSpecRunEvidence],
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    for run in runs {
        if run.status == ResearchSpecRunStatus::Completed {
            diagnostics.push(ResearchSpecDiagnostic::new(
                ResearchSpecDiagnosticKind::UnresolvedArchiveDecision,
                FIELD_RUN_UNRESOLVED,
                "run is completed but not kept or discarded",
            ));
        }
    }
}

fn is_blocking_promotion_diagnostic(diagnostic: &ResearchSpecDiagnostic) -> bool {
    diagnostic.kind != ResearchSpecDiagnosticKind::PromotionGroupDependency
}

fn metric_delta(
    study: &ResearchSpecStudyContract,
    baseline: &ResearchSpecBaselineEvidence,
    run: &ResearchSpecRunEvidence,
) -> Option<i64> {
    if baseline.primary_metric.name != study.primary_metric
        || run.primary_metric.name != study.primary_metric
    {
        return None;
    }
    let baseline_value = baseline.primary_metric.value.microunits();
    let run_value = run.primary_metric.value.microunits();
    Some(match study.metric_direction {
        ResearchSpecMetricDirection::Maximize => run_value.saturating_sub(baseline_value),
        ResearchSpecMetricDirection::Minimize => baseline_value.saturating_sub(run_value),
    })
}

fn require_blake3(
    fingerprint: &ResearchSpecFingerprint,
    field: &str,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if fingerprint.algorithm != BLAKE3_ALGORITHM {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::UnexpectedAlgorithm,
            field,
            "ResearchSpec lifecycle evidence must use BLAKE3 unless an interop contract overrides it",
        ));
    }
    require_text(&fingerprint.value_hex, field, diagnostics);
}

fn require_collection(
    values: &[String],
    field: &str,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if values.is_empty() || values.iter().any(|value| blank(value)) {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::EmptyCollection,
            field,
            "required collection is empty or contains a blank entry",
        ));
    }
}

fn require_collection_of_checks(
    checks: &[ResearchSpecCheckResult],
    field: &str,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if checks.is_empty() || checks.iter().any(|check| blank(&check.name)) {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::EmptyCollection,
            field,
            "required check collection is empty or contains a blank check name",
        ));
    }
}

fn require_failure_context(
    run: &ResearchSpecRunEvidence,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if run.rollback_reason.as_deref().is_none_or(blank) {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::FailedRunMissingContext,
            FIELD_ROLLBACK,
            "discarded, crashed, or check-failed run requires rollback reason",
        ));
    }
    if run.next_action.as_deref().is_none_or(blank) {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::FailedRunMissingContext,
            FIELD_RUN_NEXT_ACTION,
            "discarded, crashed, or check-failed run requires next action",
        ));
    }
}

fn require_guarded_fingerprints(
    guarded_fingerprints: &[ResearchSpecGuardedFingerprint],
    field: &str,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if guarded_fingerprints.is_empty() {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::EmptyCollection,
            field,
            "guarded fingerprints are required",
        ));
        return;
    }
    for guarded in guarded_fingerprints {
        require_text(&guarded.identity, field, diagnostics);
        require_blake3(&guarded.fingerprint, field, diagnostics);
    }
}

fn require_hypothesis_text(text: &str, field: &str, diagnostics: &mut Vec<ResearchSpecDiagnostic>) {
    if blank(text) {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::VagueHypothesis,
            field,
            "hypothesis requires a concrete claim, intervention, expected signal, and discard condition",
        ));
    }
}

fn require_metric_name(
    metric: &ResearchSpecMetricSample,
    expected_name: &str,
    field: &str,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if blank(&metric.name) || metric.name != expected_name {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::MissingPrimaryMetric,
            field,
            "metric sample must use the configured primary metric name",
        ));
    }
}

fn require_text(text: &str, field: &str, diagnostics: &mut Vec<ResearchSpecDiagnostic>) {
    if blank(text) {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::MissingField,
            field,
            "required field is missing or blank",
        ));
    }
}

fn run_requires_learning_context(status: ResearchSpecRunStatus) -> bool {
    matches!(
        status,
        ResearchSpecRunStatus::Discarded
            | ResearchSpecRunStatus::Crashed
            | ResearchSpecRunStatus::CheckFailed
    )
}

fn to_validation_result(diagnostics: Vec<ResearchSpecDiagnostic>) -> ResearchSpecValidationResult {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_run_hypothesis_link(
    hypothesis: &ResearchSpecHypothesis,
    run: &ResearchSpecRunEvidence,
    diagnostics: &mut Vec<ResearchSpecDiagnostic>,
) {
    if hypothesis.hypothesis_id != run.hypothesis_id
        || !hypothesis
            .linked_run_ids
            .iter()
            .any(|run_id| run_id == &run.run_id)
    {
        diagnostics.push(ResearchSpecDiagnostic::new(
            ResearchSpecDiagnosticKind::TraceabilityMissing,
            FIELD_HYPOTHESIS_LINKS,
            "run must be linked from the hypothesis it evaluates",
        ));
    }
}
