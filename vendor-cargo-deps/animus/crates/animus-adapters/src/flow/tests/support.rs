use super::super::*;

pub const WORKFLOW_ID: &str = "goose.recipe.workflow";
pub const INSTRUCTIONS: &str = "Run the bounded workflow and return structured output.";
pub const TEMPLATE: &str = "Investigate {{topic}} in {{mode}} using {{context_file}} for {{recipe.identity}}.";
pub const MODEL_PROFILE: &str = "standard-reasoning";
pub const ACTIVITY_PLAN: &str = "plan";
pub const ACTIVITY_EXECUTE: &str = "execute";
pub const CAPABILITY_MODEL: &str = "model.generate";
pub const CAPABILITY_TOOL: &str = "tool.read";
pub const TOOLSET_READONLY: &str = "readonly-tools";
pub const VALIDATION_GATE: &str = "structured-output";
pub const SUBRECIPE: &str = "goose.recipe.child";
pub const TOPIC_PARAM: &str = "topic";
pub const MODE_PARAM: &str = "mode";
pub const FILE_PARAM: &str = "context_file";
pub const MODE_REVIEW: &str = "review";
pub const MODE_FIX: &str = "fix";
pub const TOPIC_VALUE: &str = "adapter contracts";
pub const FILE_REF: &str = "content-ref:context";
pub const CHECK_LABEL: &str = "cargo-test";
pub const CLEANUP_LABEL: &str = "cleanup-target";
pub const RECEIPT_LABEL: &str = "retry-receipt";
pub const SANDBOX_READONLY: &str = "readonly";
pub const TIMEOUT_STANDARD: &str = "standard";
pub const RESPONSE_SCHEMA: &str = "workflow-result";
pub const OUTPUT_STATUS: &str = "status";
pub const OUTPUT_WARNINGS: &str = "warnings";
pub const CONTENT_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub const TRUST_HASH_CHARS: usize = 64;
pub const RETRY_ATTEMPTS: u32 = 2;
pub const ZERO_RETRY_ATTEMPTS: u32 = 0;
pub const FIRST_ITEM_INDEX: usize = 0;
pub const ACTIVITY_COUNT: usize = 2;
pub const PARAMETER_COUNT: usize = 3;
pub const DAG_STEP_RESEARCH_A: &str = "research-a";
pub const DAG_STEP_RESEARCH_B: &str = "research-b";
pub const DAG_STEP_SYNTHESIZE: &str = "synthesize";
pub const DAG_STEP_SUBRECIPE: &str = "child-run";
pub const DAG_OUTPUT_SUMMARY: &str = "summary";
pub const DAG_INPUT_RESEARCH_A_SUMMARY: &str = "research-a.summary";
pub const DAG_INPUT_RESEARCH_B_SUMMARY: &str = "research-b.summary";
pub const DAG_BATCH_COUNT: usize = 3;
pub const DAG_SINGLE_STEP_COUNT: usize = 1;
pub const DAG_TWO_STEP_COUNT: usize = 2;
pub const DAG_FIRST_BATCH_WIDTH: usize = 2;
pub const DAG_SYNTHESIS_BATCH_INDEX: usize = 1;
pub const DAG_RECEIPT_HASH_CHARS: usize = 64;
pub const RETRY_CHECK: RecipeRetryCheckDraft<'static> = RecipeRetryCheckDraft {
    label: CHECK_LABEL,
    effect_kind: "validation_check",
    capability: CAPABILITY_TOOL,
    sandbox: SANDBOX_READONLY,
    timeout_class: TIMEOUT_STANDARD,
    cleanup_request: Some(CLEANUP_LABEL),
    receipt_label: RECEIPT_LABEL,
};

pub fn workflow_draft() -> RecipeWorkflowDraft<'static> {
    RecipeWorkflowDraft {
        identity: WORKFLOW_ID,
        instructions: INSTRUCTIONS,
        initial_prompt: TEMPLATE,
        activities: &[ACTIVITY_PLAN, ACTIVITY_EXECUTE],
        parameters: &[
            RecipeParameterDraft {
                name: TOPIC_PARAM,
                kind: "text",
                required: true,
                default_value: None,
                select_options: &[],
            },
            RecipeParameterDraft {
                name: MODE_PARAM,
                kind: "select",
                required: false,
                default_value: Some(MODE_REVIEW),
                select_options: &[MODE_REVIEW, MODE_FIX],
            },
            RecipeParameterDraft {
                name: FILE_PARAM,
                kind: "file",
                required: true,
                default_value: None,
                select_options: &[],
            },
        ],
        model_profile: MODEL_PROFILE,
        capability_ceilings: &[CAPABILITY_MODEL, CAPABILITY_TOOL],
        toolsets: &[TOOLSET_READONLY],
        validation_gates: &[VALIDATION_GATE],
        retry_policy: retry_policy_draft(),
        subrecipes: &[SUBRECIPE],
        dag_steps: &[],
        response_schema: response_schema_draft(),
        provenance: admitted_provenance(),
    }
}

pub fn minimal_workflow() -> RecipeWorkflow {
    RecipeWorkflow::validate(&workflow_draft()).expect("workflow fixture should validate")
}

pub fn dag_workflow_draft() -> RecipeWorkflowDraft<'static> {
    RecipeWorkflowDraft {
        dag_steps: &[
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
                step_id: DAG_STEP_RESEARCH_B,
                kind: "tool",
                depends_on: &[],
                input_refs: &[FILE_PARAM],
                output_refs: &[DAG_OUTPUT_SUMMARY],
                validation_gates: &[],
                retry_refs: &[CHECK_LABEL],
                subrecipe_refs: &[],
                capability_ceilings: &[CAPABILITY_TOOL],
            },
            RecipeDagStepDraft {
                step_id: DAG_STEP_SYNTHESIZE,
                kind: "validation_gate",
                depends_on: &[DAG_STEP_RESEARCH_A, DAG_STEP_RESEARCH_B],
                input_refs: &[DAG_INPUT_RESEARCH_A_SUMMARY, DAG_INPUT_RESEARCH_B_SUMMARY],
                output_refs: &[OUTPUT_STATUS],
                validation_gates: &[VALIDATION_GATE],
                retry_refs: &[],
                subrecipe_refs: &[],
                capability_ceilings: &[CAPABILITY_MODEL],
            },
            RecipeDagStepDraft {
                step_id: DAG_STEP_SUBRECIPE,
                kind: "subrecipe",
                depends_on: &[DAG_STEP_SYNTHESIZE],
                input_refs: &[DAG_STEP_SYNTHESIZE_OUTPUT_STATUS],
                output_refs: &[OUTPUT_WARNINGS],
                validation_gates: &[],
                retry_refs: &[],
                subrecipe_refs: &[SUBRECIPE],
                capability_ceilings: &[CAPABILITY_MODEL],
            },
        ],
        ..workflow_draft()
    }
}

pub const DAG_STEP_SYNTHESIZE_OUTPUT_STATUS: &str = "synthesize.status";

pub fn dag_workflow() -> RecipeWorkflow {
    RecipeWorkflow::validate(&dag_workflow_draft()).expect("DAG workflow fixture should validate")
}

pub fn retry_policy_draft() -> RecipeRetryPolicyDraft<'static> {
    RecipeRetryPolicyDraft {
        max_attempts: RETRY_ATTEMPTS,
        checks: &[RETRY_CHECK],
    }
}

#[derive(Clone, Copy)]
pub struct RetryCheckFixture {
    pub effect_kind: &'static str,
    pub deadline_label: &'static str,
}

pub fn retry_check_draft(input: RetryCheckFixture) -> RecipeRetryCheckDraft<'static> {
    RecipeRetryCheckDraft {
        label: CHECK_LABEL,
        effect_kind: input.effect_kind,
        capability: CAPABILITY_TOOL,
        sandbox: SANDBOX_READONLY,
        timeout_class: input.deadline_label,
        cleanup_request: Some(CLEANUP_LABEL),
        receipt_label: RECEIPT_LABEL,
    }
}

pub fn response_schema_draft() -> RecipeResponseSchemaDraft<'static> {
    RecipeResponseSchemaDraft {
        name: RESPONSE_SCHEMA,
        required_fields: &[
            RecipeResponseFieldDraft {
                name: OUTPUT_STATUS,
                kind: "string",
            },
            RecipeResponseFieldDraft {
                name: OUTPUT_WARNINGS,
                kind: "array",
            },
        ],
    }
}

pub fn admitted_provenance() -> RecipeProvenanceDraft<'static> {
    RecipeProvenanceDraft {
        source_kind: "project",
        source_label: "config/recipe-workflows.ncl",
        content_hash: CONTENT_HASH,
        admitted: true,
    }
}

pub fn pending_provenance() -> RecipeProvenanceDraft<'static> {
    RecipeProvenanceDraft {
        admitted: false,
        ..admitted_provenance()
    }
}

pub fn valid_inputs() -> [RecipeParameterInputDraft<'static>; 3] {
    [
        RecipeParameterInputDraft {
            name: TOPIC_PARAM,
            value: TOPIC_VALUE,
            source: "literal",
        },
        RecipeParameterInputDraft {
            name: MODE_PARAM,
            value: MODE_FIX,
            source: "literal",
        },
        RecipeParameterInputDraft {
            name: FILE_PARAM,
            value: FILE_REF,
            source: "file_ref",
        },
    ]
}

pub fn inputs_using_default() -> [RecipeParameterInputDraft<'static>; 2] {
    [
        RecipeParameterInputDraft {
            name: TOPIC_PARAM,
            value: TOPIC_VALUE,
            source: "literal",
        },
        RecipeParameterInputDraft {
            name: FILE_PARAM,
            value: FILE_REF,
            source: "file_ref",
        },
    ]
}
