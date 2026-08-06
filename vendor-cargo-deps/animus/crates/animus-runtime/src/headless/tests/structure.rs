use super::super::*;
use super::support::*;

#[test]
fn structured_output_validation_accepts_required_fields_and_reports_bad_outputs() {
    let schema = StructuredSchemaDraft {
        schema_id: SCHEMA_ID,
        required_fields: &[REQUIRED_FIELD],
        allowed_fields: &[REQUIRED_FIELD],
        allow_extra_fields: false,
    };
    let valid = validate_structured_output(&schema, &StructuredOutputDraft {
        fields: &[REQUIRED_FIELD],
    })
    .expect("valid output should report success");
    let missing = validate_structured_output(&schema, &StructuredOutputDraft { fields: &[] })
        .expect("missing required field should report failure");
    let extra = validate_structured_output(&schema, &StructuredOutputDraft {
        fields: &[REQUIRED_FIELD, EXTRA_FIELD],
    })
    .expect("extra field should report failure");
    let malformed = validate_structured_output(
        &StructuredSchemaDraft {
            required_fields: &[REQUIRED_FIELD, REQUIRED_FIELD],
            ..schema
        },
        &StructuredOutputDraft {
            fields: &[REQUIRED_FIELD],
        },
    )
    .expect_err("duplicate schema fields should fail validation");

    assert_eq!(valid.schema_id(), SCHEMA_ID);
    assert_eq!(valid.status(), StructuredOutputStatus::Success);
    assert_eq!(valid.failure(), None);
    assert_eq!(missing.status(), StructuredOutputStatus::Failure);
    assert_eq!(
        missing.failure(),
        Some(&StructuredOutputFailure::MissingField {
            field: Text::from(REQUIRED_FIELD),
        })
    );
    assert_eq!(extra.status(), StructuredOutputStatus::Failure);
    assert_eq!(
        extra.failure(),
        Some(&StructuredOutputFailure::ExtraField {
            field: Text::from(EXTRA_FIELD),
        })
    );
    assert_eq!(malformed, AutomationError::DuplicateValue {
        field: STRUCTURED_FIELD,
        value: Text::from(REQUIRED_FIELD),
    });
}
