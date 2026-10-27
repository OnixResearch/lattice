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

#[allow(
    unknown_lints,
    excessive_file_length,
    reason = "the legacy TUI host remains a separately tracked decomposition; this change only carries bounded completion evidence"
)]
pub mod app;
pub mod authoring;
pub mod graph;
pub mod input;
pub mod panels;

pub use app::App;
