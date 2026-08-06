fn project_pilot_state(
    state: &HeadlessPilotState,
) -> Result<SimulationStateProjection, HeadlessPilotError> {
    debug_assert!(state.selected_choice < 2);
    debug_assert!(state.shutdown);
    let effect_status = state
        .effect_result
        .map_or("pending", SimulationEffectResult::as_str);
    SimulationStateProjection::build(
        PILOT_STATE_PROJECTION,
        vec![
            SimulationStatePair::new("choice.class", state.selected_choice.to_string()),
            SimulationStatePair::new("diagnostic.class", state.diagnostic_count.to_string()),
            SimulationStatePair::new("effect.class", PILOT_EFFECT_CLASS),
            SimulationStatePair::new("effect.status", effect_status),
            SimulationStatePair::new("lifecycle.class", "headless"),
            SimulationStatePair::new("logical.tick", state.logical_tick.to_string()),
            SimulationStatePair::new("selection.class", "explicit"),
            SimulationStatePair::new(
                "shutdown.class",
                if state.shutdown { "complete" } else { "active" },
            ),
            SimulationStatePair::new("viewport.columns", state.viewport_columns.to_string()),
            SimulationStatePair::new("viewport.rows", state.viewport_rows.to_string()),
        ],
        SimulationProjectionOverflowPolicy::Reject,
    )
    .map_err(|issue| HeadlessPilotError::StateProjection(vec![issue]))
}

fn pilot_oracles(
    state: &HeadlessPilotState,
) -> (
    Vec<SimulationOracleCatalogEntry>,
    Vec<SimulationOracleObservation>,
) {
    debug_assert!(state.selected_choice < 2);
    debug_assert!(state.effect_result.is_some());
    let catalog = vec![
        SimulationOracleCatalogEntry {
            invariant_id: SimulationInvariantId::new("simulation.invariant.effect-reached-v1"),
            category: SimulationOracleCategory::Reachable,
            detail: "effect-terminal-boundary".to_string(),
        },
        SimulationOracleCatalogEntry {
            invariant_id: SimulationInvariantId::new("simulation.invariant.failure-observed-v1"),
            category: SimulationOracleCategory::Sometimes,
            detail: "effect-failure-class".to_string(),
        },
        SimulationOracleCatalogEntry {
            invariant_id: SimulationInvariantId::new("simulation.invariant.state-valid-v1"),
            category: SimulationOracleCategory::Always,
            detail: "pilot-state-valid".to_string(),
        },
    ];
    let observations = vec![
        SimulationOracleObservation {
            invariant_id: SimulationInvariantId::new("simulation.invariant.effect-reached-v1"),
            category: SimulationOracleCategory::Reachable,
            condition: state.effect_result.is_some(),
            guidance: None,
            detail: "effect-terminal-boundary".to_string(),
        },
        SimulationOracleObservation {
            invariant_id: SimulationInvariantId::new("simulation.invariant.failure-observed-v1"),
            category: SimulationOracleCategory::Sometimes,
            condition: state.effect_result != Some(SimulationEffectResult::Completed),
            guidance: None,
            detail: "effect-failure-class".to_string(),
        },
        SimulationOracleObservation {
            invariant_id: SimulationInvariantId::new("simulation.invariant.state-valid-v1"),
            category: SimulationOracleCategory::Always,
            condition: state.selected_choice < 2 && state.shutdown,
            guidance: Some(match state.selected_choice {
                0 => 2.0,
                1 => 1.0,
                _ => 0.0,
            }),
            detail: "pilot-state-valid".to_string(),
        },
    ];
    (catalog, observations)
}

fn accepted_transition(invalidation: HeadlessPilotInvalidation) -> HeadlessPilotTransition {
    HeadlessPilotTransition {
        kind: HeadlessPilotTransitionKind::Accepted,
        effects: Vec::new(),
        diagnostics: Vec::new(),
        invalidation,
    }
}

fn rejected_transition(
    state: &mut HeadlessPilotState,
    diagnostic: HeadlessPilotDiagnosticCode,
) -> HeadlessPilotTransition {
    state.diagnostic_count = state.diagnostic_count.saturating_add(1);
    HeadlessPilotTransition {
        kind: HeadlessPilotTransitionKind::Rejected,
        effects: Vec::new(),
        diagnostics: vec![diagnostic],
        invalidation: HeadlessPilotInvalidation::None,
    }
}

fn ignored_transition() -> HeadlessPilotTransition {
    HeadlessPilotTransition {
        kind: HeadlessPilotTransitionKind::Ignored,
        effects: Vec::new(),
        diagnostics: Vec::new(),
        invalidation: HeadlessPilotInvalidation::None,
    }
}

fn probe_record(
    probe_id: &str,
    event: SimulationProbeEvent,
    subject_id: &str,
    outcome: SimulationProbeOutcome,
) -> SimulationProbeRecord {
    SimulationProbeRecord::new(
        probe_id,
        PILOT_SCENARIO,
        0,
        SimulationProbeSemantic::new(event, subject_id, outcome),
    )
}

const fn effect_probe_outcome(result: SimulationEffectResult) -> SimulationProbeOutcome {
    match result {
        SimulationEffectResult::Completed => SimulationProbeOutcome::Completed,
        SimulationEffectResult::Failed | SimulationEffectResult::ExternallyControlledFailure => {
            SimulationProbeOutcome::Failed
        }
        SimulationEffectResult::Cancelled => SimulationProbeOutcome::Cancelled,
        SimulationEffectResult::Replaced => SimulationProbeOutcome::Replaced,
        SimulationEffectResult::TimedOut => SimulationProbeOutcome::TimedOut,
        SimulationEffectResult::SourceClosed => SimulationProbeOutcome::SourceClosed,
    }
}

const fn oracle_status_label(status: SimulationOracleStatus) -> &'static str {
    match status {
        SimulationOracleStatus::AlwaysPassed => "always-passed",
        SimulationOracleStatus::AlwaysFailed => "always-failed",
        SimulationOracleStatus::SometimesObserved => "sometimes-observed",
        SimulationOracleStatus::SometimesUnobservedSuccess => "sometimes-unobserved-success",
        SimulationOracleStatus::Reached => "reached",
        SimulationOracleStatus::Unreached => "unreached",
        SimulationOracleStatus::UnreachablePreserved => "unreachable-preserved",
        SimulationOracleStatus::UnreachableViolated => "unreachable-violated",
        SimulationOracleStatus::CatalogedButUnobserved => "cataloged-but-unobserved",
    }
}
