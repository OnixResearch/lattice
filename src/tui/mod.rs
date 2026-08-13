//! TUI application layer.
//!
//! Layout:
//!  ┌─────────────────────────────────────────┬──────────────┐
//!  │              Node Canvas                │  Properties  │
//!  │          (rat-nodegraph)                │    Panel     │
//!  │                                         │              │
//!  ├─────────────────────────────────────────┴──────────────┤
//!  │           Output / Execution Log (rat-streaming)       │
//!  ├────────────────────────────────────────────────────────┤
//!  │  Status Bar                                            │
//!  └────────────────────────────────────────────────────────┘
//!
//!  Overlays: fuzzy command palette (Ctrl-P), workflow browser (Ctrl-O)

pub mod app;
pub mod graph;
pub mod input;
pub mod panels;

pub use app::App;
