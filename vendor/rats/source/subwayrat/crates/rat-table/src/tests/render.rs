#[test]
fn visible_columns_respects_scroll_and_width() {
    let table = crate::DataTable::new(
        vec![
            "AAAAAAAAAA".to_string(),
            "BBBBBBBBBB".to_string(),
            "CCCCCCCCCC".to_string(),
        ],
        vec![vec![
            "1234567890".to_string(),
            "1234567890".to_string(),
            "1234567890".to_string(),
        ]],
    );

    let visible = table.get_visible_columns(200);
    assert_eq!(visible.len(), 3);

    let visible = table.get_visible_columns(15);
    assert_eq!(visible.len(), 1);
}
