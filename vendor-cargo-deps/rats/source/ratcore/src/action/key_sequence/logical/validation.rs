use std::collections::BTreeSet;

use super::KeyModifier;
use super::KeySequenceDiagnostic;
use super::KeySequenceDiagnosticCode;
use super::LogicalKey;
use super::MAX_FUNCTION_KEY;
use super::MIN_FUNCTION_KEY;

pub(super) fn validate_key(key: &LogicalKey) -> Vec<KeySequenceDiagnostic> {
    match key {
        LogicalKey::Character(value) if value.is_control() || value.is_whitespace() => {
            vec![KeySequenceDiagnostic::new(
                KeySequenceDiagnosticCode::MalformedCharacter,
                None,
                "character shortcut tokens must be committed non-whitespace text",
            )]
        }
        LogicalKey::Function(number) if !(MIN_FUNCTION_KEY..=MAX_FUNCTION_KEY).contains(number) => {
            vec![KeySequenceDiagnostic::new(
                KeySequenceDiagnosticCode::UnsupportedFunctionKey,
                None,
                format!("function key F{number} is outside the supported range"),
            )]
        }
        LogicalKey::Unsupported(value) => vec![KeySequenceDiagnostic::new(
            KeySequenceDiagnosticCode::UnsupportedKey,
            None,
            format!("unsupported logical key token `{value}`"),
        )],
        LogicalKey::Composition(_) => vec![KeySequenceDiagnostic::new(
            KeySequenceDiagnosticCode::CompositionExcluded,
            None,
            "uncommitted composition text cannot be a shortcut",
        )],
        _ => Vec::new(),
    }
}

pub(super) fn validate_modifiers(modifiers: &[KeyModifier]) -> Vec<KeySequenceDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();
    for modifier in modifiers {
        if !modifier.is_supported() {
            diagnostics.push(KeySequenceDiagnostic::new(
                KeySequenceDiagnosticCode::UnsupportedModifier,
                None,
                format!("unsupported logical modifier `{}`", modifier.label()),
            ));
        }
        if !seen.insert(modifier.clone()) {
            diagnostics.push(KeySequenceDiagnostic::new(
                KeySequenceDiagnosticCode::DuplicateModifier,
                None,
                format!("duplicate logical modifier `{}`", modifier.label()),
            ));
        }
    }
    diagnostics
}

pub(super) fn canonical_key(key: LogicalKey) -> LogicalKey {
    match key {
        LogicalKey::Character(value) if value.is_ascii_uppercase() => LogicalKey::Character(value.to_ascii_lowercase()),
        other => other,
    }
}
