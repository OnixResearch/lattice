mod kind;
mod record;
mod redaction;
mod status;

/// Bounded assertion classes published through an optional dataspace adapter.
pub type Kind = kind::Value;
/// Bounded redacted assertion projected to an optional dataspace backend.
pub type Record = record::Entry;
/// Borrowed input used to validate a bounded dataspace assertion.
pub type Draft<'a> = record::Draft<'a>;
/// Redaction marker proving raw sensitive material was not asserted.
pub type Redaction = redaction::Value;
/// Safe lifecycle status class for a dataspace assertion.
pub type Status = status::Value;

pub(super) fn parse_kind_for_field(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<Kind, crate::AdapterContractError> {
    kind::Value::parse_for_field(field_name, value)
}

fn parse_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, crate::AdapterContractError> {
    super::parse_value(field_name, value, parser)
}
