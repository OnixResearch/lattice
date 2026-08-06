#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MoveTarget {
    Previous,
    Next,
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FocusCause {
    Semantic,
    Pointer,
}

pub(super) fn move_focus(state: &mut super::super::model::TabsState, target: MoveTarget) -> super::TabsTransition {
    let eligible = state.catalog.eligible_tabs().map(|tab| tab.id.clone()).collect::<Vec<_>>();
    let Some(current) = eligible.iter().position(|id| id == &state.focused) else {
        return super::rejected(super::super::model::TabsDiagnosticCode::IneligibleFocus, state.focused.as_str());
    };
    debug_assert!(!eligible.is_empty());
    debug_assert!(current < eligible.len());
    let last = eligible.len().saturating_sub(1);
    let index = match target {
        MoveTarget::Previous => current.checked_sub(1).unwrap_or(last),
        MoveTarget::Next => {
            let next = current.saturating_add(1);
            if next >= eligible.len() { 0 } else { next }
        }
        MoveTarget::First => 0,
        MoveTarget::Last => last,
    };
    let Some(target_id) = eligible.get(index).cloned() else {
        return super::rejected(super::super::model::TabsDiagnosticCode::IneligibleFocus, state.focused.as_str());
    };
    focus_tab(state, target_id, FocusCause::Semantic)
}

pub(super) fn focus_tab(
    state: &mut super::super::model::TabsState,
    id: super::super::model::TabId,
    cause: FocusCause,
) -> super::TabsTransition {
    let Some(tab) = state.catalog.tab(&id) else {
        return super::rejected(super::super::model::TabsDiagnosticCode::UnknownTarget, id.as_str());
    };
    if tab.is_disabled || tab.is_hidden {
        return super::rejected(super::super::model::TabsDiagnosticCode::IneligibleTarget, id.as_str());
    }
    debug_assert!(state.catalog.tab(&id).is_some());
    debug_assert!(state.catalog.is_eligible(&id));
    let mut events = Vec::with_capacity(2);
    let kind = if state.focused == id {
        super::TabsTransitionKind::Ignored
    } else {
        state.focused = id.clone();
        state.generation = state.generation.saturating_add(1);
        events.push(super::TabsTransitionEvent::FocusChanged(id.clone()));
        super::TabsTransitionKind::Accepted
    };
    let should_propose = state.activation == super::super::model::TabsActivationPolicy::Automatic
        && state.selected != id
        && cause != FocusCause::Pointer;
    if should_propose {
        events.push(super::TabsTransitionEvent::SelectionProposed(id.clone()));
        super::TabsTransition {
            kind: super::TabsTransitionKind::Accepted,
            intents: vec![super::TabsIntent::SelectionProposed {
                tab_id: id,
                cause: super::TabsIntentCause::AutomaticFocus,
            }],
            events,
            diagnostics: Vec::new(),
            invalidation: super::TabsInvalidation::Immediate,
        }
    } else {
        unchanged_or_focused(kind, events)
    }
}

fn unchanged_or_focused(
    kind: super::TabsTransitionKind,
    events: Vec<super::TabsTransitionEvent>,
) -> super::TabsTransition {
    super::TabsTransition {
        kind,
        intents: Vec::new(),
        events,
        diagnostics: Vec::new(),
        invalidation: if kind == super::TabsTransitionKind::Accepted {
            super::TabsInvalidation::Immediate
        } else {
            super::TabsInvalidation::None
        },
    }
}

pub(super) fn pointer_activate(
    state: &mut super::super::model::TabsState,
    id: super::super::model::TabId,
) -> super::TabsTransition {
    let focus = focus_tab(state, id.clone(), FocusCause::Pointer);
    if focus.kind == super::TabsTransitionKind::Rejected {
        return focus;
    }
    debug_assert!(state.catalog.is_eligible(&id));
    debug_assert_eq!(state.focused, id);
    if state.selected == id {
        return focus;
    }
    let mut events = focus.events;
    events.reserve(1);
    events.push(super::TabsTransitionEvent::SelectionProposed(id.clone()));
    super::TabsTransition {
        kind: super::TabsTransitionKind::Accepted,
        intents: vec![super::TabsIntent::SelectionProposed {
            tab_id: id,
            cause: super::TabsIntentCause::PointerActivation,
        }],
        events,
        diagnostics: Vec::new(),
        invalidation: super::TabsInvalidation::Immediate,
    }
}

pub(super) fn activate_focused(state: &super::super::model::TabsState) -> super::TabsTransition {
    if state.selected == state.focused {
        super::ignored()
    } else {
        propose(state.focused.clone(), super::TabsIntentCause::KeyboardActivation)
    }
}

fn propose(id: super::super::model::TabId, cause: super::TabsIntentCause) -> super::TabsTransition {
    super::TabsTransition {
        kind: super::TabsTransitionKind::Accepted,
        intents: vec![super::TabsIntent::SelectionProposed {
            tab_id: id.clone(),
            cause,
        }],
        events: vec![super::TabsTransitionEvent::SelectionProposed(id)],
        diagnostics: Vec::new(),
        invalidation: super::TabsInvalidation::None,
    }
}

pub(super) fn commit(
    state: &mut super::super::model::TabsState,
    id: super::super::model::TabId,
) -> super::TabsTransition {
    if !state.catalog.is_eligible(&id) {
        let code = if state.catalog.tab(&id).is_some() {
            super::super::model::TabsDiagnosticCode::IneligibleTarget
        } else {
            super::super::model::TabsDiagnosticCode::InvalidSelectionCommit
        };
        return super::rejected(code, id.as_str());
    }
    debug_assert!(state.catalog.tab(&id).is_some());
    debug_assert!(state.catalog.is_eligible(&id));
    if state.selected == id {
        return super::ignored();
    }
    state.selected = id.clone();
    state.generation = state.generation.saturating_add(1);
    super::TabsTransition {
        kind: super::TabsTransitionKind::Accepted,
        intents: Vec::new(),
        events: vec![super::TabsTransitionEvent::SelectionCommitted(id)],
        diagnostics: Vec::new(),
        invalidation: super::TabsInvalidation::Immediate,
    }
}
