use super::support::*;
use super::*;

#[test]
fn manifest_rejects_unsafe_parameter_shapes() {
    let file_default = RecipeParameterDraft {
        name: FILE_PARAM,
        kind: "file",
        required: true,
        default_value: Some(FILE_REF),
        select_options: &[],
    };
    let optional_without_default = RecipeParameterDraft {
        name: MODE_PARAM,
        kind: "text",
        required: false,
        default_value: None,
        select_options: &[],
    };
    let bad_select_default = RecipeParameterDraft {
        name: MODE_PARAM,
        kind: "select",
        required: false,
        default_value: Some("unsafe"),
        select_options: &[MODE_REVIEW, MODE_FIX],
    };

    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            parameters: &[file_default],
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::FileParameterDefault { .. })
    ));
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            parameters: &[optional_without_default],
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::MissingOptionalDefault { .. })
    ));
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            parameters: &[bad_select_default],
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::InvalidSelectValue { .. })
    ));
}

#[test]
fn binding_rejects_undeclared_and_unused_template_values() {
    let workflow = minimal_workflow();
    let inputs = valid_inputs();

    assert!(matches!(
        bind_recipe_parameters(RecipeBindingDraft {
            workflow: &workflow,
            template: "Use {{topic}} {{mode}} {{context_file}} {{unsafe}}.",
            inputs: &inputs,
        }),
        Err(RecipeWorkflowError::UndeclaredTemplateVariable { .. })
    ));
    assert!(matches!(
        bind_recipe_parameters(RecipeBindingDraft {
            workflow: &workflow,
            template: "Use {{topic}} {{context_file}}.",
            inputs: &inputs,
        }),
        Err(RecipeWorkflowError::UnusedParameter { .. })
    ));
}

#[test]
fn binding_rejects_missing_required_file_input() {
    let workflow = minimal_workflow();
    let missing_file_inputs = [
        RecipeParameterInputDraft {
            name: TOPIC_PARAM,
            value: TOPIC_VALUE,
            source: "literal",
        },
        RecipeParameterInputDraft {
            name: MODE_PARAM,
            value: MODE_REVIEW,
            source: "literal",
        },
    ];
    assert!(missing_file_inputs.len() < PARAMETER_COUNT);
    assert!(
        missing_file_inputs
            .iter()
            .all(|input| input.name != FILE_PARAM)
    );

    assert!(matches!(
        bind_recipe_parameters(RecipeBindingDraft {
            workflow: &workflow,
            template: workflow.initial_prompt(),
            inputs: &missing_file_inputs,
        }),
        Err(RecipeWorkflowError::MissingParameterValue { .. })
    ));
}

#[test]
fn binding_rejects_invalid_select_value() {
    let workflow = minimal_workflow();
    let bad_select_inputs = [
        RecipeParameterInputDraft {
            name: TOPIC_PARAM,
            value: TOPIC_VALUE,
            source: "literal",
        },
        RecipeParameterInputDraft {
            name: MODE_PARAM,
            value: "unsafe",
            source: "literal",
        },
        RecipeParameterInputDraft {
            name: FILE_PARAM,
            value: FILE_REF,
            source: "file_ref",
        },
    ];
    assert_eq!(bad_select_inputs.len(), PARAMETER_COUNT);
    assert!(
        bad_select_inputs
            .iter()
            .any(|input| input.value == "unsafe")
    );

    assert!(matches!(
        bind_recipe_parameters(RecipeBindingDraft {
            workflow: &workflow,
            template: workflow.initial_prompt(),
            inputs: &bad_select_inputs,
        }),
        Err(RecipeWorkflowError::InvalidSelectValue { .. })
    ));
}

#[test]
fn binding_rejects_invalid_file_source() {
    let workflow = minimal_workflow();
    let bad_file_source = [
        RecipeParameterInputDraft {
            name: TOPIC_PARAM,
            value: TOPIC_VALUE,
            source: "literal",
        },
        RecipeParameterInputDraft {
            name: MODE_PARAM,
            value: MODE_REVIEW,
            source: "literal",
        },
        RecipeParameterInputDraft {
            name: FILE_PARAM,
            value: FILE_REF,
            source: "literal",
        },
    ];
    assert_eq!(bad_file_source.len(), PARAMETER_COUNT);
    assert!(bad_file_source.iter().any(|input| input.name == FILE_PARAM));

    assert!(matches!(
        bind_recipe_parameters(RecipeBindingDraft {
            workflow: &workflow,
            template: workflow.initial_prompt(),
            inputs: &bad_file_source,
        }),
        Err(RecipeWorkflowError::InvalidParameterSource { .. })
    ));
}

#[test]
fn malformed_output_blocks_automation_success() {
    let workflow = minimal_workflow();
    let missing = RecipeOutputDraft {
        fields: &[RecipeOutputFieldDraft {
            name: OUTPUT_STATUS,
            kind: "string",
        }],
    };
    let wrong_kind = RecipeOutputDraft {
        fields: &[
            RecipeOutputFieldDraft {
                name: OUTPUT_STATUS,
                kind: "string",
            },
            RecipeOutputFieldDraft {
                name: OUTPUT_WARNINGS,
                kind: "string",
            },
        ],
    };

    let missing_outcome = validate_recipe_output(workflow.response_schema(), &missing)
        .expect("missing field should be recorded as failure");
    let wrong_kind_outcome = validate_recipe_output(workflow.response_schema(), &wrong_kind)
        .expect("wrong kind should be recorded as failure");

    assert_eq!(
        missing_outcome.status(),
        RecipeAutomationStatus::StructuredOutputFailure
    );
    assert!(matches!(
        missing_outcome.failure_reason(),
        Some(RecipeOutputFailure::MissingField { .. })
    ));
    assert_eq!(
        wrong_kind_outcome.status(),
        RecipeAutomationStatus::StructuredOutputFailure
    );
    assert!(matches!(
        wrong_kind_outcome.failure_reason(),
        Some(RecipeOutputFailure::KindMismatch { .. })
    ));
}

#[test]
fn retry_policy_rejects_shell_bypass_unbounded_attempts_and_missing_timeout() {
    let shell_check = retry_check_draft(RetryCheckFixture {
        effect_kind: "shell",
        deadline_label: TIMEOUT_STANDARD,
    });
    let missing_deadline_label = retry_check_draft(RetryCheckFixture {
        effect_kind: "validation_check",
        deadline_label: "",
    });

    assert!(matches!(
        RecipeRetryPolicy::validate(&RecipeRetryPolicyDraft {
            max_attempts: RETRY_ATTEMPTS,
            checks: &[shell_check],
        }),
        Err(RecipeWorkflowError::UnsafeRetryEffect { .. })
    ));
    assert_eq!(
        RecipeRetryPolicy::validate(&RecipeRetryPolicyDraft {
            max_attempts: ZERO_RETRY_ATTEMPTS,
            checks: &[retry_check_draft(RetryCheckFixture {
                effect_kind: "validation_check",
                deadline_label: TIMEOUT_STANDARD,
            })],
        }),
        Err(RecipeWorkflowError::ZeroRetryAttempts)
    );
    assert!(matches!(
        RecipeRetryPolicy::validate(&RecipeRetryPolicyDraft {
            max_attempts: RETRY_ATTEMPTS,
            checks: &[missing_deadline_label],
        }),
        Err(RecipeWorkflowError::Contract(
            crate::AdapterContractError::EmptyField { .. }
        ))
    ));
}

#[test]
fn untrusted_or_invalid_provenance_has_no_runtime_influence() {
    let pending =
        RecipeTrustReceipt::new(&pending_provenance()).expect("pending receipt should parse");
    let influence = RecipeInfluence::evaluate(&pending);

    assert_eq!(pending.status(), RecipeTrustStatus::Pending);
    assert_eq!(influence.status(), RecipeInfluenceStatus::NoInfluence);
    assert!(!influence.prompt_allowed());
    assert!(!influence.tools_allowed());
    assert!(!influence.model_allowed());
    assert!(!influence.memory_allowed());
    assert!(!influence.workflow_permissions_allowed());
    assert!(matches!(
        RecipeTrustReceipt::new(&RecipeProvenanceDraft {
            content_hash: "not-a-hash",
            ..admitted_provenance()
        }),
        Err(RecipeWorkflowError::InvalidHash { .. })
    ));
}

#[test]
fn recipe_dag_rejects_duplicate_steps_before_planning_effects() {
    let duplicate_steps = [
        RecipeDagStepDraft {
            step_id: DAG_STEP_RESEARCH_A,
            kind: "prompt",
            depends_on: &[],
            input_refs: &[TOPIC_PARAM],
            output_refs: &[DAG_OUTPUT_SUMMARY],
            validation_gates: &[],
            retry_refs: &[],
            subrecipe_refs: &[],
            capability_ceilings: &[CAPABILITY_MODEL],
        },
        RecipeDagStepDraft {
            step_id: DAG_STEP_RESEARCH_A,
            kind: "tool",
            depends_on: &[],
            input_refs: &[FILE_PARAM],
            output_refs: &[DAG_OUTPUT_SUMMARY],
            validation_gates: &[],
            retry_refs: &[],
            subrecipe_refs: &[],
            capability_ceilings: &[CAPABILITY_TOOL],
        },
    ];

    assert_eq!(duplicate_steps.len(), DAG_TWO_STEP_COUNT);
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &duplicate_steps,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::DuplicateDagStep { .. })
    ));
}

#[test]
fn recipe_dag_rejects_unknown_self_and_unsafe_edges() {
    let unknown_dependency = [RecipeDagStepDraft {
        step_id: DAG_STEP_SYNTHESIZE,
        kind: "prompt",
        depends_on: &["missing-step"],
        input_refs: &[TOPIC_PARAM],
        output_refs: &[DAG_OUTPUT_SUMMARY],
        validation_gates: &[],
        retry_refs: &[],
        subrecipe_refs: &[],
        capability_ceilings: &[CAPABILITY_MODEL],
    }];
    let self_edge = [RecipeDagStepDraft {
        step_id: DAG_STEP_RESEARCH_A,
        kind: "prompt",
        depends_on: &[DAG_STEP_RESEARCH_A],
        input_refs: &[TOPIC_PARAM],
        output_refs: &[DAG_OUTPUT_SUMMARY],
        validation_gates: &[],
        retry_refs: &[],
        subrecipe_refs: &[],
        capability_ceilings: &[CAPABILITY_MODEL],
    }];
    let unsafe_label = [RecipeDagStepDraft {
        step_id: "/home/private/raw-output",
        kind: "prompt",
        depends_on: &[],
        input_refs: &[TOPIC_PARAM],
        output_refs: &[DAG_OUTPUT_SUMMARY],
        validation_gates: &[],
        retry_refs: &[],
        subrecipe_refs: &[],
        capability_ceilings: &[CAPABILITY_MODEL],
    }];

    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &unknown_dependency,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::UnknownDagDependency { .. })
    ));
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &self_edge,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::SelfDagEdge { .. })
    ));
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &unsafe_label,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::UnsafeDagLabel { .. })
    ));
}

#[test]
fn recipe_dag_rejects_cycles_before_planning_effects() {
    let cyclic_steps = [
        RecipeDagStepDraft {
            step_id: DAG_STEP_RESEARCH_A,
            kind: "prompt",
            depends_on: &[DAG_STEP_RESEARCH_B],
            input_refs: &[TOPIC_PARAM],
            output_refs: &[DAG_OUTPUT_SUMMARY],
            validation_gates: &[],
            retry_refs: &[],
            subrecipe_refs: &[],
            capability_ceilings: &[CAPABILITY_MODEL],
        },
        RecipeDagStepDraft {
            step_id: DAG_STEP_RESEARCH_B,
            kind: "tool",
            depends_on: &[DAG_STEP_RESEARCH_A],
            input_refs: &[FILE_PARAM],
            output_refs: &[DAG_OUTPUT_SUMMARY],
            validation_gates: &[],
            retry_refs: &[],
            subrecipe_refs: &[],
            capability_ceilings: &[CAPABILITY_TOOL],
        },
    ];

    assert_eq!(cyclic_steps.len(), DAG_TWO_STEP_COUNT);
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &cyclic_steps,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::DagCycle { .. })
    ));
}

#[test]
fn recipe_dag_rejects_undeclared_outputs() {
    let undeclared_output = [
        RecipeDagStepDraft {
            step_id: DAG_STEP_RESEARCH_A,
            kind: "prompt",
            depends_on: &[],
            input_refs: &[TOPIC_PARAM],
            output_refs: &[DAG_OUTPUT_SUMMARY],
            validation_gates: &[],
            retry_refs: &[],
            subrecipe_refs: &[],
            capability_ceilings: &[CAPABILITY_MODEL],
        },
        RecipeDagStepDraft {
            step_id: DAG_STEP_SYNTHESIZE,
            kind: "validation_gate",
            depends_on: &[DAG_STEP_RESEARCH_A],
            input_refs: &["research-a.missing"],
            output_refs: &[OUTPUT_STATUS],
            validation_gates: &[VALIDATION_GATE],
            retry_refs: &[],
            subrecipe_refs: &[],
            capability_ceilings: &[CAPABILITY_MODEL],
        },
    ];

    assert_eq!(undeclared_output.len(), DAG_TWO_STEP_COUNT);
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &undeclared_output,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::UndeclaredDagOutput { .. })
    ));
}

#[test]
fn recipe_dag_rejects_untrusted_subrecipes() {
    let untrusted_subrecipe = [RecipeDagStepDraft {
        step_id: DAG_STEP_SUBRECIPE,
        kind: "subrecipe",
        depends_on: &[],
        input_refs: &[TOPIC_PARAM],
        output_refs: &[DAG_OUTPUT_SUMMARY],
        validation_gates: &[],
        retry_refs: &[],
        subrecipe_refs: &["goose.recipe.ghost"],
        capability_ceilings: &[CAPABILITY_MODEL],
    }];

    assert_eq!(untrusted_subrecipe.len(), DAG_SINGLE_STEP_COUNT);
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &untrusted_subrecipe,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::UntrustedDagSubrecipe { .. })
    ));
}

#[test]
fn recipe_dag_rejects_capability_widening() {
    let capability_widening = [RecipeDagStepDraft {
        step_id: DAG_STEP_RESEARCH_A,
        kind: "tool",
        depends_on: &[],
        input_refs: &[FILE_PARAM],
        output_refs: &[DAG_OUTPUT_SUMMARY],
        validation_gates: &[],
        retry_refs: &[],
        subrecipe_refs: &[],
        capability_ceilings: &["network.fetch"],
    }];

    assert_eq!(capability_widening.len(), DAG_SINGLE_STEP_COUNT);
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &capability_widening,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::CapabilityWidening { .. })
    ));
}

#[test]
fn recipe_dag_rejects_retry_bypass() {
    let retry_bypass = [RecipeDagStepDraft {
        step_id: DAG_STEP_RESEARCH_B,
        kind: "retry",
        depends_on: &[],
        input_refs: &[TOPIC_PARAM],
        output_refs: &[DAG_OUTPUT_SUMMARY],
        validation_gates: &[],
        retry_refs: &["shell-bypass"],
        subrecipe_refs: &[],
        capability_ceilings: &[CAPABILITY_TOOL],
    }];

    assert_eq!(retry_bypass.len(), DAG_SINGLE_STEP_COUNT);
    assert!(matches!(
        RecipeWorkflow::validate(&RecipeWorkflowDraft {
            dag_steps: &retry_bypass,
            ..workflow_draft()
        }),
        Err(RecipeWorkflowError::RetryBypass { .. })
    ));
}
