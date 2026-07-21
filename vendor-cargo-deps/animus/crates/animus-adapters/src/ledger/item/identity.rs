const ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("ledger_entry_id");

/// Validated neutral ledger entry identifier.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntryId {
    value: alloc::string::String,
}

impl EntryId {
    /// Builds a validated entry identifier.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the identifier is empty,
    /// overlong, or contains a NUL character.
    pub fn new(value: &str) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            value: crate::validation::normalized_adapter_field(ID_FIELD, value)?,
        })
    }

    /// Returns the validated identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}
