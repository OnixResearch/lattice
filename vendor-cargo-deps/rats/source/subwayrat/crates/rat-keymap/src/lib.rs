//! Generic modal keymap system for Ratatui applications.
//!
//! This crate provides a flexible keymap system that can be parameterized over
//! any action type (`A`) and mode type (`M`). It handles key event resolution,
//! override support, and help text generation.
//!
//! # Example
//!
//! ```rust
//! use rat_keymap::{Keymap, KeyCombo};
//! use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
//! use std::collections::HashMap;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! enum Mode { Normal, Insert }
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum Action { Quit, Submit, MoveUp }
//!
//! // Build a keymap with some initial bindings
//! let bindings = vec![
//!     (Mode::Normal, {
//!         let mut map = HashMap::new();
//!         map.insert(KeyCombo::new(KeyCode::Char('q'), false, false, false), Action::Quit);
//!         map
//!     }),
//! ];
//!
//! let parse_action = |s: &str| match s {
//!     "quit" => Some(Action::Quit),
//!     "submit" => Some(Action::Submit),
//!     "move_up" => Some(Action::MoveUp),
//!     _ => None,
//! };
//!
//! let keymap = Keymap::build(bindings, &[], parse_action);
//!
//! // Resolve a key event
//! let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
//! let action = keymap.resolve(&Mode::Normal, &event);
//! assert_eq!(action, Some(Action::Quit));
//! ```

pub mod combo;
pub mod keymap;

// Re-export public types.
pub type KeyCombo = combo::Chord;
pub type Keymap<A, M> = keymap::Map<A, M>;

pub fn parse_key_string(s: &str) -> Option<KeyCombo> {
    combo::parse_key_string(s)
}

pub fn format_key_combo(k: &KeyCombo) -> String {
    combo::format(k)
}
