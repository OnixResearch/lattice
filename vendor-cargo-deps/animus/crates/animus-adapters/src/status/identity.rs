const SESSION_REFERENCE_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("session_reference_kind");
const SESSION_REFERENCE_VALUE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("session_reference_value");

const SESSION_REFERENCE_KIND_NATIVE: &str = "native";

/// Optional native host session reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reference {
    kind: Kind,
    value: alloc::string::String,
}

impl Reference {
    /// Builds a validated session reference.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the kind or value is invalid
    /// or the kind is unsupported.
    pub fn new(draft: ReferenceDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        crate::validation::validate_required_adapter_field(SESSION_REFERENCE_KIND_FIELD, draft.kind)?;
        crate::validation::validate_required_adapter_field(SESSION_REFERENCE_VALUE_FIELD, draft.value)?;
        let kind = Kind::parse(draft.kind)?;
        Ok(Self {
            kind,
            value: alloc::string::String::from(draft.value),
        })
    }

    /// Returns the supported session reference kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Returns the native session reference value.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// Borrowed session-reference input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceDraft<'a> {
    /// Session reference kind string.
    pub kind: &'a str,
    /// Session reference value.
    pub value: &'a str,
}

/// Supported session reference kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// Host-native session identifier supplied by an integration.
    Native,
}

impl Kind {
    /// Parses a session-reference kind string.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the kind is not supported.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        match value.trim() {
            SESSION_REFERENCE_KIND_NATIVE => Ok(Self::Native),
            other => Err(crate::AdapterContractError::UnsupportedSessionReferenceKind {
                field: SESSION_REFERENCE_KIND_FIELD.as_str(),
                value: alloc::string::String::from(other),
            }),
        }
    }

    /// Returns the stable lowercase session-reference-kind spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Native => SESSION_REFERENCE_KIND_NATIVE,
        }
    }
}
