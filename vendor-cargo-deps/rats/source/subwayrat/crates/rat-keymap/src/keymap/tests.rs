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

const EXPECTED_DESCRIBED_BINDINGS: usize = 2;
const EMPTY_MODE_COUNT: usize = 0;
const ONE_MODE_COUNT: usize = 1;
const EMPTY_BINDING_COUNT: u64 = 0;
const ONE_BINDING_COUNT: u64 = 1;

fn test_parse_action(s: &str) -> Option<TestAction> {
    match s {
        "move_up" => Some(TestAction::MoveUp),
        "move_down" => Some(TestAction::MoveDown),
        "submit" => Some(TestAction::Submit),
        "quit" => Some(TestAction::Quit),
        _ => None,
    }
}

#[test]
fn basic_resolve() {
    let mut keymap = super::Map::new();
    keymap.set(
        TestMode::Normal,
        crate::combo::Chord::new(crate::combo::Code::Char('k'), false, false, false),
        TestAction::MoveUp,
    );

    let event =
        crate::combo::Event::new(crate::combo::Code::Char('k'), crate::combo::Modifiers::NONE);
    assert_eq!(
        keymap.resolve(&TestMode::Normal, &event),
        Some(TestAction::MoveUp)
    );

    // Different mode should return None.
    assert_eq!(keymap.resolve(&TestMode::Insert, &event), None);

    // Different key should return None.
    let other_event =
        crate::combo::Event::new(crate::combo::Code::Char('j'), crate::combo::Modifiers::NONE);
    assert_eq!(keymap.resolve(&TestMode::Normal, &other_event), None);
}

#[test]
fn build_with_overrides() {
    let mode_bindings = vec![
        (TestMode::Normal, {
            let mut map = std::collections::HashMap::new();
            map.insert(
                crate::combo::Chord::new(crate::combo::Code::Char('k'), false, false, false),
                TestAction::MoveUp,
            );
            map.insert(
                crate::combo::Chord::new(crate::combo::Code::Char('j'), false, false, false),
                TestAction::MoveDown,
            );
            map
        }),
        (TestMode::Insert, {
            let mut map = std::collections::HashMap::new();
            map.insert(
                crate::combo::Chord::new(crate::combo::Code::Enter, false, false, false),
                TestAction::Submit,
            );
            map
        }),
    ];

    let overrides = vec![(TestMode::Normal, {
        let mut map = std::collections::HashMap::new();
        map.insert("q".to_string(), "quit".to_string());
        map
    })];

    let keymap = super::Map::build(mode_bindings, &overrides, test_parse_action);

    // Test original binding.
    let k_event =
        crate::combo::Event::new(crate::combo::Code::Char('k'), crate::combo::Modifiers::NONE);
    assert_eq!(
        keymap.resolve(&TestMode::Normal, &k_event),
        Some(TestAction::MoveUp)
    );

    // Test override.
    let q_event =
        crate::combo::Event::new(crate::combo::Code::Char('q'), crate::combo::Modifiers::NONE);
    assert_eq!(
        keymap.resolve(&TestMode::Normal, &q_event),
        Some(TestAction::Quit)
    );

    // Test insert mode binding.
    let enter_event =
        crate::combo::Event::new(crate::combo::Code::Enter, crate::combo::Modifiers::NONE);
    assert_eq!(
        keymap.resolve(&TestMode::Insert, &enter_event),
        Some(TestAction::Submit)
    );
}

#[test]
fn build_ignores_invalid_override_key_and_action() {
    let mode_bindings = vec![(TestMode::Normal, std::collections::HashMap::new())];
    let overrides = vec![(TestMode::Normal, {
        let mut map = std::collections::HashMap::new();
        map.insert("not-a-real-key".to_string(), "quit".to_string());
        map.insert("q".to_string(), "not-a-real-action".to_string());
        map
    })];

    let keymap = super::Map::build(mode_bindings, &overrides, test_parse_action);
    let q_event =
        crate::combo::Event::new(crate::combo::Code::Char('q'), crate::combo::Modifiers::NONE);

    assert_eq!(keymap.resolve(&TestMode::Normal, &q_event), None);
    assert_eq!(
        keymap.mode_binding_count(&TestMode::Normal),
        EMPTY_BINDING_COUNT
    );
}

#[test]
fn describe() {
    let mut keymap = super::Map::new();
    keymap.set(
        TestMode::Normal,
        crate::combo::Chord::new(crate::combo::Code::Char('k'), false, false, false),
        TestAction::MoveUp,
    );
    keymap.set(
        TestMode::Normal,
        crate::combo::Chord::new(crate::combo::Code::Char('q'), true, false, false),
        TestAction::Quit,
    );

    let bindings = keymap.describe(&TestMode::Normal);
    assert_eq!(bindings.len(), EXPECTED_DESCRIBED_BINDINGS);

    // Check that we have the expected bindings (order may vary due to HashMap).
    let binding_map: std::collections::HashMap<String, TestAction> = bindings.into_iter().collect();
    assert_eq!(binding_map.get("k"), Some(&TestAction::MoveUp));
    assert_eq!(binding_map.get("Ctrl+q"), Some(&TestAction::Quit));

    // Empty mode should return empty list.
    let empty_bindings = keymap.describe(&TestMode::Insert);
    assert!(empty_bindings.is_empty());
}

#[test]
fn mode_management() {
    let mut keymap = super::Map::new();

    // Initially no modes.
    assert_eq!(keymap.modes().count(), EMPTY_MODE_COUNT);
    assert!(!keymap.has_mode(&TestMode::Normal));
    assert_eq!(
        keymap.mode_binding_count(&TestMode::Normal),
        EMPTY_BINDING_COUNT
    );

    // Add a binding.
    keymap.set(
        TestMode::Normal,
        crate::combo::Chord::new(crate::combo::Code::Char('k'), false, false, false),
        TestAction::MoveUp,
    );

    assert_eq!(keymap.modes().count(), ONE_MODE_COUNT);
    assert!(keymap.has_mode(&TestMode::Normal));
    assert_eq!(
        keymap.mode_binding_count(&TestMode::Normal),
        ONE_BINDING_COUNT
    );

    // Clear the mode.
    keymap.clear_mode(&TestMode::Normal);
    assert_eq!(keymap.modes().count(), EMPTY_MODE_COUNT);
    assert!(!keymap.has_mode(&TestMode::Normal));
}

#[test]
fn modifier_keys() {
    let mut keymap = super::Map::new();
    keymap.set(
        TestMode::Normal,
        crate::combo::Chord::new(crate::combo::Code::Char('c'), true, false, false),
        TestAction::Quit,
    );

    let ctrl_c = crate::combo::Event::new(
        crate::combo::Code::Char('c'),
        crate::combo::Modifiers::CONTROL,
    );
    assert_eq!(
        keymap.resolve(&TestMode::Normal, &ctrl_c),
        Some(TestAction::Quit)
    );

    let plain_c =
        crate::combo::Event::new(crate::combo::Code::Char('c'), crate::combo::Modifiers::NONE);
    assert_eq!(keymap.resolve(&TestMode::Normal, &plain_c), None);
}
