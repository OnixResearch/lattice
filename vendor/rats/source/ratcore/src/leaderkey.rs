//! Helix-style leader key menu: types, builder, and state machine.
//!
//! Generic over the action type `A`. Press a leader key → see available
//! actions → press another key to execute or open a submenu. Escape or
//! unrecognized keys dismiss the menu.
//!
//! Items are contributed dynamically via [`MenuContributor`] with
//! priority-based conflict resolution. Multiple sources (builtins,
//! plugins, user config) can register to the same menu, and higher
//! priority wins on key collisions.
//!
//! This module is rendering-agnostic. Both TUI and web frontends
//! consume the same state machine.
//!
//! # Example
//!
//! ```
//! use ratcore::leaderkey::*;
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum Act { Save, Quit }
//!
//! struct Builtins;
//! impl MenuContributor<Act> for Builtins {
//!     fn menu_items(&self) -> Vec<MenuContribution<Act>> {
//!         vec![
//!             MenuContribution {
//!                 key: 's',
//!                 label: "save".into(),
//!                 action: LeaderAction::Action(Act::Save),
//!                 placement: MenuPlacement::Root,
//!                 priority: PRIORITY_BUILTIN,
//!                 source: "builtin".into(),
//!             },
//!             MenuContribution {
//!                 key: 'q',
//!                 label: "quit".into(),
//!                 action: LeaderAction::Action(Act::Quit),
//!                 placement: MenuPlacement::Root,
//!                 priority: PRIORITY_BUILTIN,
//!                 source: "builtin".into(),
//!             },
//!         ]
//!     }
//! }
//!
//! let hidden = std::collections::HashSet::new();
//! let (mut menu, conflicts) = build(&[&Builtins], &hidden);
//! assert!(conflicts.is_empty());
//! menu.open();
//! assert_eq!(menu.handle_char('s'), Some(LeaderAction::Action(Act::Save)));
//! ```

// ═══════════════════════════════════════════════════════════════════
// Priority constants
// ═══════════════════════════════════════════════════════════════════

/// Priority for built-in (compile-time) registrations.
pub const PRIORITY_BUILTIN: u16 = 0;

/// Priority for plugin registrations (loaded at runtime).
pub const PRIORITY_PLUGIN: u16 = 100;

/// Priority for user config overrides (highest, always wins).
pub const PRIORITY_USER: u16 = 200;

// ═══════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════

/// An action that a leader menu item can trigger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaderAction<A> {
    /// Dispatch a user-defined action.
    Action(A),
    /// Execute a command string (e.g. "/compact", ":write").
    Command(String),
    /// Open a named submenu.
    Submenu(String),
}

/// A single entry in the leader key menu.
#[derive(Debug, Clone)]
pub struct LeaderMenuItem<A> {
    /// Key to press (single char).
    pub key: char,
    /// Display label.
    pub label: String,
    /// What happens when selected.
    pub action: LeaderAction<A>,
}

/// A named menu level (root or submenu).
#[derive(Debug, Clone)]
pub struct LeaderMenuDef<A> {
    /// Menu label shown in headings and breadcrumbs.
    pub label: String,
    /// Items available at this menu level.
    pub items: Vec<LeaderMenuItem<A>>,
}

/// Where a menu item should appear.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuPlacement {
    /// Top-level root menu.
    Root,
    /// Inside a named submenu (created if it doesn't exist).
    Submenu(String),
}

/// A single contribution to the leader menu from any source.
#[derive(Debug, Clone)]
pub struct MenuContribution<A> {
    /// Key to press (single char).
    pub key: char,
    /// Display label.
    pub label: String,
    /// What happens when selected.
    pub action: LeaderAction<A>,
    /// Where this item appears.
    pub placement: MenuPlacement,
    /// Priority for conflict resolution (higher wins).
    pub priority: u16,
    /// Source identifier for diagnostics ("builtin", plugin name, "config").
    pub source: String,
}

/// Anything that contributes items to the leader menu.
pub trait MenuContributor<A> {
    /// Returns this contributor's menu items.
    fn menu_items(&self) -> Vec<MenuContribution<A>>;
}

/// Set of `(key, placement)` pairs to exclude from the built menu.
pub type HiddenSet = std::collections::HashSet<(char, MenuPlacement)>;

mod assembly;
mod state;
pub use assembly::{BuildResult, build, build_from_items};
pub use state::LeaderMenu;

// ═══════════════════════════════════════════════════════════════════
// Conflicts
// ═══════════════════════════════════════════════════════════════════

/// A conflict detected during menu build.
///
/// When two sources register the same key in the same scope, the
/// higher-priority source wins and a `Conflict` is reported.
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Which registry detected the conflict.
    pub registry: &'static str,
    /// What conflicted (key char, placement, etc.).
    pub key: String,
    /// Source that won.
    pub winner: String,
    /// Source that lost.
    pub loser: String,
}

// ═══════════════════════════════════════════════════════════════════
// State machine
// ═══════════════════════════════════════════════════════════════════

/// Input event for the leader menu state machine.
///
/// Renderers convert platform-specific key events into this enum
/// before calling [`LeaderMenu::handle_input`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuInput {
    /// A printable character (with or without Shift).
    Char(char),
    /// Escape key — back one level or close.
    Escape,
    /// Any other key — dismisses the menu.
    Other,
}

// ═══════════════════════════════════════════════════════════════════
// Builder
// ═══════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
#[path = "leaderkey/buildertests.rs"]
mod builder_tests;
#[cfg(test)]
#[path = "leaderkey/statetests.rs"]
mod state_tests;
