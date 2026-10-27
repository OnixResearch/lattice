#[test]
fn invalid_steel_decision_fails_before_adapter_dispatch() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_initial_turn_budget(
        super::support::INVALID_DECISION_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::Fabric { .. }
        }
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn invalid_steel_load_fails_before_adapter_dispatch() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_initial_turn_budget(
        super::support::INVALID_LOAD_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::Fabric { .. }
        }
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn steel_policy_denial_fails_before_adapter_dispatch() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_policy_program_with_bad_profile(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::SteelPolicy { message }
        } if message == "steel policy denied: profile_mismatch"
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::SteelPolicyDenied { reason, .. } if reason == "profile_mismatch"
    )));
    assert!(
        result
            .events()
            .iter()
            .all(|event| !matches!(event, crate::RuntimeEvent::ModelRequested { .. }))
    );
}

#[test]
fn model_adapter_failure_stops_safely() {
    let mut model = super::support::ModelStub::failing();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_followup_turn_budget(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::ModelAdapter { message }
        } if message == super::support::ADAPTER_FAILURE
    ));
    assert_eq!(model.calls, super::support::EXPECTED_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn tool_adapter_failure_stops_safely() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::failing();
    let result = super::support::run_program_with_followup_turn_budget(
        super::support::TOOL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::ToolAdapter { message }
        } if message == super::support::ADAPTER_FAILURE
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::EXPECTED_ADAPTER_CALLS);
}

#[test]
fn missing_model_service_fails_closed_before_tool_dispatch() {
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_missing_model_service(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::ModelAdapter { message }
        } if message == super::support::ABSENT_SERVICE
    ));
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(
        result
            .events()
            .iter()
            .all(|event| !matches!(event, crate::RuntimeEvent::ModelResponded { .. }))
    );
}

#[test]
fn missing_tool_service_fails_closed_before_model_dispatch() {
    let mut model = super::support::ModelStub::ok();
    let result = super::support::run_program_with_missing_tool_service(
        super::support::TOOL_THEN_RESPOND_PROGRAM,
        &mut model,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::ToolAdapter { message }
        } if message == super::support::ABSENT_SERVICE
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert!(
        result
            .events()
            .iter()
            .all(|event| !matches!(event, crate::RuntimeEvent::ToolResponded { .. }))
    );
}

#[test]
fn unrelated_model_capability_fails_before_adapter_call() {
    // r[verify agent_harness_runtime.scoped_capabilities.exact_resolution.fail_closed]
    // r[verify agent_harness_runtime.model_dispatch.runtime_gate.fail_closed]
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_unrelated_model_capability(
        super::support::MODEL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::ModelAdapter { message }
        } if message == super::support::ABSENT_SERVICE
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::AdapterOutcomeClassified {
            component: crate::AdapterComponent::Model,
            class: animus_adapters::AdapterCallFailureClass::Absent,
            ..
        }
    )));
}

#[test]
fn unrelated_tool_capability_fails_before_adapter_call() {
    // r[verify agent_harness_runtime.scoped_capabilities.exact_resolution.fail_closed]
    // r[verify agent_harness_runtime.tool_calling_breaking.catalog_admission.exact_capability]
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_unrelated_tool_capability(
        super::support::TOOL_THEN_RESPOND_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert!(matches!(
        result.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::ToolAdapter { message }
        } if message == "denied_capability"
    ));
    assert_eq!(model.calls, super::support::NO_ADAPTER_CALLS);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
    assert!(result.events().iter().any(|event| matches!(
        event,
        crate::RuntimeEvent::AdapterOutcomeClassified {
            component: crate::AdapterComponent::Tool,
            class: animus_adapters::AdapterCallFailureClass::DeniedCapability,
            ..
        }
    )));
}

#[test]
fn blocked_adapter_outcome_stops_without_success_receipt() {
    let state = crate::TurnState::new(&super::support::runtime_input());
    let step = crate::apply_adapter_outcome(
        state,
        crate::AdapterOutcome::Blocked {
            component: crate::AdapterComponent::Tool,
            outcome: animus_adapters::OutcomeKind::Denied,
        },
    );

    let crate::AdapterStep::Complete { report } = step else {
        panic!("blocked adapter outcome should complete the run");
    };
    assert!(matches!(
        report.status(),
        crate::RuntimeStatus::Failed {
            failure: crate::RuntimeFailure::ToolAdapter { message }
        } if message == super::support::DENIED_SERVICE
    ));
    assert!(
        report
            .events()
            .iter()
            .all(|event| !matches!(event, crate::RuntimeEvent::ToolResponded { .. }))
    );
}

#[test]
fn typed_adapter_outcome_classes_are_preserved_in_failure_events() {
    // r[verify agent_harness_runtime.adapter_outcomes.runtime_mapping.fail_closed]
    let classes = [
        animus_adapters::AdapterCallFailureClass::Denied,
        animus_adapters::AdapterCallFailureClass::Unavailable,
        animus_adapters::AdapterCallFailureClass::MalformedOutput,
        animus_adapters::AdapterCallFailureClass::Cancelled,
        animus_adapters::AdapterCallFailureClass::TimedOut,
        animus_adapters::AdapterCallFailureClass::ExecutionFailure,
    ];

    for class in classes {
        let outcome =
            animus_adapters::AdapterCallOutcome::<animus_adapters::ModelResponse>::failed(
                class,
                animus_adapters::AdapterCallFailureMetadataDraft {
                    message: class.as_str(),
                    correlation_id: None,
                    receipt_reference: None,
                },
            )
            .expect("safe typed outcome should build");
        let step = crate::apply_adapter_outcome(
            crate::TurnState::new(&super::support::runtime_input()),
            crate::AdapterOutcome::ModelCall { outcome },
        );
        let crate::AdapterStep::Complete { report } = step else {
            panic!("typed failure should complete");
        };
        assert!(matches!(
            report.status(),
            crate::RuntimeStatus::Failed {
                failure: crate::RuntimeFailure::ModelAdapter { .. }
            }
        ));
        assert!(report.events().iter().any(|event| matches!(
            event,
            crate::RuntimeEvent::AdapterOutcomeClassified {
                component: crate::AdapterComponent::Model,
                class: event_class,
                ..
            } if *event_class == class
        )));
        assert!(
            report
                .events()
                .iter()
                .all(|event| { !matches!(event, crate::RuntimeEvent::ModelResponded { .. }) })
        );
    }
}

#[test]
fn max_turn_exhaustion_fails_closed_without_extra_adapter_call() {
    let mut model = super::support::ModelStub::ok();
    let mut tool = super::support::ToolStub::ok();
    let result = super::support::run_program_with_followup_turn_budget(
        super::support::SPINNING_MODEL_PROGRAM,
        &mut model,
        &mut tool,
    );

    assert_eq!(
        result.status(),
        &crate::RuntimeStatus::MaxTurnsExceeded {
            max_turns: super::support::FOLLOWUP_TURN_BUDGET,
        }
    );
    assert_eq!(model.calls, super::support::FOLLOWUP_TURN_BUDGET);
    assert_eq!(tool.calls, super::support::NO_ADAPTER_CALLS);
}

#[test]
fn runtime_input_rejects_empty_summary() {
    let error = crate::RuntimeInput::new(super::support::BLANK_TEXT)
        .expect_err("blank summary should fail");

    assert!(matches!(error, crate::RuntimeError::EmptyInput { .. }));
}

#[test]
fn runtime_config_rejects_zero_turn_limit() {
    let error = crate::RuntimeConfig::from_max_turn_count(super::support::INVALID_CONFIG_TURNS)
        .expect_err("zero max turns should fail");

    assert!(matches!(error, crate::RuntimeError::InvalidConfig { .. }));
}
