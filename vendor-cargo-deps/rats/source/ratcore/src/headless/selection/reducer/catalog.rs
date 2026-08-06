pub(super) fn replace(
    state: &mut super::super::model::TabsState,
    catalog: super::super::model::TabsCatalog,
) -> super::TabsTransition {
    let first = catalog.eligible_tabs().next().map(|tab| tab.id.clone());
    let Some(repair) = first else {
        let issues = super::super::model::validate_tabs_candidate(&catalog, &state.selected, &state.focused);
        return super::rejected_issues(issues);
    };
    let selected = if catalog.is_eligible(&state.selected) {
        state.selected.clone()
    } else {
        repair.clone()
    };
    let focused = if catalog.is_eligible(&state.focused) {
        state.focused.clone()
    } else {
        repair
    };
    let issues = super::super::model::validate_tabs_candidate(&catalog, &selected, &focused);
    if !issues.is_empty() {
        return super::rejected_issues(issues);
    }
    debug_assert!(catalog.is_eligible(&selected));
    debug_assert!(catalog.is_eligible(&focused));
    let is_selected_repaired = selected != state.selected;
    let is_focused_repaired = focused != state.focused;
    state.catalog = catalog;
    state.selected = selected.clone();
    state.focused = focused.clone();
    state.generation = state.generation.saturating_add(1);
    replacement_transition(RepairStatus {
        selected,
        focused,
        is_selected_repaired,
        is_focused_repaired,
    })
}

struct RepairStatus {
    selected: super::super::model::TabId,
    focused: super::super::model::TabId,
    is_selected_repaired: bool,
    is_focused_repaired: bool,
}

fn replacement_transition(repair: RepairStatus) -> super::TabsTransition {
    let mut events = Vec::with_capacity(3);
    let mut intents = Vec::with_capacity(1);
    events.push(super::TabsTransitionEvent::CatalogReplaced);
    if repair.is_focused_repaired {
        events.push(super::TabsTransitionEvent::FocusChanged(repair.focused));
    }
    if repair.is_selected_repaired {
        events.push(super::TabsTransitionEvent::SelectionProposed(repair.selected.clone()));
        intents.push(super::TabsIntent::SelectionProposed {
            tab_id: repair.selected,
            cause: super::TabsIntentCause::CatalogRepair,
        });
    }
    debug_assert!(events.len() <= 3);
    debug_assert!(intents.len() <= 1);
    super::TabsTransition {
        kind: super::TabsTransitionKind::Accepted,
        intents,
        events,
        diagnostics: Vec::new(),
        invalidation: super::TabsInvalidation::Immediate,
    }
}

pub(super) fn set_orientation(
    state: &mut super::super::model::TabsState,
    orientation: super::super::model::TabsOrientation,
) -> super::TabsTransition {
    if state.orientation == orientation {
        return super::ignored();
    }
    state.orientation = orientation;
    state.generation = state.generation.saturating_add(1);
    super::TabsTransition {
        kind: super::TabsTransitionKind::Accepted,
        intents: Vec::new(),
        events: vec![super::TabsTransitionEvent::OrientationChanged(orientation)],
        diagnostics: Vec::new(),
        invalidation: super::TabsInvalidation::Immediate,
    }
}

pub(super) fn set_activation(
    state: &mut super::super::model::TabsState,
    policy: super::super::model::TabsActivationPolicy,
) -> super::TabsTransition {
    if state.activation == policy {
        return super::ignored();
    }
    state.activation = policy;
    state.generation = state.generation.saturating_add(1);
    super::TabsTransition {
        kind: super::TabsTransitionKind::Accepted,
        intents: Vec::new(),
        events: vec![super::TabsTransitionEvent::ActivationPolicyChanged(policy)],
        diagnostics: Vec::new(),
        invalidation: super::TabsInvalidation::Immediate,
    }
}
