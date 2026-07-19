//! Inline scrollback renderer for ratatui.
//!
//! Renders styled content into terminal scrollback (not alternate screen).
//! Uses ratcore's reconciler for state preservation across rebuilds and
//! frame diffing to minimize ANSI output.
//!
//! # Architecture
//!
//! - `ratcore::inline` — framework-agnostic view tree, reconciler, commit tracking
//! - `rat-inline` (this crate) — ratatui rendering backend, builder API, terminal I/O
//!
//! # Usage
//!
//! ```no_run
//! use rat_inline::{InlineRenderer, InlineView, InlineText};
//!
//! let mut renderer = InlineRenderer::new(80);
//! let view = InlineView::new()
//!     .text("Hello from rat-inline!")
//!     .text("Content grows into scrollback.");
//! renderer.rebuild(view);
//! let output = renderer.render();
//! // Write `output` to stdout.
//! ```

mod builder;
mod renderer;
mod widget;
mod widgets;

pub use builder::InlineView;
pub use renderer::InlineRenderer;
pub use widget::InlineWidget;
pub use widgets::{InlineMarkdown, InlineText};

// Re-export ratcore inline types for convenience.
pub use ratcore::inline::{NodeKey, ViewNode, ViewTree};
