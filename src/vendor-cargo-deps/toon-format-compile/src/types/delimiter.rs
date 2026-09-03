use std::fmt;

use serde::{
    Deserialize,
    Serialize,
};

/// Delimiter character used to separate array elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Delimiter {
    #[default]
    Comma,
    Tab,
    Pipe,
}

impl Delimiter {
    /// Get the character representation of this delimiter.
    pub fn as_char(&self) -> char {
        match self {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Pipe => '|',
        }
    }

    /// Get the string representation for metadata (empty for comma, char for
    /// others).
    pub fn as_metadata_str(&self) -> &'static str {
        match self {
            Delimiter::Comma => "",
            Delimiter::Tab => "\t",
            Delimiter::Pipe => "|",
        }
    }

    /// Parse a delimiter from a character.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            ',' => Some(Delimiter::Comma),
            '\t' => Some(Delimiter::Tab),
            '|' => Some(Delimiter::Pipe),
            _ => None,
        }
    }

    /// Check if the delimiter character appears in the string.
    pub fn contains_in(&self, s: &str) -> bool {
        s.contains(self.as_char())
    }
}

impl fmt::Display for Delimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}
