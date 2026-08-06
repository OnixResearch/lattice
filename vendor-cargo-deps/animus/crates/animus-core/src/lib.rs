//! Pure deterministic core for `animus`.
//!
//! This crate is `#![no_std]` and uses `alloc` for owned strings. Keep runtime
//! inputs, environment handling, logging, async work, and I/O in `animus-std`.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]

extern crate alloc;

const SUMMARY_PREFIX: &str = "intent: ";

/// Smallest accepted intent length after normalization.
pub const MIN_INTENT_CHARS: usize = 1;

/// Largest accepted intent length after normalization.
pub const MAX_INTENT_CHARS: usize = 280;

/// A validated, normalized statement of intent.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Intent {
    value: alloc::string::String,
}

impl Intent {
    /// Builds a normalized intent from raw text.
    ///
    /// Leading and trailing Unicode whitespace are removed before validation.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the normalized text is empty, too long, or
    /// contains a Unicode control character.
    pub fn new(raw: &str) -> Result<Self, CoreError> {
        let value = normalize_intent(raw);
        validate_intent(value)?;
        Ok(Self {
            value: alloc::string::String::from(value),
        })
    }

    /// Returns the normalized intent as a borrowed string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }

    /// Consumes the intent and returns the owned normalized string.
    #[must_use]
    pub fn into_string(self) -> alloc::string::String {
        self.value
    }
}

/// Deterministic core state owned by the functional core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnimusState {
    primary_intent: Intent,
}

impl AnimusState {
    /// Builds core state from a validated intent.
    #[must_use]
    pub const fn new(primary_intent: Intent) -> Self {
        Self { primary_intent }
    }

    /// Returns the primary intent tracked by this state.
    #[must_use]
    pub const fn primary_intent(&self) -> &Intent {
        &self.primary_intent
    }

    /// Renders a deterministic summary of the core state.
    #[must_use]
    pub fn summary(&self) -> alloc::string::String {
        let mut summary = alloc::string::String::from(SUMMARY_PREFIX);
        summary.push_str(self.primary_intent.as_str());
        summary
    }
}

/// Errors produced by pure core validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreError {
    /// The normalized intent contained no user-visible characters.
    EmptyIntent,
    /// The normalized intent exceeded [`MAX_INTENT_CHARS`].
    IntentTooLong {
        /// Maximum accepted character count.
        max_chars: usize,
        /// Actual normalized character count.
        actual_chars: usize,
    },
    /// The normalized intent contained a Unicode control character.
    ControlCharacter {
        /// Byte index where the control character starts.
        byte_index: usize,
    },
}

impl core::fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyIntent => formatter.write_str("intent must not be empty"),
            Self::IntentTooLong {
                max_chars,
                actual_chars,
            } => write!(formatter, "intent must be at most {max_chars} characters; got {actual_chars}"),
            Self::ControlCharacter { byte_index } => {
                write!(formatter, "intent contains a control character at byte index {byte_index}")
            }
        }
    }
}

fn normalize_intent(raw: &str) -> &str {
    raw.trim()
}

fn validate_intent(value: &str) -> Result<(), CoreError> {
    let actual_chars = value.chars().count();

    if actual_chars < MIN_INTENT_CHARS {
        return Err(CoreError::EmptyIntent);
    }

    if actual_chars > MAX_INTENT_CHARS {
        return Err(CoreError::IntentTooLong {
            max_chars: MAX_INTENT_CHARS,
            actual_chars,
        });
    }

    if let Some((byte_index, _character)) = value.char_indices().find(|(_byte_index, character)| character.is_control())
    {
        return Err(CoreError::ControlCharacter { byte_index });
    }

    Ok(())
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_INTENT: &str = "build reliable systems";
    const TRIMMED_VALID_INTENT: &str = "build reliable systems";
    const PADDED_VALID_INTENT: &str = "  build reliable systems  ";
    const CONTROL_CHARACTER_INTENT: &str = "build\nreliable systems";
    const CONTROL_CHARACTER_BYTE_INDEX: usize = 5;
    const SUMMARY: &str = "intent: build reliable systems";

    #[test]
    fn intent_accepts_and_normalizes_valid_text() {
        let intent = Intent::new(PADDED_VALID_INTENT).expect("valid intent should parse");

        assert_eq!(intent.as_str(), TRIMMED_VALID_INTENT);
    }

    #[test]
    fn intent_rejects_empty_text() {
        let error = Intent::new("   ").expect_err("blank intent should fail");

        assert_eq!(error, CoreError::EmptyIntent);
    }

    #[test]
    fn intent_rejects_overlong_text() {
        let too_long_intent_chars = MAX_INTENT_CHARS.saturating_add(MIN_INTENT_CHARS);
        let raw = "a".repeat(too_long_intent_chars);
        let error = Intent::new(&raw).expect_err("overlong intent should fail");

        assert_eq!(error, CoreError::IntentTooLong {
            max_chars: MAX_INTENT_CHARS,
            actual_chars: too_long_intent_chars,
        });
    }

    #[test]
    fn intent_rejects_control_characters() {
        let error = Intent::new(CONTROL_CHARACTER_INTENT).expect_err("control characters should fail validation");

        assert_eq!(error, CoreError::ControlCharacter {
            byte_index: CONTROL_CHARACTER_BYTE_INDEX,
        });
    }

    #[test]
    fn state_summary_is_deterministic() {
        let intent = Intent::new(VALID_INTENT).expect("valid intent should parse");
        let state = AnimusState::new(intent);

        assert_eq!(state.summary(), SUMMARY);
    }

    #[test]
    fn no_std_core_uses_alloc_owned_state() {
        let intent = Intent::new(VALID_INTENT).expect("valid intent should parse");
        let owned = intent.into_string();

        assert_eq!(owned, VALID_INTENT);
    }
}
