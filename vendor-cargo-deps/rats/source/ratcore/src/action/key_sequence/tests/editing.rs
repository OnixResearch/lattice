use super::super::ActionBindingSource;
use super::super::ActionBindingSourceKind;
use super::super::BindingCatalogLimits;
use super::super::BindingEditCommand;
use super::super::KeyModifier;
use super::super::SequenceCapture;
use super::super::SequenceCaptureOutcome;
use super::super::apply_binding_edit;
use super::ACTION_SAVE;
use super::LAYER_APP;
use super::accepted_catalog;
use super::active;
use super::binding;
use super::registry;
use super::sequence;
use super::stroke;

#[test]
fn editor_adds_multiple_bindings_and_rejects_prefix_conflicts_atomically() -> Result<(), String> {
    let original = binding(
        "binding.save.long",
        ACTION_SAVE,
        sequence(vec![
            stroke('x', vec![KeyModifier::Control])?,
            stroke('s', vec![KeyModifier::Control])?,
        ])?,
        LAYER_APP,
    );
    let catalog = accepted_catalog(1, vec![original])?;
    let alternate = binding(
        "binding.save.alternate",
        ACTION_SAVE,
        sequence(vec![stroke('s', vec![KeyModifier::Alt])?])?,
        LAYER_APP,
    );
    let added = apply_binding_edit(
        &catalog,
        &registry(),
        &BindingEditCommand::Add {
            observed_catalog_generation: 1,
            binding: alternate,
        },
        2,
    );
    let conflicting = binding(
        "binding.save.prefix",
        ACTION_SAVE,
        sequence(vec![stroke('x', vec![KeyModifier::Control])?])?,
        LAYER_APP,
    );
    let rejected = apply_binding_edit(
        &added.snapshot,
        &registry(),
        &BindingEditCommand::Add {
            observed_catalog_generation: 2,
            binding: conflicting,
        },
        3,
    );

    assert!(added.accepted);
    assert_eq!(added.snapshot.bindings().len(), 2);
    assert!(!rejected.accepted);
    assert_eq!(rejected.snapshot, added.snapshot);
    Ok(())
}

#[test]
fn editor_rejects_reserved_sequences_without_replacing_snapshot() -> Result<(), String> {
    let reserved_sequence = sequence(vec![stroke('q', vec![KeyModifier::Control])?])?;
    let candidate = super::super::BindingCatalogCandidate::new(
        1,
        vec![super::super::BindingLayer::new(
            LAYER_APP,
            10,
            super::super::BindingLayerKind::Application,
            1,
        )],
        Vec::new(),
    )
    .with_reserved(vec![super::super::ReservedKeySequence::new(
        reserved_sequence.clone(),
        "shell reserved",
    )]);
    let build = super::super::accept_binding_catalog(candidate, &registry(), None);
    let catalog = build.snapshot.ok_or_else(|| format!("catalog diagnostics: {:?}", build.diagnostics))?;
    let outcome = apply_binding_edit(
        &catalog,
        &registry(),
        &BindingEditCommand::Add {
            observed_catalog_generation: 1,
            binding: binding("binding.quit", ACTION_SAVE, reserved_sequence, LAYER_APP),
        },
        2,
    );

    assert!(!outcome.accepted);
    assert_eq!(outcome.snapshot, catalog);
    Ok(())
}

#[test]
fn editor_replaces_clears_and_resets_with_provenance() -> Result<(), String> {
    let default = binding(
        "binding.save.default",
        ACTION_SAVE,
        sequence(vec![stroke('s', vec![KeyModifier::Control])?])?,
        LAYER_APP,
    );
    let catalog = accepted_catalog(1, vec![default])?;
    let replacement = super::super::ActionBinding::new(
        "binding.save.user",
        ACTION_SAVE,
        sequence(vec![stroke('s', vec![KeyModifier::Alt])?])?,
        LAYER_APP,
        ActionBindingSource::new("capture", ActionBindingSourceKind::UserOverride),
    );
    let replaced = apply_binding_edit(
        &catalog,
        &registry(),
        &BindingEditCommand::Replace {
            observed_catalog_generation: 1,
            binding_id: super::super::ActionBindingId::new("binding.save.default"),
            replacement,
        },
        2,
    );
    let cleared = apply_binding_edit(
        &replaced.snapshot,
        &registry(),
        &BindingEditCommand::Clear {
            observed_catalog_generation: 2,
            binding_id: super::super::ActionBindingId::new("binding.save.user"),
        },
        3,
    );
    let reset = apply_binding_edit(
        &cleared.snapshot,
        &registry(),
        &BindingEditCommand::ResetAction {
            observed_catalog_generation: 3,
            action_id: super::super::super::ActionId::new(ACTION_SAVE),
        },
        4,
    );

    assert!(replaced.accepted);
    assert!(replaced.snapshot.bindings().iter().any(|item| {
        item.id == "binding.save.user" && item.source.replaces.as_ref().is_some_and(|id| id == "binding.save.default")
    }));
    assert!(cleared.accepted);
    assert!(cleared.snapshot.bindings().iter().any(|item| { item.id == "binding.save.user" && !item.enabled }));
    assert!(reset.accepted);
    assert_eq!(reset.snapshot.bindings().len(), 1);
    assert!(reset.snapshot.bindings()[0].enabled);
    Ok(())
}

#[test]
fn stale_and_unsupported_edits_preserve_the_previous_catalog() -> Result<(), String> {
    let catalog = accepted_catalog(1, Vec::new())?;
    let stale = apply_binding_edit(
        &catalog,
        &registry(),
        &BindingEditCommand::ClearAction {
            observed_catalog_generation: 0,
            action_id: super::super::super::ActionId::new(ACTION_SAVE),
            layer_id: None,
        },
        2,
    );
    let unsupported = apply_binding_edit(
        &catalog,
        &registry(),
        &BindingEditCommand::UnsupportedCapture {
            observed_catalog_generation: 1,
            key: "Hyper".to_string(),
        },
        2,
    );

    assert!(!stale.accepted);
    assert_eq!(stale.snapshot, catalog);
    assert!(!unsupported.accepted);
    assert_eq!(unsupported.snapshot, catalog);
    Ok(())
}

#[test]
fn complete_capture_rejects_excess_and_stale_generations_without_partial_commit() -> Result<(), String> {
    let catalog = accepted_catalog(1, Vec::new())?;
    let active_layers = active(&catalog, 1, &[LAYER_APP])?;
    let capture = SequenceCapture::new(1, 1);
    let first = capture.push(stroke('x', vec![KeyModifier::Control])?, &BindingCatalogLimits::default());
    let accepted_capture = match first {
        SequenceCaptureOutcome::Capturing(state) => state,
        other => return Err(format!("capture rejected unexpectedly: {other:?}")),
    };
    let completed = accepted_capture.finish(&catalog, &active_layers);
    let limits = BindingCatalogLimits {
        sequence_strokes: 1,
        bindings: 1,
        layers: 1,
        active_layers: 1,
        continuations: 1,
    };
    let excessive = accepted_capture.push(stroke('s', Vec::new())?, &limits);
    let stale_active = active(&catalog, 2, &[LAYER_APP])?;
    let stale = accepted_capture.finish(&catalog, &stale_active);

    assert!(matches!(completed, SequenceCaptureOutcome::Complete(_)));
    assert!(matches!(excessive, SequenceCaptureOutcome::Rejected { .. }));
    assert!(matches!(stale, SequenceCaptureOutcome::Rejected { .. }));
    assert_eq!(accepted_capture.strokes().len(), 1);
    Ok(())
}
