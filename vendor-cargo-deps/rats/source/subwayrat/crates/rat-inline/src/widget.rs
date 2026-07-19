//! The `InlineWidget` trait for leaf nodes in inline view trees.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// A widget that can participate in an inline view tree.
///
/// Implementors provide height measurement (for layout) and rendering
/// (into a ratatui `Buffer` region). This is the bridge between
/// ratcore's framework-agnostic view tree and ratatui's rendering model.
pub trait InlineWidget {
    /// Measure the desired height in rows given the available width.
    fn height(&self, width: u16) -> u16;

    /// Render into the given buffer region.
    fn render(&self, area: Rect, buf: &mut Buffer);
}
