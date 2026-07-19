//! Selection list dialog

use rat_chrome::overlay_frame;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{List, ListItem};

use crate::popup_chrome::{
    POPUP_MARGIN, centered_overlay_model, clamped_axis, themed_overlay_style, title, usize_to_u16,
};
use crate::theme::WidgetTheme;

const SELECT_LIST_WIDTH: u16 = 40;
const BORDER_ROWS: u16 = 2;

/// Pure data model for a selection list — no ratatui dependency.
pub struct SelectListModel {
    pub items: Vec<String>,
    pub selected: usize,
    pub visible: bool,
}

impl SelectListModel {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected: 0,
            visible: true,
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        self.selected = (self.selected + 1).min(self.items.len().saturating_sub(1));
    }

    pub fn select(&self) -> Option<String> {
        self.items.get(self.selected).cloned()
    }
}

pub struct SelectList {
    pub model: SelectListModel,
    pub title: String,
}

impl SelectList {
    pub fn new(title: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            model: SelectListModel::new(items),
            title: title.into(),
        }
    }

    pub fn move_up(&mut self) {
        self.model.move_up();
    }

    pub fn move_down(&mut self) {
        self.model.move_down();
    }

    pub fn select(&self) -> Option<String> {
        self.model.select()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_themed(frame, area, &WidgetTheme::default());
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        if !self.model.visible {
            return;
        }

        let model = select_overlay_model(area, &self.title, self.model.items.len());
        let style = themed_overlay_style(theme, theme.primary);
        let layout = overlay_frame(frame, area, &model, &style);

        if layout.inner.width == 0 || layout.inner.height == 0 {
            return;
        }

        frame.render_widget(select_list_items(&self.model, theme), layout.inner);
    }
}

fn select_overlay_model(
    area: Rect,
    title_text: &str,
    item_count: usize,
) -> rat_chrome::OverlayModel {
    let width = clamped_axis(area.width, SELECT_LIST_WIDTH, POPUP_MARGIN);
    let preferred_height = usize_to_u16(item_count).saturating_add(BORDER_ROWS);
    let height = clamped_axis(area.height, preferred_height, POPUP_MARGIN);

    centered_overlay_model(width, height, title(title_text))
}

fn select_list_items<'a>(model: &'a SelectListModel, theme: &WidgetTheme) -> List<'a> {
    let items: Vec<ListItem> = model
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let style = if index == model.selected {
                theme.highlight_style()
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(Span::styled(item.clone(), style))
        })
        .collect();

    List::new(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rat_chrome::compute_overlay_layout;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::{Color, Modifier};
    use ratatui::text::Line;
    use ratatui::widgets::Paragraph;

    const VIEWPORT_WIDTH: u16 = 60;
    const VIEWPORT_HEIGHT: u16 = 12;
    const TINY_VIEWPORT_WIDTH: u16 = 1;
    const TINY_VIEWPORT_HEIGHT: u16 = 1;
    const BACKGROUND_CELL: &str = "x";
    const TITLE: &str = "Choose";
    const FIRST_ITEM: &str = "Alpha";
    const SECOND_ITEM: &str = "Beta";

    #[test]
    fn visible_select_list_uses_chrome_and_highlights_selection() {
        let mut list = sample_list();
        list.move_down();

        let buffer = render_select_list(&list, viewport());
        let layout = select_layout(viewport(), &list);
        let selected_y = layout.inner.y + u16::try_from(list.model.selected).unwrap();

        assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, Color::Blue);
        assert_eq!(buffer[(layout.inner.x, selected_y)].symbol(), "B");
        assert!(
            buffer[(layout.inner.x, selected_y)]
                .modifier
                .contains(Modifier::BOLD)
        );
    }

    #[test]
    fn hidden_select_list_is_noop() {
        let mut list = sample_list();
        list.model.visible = false;

        let expected = render_background(viewport());
        let actual = render_select_list(&list, viewport());

        assert_eq!(actual, expected);
    }

    #[test]
    fn tiny_select_list_area_does_not_panic() {
        let list = sample_list();

        let buffer = render_select_list(&list, tiny_viewport());

        assert_eq!(buffer.area.width, TINY_VIEWPORT_WIDTH);
        assert_eq!(buffer.area.height, TINY_VIEWPORT_HEIGHT);
    }

    fn sample_list() -> SelectList {
        SelectList::new(TITLE, vec![FIRST_ITEM.to_string(), SECOND_ITEM.to_string()])
    }

    fn render_select_list(list: &SelectList, viewport: Rect) -> Buffer {
        let backend = TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_test_background(frame, area);
                list.render(frame, area);
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

    fn select_layout(viewport: Rect, list: &SelectList) -> rat_chrome::OverlayLayout {
        compute_overlay_layout(
            viewport,
            &select_overlay_model(viewport, &list.title, list.model.items.len()),
        )
    }

    fn viewport() -> Rect {
        Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn tiny_viewport() -> Rect {
        Rect::new(0, 0, TINY_VIEWPORT_WIDTH, TINY_VIEWPORT_HEIGHT)
    }
}
