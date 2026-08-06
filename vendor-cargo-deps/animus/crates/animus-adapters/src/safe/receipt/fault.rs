/// Errors raised by safe receipt validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required collection was empty.
    EmptyCollection {
        /// Boundary collection field name.
        field: &'static str,
    },
    /// A collection exceeded the safe receipt item limit.
    CollectionTooLarge {
        /// Boundary collection field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A safe metadata field was empty after trimming.
    EmptyField {
        /// Boundary field name.
        field: &'static str,
    },
    /// A safe metadata field contained a NUL character.
    NulCharacter {
        /// Boundary field name.
        field: &'static str,
        /// Byte index where the NUL character starts.
        byte_index: usize,
    },
    /// Unsafe material was detected in safe metadata.
    UnsafeMetadata {
        /// Boundary field name.
        field: &'static str,
        /// Detected unsafe material class.
        class: super::LeakClass,
    },
    /// A bounded count exceeded `MAX_COUNT`.
    CountTooLarge {
        /// Boundary field name.
        field: &'static str,
        /// Maximum accepted count.
        max: u32,
        /// Actual count.
        actual: u32,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyCollection { field } | Self::EmptyField { field } => {
                write!(formatter, "{field} must not be empty")
            }
            Self::CollectionTooLarge {
                field,
                max_items,
                actual_items,
            } => write!(formatter, "{field} must contain at most {max_items} items; got {actual_items}"),
            Self::NulCharacter { field, byte_index } => {
                write!(formatter, "{field} contains a NUL character at byte index {byte_index}")
            }
            Self::UnsafeMetadata { field, class } => {
                write!(formatter, "{field} contains unsafe {} material", class.as_str())
            }
            Self::CountTooLarge { field, max, actual } => {
                write!(formatter, "{field} must be at most {max}; got {actual}")
            }
        }
    }
}
