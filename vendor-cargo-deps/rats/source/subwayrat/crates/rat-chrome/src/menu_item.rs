//! Shared menu item model and keyboard navigation helpers.
//!
//! Menu bars and context menus both need to skip separators, ignore disabled
//! entries, and enter/leave submenu levels. `MenuNav` keeps that pure state so
//! renderers and input adapters can stay thin.

/// A selectable or structural entry in a menu list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    /// A leaf action that may be activated by the consumer.
    Action {
        /// User-visible label.
        label: String,
        /// Optional accelerator hint such as `Ctrl+O`.
        accel: Option<String>,
        /// Disabled actions are rendered but skipped by navigation.
        enabled: bool,
    },
    /// A nested submenu with child entries.
    Submenu {
        /// User-visible label.
        label: String,
        /// Optional accelerator hint such as `Alt+F`.
        accel: Option<String>,
        /// Disabled submenus are rendered but skipped by navigation.
        enabled: bool,
        /// Child entries shown when the submenu is entered.
        children: Vec<MenuItem>,
    },
    /// Non-selectable visual separator.
    Separator,
}

impl MenuItem {
    /// Create an enabled action item.
    pub fn action(label: impl Into<String>) -> Self {
        Self::Action {
            label: label.into(),
            accel: None,
            enabled: true,
        }
    }

    /// Create an enabled action item with an accelerator hint.
    pub fn action_with_accel(label: impl Into<String>, accel: impl Into<String>) -> Self {
        Self::Action {
            label: label.into(),
            accel: Some(accel.into()),
            enabled: true,
        }
    }

    /// Create a disabled action item.
    pub fn disabled_action(label: impl Into<String>) -> Self {
        Self::Action {
            label: label.into(),
            accel: None,
            enabled: false,
        }
    }

    /// Create an enabled submenu item.
    pub fn submenu(label: impl Into<String>, children: Vec<MenuItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            accel: None,
            enabled: true,
            children,
        }
    }

    /// Create an enabled submenu item with an accelerator hint.
    pub fn submenu_with_accel(
        label: impl Into<String>,
        accel: impl Into<String>,
        children: Vec<MenuItem>,
    ) -> Self {
        Self::Submenu {
            label: label.into(),
            accel: Some(accel.into()),
            enabled: true,
            children,
        }
    }

    /// Create a disabled submenu item.
    pub fn disabled_submenu(label: impl Into<String>, children: Vec<MenuItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            accel: None,
            enabled: false,
            children,
        }
    }

    /// Create a separator entry.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Return the user-visible label, when this item has one.
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Action { label, .. } | Self::Submenu { label, .. } => Some(label.as_str()),
            Self::Separator => None,
        }
    }

    /// Return the accelerator hint, when present.
    pub fn accel(&self) -> Option<&str> {
        match self {
            Self::Action { accel, .. } | Self::Submenu { accel, .. } => accel.as_deref(),
            Self::Separator => None,
        }
    }

    /// Return submenu children, when this item is a submenu.
    pub fn children(&self) -> Option<&[MenuItem]> {
        match self {
            Self::Submenu { children, .. } => Some(children.as_slice()),
            Self::Action { .. } | Self::Separator => None,
        }
    }

    /// True when this item is a separator.
    pub fn is_separator(&self) -> bool {
        matches!(self, Self::Separator)
    }

    /// True when this item is enabled.
    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Action { enabled, .. } | Self::Submenu { enabled, .. } => *enabled,
            Self::Separator => false,
        }
    }

    /// True when keyboard navigation may focus this item.
    pub fn is_selectable(&self) -> bool {
        self.is_enabled() && !self.is_separator()
    }

    /// True when this item is an enabled submenu with selectable children.
    pub fn can_enter(&self) -> bool {
        match self {
            Self::Submenu {
                enabled, children, ..
            } => *enabled && first_selectable_index(children).is_some(),
            Self::Action { .. } | Self::Separator => false,
        }
    }
}

/// Pure keyboard navigation state for menu-like lists.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MenuNav {
    focused: Option<usize>,
    submenu_path: Vec<usize>,
}

impl MenuNav {
    /// Create navigation state with no focused item.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the focused index in the current menu level.
    pub fn focused(&self) -> Option<usize> {
        self.focused
    }

    /// Return the open submenu path from the root menu to the current level.
    pub fn submenu_path(&self) -> &[usize] {
        self.submenu_path.as_slice()
    }

    /// Clear focus and close all submenu levels.
    pub fn clear(&mut self) {
        self.focused = None;
        self.submenu_path.clear();
    }

    /// Clear only the current focused item.
    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    /// Focus the first selectable item in `items`.
    pub fn focus_first(&mut self, items: &[MenuItem]) -> Option<usize> {
        self.focused = first_selectable_index(items);
        self.focused
    }

    /// Try to focus a specific selectable index.
    pub fn set_focus(&mut self, items: &[MenuItem], index: usize) -> bool {
        if items.get(index).is_some_and(MenuItem::is_selectable) {
            self.focused = Some(index);
            true
        } else {
            false
        }
    }

    /// Move focus to the next selectable item, wrapping at the end.
    pub fn move_down(&mut self, items: &[MenuItem]) -> Option<usize> {
        self.focused = next_selectable_index(items, self.focused, Direction::Forward);
        self.focused
    }

    /// Move focus to the previous selectable item, wrapping at the start.
    pub fn move_up(&mut self, items: &[MenuItem]) -> Option<usize> {
        self.focused = next_selectable_index(items, self.focused, Direction::Backward);
        self.focused
    }

    /// Return the focused item in `items`.
    pub fn selected_item<'a>(&self, items: &'a [MenuItem]) -> Option<&'a MenuItem> {
        self.focused.and_then(|index| items.get(index))
    }

    /// Return the current item list after following the open submenu path.
    pub fn current_items<'a>(&self, root_items: &'a [MenuItem]) -> Option<&'a [MenuItem]> {
        items_at_path(root_items, &self.submenu_path)
    }

    /// Return the focused item in the current submenu level.
    pub fn current_item<'a>(&self, root_items: &'a [MenuItem]) -> Option<&'a MenuItem> {
        self.current_items(root_items)
            .and_then(|items| self.selected_item(items))
    }

    /// Enter the focused submenu and focus its first selectable child.
    pub fn enter_submenu(&mut self, root_items: &[MenuItem]) -> bool {
        let Some(current_items) = self.current_items(root_items) else {
            return false;
        };
        let Some(focused_index) = self.focused else {
            return false;
        };
        let Some(MenuItem::Submenu {
            enabled, children, ..
        }) = current_items.get(focused_index)
        else {
            return false;
        };

        if !*enabled {
            return false;
        }

        let Some(child_index) = first_selectable_index(children) else {
            return false;
        };

        self.submenu_path.push(focused_index);
        self.focused = Some(child_index);
        true
    }

    /// Leave the current submenu and restore focus to its parent item.
    pub fn leave_submenu(&mut self) -> bool {
        let Some(parent_index) = self.submenu_path.pop() else {
            return false;
        };

        self.focused = Some(parent_index);
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Forward,
    Backward,
}

fn first_selectable_index(items: &[MenuItem]) -> Option<usize> {
    items.iter().position(MenuItem::is_selectable)
}

fn next_selectable_index(
    items: &[MenuItem],
    current: Option<usize>,
    direction: Direction,
) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let start = match current {
        Some(index) if index < items.len() => index,
        Some(_) | None => return first_selectable_index(items),
    };

    let mut candidate = start;
    for _ in 0..items.len() {
        candidate = match direction {
            Direction::Forward => (candidate + 1) % items.len(),
            Direction::Backward => {
                if candidate == 0 {
                    items.len() - 1
                } else {
                    candidate - 1
                }
            }
        };

        if items[candidate].is_selectable() {
            return Some(candidate);
        }
    }

    None
}

fn items_at_path<'a>(root_items: &'a [MenuItem], path: &[usize]) -> Option<&'a [MenuItem]> {
    let mut items = root_items;

    for &index in path {
        let Some(MenuItem::Submenu { children, .. }) = items.get(index) else {
            return None;
        };
        items = children.as_slice();
    }

    Some(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_SELECTABLE: usize = 2;
    const SECOND_SELECTABLE: usize = 3;
    const SUBMENU_INDEX: usize = 0;
    const CHILD_ACTION_INDEX: usize = 0;
    const CHILD_EXIT_INDEX: usize = 2;
    const OUT_OF_RANGE_INDEX: usize = 99;

    #[test]
    fn navigation_skips_separators_and_disabled_items() {
        let items = vec![
            MenuItem::separator(),
            MenuItem::disabled_action("Disabled"),
            MenuItem::action("Open"),
            MenuItem::action("Save"),
        ];
        let mut nav = MenuNav::new();

        assert_eq!(nav.focus_first(&items), Some(FIRST_SELECTABLE));
        assert_eq!(nav.move_down(&items), Some(SECOND_SELECTABLE));
        assert_eq!(nav.move_down(&items), Some(FIRST_SELECTABLE));
        assert_eq!(nav.move_up(&items), Some(SECOND_SELECTABLE));
        assert_eq!(
            nav.selected_item(&items).and_then(MenuItem::label),
            Some("Save")
        );
    }

    #[test]
    fn submenu_enter_and_leave_tracks_path() {
        let items = vec![MenuItem::submenu(
            "File",
            vec![
                MenuItem::action("Open"),
                MenuItem::separator(),
                MenuItem::action("Exit"),
            ],
        )];
        let mut nav = MenuNav::new();

        assert_eq!(nav.focus_first(&items), Some(SUBMENU_INDEX));
        assert!(nav.enter_submenu(&items));
        assert_eq!(nav.submenu_path(), &[SUBMENU_INDEX]);
        assert_eq!(nav.focused(), Some(CHILD_ACTION_INDEX));

        let current_items = nav.current_items(&items).unwrap_or_default();
        assert_eq!(nav.move_down(current_items), Some(CHILD_EXIT_INDEX));
        assert_eq!(
            nav.current_item(&items).and_then(MenuItem::label),
            Some("Exit")
        );

        assert!(nav.leave_submenu());
        assert_eq!(nav.submenu_path(), &[]);
        assert_eq!(nav.focused(), Some(SUBMENU_INDEX));
    }

    #[test]
    fn set_focus_rejects_separator_disabled_and_out_of_range_indices() {
        let items = vec![
            MenuItem::separator(),
            MenuItem::disabled_action("Disabled"),
            MenuItem::action("Open"),
        ];
        let mut nav = MenuNav::new();

        assert!(!nav.set_focus(&items, 0));
        assert!(!nav.set_focus(&items, 1));
        assert!(!nav.set_focus(&items, OUT_OF_RANGE_INDEX));
        assert_eq!(nav.focused(), None);

        assert!(nav.set_focus(&items, FIRST_SELECTABLE));
        assert_eq!(nav.focused(), Some(FIRST_SELECTABLE));
    }

    #[test]
    fn empty_menu_has_no_focus_and_does_not_panic() {
        let items = Vec::new();
        let mut nav = MenuNav::new();

        assert_eq!(nav.focus_first(&items), None);
        assert_eq!(nav.move_down(&items), None);
        assert_eq!(nav.move_up(&items), None);
        assert_eq!(nav.selected_item(&items), None);
        assert!(!nav.leave_submenu());
    }

    #[test]
    fn all_disabled_menu_has_no_focus() {
        let items = vec![
            MenuItem::separator(),
            MenuItem::disabled_action("Disabled"),
            MenuItem::disabled_submenu("More", vec![MenuItem::action("Child")]),
        ];
        let mut nav = MenuNav::new();

        assert_eq!(nav.focus_first(&items), None);
        assert_eq!(nav.move_down(&items), None);
        assert_eq!(nav.move_up(&items), None);
        assert_eq!(nav.focused(), None);
    }

    #[test]
    fn enter_submenu_rejects_actions_disabled_submenus_and_empty_submenus() {
        let action_items = vec![MenuItem::action("Open")];
        let disabled_items = vec![MenuItem::disabled_submenu(
            "More",
            vec![MenuItem::action("Child")],
        )];
        let empty_submenu_items = vec![MenuItem::submenu("More", vec![MenuItem::separator()])];
        let mut nav = MenuNav::new();

        assert_eq!(nav.focus_first(&action_items), Some(CHILD_ACTION_INDEX));
        assert!(!nav.enter_submenu(&action_items));
        assert_eq!(nav.submenu_path(), &[]);

        assert_eq!(nav.focus_first(&disabled_items), None);
        assert!(!nav.enter_submenu(&disabled_items));
        assert_eq!(nav.submenu_path(), &[]);

        assert_eq!(nav.focus_first(&empty_submenu_items), Some(SUBMENU_INDEX));
        assert!(!nav.enter_submenu(&empty_submenu_items));
        assert_eq!(nav.submenu_path(), &[]);
    }
}
