/// Source of explicit replayable controls.
pub trait SimulationControlProvider {
    /// Returns the next explicit input or a classified missing-input diagnostic.
    ///
    /// # Errors
    ///
    /// Returns a deterministic diagnostic and never falls back to host state.
    fn next_input(&mut self) -> Result<SimulationControlInput, SimulationControlDiagnostic>;
}

/// Ordered finite scripted control provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptedSimulationControls {
    inputs: std::collections::VecDeque<SimulationControlInput>,
    remaining: u32,
}

impl ScriptedSimulationControls {
    /// Creates a provider from explicit bounded inputs.
    ///
    /// # Errors
    ///
    /// Returns `ScriptTooLarge` instead of allocating an unbounded script.
    pub fn new(
        inputs: impl IntoIterator<Item = SimulationControlInput>,
    ) -> Result<Self, SimulationControlDiagnostic> {
        let mut queue = std::collections::VecDeque::new();
        let mut remaining = 0_u32;
        for input in inputs {
            if remaining >= MAX_CONTROL_INPUTS_U32 {
                return Err(control_issue(
                    SimulationControlDiagnosticCode::ScriptTooLarge,
                    "script",
                ));
            }
            queue.push_back(input);
            remaining += 1;
        }
        Ok(Self {
            inputs: queue,
            remaining,
        })
    }

    /// Returns the number of unconsumed controls.
    #[must_use]
    pub const fn remaining(&self) -> u32 {
        self.remaining
    }
}

impl SimulationControlProvider for ScriptedSimulationControls {
    fn next_input(&mut self) -> Result<SimulationControlInput, SimulationControlDiagnostic> {
        let input = self.inputs.pop_front().ok_or_else(|| {
            control_issue(
                SimulationControlDiagnosticCode::MissingScriptedInput,
                "script",
            )
        })?;
        self.remaining = self.remaining.saturating_sub(1);
        Ok(input)
    }
}

/// No-input provider used to prove that deterministic execution fails closed.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSimulationControls;

impl SimulationControlProvider for NoopSimulationControls {
    fn next_input(&mut self) -> Result<SimulationControlInput, SimulationControlDiagnostic> {
        Err(control_issue(
            SimulationControlDiagnosticCode::MissingScriptedInput,
            "no-op-control",
        ))
    }
}

/// Recording wrapper that preserves exact consumed control order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingSimulationControls<P> {
    inner: P,
    consumed: Vec<SimulationControlInput>,
}

impl<P> RecordingSimulationControls<P> {
    /// Wraps one control provider.
    #[must_use]
    pub const fn new(inner: P) -> Self {
        Self {
            inner,
            consumed: Vec::new(),
        }
    }

    /// Returns consumed explicit inputs.
    #[must_use]
    pub fn consumed(&self) -> &[SimulationControlInput] {
        &self.consumed
    }
}

impl<P: SimulationControlProvider> SimulationControlProvider for RecordingSimulationControls<P> {
    fn next_input(&mut self) -> Result<SimulationControlInput, SimulationControlDiagnostic> {
        let input = self.inner.next_input()?;
        if self.consumed.len() >= MAX_CONTROL_INPUTS {
            return Err(control_issue(
                SimulationControlDiagnosticCode::ScriptTooLarge,
                "recorded-controls",
            ));
        }
        self.consumed.push(input.clone());
        Ok(input)
    }
}
