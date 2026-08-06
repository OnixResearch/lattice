mod render;
pub mod virtualization;

#[cfg(test)]
mod tests;

const DEFAULT_MAX_CELL_WIDTH: u16 = 40;
const DEFAULT_MIN_CELL_WIDTH: u16 = 5;
pub(crate) const DEFAULT_COLUMN_SPACING: u16 = 1;
const EMPTY_INDEX: usize = 0;
const EMPTY_COUNT: u64 = 0;

/// Position of one row in the table.
///
/// A row position is a distinct role from a column position
/// ([`TableColumnIndex`]); a column value may not be consumed where a row is
/// required (and vice-versa).
///
/// Supplying a column value where a row is required must not compile.
///
/// ```compile_fail
/// use rat_table::{TableColumnIndex, TableRowIndex};
/// let column = TableColumnIndex::from_zero_based(2);
/// let row: TableRowIndex = column;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TableRowIndex(u64);

impl TableRowIndex {
    /// Creates a row position from a zero-based backing value.
    #[must_use]
    pub const fn from_zero_based(value: u64) -> Self {
        Self(value)
    }

    /// Returns the zero-based backing value for checked row access.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Position of one column in the table.
///
/// A column position is a distinct role from a row position
/// ([`TableRowIndex`]); a row value may not be consumed where a column is
/// required (and vice-versa).
///
/// Supplying a row value where a column is required must not compile.
///
/// ```compile_fail
/// use rat_table::{TableColumnIndex, TableRowIndex};
/// let row = TableRowIndex::from_zero_based(1);
/// let column: TableColumnIndex = row;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TableColumnIndex(u64);

impl TableColumnIndex {
    /// Creates a column position from a zero-based backing value.
    #[must_use]
    pub const fn from_zero_based(value: u64) -> Self {
        Self(value)
    }

    /// Returns the zero-based backing value for checked column access.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A scrollable data table widget for ratatui with auto column sizing
#[derive(Debug, Clone)]
pub struct DataTable {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<u16>,
    selected_row: TableRowIndex,
    scroll_col: TableColumnIndex,
    max_cell_width: u16,
    min_cell_width: u16,
}

/// Styling configuration for the DataTable
#[derive(Debug, Clone)]
pub struct DataTableStyle {
    pub header_style: ratatui::style::Style,
    pub selected_style: ratatui::style::Style,
    pub normal_style: ratatui::style::Style,
    pub truncation_suffix: String,
    pub column_spacing: u16,
}

/// Information about the table for rendering info bars
#[derive(Debug, Clone)]
pub struct DataTableInfo {
    pub row_count: u64,
    pub column_count: u64,
    pub is_truncated: bool,
    pub extra: Vec<(String, String)>,
}

impl Default for DataTableStyle {
    fn default() -> Self {
        Self {
            header_style: ratatui::style::Style::new().bold().yellow(),
            selected_style: ratatui::style::Style::new().bg(ratatui::style::Color::DarkGray).bold(),
            normal_style: ratatui::style::Style::new(),
            truncation_suffix: "...".to_string(),
            column_spacing: DEFAULT_COLUMN_SPACING,
        }
    }
}

impl DataTable {
    /// Create a new DataTable with the given columns and rows
    pub fn new(columns: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        let max_cell_width = DEFAULT_MAX_CELL_WIDTH;
        let min_cell_width = DEFAULT_MIN_CELL_WIDTH;
        let column_widths = calculate_column_widths(&columns, &rows, WidthBounds::new(min_cell_width, max_cell_width));

        Self {
            columns,
            rows,
            column_widths,
            selected_row: TableRowIndex::from_zero_based(EMPTY_INDEX as u64),
            scroll_col: TableColumnIndex::from_zero_based(EMPTY_INDEX as u64),
            max_cell_width,
            min_cell_width,
        }
    }

    /// Set the maximum cell width (builder pattern)
    pub fn with_max_cell_width(mut self, width: u16) -> Self {
        self.max_cell_width = width;
        self.recalculate_widths();
        self
    }

    /// Set the minimum cell width (builder pattern)
    pub fn with_min_cell_width(mut self, width: u16) -> Self {
        self.min_cell_width = width;
        self.recalculate_widths();
        self
    }

    /// Replace the table data and recalculate column widths
    pub fn set_data(&mut self, columns: Vec<String>, rows: Vec<Vec<String>>) {
        self.columns = columns;
        self.rows = rows;
        self.selected_row = TableRowIndex::from_zero_based(EMPTY_INDEX as u64);
        self.scroll_col = TableColumnIndex::from_zero_based(EMPTY_INDEX as u64);
        self.recalculate_widths();
    }

    /// Move selection to the next row
    pub fn select_next(&mut self) {
        if !self.rows.is_empty() {
            self.selected_row = TableRowIndex::from_zero_based(
                self.selected_row.get().saturating_add(1).min(self.rows.len().saturating_sub(1) as u64),
            );
        }
    }

    /// Move selection to the previous row
    pub fn select_prev(&mut self) {
        if self.selected_row.get() > 0 {
            self.selected_row = TableRowIndex::from_zero_based(self.selected_row.get().saturating_sub(1));
        }
    }

    /// Move selection to the first row
    pub fn select_first(&mut self) {
        self.selected_row = TableRowIndex::from_zero_based(EMPTY_INDEX as u64);
    }

    /// Move selection to the last row
    pub fn select_last(&mut self) {
        if !self.rows.is_empty() {
            self.selected_row = TableRowIndex::from_zero_based(self.rows.len().saturating_sub(1) as u64);
        }
    }

    /// Scroll right to show more columns
    pub fn scroll_right(&mut self) {
        if self.scroll_col.get() < self.columns.len().saturating_sub(1) as u64 {
            self.scroll_col = TableColumnIndex::from_zero_based(self.scroll_col.get().saturating_add(1));
        }
    }

    /// Scroll left to show previous columns
    pub fn scroll_left(&mut self) {
        if self.scroll_col.get() > 0 {
            self.scroll_col = TableColumnIndex::from_zero_based(self.scroll_col.get().saturating_sub(1));
        }
    }

    /// Get the currently selected row's data
    pub fn selected_row(&self) -> Option<&[String]> {
        self.rows.get(self.selected_row.get() as usize).map(|row| row.as_slice())
    }

    /// Get the position of the currently selected row
    pub fn selected_index(&self) -> TableRowIndex {
        self.selected_row
    }

    /// Get the number of data rows (excluding header)
    pub fn row_count(&self) -> u64 {
        usize_to_public_count(self.rows.len())
    }

    /// Get the number of columns
    pub fn column_count(&self) -> u64 {
        usize_to_public_count(self.columns.len())
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Clear all data from the table
    pub fn clear(&mut self) {
        self.columns.clear();
        self.rows.clear();
        self.column_widths.clear();
        self.selected_row = TableRowIndex::from_zero_based(EMPTY_INDEX as u64);
        self.scroll_col = TableColumnIndex::from_zero_based(EMPTY_INDEX as u64);
    }

    /// Get table info for rendering info bars
    pub fn info(&self) -> DataTableInfo {
        // Check if any cells were truncated during width calculation
        let is_truncated = self.rows.iter().any(|row| {
            row.iter().enumerate().any(|(i, cell)| {
                if let Some(&width) = self.column_widths.get(i) {
                    usize_to_u16_saturating(cell.len()) > width
                } else {
                    false
                }
            })
        });

        DataTableInfo {
            row_count: self.row_count(),
            column_count: self.column_count(),
            is_truncated,
            extra: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WidthBounds {
    min: u16,
    max: u16,
}

impl WidthBounds {
    const fn new(min: u16, max: u16) -> Self {
        Self { min, max }
    }
}

fn usize_to_public_count(count: usize) -> u64 {
    match u64::try_from(count) {
        Ok(value) => value,
        Err(_) => EMPTY_COUNT,
    }
}

fn usize_to_u16_saturating(count: usize) -> u16 {
    match u16::try_from(count) {
        Ok(value) => value,
        Err(_) => u16::MAX,
    }
}

/// Calculate optimal column widths based on content and constraints
fn calculate_column_widths(columns: &[String], rows: &[Vec<String>], bounds: WidthBounds) -> Vec<u16> {
    let mut widths: Vec<u16> =
        columns.iter().map(|col| usize_to_u16_saturating(col.len()).clamp(bounds.min, bounds.max)).collect();

    for row in rows {
        for (col_idx, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(col_idx) {
                let cell_width = usize_to_u16_saturating(cell.len()).clamp(bounds.min, bounds.max);
                *width = (*width).max(cell_width);
            }
        }
    }

    widths
}

/// Truncate cell content to fit within the specified width
fn truncate_cell(content: &str, max_width: u16, suffix: &str) -> String {
    let max_width_cells = usize::from(max_width);
    let suffix_width_cells = suffix.len();
    if content.len() <= max_width_cells {
        content.to_string()
    } else if max_width_cells <= suffix_width_cells {
        suffix.chars().take(max_width_cells).collect()
    } else {
        let content_width = max_width_cells.saturating_sub(suffix_width_cells);
        format!("{}{}", &content[..content_width], suffix)
    }
}
