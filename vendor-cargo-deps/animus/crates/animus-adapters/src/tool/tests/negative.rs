mod spans;

use super::super::planning::*;
use super::super::*;
use super::support::*;

#[test]
fn request_rejects_missing_current_shape_fields() {
    // r[verify agent_harness_runtime.tool_calling_breaking.current_shape.rejects_legacy]
    let missing_name_arguments = prepared_arguments();
    let missing_name = Request::from_draft(&RequestDraft {
        name: None,
        arguments: Some(&missing_name_arguments),
        correlation_id: Some(CORRELATION_ID),
        output_policy: OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
        mutation_targets: &[],
        required_capability_labels: &[CAPABILITY_READ],
        approval_posture: APPROVAL_PRE_APPROVED,
    })
    .expect_err("missing tool name should fail");
    assert_eq!(missing_name, ContractError::MissingField {
        field: super::super::NAME_FIELD.as_str(),
    });

    let missing_arguments = Request::from_draft(&RequestDraft {
        name: Some(NAME),
        arguments: None,
        correlation_id: Some(CORRELATION_ID),
        output_policy: OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
        mutation_targets: &[],
        required_capability_labels: &[CAPABILITY_READ],
        approval_posture: APPROVAL_PRE_APPROVED,
    })
    .expect_err("missing arguments should fail");
    assert_eq!(missing_arguments, ContractError::MissingField {
        field: super::super::ARGUMENT_NAME_FIELD.as_str(),
    });
}

#[test]
fn request_rejects_missing_schema_identity_capabilities_and_bad_approval() {
    // r[verify agent_harness_runtime.tool_calling_breaking.current_shape.rejects_legacy]
    let fields = [ArgumentFieldDraft {
        name: Some(CURRENT_ARGUMENT),
        value: Some(ARGUMENT_VALUE),
    }];
    let missing_schema = PreparedArguments::prepare(&ArgumentPreparationRequest {
        current_schema_identity: BLANK_TEXT,
        allowed_field_names: &[CURRENT_ARGUMENT],
        compatibility_rules: &[],
        fields: &fields,
    })
    .expect_err("blank schema identity should fail");
    assert!(matches!(missing_schema, ContractError::Adapter(_)));

    let arguments = prepared_arguments();
    let missing_caps = Request::from_draft(&RequestDraft {
        name: Some(NAME),
        arguments: Some(&arguments),
        correlation_id: Some(CORRELATION_ID),
        output_policy: OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
        mutation_targets: &[],
        required_capability_labels: &[],
        approval_posture: APPROVAL_PRE_APPROVED,
    })
    .expect_err("missing capability labels should fail");
    assert_eq!(
        missing_caps,
        ContractError::Adapter(crate::AdapterContractError::EmptyCollection {
            field: super::super::CAPABILITY_FIELD.as_str(),
        })
    );

    let bad_approval = Request::from_draft(&RequestDraft {
        name: Some(NAME),
        arguments: Some(&arguments),
        correlation_id: Some(CORRELATION_ID),
        output_policy: OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
        mutation_targets: &[],
        required_capability_labels: &[CAPABILITY_READ],
        approval_posture: "ambient-approval",
    })
    .expect_err("unsupported approval should fail");
    assert_eq!(bad_approval, ContractError::UnsupportedApprovalPosture {
        posture: super::super::Text::from("ambient-approval"),
    });
}

#[test]
fn definition_metadata_rejects_missing_schema_and_empty_guidelines() {
    let missing_schema = Definition::validate(&DefinitionDraft {
        parameter_schema_identity: None,
        ..definition_draft(DefinitionStatus::Active, NAME)
    })
    .expect_err("missing schema identity should fail");
    assert_eq!(missing_schema, ContractError::MissingField {
        field: super::super::SCHEMA_FIELD.as_str(),
    });

    let empty_guidelines = Definition::validate(&DefinitionDraft {
        prompt_guidelines: &[],
        ..definition_draft(DefinitionStatus::Active, NAME)
    })
    .expect_err("empty guidelines should fail");
    assert_eq!(
        empty_guidelines,
        ContractError::Adapter(crate::AdapterContractError::EmptyCollection {
            field: super::super::PROMPT_GUIDELINE_FIELD.as_str(),
        })
    );
}

#[test]
fn malformed_unknown_arguments_fail() {
    // r[verify agent_harness_runtime.tool_calling_breaking.current_shape.rejects_legacy]
    let unknown_fields = [ArgumentFieldDraft {
        name: Some(EXTRA_ARGUMENT),
        value: Some(ARGUMENT_VALUE),
    }];
    let unknown = PreparedArguments::prepare(&ArgumentPreparationRequest {
        current_schema_identity: SCHEMA_ID,
        allowed_field_names: &[CURRENT_ARGUMENT],
        compatibility_rules: &[],
        fields: &unknown_fields,
    })
    .expect_err("unknown current field should fail");
    assert_eq!(unknown, ContractError::UnknownArgument {
        name: super::super::Text::from(EXTRA_ARGUMENT),
    });
}

#[test]
fn duplicate_current_arguments_fail() {
    let duplicate_fields = [
        ArgumentFieldDraft {
            name: Some(CURRENT_ARGUMENT),
            value: Some(ARGUMENT_VALUE),
        },
        ArgumentFieldDraft {
            name: Some(CURRENT_ARGUMENT),
            value: Some(ARGUMENT_VALUE),
        },
    ];
    assert!(duplicate_fields.iter().all(|field| field.name == Some(CURRENT_ARGUMENT)));
    let duplicate = PreparedArguments::prepare(&ArgumentPreparationRequest {
        current_schema_identity: SCHEMA_ID,
        allowed_field_names: &[CURRENT_ARGUMENT],
        compatibility_rules: &[],
        fields: &duplicate_fields,
    })
    .expect_err("duplicate current fields should fail");
    assert_eq!(duplicate, ContractError::DuplicateArgument {
        name: super::super::Text::from(CURRENT_ARGUMENT),
    });
}

#[test]
fn missing_argument_values_fail() {
    let missing_value_fields = [ArgumentFieldDraft {
        name: Some(CURRENT_ARGUMENT),
        value: None,
    }];
    let missing_value = PreparedArguments::prepare(&ArgumentPreparationRequest {
        current_schema_identity: SCHEMA_ID,
        allowed_field_names: &[CURRENT_ARGUMENT],
        compatibility_rules: &[],
        fields: &missing_value_fields,
    })
    .expect_err("missing argument value should fail");
    assert_eq!(missing_value, ContractError::MissingField {
        field: super::super::ARGUMENT_VALUE_FIELD.as_str(),
    });
}

#[test]
fn legacy_argument_compatibility_rules_fail() {
    let legacy_fields = [ArgumentFieldDraft {
        name: Some(LEGACY_ARGUMENT),
        value: Some(ARGUMENT_VALUE),
    }];
    let legacy_rules = [ArgumentCompatibilityRule {
        legacy_name: LEGACY_ARGUMENT,
        current_name: CURRENT_ARGUMENT,
    }];
    let legacy = PreparedArguments::prepare(&ArgumentPreparationRequest {
        current_schema_identity: SCHEMA_ID,
        allowed_field_names: &[CURRENT_ARGUMENT],
        compatibility_rules: &legacy_rules,
        fields: &legacy_fields,
    })
    .expect_err("legacy compatibility should be disabled");
    assert_eq!(legacy, ContractError::LegacyCompatibilityDisabled);
}

#[test]
fn unsafe_output_reference_and_raw_body_hazards_are_rejected() {
    // r[verify agent_harness_runtime.tool_calling_breaking.bounded_result.no_raw_body]
    let unsafe_reference = BoundedOutput::new(
        &OutputDraft {
            body: OUTPUT,
            full_output_reference: Some(UNSAFE_REFERENCE),
            hash_requirement: HashRequirement::Required,
        },
        OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
    )
    .expect_err("absolute references should fail");
    assert_eq!(unsafe_reference, ContractError::UnsafeReference {
        reference: super::super::Text::from(UNSAFE_REFERENCE),
    });

    let nul_output = BoundedOutput::new(
        &OutputDraft {
            body: "bad\0output",
            full_output_reference: None,
            hash_requirement: HashRequirement::Required,
        },
        OutputPolicy {
            visible_char_budget: VISIBLE_BUDGET,
        },
    )
    .expect_err("NUL output should fail");
    assert!(matches!(nul_output, ContractError::Adapter(_)));
}

#[test]
fn missing_executor_denied_capability_and_unsafe_output_fail_without_fallback() {
    let active_definition = definition(DefinitionStatus::Active, NAME);
    let missing_executor = evaluate_dispatch(DispatchRequest {
        definition: Some(&active_definition),
        executor_status: ExecutorStatus::Missing,
        capability_status: CapabilityStatus::Allowed,
        output_boundary_status: OutputBoundaryStatus::Bounded,
    });
    assert_eq!(missing_executor.status(), DispatchStatus::Failed);
    assert_eq!(missing_executor.failure_class(), Some(FailureClass::MissingExecutor));
    assert!(!missing_executor.fallback_attempted());

    let denied = evaluate_dispatch(DispatchRequest {
        definition: Some(&active_definition),
        executor_status: ExecutorStatus::Available,
        capability_status: CapabilityStatus::Denied,
        output_boundary_status: OutputBoundaryStatus::Bounded,
    });
    assert_eq!(denied.status(), DispatchStatus::Failed);
    assert_eq!(denied.failure_class(), Some(FailureClass::DeniedCapability));
    assert!(!denied.fallback_attempted());

    let unsafe_output = evaluate_dispatch(DispatchRequest {
        definition: Some(&active_definition),
        executor_status: ExecutorStatus::Available,
        capability_status: CapabilityStatus::Allowed,
        output_boundary_status: OutputBoundaryStatus::UnsafeReference,
    });
    assert_eq!(unsafe_output.status(), DispatchStatus::Failed);
    assert_eq!(unsafe_output.failure_class(), Some(FailureClass::UnsafeOutputReference));
    assert!(!unsafe_output.fallback_attempted());
}

#[test]
fn plan_fabric_rejects_arbitrary_host_actions() {
    // r[verify agent_harness_runtime.goose_code_mode_tool_planning.fabric.denied]
    let denied = admit_plan_fabric(&PlanFabricDraft {
        allowed_host_actions: &[FABRIC_ACTION_PROCESS],
        ..admitted_fabric()
    })
    .expect("safe labels should parse");

    assert_eq!(denied.status(), PlanFabricStatus::Denied);
    assert_eq!(denied.issue(), PlanFabricIssue::UnsupportedHostAction);
}

#[test]
fn plan_denies_undiscovered_tools_without_ambient_lookup() {
    // r[verify agent_harness_runtime.goose_code_mode_tool_planning.step_admission.denied]
    let definitions = [
        definition(DefinitionStatus::Active, NAME),
        definition(DefinitionStatus::Inactive, INACTIVE_NAME),
    ];
    let discovered = discovery();
    let hidden_request = request_for(RequestFixture {
        name: INACTIVE_NAME,
        approval_posture: APPROVAL_PRE_APPROVED,
        mutation_targets: &[TARGET_OTHER],
        required_capability_labels: &[CAPABILITY_READ],
    });
    let steps = [PlanStepDraft {
        step_id: Some(STEP_FIRST),
        request: &hidden_request,
        cancellation_posture: PlanCancellationPosture::Continue,
        timeout_posture: PlanTimeoutPosture::WithinDeadline,
        depends_on: &[],
        intermediate_bindings: &[],
    }];
    let plan = admit_plan(
        &PlanDraft {
            batch_id: Some(PLAN_BATCH),
            fabric: admitted_fabric(),
            batch_policy: PLAN_BATCH_POLICY_SERIAL,
            steps: &steps,
        },
        PlanAdmissionContext {
            discovery: &discovered,
            definitions: &definitions,
        },
    )
    .expect("denied plan should still be typed");

    assert_eq!(plan.status(), PlanStatus::Denied);
    assert_eq!(plan.issue(), PlanIssue::UndiscoveredTool);
    assert_eq!(plan.steps().len(), SINGLE_STEP_COUNT);
    assert_eq!(plan.steps()[FIRST_INDEX].issue(), PlanIssue::UndiscoveredTool);
}

#[test]
fn plan_denies_cancelled_and_timed_out_steps() {
    let definitions = [definition(DefinitionStatus::Active, NAME)];
    let discovered = discovery();
    let tool_request = request();
    let cancelled_steps = [PlanStepDraft {
        step_id: Some(STEP_FIRST),
        request: &tool_request,
        cancellation_posture: PlanCancellationPosture::Cancelled,
        timeout_posture: PlanTimeoutPosture::WithinDeadline,
        depends_on: &[],
        intermediate_bindings: &[],
    }];
    let cancelled = admit_plan(
        &PlanDraft {
            batch_id: Some(PLAN_BATCH),
            fabric: admitted_fabric(),
            batch_policy: PLAN_BATCH_POLICY_SERIAL,
            steps: &cancelled_steps,
        },
        PlanAdmissionContext {
            discovery: &discovered,
            definitions: &definitions,
        },
    )
    .expect("cancelled plan should remain typed");
    assert_eq!(cancelled.status(), PlanStatus::Denied);
    assert_eq!(cancelled.issue(), PlanIssue::Cancelled);

    let timed_out_steps = [PlanStepDraft {
        step_id: Some(STEP_FIRST),
        request: &tool_request,
        cancellation_posture: PlanCancellationPosture::Continue,
        timeout_posture: PlanTimeoutPosture::TimedOut,
        depends_on: &[],
        intermediate_bindings: &[],
    }];
    let timed_out = admit_plan(
        &PlanDraft {
            batch_id: Some(PLAN_BATCH),
            fabric: admitted_fabric(),
            batch_policy: PLAN_BATCH_POLICY_SERIAL,
            steps: &timed_out_steps,
        },
        PlanAdmissionContext {
            discovery: &discovered,
            definitions: &definitions,
        },
    )
    .expect("timed-out plan should remain typed");
    assert_eq!(timed_out.status(), PlanStatus::Denied);
    assert_eq!(timed_out.issue(), PlanIssue::TimedOut);
}

#[test]
fn plan_rejects_binary_intermediate_as_text() {
    // r[verify agent_harness_runtime.goose_code_mode_tool_planning.intermediates.binary]
    let definitions = [definition(DefinitionStatus::Active, NAME)];
    let discovered = discovery();
    let first_request = request();
    let second_request = request_for(RequestFixture {
        name: NAME,
        approval_posture: APPROVAL_PRE_APPROVED,
        mutation_targets: &[TARGET_OTHER],
        required_capability_labels: &[CAPABILITY_READ],
    });
    let binary_text_binding = PlanIntermediateBindingDraft {
        binding_id: Some(BINDING_FIRST),
        source_step_id: Some(STEP_FIRST),
        target_argument: Some(TARGET_ARGUMENT),
        value_kind: PlanIntermediateKind::Binary,
        use_kind: PlanIntermediateUse::ArgumentText,
        char_count: VISIBLE_BUDGET,
        content_hash: Some(HASH_C),
    };
    let steps = [
        PlanStepDraft {
            step_id: Some(STEP_FIRST),
            request: &first_request,
            cancellation_posture: PlanCancellationPosture::Continue,
            timeout_posture: PlanTimeoutPosture::WithinDeadline,
            depends_on: &[],
            intermediate_bindings: &[],
        },
        PlanStepDraft {
            step_id: Some(STEP_SECOND),
            request: &second_request,
            cancellation_posture: PlanCancellationPosture::Continue,
            timeout_posture: PlanTimeoutPosture::WithinDeadline,
            depends_on: &[STEP_FIRST],
            intermediate_bindings: &[binary_text_binding],
        },
    ];
    let plan = admit_plan(
        &PlanDraft {
            batch_id: Some(PLAN_BATCH),
            fabric: admitted_fabric(),
            batch_policy: PLAN_BATCH_POLICY_SERIAL,
            steps: &steps,
        },
        PlanAdmissionContext {
            discovery: &discovered,
            definitions: &definitions,
        },
    )
    .expect("unsafe intermediate should deny plan without raw payload");

    assert_eq!(plan.status(), PlanStatus::Denied);
    assert_eq!(plan.issue(), PlanIssue::UnsafeIntermediate);
    assert_eq!(plan.steps()[SECOND_INDEX].issue(), PlanIssue::UnsafeIntermediate);
}

#[test]
fn plan_receipt_rejects_raw_prompt_diagnostics() {
    // r[verify agent_harness_runtime.goose_code_mode_tool_planning.receipts.rejects_leaks]
    let definitions = [definition(DefinitionStatus::Active, NAME)];
    let discovered = discovery();
    let tool_request = request();
    let steps = [PlanStepDraft {
        step_id: Some(STEP_FIRST),
        request: &tool_request,
        cancellation_posture: PlanCancellationPosture::Continue,
        timeout_posture: PlanTimeoutPosture::WithinDeadline,
        depends_on: &[],
        intermediate_bindings: &[],
    }];
    let plan = admit_plan(
        &PlanDraft {
            batch_id: Some(PLAN_BATCH),
            fabric: admitted_fabric(),
            batch_policy: PLAN_BATCH_POLICY_SERIAL,
            steps: &steps,
        },
        PlanAdmissionContext {
            discovery: &discovered,
            definitions: &definitions,
        },
    )
    .expect("plan should validate before receipt leak check");
    assert_eq!(plan.status(), PlanStatus::Admitted);

    let leak = PlanReceipt::new(&PlanReceiptDraft {
        plan: &plan,
        policy_hash: HASH_A,
        diagnostics: &[DIAGNOSTIC_RAW_PROMPT],
    })
    .expect_err("raw prompt diagnostics must be rejected");

    assert!(matches!(leak, crate::safe::receipt::Error::UnsafeMetadata { .. }));
}
