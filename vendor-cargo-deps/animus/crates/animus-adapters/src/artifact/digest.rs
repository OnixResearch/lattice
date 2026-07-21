const FIELD: &str = "artifact_hash";
const LENGTH_REASON: &str = "artifact hash must use the configured BLAKE3 hex length";
const NOT_HEX_REASON: &str = "artifact hash must contain lowercase hex only";
const BLAKE3_HASH_BYTES: usize = 32;
const HEX_CHARS_PER_BYTE: usize = 2;
const HEX_RADIX: usize = 16;
const HEX_NIBBLE_SHIFT: u8 = 4;
const HEX_LOW_NIBBLE_MASK: u8 = 0x0f;
const LOWER_HEX: &[u8; HEX_RADIX] = b"0123456789abcdef";

/// Character length of a lowercase BLAKE3 artifact hash in hex form.
pub const BLAKE3_HASH_HEX_CHARS: usize = BLAKE3_HASH_BYTES.saturating_mul(HEX_CHARS_PER_BYTE);

/// Validated lowercase BLAKE3 identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hash {
    value: alloc::string::String,
}

impl Hash {
    /// Builds a hash from validated lowercase BLAKE3 hex text.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the hash is empty, has the wrong length,
    /// contains uppercase hex, or contains non-hex characters.
    pub fn new(value: &str) -> Result<Self, super::Error> {
        validate(value)?;
        Ok(Self {
            value: alloc::string::String::from(value),
        })
    }

    /// Builds a hash from raw BLAKE3 bytes.
    ///
    /// # Panics
    ///
    /// Panics if the internal lowercase hex encoder violates its fixed BLAKE3
    /// length or nibble-table invariants.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; BLAKE3_HASH_BYTES]) -> Self {
        let mut value = alloc::string::String::with_capacity(BLAKE3_HASH_HEX_CHARS);
        for byte in bytes {
            let high = usize::from(byte >> HEX_NIBBLE_SHIFT);
            let low = usize::from(byte & HEX_LOW_NIBBLE_MASK);
            assert!(high < HEX_RADIX, "high hex nibble must fit hex table");
            assert!(low < HEX_RADIX, "low hex nibble must fit hex table");
            value.push(char::from(LOWER_HEX[high]));
            value.push(char::from(LOWER_HEX[low]));
        }
        assert_eq!(
            value.len(),
            BLAKE3_HASH_HEX_CHARS,
            "BLAKE3 hex encoding must produce the configured length"
        );
        Self { value }
    }

    /// Returns the validated hash text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

impl core::fmt::Display for Hash {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl core::str::FromStr for Hash {
    type Err = super::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn validate(value: &str) -> Result<(), super::Error> {
    let actual_chars = value.chars().count();
    assert!(
        actual_chars <= value.len(),
        "hash char count must fit byte length"
    );
    validate_length(actual_chars)?;
    validate_alphabet(value)
}

fn validate_length(actual_chars: usize) -> Result<(), super::Error> {
    if actual_chars != BLAKE3_HASH_HEX_CHARS {
        return Err(super::Error::InvalidHash {
            field: FIELD,
            reason: LENGTH_REASON,
        });
    }
    Ok(())
}

fn validate_alphabet(value: &str) -> Result<(), super::Error> {
    if !has_lowercase_hex_only(value) {
        return Err(super::Error::InvalidHash {
            field: FIELD,
            reason: NOT_HEX_REASON,
        });
    }
    Ok(())
}

fn has_lowercase_hex_only(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}

#[cfg(test)]
pub(super) mod test_support {
    pub(in crate::artifact) const FIELD: &str = super::FIELD;
    pub(in crate::artifact) const NOT_HEX_REASON: &str = super::NOT_HEX_REASON;
}
