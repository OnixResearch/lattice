use super::catalog::BindingCatalogLimits;

const MODIFIER_SEPARATOR: &str = "+";
const STROKE_SEPARATOR: &str = " ";
const MIN_FUNCTION_KEY: u8 = 1;
const MAX_FUNCTION_KEY: u8 = 35;

/// Canonical modifier understood by the shared logical key model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyModifier {
    /// Control modifier.
    Control,
    /// Alternate/Option modifier.
    Alt,
    /// Shift modifier.
    Shift,
    /// Meta modifier when reported distinctly by a renderer.
    Meta,
    /// Super/Command/Windows modifier when reported distinctly by a renderer.
    Super,
    /// Explicit renderer input that the shared model does not support.
    Unsupported(String),
}

impl KeyModifier {
    pub(super) fn label(&self) -> &str {
        match self {
            Self::Control => "Ctrl",
            Self::Alt => "Alt",
            Self::Shift => "Shift",
            Self::Meta => "Meta",
            Self::Super => "Super",
            Self::Unsupported(value) => value.as_str(),
        }
    }

    pub(super) const fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported(_))
    }
}

/// Renderer-neutral logical key token.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogicalKey {
    /// A committed non-control character used as a shortcut token.
    Character(char),
    /// Space key.
    Space,
    /// Enter/Return key.
    Enter,
    /// Escape key.
    Escape,
    /// Tab key.
    Tab,
    /// Backspace key.
    Backspace,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Up arrow.
    ArrowUp,
    /// Down arrow.
    ArrowDown,
    /// Left arrow.
    ArrowLeft,
    /// Right arrow.
    ArrowRight,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Function key number.
    Function(u8),
    /// Renderer key token with no accepted shared representation.
    Unsupported(String),
    /// Uncommitted IME/composition text, which is never a shortcut token.
    Composition(String),
}

impl LogicalKey {
    pub(super) fn label(&self) -> String {
        match self {
            Self::Character(value) => value.to_string(),
            Self::Space => "Space".to_string(),
            Self::Enter => "Enter".to_string(),
            Self::Escape => "Escape".to_string(),
            Self::Tab => "Tab".to_string(),
            Self::Backspace => "Backspace".to_string(),
            Self::Delete => "Delete".to_string(),
            Self::Insert => "Insert".to_string(),
            Self::ArrowUp => "Up".to_string(),
            Self::ArrowDown => "Down".to_string(),
            Self::ArrowLeft => "Left".to_string(),
            Self::ArrowRight => "Right".to_string(),
            Self::Home => "Home".to_string(),
            Self::End => "End".to_string(),
            Self::PageUp => "PageUp".to_string(),
            Self::PageDown => "PageDown".to_string(),
            Self::Function(number) => format!("F{number}"),
            Self::Unsupported(value) | Self::Composition(value) => value.clone(),
        }
    }
}

/// Stable validation code for logical strokes and sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeySequenceDiagnosticCode {
    /// A sequence contained no strokes.
    EmptySequence,
    /// A sequence exceeded its configured bound.
    SequenceTooLong,
    /// A key token is unsupported.
    UnsupportedKey,
    /// A modifier is unsupported.
    UnsupportedModifier,
    /// The same modifier appeared more than once.
    DuplicateModifier,
    /// A character token is control or whitespace text.
    MalformedCharacter,
    /// A function-key number is outside the accepted range.
    UnsupportedFunctionKey,
    /// Uncommitted composition text was offered as a shortcut.
    CompositionExcluded,
}

/// Deterministic logical key/sequence diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeySequenceDiagnostic {
    /// Stable diagnostic code.
    pub code: KeySequenceDiagnosticCode,
    /// Zero-based stroke position when applicable.
    pub stroke_index: Option<u16>,
    /// Human-readable explanation.
    pub message: String,
}

impl KeySequenceDiagnostic {
    pub(super) fn new(code: KeySequenceDiagnosticCode, stroke_index: Option<u16>, message: impl Into<String>) -> Self {
        Self {
            code,
            stroke_index,
            message: message.into(),
        }
    }
}

/// Validated renderer-neutral keystroke with canonical modifier order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalKeystroke {
    key: LogicalKey,
    modifiers: Vec<KeyModifier>,
}

impl LogicalKeystroke {
    /// Validates and constructs one logical keystroke.
    ///
    /// # Errors
    ///
    /// Returns stable diagnostics for unsupported keys/modifiers, duplicate
    /// modifiers, malformed characters, or composition input.
    pub fn try_new(key: LogicalKey, modifiers: Vec<KeyModifier>) -> Result<Self, Vec<KeySequenceDiagnostic>> {
        let mut diagnostics = validation::validate_key(&key);
        diagnostics.extend(validation::validate_modifiers(&modifiers));
        diagnostics.sort();
        diagnostics.dedup();
        if diagnostics.is_empty() {
            let mut canonical_modifiers = modifiers;
            canonical_modifiers.sort();
            Ok(Self {
                key: validation::canonical_key(key),
                modifiers: canonical_modifiers,
            })
        } else {
            Err(diagnostics)
        }
    }

    /// Returns the canonical key token.
    #[must_use]
    pub const fn key(&self) -> &LogicalKey {
        &self.key
    }

    /// Returns modifiers in canonical order.
    #[must_use]
    pub fn modifiers(&self) -> &[KeyModifier] {
        &self.modifiers
    }

    /// Returns deterministic neutral notation for diagnostics and fallback UI.
    #[must_use]
    pub fn canonical_label(&self) -> String {
        let mut parts = self.modifiers.iter().map(|modifier| modifier.label().to_string()).collect::<Vec<_>>();
        parts.push(self.key.label());
        parts.join(MODIFIER_SEPARATOR)
    }
}

/// Validated bounded non-empty key sequence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeySequence {
    strokes: Vec<LogicalKeystroke>,
}

impl KeySequence {
    /// Validates and constructs a sequence under catalog limits.
    ///
    /// # Errors
    ///
    /// Returns diagnostics when the sequence is empty or exceeds the accepted
    /// stroke bound.
    pub fn try_new(
        strokes: Vec<LogicalKeystroke>,
        limits: &BindingCatalogLimits,
    ) -> Result<Self, Vec<KeySequenceDiagnostic>> {
        let mut diagnostics = Vec::new();
        if strokes.is_empty() {
            diagnostics.push(KeySequenceDiagnostic::new(
                KeySequenceDiagnosticCode::EmptySequence,
                None,
                "key sequence must contain at least one stroke",
            ));
        }
        if strokes.len() > usize::from(limits.sequence_strokes) {
            diagnostics.push(KeySequenceDiagnostic::new(
                KeySequenceDiagnosticCode::SequenceTooLong,
                None,
                format!("key sequence has {} strokes; limit is {}", strokes.len(), limits.sequence_strokes),
            ));
        }
        if diagnostics.is_empty() {
            Ok(Self { strokes })
        } else {
            Err(diagnostics)
        }
    }

    /// Returns sequence strokes in order.
    #[must_use]
    pub fn strokes(&self) -> &[LogicalKeystroke] {
        &self.strokes
    }

    /// Returns deterministic neutral notation for diagnostics and fallback UI.
    #[must_use]
    pub fn canonical_label(&self) -> String {
        self.strokes
            .iter()
            .map(LogicalKeystroke::canonical_label)
            .collect::<Vec<_>>()
            .join(STROKE_SEPARATOR)
    }

    pub(super) fn pushed(
        &self,
        stroke: LogicalKeystroke,
        limits: &BindingCatalogLimits,
    ) -> Result<Self, Vec<KeySequenceDiagnostic>> {
        let mut strokes = self.strokes.clone();
        strokes.push(stroke);
        Self::try_new(strokes, limits)
    }

    pub(super) fn is_prefix_of(&self, other: &Self) -> bool {
        self.strokes.len() < other.strokes.len() && other.strokes.starts_with(self.strokes.as_slice())
    }

    pub(super) fn starts_with(&self, prefix: &Self) -> bool {
        self.strokes.starts_with(prefix.strokes.as_slice())
    }
}

#[path = "logical/validation.rs"]
mod validation;
