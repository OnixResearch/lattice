use super::support::*;
use super::*;

#[test]
fn workflow_manifest_accepts_typed_recipe_with_toolsets() {
    let workflow = minimal_workflow();

    assert_eq!(workflow.identity(), WORKFLOW_ID);
    assert_eq!(workflow.activities().len(), ACTIVITY_COUNT);
    assert_eq!(workflow.parameters().len(), PARAMETER_COUNT);
    assert_eq!(workflow.model_profile(), MODEL_PROFILE);
    assert_eq!(workflow.toolsets()[FIRST_ITEM_INDEX], TOOLSET_READONLY);
    assert_eq!(
        workflow.validation_gates()[FIRST_ITEM_INDEX],
        VALIDATION_GATE
    );
    assert_eq!(workflow.retry_policy().max_attempts(), RETRY_ATTEMPTS);
    assert_eq!(workflow.response_schema().name(), RESPONSE_SCHEMA);
    assert_eq!(workflow.provenance().status(), RecipeTrustStatus::Admitted);
    assert_eq!(
        workflow.provenance().identity_hash().len(),
        TRUST_HASH_CHARS
    );
}

#[test]
fn parameter_binding_accepts_declared_variables_defaults_and_file_refs() {
    let workflow = minimal_workflow();
    let inputs = inputs_using_default();
    let binding = bind_recipe_parameters(RecipeBindingDraft {
        workflow: &workflow,
        template: workflow.initial_prompt(),
        inputs: &inputs,
    })
    .expect("binding should validate");

    assert!(
        binding
            .template_variables()
            .iter()
            .any(|variable| variable == TOPIC_PARAM)
    );
    assert!(
        binding
            .template_variables()
            .iter()
            .any(|variable| variable == "recipe.identity")
    );
    assert!(binding.bound_parameters().iter().any(|input| {
        input.name() == MODE_PARAM
            && input.value() == MODE_REVIEW
            && input.kind() == RecipeParameterInputKind::DefaultValue
    }));
    assert!(binding.bound_parameters().iter().any(|input| {
        input.name() == FILE_PARAM
            && input.value() == FILE_REF
            && input.kind() == RecipeParameterInputKind::FileContentRef
    }));
}

#[test]
fn structured_output_accepts_required_fields() {
    let workflow = minimal_workflow();
    let output = RecipeOutputDraft {
        fields: &[
            RecipeOutputFieldDraft {
                name: OUTPUT_STATUS,
                kind: "string",
            },
            RecipeOutputFieldDraft {
                name: OUTPUT_WARNINGS,
                kind: "array",
            },
        ],
    };

    let outcome = validate_recipe_output(workflow.response_schema(), &output)
        .expect("output fields should parse");

    assert_eq!(outcome.status(), RecipeAutomationStatus::Success);
    assert_eq!(outcome.failure_reason(), None);
}

#[test]
fn retry_plan_records_pending_and_denied_host_effect_requests() {
    let workflow = minimal_workflow();
    let pending = plan_recipe_retry(RecipeRetryPlanDraft {
        policy: workflow.retry_policy(),
        denied_capabilities: &[],
        denied_sandboxes: &[],
    });
    let denied = plan_recipe_retry(RecipeRetryPlanDraft {
        policy: workflow.retry_policy(),
        denied_capabilities: &[CAPABILITY_TOOL],
        denied_sandboxes: &[],
    });

    assert_eq!(
        pending.requests()[FIRST_ITEM_INDEX].status(),
        RecipeRetryRequestStatus::PendingHostEffect
    );
    assert_eq!(
        denied.requests()[FIRST_ITEM_INDEX].status(),
        RecipeRetryRequestStatus::Denied {
            reason: RecipeRetryDenialReason::Capability,
        }
    );
}

#[test]
fn admitted_trust_allows_bounded_recipe_influence() {
    let receipt = RecipeTrustReceipt::new(&admitted_provenance()).expect("receipt should parse");
    let influence = RecipeInfluence::evaluate(&receipt);

    assert_eq!(influence.status(), RecipeInfluenceStatus::Active);
    assert!(influence.prompt_allowed());
    assert!(influence.tools_allowed());
    assert!(influence.model_allowed());
    assert!(influence.memory_allowed());
    assert!(influence.workflow_permissions_allowed());
}

#[test]
fn recipe_dag_accepts_parallel_fanin_subrecipe_and_retry_plan() {
    let workflow = dag_workflow();
    let plan = workflow.dag_plan().expect("DAG plan should be present");

    assert_eq!(plan.steps().len(), dag_workflow_draft().dag_steps.len());
    assert_eq!(plan.batches().len(), DAG_BATCH_COUNT);
    assert_eq!(
        plan.batches()[FIRST_ITEM_INDEX].step_ids().len(),
        DAG_FIRST_BATCH_WIDTH
    );
    assert!(
        plan.batches()[FIRST_ITEM_INDEX]
            .step_ids()
            .iter()
            .any(|step| step == DAG_STEP_RESEARCH_A)
    );
    assert!(
        plan.batches()[FIRST_ITEM_INDEX]
            .step_ids()
            .iter()
            .any(|step| step == DAG_STEP_RESEARCH_B)
    );
    assert_eq!(
        plan.batches()[DAG_SYNTHESIS_BATCH_INDEX].step_ids()[FIRST_ITEM_INDEX],
        DAG_STEP_SYNTHESIZE
    );
    assert_eq!(plan.blocked_steps(), &[]);
    assert_eq!(
        plan.receipt().schema_label(),
        "animus.recipe-dag.receipt.v1"
    );
    assert_eq!(plan.receipt().identity_hash().len(), DAG_RECEIPT_HASH_CHARS);
    assert_eq!(plan.receipt().issue_codes()[FIRST_ITEM_INDEX], "none");
}

#[test]
fn recipe_dag_admission_blocks_denied_subrecipes_without_execution() {
    let workflow = dag_workflow();
    let plan = plan_recipe_dag(
        &workflow,
        RecipeDagAdmission {
            capabilities: &[],
            validation_gates: &[],
            subrecipes: &[SUBRECIPE],
        },
    )
    .expect("admission labels should be safe")
    .expect("DAG plan should be present");

    assert!(plan.blocked_steps().iter().any(|blocked| {
        blocked.step_id() == DAG_STEP_SUBRECIPE
            && blocked.reason() == RecipeDagBlockReason::SubrecipeDenied
            && blocked.detail() == SUBRECIPE
    }));
    assert!(
        plan.receipt()
            .issue_codes()
            .iter()
            .any(|code| code == "RECIPE_DAG_SUBRECIPE_DENIED")
    );
}
