use super::*;

#[test]
fn horizontal_and_vertical_keys_normalize_without_terminal_objects() {
    assert_eq!(
        normalize_terminal_tab_key(TabsOrientation::Horizontal, TerminalTabKey::Right),
        Some(TabsMessage::MoveNext)
    );
    assert_eq!(
        normalize_terminal_tab_key(TabsOrientation::Vertical, TerminalTabKey::Down),
        Some(TabsMessage::MoveNext)
    );
    assert_eq!(normalize_terminal_tab_key(TabsOrientation::Horizontal, TerminalTabKey::Down), None);
    assert_eq!(normalize_terminal_tab_key(TabsOrientation::Vertical, TerminalTabKey::Right), None);
}

#[test]
fn adapter_keeps_focus_before_caller_controlled_selection() {
    let state = canonical_tabs_state(TabsActivationPolicy::Automatic).expect("fixture state");
    let mut adapter = SubwayTabsAdapter { state };
    let transition = adapter.handle_key(TerminalTabKey::Right).expect("horizontal arrow is supported");

    assert!(adapter.projection().tabs[1].is_focused());
    assert!(adapter.projection().tabs[0].is_selected());
    assert!(matches!(transition.events.as_slice(), [
        ratcore::headless_components::TabsTransitionEvent::FocusChanged(_),
        ratcore::headless_components::TabsTransitionEvent::SelectionProposed(_)
    ]));
}

#[test]
fn subway_receipt_is_deterministic_and_semantic_only() {
    let first = subway_tabs_adapter_receipt().expect("first receipt");
    let second = subway_tabs_adapter_receipt().expect("second receipt");

    assert_eq!(first, second);
    assert_eq!(first.selected.as_str(), "tab.history");
    assert_eq!(first.focused.as_str(), "tab.history");
    assert_eq!(first.diagnostics.len(), 1);
    assert_eq!(first.diagnostics[0].code, ratcore::headless_components::TabsDiagnosticCode::UnknownTarget);
    assert!(!first.semantic_identity.is_empty());
}
