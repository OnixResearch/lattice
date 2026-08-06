use super::super::ActionBinding;
use super::super::ActionBindingSource;
use super::super::ActionBindingSourceKind;
use super::super::ActiveBindingLayer;
use super::super::BindingCatalogCandidate;
use super::super::BindingCatalogDiagnosticCode;
use super::super::BindingCatalogLimits;
use super::super::BindingLayer;
use super::super::BindingLayerKind;
use super::super::KeyModifier;
use super::super::ReservedKeySequence;
use super::ACTION_OPEN;
use super::ACTION_SAVE;
use super::LAYER_APP;
use super::LAYER_FOCUS;
use super::accepted_catalog;
use super::binding;
use super::layers;
use super::registry;
use super::sequence;
use super::stroke;

#[test]
fn catalog_accepts_zero_to_many_bindings_per_action() -> Result<(), String> {
    let first = binding(
        "binding.save.primary",
        ACTION_SAVE,
        sequence(vec![stroke('s', vec![KeyModifier::Control])?])?,
        LAYER_APP,
    );
    let second = binding(
        "binding.save.alternate",
        ACTION_SAVE,
        sequence(vec![stroke('s', vec![KeyModifier::Alt])?])?,
        LAYER_APP,
    );
    let catalog = accepted_catalog(1, vec![second, first])?;

    assert_eq!(catalog.bindings().len(), 2);
    assert_eq!(catalog.bindings()[0].id, "binding.save.alternate");
    assert_eq!(catalog.bindings()[1].id, "binding.save.primary");
    Ok(())
}

#[test]
fn duplicate_unknown_reserved_and_prefix_conflicts_reject_atomically() -> Result<(), String> {
    let previous = accepted_catalog(1, vec![binding(
        "binding.open",
        ACTION_OPEN,
        sequence(vec![stroke('o', vec![KeyModifier::Control])?])?,
        LAYER_APP,
    )])?;
    let prefix = sequence(vec![stroke('x', vec![KeyModifier::Control])?])?;
    let longer = sequence(vec![
        stroke('x', vec![KeyModifier::Control])?,
        stroke('s', vec![KeyModifier::Control])?,
    ])?;
    let reserved = sequence(vec![stroke('q', vec![KeyModifier::Control])?])?;
    let bindings = vec![
        binding("duplicate", ACTION_SAVE, prefix, LAYER_APP),
        binding("duplicate", ACTION_OPEN, longer, LAYER_APP),
        binding("binding.unknown-action", "missing.action", reserved.clone(), LAYER_FOCUS),
        binding("binding.unknown-layer", ACTION_SAVE, reserved.clone(), "missing.layer"),
        binding("binding.duplicate-sequence.one", ACTION_SAVE, reserved.clone(), LAYER_APP),
        binding("binding.duplicate-sequence.two", ACTION_OPEN, reserved.clone(), LAYER_APP),
    ];
    let candidate = BindingCatalogCandidate::new(2, layers(2), bindings)
        .with_reserved(vec![ReservedKeySequence::new(reserved, "shell reserved")]);
    let build = super::super::accept_binding_catalog(candidate, &registry(), Some(&previous));
    let codes = build.diagnostics.iter().map(|item| item.code).collect::<std::collections::BTreeSet<_>>();

    assert!(!build.candidate_accepted);
    assert_eq!(build.snapshot.as_ref(), Some(&previous));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::DuplicateBindingId));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::UnknownAction));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::UnknownLayer));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::ReservedSequence));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::DuplicateSequence));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::PrefixConflict));
    Ok(())
}

#[test]
fn malformed_override_and_excessive_limits_fail_closed() -> Result<(), String> {
    let sequence = sequence(vec![stroke('s', vec![KeyModifier::Control])?])?;
    let default = binding("binding.default", ACTION_SAVE, sequence.clone(), LAYER_APP).disabled("replaced");
    let override_binding = ActionBinding::new(
        "binding.override",
        ACTION_OPEN,
        sequence,
        LAYER_FOCUS,
        ActionBindingSource::new("user", ActionBindingSourceKind::UserOverride).replacing("binding.default"),
    );
    let limits = BindingCatalogLimits {
        sequence_strokes: 1,
        bindings: 1,
        layers: 1,
        active_layers: 1,
        continuations: 1,
    };
    let candidate = BindingCatalogCandidate::new(1, layers(1), vec![default, override_binding]).with_limits(limits);
    let build = super::super::accept_binding_catalog(candidate, &registry(), None);

    assert!(build.snapshot.is_none());
    assert!(
        build
            .diagnostics
            .iter()
            .any(|item| { item.code == BindingCatalogDiagnosticCode::MalformedOverride })
    );
    assert!(build.diagnostics.iter().any(|item| item.code == BindingCatalogDiagnosticCode::LimitExceeded));
    Ok(())
}

#[test]
fn excessive_continuation_fanout_is_rejected() -> Result<(), String> {
    let limits = BindingCatalogLimits {
        sequence_strokes: 2,
        bindings: 8,
        layers: 4,
        active_layers: 4,
        continuations: 1,
    };
    let candidate = BindingCatalogCandidate::new(
        1,
        vec![BindingLayer::new(LAYER_APP, 10, BindingLayerKind::Application, 1)],
        vec![
            binding("binding.save", ACTION_SAVE, sequence(vec![stroke('s', Vec::new())?])?, LAYER_APP),
            binding("binding.open", ACTION_OPEN, sequence(vec![stroke('o', Vec::new())?])?, LAYER_APP),
        ],
    )
    .with_limits(limits);
    let build = super::super::accept_binding_catalog(candidate, &registry(), None);

    assert!(build.snapshot.is_none());
    assert!(
        build
            .diagnostics
            .iter()
            .any(|item| { item.code == BindingCatalogDiagnosticCode::ContinuationFanoutExceeded })
    );
    Ok(())
}

#[test]
fn active_layers_reject_duplicates_disabled_unknown_and_stale() -> Result<(), String> {
    let mut candidate_layers = vec![
        BindingLayer::new(LAYER_APP, 10, BindingLayerKind::Application, 1),
        BindingLayer::new(LAYER_FOCUS, 20, BindingLayerKind::FocusedComponent, 1).disabled("focus unavailable"),
    ];
    candidate_layers.reverse();
    let build = super::super::accept_binding_catalog(
        BindingCatalogCandidate::new(1, candidate_layers, Vec::new()),
        &registry(),
        None,
    );
    let catalog = build.snapshot.ok_or_else(|| format!("catalog diagnostics: {:?}", build.diagnostics))?;
    let result = catalog.activate(1, vec![
        ActiveBindingLayer::new(LAYER_APP, 0),
        ActiveBindingLayer::new(LAYER_APP, 1),
        ActiveBindingLayer::new(LAYER_FOCUS, 1),
        ActiveBindingLayer::new("missing.layer", 1),
    ]);
    let codes = result
        .err()
        .ok_or_else(|| "invalid active layers were accepted".to_string())?
        .into_iter()
        .map(|item| item.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(codes.contains(&BindingCatalogDiagnosticCode::DuplicateActiveLayer));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::StaleLayerGeneration));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::DisabledLayer));
    assert!(codes.contains(&BindingCatalogDiagnosticCode::UnknownLayer));
    Ok(())
}
