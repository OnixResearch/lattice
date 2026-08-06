//! Backend-neutral component-local keymaps.
//!
//! `rat-keymap` stores validated Ratcore logical strokes. Physical Crossterm,
//! browser, Dioxus, Ratzilla, or graphical events must be normalized by their
//! owning adapter before they reach this crate. The map remains a bounded
//! one-stroke compatibility mechanism; application actions use Ratcore's
//! generation-bound layered binding catalog and contextual router.
//!
//! ```rust
//! use rat_keymap::{KeyCombo, Keymap, LogicalKey, LogicalKeystroke};
//! use std::collections::HashMap;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! enum Mode { Normal }
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! enum Action { Quit }
//!
//! let quit = LogicalKeystroke::try_new(LogicalKey::Character('q'), Vec::new())
//!     .expect("valid logical stroke");
//! let bindings = vec![(Mode::Normal, HashMap::from([(
//!     quit.clone(),
//!     Action::Quit,
//! )]))];
//! let keymap: Keymap<Action, Mode> = Keymap::build(bindings, &[], |_| None);
//! assert_eq!(keymap.resolve(&Mode::Normal, &quit), Some(Action::Quit));
//! let _: KeyCombo = quit;
//! ```

pub mod combo;
pub mod keymap;

/// Compatibility name for one canonical logical stroke.
pub type KeyCombo = combo::Chord;
/// Compatibility name for the one-stroke local map.
pub type Keymap<A, M> = keymap::Map<A, M>;

pub use combo::try_chord;
pub use keymap::LocalSequenceResolution;
pub use ratcore::action_registry::ActiveBindingLayer;
pub use ratcore::action_registry::ActiveBindingLayers;
pub use ratcore::action_registry::BindingCatalogSnapshot;
pub use ratcore::action_registry::KeyModifier;
pub use ratcore::action_registry::KeySequence;
pub use ratcore::action_registry::KeySequenceDiagnostic;
pub use ratcore::action_registry::KeySequenceDiagnosticCode;
pub use ratcore::action_registry::KeySequenceInput;
pub use ratcore::action_registry::KeySequenceState;
pub use ratcore::action_registry::LogicalKey;
pub use ratcore::action_registry::LogicalKeystroke;
pub use ratcore::action_registry::reduce_key_sequence;

/// Parses one deterministic logical compatibility binding.
#[must_use]
pub fn parse_key_string(value: &str) -> Option<KeyCombo> {
    combo::parse_key_string(value)
}

/// Formats one logical compatibility binding.
#[must_use]
pub fn format_key_combo(combo: &KeyCombo) -> String {
    combo::format(combo)
}
