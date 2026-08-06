/// Closed value-kind vocabulary for contextual facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionContextFactValueKind {
    /// Boolean state.
    Boolean,
    /// Signed integer state.
    Integer,
    /// Bounded semantic text.
    Text,
}

/// A pure typed contextual fact value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionContextFactValue {
    /// Boolean state.
    Boolean(bool),
    /// Signed integer state.
    Integer(i64),
    /// Bounded semantic text.
    Text(String),
}

impl ActionContextFactValue {
    /// Returns the closed value kind represented by this value.
    #[must_use]
    pub const fn kind(&self) -> ActionContextFactValueKind {
        match self {
            Self::Boolean(_) => ActionContextFactValueKind::Boolean,
            Self::Integer(_) => ActionContextFactValueKind::Integer,
            Self::Text(_) => ActionContextFactValueKind::Text,
        }
    }
}

impl From<bool> for ActionContextFactValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i64> for ActionContextFactValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<String> for ActionContextFactValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for ActionContextFactValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

/// Schema entry for a typed contextual fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionContextFactDefinition {
    /// Stable fact identity.
    pub id: super::super::ActionContextFactId,
    /// Required value kind.
    pub value_kind: ActionContextFactValueKind,
}

impl ActionContextFactDefinition {
    /// Creates a typed fact schema entry.
    #[must_use]
    pub fn new(id: impl Into<super::super::ActionContextFactId>, value_kind: ActionContextFactValueKind) -> Self {
        Self {
            id: id.into(),
            value_kind,
        }
    }
}
