use super::super::BindingCatalogLimits;
use super::super::KeyAdapterDiagnosticCode;
use super::super::KeyModifier;
use super::super::KeySequence;
use super::super::KeySequenceDiagnosticCode;
use super::super::LogicalKey;
use super::super::LogicalKeystroke;
use super::super::RendererKeyCapabilities;
use super::super::RendererKeyInput;
use super::super::normalize_renderer_key_input;

#[test]
fn logical_strokes_are_canonical_and_sequences_are_bounded() -> Result<(), String> {
    let stroke = LogicalKeystroke::try_new(LogicalKey::Character('K'), vec![KeyModifier::Shift, KeyModifier::Control])
        .map_err(|issues| format!("unexpected diagnostics: {issues:?}"))?;
    let sequence = KeySequence::try_new(vec![stroke.clone()], &BindingCatalogLimits::default())
        .map_err(|issues| format!("unexpected diagnostics: {issues:?}"))?;

    assert_eq!(stroke.key(), &LogicalKey::Character('k'));
    assert_eq!(stroke.modifiers(), &[KeyModifier::Control, KeyModifier::Shift]);
    assert_eq!(sequence.canonical_label(), "Ctrl+Shift+k");
    Ok(())
}

#[test]
fn malformed_logical_input_fails_closed() {
    let duplicate =
        LogicalKeystroke::try_new(LogicalKey::Character('k'), vec![KeyModifier::Control, KeyModifier::Control]);
    let unsupported = LogicalKeystroke::try_new(LogicalKey::Unsupported("MediaPlay".to_string()), Vec::new());
    let composition = LogicalKeystroke::try_new(LogicalKey::Composition("é".to_string()), Vec::new());
    let empty = KeySequence::try_new(Vec::new(), &BindingCatalogLimits::default());

    assert!(duplicate.is_err());
    assert!(unsupported.is_err());
    assert!(composition.is_err());
    assert!(matches!(
        empty,
        Err(ref issues)
            if issues.iter().any(|issue| issue.code == KeySequenceDiagnosticCode::EmptySequence)
    ));
}

#[test]
fn renderer_adapters_preserve_capability_truth_and_logical_parity() -> Result<(), String> {
    let terminal = normalize_renderer_key_input(
        &RendererKeyInput::new("k", vec!["Ctrl".to_string()]),
        &RendererKeyCapabilities::subwayrat(),
    )
    .map_err(|issues| format!("terminal diagnostics: {issues:?}"))?;
    let meteorite = normalize_renderer_key_input(
        &RendererKeyInput::new("K", vec!["Control".to_string()]),
        &RendererKeyCapabilities::meteorite(),
    )
    .map_err(|issues| format!("meteorite diagnostics: {issues:?}"))?;
    let unavailable = normalize_renderer_key_input(
        &RendererKeyInput::new("k", vec!["Super".to_string()]),
        &RendererKeyCapabilities::subwayrat(),
    );

    assert_eq!(terminal, meteorite);
    assert!(matches!(
        unavailable,
        Err(ref issues)
            if issues.iter().any(|issue| issue.code == KeyAdapterDiagnosticCode::UnsupportedModifier)
    ));
    Ok(())
}

#[test]
fn composition_repeat_and_duplicate_modifiers_remain_renderer_owned() {
    let mut composing = RendererKeyInput::new("Dead", Vec::new());
    composing.composing = true;
    let mut repeated = RendererKeyInput::new("k", Vec::new());
    repeated.repeat = true;
    let duplicate = RendererKeyInput::new("k", vec!["Ctrl".to_string(), "Control".to_string()]);

    assert!(normalize_renderer_key_input(&composing, &RendererKeyCapabilities::meteorite()).is_err());
    assert!(normalize_renderer_key_input(&repeated, &RendererKeyCapabilities::subwayrat()).is_err());
    assert!(normalize_renderer_key_input(&duplicate, &RendererKeyCapabilities::subwayrat()).is_err());
}
