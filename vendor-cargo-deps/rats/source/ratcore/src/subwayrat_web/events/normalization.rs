//! Ratzilla-local keyboard facts and logical normalization.

/// Ratzilla key event normalized at the shell boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatzillaKeyInput {
    /// Key label.
    pub key: String,
    /// Modifier labels.
    pub modifiers: Vec<String>,
    /// Whether the browser event is uncommitted composition input.
    pub composing: bool,
    /// Whether the browser event is a repeat.
    pub repeat: bool,
    /// Optional renderer-owned physical location label.
    pub location: Option<String>,
}

impl RatzillaKeyInput {
    /// Creates committed, initial key facts.
    #[must_use]
    pub fn new(key: impl Into<String>, modifiers: Vec<String>) -> Self {
        Self {
            key: key.into(),
            modifiers,
            composing: false,
            repeat: false,
            location: None,
        }
    }

    /// Marks uncommitted composition facts.
    #[must_use]
    pub const fn with_composing(mut self, composing: bool) -> Self {
        self.composing = composing;
        self
    }

    /// Marks backend repeat facts.
    #[must_use]
    pub const fn with_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    /// Retains renderer-local physical location for diagnostics only.
    #[must_use]
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

/// Normalizes Ratzilla-local facts into the canonical logical-key model.
///
/// # Errors
///
/// Returns shared capability, composition, repeat, key, or modifier diagnostics.
pub fn normalize_ratzilla_key_input(
    input: &RatzillaKeyInput,
) -> Result<crate::action_registry::LogicalKeystroke, Vec<crate::action_registry::KeyAdapterDiagnostic>> {
    crate::action_registry::normalize_renderer_key_input(
        &crate::action_registry::RendererKeyInput {
            key: input.key.clone(),
            modifiers: input.modifiers.clone(),
            composing: input.composing,
            repeat: input.repeat,
            location: input.location.clone(),
        },
        &crate::action_registry::RendererKeyCapabilities::ratzilla(),
    )
}

/// Adapts a normalized Ratzilla key into this shell's bounded local actions.
#[must_use]
pub fn adapt_ratzilla_key_input(input: &RatzillaKeyInput) -> super::WebInputOutcome {
    let stroke = match normalize_ratzilla_key_input(input) {
        Ok(stroke) => stroke,
        Err(diagnostics) => {
            return super::diagnostic_outcome(failure_code(&diagnostics), "Ratzilla key normalization failed", None);
        }
    };
    logical_action(&stroke)
}

fn logical_action(stroke: &crate::action_registry::LogicalKeystroke) -> super::WebInputOutcome {
    match stroke.key() {
        crate::action_registry::LogicalKey::Enter => super::action_outcome(super::ACTION_ACTIVATE),
        crate::action_registry::LogicalKey::Escape => super::action_outcome(super::ACTION_CANCEL),
        _ => super::diagnostic_outcome(
            super::WebInputDiagnosticCode::UnsupportedKey,
            format!("unsupported logical web key: {}", stroke.canonical_label()),
            None,
        ),
    }
}

fn failure_code(diagnostics: &[crate::action_registry::KeyAdapterDiagnostic]) -> super::WebInputDiagnosticCode {
    let contains = |code| diagnostics.iter().any(|diagnostic| diagnostic.code == code);
    if contains(crate::action_registry::KeyAdapterDiagnosticCode::CompositionExcluded) {
        super::WebInputDiagnosticCode::CompositionExcluded
    } else if contains(crate::action_registry::KeyAdapterDiagnosticCode::RepeatExcluded) {
        super::WebInputDiagnosticCode::RepeatExcluded
    } else if contains(crate::action_registry::KeyAdapterDiagnosticCode::UnsupportedModifier) {
        super::WebInputDiagnosticCode::UnsupportedModifierCombination
    } else {
        super::WebInputDiagnosticCode::UnsupportedKey
    }
}
