//! Streaming output buffer with head/tail truncation and incremental search.

pub mod output_search;
pub mod search_render;
pub mod streaming_output;
pub mod streaming_render;

#[cfg(feature = "inline")]
mod inline_widget;

pub use output_search::OutputSearch;
pub use output_search::SearchMatch;
pub use output_search::SearchMode;
pub use search_render::apply_search_highlights;
pub use search_render::render_search_overlay;
pub use streaming_output::StreamingConfig;
pub use streaming_output::StreamingOutput;
pub use streaming_output::StreamingOutputManager;
pub use streaming_render::render_streaming_lines;
pub use streaming_render::render_streaming_stats;
