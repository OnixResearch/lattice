use std::collections::BTreeSet;

use super::super::KeyModifier;
use super::super::LogicalKey;
use super::KeyAdapterDiagnostic;
use super::KeyAdapterDiagnosticCode;
use super::RendererKeyCapabilities;
use super::RendererKeyInput;

pub(super) fn parse_modifiers(
    input: &RendererKeyInput,
    capabilities: &RendererKeyCapabilities,
    diagnostics: &mut Vec<KeyAdapterDiagnostic>,
) -> Vec<KeyModifier> {
    let mut modifiers = Vec::new();
    let mut seen = BTreeSet::new();
    for value in &input.modifiers {
        let modifier = parse_modifier(value);
        if !capabilities.supports_modifier(&modifier) {
            diagnostics.push(diagnostic(
                capabilities,
                KeyAdapterDiagnosticCode::UnsupportedModifier,
                format!("renderer does not support modifier `{value}`"),
            ));
        }
        if !seen.insert(modifier.clone()) {
            diagnostics.push(diagnostic(
                capabilities,
                KeyAdapterDiagnosticCode::MalformedStroke,
                format!("renderer reported duplicate modifier `{value}`"),
            ));
        }
        modifiers.push(modifier);
    }
    modifiers
}

fn parse_modifier(value: &str) -> KeyModifier {
    match value.trim().to_ascii_lowercase().as_str() {
        "ctrl" | "control" => KeyModifier::Control,
        "alt" | "option" => KeyModifier::Alt,
        "shift" => KeyModifier::Shift,
        "meta" => KeyModifier::Meta,
        "super" | "cmd" | "command" | "win" => KeyModifier::Super,
        _ => KeyModifier::Unsupported(value.to_string()),
    }
}

pub(super) fn parse_key(value: &str) -> LogicalKey {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        " " | "space" | "spacebar" => LogicalKey::Space,
        "enter" | "return" => LogicalKey::Enter,
        "escape" | "esc" => LogicalKey::Escape,
        "tab" => LogicalKey::Tab,
        "backspace" => LogicalKey::Backspace,
        "delete" | "del" => LogicalKey::Delete,
        "insert" | "ins" => LogicalKey::Insert,
        "arrowup" | "up" => LogicalKey::ArrowUp,
        "arrowdown" | "down" => LogicalKey::ArrowDown,
        "arrowleft" | "left" => LogicalKey::ArrowLeft,
        "arrowright" | "right" => LogicalKey::ArrowRight,
        "home" => LogicalKey::Home,
        "end" => LogicalKey::End,
        "pageup" => LogicalKey::PageUp,
        "pagedown" => LogicalKey::PageDown,
        _ => parse_literal_key(value.trim()),
    }
}

fn parse_literal_key(value: &str) -> LogicalKey {
    let mut characters = value.chars();
    match (characters.next(), characters.next()) {
        (Some(character), None) => LogicalKey::Character(character),
        _ => value
            .strip_prefix(['f', 'F'])
            .and_then(|number| number.parse::<u8>().ok())
            .map_or_else(|| LogicalKey::Unsupported(value.to_string()), LogicalKey::Function),
    }
}

pub(super) fn diagnostic(
    capabilities: &RendererKeyCapabilities,
    code: KeyAdapterDiagnosticCode,
    message: impl Into<String>,
) -> KeyAdapterDiagnostic {
    KeyAdapterDiagnostic {
        code,
        renderer: capabilities.renderer,
        message: message.into(),
    }
}
