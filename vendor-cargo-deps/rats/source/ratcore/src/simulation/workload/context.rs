struct PilotRunContext<'a> {
    emitter: PilotProbeEmitter<'a>,
    state: HeadlessPilotState,
    transition_kinds: Vec<HeadlessPilotTransitionKind>,
    diagnostics: Vec<HeadlessPilotDiagnosticCode>,
    behavior_bytes: Vec<u8>,
}

impl<'a> PilotRunContext<'a> {
    const fn new(
        observer: Option<&'a mut dyn super::observation::SimulationProbeObserver>,
    ) -> Self {
        Self {
            emitter: PilotProbeEmitter::new(observer),
            state: HeadlessPilotState {
                logical_tick: 0,
                selected_choice: 0,
                viewport_columns: 0,
                viewport_rows: 0,
                effect_result: None,
                diagnostic_count: 0,
                shutdown: false,
            },
            transition_kinds: Vec::new(),
            diagnostics: Vec::new(),
            behavior_bytes: Vec::new(),
        }
    }

    fn apply(
        &mut self,
        registry: &SimulationIdentityRegistry,
        message: HeadlessPilotMessage,
    ) -> HeadlessPilotTransition {
        self.emit_message(registry, message);
        let transition = update_headless_pilot(&mut self.state, message);
        self.transition_kinds.push(transition.kind);
        self.diagnostics
            .extend(transition.diagnostics.iter().copied());
        self.behavior_bytes
            .extend_from_slice(message.as_str().as_bytes());
        self.behavior_bytes.push(b':');
        self.behavior_bytes
            .extend_from_slice(transition.kind.as_str().as_bytes());
        self.behavior_bytes.push(b'\n');
        self.emit_transition(registry, message, &transition);
        transition
    }

    fn emit_message(
        &mut self,
        registry: &SimulationIdentityRegistry,
        message: HeadlessPilotMessage,
    ) {
        self.emitter.emit(
            registry,
            probe_record(
                "simulation.probe.message-v1",
                SimulationProbeEvent::MessageDelivered,
                message.as_str(),
                SimulationProbeOutcome::Observed,
            )
            .with_metadata(vec![SimulationProbeMetadata::new(
                "message.class",
                message.as_str(),
            )]),
        );
    }

    fn emit_transition(
        &mut self,
        registry: &SimulationIdentityRegistry,
        message: HeadlessPilotMessage,
        transition: &HeadlessPilotTransition,
    ) {
        self.emitter.emit(
            registry,
            probe_record(
                "simulation.probe.transition-v1",
                SimulationProbeEvent::TransitionEvaluated,
                message.as_str(),
                transition.kind.probe_outcome(),
            )
            .with_metadata(vec![SimulationProbeMetadata::new(
                "transition.kind",
                transition.kind.as_str(),
            )]),
        );
        for diagnostic in &transition.diagnostics {
            self.emit_diagnostic(registry, *diagnostic);
        }
        self.emit_invalidation(registry, message, transition.invalidation);
    }

    fn emit_diagnostic(
        &mut self,
        registry: &SimulationIdentityRegistry,
        diagnostic: HeadlessPilotDiagnosticCode,
    ) {
        self.behavior_bytes.extend_from_slice(b"diagnostic:");
        self.behavior_bytes
            .extend_from_slice(diagnostic.as_str().as_bytes());
        self.behavior_bytes.push(b'\n');
        self.emitter.emit(
            registry,
            probe_record(
                "simulation.probe.diagnostic-v1",
                SimulationProbeEvent::DiagnosticProduced,
                diagnostic.as_str(),
                SimulationProbeOutcome::Observed,
            )
            .with_metadata(vec![SimulationProbeMetadata::new(
                "diagnostic.code",
                diagnostic.as_str(),
            )]),
        );
    }

    fn emit_invalidation(
        &mut self,
        registry: &SimulationIdentityRegistry,
        message: HeadlessPilotMessage,
        invalidation: HeadlessPilotInvalidation,
    ) {
        self.emitter.emit(
            registry,
            probe_record(
                "simulation.probe.invalidation-v1",
                SimulationProbeEvent::InvalidationClassified,
                message.as_str(),
                SimulationProbeOutcome::Observed,
            )
            .with_metadata(vec![SimulationProbeMetadata::new(
                "invalidation.class",
                invalidation.as_str(),
            )]),
        );
    }
}

fn observe_effect(
    registry: &SimulationIdentityRegistry,
    emitter: &mut PilotProbeEmitter<'_>,
    tracker: &mut SimulationEffectTracker,
    request: &HeadlessPilotEffectRequest,
    lifecycle: SimulationEffectLifecycle,
) -> Result<(), HeadlessPilotError> {
    debug_assert!(!request.request_id.is_empty());
    debug_assert!(!request.effect_class.is_empty());
    tracker
        .observe(&request.request_id, &request.effect_class, lifecycle)
        .map_err(HeadlessPilotError::EffectLifecycle)?;
    let outcome = match lifecycle {
        SimulationEffectLifecycle::Requested => SimulationProbeOutcome::Requested,
        SimulationEffectLifecycle::Started => SimulationProbeOutcome::Started,
        SimulationEffectLifecycle::Result(result) => effect_probe_outcome(result),
    };
    emitter.emit(
        registry,
        probe_record(
            "simulation.probe.effect-v1",
            SimulationProbeEvent::EffectLifecycle,
            &request.request_id,
            outcome,
        )
        .with_metadata(vec![
            SimulationProbeMetadata::new("effect.class", request.effect_class.as_str()),
            SimulationProbeMetadata::new("request.id", &request.request_id),
        ]),
    );
    Ok(())
}
