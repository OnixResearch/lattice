mod candidate;

use super::super::*;

#[test]
fn model_rejects_duplicate_missing_empty_ineligible_and_no_eligible_candidates() {
    let mut duplicate = catalog();
    duplicate.tabs.push(duplicate.tabs[0].clone());
    let duplicate_codes = candidate::codes(duplicate, candidate::AcceptedIds::same("tab.one"));
    assert!(duplicate_codes.contains(&TabsDiagnosticCode::DuplicateTab));

    let mut missing = catalog();
    missing.panels.remove(0);
    let missing_codes = candidate::codes(missing, candidate::AcceptedIds::same("tab.one"));
    assert!(missing_codes.contains(&TabsDiagnosticCode::MissingPanel));

    let empty_codes = candidate::codes(
        TabsCatalog {
            tabs: Vec::new(),
            panels: Vec::new(),
        },
        candidate::AcceptedIds::same("tab.missing"),
    );
    assert!(empty_codes.contains(&TabsDiagnosticCode::EmptyCatalog));
    assert!(empty_codes.contains(&TabsDiagnosticCode::NoEligibleTab));

    let mut ineligible = catalog();
    ineligible.tabs[0].is_disabled = true;
    let ineligible_codes = candidate::codes(ineligible, candidate::AcceptedIds::same("tab.one"));
    assert!(ineligible_codes.contains(&TabsDiagnosticCode::IneligibleSelection));
    assert!(ineligible_codes.contains(&TabsDiagnosticCode::IneligibleFocus));

    let mut unavailable = catalog();
    for tab in &mut unavailable.tabs {
        tab.is_hidden = true;
    }
    let unavailable_codes = candidate::codes(unavailable, candidate::AcceptedIds::same("tab.one"));
    assert!(unavailable_codes.contains(&TabsDiagnosticCode::NoEligibleTab));
}

#[test]
fn manual_and_automatic_activation_keep_selection_caller_controlled() {
    let mut manual = state(TabsActivationPolicy::Manual);
    let unchanged = reduce_tabs(&mut manual, TabsMessage::ActivateFocused);
    assert_eq!(unchanged.kind, TabsTransitionKind::Ignored);
    assert!(unchanged.intents.is_empty());

    let moved = reduce_tabs(&mut manual, TabsMessage::MoveNext);
    assert_eq!(manual.focused().as_str(), "tab.two");
    assert_eq!(manual.selected().as_str(), "tab.one");
    assert!(moved.intents.is_empty());

    let proposed = reduce_tabs(&mut manual, TabsMessage::ActivateFocused);
    assert_eq!(manual.selected().as_str(), "tab.one");
    assert_eq!(proposed.events.len(), 1);
    assert_eq!(proposed.intents.len(), 1);
    let committed = reduce_tabs(&mut manual, TabsMessage::CommitSelection(TabId::new("tab.two")));
    assert_eq!(committed.kind, TabsTransitionKind::Accepted);
    assert_eq!(manual.selected().as_str(), "tab.two");

    let mut automatic = state(TabsActivationPolicy::Automatic);
    let moved = reduce_tabs(&mut automatic, TabsMessage::MoveNext);
    assert_eq!(automatic.focused().as_str(), "tab.two");
    assert_eq!(automatic.selected().as_str(), "tab.one");
    assert_eq!(moved.events.len(), 2);
    assert!(matches!(moved.events.as_slice(), [
        TabsTransitionEvent::FocusChanged(_),
        TabsTransitionEvent::SelectionProposed(_)
    ]));
}

#[test]
fn pointer_orders_focus_before_selection_and_waits_for_caller_commit() {
    let mut accepted = state(TabsActivationPolicy::Manual);
    let transition = reduce_tabs(&mut accepted, TabsMessage::PointerActivate(TabId::new("tab.two")));

    assert_eq!(accepted.focused().as_str(), "tab.two");
    assert_eq!(accepted.selected().as_str(), "tab.one");
    assert!(matches!(transition.events.as_slice(), [
        TabsTransitionEvent::FocusChanged(_),
        TabsTransitionEvent::SelectionProposed(_)
    ]));
    assert!(matches!(transition.intents.as_slice(), [TabsIntent::SelectionProposed {
        cause: TabsIntentCause::PointerActivation,
        ..
    }]));
}

#[test]
fn policy_messages_update_generation_and_ignore_repeated_values() {
    let mut accepted = state(TabsActivationPolicy::Manual);
    let generation = accepted.generation();
    let orientation = reduce_tabs(&mut accepted, TabsMessage::SetOrientation(TabsOrientation::Vertical));
    assert_eq!(orientation.kind, TabsTransitionKind::Accepted);
    assert_eq!(accepted.orientation(), TabsOrientation::Vertical);
    assert_eq!(accepted.generation(), generation.saturating_add(1));

    let activation = reduce_tabs(&mut accepted, TabsMessage::SetActivationPolicy(TabsActivationPolicy::Automatic));
    assert_eq!(activation.kind, TabsTransitionKind::Accepted);
    assert_eq!(accepted.activation(), TabsActivationPolicy::Automatic);
    let repeated = reduce_tabs(&mut accepted, TabsMessage::SetActivationPolicy(TabsActivationPolicy::Automatic));
    assert_eq!(repeated.kind, TabsTransitionKind::Ignored);
    assert_eq!(repeated.invalidation, TabsInvalidation::None);
}

#[test]
fn traversal_wraps_supports_home_end_and_skips_disabled_hidden_tabs() {
    let mut candidate = catalog();
    candidate.tabs[1].is_disabled = true;
    candidate.tabs[2].is_hidden = true;
    let mut accepted = TabsState::new(
        candidate,
        TabId::new("tab.one"),
        TabId::new("tab.one"),
        TabsOrientation::Vertical,
        TabsActivationPolicy::Manual,
    )
    .expect("one and four remain eligible");

    let _ = reduce_tabs(&mut accepted, TabsMessage::MoveNext);
    assert_eq!(accepted.focused().as_str(), "tab.four");
    let _ = reduce_tabs(&mut accepted, TabsMessage::MoveNext);
    assert_eq!(accepted.focused().as_str(), "tab.one");
    let _ = reduce_tabs(&mut accepted, TabsMessage::MovePrevious);
    assert_eq!(accepted.focused().as_str(), "tab.four");
    let _ = reduce_tabs(&mut accepted, TabsMessage::MoveFirst);
    assert_eq!(accepted.focused().as_str(), "tab.one");
    let _ = reduce_tabs(&mut accepted, TabsMessage::MoveLast);
    assert_eq!(accepted.focused().as_str(), "tab.four");
}

#[test]
fn catalog_replacement_preserves_valid_identity_and_repairs_removed_identity() {
    let mut accepted = state(TabsActivationPolicy::Automatic);
    let _ = reduce_tabs(&mut accepted, TabsMessage::FocusTab(TabId::new("tab.two")));
    let _ = reduce_tabs(&mut accepted, TabsMessage::CommitSelection(TabId::new("tab.two")));
    let reordered = reordered_catalog();
    let preserved = reduce_tabs(&mut accepted, TabsMessage::ReplaceCatalog(reordered));
    assert_eq!(preserved.kind, TabsTransitionKind::Accepted);
    assert_eq!(accepted.selected().as_str(), "tab.two");
    assert_eq!(accepted.focused().as_str(), "tab.two");

    let mut removed = catalog();
    removed.tabs.retain(|tab| tab.id.as_str() != "tab.two");
    removed.panels.retain(|panel| panel.id.as_str() != "panel.two");
    let repaired = reduce_tabs(&mut accepted, TabsMessage::ReplaceCatalog(removed));
    assert_eq!(accepted.selected().as_str(), "tab.one");
    assert_eq!(accepted.focused().as_str(), "tab.one");
    assert!(matches!(repaired.intents.as_slice(), [TabsIntent::SelectionProposed {
        cause: TabsIntentCause::CatalogRepair,
        ..
    }]));
}

#[test]
fn rejected_focus_pointer_and_selection_preserve_accepted_state() {
    let mut accepted = state(TabsActivationPolicy::Manual);
    let before = accepted.clone();
    let unknown = reduce_tabs(&mut accepted, TabsMessage::PointerActivate(TabId::new("tab.unknown")));
    assert_eq!(unknown.kind, TabsTransitionKind::Rejected);
    assert_eq!(accepted, before);

    let ineligible = reduce_tabs(&mut accepted, TabsMessage::FocusTab(TabId::new("tab.disabled")));
    assert_eq!(ineligible.kind, TabsTransitionKind::Rejected);
    assert_eq!(accepted, before);

    let commit = reduce_tabs(&mut accepted, TabsMessage::CommitSelection(TabId::new("tab.unknown")));
    assert_eq!(commit.kind, TabsTransitionKind::Rejected);
    assert_eq!(accepted, before);

    let mut invalid_catalog = catalog();
    invalid_catalog.tabs.push(invalid_catalog.tabs[0].clone());
    let replacement = reduce_tabs(&mut accepted, TabsMessage::ReplaceCatalog(invalid_catalog));
    assert_eq!(replacement.kind, TabsTransitionKind::Rejected);
    assert!(replacement.diagnostics.iter().any(|issue| issue.code == TabsDiagnosticCode::DuplicateTab));
    assert_eq!(accepted, before);
}

fn state(activation: TabsActivationPolicy) -> TabsState {
    TabsState::new(catalog(), TabId::new("tab.one"), TabId::new("tab.one"), TabsOrientation::Horizontal, activation)
        .expect("test catalog is valid")
}

fn catalog() -> TabsCatalog {
    catalog_from_rows(&[
        ("tab.one", "panel.one", false, false),
        ("tab.two", "panel.two", false, false),
        ("tab.disabled", "panel.disabled", true, false),
        ("tab.four", "panel.four", false, false),
    ])
}

fn reordered_catalog() -> TabsCatalog {
    catalog_from_rows(&[
        ("tab.four", "panel.four", false, false),
        ("tab.two", "panel.two", false, false),
        ("tab.one", "panel.one", false, false),
        ("tab.disabled", "panel.disabled", true, false),
    ])
}

fn catalog_from_rows(rows: &[(&str, &str, bool, bool)]) -> TabsCatalog {
    TabsCatalog {
        tabs: rows
            .iter()
            .map(|(id, panel, disabled, hidden)| TabDescriptor {
                id: TabId::new(*id),
                panel_id: TabPanelId::new(*panel),
                label: id.replace("tab.", "Tab "),
                is_disabled: *disabled,
                is_hidden: *hidden,
            })
            .collect(),
        panels: rows
            .iter()
            .map(|(_, panel, _, _)| TabPanelDescriptor {
                id: TabPanelId::new(*panel),
                slot_id: TabSlotId::new(panel.replace("panel.", "slot.")),
            })
            .collect(),
    }
}
