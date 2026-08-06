const RUN_ID: &str = "run.runtime-control";
const EXPECTED_MODEL_CHECKPOINT_COUNT: usize = 3;

struct ScriptedWaiter {
    outcomes:
        std::collections::VecDeque<Result<animus_adapters::AgentControlPermit, animus_adapters::AgentControlWaitError>>,
    checkpoints: Vec<animus_adapters::AgentControlCheckpointKind>,
}

impl ScriptedWaiter {
    fn new(
        outcomes: impl IntoIterator<
            Item = Result<animus_adapters::AgentControlPermit, animus_adapters::AgentControlWaitError>,
        >,
    ) -> Self {
        Self {
            outcomes: outcomes.into_iter().collect(),
            checkpoints: Vec::new(),
        }
    }
}

impl animus_adapters::AgentControlWait for ScriptedWaiter {
    fn wait_until_runnable(
        &mut self,
        checkpoint: &animus_adapters::AgentControlCheckpoint,
    ) -> Result<animus_adapters::AgentControlPermit, animus_adapters::AgentControlWaitError> {
        self.checkpoints.push(checkpoint.kind());
        self.outcomes.pop_front().unwrap_or(Ok(animus_adapters::AgentControlPermit::Continue))
    }
}

#[test]
fn injected_control_gates_fabric_and_model_dispatch() {
    // r[verify agent_harness_runtime.agent_control.runtime.gate]
    // r[verify agent_harness_runtime.agent_control.verification.positive]
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let mut waiter = ScriptedWaiter::new([
        Ok(animus_adapters::AgentControlPermit::Continue),
        Ok(animus_adapters::AgentControlPermit::Continue),
        Ok(animus_adapters::AgentControlPermit::Continue),
    ]);
    let adapters = super::super::RuntimeAdapters::new(&mut model, &mut tool).with_agent_control(run_id(), &mut waiter);
    let run_result = super::super::run_with_steel_program(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &super::support::runtime_input(),
        super::support::runtime_config_with_followup_turn_budget(),
        adapters,
    );

    assert!(matches!(run_result.status(), crate::RuntimeStatus::FinalResponse { .. }));
    assert_eq!(model.calls, super::support::EXPECTED_ADAPTER_CALLS);
    assert_eq!(waiter.checkpoints.len(), EXPECTED_MODEL_CHECKPOINT_COUNT);
    assert_eq!(waiter.checkpoints.as_slice(), &[
        animus_adapters::AgentControlCheckpointKind::BeforeFabric,
        animus_adapters::AgentControlCheckpointKind::BeforeModel,
        animus_adapters::AgentControlCheckpointKind::BeforeFabric,
    ]);
}

#[test]
fn cancellation_stops_before_fabric_or_adapter_dispatch() {
    // r[verify agent_harness_runtime.agent_control.concurrency.cancel]
    // r[verify agent_harness_runtime.agent_control.verification.negative]
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let mut waiter = ScriptedWaiter::new([Ok(animus_adapters::AgentControlPermit::Cancel)]);
    let adapters = super::super::RuntimeAdapters::new(&mut model, &mut tool).with_agent_control(run_id(), &mut waiter);
    let run_result = super::super::run_with_steel_program(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &super::support::runtime_input(),
        super::support::runtime_config_with_followup_turn_budget(),
        adapters,
    );

    assert_eq!(run_result.status(), &crate::RuntimeStatus::Stopped);
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(run_result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::Cancelled { reason } if reason == "agent_control_cancelled"
    )));
}

#[test]
fn control_failure_stops_without_implicit_running_fallback() {
    // r[verify agent_harness_runtime.agent_control.concurrency.close]
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let mut waiter = ScriptedWaiter::new([Err(animus_adapters::AgentControlWaitError::Closed)]);
    let adapters = super::super::RuntimeAdapters::new(&mut model, &mut tool).with_agent_control(run_id(), &mut waiter);
    let run_result = super::super::run_with_steel_program(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &super::support::runtime_input(),
        super::support::runtime_config_with_followup_turn_budget(),
        adapters,
    );

    assert_eq!(run_result.status(), &crate::RuntimeStatus::Stopped);
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert!(run_result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::Cancelled { reason } if reason == "agent_control_wait_failed"
    )));
}

fn run_id() -> animus_adapters::AgentControlRunId {
    animus_adapters::AgentControlRunId::new(RUN_ID).expect("run id must validate")
}
