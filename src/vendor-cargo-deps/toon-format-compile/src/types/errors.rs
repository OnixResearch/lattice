use thiserror::Error;

/// Result type alias for TOON operations.
pub type ToonResult<T> = std::result::Result<T, ToonError>;

/// Errors that can occur during TOON encoding or decoding.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ToonError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
        #[source]
        context: Option<Box<ErrorContext>>,
    },

    #[error("Invalid character '{char}' at position {position}")]
    InvalidCharacter { char: char, position: usize },

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Invalid delimiter: {0}")]
    InvalidDelimiter(String),

    #[error("Array length mismatch: expected {expected}, found {found}")]
    LengthMismatch {
        expected: usize,
        found: usize,
        #[source]
        context: Option<Box<ErrorContext>>,
    },

    #[error("Invalid structure: {0}")]
    InvalidStructure(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// Contextual information for error reporting, including source location
/// and suggestions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorContext {
    pub source_line: String,
    pub preceding_lines: Vec<String>,
    pub following_lines: Vec<String>,
    pub suggestion: Option<String>,
    pub indicator: Option<String>,
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\nContext:")?;

        for line in &self.preceding_lines {
            writeln!(f, "  {line}")?;
        }

        writeln!(f, "> {}", self.source_line)?;

        if let Some(indicator) = &self.indicator {
            writeln!(f, "  {indicator}")?;
        }

        for line in &self.following_lines {
            writeln!(f, "  {line}")?;
        }

        if let Some(suggestion) = &self.suggestion {
            writeln!(f, "\nSuggestion: {suggestion}")?;
        }

        Ok(())
    }
}

impl std::error::Error for ErrorContext {}

impl ErrorContext {
    /// Create a new error context with a source line.
    pub fn new(source_line: impl Into<String>) -> Self {
        Self {
            source_line: source_line.into(),
            preceding_lines: Vec::new(),
            following_lines: Vec::new(),
            suggestion: None,
            indicator: None,
        }
    }

    /// Add preceding context lines.
    pub fn with_preceding_lines(mut self, lines: Vec<String>) -> Self {
        self.preceding_lines = lines;
        self
    }

    /// Add following context lines.
    pub fn with_following_lines(mut self, lines: Vec<String>) -> Self {
        self.following_lines = lines;
        self
    }

    /// Add a suggestion message to help fix the error.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add a column indicator (caret) pointing to the error position.
    pub fn with_indicator(mut self, column: usize) -> Self {
        let indicator = format!("{}^", " ".repeat(column));
        self.indicator = Some(indicator);
        self
    }

    /// Create error context from input string with automatic context
    /// extraction.
    pub fn from_input(
        input: &str,
        line: usize,
        column: usize,
        context_lines: usize,
    ) -> Option<Self> {
        let lines: Vec<&str> = input.lines().collect();

        if line == 0 || line > lines.len() {
            return None;
        }

        let line_idx = line - 1;
        let source_line = lines.get(line_idx)?.to_string();

        let start_line = line_idx.saturating_sub(context_lines);
        let end_line = (line_idx + context_lines + 1).min(lines.len());

        let preceding_lines = lines[start_line..line_idx]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let following_lines = lines[(line_idx + 1)..end_line]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Some(Self {
            source_line,
            preceding_lines,
            following_lines,
            suggestion: None,
            indicator: Some(format!("{}^", " ".repeat(column.saturating_sub(1)))),
        })
    }
}

impl ToonError {
    /// Create a parse error at the given position.
    pub fn parse_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        ToonError::ParseError {
            line,
            column,
            message: message.into(),
            context: None,
        }
    }

    /// Create a parse error with additional context information.
    pub fn parse_error_with_context(
        line: usize,
        column: usize,
        message: impl Into<String>,
        context: ErrorContext,
    ) -> Self {
        ToonError::ParseError {
            line,
            column,
            message: message.into(),
            context: Some(Box::new(context)),
        }
    }

    /// Create an error for an invalid character.
    pub fn invalid_char(char: char, position: usize) -> Self {
        ToonError::InvalidCharacter { char, position }
    }

    /// Create an error for a type mismatch.
    pub fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        ToonError::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
        }
    }

    /// Create an error for array length mismatch.
    pub fn length_mismatch(expected: usize, found: usize) -> Self {
        ToonError::LengthMismatch {
            expected,
            found,
            context: None,
        }
    }

    /// Create an array length mismatch error with context.
    pub fn length_mismatch_with_context(
        expected: usize,
        found: usize,
        context: ErrorContext,
    ) -> Self {
        ToonError::LengthMismatch {
            expected,
            found,
            context: Some(Box::new(context)),
        }
    }

    /// Add context to an error if it supports it.
    pub fn with_context(self, context: ErrorContext) -> Self {
        match self {
            ToonError::ParseError {
                line,
                column,
                message,
                ..
            } => ToonError::ParseError {
                line,
                column,
                message,
                context: Some(Box::new(context)),
            },
            ToonError::LengthMismatch {
                expected, found, ..
            } => ToonError::LengthMismatch {
                expected,
                found,
                context: Some(Box::new(context)),
            },
            other => other,
        }
    }

    /// Add a suggestion to help fix the error.
    pub fn with_suggestion(self, suggestion: impl Into<String>) -> Self {
        let suggestion = suggestion.into();
        match self {
            ToonError::ParseError {
                line,
                column,
                message,
                context,
            } => {
                let new_context = context
                    .map(|c| Box::new(c.with_suggestion(suggestion.clone())))
                    .or_else(|| Some(Box::new(ErrorContext::new("").with_suggestion(suggestion))));
                ToonError::ParseError {
                    line,
                    column,
                    message,
                    context: new_context,
                }
            }
            other => other,
        }
    }
}
