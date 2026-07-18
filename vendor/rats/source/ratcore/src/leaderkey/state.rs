use super::*;

/// Leader key menu state and navigation.
///
/// Generic over `A`, the application's action type.
pub struct LeaderMenu<A> {
    /// Whether the overlay is visible.
    pub visible: bool,
    /// Stack of menu levels (root at bottom, current at top).
    stack: Vec<LeaderMenuDef<A>>,
    /// Breadcrumb labels for the title bar.
    breadcrumb: Vec<String>,
    /// All submenu definitions.
    submenus: Vec<LeaderMenuDef<A>>,
    /// The root menu definition.
    root: LeaderMenuDef<A>,
}

impl<A> LeaderMenu<A> {
    /// Create from pre-built parts (used by builders).
    pub(super) fn from_parts(root: LeaderMenuDef<A>, submenus: Vec<LeaderMenuDef<A>>) -> Self {
        Self {
            visible: false,
            stack: Vec::new(),
            breadcrumb: Vec::new(),
            submenus,
            root,
        }
    }

    /// The root menu definition.
    #[must_use]
    pub fn root_def(&self) -> &LeaderMenuDef<A> {
        &self.root
    }

    /// All submenu definitions.
    #[must_use]
    pub fn submenu_defs(&self) -> &[LeaderMenuDef<A>] {
        &self.submenus
    }

    /// The currently displayed menu level, or `None` when the menu is closed.
    #[must_use]
    pub fn current(&self) -> Option<&LeaderMenuDef<A>> {
        self.stack.last()
    }

    /// Breadcrumb trail for the title bar.
    #[must_use]
    pub fn breadcrumb(&self) -> &[String] {
        &self.breadcrumb
    }

    /// Current submenu depth (`0` = root, `1` = first submenu, etc.).
    #[must_use]
    pub fn depth(&self) -> u64 {
        self.stack.len().saturating_sub(1) as u64
    }

    /// Creates a menu from pre-built parts for platform wrappers and tests.
    #[must_use]
    pub fn test_from_parts(root: LeaderMenuDef<A>, submenus: Vec<LeaderMenuDef<A>>) -> Self {
        Self::from_parts(root, submenus)
    }

    /// Close the menu entirely.
    pub fn close(&mut self) {
        self.visible = false;
        self.stack.clear();
        self.breadcrumb.clear();
    }
}

impl<A: Clone> LeaderMenu<A> {
    /// Open the menu (shows root level).
    pub fn open(&mut self) {
        self.visible = true;
        self.stack.clear();
        self.breadcrumb.clear();
        self.stack.push(self.root.clone());
    }

    /// Handle a platform-neutral input event.
    ///
    /// Returns `Some(action)` if an action should be dispatched,
    /// `None` if the key was consumed internally (submenu nav, close).
    pub fn handle_input(&mut self, input: MenuInput) -> Option<LeaderAction<A>> {
        if !self.visible {
            return None;
        }

        match input {
            MenuInput::Escape => {
                if self.stack.len() > 1 {
                    self.stack.pop();
                    self.breadcrumb.pop();
                } else {
                    self.close();
                }
                None
            }
            MenuInput::Char(ch) => self.handle_char_inner(ch),
            MenuInput::Other => {
                self.close();
                None
            }
        }
    }

    /// Convenience: handle a single character directly.
    ///
    /// Equivalent to `handle_input(MenuInput::Char(ch))`.
    pub fn handle_char(&mut self, ch: char) -> Option<LeaderAction<A>> {
        self.handle_input(MenuInput::Char(ch))
    }

    fn handle_char_inner(&mut self, ch: char) -> Option<LeaderAction<A>> {
        let Some(current) = self.current() else {
            self.close();
            return None;
        };

        if let Some(item) = current.items.iter().find(|i| i.key == ch) {
            match &item.action {
                LeaderAction::Submenu(name) => {
                    if let Some(sub) = self.submenus.iter().find(|s| s.label == *name) {
                        self.breadcrumb.push(item.label.clone());
                        self.stack.push(sub.clone());
                    } else {
                        self.close();
                    }
                    None
                }
                action => {
                    let result = action.clone();
                    self.close();
                    Some(result)
                }
            }
        } else {
            // Unknown key → dismiss.
            self.close();
            None
        }
    }
}
