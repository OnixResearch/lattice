/// Monotonic event sequence value.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EventSequence {
    value: u64,
}

impl EventSequence {
    /// Validates a non-zero event sequence.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::AutomationError`] when the sequence is below the
    /// minimum.
    pub const fn new(value: u64) -> Result<Self, super::super::AutomationError> {
        if value < super::super::MIN_EVENT_SEQUENCE {
            return Err(super::super::AutomationError::InvalidSequence {
                field: super::super::EVENT_SEQUENCE_FIELD,
            });
        }
        Ok(Self { value })
    }

    /// Returns the raw sequence value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }
}
