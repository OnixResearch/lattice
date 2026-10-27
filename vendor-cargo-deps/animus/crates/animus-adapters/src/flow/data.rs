const IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_identity");
const INSTRUCTIONS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_instructions");
const PROMPT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_initial_prompt");
const ACTIVITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_activity");
const MODEL_PROFILE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_model_profile");
const CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_capability");
const TOOLSET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_toolset");
const GATE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_validation_gate");
const SUBRECIPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_subrecipe");
type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;
/// Untrusted recipe/workflow manifest fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeWorkflowDraft<'a> {
    /// Stable recipe identity.
    pub identity: &'a str,
    /// Instruction block used by prompt assembly after policy admission.
    pub instructions: &'a str,
    /// Initial prompt template before parameter binding.
    pub initial_prompt: &'a str,
    /// Ordered activity labels for the workflow.
    pub activities: &'a [&'a str],
    /// Declared input parameters.
    pub parameters: &'a [super::field::RecipeParameterDraft<'a>],
    /// Model profile label requested by the recipe.
    pub model_profile: &'a str,
    /// Capability ceiling labels the recipe may request.
    pub capability_ceilings: &'a [&'a str],
    /// Toolset reference labels declared by the recipe.
    pub toolsets: &'a [&'a str],
    /// Validation gate labels that must pass before success is reported.
    pub validation_gates: &'a [&'a str],
    /// Host-mediated retry policy metadata.
    pub retry_policy: super::attempt::RecipeRetryPolicyDraft<'a>,
    /// Referenced subrecipe identities.
    pub subrecipes: &'a [&'a str],
    /// Optional recipe DAG step descriptors.
    pub dag_steps: &'a [super::graph::RecipeDagStepDraft<'a>],
    /// Structured final response schema.
    pub response_schema: super::shape::RecipeResponseSchemaDraft<'a>,
    /// Trust and provenance metadata.
    pub provenance: super::admit::RecipeProvenanceDraft<'a>,
}
/// Validated recipe workflow manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeWorkflow {
    identity: Text,
    instructions: Text,
    initial_prompt: Text,
    activities: Collection<Text>,
    parameters: Collection<super::field::RecipeParameter>,
    model_profile: Text,
    capability_ceilings: Collection<Text>,
    toolsets: Collection<Text>,
    validation_gates: Collection<Text>,
    retry_policy: super::attempt::RecipeRetryPolicy,
    subrecipes: Collection<Text>,
    dag_plan: Option<super::graph::RecipeDagPlan>,
    response_schema: super::shape::RecipeResponseSchema,
    provenance: super::admit::RecipeTrustReceipt,
}

impl RecipeWorkflow {
    /// Validates untrusted recipe workflow metadata without I/O or effects.
    ///
    /// # Errors
    ///
    /// Returns [`super::RecipeWorkflowError`] when required fields are empty,
    /// collections are invalid, parameters are unsafe, retry policy is
    /// unbounded, structured output schema is malformed, or provenance is not a
    /// valid BLAKE3-identified trust receipt.
    pub fn validate(draft: &RecipeWorkflowDraft<'_>) -> Result<Self, super::RecipeWorkflowError> {
        let identity = super::required_text(IDENTITY_FIELD, draft.identity)?;
        let instructions = super::required_text(INSTRUCTIONS_FIELD, draft.instructions)?;
        let initial_prompt = super::required_text(PROMPT_FIELD, draft.initial_prompt)?;
        let activities = super::text_collection(ACTIVITY_FIELD, draft.activities, true)?;
        let parameters = super::field::collect_parameters(draft.parameters)?;
        let model_profile = super::required_text(MODEL_PROFILE_FIELD, draft.model_profile)?;
        let capability_ceilings =
            super::text_collection(CAPABILITY_FIELD, draft.capability_ceilings, true)?;
        let toolsets = super::text_collection(TOOLSET_FIELD, draft.toolsets, false)?;
        let validation_gates = super::text_collection(GATE_FIELD, draft.validation_gates, true)?;
        let retry_policy = super::attempt::RecipeRetryPolicy::validate(&draft.retry_policy)?;
        let subrecipes = super::text_collection(SUBRECIPE_FIELD, draft.subrecipes, false)?;
        let response_schema = super::shape::RecipeResponseSchema::validate(&draft.response_schema)?;
        let provenance = super::admit::RecipeTrustReceipt::new(&draft.provenance)?;
        let dag_plan = super::graph::validate_recipe_dag(
            draft.dag_steps,
            super::graph::RecipeDagContext {
                workflow_id: identity.as_str(),
                parameters: parameters.as_slice(),
                capability_ceilings: capability_ceilings.as_slice(),
                validation_gates: validation_gates.as_slice(),
                retry_policy: &retry_policy,
                subrecipes: subrecipes.as_slice(),
            },
        )?;

        Ok(Self {
            identity,
            instructions,
            initial_prompt,
            activities,
            parameters,
            model_profile,
            capability_ceilings,
            toolsets,
            validation_gates,
            retry_policy,
            subrecipes,
            dag_plan,
            response_schema,
            provenance,
        })
    }

    /// Returns the stable recipe identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the instruction block.
    #[must_use]
    pub fn instructions(&self) -> &str {
        self.instructions.as_str()
    }

    /// Returns the initial prompt template.
    #[must_use]
    pub fn initial_prompt(&self) -> &str {
        self.initial_prompt.as_str()
    }

    /// Returns ordered activity labels.
    #[must_use]
    pub fn activities(&self) -> &[Text] {
        self.activities.as_slice()
    }

    /// Returns validated parameter declarations.
    #[must_use]
    pub fn parameters(&self) -> &[super::field::RecipeParameter] {
        self.parameters.as_slice()
    }

    /// Returns the requested model profile label.
    #[must_use]
    pub fn model_profile(&self) -> &str {
        self.model_profile.as_str()
    }

    /// Returns capability ceiling labels.
    #[must_use]
    pub fn capability_ceilings(&self) -> &[Text] {
        self.capability_ceilings.as_slice()
    }

    /// Returns declared toolset references.
    #[must_use]
    pub fn toolsets(&self) -> &[Text] {
        self.toolsets.as_slice()
    }

    /// Returns validation gate labels.
    #[must_use]
    pub fn validation_gates(&self) -> &[Text] {
        self.validation_gates.as_slice()
    }

    /// Returns the host-mediated retry policy.
    #[must_use]
    pub const fn retry_policy(&self) -> &super::attempt::RecipeRetryPolicy {
        &self.retry_policy
    }

    /// Returns referenced subrecipe identities.
    #[must_use]
    pub fn subrecipes(&self) -> &[Text] {
        self.subrecipes.as_slice()
    }

    /// Returns the optional validated recipe DAG plan.
    #[must_use]
    pub const fn dag_plan(&self) -> Option<&super::graph::RecipeDagPlan> {
        self.dag_plan.as_ref()
    }

    /// Returns the structured final response schema.
    #[must_use]
    pub const fn response_schema(&self) -> &super::shape::RecipeResponseSchema {
        &self.response_schema
    }

    /// Returns the recipe trust receipt.
    #[must_use]
    pub const fn provenance(&self) -> &super::admit::RecipeTrustReceipt {
        &self.provenance
    }
}
