mod branch;

pub use branch::replay_snapshot;

const LABEL_TEXT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("ledger_label_text");

/// One append-only ledger entry with an optional parent link.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    entry: super::Entry,
    parent_id: Option<super::EntryId>,
}

impl Node {
    /// Builds a root node with no parent link.
    #[must_use]
    pub const fn root(entry: super::Entry) -> Self {
        Self {
            entry,
            parent_id: None,
        }
    }

    /// Builds a child node linked to a parent entry identifier.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the parent identifier is
    /// empty, overlong, or contains a NUL character.
    pub fn child(
        entry: super::Entry,
        parent_id: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            entry,
            parent_id: Some(super::EntryId::new(parent_id)?),
        })
    }

    /// Returns the wrapped neutral ledger entry.
    #[must_use]
    pub const fn entry(&self) -> &super::Entry {
        &self.entry
    }

    /// Returns the optional parent entry identifier.
    #[must_use]
    pub const fn parent_id(&self) -> Option<&super::EntryId> {
        self.parent_id.as_ref()
    }
}

/// Operator-facing label attached to one ledger entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    target_id: super::EntryId,
    text: alloc::string::String,
}

impl Label {
    /// Builds a label attached to a target entry identifier.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the target identifier or
    /// label text is empty, overlong, or contains a NUL character.
    pub fn new(target_id: &str, text: &str) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            target_id: super::EntryId::new(target_id)?,
            text: crate::validation::normalized_adapter_field(LABEL_TEXT_FIELD, text)?,
        })
    }

    /// Returns the entry identifier this label annotates.
    #[must_use]
    pub const fn target_id(&self) -> &super::EntryId {
        &self.target_id
    }

    /// Returns the normalized label text.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}

/// In-memory session tree snapshot selected by an active leaf.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    entries: alloc::vec::Vec<Node>,
    active_leaf: super::EntryId,
    labels: alloc::vec::Vec<Label>,
}

impl Snapshot {
    /// Builds a snapshot with caller-owned entries, active leaf, and labels.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the active leaf identifier
    /// is empty, overlong, or contains a NUL character.
    pub fn new(
        entries: alloc::vec::Vec<Node>,
        active_leaf: &str,
        labels: alloc::vec::Vec<Label>,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            entries,
            active_leaf: super::EntryId::new(active_leaf)?,
            labels,
        })
    }

    /// Returns append-only ledger nodes in storage order.
    #[must_use]
    pub fn entries(&self) -> &[Node] {
        self.entries.as_slice()
    }

    /// Returns the selected active leaf identifier.
    #[must_use]
    pub const fn active_leaf(&self) -> &super::EntryId {
        &self.active_leaf
    }

    /// Returns metadata labels attached to entries.
    #[must_use]
    pub fn labels(&self) -> &[Label] {
        self.labels.as_slice()
    }

    /// Iterates labels attached to one entry.
    pub fn labels_for<'a>(
        &'a self,
        target_id: &'a super::EntryId,
    ) -> impl Iterator<Item = &'a Label> + 'a {
        self.labels
            .iter()
            .filter(move |label| label.target_id() == target_id)
    }
}
