use ratcore::action_registry::BindingCatalogLimits;
use ratcore::action_registry::KeyModifier;
use ratcore::action_registry::KeySequence;
use ratcore::action_registry::LogicalKey;
use ratcore::action_registry::LogicalKeystroke;

use super::LocalSequenceResolution;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TestMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestAction {
    MoveUp,
    MoveDown,
    Submit,
    Quit,
}

fn stroke(key: LogicalKey, modifiers: Vec<KeyModifier>) -> LogicalKeystroke {
    LogicalKeystroke::try_new(key, modifiers).expect("test stroke is valid")
}

fn test_parse_action(value: &str) -> Option<TestAction> {
    match value {
        "move_up" => Some(TestAction::MoveUp),
        "move_down" => Some(TestAction::MoveDown),
        "submit" => Some(TestAction::Submit),
        "quit" => Some(TestAction::Quit),
        _ => None,
    }
}

#[test]
fn resolves_only_equal_logical_identity_in_the_requested_mode() {
    let mut keymap = super::Map::new();
    let up = stroke(LogicalKey::Character('k'), Vec::new());
    keymap.set(TestMode::Normal, up.clone(), TestAction::MoveUp);

    assert_eq!(keymap.resolve(&TestMode::Normal, &up), Some(TestAction::MoveUp));
    assert_eq!(keymap.resolve(&TestMode::Insert, &up), None);
    assert_eq!(keymap.resolve(&TestMode::Normal, &stroke(LogicalKey::Character('j'), Vec::new())), None);
}

#[test]
fn builds_with_valid_overrides_and_rejects_malformed_entries() {
    let mode_bindings = vec![
        (TestMode::Normal, {
            let mut map = std::collections::HashMap::new();
            map.insert(stroke(LogicalKey::Character('k'), Vec::new()), TestAction::MoveUp);
            map.insert(stroke(LogicalKey::Character('j'), Vec::new()), TestAction::MoveDown);
            map
        }),
        (TestMode::Insert, {
            let mut map = std::collections::HashMap::new();
            map.insert(stroke(LogicalKey::Enter, Vec::new()), TestAction::Submit);
            map
        }),
    ];
    let overrides = vec![(TestMode::Normal, {
        let mut map = std::collections::HashMap::new();
        map.insert("q".to_string(), "quit".to_string());
        map.insert("Hyper+x".to_string(), "quit".to_string());
        map.insert("x".to_string(), "not-an-action".to_string());
        map
    })];

    let keymap = super::Map::build(mode_bindings, &overrides, test_parse_action);
    assert_eq!(
        keymap.resolve(&TestMode::Normal, &stroke(LogicalKey::Character('q'), Vec::new())),
        Some(TestAction::Quit)
    );
    assert_eq!(keymap.resolve(&TestMode::Insert, &stroke(LogicalKey::Enter, Vec::new())), Some(TestAction::Submit));
    assert_eq!(keymap.mode_binding_count(&TestMode::Normal), 3);
}

#[test]
fn descriptions_are_deterministic_and_use_canonical_modifier_order() {
    let mut keymap = super::Map::new();
    keymap.set(TestMode::Normal, stroke(LogicalKey::Character('q'), vec![KeyModifier::Control]), TestAction::Quit);
    keymap.set(TestMode::Normal, stroke(LogicalKey::Character('k'), Vec::new()), TestAction::MoveUp);

    assert_eq!(keymap.describe(&TestMode::Normal), vec![
        ("Ctrl+q".to_string(), TestAction::Quit),
        ("k".to_string(), TestAction::MoveUp),
    ]);
    assert!(keymap.describe(&TestMode::Insert).is_empty());
}

#[test]
fn multi_stroke_sequences_are_handed_to_the_layered_resolver() {
    let local = stroke(LogicalKey::Character('k'), Vec::new());
    let continuation = stroke(LogicalKey::Character('c'), Vec::new());
    let one = KeySequence::try_new(vec![local.clone()], &BindingCatalogLimits::default()).expect("one stroke sequence");
    let two = KeySequence::try_new(vec![local.clone(), continuation], &BindingCatalogLimits::default())
        .expect("two stroke sequence");
    let mut keymap = super::Map::new();
    keymap.set(TestMode::Normal, local, TestAction::MoveUp);

    assert_eq!(
        keymap.resolve_local_sequence(&TestMode::Normal, &one),
        LocalSequenceResolution::Local(TestAction::MoveUp)
    );
    assert_eq!(
        keymap.resolve_local_sequence(&TestMode::Normal, &two),
        LocalSequenceResolution::LayeredResolutionRequired
    );
}

#[test]
fn modes_and_clear_are_bounded_local_state() {
    let mut keymap = super::Map::new();
    assert!(!keymap.has_mode(&TestMode::Normal));
    keymap.set(TestMode::Normal, stroke(LogicalKey::Character('k'), Vec::new()), TestAction::MoveUp);
    assert!(keymap.has_mode(&TestMode::Normal));
    assert_eq!(keymap.modes().count(), 1);
    keymap.clear_mode(&TestMode::Normal);
    assert!(!keymap.has_mode(&TestMode::Normal));
}
