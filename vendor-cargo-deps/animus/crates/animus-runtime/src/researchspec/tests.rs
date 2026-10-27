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

use super::*;

const STUDY_ID: &str = "study.latency";
const OBJECTIVE: &str = "Reduce deterministic turn latency without weakening checks";
const PRIMARY_METRIC: &str = "turn_latency_micros";
const WORKLOAD: &str = "bench.turn.latency";
const CHECK: &str = "cargo test -p animus-runtime researchspec";
const SCOPE_FILE: &str = "crates/animus-runtime/src/researchspec";
const OFF_LIMITS_FILE: &str = "benches/fixtures/turn-inputs.jsonl";
const GUARDED_FILE: &str = "benches/fixtures/turn-inputs.jsonl";
const SOURCE: &str = "cairn/change/adapt-hermes-researchspec-lifecycle";
const AUTHOR: &str = "animus-test";
const RATIONALE: &str = "exercise ResearchSpec lifecycle gates";
const ARGS_LABEL: &str = "args";
const CONFIG_LABEL: &str = "config";
const WORKLOAD_LABEL: &str = "workload";
const GUARDED_LABEL: &str = "guarded";
const BASELINE_LABEL: &str = "baseline";
const DIFF_LABEL: &str = "diff";
const CHANGED_FILE: &str = "crates/animus-runtime/src/researchspec/core.rs";
const SIDE_INFO: &str = "local deterministic fixture";
const HYPOTHESIS_ID: &str = "h.reduce-validation-cost";
const RUN_ID: &str = "run.candidate";
const RUN_SEQUENCE: &str = "seq-0001";
const CLAIM: &str = "Caching validation shape lowers latency";
const INTERVENTION: &str = "Precompute immutable validation fields";
const EXPECTED_SIGNAL: &str = "Primary latency metric improves beyond threshold";
const FALSIFICATION: &str = "Discard if checks fail or latency regresses";
const RISK_NOTES: &str = "Guard benchmark inputs and check definitions";
const ROLLBACK_REASON: &str = "Candidate changed measurement inputs";
const NEXT_ACTION: &str = "Restore inputs and retry with isolated change";
const GROUP_A: &str = "group.a";
const GROUP_B: &str = "group.b";
const GROUP_A_FILE: &str = "src/a.rs";
const GROUP_B_FILE: &str = "src/b.rs";
const IDEA_ID: &str = "idea.unresolved";
const DOWNSTREAM_LINK: &str = ".cairn/archive/example";
const BASELINE_METRIC: i64 = 1_000_000;
const BETTER_METRIC: i64 = 1_200_000;
const SMALL_IMPROVEMENT_METRIC: i64 = 1_050_000;
const WORSE_METRIC: i64 = 900_000;
const MINIMIZE_BASELINE_METRIC: i64 = 2_000_000;
const MINIMIZE_BETTER_METRIC: i64 = 1_700_000;
const THRESHOLD_MICROUNITS: u64 = 100_000;
const PROMOTION_THRESHOLD_MICROUNITS: u64 = 150_000;
const FINAL_DELTA_MICROUNITS: i64 = 200_000;

#[test]
fn complete_study_contract_validates_and_names_missing_fields() {
    // r[verify agent_harness_runtime.hermes_researchspec.study_contract.valid]
    // r[verify agent_harness_runtime.hermes_researchspec.study_contract.incomplete]
    let valid = study();
    assert!(validate_research_study_contract(&valid).is_ok());

    let invalid = ResearchSpecStudyContract {
        primary_metric: String::new(),
        correctness_checks: Vec::new(),
        scope_files: vec![String::new()],
        off_limits_files: Vec::new(),
        ..valid
    };
    let diagnostics = validate_research_study_contract(&invalid).expect_err("invalid study");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::MissingField
            && diagnostic.field == FIELD_PRIMARY_METRIC
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::EmptyCollection
            && diagnostic.field == FIELD_CHECKS
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::EmptyCollection
            && diagnostic.field == FIELD_SCOPE
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::EmptyCollection
            && diagnostic.field == FIELD_OFF_LIMITS
    }));
}

#[test]
fn baseline_records_guarded_fingerprints_and_rejects_missing_primary_metric() {
    // r[verify agent_harness_runtime.hermes_researchspec.baseline.fingerprints]
    // r[verify agent_harness_runtime.hermes_researchspec.baseline.missing_metric]
    let study = study();
    let valid = baseline(&study);
    assert!(validate_research_baseline(&study, &valid).is_ok());
    assert_eq!(valid.guarded_fingerprints[0].identity, GUARDED_FILE);
    assert_eq!(
        valid.guarded_fingerprints[0].fingerprint.algorithm,
        BLAKE3_ALGORITHM
    );

    let invalid = ResearchSpecBaselineEvidence {
        primary_metric: metric("wrong_metric", BASELINE_METRIC),
        ..valid
    };
    let diagnostics = validate_research_baseline(&study, &invalid).expect_err("missing metric");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::MissingPrimaryMetric
            && diagnostic.field == FIELD_BASELINE_PRIMARY_METRIC
    }));
}

#[test]
fn vague_hypothesis_is_rejected_with_actionable_diagnostic() {
    // r[verify agent_harness_runtime.hermes_researchspec.hypotheses.vague]
    let valid = hypothesis(ResearchSpecHypothesisDecision::Undecided);
    assert!(validate_research_hypothesis(&valid).is_ok());

    let vague = ResearchSpecHypothesis {
        expected_signal: String::new(),
        falsification_criteria: String::new(),
        ..valid
    };
    let diagnostics = validate_research_hypothesis(&vague).expect_err("vague hypothesis");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::VagueHypothesis
            && diagnostic.field == FIELD_EXPECTED_SIGNAL
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::VagueHypothesis
            && diagnostic.field == FIELD_FALSIFICATION
    }));
}

#[test]
fn failed_run_requires_rollback_reason_and_next_action() {
    // r[verify agent_harness_runtime.hermes_researchspec.runs.failed_requires_context]
    let study = study();
    let valid = run(&study, BETTER_METRIC, Vec::new(), guarded(GUARDED_LABEL));
    assert!(validate_research_run(&study, &valid).is_ok());

    let failed = ResearchSpecRunEvidence {
        status: ResearchSpecRunStatus::CheckFailed,
        rollback_reason: None,
        next_action: None,
        ..valid
    };
    let diagnostics = validate_research_run(&study, &failed).expect_err("missing failure context");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::FailedRunMissingContext
            && diagnostic.field == FIELD_ROLLBACK
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::FailedRunMissingContext
            && diagnostic.field == FIELD_RUN_NEXT_ACTION
    }));
}

#[test]
fn keep_gate_accepts_clean_improvement_and_blocks_suspicious_changes() {
    // r[verify agent_harness_runtime.hermes_researchspec.keep_promotion.suspicious]
    let study = study();
    let baseline = baseline(&study);
    let hypothesis = linked_hypothesis(ResearchSpecHypothesisDecision::Undecided);
    let clean_run = run(&study, BETTER_METRIC, Vec::new(), guarded(GUARDED_LABEL));

    let decision = evaluate_research_keep_gate(&study, &baseline, &hypothesis, &clean_run);
    assert_eq!(decision.outcome, ResearchSpecKeepOutcome::Eligible);
    assert_eq!(
        decision.metric_delta_microunits,
        Some(FINAL_DELTA_MICROUNITS)
    );

    let suspicious = ResearchSpecRunEvidence {
        primary_metric: metric(PRIMARY_METRIC, SMALL_IMPROVEMENT_METRIC),
        guarded_fingerprints: vec![guarded("changed")],
        measurement_integrity_risks: vec![ResearchSpecMeasurementRisk::MetricExtractorChanged],
        ..clean_run
    };
    let blocked = evaluate_research_keep_gate(&study, &baseline, &hypothesis, &suspicious);

    assert_eq!(blocked.outcome, ResearchSpecKeepOutcome::Blocked);
    assert!(blocked.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::MeasurementIntegrityRisk
    }));
    assert!(
        blocked
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ResearchSpecDiagnosticKind::GuardedDrift })
    );
    assert!(
        blocked
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == ResearchSpecDiagnosticKind::ThresholdNotMet })
    );
}

#[test]
fn keep_gate_honors_minimize_direction() {
    // r[verify agent_harness_runtime.hermes_researchspec.keep_promotion]
    let study = ResearchSpecStudyContract {
        metric_direction: ResearchSpecMetricDirection::Minimize,
        threshold_policy: ResearchSpecThresholdPolicy::NoiseFloorMicrounits(THRESHOLD_MICROUNITS),
        ..study()
    };
    let baseline = ResearchSpecBaselineEvidence {
        primary_metric: metric(PRIMARY_METRIC, MINIMIZE_BASELINE_METRIC),
        ..baseline(&study)
    };
    let hypothesis = linked_hypothesis(ResearchSpecHypothesisDecision::Undecided);
    let run = run(
        &study,
        MINIMIZE_BETTER_METRIC,
        Vec::new(),
        guarded(GUARDED_LABEL),
    );

    let decision = evaluate_research_keep_gate(&study, &baseline, &hypothesis, &run);

    assert_eq!(decision.outcome, ResearchSpecKeepOutcome::Eligible);
    assert_eq!(
        decision.metric_delta_microunits,
        Some(MINIMIZE_BASELINE_METRIC - MINIMIZE_BETTER_METRIC)
    );
}

#[test]
fn keep_gate_rejects_metric_regression() {
    // r[verify agent_harness_runtime.hermes_researchspec.keep_promotion]
    let study = study();
    let baseline = baseline(&study);
    let hypothesis = linked_hypothesis(ResearchSpecHypothesisDecision::Undecided);
    let regressed_run = run(&study, WORSE_METRIC, Vec::new(), guarded(GUARDED_LABEL));

    let decision = evaluate_research_keep_gate(&study, &baseline, &hypothesis, &regressed_run);

    assert_eq!(decision.outcome, ResearchSpecKeepOutcome::Blocked);
    assert!(
        decision.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == ResearchSpecDiagnosticKind::MetricDidNotImprove
        })
    );
}

#[test]
fn promotion_groups_detect_independent_dependent_and_overlapping_groups() {
    // r[verify agent_harness_runtime.hermes_researchspec.keep_promotion.overlap]
    let independent = evaluate_research_promotion_groups(&[
        promotion_group(GROUP_A, RUN_ID, GROUP_A_FILE, Vec::new()),
        promotion_group(GROUP_B, "run.other", GROUP_B_FILE, Vec::new()),
    ]);
    assert_eq!(
        independent.outcome,
        ResearchSpecPromotionOutcome::Independent
    );
    assert!(independent.diagnostics.is_empty());

    let dependent = evaluate_research_promotion_groups(&[
        promotion_group(GROUP_A, RUN_ID, GROUP_A_FILE, Vec::new()),
        promotion_group(
            GROUP_B,
            "run.other",
            GROUP_B_FILE,
            vec![GROUP_A.to_string()],
        ),
    ]);
    assert_eq!(dependent.outcome, ResearchSpecPromotionOutcome::Dependent);
    assert!(dependent.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::PromotionGroupDependency
    }));

    let overlapping = evaluate_research_promotion_groups(&[
        promotion_group(GROUP_A, RUN_ID, GROUP_A_FILE, Vec::new()),
        promotion_group(GROUP_B, "run.other", GROUP_A_FILE, Vec::new()),
    ]);
    assert_eq!(overlapping.outcome, ResearchSpecPromotionOutcome::Blocked);
    assert!(overlapping.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::PromotionGroupOverlap
    }));
}

#[test]
fn archive_rejects_unresolved_hypotheses_runs_and_ideas() {
    // r[verify agent_harness_runtime.hermes_researchspec.archive.unresolved]
    let study = study();
    let baseline = baseline(&study);
    let kept_run = ResearchSpecRunEvidence {
        status: ResearchSpecRunStatus::Kept,
        ..run(&study, BETTER_METRIC, Vec::new(), guarded(GUARDED_LABEL))
    };
    let kept_hypothesis = linked_hypothesis(ResearchSpecHypothesisDecision::Kept);
    let valid_archive = archive(&study, &baseline, Some(RUN_ID), Vec::new());
    assert!(
        validate_research_archive(
            &study,
            &[kept_hypothesis],
            std::slice::from_ref(&kept_run),
            &valid_archive,
        )
        .is_ok()
    );

    let unresolved_hypothesis = linked_hypothesis(ResearchSpecHypothesisDecision::Undecided);
    let unresolved_run = ResearchSpecRunEvidence {
        status: ResearchSpecRunStatus::Completed,
        ..kept_run
    };
    let invalid_archive = archive(
        &study,
        &baseline,
        Some("missing.run"),
        vec![IDEA_ID.to_string()],
    );
    let diagnostics = validate_research_archive(
        &study,
        &[unresolved_hypothesis],
        &[unresolved_run],
        &invalid_archive,
    )
    .expect_err("unresolved archive");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::UnresolvedArchiveDecision
            && diagnostic.field == FIELD_HYPOTHESIS_ID
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::UnresolvedArchiveDecision
            && diagnostic.field == FIELD_ARCHIVE_IDEAS
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == ResearchSpecDiagnosticKind::UnknownReference
            && diagnostic.field == FIELD_ARCHIVE_BEST_RUN
    }));
}

fn archive(
    study: &ResearchSpecStudyContract,
    baseline: &ResearchSpecBaselineEvidence,
    best_run_id: Option<&str>,
    unresolved_idea_ids: Vec<String>,
) -> ResearchSpecArchiveConclusion {
    ResearchSpecArchiveConclusion {
        study_id: study.study_id.clone(),
        objective: study.objective.clone(),
        baseline_fingerprint: baseline.evidence_fingerprint.clone(),
        best_run_id: best_run_id.map(str::to_string),
        kept_run_ids: vec![RUN_ID.to_string()],
        discarded_hypothesis_ids: Vec::new(),
        final_metric_delta_microunits: Some(FINAL_DELTA_MICROUNITS),
        unresolved_idea_ids,
        downstream_links: vec![DOWNSTREAM_LINK.to_string()],
        run_evidence_ids: vec![RUN_ID.to_string()],
    }
}

fn baseline(study: &ResearchSpecStudyContract) -> ResearchSpecBaselineEvidence {
    ResearchSpecBaselineEvidence {
        study_id: study.study_id.clone(),
        primary_metric: metric(PRIMARY_METRIC, BASELINE_METRIC),
        secondary_metrics: Vec::new(),
        checks: vec![passing_check()],
        config_fingerprint: fp(CONFIG_LABEL),
        workload_fingerprint: fp(WORKLOAD_LABEL),
        guarded_fingerprints: vec![guarded(GUARDED_LABEL)],
        evidence_fingerprint: fp(BASELINE_LABEL),
    }
}

fn fp(label: &str) -> ResearchSpecFingerprint {
    blake3_researchspec_fingerprint(label, label.as_bytes())
}

fn guarded(label: &str) -> ResearchSpecGuardedFingerprint {
    ResearchSpecGuardedFingerprint {
        identity: GUARDED_FILE.to_string(),
        fingerprint: fp(label),
    }
}

fn hypothesis(decision: ResearchSpecHypothesisDecision) -> ResearchSpecHypothesis {
    ResearchSpecHypothesis {
        hypothesis_id: HYPOTHESIS_ID.to_string(),
        claim: CLAIM.to_string(),
        intervention: INTERVENTION.to_string(),
        expected_signal: EXPECTED_SIGNAL.to_string(),
        falsification_criteria: FALSIFICATION.to_string(),
        risk_notes: RISK_NOTES.to_string(),
        linked_run_ids: Vec::new(),
        decision,
    }
}

fn linked_hypothesis(decision: ResearchSpecHypothesisDecision) -> ResearchSpecHypothesis {
    ResearchSpecHypothesis {
        linked_run_ids: vec![RUN_ID.to_string()],
        ..hypothesis(decision)
    }
}

fn metric(name: &str, value: i64) -> ResearchSpecMetricSample {
    ResearchSpecMetricSample {
        name: name.to_string(),
        value: ResearchSpecMetricValue::from_microunits(value),
    }
}

fn passing_check() -> ResearchSpecCheckResult {
    ResearchSpecCheckResult {
        name: CHECK.to_string(),
        status: ResearchSpecCheckStatus::Passed,
    }
}

fn promotion_group(
    group_id: &str,
    run_id: &str,
    file: &str,
    depends_on_group_ids: Vec<String>,
) -> ResearchSpecPromotionGroup {
    ResearchSpecPromotionGroup {
        group_id: group_id.to_string(),
        run_ids: vec![run_id.to_string()],
        touched_files: vec![file.to_string()],
        depends_on_group_ids,
    }
}

fn run(
    study: &ResearchSpecStudyContract,
    primary_metric_value: i64,
    measurement_integrity_risks: Vec<ResearchSpecMeasurementRisk>,
    guarded_fingerprint: ResearchSpecGuardedFingerprint,
) -> ResearchSpecRunEvidence {
    ResearchSpecRunEvidence {
        run_id: RUN_ID.to_string(),
        study_id: study.study_id.clone(),
        hypothesis_id: HYPOTHESIS_ID.to_string(),
        timestamp_or_sequence: RUN_SEQUENCE.to_string(),
        diff_fingerprint: fp(DIFF_LABEL),
        primary_metric: metric(PRIMARY_METRIC, primary_metric_value),
        secondary_metrics: Vec::new(),
        status: ResearchSpecRunStatus::Completed,
        checks: vec![passing_check()],
        changed_files: vec![CHANGED_FILE.to_string()],
        side_information: vec![SIDE_INFO.to_string()],
        guarded_fingerprints: vec![guarded_fingerprint],
        approved_guarded_drift: false,
        measurement_integrity_risks,
        rollback_reason: Some(ROLLBACK_REASON.to_string()),
        next_action: Some(NEXT_ACTION.to_string()),
    }
}

fn study() -> ResearchSpecStudyContract {
    ResearchSpecStudyContract {
        study_id: STUDY_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        primary_metric: PRIMARY_METRIC.to_string(),
        metric_direction: ResearchSpecMetricDirection::Maximize,
        threshold_policy: ResearchSpecThresholdPolicy::MinimumDeltaMicrounits(THRESHOLD_MICROUNITS),
        workload: ResearchSpecWorkloadIdentity {
            command_identity: WORKLOAD.to_string(),
            arguments_fingerprint: Some(fp(ARGS_LABEL)),
        },
        correctness_checks: vec![CHECK.to_string()],
        scope_files: vec![SCOPE_FILE.to_string()],
        off_limits_files: vec![OFF_LIMITS_FILE.to_string()],
        guarded_measurement_files: vec![GUARDED_FILE.to_string()],
        promotion_threshold: ResearchSpecThresholdPolicy::MinimumDeltaMicrounits(
            PROMOTION_THRESHOLD_MICROUNITS,
        ),
        provenance: ResearchSpecProvenance {
            source: SOURCE.to_string(),
            author: Some(AUTHOR.to_string()),
            rationale: Some(RATIONALE.to_string()),
        },
    }
}
