# Nickel is the source of truth for Animus recipe workflow manifests.
# Export derived JSON with scripts/config/export-nickel-configs.

let NonEmptyString = std.contract.from_predicate (fun value => value != "") in
let PositiveNumber = std.contract.from_predicate (fun value => value > 0) in
let ParameterKind = [| 'text, 'select, 'file |] in
let SourceKind = [| 'first_run, 'changed, 'imported, 'project |] in
let HostEffectKind = [| 'validation_check, 'cleanup_request |] in
let RecipeDagStepKind = [| 'prompt, 'tool, 'validation_gate, 'subrecipe, 'retry, 'cleanup |] in
let ResponseFieldKind = [| 'string, 'number, 'bool, 'object, 'array |] in
let Parameter = {
  name | NonEmptyString,
  kind | ParameterKind,
  required | Bool,
  default_value | String,
  select_options | Array String,
} in
let RetryCheck = {
  label | NonEmptyString,
  effect_kind | HostEffectKind,
  capability | NonEmptyString,
  sandbox | NonEmptyString,
  timeout_class | NonEmptyString,
  cleanup_request | String,
  receipt_label | NonEmptyString,
} in
let RetryPolicy = {
  max_attempts | PositiveNumber,
  checks | Array RetryCheck,
} in
let ResponseField = {
  name | NonEmptyString,
  kind | ResponseFieldKind,
} in
let ResponseSchema = {
  name | NonEmptyString,
  required_fields | Array ResponseField,
} in
let RecipeDagStep = {
  step_id | NonEmptyString,
  kind | RecipeDagStepKind,
  depends_on | Array String,
  input_refs | Array String,
  output_refs | Array String,
  validation_gates | Array String,
  retry_refs | Array String,
  subrecipe_refs | Array String,
  capability_ceilings | Array String,
} in
let Provenance = {
  source_kind | SourceKind,
  source_label | NonEmptyString,
  content_hash | NonEmptyString,
  admitted | Bool,
} in
let Workflow = {
  identity | NonEmptyString,
  instructions | NonEmptyString,
  initial_prompt | NonEmptyString,
  activities | Array String,
  parameters | Array Parameter,
  model_profile | NonEmptyString,
  capability_ceilings | Array String,
  toolsets | Array String,
  validation_gates | Array String,
  retry_policy | RetryPolicy,
  subrecipes | Array String,
  dag_steps | Array RecipeDagStep,
  response_schema | ResponseSchema,
  provenance | Provenance,
} in
let WorkflowCatalog = {
  schema | NonEmptyString,
  workflows | Array Workflow,
} in

let catalog_schema = "animus.recipe-workflows.v1" in
let workflow_identity = "goose.recipe.workflow" in
let workflow_instructions = "Run the bounded workflow and return structured output." in
let workflow_prompt = "Investigate {{topic}} in {{mode}} using {{context_file}} for {{recipe.identity}}." in
let text_parameter = 'text in
let select_parameter = 'select in
let file_parameter = 'file in
let validation_effect = 'validation_check in
let project_source = 'project in
let string_field = 'string in
let array_field = 'array in
let retry_attempts = 2 in
let empty_default = "" in
let topic_parameter = "topic" in
let mode_parameter = "mode" in
let context_file_parameter = "context_file" in
let review_mode = "review" in
let fix_mode = "fix" in
let model_profile_label = "standard-reasoning" in
let model_capability = "model.generate" in
let tool_capability = "tool.read" in
let readonly_toolset = "readonly-tools" in
let output_gate = "structured-output" in
let child_recipe = "goose.recipe.child" in
let retry_check = "cargo-test" in
let readonly_sandbox = "readonly" in
let standard_timeout = "standard" in
let cleanup_request_label = "cleanup-target" in
let retry_receipt = "retry-receipt" in
let response_schema_name = "workflow-result" in
let output_status = "status" in
let output_warnings = "warnings" in
let dag_step_research_a = "research-a" in
let dag_step_research_b = "research-b" in
let dag_step_synthesize = "synthesize" in
let dag_step_child_run = "child-run" in
let dag_output_summary = "summary" in
let dag_research_a_summary = "research-a.summary" in
let dag_research_b_summary = "research-b.summary" in
let dag_synthesize_status = "synthesize.status" in
let dag_kind_prompt = 'prompt in
let dag_kind_tool = 'tool in
let dag_kind_validation_gate = 'validation_gate in
let dag_kind_subrecipe = 'subrecipe in
let recipe_source = "config/recipe-workflows.ncl" in
let recipe_content_hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in
{
  recipe_workflow_catalog_contract = WorkflowCatalog,
  recipe_workflow_catalog | WorkflowCatalog = {
    schema = catalog_schema,
    workflows = [
      {
        identity = workflow_identity,
        instructions = workflow_instructions,
        initial_prompt = workflow_prompt,
        activities = ["plan", "execute"],
        parameters = [
          {
            name = topic_parameter,
            kind = text_parameter,
            required = true,
            default_value = empty_default,
            select_options = [],
          },
          {
            name = mode_parameter,
            kind = select_parameter,
            required = false,
            default_value = review_mode,
            select_options = [review_mode, fix_mode],
          },
          {
            name = context_file_parameter,
            kind = file_parameter,
            required = true,
            default_value = empty_default,
            select_options = [],
          },
        ],
        model_profile = model_profile_label,
        capability_ceilings = [model_capability, tool_capability],
        toolsets = [readonly_toolset],
        validation_gates = [output_gate],
        retry_policy = {
          max_attempts = retry_attempts,
          checks = [
            {
              label = retry_check,
              effect_kind = validation_effect,
              capability = tool_capability,
              sandbox = readonly_sandbox,
              timeout_class = standard_timeout,
              cleanup_request = cleanup_request_label,
              receipt_label = retry_receipt,
            },
          ],
        },
        subrecipes = [child_recipe],
        dag_steps = [
          {
            step_id = dag_step_research_a,
            kind = dag_kind_prompt,
            depends_on = [],
            input_refs = [topic_parameter],
            output_refs = [dag_output_summary],
            validation_gates = [],
            retry_refs = [],
            subrecipe_refs = [],
            capability_ceilings = [model_capability],
          },
          {
            step_id = dag_step_research_b,
            kind = dag_kind_tool,
            depends_on = [],
            input_refs = [context_file_parameter],
            output_refs = [dag_output_summary],
            validation_gates = [],
            retry_refs = [retry_check],
            subrecipe_refs = [],
            capability_ceilings = [tool_capability],
          },
          {
            step_id = dag_step_synthesize,
            kind = dag_kind_validation_gate,
            depends_on = [dag_step_research_a, dag_step_research_b],
            input_refs = [dag_research_a_summary, dag_research_b_summary],
            output_refs = [output_status],
            validation_gates = [output_gate],
            retry_refs = [],
            subrecipe_refs = [],
            capability_ceilings = [model_capability],
          },
          {
            step_id = dag_step_child_run,
            kind = dag_kind_subrecipe,
            depends_on = [dag_step_synthesize],
            input_refs = [dag_synthesize_status],
            output_refs = [output_warnings],
            validation_gates = [],
            retry_refs = [],
            subrecipe_refs = [child_recipe],
            capability_ceilings = [model_capability],
          },
        ],
        response_schema = {
          name = response_schema_name,
          required_fields = [
            {
              name = output_status,
              kind = string_field,
            },
            {
              name = output_warnings,
              kind = array_field,
            },
          ],
        },
        provenance = {
          source_kind = project_source,
          source_label = recipe_source,
          content_hash = recipe_content_hash,
          admitted = true,
        },
      },
    ],
  },
}
