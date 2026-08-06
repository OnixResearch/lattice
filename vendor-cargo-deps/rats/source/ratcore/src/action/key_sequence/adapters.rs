use super::KeyModifier;
use super::LogicalKey;
use super::LogicalKeystroke;
const DEFAULT_MAX_FUNCTION_KEY: u8 = 35;

/// Plain renderer-owned key facts before shared logical normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererKeyInput {
    /// Renderer key value, such as `k`, `Enter`, or `F5`.
    pub key: String,
    /// Renderer modifier names.
    pub modifiers: Vec<String>,
    /// Whether this is uncommitted IME/composition input.
    pub composing: bool,
    /// Whether this is a backend repeat event.
    pub repeat: bool,
    /// Optional renderer-local physical key location.
    pub location: Option<String>,
}

impl RendererKeyInput {
    /// Creates committed, non-repeat renderer input.
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
}

/// Availability of one renderer key capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCapabilityStatus {
    /// Capability is available.
    Available,
    /// Capability is unavailable and must not be inferred.
    Unavailable,
}

impl KeyCapabilityStatus {
    const fn is_available(self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Explicit keyboard capability report for one renderer adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererKeyCapabilities {
    /// Renderer identity.
    pub renderer: super::super::ActionRendererAdapter,
    /// Modifiers the renderer can report distinctly.
    pub modifiers: Vec<KeyModifier>,
    /// Availability of committed character keys.
    pub characters: KeyCapabilityStatus,
    /// Highest supported function-key number.
    pub max_function_key: Option<u8>,
    /// Whether repeat events are accepted as command strokes.
    pub repeat: KeyCapabilityStatus,
    /// Whether physical location is reported; location remains renderer-owned.
    pub location: KeyCapabilityStatus,
    /// Whether composition state is reported distinctly.
    pub composition: KeyCapabilityStatus,
}

impl RendererKeyCapabilities {
    /// Conservative Subwayrat terminal capability profile.
    #[must_use]
    pub fn subwayrat() -> Self {
        Self {
            renderer: super::super::ActionRendererAdapter::Subwayrat,
            modifiers: vec![KeyModifier::Control, KeyModifier::Alt, KeyModifier::Shift],
            characters: KeyCapabilityStatus::Available,
            max_function_key: Some(DEFAULT_MAX_FUNCTION_KEY),
            repeat: KeyCapabilityStatus::Unavailable,
            location: KeyCapabilityStatus::Unavailable,
            composition: KeyCapabilityStatus::Unavailable,
        }
    }

    /// Conservative Ratzilla browser-terminal capability profile.
    #[must_use]
    pub fn ratzilla() -> Self {
        Self {
            renderer: super::super::ActionRendererAdapter::Subwayrat,
            modifiers: vec![KeyModifier::Control, KeyModifier::Alt, KeyModifier::Shift],
            characters: KeyCapabilityStatus::Available,
            max_function_key: Some(DEFAULT_MAX_FUNCTION_KEY),
            repeat: KeyCapabilityStatus::Unavailable,
            location: KeyCapabilityStatus::Unavailable,
            composition: KeyCapabilityStatus::Unavailable,
        }
    }

    /// Conservative Meteorite browser/desktop capability profile.
    #[must_use]
    pub fn meteorite() -> Self {
        Self {
            renderer: super::super::ActionRendererAdapter::Meteorite,
            modifiers: vec![
                KeyModifier::Control,
                KeyModifier::Alt,
                KeyModifier::Shift,
                KeyModifier::Meta,
                KeyModifier::Super,
            ],
            characters: KeyCapabilityStatus::Available,
            max_function_key: Some(DEFAULT_MAX_FUNCTION_KEY),
            repeat: KeyCapabilityStatus::Unavailable,
            location: KeyCapabilityStatus::Available,
            composition: KeyCapabilityStatus::Available,
        }
    }

    /// Conservative capability profile for the mounted experimental Iced pilot.
    #[must_use]
    pub fn iced_pilot() -> Self {
        Self {
            renderer: super::super::ActionRendererAdapter::IcedPilot,
            modifiers: vec![KeyModifier::Control, KeyModifier::Alt, KeyModifier::Shift],
            characters: KeyCapabilityStatus::Available,
            max_function_key: Some(DEFAULT_MAX_FUNCTION_KEY),
            repeat: KeyCapabilityStatus::Unavailable,
            location: KeyCapabilityStatus::Available,
            composition: KeyCapabilityStatus::Unavailable,
        }
    }

    pub(super) fn supports_modifier(&self, modifier: &KeyModifier) -> bool {
        self.modifiers.contains(modifier)
    }

    pub(super) fn supports_key(&self, key: &LogicalKey) -> bool {
        match key {
            LogicalKey::Character(_) => self.characters.is_available(),
            LogicalKey::Function(number) => self.max_function_key.is_some_and(|limit| number <= &limit),
            LogicalKey::Unsupported(_) | LogicalKey::Composition(_) => false,
            _ => true,
        }
    }
}

/// Stable renderer-normalization diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyAdapterDiagnosticCode {
    /// Renderer key value has no accepted logical representation.
    UnsupportedKey,
    /// Renderer modifier is unavailable or unknown.
    UnsupportedModifier,
    /// Uncommitted composition input was excluded.
    CompositionExcluded,
    /// Repeat input is not accepted by this renderer policy.
    RepeatExcluded,
    /// Logical stroke validation rejected normalized data.
    MalformedStroke,
}

/// Deterministic renderer key adapter diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyAdapterDiagnostic {
    /// Stable diagnostic code.
    pub code: KeyAdapterDiagnosticCode,
    /// Renderer producing the input.
    pub renderer: super::super::ActionRendererAdapter,
    /// Human-readable explanation.
    pub message: String,
}

/// Normalizes plain renderer key facts into one logical stroke.
///
/// Repeat, location, and composition policy remains explicit at this adapter
/// boundary and none of those renderer facts is stored in Ratcore.
///
/// # Errors
///
/// Returns deterministic diagnostics when key/modifier capabilities are absent,
/// composition or repeat policy rejects input, or logical validation fails.
pub fn normalize_renderer_key_input(
    input: &RendererKeyInput,
    capabilities: &RendererKeyCapabilities,
) -> Result<LogicalKeystroke, Vec<KeyAdapterDiagnostic>> {
    let mut diagnostics = Vec::new();
    if input.composing {
        diagnostics.push(normalization::diagnostic(
            capabilities,
            KeyAdapterDiagnosticCode::CompositionExcluded,
            "uncommitted composition input cannot be normalized as a shortcut",
        ));
    }
    if input.repeat && !capabilities.repeat.is_available() {
        diagnostics.push(normalization::diagnostic(
            capabilities,
            KeyAdapterDiagnosticCode::RepeatExcluded,
            "renderer repeat input is excluded by shortcut policy",
        ));
    }
    let key = normalization::parse_key(input.key.as_str());
    if !capabilities.supports_key(&key) {
        diagnostics.push(normalization::diagnostic(
            capabilities,
            KeyAdapterDiagnosticCode::UnsupportedKey,
            format!("renderer does not support logical key `{}`", input.key),
        ));
    }
    let modifiers = normalization::parse_modifiers(input, capabilities, &mut diagnostics);
    if !diagnostics.is_empty() {
        diagnostics.sort_by(|left, right| left.code.cmp(&right.code).then_with(|| left.message.cmp(&right.message)));
        diagnostics.dedup();
        return Err(diagnostics);
    }
    LogicalKeystroke::try_new(key, modifiers).map_err(|issues| {
        issues
            .into_iter()
            .map(|issue| {
                normalization::diagnostic(capabilities, KeyAdapterDiagnosticCode::MalformedStroke, issue.message)
            })
            .collect()
    })
}

#[path = "adapters/normalization.rs"]
mod normalization;
