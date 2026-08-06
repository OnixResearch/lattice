const HASH_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("effect_receipt_hash");
const HASH_LENGTH_REASON: &str = "effect hash must use the configured BLAKE3 hex length";
const HASH_NOT_HEX_REASON: &str = "effect hash must contain lowercase hex only";
const EMPTY_PAYLOAD_COUNT: u32 = 0;

/// Character length of a lowercase BLAKE3 receipt hash in hex form.
pub const SAFE_HASH_HEX_CHARS: usize = 64;
/// Stable contract identity for safe receipt DTOs.
pub const SAFE_RECEIPT_CONTRACT_IDENTITY: &str = "animus.safe-receipt.v1";

const _: () = assert!(SAFE_HASH_HEX_CHARS > 0);

/// Validated lowercase BLAKE3 hash used by safe receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeHash {
    value: alloc::string::String,
}

impl SafeHash {
    /// Builds a validated receipt hash.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the hash is empty, has the
    /// wrong length, contains uppercase hex, or contains non-hex characters.
    pub fn new(value: &str) -> Result<Self, crate::AdapterContractError> {
        validate_hash(value)?;
        Ok(Self {
            value: alloc::string::String::from(value),
        })
    }

    /// Builds a safe hash directly from raw bytes with BLAKE3.
    #[must_use]
    pub fn from_blake3_bytes(bytes: &[u8]) -> Self {
        Self {
            value: alloc::format!("{}", blake3::hash(bytes).to_hex()),
        }
    }

    /// Returns the validated hash text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// Draft used to build a redacted receipt without raw payload fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptDraft {
    service_kind: super::ServiceKind,
    capability: super::CapabilityClass,
    outcome: super::OutcomeKind,
    reason: super::Reason,
    input_hash: Option<SafeHash>,
    output_hash: Option<SafeHash>,
    payload_count: u32,
    redaction: super::RedactionMarker,
}

impl ReceiptDraft {
    /// Builds a receipt draft with payload capture disabled by default.
    #[must_use]
    pub const fn new(
        service_kind: super::ServiceKind,
        capability: super::CapabilityClass,
        outcome: super::OutcomeKind,
        reason: super::Reason,
    ) -> Self {
        Self {
            service_kind,
            capability,
            outcome,
            reason,
            input_hash: None,
            output_hash: None,
            payload_count: EMPTY_PAYLOAD_COUNT,
            redaction: super::RedactionMarker::PayloadNotCaptured,
        }
    }

    /// Adds a safe input hash to the receipt draft.
    #[must_use]
    pub fn with_input_hash(mut self, input_hash: SafeHash) -> Self {
        self.input_hash = Some(input_hash);
        self
    }

    /// Adds a safe output hash to the receipt draft.
    #[must_use]
    pub fn with_output_hash(mut self, output_hash: SafeHash) -> Self {
        self.output_hash = Some(output_hash);
        self
    }

    /// Adds a bounded payload count to the receipt draft.
    #[must_use]
    pub const fn with_payload_count(mut self, payload_count: u32) -> Self {
        self.payload_count = payload_count;
        self
    }

    /// Sets the redaction marker for the receipt draft.
    #[must_use]
    pub const fn with_redaction(mut self, redaction: super::RedactionMarker) -> Self {
        self.redaction = redaction;
        self
    }
}

/// Redacted receipt for one injected-service decision or effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    service_kind: super::ServiceKind,
    capability: super::CapabilityClass,
    outcome: super::OutcomeKind,
    reason: super::Reason,
    input_hash: Option<SafeHash>,
    output_hash: Option<SafeHash>,
    payload_count: u32,
    redaction: super::RedactionMarker,
}

impl Receipt {
    /// Builds a redacted receipt from a typed safe draft.
    #[must_use]
    pub fn new(draft: ReceiptDraft) -> Self {
        Self {
            service_kind: draft.service_kind,
            capability: draft.capability,
            outcome: draft.outcome,
            reason: draft.reason,
            input_hash: draft.input_hash,
            output_hash: draft.output_hash,
            payload_count: draft.payload_count,
            redaction: draft.redaction,
        }
    }

    /// Returns the service kind.
    #[must_use]
    pub const fn service_kind(&self) -> super::ServiceKind {
        self.service_kind
    }

    /// Returns the capability class.
    #[must_use]
    pub const fn capability(&self) -> super::CapabilityClass {
        self.capability
    }

    /// Returns the outcome class.
    #[must_use]
    pub const fn outcome(&self) -> super::OutcomeKind {
        self.outcome
    }

    /// Returns the reason class.
    #[must_use]
    pub const fn reason(&self) -> super::Reason {
        self.reason
    }

    /// Returns the optional safe input hash.
    #[must_use]
    pub const fn input_hash(&self) -> Option<&SafeHash> {
        self.input_hash.as_ref()
    }

    /// Returns the optional safe output hash.
    #[must_use]
    pub const fn output_hash(&self) -> Option<&SafeHash> {
        self.output_hash.as_ref()
    }

    /// Returns the bounded payload count captured by this receipt.
    #[must_use]
    pub const fn payload_count(&self) -> u32 {
        self.payload_count
    }

    /// Returns the redaction marker.
    #[must_use]
    pub const fn redaction(&self) -> super::RedactionMarker {
        self.redaction
    }

    /// Returns the stable safe receipt contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        SAFE_RECEIPT_CONTRACT_IDENTITY
    }
}

fn validate_hash(value: &str) -> Result<(), crate::AdapterContractError> {
    crate::validation::validate_required_adapter_field(HASH_FIELD, value)?;
    let actual_chars = value.chars().count();
    assert!(actual_chars > 0, "validated hash cannot be empty");
    assert!(actual_chars <= value.len(), "hash char count must fit in byte length");
    if actual_chars != SAFE_HASH_HEX_CHARS {
        return Err(crate::AdapterContractError::InvalidEvidenceHash {
            field: HASH_FIELD.as_str(),
            reason: HASH_LENGTH_REASON,
        });
    }
    if !value.chars().all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase()) {
        return Err(crate::AdapterContractError::InvalidEvidenceHash {
            field: HASH_FIELD.as_str(),
            reason: HASH_NOT_HEX_REASON,
        });
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod test_support {
    pub(crate) const HASH_FIELD: crate::validation::FieldName = super::HASH_FIELD;
    pub(crate) const HASH_LENGTH_REASON: &str = super::HASH_LENGTH_REASON;
    pub(crate) const HASH_NOT_HEX_REASON: &str = super::HASH_NOT_HEX_REASON;
}
