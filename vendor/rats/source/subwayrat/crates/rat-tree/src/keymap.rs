//! Tree navigation keymap implementation.
//!
//! This module provides `TreeAction` variants for tree navigation and a default
//! vim-style keymap for tree widgets. The keymap uses `rat_keymap::Keymap<TreeAction, ()>`
//! where the unit type `()` represents a single mode.

use rat_keymap::{KeyCombo, Keymap};
use ratatui::crossterm::event::KeyCode;
use std::collections::HashMap;

/// Actions for tree navigation and interaction.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TreeAction {
    /// Move cursor up one item
    Up,
    /// Move cursor down one item
    Down,
    /// Expand the current node (if it has children)
    Expand,
    /// Collapse the current node (if it's expanded)
    Collapse,
    /// Toggle expand/collapse state of current node
    Toggle,
    /// Move to parent node
    Parent,
    /// Move to first child of current node
    FirstChild,
    /// Move to next sibling at same level
    NextSibling,
    /// Move to previous sibling at same level
    PrevSibling,
    /// Move to first item in tree
    First,
    /// Move to last item in tree
    Last,
    /// Page up (move up by page size)
    PageUp,
    /// Page down (move down by page size)
    PageDown,
    /// Select current item (consumers handle leaf vs expand contextually)
    Select,
}

/// Parse a snake_case action string into a TreeAction variant.
///
/// # Examples
///
/// ```rust,no_run
/// use rat_tree::keymap::{parse_tree_action, TreeAction};
///
/// assert_eq!(parse_tree_action("up"), Some(TreeAction::Up));
/// assert_eq!(parse_tree_action("next_sibling"), Some(TreeAction::NextSibling));
/// assert_eq!(parse_tree_action("first_child"), Some(TreeAction::FirstChild));
/// assert_eq!(parse_tree_action("unknown"), None);
/// ```
pub fn parse_tree_action(s: &str) -> Option<TreeAction> {
    match s {
        "up" => Some(TreeAction::Up),
        "down" => Some(TreeAction::Down),
        "expand" => Some(TreeAction::Expand),
        "collapse" => Some(TreeAction::Collapse),
        "toggle" => Some(TreeAction::Toggle),
        "parent" => Some(TreeAction::Parent),
        "first_child" => Some(TreeAction::FirstChild),
        "next_sibling" => Some(TreeAction::NextSibling),
        "prev_sibling" => Some(TreeAction::PrevSibling),
        "first" => Some(TreeAction::First),
        "last" => Some(TreeAction::Last),
        "page_up" => Some(TreeAction::PageUp),
        "page_down" => Some(TreeAction::PageDown),
        "select" => Some(TreeAction::Select),
        _ => None,
    }
}

/// Create a default vim-style keymap for tree navigation.
///
/// # Keybindings
///
/// - `k`, `Up` → Up
/// - `j`, `Down` → Down
/// - `g` → First
/// - `G` (Shift+g) → Last
/// - `l`, `Right` → Expand
/// - `h`, `Left` → Collapse
/// - `Space` → Toggle
/// - `p` → Parent
/// - `Enter` → Select
/// - `J` (Shift+j) → NextSibling
/// - `K` (Shift+k) → PrevSibling
/// - `Ctrl+d`, `PageDown` → PageDown
/// - `Ctrl+u`, `PageUp` → PageUp
/// - `o` → FirstChild
///
/// # Examples
///
/// ```rust,no_run
/// use rat_tree::keymap::default_keymap;
/// use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
///
/// let keymap = default_keymap();
/// let event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
/// let action = keymap.resolve(&(), &event);
/// // action should be Some(TreeAction::Up)
/// ```
pub fn default_keymap() -> Keymap<TreeAction, ()> {
    let mut bindings = HashMap::new();

    // Up/Down navigation
    bindings.insert(
        KeyCombo::new(KeyCode::Char('k'), false, false, false),
        TreeAction::Up,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Up, false, false, false),
        TreeAction::Up,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Char('j'), false, false, false),
        TreeAction::Down,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Down, false, false, false),
        TreeAction::Down,
    );

    // First/Last
    bindings.insert(
        KeyCombo::new(KeyCode::Char('g'), false, false, false),
        TreeAction::First,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Char('G'), false, false, true),
        TreeAction::Last,
    );

    // Expand/Collapse
    bindings.insert(
        KeyCombo::new(KeyCode::Char('l'), false, false, false),
        TreeAction::Expand,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Right, false, false, false),
        TreeAction::Expand,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Char('h'), false, false, false),
        TreeAction::Collapse,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Left, false, false, false),
        TreeAction::Collapse,
    );

    // Toggle
    bindings.insert(
        KeyCombo::new(KeyCode::Char(' '), false, false, false),
        TreeAction::Toggle,
    );

    // Parent
    bindings.insert(
        KeyCombo::new(KeyCode::Char('p'), false, false, false),
        TreeAction::Parent,
    );

    // Select
    bindings.insert(
        KeyCombo::new(KeyCode::Enter, false, false, false),
        TreeAction::Select,
    );

    // Sibling navigation
    bindings.insert(
        KeyCombo::new(KeyCode::Char('J'), false, false, true),
        TreeAction::NextSibling,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Char('K'), false, false, true),
        TreeAction::PrevSibling,
    );

    // Page navigation
    bindings.insert(
        KeyCombo::new(KeyCode::Char('d'), true, false, false),
        TreeAction::PageDown,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::PageDown, false, false, false),
        TreeAction::PageDown,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::Char('u'), true, false, false),
        TreeAction::PageUp,
    );
    bindings.insert(
        KeyCombo::new(KeyCode::PageUp, false, false, false),
        TreeAction::PageUp,
    );

    // First child
    bindings.insert(
        KeyCombo::new(KeyCode::Char('o'), false, false, false),
        TreeAction::FirstChild,
    );

    let mode_bindings = vec![((), bindings)];
    Keymap::build(mode_bindings, &[], parse_tree_action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn parse_tree_action_all_variants() {
        assert_eq!(parse_tree_action("up"), Some(TreeAction::Up));
        assert_eq!(parse_tree_action("down"), Some(TreeAction::Down));
        assert_eq!(parse_tree_action("expand"), Some(TreeAction::Expand));
        assert_eq!(parse_tree_action("collapse"), Some(TreeAction::Collapse));
        assert_eq!(parse_tree_action("toggle"), Some(TreeAction::Toggle));
        assert_eq!(parse_tree_action("parent"), Some(TreeAction::Parent));
        assert_eq!(
            parse_tree_action("first_child"),
            Some(TreeAction::FirstChild)
        );
        assert_eq!(
            parse_tree_action("next_sibling"),
            Some(TreeAction::NextSibling)
        );
        assert_eq!(
            parse_tree_action("prev_sibling"),
            Some(TreeAction::PrevSibling)
        );
        assert_eq!(parse_tree_action("first"), Some(TreeAction::First));
        assert_eq!(parse_tree_action("last"), Some(TreeAction::Last));
        assert_eq!(parse_tree_action("page_up"), Some(TreeAction::PageUp));
        assert_eq!(parse_tree_action("page_down"), Some(TreeAction::PageDown));
        assert_eq!(parse_tree_action("select"), Some(TreeAction::Select));
    }

    #[test]
    fn parse_tree_action_unknown() {
        assert_eq!(parse_tree_action("unknown"), None);
        assert_eq!(parse_tree_action("invalid_action"), None);
        assert_eq!(parse_tree_action(""), None);
        assert_eq!(parse_tree_action("Up"), None); // case sensitive
    }

    #[test]
    fn default_keymap_resolves_expected_keys() {
        let keymap = default_keymap();

        // Test basic navigation
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
            Some(TreeAction::Up)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Some(TreeAction::Up)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
            Some(TreeAction::Down)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Some(TreeAction::Down)
        );

        // Test first/last
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE)),
            Some(TreeAction::First)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT)),
            Some(TreeAction::Last)
        );

        // Test expand/collapse
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
            Some(TreeAction::Expand)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
            Some(TreeAction::Expand)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE)),
            Some(TreeAction::Collapse)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
            Some(TreeAction::Collapse)
        );

        // Test toggle
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            Some(TreeAction::Toggle)
        );

        // Test parent
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE)),
            Some(TreeAction::Parent)
        );

        // Test select
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some(TreeAction::Select)
        );

        // Test sibling navigation
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('J'), KeyModifiers::SHIFT)),
            Some(TreeAction::NextSibling)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT)),
            Some(TreeAction::PrevSibling)
        );

        // Test page navigation
        assert_eq!(
            keymap.resolve(
                &(),
                &KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL)
            ),
            Some(TreeAction::PageDown)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
            Some(TreeAction::PageDown)
        );
        assert_eq!(
            keymap.resolve(
                &(),
                &KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL)
            ),
            Some(TreeAction::PageUp)
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
            Some(TreeAction::PageUp)
        );

        // Test first child
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE)),
            Some(TreeAction::FirstChild)
        );
    }

    #[test]
    fn default_keymap_unmapped_keys() {
        let keymap = default_keymap();

        // Test that unmapped keys return None
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
            None
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            None
        );
        assert_eq!(
            keymap.resolve(&(), &KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE)),
            None
        );
    }

    #[test]
    fn tree_action_derive_traits() {
        let action1 = TreeAction::Up;
        let action2 = TreeAction::Up.clone();
        let action3 = TreeAction::Down;

        // Test Clone
        assert_eq!(action1, action2);

        // Test PartialEq and Eq
        assert_eq!(action1, action2);
        assert_ne!(action1, action3);

        // Test Debug (just ensure it doesn't panic)
        let debug_str = format!("{:?}", action1);
        assert!(debug_str.contains("Up"));
    }

    #[test]
    fn keymap_has_all_expected_bindings() {
        let keymap = default_keymap();

        // Get all bindings and verify we have the expected number
        let bindings = keymap.describe(&());

        // We should have exactly 20 bindings based on the default_keymap function
        // (some keys like k/Up map to the same action, but we count unique KeyCombo -> Action pairs)
        assert_eq!(bindings.len(), 20);

        // Verify that all TreeAction variants can be triggered
        let actions: std::collections::HashSet<_> =
            bindings.iter().map(|(_, action)| action).collect();

        // All 14 TreeAction variants should be reachable
        assert_eq!(actions.len(), 14);
        assert!(actions.contains(&TreeAction::Up));
        assert!(actions.contains(&TreeAction::Down));
        assert!(actions.contains(&TreeAction::Expand));
        assert!(actions.contains(&TreeAction::Collapse));
        assert!(actions.contains(&TreeAction::Toggle));
        assert!(actions.contains(&TreeAction::Parent));
        assert!(actions.contains(&TreeAction::FirstChild));
        assert!(actions.contains(&TreeAction::NextSibling));
        assert!(actions.contains(&TreeAction::PrevSibling));
        assert!(actions.contains(&TreeAction::First));
        assert!(actions.contains(&TreeAction::Last));
        assert!(actions.contains(&TreeAction::PageUp));
        assert!(actions.contains(&TreeAction::PageDown));
        assert!(actions.contains(&TreeAction::Select));
    }
}
