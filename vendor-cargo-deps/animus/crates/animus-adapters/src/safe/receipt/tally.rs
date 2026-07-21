/// Borrowed bounded-count input for a safe receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedCountDraft<'a> {
    pub(super) label: &'a str,
    pub(super) value: u32,
}

impl<'a> BoundedCountDraft<'a> {
    /// Builds a bounded-count draft.
    #[must_use]
    pub const fn new(label: &'a str, value: u32) -> Self {
        Self { label, value }
    }
}

/// Bounded count retained by a safe receipt.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BoundedCount {
    label: alloc::string::String,
    value: u32,
}

impl BoundedCount {
    pub(super) fn new(label: alloc::string::String, value: u32) -> Self {
        Self { label, value }
    }

    /// Returns the safe count label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the bounded count value.
    #[must_use]
    pub const fn value(&self) -> u32 {
        self.value
    }
}
