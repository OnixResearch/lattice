use super::super::BindingLayerId;
use super::super::KeyModifier;
use super::super::KeySequence;
use super::super::LogicalKey;
use super::super::LogicalKeystroke;
use super::ActionBinding;
use super::ActionBindingSource;
use super::ActionBindingSourceKind;
use super::BindingCatalogCandidate;
use super::BindingCatalogLimits;
use super::BindingLayer;
use super::BindingLayerKind;

const COMPAT_LAYER_PREFIX: &str = "compat.shortcut.";

/// Converts legacy descriptor shortcuts into one-stroke compatibility bindings.
///
/// The returned candidate is a migration input, not a second runtime authority.
///
/// # Errors
///
/// Returns logical-key diagnostics when a legacy shortcut contains an unsupported
/// key/modifier or otherwise cannot become a validated one-stroke sequence.
pub fn binding_catalog_from_registry_defaults(
    registry: &super::super::super::ActionRegistrySnapshot,
    generation: u64,
) -> Result<BindingCatalogCandidate, Vec<super::super::KeySequenceDiagnostic>> {
    let limits = BindingCatalogLimits::default();
    let mut layers = std::collections::BTreeMap::new();
    let mut bindings = Vec::new();
    let mut diagnostics = Vec::new();
    for action in registry.actions() {
        let Some(shortcut) = action.shortcut.as_ref() else {
            continue;
        };
        let layer_id =
            BindingLayerId::new(format!("{COMPAT_LAYER_PREFIX}{}", shortcut.scope.trim().to_ascii_lowercase()));
        layers
            .entry(layer_id.clone())
            .or_insert_with(|| BindingLayer::new(layer_id.clone(), 0, BindingLayerKind::Application, generation));
        match compatibility_sequence(shortcut, &limits) {
            Ok(sequence) => bindings.push(ActionBinding::new(
                format!("compat.binding.{}", action.id),
                action.id.clone(),
                sequence,
                layer_id,
                ActionBindingSource::new("action-descriptor", ActionBindingSourceKind::Default),
            )),
            Err(mut issues) => diagnostics.append(&mut issues),
        }
    }
    if diagnostics.is_empty() {
        Ok(BindingCatalogCandidate::new(generation, layers.into_values().collect(), bindings))
    } else {
        diagnostics.sort();
        Err(diagnostics)
    }
}

fn compatibility_sequence(
    shortcut: &super::super::super::ActionShortcut,
    limits: &BindingCatalogLimits,
) -> Result<KeySequence, Vec<super::super::KeySequenceDiagnostic>> {
    let key = compatibility_key(shortcut.key.trim());
    let modifiers = shortcut.modifiers.iter().map(|value| compatibility_modifier(value.trim())).collect();
    let stroke = LogicalKeystroke::try_new(key, modifiers)?;
    KeySequence::try_new(vec![stroke], limits)
}

fn compatibility_modifier(value: &str) -> KeyModifier {
    match value.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => KeyModifier::Control,
        "alt" | "option" => KeyModifier::Alt,
        "shift" => KeyModifier::Shift,
        "meta" => KeyModifier::Meta,
        "super" | "cmd" | "command" | "win" => KeyModifier::Super,
        _ => KeyModifier::Unsupported(value.to_string()),
    }
}

fn compatibility_key(value: &str) -> LogicalKey {
    let normalized = value.to_ascii_lowercase();
    match normalized.as_str() {
        "space" | "spc" => LogicalKey::Space,
        "enter" | "return" => LogicalKey::Enter,
        "escape" | "esc" => LogicalKey::Escape,
        "tab" => LogicalKey::Tab,
        "backspace" => LogicalKey::Backspace,
        "delete" | "del" => LogicalKey::Delete,
        "insert" | "ins" => LogicalKey::Insert,
        "up" => LogicalKey::ArrowUp,
        "down" => LogicalKey::ArrowDown,
        "left" => LogicalKey::ArrowLeft,
        "right" => LogicalKey::ArrowRight,
        "home" => LogicalKey::Home,
        "end" => LogicalKey::End,
        "pageup" | "pgup" => LogicalKey::PageUp,
        "pagedown" | "pgdown" => LogicalKey::PageDown,
        _ => compatibility_literal_key(value),
    }
}

fn compatibility_literal_key(value: &str) -> LogicalKey {
    let mut characters = value.chars();
    let first = characters.next();
    if let (Some(character), None) = (first, characters.next()) {
        return LogicalKey::Character(character);
    }
    let function_number = value.strip_prefix(['f', 'F']).and_then(|number| number.parse::<u8>().ok());
    function_number.map_or_else(|| LogicalKey::Unsupported(value.to_string()), LogicalKey::Function)
}
