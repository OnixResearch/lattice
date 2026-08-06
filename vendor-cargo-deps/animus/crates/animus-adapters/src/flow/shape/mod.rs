#![allow(non_trait_imports)]

const SCHEMA_NAME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("recipe_response_schema_name");
const FIELD_NAME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_response_field_name");
const FIELD_KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_response_field_kind");

const KIND_STRING: &str = "string";
const KIND_NUMBER: &str = "number";
const KIND_BOOL: &str = "bool";
const KIND_OBJECT: &str = "object";
const KIND_ARRAY: &str = "array";

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

/// Untrusted structured response schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeResponseSchemaDraft<'a> {
    /// Stable schema name.
    pub name: &'a str,
    /// Required fields for successful automation output.
    pub required_fields: &'a [RecipeResponseFieldDraft<'a>],
}

/// Untrusted response field declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeResponseFieldDraft<'a> {
    /// Field name.
    pub name: &'a str,
    /// Field kind label.
    pub kind: &'a str,
}
/// Supported structured response field kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeResponseFieldKind {
    /// JSON string value.
    String,
    /// JSON number value.
    Number,
    /// JSON boolean value.
    Bool,
    /// JSON object value.
    Object,
    /// JSON array value.
    Array,
}

impl RecipeResponseFieldKind {
    /// Parses a stable response field kind label.
    ///
    /// # Errors
    ///
    /// Returns [`super::RecipeWorkflowError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::RecipeWorkflowError> {
        super::parse_workflow_value(FIELD_KIND_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            KIND_STRING => Some(Self::String),
            KIND_NUMBER => Some(Self::Number),
            KIND_BOOL => Some(Self::Bool),
            KIND_OBJECT => Some(Self::Object),
            KIND_ARRAY => Some(Self::Array),
            _ => None,
        }
    }

    /// Returns the stable response field kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => KIND_STRING,
            Self::Number => KIND_NUMBER,
            Self::Bool => KIND_BOOL,
            Self::Object => KIND_OBJECT,
            Self::Array => KIND_ARRAY,
        }
    }
}

/// Validated structured response schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeResponseSchema {
    name: Text,
    required_fields: Collection<RecipeResponseField>,
}

impl RecipeResponseSchema {
    /// Validates a final response schema.
    ///
    /// # Errors
    ///
    /// Returns [`super::RecipeWorkflowError`] when the schema is unnamed,
    /// declares no required fields, repeats a field, or names an unsupported kind.
    pub fn validate(draft: &RecipeResponseSchemaDraft<'_>) -> Result<Self, super::RecipeWorkflowError> {
        let name = super::required_text(SCHEMA_NAME_FIELD, draft.name)?;
        if draft.required_fields.is_empty() {
            return Err(super::RecipeWorkflowError::EmptyCollection {
                field: FIELD_NAME_FIELD.as_str(),
            });
        }
        let required_fields = collect_fields(draft.required_fields)?;

        Ok(Self { name, required_fields })
    }

    /// Returns the stable schema name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns required output fields.
    #[must_use]
    pub fn required_fields(&self) -> &[RecipeResponseField] {
        self.required_fields.as_slice()
    }
}

/// Validated response field declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeResponseField {
    name: Text,
    kind: RecipeResponseFieldKind,
}

impl RecipeResponseField {
    fn validate(draft: &RecipeResponseFieldDraft<'_>) -> Result<Self, super::RecipeWorkflowError> {
        let name = super::required_text(FIELD_NAME_FIELD, draft.name)?;
        let kind = RecipeResponseFieldKind::parse(draft.kind)?;
        Ok(Self { name, kind })
    }

    /// Returns the field name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the field kind.
    #[must_use]
    pub const fn kind(&self) -> RecipeResponseFieldKind {
        self.kind
    }
}

mod check;

pub use check::RecipeAutomationOutcome;
pub use check::RecipeAutomationStatus;
pub use check::RecipeOutputDraft;
pub use check::RecipeOutputFailure;
pub use check::RecipeOutputFieldDraft;
pub use check::validate_recipe_output;
fn collect_fields(
    drafts: &[RecipeResponseFieldDraft<'_>],
) -> Result<Collection<RecipeResponseField>, super::RecipeWorkflowError> {
    let mut fields: Collection<RecipeResponseField> = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let field = RecipeResponseField::validate(draft)?;
        if fields.iter().any(|existing| existing.name() == field.name()) {
            return Err(super::RecipeWorkflowError::DuplicateName {
                field: FIELD_NAME_FIELD.as_str(),
                value: Text::from(field.name()),
            });
        }
        fields.push(field);
    }
    Ok(fields)
}
