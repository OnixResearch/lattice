use super::super::*;

fn valid_projection() -> SimulationStateProjection {
    SimulationStateProjection::build(
        "simulation.state.workspace-effect-v1",
        vec![
            SimulationStatePair::new("choice.class", "1"),
            SimulationStatePair::new("effect.status", "completed"),
            SimulationStatePair::new("logical.tick", "10"),
        ],
        SimulationProjectionOverflowPolicy::Reject,
    )
    .expect("projection is within bounds")
}

#[test]
fn equivalent_semantic_state_has_stable_bytes_and_hash() {
    let first = valid_projection();
    let second = valid_projection();

    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.blake3_identity(), second.blake3_identity());
    assert_eq!(first.coverage_pairs()[1], ("effect.status", "completed"));
    assert!(validate_state_projection(&super::registry(), &first).is_empty());
}

#[test]
fn version_order_bounds_and_truncation_are_explicit() {
    let registry = super::registry();
    let mut drift = valid_projection();
    drift.version = 2;
    drift.pairs.swap(0, 1);
    let issues = validate_state_projection(&registry, &drift);
    assert!(issues.iter().any(|issue| { issue.code == SimulationStateDiagnosticCode::UnknownProjectionVersion }));
    assert!(issues.iter().any(|issue| { issue.code == SimulationStateDiagnosticCode::ProjectionNotOrdered }));

    let too_many: Vec<_> = (0..33).map(|index| SimulationStatePair::new("choice.class", index.to_string())).collect();
    assert_eq!(
        SimulationStateProjection::build(
            "simulation.state.workspace-effect-v1",
            too_many.clone(),
            SimulationProjectionOverflowPolicy::Reject,
        )
        .expect_err("reject policy must fail")
        .code,
        SimulationStateDiagnosticCode::ProjectionTooLarge
    );
    let truncated = SimulationStateProjection::build(
        "simulation.state.workspace-effect-v1",
        too_many,
        SimulationProjectionOverflowPolicy::Truncate,
    )
    .expect("truncate policy is explicit");
    assert_eq!(truncated.pairs.len(), 32);
    assert!(truncated.is_truncated);
}

#[test]
fn renderer_text_path_and_secret_state_are_rejected() {
    let registry = super::registry();
    let unsafe_values = [
        "renderer-handle-42",
        "dom-node-7",
        "/home/operator/file",
        "credential=private",
        "terminal-output",
    ];
    assert_eq!(unsafe_values.len(), 5);
    assert!(unsafe_values.iter().all(|value| !value.is_empty()));
    for value in unsafe_values {
        let projection = SimulationStateProjection::build(
            "simulation.state.workspace-effect-v1",
            vec![SimulationStatePair::new("effect.status", value)],
            SimulationProjectionOverflowPolicy::Reject,
        )
        .expect("shape construction does not validate content");
        assert!(validate_state_projection(&registry, &projection).iter().any(|issue| {
            issue.code == SimulationStateDiagnosticCode::SensitiveOrRendererState
                || issue.code == SimulationStateDiagnosticCode::StateValueInvalid
        }));
    }
}

fn oracle_catalog() -> Vec<SimulationOracleCatalogEntry> {
    vec![
        SimulationOracleCatalogEntry {
            invariant_id: SimulationInvariantId::new("simulation.invariant.effect-reached-v1"),
            category: SimulationOracleCategory::Reachable,
            detail: "effect-reached".to_string(),
        },
        SimulationOracleCatalogEntry {
            invariant_id: SimulationInvariantId::new("simulation.invariant.failure-observed-v1"),
            category: SimulationOracleCategory::Sometimes,
            detail: "failure-observed".to_string(),
        },
        SimulationOracleCatalogEntry {
            invariant_id: SimulationInvariantId::new("simulation.invariant.state-valid-v1"),
            category: SimulationOracleCategory::Always,
            detail: "state-valid".to_string(),
        },
    ]
}

#[test]
fn oracle_summary_distinguishes_pass_fail_missing_and_reachability() {
    let observations = vec![
        SimulationOracleObservation {
            invariant_id: SimulationInvariantId::new("simulation.invariant.effect-reached-v1"),
            category: SimulationOracleCategory::Reachable,
            condition: false,
            guidance: None,
            detail: "effect-reached".to_string(),
        },
        SimulationOracleObservation {
            invariant_id: SimulationInvariantId::new("simulation.invariant.failure-observed-v1"),
            category: SimulationOracleCategory::Sometimes,
            condition: false,
            guidance: None,
            detail: "failure-observed".to_string(),
        },
        SimulationOracleObservation {
            invariant_id: SimulationInvariantId::new("simulation.invariant.state-valid-v1"),
            category: SimulationOracleCategory::Always,
            condition: false,
            guidance: Some(0.0),
            detail: "state-valid".to_string(),
        },
    ];
    let summaries = summarize_oracles(&super::registry(), &oracle_catalog(), &observations).expect("oracles are valid");
    let statuses: Vec<_> = summaries.iter().map(|row| row.status).collect();

    assert!(statuses.contains(&SimulationOracleStatus::Unreached));
    assert!(statuses.contains(&SimulationOracleStatus::SometimesUnobservedSuccess));
    assert!(statuses.contains(&SimulationOracleStatus::AlwaysFailed));
}

#[test]
fn cataloged_but_unobserved_is_not_reported_as_pass() {
    let summaries = summarize_oracles(&super::registry(), &oracle_catalog(), &[])
        .expect("empty observations are evidence gaps, not validator errors");
    let unobserved = unobserved_oracle_ids(&summaries);

    assert_eq!(unobserved.len(), 3);
    assert!(summaries.iter().all(|summary| {
        summary.status == SimulationOracleStatus::CatalogedButUnobserved
            || summary.status == SimulationOracleStatus::UnreachablePreserved
    }));
}

#[test]
fn invalid_guidance_duplicate_and_unknown_oracles_fail_closed() {
    let mut catalog = oracle_catalog();
    catalog.push(catalog[0].clone());
    let observations = vec![
        SimulationOracleObservation {
            invariant_id: SimulationInvariantId::new("simulation.invariant.state-valid-v1"),
            category: SimulationOracleCategory::Sometimes,
            condition: true,
            guidance: Some(f64::NAN),
            detail: "state-valid".to_string(),
        },
        SimulationOracleObservation {
            invariant_id: SimulationInvariantId::new("invariant.unknown"),
            category: SimulationOracleCategory::Always,
            condition: true,
            guidance: Some(f64::INFINITY),
            detail: "unknown".to_string(),
        },
    ];
    let issues =
        summarize_oracles(&super::registry(), &catalog, &observations).expect_err("invalid oracle evidence must fail");
    let codes: Vec<_> = issues.iter().map(|issue| issue.code).collect();

    assert!(codes.contains(&SimulationOracleDiagnosticCode::DuplicateInvariant));
    assert!(codes.contains(&SimulationOracleDiagnosticCode::CategoryMismatch));
    assert!(codes.contains(&SimulationOracleDiagnosticCode::InvalidGuidance));
    assert!(codes.contains(&SimulationOracleDiagnosticCode::UnknownInvariant));
}
