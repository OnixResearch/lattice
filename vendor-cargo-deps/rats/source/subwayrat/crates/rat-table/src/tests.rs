mod render;

use super::*;

#[test]
fn test_data_table_creation() {
    let columns = vec!["Name".to_string(), "Age".to_string()];
    let rows = vec![
        vec!["Alice".to_string(), "25".to_string()],
        vec!["Bob".to_string(), "30".to_string()],
    ];

    let table = DataTable::new(columns.clone(), rows.clone());

    assert_eq!(table.columns, columns);
    assert_eq!(table.rows, rows);
    assert_eq!(table.selected_row, 0);
    assert_eq!(table.scroll_col, 0);
    assert_eq!(table.row_count(), 2);
    assert_eq!(table.column_count(), 2);
}

#[test]
fn test_navigation() {
    let mut table = DataTable::new(
        vec!["Col1".to_string()],
        vec![
            vec!["Row1".to_string()],
            vec!["Row2".to_string()],
            vec!["Row3".to_string()],
        ],
    );

    assert_eq!(table.selected_index(), 0);

    table.select_next();
    assert_eq!(table.selected_index(), 1);

    table.select_last();
    assert_eq!(table.selected_index(), 2);

    table.select_prev();
    assert_eq!(table.selected_index(), 1);

    table.select_first();
    assert_eq!(table.selected_index(), 0);
}

#[test]
fn test_scrolling() {
    let mut table = DataTable::new(
        vec!["Col1".to_string(), "Col2".to_string(), "Col3".to_string()],
        vec![vec!["A".to_string(), "B".to_string(), "C".to_string()]],
    );

    assert_eq!(table.scroll_col, 0);

    table.scroll_right();
    assert_eq!(table.scroll_col, 1);

    table.scroll_right();
    assert_eq!(table.scroll_col, 2);

    // Should not scroll past last column
    table.scroll_right();
    assert_eq!(table.scroll_col, 2);

    table.scroll_left();
    assert_eq!(table.scroll_col, 1);
}

#[test]
fn test_column_width_calculation() {
    let columns = vec!["Short".to_string(), "Very Long Header".to_string()];
    let rows = vec![
        vec!["A".to_string(), "Small".to_string()],
        vec!["Very Long Content".to_string(), "B".to_string()],
    ];

    let widths = calculate_column_widths(&columns, &rows, WidthBounds::new(5, 40));

    // First column: max("Short".len(), "Very Long Content".len()) = 17
    // Second column: max("Very Long Header".len(), "Small".len()) = 16
    assert_eq!(widths[0], 17);
    assert_eq!(widths[1], 16);
}

#[test]
fn test_truncation() {
    let result = truncate_cell("This is a very long string", 10, "...");
    assert_eq!(result, "This is..."); // 10 - 3 = 7 chars of content

    let result = truncate_cell("Short", 10, "...");
    assert_eq!(result, "Short");

    // Test edge case where suffix is longer than max width
    let result = truncate_cell("Test", 2, "...");
    assert_eq!(result, "..");
}

#[test]
fn test_clear() {
    let mut table = DataTable::new(vec!["Col1".to_string()], vec![vec!["Row1".to_string()]]);

    assert!(!table.is_empty());
    table.clear();
    assert!(table.is_empty());
    assert_eq!(table.row_count(), 0);
    assert_eq!(table.column_count(), 0);
}

#[test]
fn test_selected_row() {
    let mut table = DataTable::new(
        vec!["Name".to_string()],
        vec![vec!["Alice".to_string()], vec!["Bob".to_string()]],
    );

    assert_eq!(
        table.selected_row(),
        Some(vec!["Alice".to_string()].as_slice())
    );

    table.select_next();
    assert_eq!(
        table.selected_row(),
        Some(vec!["Bob".to_string()].as_slice())
    );
}

#[test]
fn empty_table_operations() {
    let mut table = DataTable::new(vec![], vec![]);

    assert!(table.is_empty());
    assert_eq!(table.row_count(), 0);
    assert_eq!(table.column_count(), 0);
    assert_eq!(table.selected_row(), None);
    assert_eq!(table.selected_index(), 0);

    // Navigation on empty table should not panic.
    table.select_next();
    table.select_prev();
    table.select_first();
    table.select_last();
    table.scroll_left();
    table.scroll_right();
    assert_eq!(table.selected_index(), 0);
}

#[test]
fn navigation_clamps_at_boundaries() {
    let mut table = DataTable::new(
        vec!["X".to_string()],
        vec![vec!["a".to_string()], vec!["b".to_string()]],
    );

    // prev at 0 stays at 0
    table.select_prev();
    assert_eq!(table.selected_index(), 0);

    // next past last stays at last
    table.select_last();
    assert_eq!(table.selected_index(), 1);
    table.select_next();
    assert_eq!(table.selected_index(), 1);
}

#[test]
fn scroll_clamps_at_boundaries() {
    let mut table = DataTable::new(
        vec!["A".to_string(), "B".to_string()],
        vec![vec!["1".to_string(), "2".to_string()]],
    );

    // left at 0 stays at 0
    table.scroll_left();
    assert_eq!(table.scroll_col, 0);

    // right stops at last column index
    table.scroll_right();
    assert_eq!(table.scroll_col, 1);
    table.scroll_right();
    assert_eq!(table.scroll_col, 1);
}

#[test]
fn with_max_cell_width_clamps_columns() {
    let table = DataTable::new(
        vec!["Header".to_string()],
        vec![vec![
            "A very long cell value that exceeds the cap".to_string(),
        ]],
    )
    .with_max_cell_width(10);

    assert!(table.column_widths.iter().all(|&w| w <= 10));
}

#[test]
fn with_min_cell_width_enforces_floor() {
    let table =
        DataTable::new(vec!["H".to_string()], vec![vec!["X".to_string()]]).with_min_cell_width(12);

    assert!(table.column_widths.iter().all(|&w| w >= 12));
}

#[test]
fn set_data_resets_state() {
    let mut table = DataTable::new(
        vec!["Old".to_string()],
        vec![
            vec!["r1".to_string()],
            vec!["r2".to_string()],
            vec!["r3".to_string()],
        ],
    );
    table.select_last();
    table.scroll_right();

    table.set_data(
        vec!["New".to_string(), "Cols".to_string()],
        vec![vec!["a".to_string(), "b".to_string()]],
    );

    assert_eq!(table.selected_index(), 0);
    assert_eq!(table.scroll_col, 0);
    assert_eq!(table.column_count(), 2);
    assert_eq!(table.row_count(), 1);
}

#[test]
fn info_reports_truncation() {
    let table = DataTable::new(
        vec!["Name".to_string()],
        vec![vec![
            "A string longer than the forty character maximum width".to_string(),
        ]],
    );

    let info = table.info();
    assert!(info.is_truncated);
    assert_eq!(info.row_count, 1);
    assert_eq!(info.column_count, 1);
}

#[test]
fn info_no_truncation_for_short_cells() {
    let table = DataTable::new(vec!["Name".to_string()], vec![vec!["Short".to_string()]]);

    let info = table.info();
    assert!(!info.is_truncated);
}

#[test]
fn column_widths_clamped_to_min_max() {
    let widths = calculate_column_widths(
        &["X".to_string()],
        &[vec!["Y".to_string()]],
        WidthBounds::new(8, 20),
    );
    // Both "X" and "Y" are 1 char, but min is 8.
    assert_eq!(widths[0], 8);

    let widths = calculate_column_widths(
        &["A".to_string()],
        &[vec![
            "This string is way beyond the twenty char max".to_string(),
        ]],
        WidthBounds::new(5, 20),
    );
    assert_eq!(widths[0], 20);
}

#[test]
fn rows_with_fewer_columns_than_header() {
    let table = DataTable::new(
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec![vec!["only_one".to_string()]],
    );

    // Should have widths for all 3 header columns.
    assert_eq!(table.column_widths.len(), 3);
    // Missing cells don't crash selected_row.
    assert_eq!(table.selected_row().unwrap().len(), 1);
}
