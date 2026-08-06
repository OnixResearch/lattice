//! Inline scrollback renderer.
//!
//! Manages a ratatui `Buffer`, performs frame diffing, and emits
//! ANSI escape sequences for terminal output. Content grows into
//! scrollback via newline emission.

use std::io::Write;

use ratatui::buffer::Buffer;
use ratatui::buffer::Cell;
use ratatui::layout::Rect;
use ratcore::inline::NodeKey;
use ratcore::inline::ViewTree;
use ratcore::inline::{self};

use crate::builder::InlineView;
use crate::widget::InlineWidget;

/// Inline renderer that writes styled content into terminal scrollback.
pub struct InlineRenderer {
    /// Current frame buffer.
    current: Buffer,
    /// Previous frame buffer (for diffing).
    previous: Buffer,
    /// Terminal width.
    width: u16,
    /// Number of terminal rows we've claimed via newlines.
    claimed_rows: u16,
    /// The reconciled view tree from the last rebuild.
    tree: ViewTree,
    /// The widgets corresponding to each node in the tree.
    widgets: Vec<Box<dyn InlineWidget>>,
    /// Callback fired when nodes scroll off the viewport.
    on_commit: Option<Box<dyn FnMut(&NodeKey)>>,
}

impl InlineRenderer {
    /// Create a new renderer at the given terminal width.
    pub fn new(width: u16) -> Self {
        let empty = Buffer::empty(Rect::new(0, 0, width, 0));
        Self {
            current: empty.clone(),
            previous: empty,
            width,
            claimed_rows: 0,
            tree: ViewTree::new(),
            widgets: Vec::new(),
            on_commit: None,
        }
    }

    /// Set the callback for when nodes scroll off the viewport.
    pub fn on_commit<F: FnMut(&NodeKey) + 'static>(&mut self, f: F) {
        self.on_commit = Some(Box::new(f));
    }

    /// Rebuild the view tree. Reconciles against the previous tree
    /// to preserve widget state, then measures and renders.
    pub fn rebuild(&mut self, view: InlineView) {
        let (new_tree, new_widgets) = view.build();

        // Reconcile to preserve state.
        let old_nodes = std::mem::take(&mut self.tree).nodes;
        let result = inline::reconcile(old_nodes, new_tree.nodes);

        self.tree = ViewTree { nodes: result.nodes };
        self.widgets = new_widgets;
    }

    /// Check for terminal width changes and update if needed.
    pub fn update_width(&mut self) {
        if let Ok((w, _)) = crossterm::terminal::size() {
            if w != self.width && w > 0 {
                self.width = w;
                // Force full re-render by resetting previous buffer.
                self.previous = Buffer::empty(Rect::new(0, 0, self.width, 0));
                self.claimed_rows = 0;
            }
        }
    }

    /// Measure total content height.
    fn measure_height(&self) -> u16 {
        self.widgets.iter().map(|w| w.height(self.width)).sum()
    }

    /// Render the current frame into the buffer and return the ANSI
    /// diff output. Call `update_width` before this if you want
    /// resize tracking.
    pub fn render(&mut self) -> Vec<u8> {
        let total_height = self.measure_height();
        if total_height == 0 {
            return Vec::new();
        }

        // Allocate new buffer.
        let area = Rect::new(0, 0, self.width, total_height);
        let mut buf = Buffer::empty(area);

        // Render each widget into its vertical slice.
        let mut y = 0u16;
        for widget in &self.widgets {
            let h = widget.height(self.width);
            if h == 0 {
                continue;
            }
            let widget_area = Rect::new(0, y, self.width, h);
            widget.render(widget_area, &mut buf);
            y += h;
        }

        // Swap buffers.
        self.previous = std::mem::replace(&mut self.current, buf);

        // Compute diff.
        let mut output = Vec::new();
        self.write_diff(&mut output, total_height);
        output
    }

    /// Write the ANSI diff between previous and current buffers.
    fn write_diff(&mut self, output: &mut Vec<u8>, total_height: u16) {
        // Growth: emit newlines to claim new rows.
        let new_rows = total_height.saturating_sub(self.claimed_rows);
        if new_rows > 0 {
            for _ in 0..new_rows {
                output.extend_from_slice(b"\n");
            }
            self.claimed_rows = total_height;
        } else if total_height < self.claimed_rows {
            // Content shrank — clear excess rows later.
            self.claimed_rows = total_height;
        }

        // Check if there are any changes.
        let has_changes = self.has_changes(total_height);
        if !has_changes {
            return;
        }

        // DEC synchronized output start.
        output.extend_from_slice(b"\x1b[?2026h");

        // Move cursor to the top of our render region.
        // We're at the bottom after emitting newlines, so move up.
        if self.claimed_rows > 0 {
            write!(output, "\x1b[{}A", self.claimed_rows).ok();
        }
        output.extend_from_slice(b"\r"); // column 0

        // Emit only changed cells.
        let mut last_row = 0u16;
        let mut last_col = 0u16;
        let mut cursor_positioned = true; // we just moved to (0,0)

        for row in 0..total_height {
            for col in 0..self.width {
                let current_cell = self.current.cell(ratatui::layout::Position::new(col, row));
                let prev_cell = if row < self.previous.area.height {
                    self.previous.cell(ratatui::layout::Position::new(col, row))
                } else {
                    None
                };

                let changed = match (current_cell, prev_cell) {
                    (Some(c), Some(p)) => c != p,
                    (Some(_), None) => true,
                    _ => continue,
                };

                if !changed {
                    cursor_positioned = false;
                    continue;
                }

                let cell = current_cell.unwrap();

                // Position cursor if needed.
                if !cursor_positioned || row != last_row || col != last_col {
                    // Move to absolute position within our region.
                    // Row offset from top of region: row
                    // We're at row 0 of our region, so move down `row` and to col.
                    write!(output, "\x1b[{};{}H", row + 1, col + 1).ok();
                }

                // Emit style + content.
                write_styled_cell(output, cell);

                last_row = row;
                last_col = col + 1;
                cursor_positioned = true;
            }
        }

        // Move cursor to bottom of our region.
        write!(output, "\x1b[{};1H", self.claimed_rows).ok();

        // DEC synchronized output end.
        output.extend_from_slice(b"\x1b[?2026l");
    }

    /// Check if any cells differ between current and previous buffers.
    fn has_changes(&self, total_height: u16) -> bool {
        for row in 0..total_height {
            for col in 0..self.width {
                let current = self.current.cell(ratatui::layout::Position::new(col, row));
                let prev = if row < self.previous.area.height {
                    self.previous.cell(ratatui::layout::Position::new(col, row))
                } else {
                    None
                };
                match (current, prev) {
                    (Some(c), Some(p)) if c != p => return true,
                    (Some(_), None) => return true,
                    _ => {}
                }
            }
        }
        false
    }

    /// Process commit callbacks for nodes that have scrolled off-screen.
    pub fn process_commits(&mut self, viewport_height: u16) {
        if self.on_commit.is_none() {
            return;
        }

        let heights: Vec<u16> = self.widgets.iter().map(|w| w.height(self.width)).collect();

        let scroll_offset = self.claimed_rows.saturating_sub(viewport_height);
        let viewport = inline::CommitViewport {
            viewport_height,
            scroll_offset,
        };
        let committed = inline::compute_commits(&heights, viewport);

        if committed.is_empty() {
            return;
        }

        // Fire callbacks for committed nodes.
        if let Some(ref mut callback) = self.on_commit {
            for &idx in &committed {
                let index = committed_index_to_usize(idx);
                if let Some(ref key) = self.tree.nodes[index].key {
                    callback(key);
                }
            }
        }

        // Remove committed nodes (in reverse to preserve indices).
        for &idx in committed.iter().rev() {
            let index = committed_index_to_usize(idx);
            self.tree.nodes.remove(index);
            self.widgets.remove(index);
        }
    }

    /// Current terminal width.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Number of nodes in the current view tree.
    pub fn node_count(&self) -> usize {
        self.tree.nodes.len()
    }
}

fn committed_index_to_usize(index: u64) -> usize {
    usize::try_from(index).expect("committed node index must fit usize")
}

/// Write a styled cell's content as ANSI escape sequences.
fn write_styled_cell(output: &mut Vec<u8>, cell: &Cell) {
    use ratatui::style::Color;

    let style = cell.style();

    // Reset.
    output.extend_from_slice(b"\x1b[0m");

    // Foreground.
    match style.fg {
        Some(Color::Rgb(r, g, b)) => {
            write!(output, "\x1b[38;2;{r};{g};{b}m").ok();
        }
        Some(Color::Indexed(n)) => {
            write!(output, "\x1b[38;5;{n}m").ok();
        }
        Some(color) => {
            let code = ansi_fg_code(color);
            if code > 0 {
                write!(output, "\x1b[{code}m").ok();
            }
        }
        None => {}
    }

    // Background.
    match style.bg {
        Some(Color::Rgb(r, g, b)) => {
            write!(output, "\x1b[48;2;{r};{g};{b}m").ok();
        }
        Some(Color::Indexed(n)) => {
            write!(output, "\x1b[48;5;{n}m").ok();
        }
        Some(color) => {
            let code = ansi_bg_code(color);
            if code > 0 {
                write!(output, "\x1b[{code}m").ok();
            }
        }
        None => {}
    }

    // Modifiers.
    let mods = style.add_modifier;
    if mods.contains(ratatui::style::Modifier::BOLD) {
        output.extend_from_slice(b"\x1b[1m");
    }
    if mods.contains(ratatui::style::Modifier::DIM) {
        output.extend_from_slice(b"\x1b[2m");
    }
    if mods.contains(ratatui::style::Modifier::ITALIC) {
        output.extend_from_slice(b"\x1b[3m");
    }
    if mods.contains(ratatui::style::Modifier::UNDERLINED) {
        output.extend_from_slice(b"\x1b[4m");
    }

    // Content.
    write!(output, "{}", cell.symbol()).ok();
}

fn ansi_fg_code(color: ratatui::style::Color) -> u8 {
    use ratatui::style::Color;
    match color {
        Color::Black => 30,
        Color::Red => 31,
        Color::Green => 32,
        Color::Yellow => 33,
        Color::Blue => 34,
        Color::Magenta => 35,
        Color::Cyan => 36,
        Color::White | Color::Gray => 37,
        Color::DarkGray => 90,
        Color::LightRed => 91,
        Color::LightGreen => 92,
        Color::LightYellow => 93,
        Color::LightBlue => 94,
        Color::LightMagenta => 95,
        Color::LightCyan => 96,
        _ => 0,
    }
}

fn ansi_bg_code(color: ratatui::style::Color) -> u8 {
    use ratatui::style::Color;
    match color {
        Color::Black => 40,
        Color::Red => 41,
        Color::Green => 42,
        Color::Yellow => 43,
        Color::Blue => 44,
        Color::Magenta => 45,
        Color::Cyan => 46,
        Color::White | Color::Gray => 47,
        Color::DarkGray => 100,
        Color::LightRed => 101,
        Color::LightGreen => 102,
        Color::LightYellow => 103,
        Color::LightBlue => 104,
        Color::LightMagenta => 105,
        Color::LightCyan => 106,
        _ => 0,
    }
}
