/// Behavior and evidence receipt for one headless pilot run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessPilotReceipt {
    /// Final accepted state.
    pub final_state: HeadlessPilotState,
    /// Final canonical state projection.
    pub final_projection: SimulationStateProjection,
    /// BLAKE3 of ordered transition/effect/diagnostic evidence.
    pub behavior_identity: String,
    /// Ordered transition classes.
    pub transition_kinds: Vec<HeadlessPilotTransitionKind>,
    /// Ordered deterministic diagnostics.
    pub diagnostics: Vec<HeadlessPilotDiagnosticCode>,
    /// Oracle summary rows.
    pub oracle_summary: Vec<SimulationOracleSummary>,
    /// Probe validation/drop diagnostics, excluded from behavior identity.
    pub probe_diagnostics: Vec<SimulationProbeDiagnostic>,
    /// Number of bounded probe drops, excluded from behavior identity.
    pub dropped_probes: u32,
}

impl HeadlessPilotReceipt {
    /// Renders a deterministic human-reviewable receipt.
    #[must_use]
    pub fn render(&self) -> String {
        let mut output = format!(
            "schema=rats.deterministic-simulation-headless-receipt.v1\nevidence_class=provider-neutral-headless\nscenario={}\nadapter={}\nbehavior_blake3={}\nstate_blake3={}\nprobe_diagnostics={}\ndropped_probes={}\n",
            PILOT_SCENARIO,
            PILOT_ADAPTER,
            self.behavior_identity,
            self.final_projection.blake3_identity(),
            self.probe_diagnostics.len(),
            self.dropped_probes,
        );
        for summary in &self.oracle_summary {
            output.push_str("oracle=");
            output.push_str(summary.invariant_id.as_str());
            output.push(':');
            output.push_str(summary.category.as_str());
            output.push(':');
            output.push_str(oracle_status_label(summary.status));
            output.push('\n');
        }
        output.push_str("claims=local-controlled-hooks-only\n");
        output.push_str(
            "not_claimed=in-process-simulator,vm-replay,snapshot-reproduction,fault-coverage\n",
        );
        output
    }
}

/// Stable headless-pilot execution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadlessPilotError {
    /// Built-in identity registry failed validation.
    IdentityRegistry(Vec<SimulationIdentityDiagnostic>),
    /// Explicit controls failed validation.
    Controls(Vec<SimulationControlDiagnostic>),
    /// Pure pilot failed to emit its required declarative effect.
    MissingDeclarativeEffect,
    /// Effect lifecycle was inconsistent.
    EffectLifecycle(SimulationEffectDiagnostic),
    /// Canonical state projection was invalid.
    StateProjection(Vec<SimulationStateDiagnostic>),
    /// Oracle catalog or observations were invalid.
    Oracles(Vec<SimulationOracleDiagnostic>),
}

/// Returns the checked built-in registry used by the pilot and bridge.
///
/// # Errors
///
/// Returns identity-shape or projection-collision diagnostics.
pub fn headless_pilot_registry()
-> Result<SimulationIdentityRegistry, Vec<SimulationIdentityDiagnostic>> {
    SimulationIdentityRegistry::new(vec![
        SimulationIdentityEntry::new(SimulationIdentityKind::AdapterVersion, PILOT_ADAPTER),
        SimulationIdentityEntry::new(SimulationIdentityKind::ChoicePoint, PILOT_CHOICE_POINT),
        SimulationIdentityEntry::new(SimulationIdentityKind::EffectClass, PILOT_EFFECT_CLASS),
        SimulationIdentityEntry::new(
            SimulationIdentityKind::Invariant,
            "simulation.invariant.effect-reached-v1",
        ),
        SimulationIdentityEntry::new(
            SimulationIdentityKind::Invariant,
            "simulation.invariant.failure-observed-v1",
        ),
        SimulationIdentityEntry::new(
            SimulationIdentityKind::Invariant,
            "simulation.invariant.state-valid-v1",
        ),
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "simulation.probe.choice-v1"),
        SimulationIdentityEntry::new(
            SimulationIdentityKind::Probe,
            "simulation.probe.diagnostic-v1",
        ),
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "simulation.probe.effect-v1"),
        SimulationIdentityEntry::new(
            SimulationIdentityKind::Probe,
            "simulation.probe.invalidation-v1",
        ),
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "simulation.probe.message-v1"),
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "simulation.probe.oracle-v1"),
        SimulationIdentityEntry::new(SimulationIdentityKind::Probe, "simulation.probe.state-v1"),
        SimulationIdentityEntry::new(
            SimulationIdentityKind::Probe,
            "simulation.probe.transition-v1",
        ),
        SimulationIdentityEntry::new(SimulationIdentityKind::Scenario, PILOT_SCENARIO),
        SimulationIdentityEntry::new(
            SimulationIdentityKind::StateProjection,
            PILOT_STATE_PROJECTION,
        ),
    ])
}
