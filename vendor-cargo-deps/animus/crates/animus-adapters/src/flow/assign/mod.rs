#![allow(non_trait_imports)]

const TEMPLATE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_template");
const TEMPLATE_VARIABLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_template_variable");
const PARAMETER_INPUT_NAME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_parameter_input_name");
const PARAMETER_INPUT_VALUE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_parameter_input_value");
const PARAMETER_INPUT_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_parameter_input_kind");

const TEMPLATE_OPEN: &str = "{{";
const TEMPLATE_CLOSE: &str = "}}";
const INPUT_LITERAL: &str = "literal";
const INPUT_FILE_REF: &str = "file_ref";
const INPUT_DEFAULT: &str = "default";

/// Template variables that reusable core treats as safe host-provided labels.
pub const SAFE_TEMPLATE_BUILTINS: &[&str] = &["recipe.identity", "recipe.source_label", "host.project_label"];

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

/// Untrusted parameter binding request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeBindingDraft<'a> {
    /// Validated recipe workflow whose parameters are being bound.
    pub workflow: &'a super::data::RecipeWorkflow,
    /// Prompt or activity template that references `{{variables}}`.
    pub template: &'a str,
    /// Caller-supplied parameter values.
    pub inputs: &'a [RecipeParameterInputDraft<'a>],
}

/// Untrusted parameter value supplied by a caller or host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeParameterInputDraft<'a> {
    /// Parameter name.
    pub name: &'a str,
    /// Parameter value or host content-reference label.
    pub value: &'a str,
    /// Input source kind label.
    pub source: &'a str,
}

/// Parameter input source kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeParameterInputKind {
    /// Literal caller-provided value.
    Literal,
    /// Host-provided file content reference label.
    FileContentRef,
    /// Validated default value from the manifest.
    DefaultValue,
}

impl RecipeParameterInputKind {
    /// Parses a stable parameter input kind label.
    ///
    /// # Errors
    ///
    /// Returns [`super::RecipeWorkflowError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::RecipeWorkflowError> {
        super::parse_workflow_value(PARAMETER_INPUT_KIND_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            INPUT_LITERAL => Some(Self::Literal),
            INPUT_FILE_REF => Some(Self::FileContentRef),
            INPUT_DEFAULT => Some(Self::DefaultValue),
            _ => None,
        }
    }

    /// Returns the stable input source kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Literal => INPUT_LITERAL,
            Self::FileContentRef => INPUT_FILE_REF,
            Self::DefaultValue => INPUT_DEFAULT,
        }
    }
}

/// Validated parameter value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeParameterInput {
    name: Text,
    value: Text,
    kind: RecipeParameterInputKind,
}

impl RecipeParameterInput {
    fn validate(draft: &RecipeParameterInputDraft<'_>) -> Result<Self, super::RecipeWorkflowError> {
        let name = super::required_text(PARAMETER_INPUT_NAME_FIELD, draft.name)?;
        let value = super::required_text(PARAMETER_INPUT_VALUE_FIELD, draft.value)?;
        let kind = RecipeParameterInputKind::parse(draft.source)?;
        Ok(Self { name, value, kind })
    }

    fn from_default(name: &str, value: &str) -> Self {
        Self {
            name: Text::from(name),
            value: Text::from(value),
            kind: RecipeParameterInputKind::DefaultValue,
        }
    }

    /// Returns the parameter name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the parameter value or host content-reference label.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Returns the parameter input source kind.
    #[must_use]
    pub const fn kind(&self) -> RecipeParameterInputKind {
        self.kind
    }
}

/// Validated parameter binding facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeBinding {
    template_variables: Collection<Text>,
    bound_parameters: Collection<RecipeParameterInput>,
}

impl RecipeBinding {
    /// Returns variables referenced by the template.
    #[must_use]
    pub fn template_variables(&self) -> &[Text] {
        self.template_variables.as_slice()
    }

    /// Returns bound parameter values.
    #[must_use]
    pub fn bound_parameters(&self) -> &[RecipeParameterInput] {
        self.bound_parameters.as_slice()
    }
}

/// Validates parameter binding before prompt assembly.
///
/// # Errors
///
/// Returns [`super::RecipeWorkflowError`] when the template is malformed,
/// references undeclared variables, leaves declared parameters unused, omits
/// required values, violates select options, or injects file parameters without a
/// host content-reference source.
pub fn bind_recipe_parameters(draft: RecipeBindingDraft<'_>) -> Result<RecipeBinding, super::RecipeWorkflowError> {
    crate::validation::validate_required_adapter_field(TEMPLATE_FIELD, draft.template)?;
    let template_variables = tokens::collect_template_variables(draft.template)?;
    tokens::validate_template_variables(draft.workflow, template_variables.as_slice())?;
    let supplied = collect_inputs(draft.inputs)?;
    let bound_parameters = bind_parameters(draft.workflow.parameters(), supplied.as_slice())?;

    Ok(RecipeBinding {
        template_variables,
        bound_parameters,
    })
}

mod check;
mod tokens;
fn collect_inputs(
    drafts: &[RecipeParameterInputDraft<'_>],
) -> Result<Collection<RecipeParameterInput>, super::RecipeWorkflowError> {
    let mut inputs: Collection<RecipeParameterInput> = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let input = RecipeParameterInput::validate(draft)?;
        if inputs.iter().any(|existing| existing.name() == input.name()) {
            return Err(super::RecipeWorkflowError::DuplicateName {
                field: PARAMETER_INPUT_NAME_FIELD.as_str(),
                value: Text::from(input.name()),
            });
        }
        inputs.push(input);
    }
    Ok(inputs)
}

fn bind_parameters(
    parameters: &[super::field::RecipeParameter],
    supplied: &[RecipeParameterInput],
) -> Result<Collection<RecipeParameterInput>, super::RecipeWorkflowError> {
    let mut bound: Collection<RecipeParameterInput> = Collection::with_capacity(parameters.len());
    for parameter in parameters {
        let input = match check::find_input(supplied, parameter.name()) {
            Some(input) => input.clone(),
            None => default_input(parameter)?,
        };
        check::validate_input_for_parameter(parameter, &input)?;
        bound.push(input);
    }
    Ok(bound)
}

fn default_input(
    parameter: &super::field::RecipeParameter,
) -> Result<RecipeParameterInput, super::RecipeWorkflowError> {
    parameter.default_value().map_or_else(
        || {
            Err(super::RecipeWorkflowError::MissingParameterValue {
                parameter: Text::from(parameter.name()),
            })
        },
        |default_value| Ok(RecipeParameterInput::from_default(parameter.name(), default_value)),
    )
}
