use super::Collection;
use super::FIELD_NAME_FIELD;
use super::RecipeResponseFieldKind;
use super::RecipeResponseSchema;
use super::Text;

/// Untrusted observed final output facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeOutputDraft<'a> {
    /// Observed top-level fields and their kinds.
    pub fields: &'a [RecipeOutputFieldDraft<'a>],
}

/// Untrusted observed final output field fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeOutputFieldDraft<'a> {
    /// Observed field name.
    pub name: &'a str,
    /// Observed field kind label.
    pub kind: &'a str,
}
/// Result status for structured automation output validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeAutomationStatus {
    /// Final output satisfied the declared schema.
    Success,
    /// Final output failed schema validation and must not be consumed as success.
    StructuredOutputFailure,
}

/// Structured output validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecipeOutputFailure {
    /// A required field was absent.
    MissingField {
        /// Missing field name.
        field: Text,
    },
    /// A required field had the wrong kind.
    KindMismatch {
        /// Field name.
        field: Text,
        /// Expected field kind.
        expected: RecipeResponseFieldKind,
        /// Actual observed field kind.
        actual: RecipeResponseFieldKind,
    },
}

/// Automation outcome recorded before an external caller consumes recipe output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeAutomationOutcome {
    status: RecipeAutomationStatus,
    failure: Option<RecipeOutputFailure>,
}

impl RecipeAutomationOutcome {
    fn success() -> Self {
        Self {
            status: RecipeAutomationStatus::Success,
            failure: None,
        }
    }

    fn failure(failure: RecipeOutputFailure) -> Self {
        Self {
            status: RecipeAutomationStatus::StructuredOutputFailure,
            failure: Some(failure),
        }
    }

    /// Returns the automation validation status.
    #[must_use]
    pub const fn status(&self) -> RecipeAutomationStatus {
        self.status
    }

    /// Returns the structured validation failure, when any.
    #[must_use]
    pub const fn failure_reason(&self) -> Option<&RecipeOutputFailure> {
        self.failure.as_ref()
    }
}

/// Validates observed output facts against the declared final response schema.
///
/// # Errors
///
/// Returns [`super::RecipeWorkflowError`] when observed field labels are malformed
/// or repeated. Missing or wrong-typed fields produce a failure outcome instead
/// of an error so automation can record the denial explicitly.
///
/// # Panics
///
/// Panics if a response schema was constructed without required fields.
pub fn validate_recipe_output(
    schema: &RecipeResponseSchema,
    output: &RecipeOutputDraft<'_>,
) -> Result<RecipeAutomationOutcome, super::super::RecipeWorkflowError> {
    assert!(!schema.required_fields().is_empty(), "schemas require output fields");
    assert!(
        schema.required_fields().iter().all(|field| !field.name().is_empty()),
        "schema field names must be validated"
    );
    let observed = collect_output_fields(output.fields)?;
    assert!(observed.len() <= output.fields.len(), "deduplicated output fields never exceed observed fields");
    for required in schema.required_fields() {
        match find_field(observed.as_slice(), required.name()) {
            None => {
                return Ok(RecipeAutomationOutcome::failure(RecipeOutputFailure::MissingField {
                    field: Text::from(required.name()),
                }));
            }
            Some(actual) if actual.kind != required.kind() => {
                return Ok(RecipeAutomationOutcome::failure(RecipeOutputFailure::KindMismatch {
                    field: Text::from(required.name()),
                    expected: required.kind(),
                    actual: actual.kind,
                }));
            }
            Some(_actual) => {}
        }
    }
    Ok(RecipeAutomationOutcome::success())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedField {
    name: Text,
    kind: RecipeResponseFieldKind,
}
fn collect_output_fields(
    drafts: &[RecipeOutputFieldDraft<'_>],
) -> Result<Collection<ObservedField>, super::super::RecipeWorkflowError> {
    let mut fields: Collection<ObservedField> = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let name = super::super::required_text(FIELD_NAME_FIELD, draft.name)?;
        let kind = RecipeResponseFieldKind::parse(draft.kind)?;
        if fields.iter().any(|existing| existing.name.as_str() == name) {
            return Err(super::super::RecipeWorkflowError::DuplicateName {
                field: FIELD_NAME_FIELD.as_str(),
                value: name,
            });
        }
        fields.push(ObservedField { name, kind });
    }
    Ok(fields)
}

fn find_field<'a>(fields: &'a [ObservedField], name: &str) -> Option<&'a ObservedField> {
    fields.iter().find(|field| field.name.as_str() == name)
}
