//! Renderer-neutral action registry and projection helpers.
//!
//! The registry is a pure metadata core for command-like UI surfaces. It keeps
//! action descriptors independent of ratatui, Dioxus, crossterm, or browser
//! event types, then exposes deterministic projection structs that renderer
//! crates can adapt into keymaps, command palettes, menus, toolbars, and
//! shortcut help.
//!
//! # Example
//!
//! ```
//! use ratcore::action_registry::{
//!     ActionDescriptor, ActionShortcut, ActionSource, build_registry,
//! };
//!
//! let registry = build_registry(vec![
//!     ActionDescriptor::new("file.save", "Save", ActionSource::new("builtin"))
//!         .with_category("file")
//!         .with_shortcut(ActionShortcut::global("s", vec!["Ctrl".to_string()])),
//! ]);
//!
//! assert!(registry.is_valid());
//! let tui_menu_items = registry.snapshot.menu_items();
//! let meteorite_palette_items = registry.snapshot.command_palette_items();
//! assert_eq!(tui_menu_items[0].action_id, "file.save");
//! assert_eq!(meteorite_palette_items[0].action_id, "file.save");
//! ```
//!
//! Migration path: keep existing renderer-specific command structs as thin
//! adapters, build this registry at application startup, fail startup or disable
//! ambiguous bindings when [`ActionRegistryBuild::issues`] is non-empty, and
//! feed the resulting snapshot into each renderer projection instead of copying
//! labels, shortcuts, and enablement decisions by hand.

const DEFAULT_CATEGORY: &str = "general";
const DEFAULT_SHORTCUT_SCOPE: &str = "global";
const DISPLAY_SEPARATOR: &str = "+";
const FIELD_ID: &str = "id";
const FIELD_LABEL: &str = "label";
const FIELD_SOURCE: &str = "source";
const FIELD_CATEGORY: &str = "category";
const FIELD_SHORTCUT_SCOPE: &str = "shortcut.scope";
const FIELD_SHORTCUT_KEY: &str = "shortcut.key";
const FIELD_SHORTCUT_MODIFIER: &str = "shortcut.modifier";
const BLOCKED_CONFLICT_REASON: &str = "shortcut conflict";

/// Metadata describing where an action came from.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActionSource {
    /// Stable source name, such as `builtin`, `plugin:git`, or `user-config`.
    pub name: String,
    /// Optional source kind for diagnostics and grouping.
    pub kind: Option<String>,
}

impl ActionSource {
    /// Creates source metadata with a stable source name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: None,
        }
    }

    /// Adds a source kind.
    #[must_use]
    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }
}

/// Deterministic action enablement state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionEnablement {
    /// The action can be activated when no registry conflict blocks it.
    Enabled,
    /// The action is visible but cannot be activated.
    Disabled {
        /// Human-readable disabled reason.
        reason: String,
    },
}

impl ActionEnablement {
    /// Returns true when the descriptor declares itself enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }

    /// Returns the disabled reason when present.
    #[must_use]
    pub fn disabled_reason(&self) -> Option<&str> {
        match self {
            Self::Enabled => None,
            Self::Disabled { reason } => Some(reason.as_str()),
        }
    }
}

/// A keyboard shortcut in a named command scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionShortcut {
    /// Shortcut scope, such as `global`, `editor`, or `palette`.
    pub scope: String,
    /// Key name, such as `k`, `Enter`, or `F1`.
    pub key: String,
    /// Modifier names, such as `Ctrl`, `Alt`, or `Shift`.
    pub modifiers: Vec<String>,
}

impl ActionShortcut {
    /// Creates a shortcut in a named scope.
    #[must_use]
    pub fn new(scope: impl Into<String>, key: impl Into<String>, modifiers: Vec<String>) -> Self {
        Self {
            scope: scope.into(),
            key: key.into(),
            modifiers,
        }
    }

    /// Creates a shortcut in the global command scope.
    #[must_use]
    pub fn global(key: impl Into<String>, modifiers: Vec<String>) -> Self {
        Self::new(DEFAULT_SHORTCUT_SCOPE, key, modifiers)
    }

    /// Returns a deterministic user-facing shortcut label.
    #[must_use]
    pub fn display(&self) -> String {
        let mut parts = self.modifiers.clone();
        parts.push(self.key.clone());
        parts.join(DISPLAY_SEPARATOR)
    }
}

/// Renderer-neutral action descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDescriptor {
    /// Stable action ID.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Optional longer description.
    pub description: Option<String>,
    /// Optional keyboard shortcut.
    pub shortcut: Option<ActionShortcut>,
    /// Optional icon name or glyph identifier.
    pub icon: Option<String>,
    /// Category used for deterministic grouping.
    pub category: String,
    /// Source metadata for diagnostics.
    pub source: ActionSource,
    /// Deterministic enablement state.
    pub enablement: ActionEnablement,
}

impl ActionDescriptor {
    /// Creates a descriptor with the required stable metadata.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, source: ActionSource) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            shortcut: None,
            icon: None,
            category: DEFAULT_CATEGORY.to_string(),
            source,
            enablement: ActionEnablement::Enabled,
        }
    }

    /// Adds a longer description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a shortcut.
    #[must_use]
    pub fn with_shortcut(mut self, shortcut: ActionShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Adds an icon identifier.
    #[must_use]
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Sets a category.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    /// Sets the enablement state.
    #[must_use]
    pub fn with_enablement(mut self, enablement: ActionEnablement) -> Self {
        self.enablement = enablement;
        self
    }
}

mod assembly;
mod desktop_menu;
mod diagnostics;
mod keybinding_editor;
mod surfaces;
pub use assembly::build_registry;
pub use desktop_menu::{
    DesktopMenuCheckedState, DesktopMenuDiagnostic, DesktopMenuDiagnosticCode, DesktopMenuEntry,
    DesktopMenuFocusNode, DesktopMenuFocusProjection, DesktopMenuFocusRole, DesktopMenuItemKind,
    DesktopMenuItemProjection, DesktopMenuProjection, DesktopMenuRenderer,
    DesktopMenuRendererProjection, build_desktop_menu_projection, project_desktop_menu_focus,
    project_desktop_menu_renderer,
};
pub use diagnostics::{ActionRegistryIssue, ActionRegistryIssueCode};
pub use keybinding_editor::{
    KeybindingDefault, KeybindingEditorDiagnostic, KeybindingEditorDiagnosticCode,
    KeybindingEditorModel, KeybindingEditorRenderer, KeybindingEditorRendererProjection,
    KeybindingEditorRow, KeybindingOverride, ReservedShortcut, ShortcutEditCommand,
    ShortcutEditOutcome, apply_shortcut_edit, build_keybinding_editor_model,
    project_keybinding_editor_renderer,
};
pub use surfaces::{
    ActionRegistrySnapshot, CommandPaletteItemProjection, KeymapBindingProjection,
    LeaderEntryProjection, MenuItemProjection, ShortcutHelpEntryProjection,
    ShortcutHelpSectionProjection, ToolbarItemProjection,
};

/// Result of building an action registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRegistryBuild {
    /// Valid registry snapshot. Malformed and duplicate-ID descriptors are omitted.
    pub snapshot: ActionRegistrySnapshot,
    /// Deterministic validation diagnostics.
    pub issues: Vec<ActionRegistryIssue>,
}

impl ActionRegistryBuild {
    /// Returns true when the registry had no validation issues.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
