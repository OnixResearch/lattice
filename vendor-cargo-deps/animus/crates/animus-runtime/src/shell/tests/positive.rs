#[test]
fn steel_runtime_returns_immediate_final_response() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result =
        super::support::run_program_with_initial_turn_budget(super::support::RESPOND_PROGRAM, &mut model, &mut tool);

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::FinalResponse { response } if response.message() == super::support::MODEL_MESSAGE
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn steel_runtime_stops_without_response() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result =
        super::support::run_program_with_initial_turn_budget(super::support::STOP_PROGRAM, &mut model, &mut tool);

    assert_eq!(result.status(), &crate::RuntimeStatus::Stopped);
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn steel_runtime_dispatches_one_model_call_then_final_response() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_followup_turn_budget(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::FinalResponse { response } if response.message() == super::support::MODEL_MESSAGE
    ));
    assert_eq!(model.calls, super::support::EXPECTED_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::ModelResponded { response } if response.message() == super::support::MODEL_MESSAGE
    )));
}

#[test]
fn steel_policy_runtime_allows_matching_profile_before_dispatch() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_policy_program_with_followup_turn_budget(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(
        matches!(
            result.status(),
            crate::RuntimeStatus::FinalResponse { response }
                if response.message() == super::support::MODEL_MESSAGE
        ),
        "unexpected policy runtime status: {:?}",
        result.status()
    );
    assert_eq!(model.calls, super::support::EXPECTED_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn steel_runtime_dispatches_one_tool_call_then_final_response() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_followup_turn_budget(
        super::support::TOOL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::FinalResponse { response } if response.message() == super::support::TOOL_OUTPUT
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::EXPECTED_ADAPTER_CALLS);
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::ToolResponded { response } if response.output() == super::support::TOOL_OUTPUT
    )));
}
