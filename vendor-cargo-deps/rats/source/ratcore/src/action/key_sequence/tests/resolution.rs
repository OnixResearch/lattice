use super::super::BindingCatalogLimits;
use super::super::BindingCatalogSnapshot;
use super::super::BindingDecision;
use super::super::BindingInspectionState;
use super::super::BindingLayer;
use super::super::BindingLayerKind;
use super::super::BindingResolutionOutcome;
use super::super::KeyModifier;
use super::super::RendererKeyCapabilities;
use super::super::describe_key_sequence;
use super::super::project_binding_command_views;
use super::super::resolve_key_sequence;
use super::super::where_is_action;
use super::ACTION_OPEN;
use super::ACTION_SAVE;
use super::LAYER_APP;
use super::LAYER_FOCUS;
use super::LAYER_MODAL;
use super::LAYER_TRANSIENT;
use super::accepted_catalog;
use super::active;
use super::binding;
use super::registry;
use super::sequence;
use super::stroke;

#[test]
fn focused_layer_wins_and_application_falls_back_deterministically() -> Result<(), String> {
    let shortcut = sequence(vec![stroke('s', vec![KeyModifier::Control])?])?;
    let app = binding("binding.application.save", ACTION_SAVE, shortcut.clone(), LAYER_APP);
    let focus = binding("binding.focus.open", ACTION_OPEN, shortcut.clone(), LAYER_FOCUS);
    let catalog = accepted_catalog(1, vec![focus, app])?;
    let focused = active(&catalog, 1, &[LAYER_APP, LAYER_FOCUS])?;
    let application = active(&catalog, 1, &[LAYER_APP])?;

    let focused_result = resolve_key_sequence(&catalog, &registry(), &focused, &shortcut);
    let fallback = resolve_key_sequence(&catalog, &registry(), &application, &shortcut);

    assert!(matches!(
        focused_result.outcome,
        BindingResolutionOutcome::Resolved(ref item) if item.action_id == ACTION_OPEN
    ));
    assert!(matches!(
        fallback.outcome,
        BindingResolutionOutcome::Resolved(ref item) if item.action_id == ACTION_SAVE
    ));
    assert!(
        focused_result.provenance.considerations.iter().any(|item| {
            item.binding_id == "binding.application.save" && item.decision == BindingDecision::Shadowed
        })
    );
    Ok(())
}

#[test]
fn higher_prefix_controls_lower_exact_until_continuation() -> Result<(), String> {
    let prefix = sequence(vec![stroke('x', vec![KeyModifier::Control])?])?;
    let longer = sequence(vec![
        stroke('x', vec![KeyModifier::Control])?,
        stroke('s', vec![KeyModifier::Control])?,
    ])?;
    let catalog = accepted_catalog(1, vec![
        binding("binding.application.open", ACTION_OPEN, prefix.clone(), LAYER_APP),
        binding("binding.focus.save", ACTION_SAVE, longer.clone(), LAYER_FOCUS),
    ])?;
    let active = active(&catalog, 1, &[LAYER_APP, LAYER_FOCUS])?;

    let pending = resolve_key_sequence(&catalog, &registry(), &active, &prefix);
    let resolved = resolve_key_sequence(&catalog, &registry(), &active, &longer);

    assert!(matches!(
        pending.outcome,
        BindingResolutionOutcome::Pending(ref continuations) if continuations.len() == 1
    ));
    assert!(matches!(
        resolved.outcome,
        BindingResolutionOutcome::Resolved(ref item) if item.binding_id == "binding.focus.save"
    ));
    Ok(())
}

#[test]
fn transient_layer_overrides_and_modal_layer_blocks_fallback() -> Result<(), String> {
    let shortcut = sequence(vec![stroke('o', vec![KeyModifier::Control])?])?;
    let catalog = accepted_catalog(1, vec![
        binding("binding.application.open", ACTION_OPEN, shortcut.clone(), LAYER_APP),
        binding("binding.transient.save", ACTION_SAVE, shortcut.clone(), LAYER_TRANSIENT),
    ])?;
    let transient = active(&catalog, 1, &[LAYER_APP, LAYER_TRANSIENT])?;
    let modal = active(&catalog, 1, &[LAYER_APP, LAYER_MODAL])?;

    let overridden = resolve_key_sequence(&catalog, &registry(), &transient, &shortcut);
    let blocked = resolve_key_sequence(&catalog, &registry(), &modal, &shortcut);

    assert!(matches!(
        overridden.outcome,
        BindingResolutionOutcome::Resolved(ref item) if item.action_id == ACTION_SAVE
    ));
    assert!(matches!(blocked.outcome, BindingResolutionOutcome::Blocked(_)));
    Ok(())
}

#[test]
fn defensive_resolver_reports_equal_precedence_ambiguity() -> Result<(), String> {
    let shortcut = sequence(vec![stroke('s', vec![KeyModifier::Control])?])?;
    let catalog = BindingCatalogSnapshot {
        generation: 1,
        layers: vec![
            BindingLayer::new(LAYER_APP, 10, BindingLayerKind::Application, 1),
            BindingLayer::new(LAYER_FOCUS, 10, BindingLayerKind::FocusedComponent, 1),
        ],
        bindings: vec![
            binding("binding.application.save", ACTION_SAVE, shortcut.clone(), LAYER_APP),
            binding("binding.focus.open", ACTION_OPEN, shortcut.clone(), LAYER_FOCUS),
        ],
        reserved: Vec::new(),
        limits: BindingCatalogLimits::default(),
    };
    let active = active(&catalog, 1, &[LAYER_APP, LAYER_FOCUS])?;
    let result = resolve_key_sequence(&catalog, &registry(), &active, &shortcut);

    assert!(matches!(
        result.outcome,
        BindingResolutionOutcome::Ambiguous(ref ids) if ids.len() == 2
    ));
    Ok(())
}

#[test]
fn insertion_order_does_not_change_resolution_or_introspection() -> Result<(), String> {
    let sequence = sequence(vec![stroke('s', vec![KeyModifier::Control])?])?;
    let bindings = vec![
        binding("binding.application.save", ACTION_SAVE, sequence.clone(), LAYER_APP),
        binding("binding.focus.open", ACTION_OPEN, sequence.clone(), LAYER_FOCUS),
    ];
    let first = accepted_catalog(1, bindings.clone())?;
    let second = accepted_catalog(1, bindings.into_iter().rev().collect())?;
    let first_active = active(&first, 1, &[LAYER_FOCUS, LAYER_APP])?;
    let second_active = active(&second, 1, &[LAYER_APP, LAYER_FOCUS])?;

    let first_resolution = resolve_key_sequence(&first, &registry(), &first_active, &sequence);
    let second_resolution = resolve_key_sequence(&second, &registry(), &second_active, &sequence);
    let described =
        describe_key_sequence(&first, &registry(), &first_active, &sequence, &RendererKeyCapabilities::subwayrat());

    assert_eq!(first_resolution, second_resolution);
    assert_eq!(described.resolution, first_resolution);
    assert!(
        described
            .bindings
            .iter()
            .any(|item| { item.binding_id == "binding.focus.open" && item.state == BindingInspectionState::Winner })
    );
    assert!(described.bindings.iter().any(|item| {
        item.binding_id == "binding.application.save" && item.state == BindingInspectionState::Shadowed
    }));
    Ok(())
}

#[test]
fn where_is_reports_multiple_bindings_and_renderer_support() -> Result<(), String> {
    let control = sequence(vec![stroke('s', vec![KeyModifier::Control])?])?;
    let super_key = sequence(vec![stroke('s', vec![KeyModifier::Super])?])?;
    let catalog = accepted_catalog(1, vec![
        binding("binding.save.control", ACTION_SAVE, control, LAYER_APP),
        binding("binding.save.super", ACTION_SAVE, super_key, LAYER_APP),
    ])?;
    let active = active(&catalog, 1, &[LAYER_APP])?;
    let capabilities = RendererKeyCapabilities::subwayrat();
    let inspections = where_is_action(
        &catalog,
        &registry(),
        &active,
        &super::super::super::ActionId::new(ACTION_SAVE),
        &capabilities,
    );
    let views = project_binding_command_views(&catalog, &registry(), &active, &capabilities);

    assert_eq!(inspections.len(), 2);
    assert!(inspections.iter().any(|item| item.renderer_support.supported));
    assert!(inspections.iter().any(|item| !item.renderer_support.supported));
    assert!(views.menu.iter().any(|row| { row.action_id == ACTION_SAVE && row.sequence_labels.len() == 2 }));
    assert_eq!(views.palette, views.menu);
    assert!(views.shortcut_help.iter().any(|row| { row.action_id == ACTION_SAVE && row.bindings.len() == 2 }));
    Ok(())
}
