const HASH_LENGTH_REASON: &str = "checkpoint hash must use the configured BLAKE3 hex length";
const HASH_NOT_HEX_REASON: &str = "checkpoint hash must contain lowercase hex only";
const TOO_MANY_CHANGED_FILES_REASON: &str = "changed file count exceeds the safe receipt bound";

/// Checkpoint-specific validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckpointError {
    /// A shared adapter field contract failed.
    AdapterContract(crate::AdapterContractError),
    /// A field used an unsupported enum label.
    UnsupportedValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: alloc::string::String,
    },
    /// A changed-file count exceeded the safe bounded count.
    ChangedFileCountTooLarge {
        /// Maximum accepted changed-file count.
        max_count: u32,
        /// Actual changed-file count.
        actual_count: u32,
    },
}

impl From<crate::AdapterContractError> for CheckpointError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::AdapterContract(error)
    }
}

impl core::fmt::Display for CheckpointError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AdapterContract(error) => write!(formatter, "{error}"),
            Self::UnsupportedValue { field, value } => {
                write!(formatter, "{field} contains unsupported checkpoint value {value}")
            }
            Self::ChangedFileCountTooLarge {
                max_count,
                actual_count,
            } => write!(formatter, "{TOO_MANY_CHANGED_FILES_REASON}: maximum {max_count}, got {actual_count}"),
        }
    }
}

/// Validated lowercase BLAKE3-style checkpoint hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointHash {
    value: alloc::string::String,
}

impl CheckpointHash {
    /// Builds a validated checkpoint hash.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointError`] when the field is empty, too long, contains a
    /// NUL character, has the wrong hash length, or contains non-lowercase-hex
    /// characters.
    pub fn new(value: &str) -> Result<Self, CheckpointError> {
        Self::new_for_field(super::CHECKPOINT_RECEIPT_HASH_FIELD, value)
    }

    pub(super) fn new_for_field(
        field_name: crate::validation::FieldName,
        value: &str,
    ) -> Result<Self, CheckpointError> {
        validate_hash(field_name, value)?;
        Ok(Self {
            value: alloc::string::String::from(value),
        })
    }

    /// Builds a checkpoint hash from raw bytes using BLAKE3.
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

pub(super) fn parse_enum<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, CheckpointError> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or(CheckpointError::UnsupportedValue {
        field: field_name.as_str(),
        value: normalized,
    })
}

pub(super) fn required_string(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<alloc::string::String, CheckpointError> {
    Ok(crate::validation::normalized_adapter_field(field_name, value)?)
}

pub(super) fn optional_string(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, CheckpointError> {
    value.map_or(Ok(None), |text| required_string(field_name, text).map(Some))
}

pub(super) fn optional_hash(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<CheckpointHash>, CheckpointError> {
    value.map_or(Ok(None), |text| CheckpointHash::new_for_field(field_name, text).map(Some))
}

pub(super) fn validate_changed_file_count(value: u32) -> Result<(), CheckpointError> {
    if value > super::MAX_CHANGED_FILE_COUNT {
        return Err(CheckpointError::ChangedFileCountTooLarge {
            max_count: super::MAX_CHANGED_FILE_COUNT,
            actual_count: value,
        });
    }
    Ok(())
}

fn validate_hash(field_name: crate::validation::FieldName, value: &str) -> Result<(), CheckpointError> {
    crate::validation::validate_required_adapter_field(field_name, value)?;
    let actual_chars = value.chars().count();
    assert!(actual_chars > 0, "validated hash cannot be empty");
    assert!(actual_chars <= value.len(), "hash char count must fit in byte length");
    if actual_chars != super::HASH_HEX_CHARS {
        return Err(crate::AdapterContractError::InvalidEvidenceHash {
            field: field_name.as_str(),
            reason: HASH_LENGTH_REASON,
        }
        .into());
    }
    if !value.chars().all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase()) {
        return Err(crate::AdapterContractError::InvalidEvidenceHash {
            field: field_name.as_str(),
            reason: HASH_NOT_HEX_REASON,
        }
        .into());
    }
    Ok(())
}
