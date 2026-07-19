//! Interactive tree navigation widget for ratatui with keymap integration.
//!
//! Provides a generic, keyboard-driven tree view that renders hierarchical data
//! with expand/collapse, cursor navigation, guide lines, and configurable styles.
//! Integrates with `rat-keymap` for customizable key bindings.
//!
//! # Example
//!
//! ```rust,ignore
//! use rat_tree::{Tree, TreeState, TreeStyle, SimpleTree, TreeAction, default_keymap};
//! use ratatui::Frame;
//! use ratatui::layout::Rect;
//!
//! // Build tree data
//! let data = SimpleTree::new(vec![
//!     (0, None, "root".into()),
//!     (1, Some(0), "child-a".into()),
//!     (2, Some(0), "child-b".into()),
//!     (3, Some(1), "grandchild".into()),
//! ]);
//!
//! // Create state and keymap
//! let mut state = TreeState::new(&data);
//! let keymap = default_keymap();
//! let style = TreeStyle::default();
//!
//! // In your event loop, resolve keys to actions:
//! // if let Some(action) = keymap.resolve(&(), &key_event) {
//! //     state.apply(action, &data, viewport_height);
//! // }
//! ```

pub mod keymap;
pub mod model;
pub mod navigation;
pub mod render;
pub mod state;
pub mod style;

pub use keymap::{TreeAction, default_keymap, parse_tree_action};
pub use model::{NodeId, SimpleTree, TreeData, VisibleRow, compute_visible_rows};
pub use render::{Tree, TreeInfo};
pub use state::TreeState;
pub use style::TreeStyle;
