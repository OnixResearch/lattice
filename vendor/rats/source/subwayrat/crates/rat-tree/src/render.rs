//! Tree widget rendering via ratatui `StatefulWidget`.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, StatefulWidget, Widget},
};

use crate::model::{NodeId, TreeData};
use crate::state::TreeState;
use crate::style::TreeStyle;

/// A tree widget that renders hierarchical data with guide lines, indicators,
/// and a cursor highlight.
///
/// Use the builder methods to configure, then render via `StatefulWidget`.
///
/// ```rust,ignore
/// use rat_tree::{Tree, TreeState, TreeStyle, SimpleTree};
/// use ratatui::widgets::Block;
///
/// let data = SimpleTree::new(vec![(0, None, "root".into())]);
/// let mut state = TreeState::new(&data);
///
/// let widget = Tree::new(&data)
///     .style(TreeStyle::default())
///     .block(Block::bordered().title("Files"));
/// // frame.render_stateful_widget(widget, area, &mut state);
/// ```
pub struct Tree<'a> {
    data: &'a dyn TreeData,
    style: TreeStyle,
    block: Option<Block<'a>>,
}

impl<'a> Tree<'a> {
    pub fn new(data: &'a dyn TreeData) -> Self {
        Self {
            data,
            style: TreeStyle::default(),
            block: None,
        }
    }

    pub fn style(mut self, style: TreeStyle) -> Self {
        self.style = style;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for Tree<'_> {
    type State = TreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render block and get inner area
        let inner = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let viewport_height = inner.height as usize;

        // Ensure scroll is valid
        state.ensure_cursor_visible(viewport_height);

        let s = &self.style;

        for row_offset in 0..viewport_height {
            let row_idx = state.scroll_offset + row_offset;
            let y = inner.y + row_offset as u16;

            let Some(vis_row) = state.visible_rows.get(row_idx) else {
                break;
            };

            let is_selected = row_idx == state.cursor;
            let row_style = if is_selected {
                s.selected_style
            } else {
                s.normal_style
            };

            // Build the line spans
            let mut spans: Vec<Span> = Vec::new();

            // Guide lines for each ancestor level
            for d in 0..vis_row.depth {
                let ancestor_is_last = vis_row.ancestors_last.get(d).copied().unwrap_or(false);
                let guide = if ancestor_is_last {
                    &s.guide_space
                } else {
                    &s.guide_pipe
                };
                spans.push(Span::styled(guide.clone(), row_style));
            }

            // Connector for this node (only if depth > 0)
            if vis_row.depth > 0 {
                let connector = if vis_row.is_last_sibling {
                    &s.guide_corner
                } else {
                    &s.guide_tee
                };
                spans.push(Span::styled(connector.clone(), row_style));
            }

            // Expand/collapse indicator
            let indicator = if vis_row.has_children {
                if vis_row.is_expanded {
                    &s.collapse_indicator
                } else {
                    &s.expand_indicator
                }
            } else {
                &s.leaf_indicator
            };
            spans.push(Span::styled(indicator.clone(), row_style));

            // Icon (if present)
            if let Some(icon) = self.data.node_icon(vis_row.node_id) {
                spans.push(Span::styled(
                    format!("{icon} "),
                    s.icon_style.patch(row_style),
                ));
            }

            // Label
            let label = self.data.node_label(vis_row.node_id);
            spans.push(Span::styled(label.to_string(), row_style));

            // Fill remaining width with background style for full-row highlight
            let line = Line::from(spans);
            let line_width = line.width() as u16;

            // Render the line content
            buf.set_line(inner.x, y, &line, inner.width);

            // Fill rest of the row with the row style for clean highlight
            if line_width < inner.width {
                for x in (inner.x + line_width)..(inner.x + inner.width) {
                    buf[(x, y)].set_style(row_style);
                }
            }
        }
    }
}

/// Computed snapshot of tree state for status bars and info displays.
#[derive(Debug, Clone)]
pub struct TreeInfo {
    /// Total number of visible rows.
    pub visible_count: usize,
    /// Node id at the cursor position.
    pub cursor_node_id: Option<NodeId>,
    /// Depth of the cursor node.
    pub cursor_depth: Option<usize>,
    /// Whether the cursor is on a leaf node.
    pub cursor_is_leaf: Option<bool>,
}

impl TreeState {
    /// Compute a snapshot of the current tree state for info displays.
    pub fn info(&self) -> TreeInfo {
        let row = self.visible_rows.get(self.cursor);
        TreeInfo {
            visible_count: self.visible_rows.len(),
            cursor_node_id: row.map(|r| r.node_id),
            cursor_depth: row.map(|r| r.depth),
            cursor_is_leaf: row.map(|r| !r.has_children),
        }
    }
}
