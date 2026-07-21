const PARAMETER_NAME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_parameter_name");
const PARAMETER_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_parameter_kind");
const PARAMETER_DEFAULT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_parameter_default");
const PARAMETER_OPTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_workflow_parameter_option");
const KIND_TEXT: &str = "text";
const KIND_SELECT: &str = "select";
const KIND_FILE: &str = "file";

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;
/// Untrusted recipe parameter declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeParameterDraft<'a> {
    /// Stable parameter name.
    pub name: &'a str,
    /// Parameter kind label.
    pub kind: &'a str,
    /// Whether a caller must provide this parameter explicitly.
    pub required: bool,
    /// Optional default value for non-file optional parameters.
    pub default_value: Option<&'a str>,
    /// Select-only allowed values.
    pub select_options: &'a [&'a str],
}
/// Supported recipe parameter kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeParameterKind {
    /// Plain literal text supplied by the caller or default.
    Text,
    /// Literal text constrained to declared options.
    Select,
    /// Host-provided file content reference, never raw default content.
    File,
}

impl RecipeParameterKind {
    /// Parses a stable parameter kind label.
    ///
    /// # Errors
    ///
    /// Returns [`super::RecipeWorkflowError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::RecipeWorkflowError> {
        super::parse_workflow_value(PARAMETER_KIND_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            KIND_TEXT => Some(Self::Text),
            KIND_SELECT => Some(Self::Select),
            KIND_FILE => Some(Self::File),
            _ => None,
        }
    }

    /// Returns the stable parameter kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => KIND_TEXT,
            Self::Select => KIND_SELECT,
            Self::File => KIND_FILE,
        }
    }
}

/// Validated recipe parameter declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeParameter {
    name: Text,
    kind: RecipeParameterKind,
    required: bool,
    default_value: Option<Text>,
    select_options: Collection<Text>,
}

impl RecipeParameter {
    fn validate(draft: &RecipeParameterDraft<'_>) -> Result<Self, super::RecipeWorkflowError> {
        let name = super::required_text(PARAMETER_NAME_FIELD, draft.name)?;
        let kind = RecipeParameterKind::parse(draft.kind)?;
        let default_value = super::optional_text(PARAMETER_DEFAULT_FIELD, draft.default_value)?;
        let select_options =
            super::text_collection(PARAMETER_OPTION_FIELD, draft.select_options, false)?;
        validate_parameter_defaults(name.as_str(), kind, draft.required, default_value.as_ref())?;
        validate_parameter_options(name.as_str(), kind, default_value.as_ref(), &select_options)?;

        Ok(Self {
            name,
            kind,
            required: draft.required,
            default_value,
            select_options,
        })
    }

    /// Returns the stable parameter name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the parameter kind.
    #[must_use]
    pub const fn kind(&self) -> RecipeParameterKind {
        self.kind
    }

    /// Returns whether the caller must supply this parameter explicitly.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }

    /// Returns the validated default value, when any.
    #[must_use]
    pub fn default_value(&self) -> Option<&str> {
        self.default_value.as_deref()
    }

    /// Returns declared select options.
    #[must_use]
    pub fn select_options(&self) -> &[Text] {
        self.select_options.as_slice()
    }
}

pub(super) fn collect_parameters(
    drafts: &[RecipeParameterDraft<'_>],
) -> Result<Collection<RecipeParameter>, super::RecipeWorkflowError> {
    let mut parameters: Collection<RecipeParameter> = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let parameter = RecipeParameter::validate(draft)?;
        if parameters
            .iter()
            .any(|existing| existing.name() == parameter.name())
        {
            return Err(super::RecipeWorkflowError::DuplicateName {
                field: PARAMETER_NAME_FIELD.as_str(),
                value: Text::from(parameter.name()),
            });
        }
        parameters.push(parameter);
    }
    Ok(parameters)
}

fn validate_parameter_defaults(
    name: &str,
    kind: RecipeParameterKind,
    required: bool,
    default_value: Option<&Text>,
) -> Result<(), super::RecipeWorkflowError> {
    let parameter = Text::from(name);
    if kind == RecipeParameterKind::File && default_value.is_some() {
        return Err(super::RecipeWorkflowError::FileParameterDefault { parameter });
    }
    if kind == RecipeParameterKind::File && !required {
        return Err(super::RecipeWorkflowError::OptionalFileParameter { parameter });
    }
    if !required && default_value.is_none() {
        return Err(super::RecipeWorkflowError::MissingOptionalDefault { parameter });
    }
    Ok(())
}

fn validate_parameter_options(
    name: &str,
    kind: RecipeParameterKind,
    default_value: Option<&Text>,
    select_options: &[Text],
) -> Result<(), super::RecipeWorkflowError> {
    assert!(!name.is_empty(), "parameter name was normalized");
    assert!(
        !PARAMETER_NAME_FIELD.as_str().is_empty(),
        "parameter fields are stable"
    );
    let parameter = Text::from(name);
    if kind == RecipeParameterKind::Select && select_options.is_empty() {
        return Err(super::RecipeWorkflowError::EmptySelectOptions { parameter });
    }
    if kind != RecipeParameterKind::Select && !select_options.is_empty() {
        return Err(super::RecipeWorkflowError::UnexpectedSelectOptions { parameter });
    }
    if let Some(default_text) = default_value {
        if kind == RecipeParameterKind::Select
            && !super::contains_text(select_options, default_text.as_str())
        {
            return Err(super::RecipeWorkflowError::InvalidSelectValue {
                parameter,
                value: default_text.clone(),
            });
        }
    }
    Ok(())
}
