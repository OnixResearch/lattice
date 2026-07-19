//! Streaming output buffer with head/tail truncation and incremental search.

pub mod output_search;
pub mod search_render;
pub mod streaming_output;
pub mod streaming_render;

#[cfg(feature = "inline")]
mod inline_widget;

pub use output_search::{OutputSearch, SearchMatch, SearchMode};
pub use search_render::{apply_search_highlights, render_search_overlay};
pub use streaming_output::{StreamingConfig, StreamingOutput, StreamingOutputManager};
pub use streaming_render::{render_streaming_lines, render_streaming_stats};
