impl super::DataTable {
    /// Render the data table.
    pub fn render(
        &self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        block: Option<ratatui::widgets::Block>,
        style: &super::DataTableStyle,
    ) {
        let render_area = if let Some(block) = block.clone() {
            let inner = block.inner(area);
            frame.render_widget(block, area);
            inner
        } else {
            area
        };

        if self.columns.is_empty() {
            draw_empty_table(frame, render_area);
            return;
        }

        let visible_columns = self.get_visible_columns(render_area.width);
        if visible_columns.is_empty() {
            draw_empty_table(frame, render_area);
            return;
        }

        let constraints: Vec<ratatui::layout::Constraint> = visible_columns
            .iter()
            .map(|(_, width)| {
                ratatui::layout::Constraint::Length(width.saturating_add(style.column_spacing))
            })
            .collect();

        let header_cells: Vec<ratatui::widgets::Cell> = visible_columns
            .iter()
            .map(|(col_name, width)| {
                let display_name = super::truncate_cell(col_name, *width, &style.truncation_suffix);
                ratatui::widgets::Cell::from(display_name).style(style.header_style)
            })
            .collect();
        let header_row = ratatui::widgets::Row::new(header_cells);

        let data_rows: Vec<ratatui::widgets::Row> = self
            .rows
            .iter()
            .enumerate()
            .map(|(row_idx, row_data)| {
                let cells: Vec<ratatui::widgets::Cell> = visible_columns
                    .iter()
                    .enumerate()
                    .map(|(visible_idx, (_, width))| {
                        let col_idx = self.scroll_col.saturating_add(visible_idx);
                        let cell_content = row_data.get(col_idx).map_or("", String::as_str);
                        let display_content =
                            super::truncate_cell(cell_content, *width, &style.truncation_suffix);

                        let cell_style = if row_idx == self.selected_row {
                            style.selected_style
                        } else {
                            style.normal_style
                        };

                        ratatui::widgets::Cell::from(display_content).style(cell_style)
                    })
                    .collect();
                ratatui::widgets::Row::new(cells)
            })
            .collect();

        let table = ratatui::widgets::Table::new(data_rows, constraints).header(header_row);
        let mut table_state = ratatui::widgets::TableState::new();
        if !self.rows.is_empty() {
            table_state.select(Some(self.selected_row));
        }

        frame.render_stateful_widget(table, render_area, &mut table_state);
    }

    /// Recalculate column widths based on current min/max settings.
    pub(crate) fn recalculate_widths(&mut self) {
        self.column_widths = super::calculate_column_widths(
            &self.columns,
            &self.rows,
            super::WidthBounds::new(self.min_cell_width, self.max_cell_width),
        );
    }

    /// Get visible columns that fit within the given width.
    pub(crate) fn get_visible_columns(&self, available_width: u16) -> Vec<(String, u16)> {
        let mut visible = Vec::with_capacity(self.columns.len().saturating_sub(self.scroll_col));
        let mut used_width = 0_u16;

        for column_index in self.scroll_col..self.columns.len() {
            if let (Some(col_name), Some(&col_width)) = (
                self.columns.get(column_index),
                self.column_widths.get(column_index),
            ) {
                let spacing = if visible.is_empty() {
                    0
                } else {
                    super::DEFAULT_COLUMN_SPACING
                };
                let needed_width = col_width.saturating_add(spacing);

                if used_width.saturating_add(needed_width) <= available_width {
                    visible.push((col_name.clone(), col_width));
                    used_width = used_width.saturating_add(needed_width);
                } else {
                    break;
                }
            }
        }

        visible
    }
}

fn draw_empty_table(frame: &mut ratatui::Frame, render_area: ratatui::layout::Rect) {
    let rows: Vec<ratatui::widgets::Row> = Vec::new();
    let constraints: Vec<ratatui::layout::Constraint> = Vec::new();
    frame.render_widget(ratatui::widgets::Table::new(rows, constraints), render_area);
}
