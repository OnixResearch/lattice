//! Tree view for session navigation

use rat_chrome::overlay_frame;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::popup_chrome::POPUP_MARGIN;
use crate::popup_chrome::centered_overlay_model;
use crate::popup_chrome::clamped_axis;
use crate::popup_chrome::themed_overlay_style;
use crate::popup_chrome::title;
use crate::theme::WidgetTheme;

const TREE_VIEW_WIDTH: u16 = 50;
const TREE_VIEW_HEIGHT: u16 = 20;
const TREE_INDENT: &str = "  ";

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub id: String,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub depth: usize,
}

/// Pure data model for a tree view — no ratatui dependency.
pub struct TreeViewModel {
    pub root: Vec<TreeNode>,
    pub selected: usize,
    pub visible: bool,
    pub flat_cache: Vec<(String, String, usize)>, // (label, id, depth)
}

impl TreeViewModel {
    pub fn new(root: Vec<TreeNode>) -> Self {
        let mut model = Self {
            root,
            selected: 0,
            visible: false,
            flat_cache: Vec::new(),
        };
        model.rebuild_cache();
        model
    }

    pub fn rebuild_cache(&mut self) {
        self.flat_cache.clear();
        fn flatten(nodes: &[TreeNode], cache: &mut Vec<(String, String, usize)>) {
            for node in nodes {
                cache.push((node.label.clone(), node.id.clone(), node.depth));
                if node.expanded {
                    flatten(&node.children, cache);
                }
            }
        }
        flatten(&self.root, &mut self.flat_cache);
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        self.selected = (self.selected + 1).min(self.flat_cache.len().saturating_sub(1));
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.flat_cache.get(self.selected).map(|(_, id, _)| id.as_str())
    }
}

pub struct TreeView {
    pub model: TreeViewModel,
    pub title: String,
}

impl TreeView {
    pub fn new(title: impl Into<String>, root: Vec<TreeNode>) -> Self {
        Self {
            model: TreeViewModel::new(root),
            title: title.into(),
        }
    }

    pub fn move_up(&mut self) {
        self.model.move_up();
    }

    pub fn move_down(&mut self) {
        self.model.move_down();
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.model.selected_id()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.model.visible {
            return;
        }

        let model = tree_overlay_model(area, &self.title);
        let theme = WidgetTheme::default();
        let style = themed_overlay_style(&theme, Color::Blue);
        let layout = overlay_frame(frame, area, &model, &style);

        if layout.inner.width == 0 || layout.inner.height == 0 {
            return;
        }

        let selected_style = Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD);
        frame.render_widget(Paragraph::new(tree_lines(&self.model, selected_style, Style::default())), layout.inner);
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        if !self.model.visible {
            return;
        }

        let model = tree_overlay_model(area, &self.title);
        let style = themed_overlay_style(theme, theme.primary);
        let layout = overlay_frame(frame, area, &model, &style);

        if layout.inner.width == 0 || layout.inner.height == 0 {
            return;
        }

        frame.render_widget(
            Paragraph::new(tree_lines(&self.model, theme.highlight_style(), Style::default())),
            layout.inner,
        );
    }
}

fn tree_overlay_model(area: Rect, title_text: &str) -> rat_chrome::OverlayModel {
    let width = clamped_axis(area.width, TREE_VIEW_WIDTH, POPUP_MARGIN);
    let height = clamped_axis(area.height, TREE_VIEW_HEIGHT, POPUP_MARGIN);

    centered_overlay_model(width, height, title(title_text))
}

fn tree_lines(model: &TreeViewModel, selected_style: Style, normal_style: Style) -> Vec<Line<'static>> {
    model
        .flat_cache
        .iter()
        .enumerate()
        .map(|(index, (label, _, depth))| {
            let indent = TREE_INDENT.repeat(*depth);
            let style = if index == model.selected {
                selected_style
            } else {
                normal_style
            };
            Line::from(Span::styled(format!("{indent}{label}"), style))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rat_chrome::compute_overlay_layout;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::text::Line;

    use super::*;

    const VIEWPORT_WIDTH: u16 = 70;
    const VIEWPORT_HEIGHT: u16 = 18;
    const TINY_VIEWPORT_WIDTH: u16 = 1;
    const TINY_VIEWPORT_HEIGHT: u16 = 1;
    const BACKGROUND_CELL: &str = "x";
    const TREE_TITLE: &str = "Tree";
    const ROOT_LABEL: &str = "Root";
    const ROOT_ID: &str = "root";

    #[test]
    fn visible_tree_view_uses_chrome_and_renders_root() {
        let mut tree = sample_tree();
        tree.model.visible = true;

        let buffer = render_tree(&tree, viewport());
        let layout = tree_layout(viewport(), &tree);

        assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, Color::Blue);
        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].symbol(), "R");
        assert!(buffer[(layout.inner.x, layout.inner.y)].modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn hidden_tree_view_is_noop() {
        let tree = sample_tree();

        let expected = render_background(viewport());
        let actual = render_tree(&tree, viewport());

        assert_eq!(actual, expected);
    }

    #[test]
    fn tiny_tree_view_area_does_not_panic() {
        let mut tree = sample_tree();
        tree.model.visible = true;

        let buffer = render_tree(&tree, tiny_viewport());

        assert_eq!(buffer.area.width, TINY_VIEWPORT_WIDTH);
        assert_eq!(buffer.area.height, TINY_VIEWPORT_HEIGHT);
    }

    fn sample_tree() -> TreeView {
        TreeView::new(TREE_TITLE, vec![TreeNode {
            label: ROOT_LABEL.to_string(),
            id: ROOT_ID.to_string(),
            children: Vec::new(),
            expanded: true,
            depth: 0,
        }])
    }

    fn render_tree(tree: &TreeView, viewport: Rect) -> Buffer {
        let backend = TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_test_background(frame, area);
                tree.render(frame, area);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn render_background(viewport: Rect) -> Buffer {
        let backend = TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_test_background(frame, area);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn render_test_background(frame: &mut Frame, area: Rect) {
        let line = Line::from(BACKGROUND_CELL.repeat(usize::from(area.width)));
        let rows = vec![line; usize::from(area.height)];
        frame.render_widget(Paragraph::new(rows), area);
    }

    fn tree_layout(viewport: Rect, tree: &TreeView) -> rat_chrome::OverlayLayout {
        compute_overlay_layout(viewport, &tree_overlay_model(viewport, &tree.title))
    }

    fn viewport() -> Rect {
        Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn tiny_viewport() -> Rect {
        Rect::new(0, 0, TINY_VIEWPORT_WIDTH, TINY_VIEWPORT_HEIGHT)
    }
}
