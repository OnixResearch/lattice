semantic_string_id! {
    /// Canonical stable identity of caller-owned docking panel content.
    ///
    /// Identity namespaces do not implicitly interchange.
    ///
    /// ```compile_fail
    /// use ratcore::docking::{DockNodeId, PanelId};
    /// let panel = PanelId::new("editor");
    /// let node: DockNodeId = panel;
    /// ```
    pub struct PanelId;
}

semantic_string_id! {
    /// Canonical stable identity of one flexible docking tree node.
    pub struct DockNodeId;
}

semantic_string_id! {
    /// Stable legacy fixed-zone name used only at migration boundaries.
    ///
    /// ```compile_fail
    /// use ratcore::docking::{DockNodeId, LegacyZoneName};
    /// let legacy = LegacyZoneName::new("left");
    /// let node: DockNodeId = legacy;
    /// ```
    pub struct LegacyZoneName;
}

impl DockNodeId {
    /// Creates a generated node identity from an explicit deterministic value.
    #[must_use]
    pub fn generated(value: impl Into<String>) -> Self {
        Self::new(value)
    }

    /// Creates a node identity through the named legacy-zone migration boundary.
    #[must_use]
    pub fn from_legacy_name(name: &LegacyZoneName) -> Self {
        Self::new(name.as_str())
    }
}

/// Opaque public panel/tab insertion position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DockInsertionIndex(u64);

impl DockInsertionIndex {
    /// Creates a cross-platform insertion position.
    #[must_use]
    pub const fn new(index: u64) -> Self {
        Self(index)
    }

    /// Returns the cross-platform integer representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns an index clamped to the insertion range `0..=len`.
    #[must_use]
    pub fn clamped_for_len(self, len: u64) -> u64 {
        self.0.min(len)
    }

    /// Returns an index when it is inside the insertion range `0..=len`.
    #[must_use]
    pub fn checked_for_len(self, len: u64) -> Option<u64> {
        (self.0 <= len).then_some(self.0)
    }
}

impl From<u32> for DockInsertionIndex {
    fn from(value: u32) -> Self {
        Self(u64::from(value))
    }
}

impl From<u64> for DockInsertionIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<u128> for DockInsertionIndex {
    type Error = super::DockingError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        u64::try_from(value).map(Self).map_err(|_| super::DockingError::InvalidInsertionIndex(value))
    }
}
