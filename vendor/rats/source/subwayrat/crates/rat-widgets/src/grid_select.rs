//! Grid selection dialog with 2D navigation.

use rat_chrome::overlay_frame;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::popup_chrome::{
    POPUP_MARGIN, centered_overlay_model, clamped_axis, themed_overlay_style, title, usize_to_u16,
};
use crate::theme::WidgetTheme;

const GRID_CELL_WIDTH: u16 = 20;
const GRID_BORDER_COLUMNS: u16 = 2;
const GRID_BORDER_ROWS: u16 = 2;
const EMPTY_GRID_WIDTH: u16 = 30;
const EMPTY_GRID_HEIGHT: u16 = 3;

#[derive(Debug, Clone)]
pub struct GridItem {
    pub label: String,
    pub color: Option<Color>,
}

impl GridItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// Pure data model for a grid selection — no ratatui dependency.
pub struct GridSelectModel {
    pub items: Vec<GridItem>,
    pub columns: usize,
    pub selected: usize,
    pub visible: bool,
}

impl GridSelectModel {
    pub fn new(items: Vec<GridItem>, columns: usize) -> Self {
        Self {
            items,
            columns: columns.max(1), // Ensure at least 1 column
            selected: 0,
            visible: true,
        }
    }

    pub fn move_left(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(self.columns);
    }

    pub fn move_down(&mut self) {
        let new_pos = self.selected + self.columns;
        if new_pos < self.items.len() {
            self.selected = new_pos;
        } else if !self.items.is_empty() {
            // Clamp to last item
            self.selected = self.items.len() - 1;
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_item(&self) -> Option<&GridItem> {
        self.items.get(self.selected)
    }

    pub fn set_selected(&mut self, idx: usize) {
        if !self.items.is_empty() {
            self.selected = idx.min(self.items.len() - 1);
        }
    }
}

pub struct GridSelect {
    pub model: GridSelectModel,
    pub title: String,
}

impl GridSelect {
    pub fn new(title: impl Into<String>, items: Vec<GridItem>, columns: usize) -> Self {
        Self {
            model: GridSelectModel::new(items, columns),
            title: title.into(),
        }
    }

    pub fn move_left(&mut self) {
        self.model.move_left();
    }

    pub fn move_right(&mut self) {
        self.model.move_right();
    }

    pub fn move_up(&mut self) {
        self.model.move_up();
    }

    pub fn move_down(&mut self) {
        self.model.move_down();
    }

    pub fn selected_index(&self) -> usize {
        self.model.selected_index()
    }

    pub fn selected_item(&self) -> Option<&GridItem> {
        self.model.selected_item()
    }

    pub fn set_selected(&mut self, idx: usize) {
        self.model.set_selected(idx);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_themed(frame, area, &WidgetTheme::default());
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        if !self.model.visible {
            return;
        }

        if self.model.items.is_empty() {
            self.render_empty(frame, area, theme);
            return;
        }

        let model = grid_overlay_model(
            area,
            &self.title,
            self.model.items.len(),
            self.model.columns,
        );
        let style = themed_overlay_style(theme, theme.primary);
        let layout = overlay_frame(frame, area, &model, &style);

        if layout.inner.width == 0 || layout.inner.height == 0 {
            return;
        }

        self.render_grid(frame, layout.inner, theme);
    }

    fn render_empty(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        let model = empty_grid_overlay_model(area, &self.title);
        let style = themed_overlay_style(theme, theme.primary);
        let layout = overlay_frame(frame, area, &model, &style);

        if layout.inner.width == 0 || layout.inner.height == 0 {
            return;
        }

        let line = Line::from(Span::styled("(empty)", theme.text_muted));
        frame.render_widget(Paragraph::new(line), layout.inner);
    }

    fn render_grid(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        let rows = self.model.items.len().div_ceil(self.model.columns);

        // Create row constraints
        let row_constraints: Vec<Constraint> = (0..rows).map(|_| Constraint::Length(1)).collect();

        let row_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        for row in 0..rows {
            if row >= row_layout.len() {
                break;
            }

            let row_area = row_layout[row];

            // Create column constraints for this row
            let items_in_row = ((row + 1) * self.model.columns).min(self.model.items.len())
                - row * self.model.columns;
            let col_constraints: Vec<Constraint> = (0..items_in_row)
                .map(|_| Constraint::Ratio(1, items_in_row as u32))
                .collect();

            let col_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(row_area);

            for col in 0..items_in_row {
                let item_index = row * self.model.columns + col;
                if item_index >= self.model.items.len() {
                    break;
                }

                let item = &self.model.items[item_index];
                let cell_area = col_layout[col];

                let is_selected = item_index == self.model.selected;
                self.render_cell(frame, cell_area, item, is_selected, theme);
            }
        }
    }

    fn render_cell(
        &self,
        frame: &mut Frame,
        area: Rect,
        item: &GridItem,
        selected: bool,
        theme: &WidgetTheme,
    ) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let style = if selected {
            theme.highlight_style()
        } else {
            Style::default().fg(theme.text)
        };

        let mut spans = Vec::new();

        // Add color swatch if present
        if let Some(color) = item.color {
            spans.push(Span::styled("█ ", Style::default().fg(color)));
        }

        // Add label (truncate if necessary)
        let available_width = if item.color.is_some() {
            area.width.saturating_sub(2) // Account for swatch + space
        } else {
            area.width
        } as usize;

        let label = if item.label.len() > available_width {
            format!("{}…", &item.label[..available_width.saturating_sub(1)])
        } else {
            item.label.clone()
        };

        spans.push(Span::styled(label, style));

        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }
}

fn grid_overlay_model(
    area: Rect,
    title_text: &str,
    item_count: usize,
    columns: usize,
) -> rat_chrome::OverlayModel {
    let rows = item_count.div_ceil(columns.max(1));
    let content_columns = columns
        .saturating_mul(usize::from(GRID_CELL_WIDTH))
        .saturating_add(columns.saturating_sub(1));
    let preferred_width = usize_to_u16(content_columns).saturating_add(GRID_BORDER_COLUMNS);
    let preferred_height = usize_to_u16(rows).saturating_add(GRID_BORDER_ROWS);
    let width = clamped_axis(area.width, preferred_width, POPUP_MARGIN);
    let height = clamped_axis(area.height, preferred_height, POPUP_MARGIN);

    centered_overlay_model(width, height, title(title_text))
}

fn empty_grid_overlay_model(area: Rect, title_text: &str) -> rat_chrome::OverlayModel {
    let width = clamped_axis(area.width, EMPTY_GRID_WIDTH, POPUP_MARGIN);
    let height = EMPTY_GRID_HEIGHT.min(area.height);

    centered_overlay_model(width, height, title(title_text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_select() {
        let items = vec![
            GridItem::new("Item 1"),
            GridItem::new("Item 2"),
            GridItem::new("Item 3"),
        ];
        let grid = GridSelect::new("Test", items, 2);

        assert_eq!(grid.title, "Test");
        assert_eq!(grid.model.items.len(), 3);
        assert_eq!(grid.model.columns, 2);
        assert_eq!(grid.model.selected, 0);
        assert!(grid.model.visible);
    }

    #[test]
    fn new_empty_grid() {
        let grid = GridSelect::new("Empty", vec![], 3);
        assert_eq!(grid.model.items.len(), 0);
        assert_eq!(grid.model.selected_index(), 0);
        assert!(grid.model.selected_item().is_none());
    }

    #[test]
    fn columns_minimum_one() {
        let items = vec![GridItem::new("Item")];
        let grid = GridSelect::new("Test", items, 0);
        assert_eq!(grid.model.columns, 1);
    }

    #[test]
    fn move_right_within_bounds() {
        let items = vec![GridItem::new("A"), GridItem::new("B"), GridItem::new("C")];
        let mut grid = GridSelect::new("Test", items, 3);

        assert_eq!(grid.model.selected, 0);
        grid.move_right();
        assert_eq!(grid.model.selected, 1);
        grid.move_right();
        assert_eq!(grid.model.selected, 2);
    }

    #[test]
    fn move_right_at_end_stays() {
        let items = vec![GridItem::new("A"), GridItem::new("B")];
        let mut grid = GridSelect::new("Test", items, 2);

        grid.set_selected(1); // Last item
        grid.move_right();
        assert_eq!(grid.model.selected, 1); // Should stay at last item
    }

    #[test]
    fn move_left_within_bounds() {
        let items = vec![GridItem::new("A"), GridItem::new("B"), GridItem::new("C")];
        let mut grid = GridSelect::new("Test", items, 3);

        grid.set_selected(2);
        assert_eq!(grid.model.selected, 2);
        grid.move_left();
        assert_eq!(grid.model.selected, 1);
        grid.move_left();
        assert_eq!(grid.model.selected, 0);
    }

    #[test]
    fn move_left_at_start_stays() {
        let items = vec![GridItem::new("A"), GridItem::new("B")];
        let mut grid = GridSelect::new("Test", items, 2);

        assert_eq!(grid.model.selected, 0);
        grid.move_left();
        assert_eq!(grid.model.selected, 0); // Should stay at 0
    }

    #[test]
    fn move_down_by_columns() {
        let items = vec![
            GridItem::new("A"),
            GridItem::new("B"),
            GridItem::new("C"),
            GridItem::new("D"),
            GridItem::new("E"),
            GridItem::new("F"),
        ];
        let mut grid = GridSelect::new("Test", items, 3);

        grid.set_selected(1); // Second item in first row
        grid.move_down();
        assert_eq!(grid.model.selected, 4); // Second item in second row (1 + 3)
    }

    #[test]
    fn move_down_clamps_to_last() {
        let items = vec![
            GridItem::new("A"),
            GridItem::new("B"),
            GridItem::new("C"),
            GridItem::new("D"),
            GridItem::new("E"), // 5 items in 3-column grid
        ];
        let mut grid = GridSelect::new("Test", items, 3);

        grid.set_selected(2); // Third item in first row
        grid.move_down();
        assert_eq!(grid.model.selected, 4); // Should clamp to last item (index 4)
    }

    #[test]
    fn move_up_by_columns() {
        let items = vec![
            GridItem::new("A"),
            GridItem::new("B"),
            GridItem::new("C"),
            GridItem::new("D"),
            GridItem::new("E"),
            GridItem::new("F"),
        ];
        let mut grid = GridSelect::new("Test", items, 3);

        grid.set_selected(4); // Second item in second row
        grid.move_up();
        assert_eq!(grid.model.selected, 1); // Second item in first row (4 - 3)
    }

    #[test]
    fn move_up_clamps_to_zero() {
        let items = vec![
            GridItem::new("A"),
            GridItem::new("B"),
            GridItem::new("C"),
            GridItem::new("D"),
        ];
        let mut grid = GridSelect::new("Test", items, 3);

        grid.set_selected(1); // Second item in first row
        grid.move_up();
        assert_eq!(grid.model.selected, 0); // Should saturating_sub to 0
    }

    #[test]
    fn selected_item_access() {
        let items = vec![GridItem::new("First"), GridItem::new("Second")];
        let mut grid = GridSelect::new("Test", items, 2);

        assert_eq!(grid.model.selected_item().unwrap().label, "First");
        grid.set_selected(1);
        assert_eq!(grid.model.selected_item().unwrap().label, "Second");

        // Out of bounds
        grid.set_selected(10);
        assert_eq!(grid.model.selected_index(), 1); // Should clamp to last valid index
    }

    #[test]
    fn grid_item_with_color() {
        let item = GridItem::new("Red Item").with_color(Color::Red);
        assert_eq!(item.label, "Red Item");
        assert_eq!(item.color, Some(Color::Red));
    }

    #[test]
    fn grid_item_without_color() {
        let item = GridItem::new("Plain Item");
        assert_eq!(item.label, "Plain Item");
        assert_eq!(item.color, None);
    }

    #[test]
    fn set_selected_clamps() {
        let items = vec![GridItem::new("A"), GridItem::new("B")];
        let mut grid = GridSelect::new("Test", items, 2);

        grid.set_selected(5); // Way out of bounds
        assert_eq!(grid.model.selected, 1); // Should clamp to last valid index (1)
    }

    #[test]
    fn empty_grid_navigation() {
        let mut grid = GridSelect::new("Empty", vec![], 3);

        // All operations should be safe on empty grid
        grid.move_left();
        grid.move_right();
        grid.move_up();
        grid.move_down();
        assert_eq!(grid.model.selected, 0);
        assert!(grid.model.selected_item().is_none());
    }

    #[test]
    fn complex_navigation_scenario() {
        // 8 items in 3 columns:
        // 0 1 2
        // 3 4 5
        // 6 7
        let items: Vec<GridItem> = (0..8)
            .map(|i| GridItem::new(format!("Item {}", i)))
            .collect();
        let mut grid = GridSelect::new("Test", items, 3);

        // Start at 0
        assert_eq!(grid.model.selected, 0);

        // Right to 1
        grid.move_right();
        assert_eq!(grid.model.selected, 1);

        // Down to 4
        grid.move_down();
        assert_eq!(grid.model.selected, 4);

        // Right to 5
        grid.move_right();
        assert_eq!(grid.model.selected, 5);

        // Down to 7 (clamped from 5+3=8 to last item 7)
        grid.move_down();
        assert_eq!(grid.model.selected, 7);

        // Up to 4
        grid.move_up();
        assert_eq!(grid.model.selected, 4);

        // Left to 3
        grid.move_left();
        assert_eq!(grid.model.selected, 3);
    }

    const VIEWPORT_WIDTH: u16 = 80;
    const VIEWPORT_HEIGHT: u16 = 16;
    const TINY_VIEWPORT_WIDTH: u16 = 1;
    const TINY_VIEWPORT_HEIGHT: u16 = 1;
    const BACKGROUND_CELL: &str = "x";
    const GRID_TITLE: &str = "Grid";

    #[test]
    fn visible_grid_select_uses_chrome_and_renders_selected_cell() {
        let grid = sample_grid();

        let buffer = render_grid_select(&grid, viewport());
        let layout = grid_layout(viewport(), &grid);

        assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, Color::Blue);
        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].symbol(), "A");
        assert!(
            buffer[(layout.inner.x, layout.inner.y)]
                .modifier
                .contains(ratatui::style::Modifier::BOLD)
        );
    }

    #[test]
    fn empty_grid_select_reports_empty_inside_chrome() {
        let grid = GridSelect::new(GRID_TITLE, Vec::new(), 2);

        let buffer = render_grid_select(&grid, viewport());
        let layout = empty_grid_layout(viewport(), &grid);

        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].symbol(), "(");
        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].fg, Color::DarkGray);
    }

    #[test]
    fn hidden_grid_select_is_noop() {
        let mut grid = sample_grid();
        grid.model.visible = false;

        let expected = render_background(viewport());
        let actual = render_grid_select(&grid, viewport());

        assert_eq!(actual, expected);
    }

    #[test]
    fn tiny_grid_select_area_does_not_panic() {
        let grid = sample_grid();

        let buffer = render_grid_select(&grid, tiny_viewport());

        assert_eq!(buffer.area.width, TINY_VIEWPORT_WIDTH);
        assert_eq!(buffer.area.height, TINY_VIEWPORT_HEIGHT);
    }

    fn sample_grid() -> GridSelect {
        GridSelect::new(
            GRID_TITLE,
            vec![GridItem::new("Alpha"), GridItem::new("Beta")],
            2,
        )
    }

    fn render_grid_select(grid: &GridSelect, viewport: Rect) -> ratatui::buffer::Buffer {
        let backend =
            ratatui::backend::TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_test_background(frame, area);
                grid.render(frame, area);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn render_background(viewport: Rect) -> ratatui::buffer::Buffer {
        let backend =
            ratatui::backend::TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
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

    fn grid_layout(viewport: Rect, grid: &GridSelect) -> rat_chrome::OverlayLayout {
        rat_chrome::compute_overlay_layout(
            viewport,
            &grid_overlay_model(
                viewport,
                &grid.title,
                grid.model.items.len(),
                grid.model.columns,
            ),
        )
    }

    fn empty_grid_layout(viewport: Rect, grid: &GridSelect) -> rat_chrome::OverlayLayout {
        rat_chrome::compute_overlay_layout(
            viewport,
            &empty_grid_overlay_model(viewport, &grid.title),
        )
    }

    fn viewport() -> Rect {
        Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn tiny_viewport() -> Rect {
        Rect::new(0, 0, TINY_VIEWPORT_WIDTH, TINY_VIEWPORT_HEIGHT)
    }
}
