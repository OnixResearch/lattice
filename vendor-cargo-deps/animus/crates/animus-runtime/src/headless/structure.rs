use super::AutomationError;
use super::SCHEMA_ID_FIELD;
use super::STRUCTURED_FIELD;
use super::Text;
use super::TextField;
use super::contains_text;
use super::required_text;
use super::unique_texts;

/// Borrowed schema contract for structured output validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructuredSchemaDraft<'a> {
    /// Schema identity supplied by the shell or controller.
    pub schema_id: &'a str,
    /// Required top-level fields.
    pub required_fields: &'a [&'a str],
    /// Allowed top-level fields when extras are denied.
    pub allowed_fields: &'a [&'a str],
    /// Whether fields outside `allowed_fields` are accepted.
    pub allow_extra_fields: bool,
}

/// Borrowed observed structured-output field facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructuredOutputDraft<'a> {
    /// Observed top-level field names.
    pub fields: &'a [&'a str],
}

/// Outcome of structured output validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredOutputReport {
    schema_id: Text,
    status: StructuredOutputStatus,
    failure: Option<StructuredOutputFailure>,
}

impl StructuredOutputReport {
    fn success(schema_id: Text) -> Self {
        Self {
            schema_id,
            status: StructuredOutputStatus::Success,
            failure: None,
        }
    }

    fn failed(schema_id: Text, failure: StructuredOutputFailure) -> Self {
        Self {
            schema_id,
            status: StructuredOutputStatus::Failure,
            failure: Some(failure),
        }
    }

    /// Returns the schema identity.
    #[must_use]
    pub fn schema_id(&self) -> &str {
        self.schema_id.as_str()
    }

    /// Returns the validation status.
    #[must_use]
    pub const fn status(&self) -> StructuredOutputStatus {
        self.status
    }

    /// Returns the validation failure, when any.
    #[must_use]
    pub const fn failure(&self) -> Option<&StructuredOutputFailure> {
        self.failure.as_ref()
    }
}

/// Structured final-output validation status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuredOutputStatus {
    /// Output satisfied the schema contract.
    Success,
    /// Output failed the schema contract and must not be reported as success.
    Failure,
}

/// Structured final-output validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructuredOutputFailure {
    /// Required field was absent.
    MissingField {
        /// Missing field name.
        field: Text,
    },
    /// Extra field was present while extras were disallowed.
    ExtraField {
        /// Unexpected field name.
        field: Text,
    },
}

/// Validates observed final output against a schema-identified contract.
///
/// # Errors
///
/// Returns [`AutomationError`] when schema or observed field declarations are
/// malformed or contain duplicates. Schema mismatch results are returned as a
/// failure report so callers can record the failed run explicitly.
///
/// # Panics
///
/// Panics if required schema identity validation returns an empty identity.
pub fn validate_structured_output(
    schema: &StructuredSchemaDraft<'_>,
    output: &StructuredOutputDraft<'_>,
) -> Result<StructuredOutputReport, AutomationError> {
    let schema_id = required_text(TextField {
        field: SCHEMA_ID_FIELD,
        value: schema.schema_id,
    })?;
    let required_fields = unique_texts(STRUCTURED_FIELD, schema.required_fields)?;
    let allowed_fields = unique_texts(STRUCTURED_FIELD, schema.allowed_fields)?;
    let observed_fields = unique_texts(STRUCTURED_FIELD, output.fields)?;
    assert!(!schema_id.is_empty(), "schema identity is required");
    assert!(
        required_fields.len() <= schema.required_fields.len(),
        "deduplicated required fields never exceed source facts"
    );

    for field in &required_fields {
        if !contains_text(observed_fields.as_slice(), field.as_str()) {
            return Ok(StructuredOutputReport::failed(schema_id, StructuredOutputFailure::MissingField {
                field: field.clone(),
            }));
        }
    }
    if !schema.allow_extra_fields {
        for field in &observed_fields {
            if !contains_text(allowed_fields.as_slice(), field.as_str()) {
                return Ok(StructuredOutputReport::failed(schema_id, StructuredOutputFailure::ExtraField {
                    field: field.clone(),
                }));
            }
        }
    }
    Ok(StructuredOutputReport::success(schema_id))
}
