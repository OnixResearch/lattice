//! Logical key-combination parsing and formatting.

use ratcore::action_registry::KeyModifier;
use ratcore::action_registry::LogicalKey;
use ratcore::action_registry::LogicalKeystroke;

const MIN_FUNCTION_KEY_NUMBER: u8 = 1;
const MAX_FUNCTION_KEY_NUMBER: u8 = 12;

/// Compatibility name for one validated logical keystroke.
pub type Chord = LogicalKeystroke;

/// Constructs one validated logical key combination using the legacy boolean
/// modifier order. Backend events are intentionally not accepted here.
///
/// # Errors
///
/// Returns Ratcore logical-key diagnostics when the key is malformed or unsupported.
pub fn try_chord(
    key: LogicalKey,
    ctrl: bool,
    alt: bool,
    shift: bool,
) -> Result<Chord, Vec<ratcore::action_registry::KeySequenceDiagnostic>> {
    let mut modifiers = Vec::new();
    if ctrl {
        modifiers.push(KeyModifier::Control);
    }
    if alt {
        modifiers.push(KeyModifier::Alt);
    }
    if shift {
        modifiers.push(KeyModifier::Shift);
    }
    LogicalKeystroke::try_new(key, modifiers)
}

/// Parses a human-readable logical key such as `Ctrl+k`, `Alt+Enter`, or `e`.
///
/// Modifier names are explicit policy. Unknown, duplicate, or empty parts are
/// rejected rather than ignored.
#[must_use]
pub fn parse_key_string(value: &str) -> Option<Chord> {
    let parts = value.split('+').map(str::trim).collect::<Vec<_>>();
    let (key_part, modifier_parts) = parts.split_last()?;
    if key_part.is_empty() || modifier_parts.iter().any(|part| part.is_empty()) {
        return None;
    }

    let mut modifiers = Vec::with_capacity(modifier_parts.len());
    for part in modifier_parts {
        let modifier = match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => KeyModifier::Control,
            "alt" | "option" => KeyModifier::Alt,
            "shift" => KeyModifier::Shift,
            "meta" => KeyModifier::Meta,
            "super" | "cmd" | "command" | "win" => KeyModifier::Super,
            _ => return None,
        };
        if modifiers.contains(&modifier) {
            return None;
        }
        modifiers.push(modifier);
    }

    LogicalKeystroke::try_new(parse_key(key_part)?, modifiers).ok()
}

fn parse_key(value: &str) -> Option<LogicalKey> {
    let normalized = value.to_ascii_lowercase();
    let key = match normalized.as_str() {
        "enter" | "return" | "cr" => LogicalKey::Enter,
        "esc" | "escape" => LogicalKey::Escape,
        "tab" => LogicalKey::Tab,
        "backspace" | "bs" => LogicalKey::Backspace,
        "delete" | "del" => LogicalKey::Delete,
        "insert" | "ins" => LogicalKey::Insert,
        "up" | "arrowup" => LogicalKey::ArrowUp,
        "down" | "arrowdown" => LogicalKey::ArrowDown,
        "left" | "arrowleft" => LogicalKey::ArrowLeft,
        "right" | "arrowright" => LogicalKey::ArrowRight,
        "home" => LogicalKey::Home,
        "end" => LogicalKey::End,
        "pageup" | "pgup" => LogicalKey::PageUp,
        "pagedown" | "pgdn" => LogicalKey::PageDown,
        "space" | "spc" | " " => LogicalKey::Space,
        _ => return parse_literal_key(value),
    };
    Some(key)
}

fn parse_literal_key(value: &str) -> Option<LogicalKey> {
    let mut characters = value.chars();
    if let (Some(character), None) = (characters.next(), characters.next()) {
        return Some(LogicalKey::Character(character));
    }

    let number = value.strip_prefix(['f', 'F'])?.parse::<u8>().ok()?;
    (MIN_FUNCTION_KEY_NUMBER..=MAX_FUNCTION_KEY_NUMBER)
        .contains(&number)
        .then_some(LogicalKey::Function(number))
}

/// Formats a logical key combination using deterministic compatibility labels.
#[must_use]
pub fn format(stroke: &Chord) -> String {
    let mut parts = stroke
        .modifiers()
        .iter()
        .map(|modifier| match modifier {
            KeyModifier::Control => "Ctrl".to_string(),
            KeyModifier::Alt => "Alt".to_string(),
            KeyModifier::Shift => "Shift".to_string(),
            KeyModifier::Meta => "Meta".to_string(),
            KeyModifier::Super => "Super".to_string(),
            KeyModifier::Unsupported(value) => value.clone(),
        })
        .collect::<Vec<_>>();
    parts.push(format_key(stroke.key(), stroke.modifiers().contains(&KeyModifier::Shift)));
    parts.join("+")
}

fn format_key(key: &LogicalKey, shifted: bool) -> String {
    match key {
        LogicalKey::Character(value) if shifted && value.is_ascii_lowercase() => value.to_ascii_uppercase().to_string(),
        LogicalKey::Character(value) => value.to_string(),
        LogicalKey::Space => "Space".to_string(),
        LogicalKey::Enter => "Enter".to_string(),
        LogicalKey::Escape => "Esc".to_string(),
        LogicalKey::Tab => "Tab".to_string(),
        LogicalKey::Backspace => "Backspace".to_string(),
        LogicalKey::Delete => "Delete".to_string(),
        LogicalKey::Insert => "Insert".to_string(),
        LogicalKey::ArrowUp => "Up".to_string(),
        LogicalKey::ArrowDown => "Down".to_string(),
        LogicalKey::ArrowLeft => "Left".to_string(),
        LogicalKey::ArrowRight => "Right".to_string(),
        LogicalKey::Home => "Home".to_string(),
        LogicalKey::End => "End".to_string(),
        LogicalKey::PageUp => "PageUp".to_string(),
        LogicalKey::PageDown => "PageDown".to_string(),
        LogicalKey::Function(number) => format!("F{number}"),
        LogicalKey::Unsupported(value) | LogicalKey::Composition(value) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use ratcore::action_registry::KeySequenceDiagnosticCode;

    use super::*;

    #[test]
    fn accepted_compatibility_strings_are_logical_and_deterministic() {
        let plain = parse_key_string("k").expect("plain key");
        let ctrl = parse_key_string("Ctrl+c").expect("control key");
        let function = parse_key_string("Shift+Ctrl+F1").expect("function key");

        assert_eq!(plain.key(), &LogicalKey::Character('k'));
        assert_eq!(format(&plain), "k");
        assert_eq!(format(&ctrl), "Ctrl+c");
        assert_eq!(format(&function), "Ctrl+Shift+F1");
        assert_eq!(format(&parse_key_string("escape").expect("escape")), "Esc");
        assert_eq!(format(&parse_key_string("space").expect("space")), "Space");
    }

    #[test]
    fn malformed_unknown_duplicate_and_unsupported_values_are_rejected() {
        assert!(parse_key_string("definitely-not-a-key").is_none());
        assert!(parse_key_string("F13").is_none());
        assert!(parse_key_string("Hyper+k").is_none());
        assert!(parse_key_string("Ctrl+Control+k").is_none());
        assert!(parse_key_string("Ctrl++k").is_none());

        let issues = LogicalKeystroke::try_new(LogicalKey::Composition("é".to_string()), Vec::new())
            .expect_err("composition cannot become a shortcut");
        assert_eq!(issues[0].code, KeySequenceDiagnosticCode::CompositionExcluded);
    }

    #[test]
    fn construction_canonicalizes_modifiers_and_nominal_key_identity() {
        let first =
            LogicalKeystroke::try_new(LogicalKey::Character('K'), vec![KeyModifier::Shift, KeyModifier::Control])
                .expect("valid stroke");
        let second =
            LogicalKeystroke::try_new(LogicalKey::Character('k'), vec![KeyModifier::Control, KeyModifier::Shift])
                .expect("valid stroke");

        let control = parse_key_string("Control+s").expect("control profile");
        let meta = parse_key_string("Meta+s").expect("meta profile");

        assert_eq!(first, second);
        assert_eq!(format(&first), "Ctrl+Shift+K");
        assert_ne!(first.key(), &LogicalKey::Unsupported("k".to_string()));
        assert_ne!(control, meta);
        assert_eq!(format(&control), "Ctrl+s");
        assert_eq!(format(&meta), "Meta+s");
    }
}
