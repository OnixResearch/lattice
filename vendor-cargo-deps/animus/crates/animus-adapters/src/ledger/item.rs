mod identity;
mod record;
mod taxonomy;

pub use identity::EntryId;
pub use record::Entry;
pub use taxonomy::EntryKind;
pub use taxonomy::SummaryKind;

const TEXT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("ledger_text");
const LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("ledger_label");

fn normalized_text(value: &str) -> Result<alloc::string::String, crate::AdapterContractError> {
    crate::validation::normalized_adapter_field(TEXT_FIELD, value)
}

fn normalized_label(value: &str) -> Result<alloc::string::String, crate::AdapterContractError> {
    crate::validation::normalized_adapter_field(LABEL_FIELD, value)
}
