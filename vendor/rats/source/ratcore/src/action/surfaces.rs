use super::*;

/// Deterministic action registry snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRegistrySnapshot {
    pub(super) actions: Vec<ActionDescriptor>,
    pub(super) blocked_reasons: std::collections::BTreeMap<String, String>,
}

impl ActionRegistrySnapshot {
    /// Returns descriptors sorted by category, label, and ID.
    #[must_use]
    pub fn actions(&self) -> &[ActionDescriptor] {
        &self.actions
    }

    /// Finds an action by ID.
    #[must_use]
    pub fn action(&self, id: &str) -> Option<&ActionDescriptor> {
        self.actions.iter().find(|action| action.id == id)
    }

    /// Returns true when an action is enabled and not blocked by registry diagnostics.
    #[must_use]
    pub fn is_active(&self, action: &ActionDescriptor) -> bool {
        action.enablement.is_enabled() && !self.blocked_reasons.contains_key(&action.id)
    }

    /// Returns the inactive reason for a descriptor, when one exists.
    #[must_use]
    pub fn inactive_reason(&self, action: &ActionDescriptor) -> Option<String> {
        self.blocked_reasons
            .get(&action.id)
            .cloned()
            .or_else(|| action.enablement.disabled_reason().map(ToOwned::to_owned))
    }

    /// Projects active shortcut bindings for keymap-like consumers.
    #[must_use]
    pub fn keymap_bindings(&self) -> Vec<KeymapBindingProjection> {
        self.actions
            .iter()
            .filter(|action| self.is_active(action))
            .filter_map(|action| {
                let shortcut = action.shortcut.as_ref()?;
                Some(KeymapBindingProjection {
                    action_id: action.id.clone(),
                    label: action.label.clone(),
                    scope: shortcut.scope.clone(),
                    key: shortcut.key.clone(),
                    modifiers: shortcut.modifiers.clone(),
                })
            })
            .collect()
    }

    /// Projects active shortcut entries for leader-key-like consumers.
    #[must_use]
    pub fn leader_entries(&self) -> Vec<LeaderEntryProjection> {
        self.keymap_bindings()
            .into_iter()
            .map(|binding| LeaderEntryProjection {
                action_id: binding.action_id,
                label: binding.label,
                shortcut_label: shortcut_label(&binding.modifiers, &binding.key),
                scope: binding.scope,
            })
            .collect()
    }

    /// Projects all valid actions for command-palette-like consumers.
    #[must_use]
    pub fn command_palette_items(&self) -> Vec<CommandPaletteItemProjection> {
        self.actions
            .iter()
            .map(|action| CommandPaletteItemProjection {
                action_id: action.id.clone(),
                label: action.label.clone(),
                description: action.description.clone(),
                category: action.category.clone(),
                shortcut_label: action.shortcut.as_ref().map(ActionShortcut::display),
                enabled: self.is_active(action),
                disabled_reason: self.inactive_reason(action),
            })
            .collect()
    }

    /// Projects all valid actions for menu-like consumers.
    #[must_use]
    pub fn menu_items(&self) -> Vec<MenuItemProjection> {
        self.actions
            .iter()
            .map(|action| MenuItemProjection {
                action_id: action.id.clone(),
                label: action.label.clone(),
                category: action.category.clone(),
                shortcut_label: action.shortcut.as_ref().map(ActionShortcut::display),
                enabled: self.is_active(action),
                disabled_reason: self.inactive_reason(action),
            })
            .collect()
    }

    /// Projects all valid actions for toolbar-like consumers.
    #[must_use]
    pub fn toolbar_items(&self) -> Vec<ToolbarItemProjection> {
        self.actions
            .iter()
            .map(|action| ToolbarItemProjection {
                action_id: action.id.clone(),
                label: action.label.clone(),
                icon: action.icon.clone(),
                enabled: self.is_active(action),
                disabled_reason: self.inactive_reason(action),
            })
            .collect()
    }

    /// Projects shortcut help entries grouped by category.
    #[must_use]
    pub fn shortcut_help_sections(&self) -> Vec<ShortcutHelpSectionProjection> {
        let mut grouped: std::collections::BTreeMap<String, Vec<ShortcutHelpEntryProjection>> =
            std::collections::BTreeMap::new();
        for action in &self.actions {
            let Some(shortcut) = &action.shortcut else {
                continue;
            };
            grouped
                .entry(action.category.clone())
                .or_default()
                .push(ShortcutHelpEntryProjection {
                    action_id: action.id.clone(),
                    label: action.label.clone(),
                    shortcut_label: shortcut.display(),
                    enabled: self.is_active(action),
                    disabled_reason: self.inactive_reason(action),
                });
        }
        grouped
            .into_iter()
            .map(|(category, entries)| ShortcutHelpSectionProjection { category, entries })
            .collect()
    }
}

/// Projection for keymap binding consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapBindingProjection {
    /// Stable action ID.
    pub action_id: String,
    /// Action label.
    pub label: String,
    /// Shortcut scope.
    pub scope: String,
    /// Shortcut key.
    pub key: String,
    /// Shortcut modifiers.
    pub modifiers: Vec<String>,
}

/// Projection for leader-key menu consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderEntryProjection {
    /// Stable action ID.
    pub action_id: String,
    /// Action label.
    pub label: String,
    /// User-facing shortcut label.
    pub shortcut_label: String,
    /// Shortcut scope.
    pub scope: String,
}

/// Projection for command palette consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteItemProjection {
    /// Stable action ID.
    pub action_id: String,
    /// Action label.
    pub label: String,
    /// Optional action description.
    pub description: Option<String>,
    /// Action category.
    pub category: String,
    /// Optional user-facing shortcut label.
    pub shortcut_label: Option<String>,
    /// Whether the item can be activated.
    pub enabled: bool,
    /// Disabled reason when the item cannot be activated.
    pub disabled_reason: Option<String>,
}

/// Projection for menu consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItemProjection {
    /// Stable action ID.
    pub action_id: String,
    /// Action label.
    pub label: String,
    /// Action category.
    pub category: String,
    /// Optional user-facing shortcut label.
    pub shortcut_label: Option<String>,
    /// Whether the item can be activated.
    pub enabled: bool,
    /// Disabled reason when the item cannot be activated.
    pub disabled_reason: Option<String>,
}

/// Projection for toolbar consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolbarItemProjection {
    /// Stable action ID.
    pub action_id: String,
    /// Action label.
    pub label: String,
    /// Optional icon name or glyph identifier.
    pub icon: Option<String>,
    /// Whether the item can be activated.
    pub enabled: bool,
    /// Disabled reason when the item cannot be activated.
    pub disabled_reason: Option<String>,
}

/// Shortcut help section projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutHelpSectionProjection {
    /// Category heading.
    pub category: String,
    /// Entries in the category.
    pub entries: Vec<ShortcutHelpEntryProjection>,
}

/// Shortcut help entry projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutHelpEntryProjection {
    /// Stable action ID.
    pub action_id: String,
    /// Action label.
    pub label: String,
    /// User-facing shortcut label.
    pub shortcut_label: String,
    /// Whether the shortcut can currently be activated.
    pub enabled: bool,
    /// Disabled reason when the shortcut cannot be activated.
    pub disabled_reason: Option<String>,
}

fn shortcut_label(modifiers: &[String], key: &str) -> String {
    let mut parts = modifiers.to_vec();
    parts.push(key.to_string());
    parts.join(DISPLAY_SEPARATOR)
}
