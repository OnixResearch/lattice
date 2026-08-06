//! Backend-neutral tree navigation bindings.

use std::collections::HashMap;

use rat_keymap::Keymap;
use ratcore::action_registry::KeyModifier;
use ratcore::action_registry::LogicalKey;
use ratcore::action_registry::LogicalKeystroke;

/// Actions for tree navigation and interaction.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TreeAction {
    /// Move cursor up one item.
    Up,
    /// Move cursor down one item.
    Down,
    /// Expand the current node.
    Expand,
    /// Collapse the current node.
    Collapse,
    /// Toggle current expansion.
    Toggle,
    /// Move to the parent.
    Parent,
    /// Move to the first child.
    FirstChild,
    /// Move to the next sibling.
    NextSibling,
    /// Move to the previous sibling.
    PrevSibling,
    /// Move to the first item.
    First,
    /// Move to the last item.
    Last,
    /// Move up by one page.
    PageUp,
    /// Move down by one page.
    PageDown,
    /// Select the current item.
    Select,
}

/// Parses one stable tree action name.
#[must_use]
pub fn parse_tree_action(value: &str) -> Option<TreeAction> {
    match value {
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

fn stroke(key: LogicalKey, modifiers: Vec<KeyModifier>) -> LogicalKeystroke {
    LogicalKeystroke::try_new(key, modifiers).expect("built-in tree bindings are valid logical strokes")
}

/// Creates the deterministic vim-style logical keymap for tree navigation.
///
/// The returned map accepts only normalized [`LogicalKeystroke`] values. A
/// Crossterm, browser, Dioxus, Ratzilla, or graphical event must be converted by
/// its owning adapter first.
#[must_use]
pub fn default_keymap() -> Keymap<TreeAction, ()> {
    use KeyModifier::Control;
    use KeyModifier::Shift;
    use LogicalKey::ArrowDown;
    use LogicalKey::ArrowLeft;
    use LogicalKey::ArrowRight;
    use LogicalKey::ArrowUp;
    use LogicalKey::Character;
    use LogicalKey::Enter;
    use LogicalKey::PageDown;
    use LogicalKey::PageUp;
    use LogicalKey::Space;

    let bindings = HashMap::from([
        (stroke(Character('k'), Vec::new()), TreeAction::Up),
        (stroke(ArrowUp, Vec::new()), TreeAction::Up),
        (stroke(Character('j'), Vec::new()), TreeAction::Down),
        (stroke(ArrowDown, Vec::new()), TreeAction::Down),
        (stroke(Character('g'), Vec::new()), TreeAction::First),
        (stroke(Character('g'), vec![Shift]), TreeAction::Last),
        (stroke(Character('l'), Vec::new()), TreeAction::Expand),
        (stroke(ArrowRight, Vec::new()), TreeAction::Expand),
        (stroke(Character('h'), Vec::new()), TreeAction::Collapse),
        (stroke(ArrowLeft, Vec::new()), TreeAction::Collapse),
        (stroke(Space, Vec::new()), TreeAction::Toggle),
        (stroke(Character('p'), Vec::new()), TreeAction::Parent),
        (stroke(Enter, Vec::new()), TreeAction::Select),
        (stroke(Character('j'), vec![Shift]), TreeAction::NextSibling),
        (stroke(Character('k'), vec![Shift]), TreeAction::PrevSibling),
        (stroke(Character('d'), vec![Control]), TreeAction::PageDown),
        (stroke(PageDown, Vec::new()), TreeAction::PageDown),
        (stroke(Character('u'), vec![Control]), TreeAction::PageUp),
        (stroke(PageUp, Vec::new()), TreeAction::PageUp),
        (stroke(Character('o'), Vec::new()), TreeAction::FirstChild),
    ]);
    Keymap::build(vec![((), bindings)], &[], parse_tree_action)
}

#[cfg(test)]
mod tests {
    use ratcore::action_registry::RendererKeyCapabilities;
    use ratcore::action_registry::RendererKeyInput;
    use ratcore::action_registry::normalize_renderer_key_input;

    use super::*;

    #[test]
    fn parses_every_action_and_rejects_unknown_names() {
        let names = [
            "up",
            "down",
            "expand",
            "collapse",
            "toggle",
            "parent",
            "first_child",
            "next_sibling",
            "prev_sibling",
            "first",
            "last",
            "page_up",
            "page_down",
            "select",
        ];
        assert!(names.into_iter().all(|name| parse_tree_action(name).is_some()));
        assert!(parse_tree_action("Up").is_none());
        assert!(parse_tree_action("unknown").is_none());
    }

    #[test]
    fn defaults_preserve_actions_labels_and_binding_count() {
        let keymap = default_keymap();
        let expected = [
            ("k", TreeAction::Up),
            ("Up", TreeAction::Up),
            ("j", TreeAction::Down),
            ("Down", TreeAction::Down),
            ("g", TreeAction::First),
            ("Shift+G", TreeAction::Last),
            ("l", TreeAction::Expand),
            ("Right", TreeAction::Expand),
            ("h", TreeAction::Collapse),
            ("Left", TreeAction::Collapse),
            ("Space", TreeAction::Toggle),
            ("p", TreeAction::Parent),
            ("Enter", TreeAction::Select),
            ("Shift+J", TreeAction::NextSibling),
            ("Shift+K", TreeAction::PrevSibling),
            ("Ctrl+d", TreeAction::PageDown),
            ("PageDown", TreeAction::PageDown),
            ("Ctrl+u", TreeAction::PageUp),
            ("PageUp", TreeAction::PageUp),
            ("o", TreeAction::FirstChild),
        ];

        assert_eq!(keymap.describe(&()).len(), expected.len());
        for (label, action) in expected {
            let logical = rat_keymap::parse_key_string(label).expect("default label parses");
            assert_eq!(keymap.resolve(&(), &logical), Some(action));
        }
        assert_eq!(keymap.resolve(&(), &stroke(LogicalKey::Character('x'), Vec::new())), None);
    }

    #[test]
    fn equivalent_terminal_and_browser_facts_drive_identical_tree_action() {
        let input = RendererKeyInput::new("k", Vec::new());
        let terminal =
            normalize_renderer_key_input(&input, &RendererKeyCapabilities::subwayrat()).expect("terminal stroke");
        let browser =
            normalize_renderer_key_input(&input, &RendererKeyCapabilities::meteorite()).expect("browser stroke");
        let keymap = default_keymap();

        assert_eq!(terminal, browser);
        assert_eq!(keymap.resolve(&(), &terminal), Some(TreeAction::Up));
        assert_eq!(keymap.resolve(&(), &browser), Some(TreeAction::Up));
        assert_eq!(keymap.describe(&()), keymap.describe(&()));
    }
}
