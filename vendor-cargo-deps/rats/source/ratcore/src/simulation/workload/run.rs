/// Runs the bounded pilot with optional read-only observation.
///
/// # Errors
///
/// Returns only identity, controlled-input, lifecycle, projection, or oracle
/// validation failures. Observer validation never changes application behavior.
pub fn run_headless_pilot(
    controls: HeadlessPilotControls,
    observer: Option<&mut dyn super::observation::SimulationProbeObserver>,
) -> Result<HeadlessPilotReceipt, HeadlessPilotError> {
    let registry = headless_pilot_registry().map_err(HeadlessPilotError::IdentityRegistry)?;
    let control_issues =
        super::inputs::validate_control_history(&registry, &pilot_control_history(controls));
    if !control_issues.is_empty() {
        return Err(HeadlessPilotError::Controls(control_issues));
    }

    let mut context = PilotRunContext::new(observer);
    run_initial_controls(&registry, &mut context, controls);
    run_effect_path(&registry, &mut context, controls.effect_result)?;
    context.apply(&registry, HeadlessPilotMessage::Shutdown);
    finish_pilot(&registry, context)
}

fn run_initial_controls(
    registry: &SimulationIdentityRegistry,
    context: &mut PilotRunContext<'_>,
    controls: HeadlessPilotControls,
) {
    debug_assert!(controls.selected_choice < 2);
    debug_assert!(context.transition_kinds.is_empty());
    context.apply(
        registry,
        HeadlessPilotMessage::Resize {
            columns: PILOT_VIEWPORT_COLUMNS,
            rows: PILOT_VIEWPORT_ROWS,
        },
    );
    let messages = match controls.event_order {
        HeadlessPilotEventOrder::TickThenChoice => [
            HeadlessPilotMessage::LogicalTick(controls.logical_tick),
            HeadlessPilotMessage::SelectPane(controls.selected_choice),
        ],
        HeadlessPilotEventOrder::ChoiceThenTick => [
            HeadlessPilotMessage::SelectPane(controls.selected_choice),
            HeadlessPilotMessage::LogicalTick(controls.logical_tick),
        ],
    };
    for message in messages {
        context.apply(registry, message);
    }
    context.emitter.emit(
        registry,
        probe_record(
            "simulation.probe.choice-v1",
            SimulationProbeEvent::ChoiceSelected,
            PILOT_CHOICE_POINT,
            SimulationProbeOutcome::Selected,
        )
        .with_logical_tick(controls.logical_tick)
        .with_metadata(vec![
            SimulationProbeMetadata::new("choice.index", controls.selected_choice.to_string()),
            SimulationProbeMetadata::new("choice.point", PILOT_CHOICE_POINT),
        ]),
    );
}

fn run_effect_path(
    registry: &SimulationIdentityRegistry,
    context: &mut PilotRunContext<'_>,
    result: SimulationEffectResult,
) -> Result<(), HeadlessPilotError> {
    debug_assert!(context.state.effect_result.is_none());
    debug_assert!(!context.state.shutdown);
    let transition = context.apply(registry, HeadlessPilotMessage::RequestPersistence);
    let Some(request) = transition.effects.first().cloned() else {
        return Err(HeadlessPilotError::MissingDeclarativeEffect);
    };
    let mut tracker = SimulationEffectTracker::default();
    for lifecycle in [
        SimulationEffectLifecycle::Requested,
        SimulationEffectLifecycle::Started,
        SimulationEffectLifecycle::Result(result),
    ] {
        observe_effect(
            registry,
            &mut context.emitter,
            &mut tracker,
            &request,
            lifecycle,
        )?;
    }
    context.apply(registry, HeadlessPilotMessage::EffectResult(result));
    Ok(())
}

fn finish_pilot(
    registry: &SimulationIdentityRegistry,
    mut context: PilotRunContext<'_>,
) -> Result<HeadlessPilotReceipt, HeadlessPilotError> {
    debug_assert!(context.state.effect_result.is_some());
    debug_assert!(context.state.shutdown);
    let projection = project_pilot_state(&context.state)?;
    let projection_issues = super::snapshot::validate_state_projection(registry, &projection);
    if !projection_issues.is_empty() {
        return Err(HeadlessPilotError::StateProjection(projection_issues));
    }
    let state_identity = projection.blake3_identity();
    emit_state_probe(registry, &mut context, &state_identity);
    let (catalog, observations) = pilot_oracles(&context.state);
    let oracle_summary = super::judgment::summarize_oracles(registry, &catalog, &observations)
        .map_err(HeadlessPilotError::Oracles)?;
    emit_oracle_probes(registry, &mut context.emitter, &oracle_summary);

    context.behavior_bytes.extend_from_slice(b"state=");
    context
        .behavior_bytes
        .extend_from_slice(state_identity.as_bytes());
    context.behavior_bytes.push(b'\n');
    let behavior_identity = blake3::hash(&context.behavior_bytes).to_hex().to_string();
    Ok(HeadlessPilotReceipt {
        final_state: context.state,
        final_projection: projection,
        behavior_identity,
        transition_kinds: context.transition_kinds,
        diagnostics: context.diagnostics,
        oracle_summary,
        probe_diagnostics: context.emitter.diagnostics,
        dropped_probes: context.emitter.dropped,
    })
}

fn emit_state_probe(
    registry: &SimulationIdentityRegistry,
    context: &mut PilotRunContext<'_>,
    state_identity: &str,
) {
    context.emitter.emit(
        registry,
        probe_record(
            "simulation.probe.state-v1",
            SimulationProbeEvent::StateCheckpoint,
            PILOT_STATE_PROJECTION,
            SimulationProbeOutcome::Observed,
        )
        .with_logical_tick(context.state.logical_tick)
        .with_state_identity(state_identity)
        .with_metadata(vec![SimulationProbeMetadata::new(
            "state.projection",
            PILOT_STATE_PROJECTION,
        )]),
    );
}

fn emit_oracle_probes(
    registry: &SimulationIdentityRegistry,
    emitter: &mut PilotProbeEmitter<'_>,
    summaries: &[SimulationOracleSummary],
) {
    for summary in summaries {
        emitter.emit(
            registry,
            probe_record(
                "simulation.probe.oracle-v1",
                SimulationProbeEvent::OracleObserved,
                summary.invariant_id.as_str(),
                oracle_probe_outcome(summary.status),
            )
            .with_metadata(vec![
                SimulationProbeMetadata::new("oracle.category", summary.category.as_str()),
                SimulationProbeMetadata::new(
                    "oracle.condition",
                    oracle_status_label(summary.status),
                ),
            ]),
        );
    }
}

const fn oracle_probe_outcome(status: SimulationOracleStatus) -> SimulationProbeOutcome {
    match status {
        SimulationOracleStatus::AlwaysFailed
        | SimulationOracleStatus::SometimesUnobservedSuccess
        | SimulationOracleStatus::Unreached
        | SimulationOracleStatus::UnreachableViolated => SimulationProbeOutcome::FailedOracle,
        SimulationOracleStatus::CatalogedButUnobserved => SimulationProbeOutcome::Unobserved,
        _ => SimulationProbeOutcome::Passed,
    }
}

fn pilot_control_history(controls: HeadlessPilotControls) -> Vec<SimulationControlInput> {
    vec![
        SimulationControlInput::EventSelection {
            ready_sources: vec!["source.choice".to_string(), "source.tick".to_string()],
            selected_source: controls.event_order.selected_source().to_string(),
        },
        SimulationControlInput::LogicalTick {
            source_id: "source.tick".to_string(),
            tick: controls.logical_tick,
        },
        SimulationControlInput::FiniteChoice {
            choice_point: SimulationChoicePointId::new(PILOT_CHOICE_POINT),
            bound: 2,
            selected: controls.selected_choice,
        },
        SimulationControlInput::EffectResult {
            request_id: PILOT_REQUEST.to_string(),
            effect_class: SimulationEffectClassId::new(PILOT_EFFECT_CLASS),
            result: controls.effect_result,
        },
    ]
}
