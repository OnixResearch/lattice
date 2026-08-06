use super::*;

const FIELD_TITLE: &str = "title";
const FIELD_TAGS: &str = "tags";
const FIELD_DATE: &str = "date";
const FIELD_PRIORITY: &str = "priority";
const FIELD_DONE: &str = "done";
const UNKNOWN_FIELD: &str = "missing";
const TAG_RUST: &str = "rust";
const TAG_DOCS: &str = "docs";
const TAG_UI: &str = "ui";
const SEARCH_TERM: &str = "ship";
const FIRST_INDEX: usize = 0;
const SECOND_INDEX: usize = 1;
const THIRD_INDEX: usize = 2;
const LOW_PRIORITY: i64 = 1;
const MEDIUM_PRIORITY: i64 = 2;
const HIGH_PRIORITY: i64 = 3;
const DATE_YEAR_FACTOR: i32 = 10_000;
const DATE_MONTH_FACTOR: i32 = 100;
const TEST_YEAR: i32 = 2026;
const TEST_MONTH: i32 = 6;
const FIRST_DAY: i32 = 1;
const FIFTH_DAY: i32 = 5;
const LAST_DAY: i32 = 30;
const START_DATE: i32 = TEST_YEAR * DATE_YEAR_FACTOR + TEST_MONTH * DATE_MONTH_FACTOR + FIRST_DAY;
const MIDDLE_DATE: i32 = TEST_YEAR * DATE_YEAR_FACTOR + TEST_MONTH * DATE_MONTH_FACTOR + FIFTH_DAY;
const END_DATE: i32 = TEST_YEAR * DATE_YEAR_FACTOR + TEST_MONTH * DATE_MONTH_FACTOR + LAST_DAY;

fn row(
    title: &str,
    tags: Vec<&str>,
    date: i32,
    priority: i64,
    done: bool,
) -> std::collections::BTreeMap<String, QueryValue> {
    let mut row = std::collections::BTreeMap::new();
    row.insert(FIELD_TITLE.to_string(), QueryValue::Text(title.to_string()));
    row.insert(FIELD_TAGS.to_string(), QueryValue::Tags(tags.into_iter().map(ToOwned::to_owned).collect()));
    row.insert(FIELD_DATE.to_string(), QueryValue::DateOrdinal(date));
    row.insert(FIELD_PRIORITY.to_string(), QueryValue::Number(priority));
    row.insert(FIELD_DONE.to_string(), QueryValue::Bool(done));
    row
}

fn records() -> Vec<std::collections::BTreeMap<String, QueryValue>> {
    vec![
        row("Ship command palette", vec![TAG_RUST, TAG_UI], MIDDLE_DATE, HIGH_PRIORITY, false),
        row("Write docs", vec![TAG_DOCS], START_DATE, LOW_PRIORITY, false),
        row("Ship viewport docs", vec![TAG_RUST, TAG_DOCS], END_DATE, MEDIUM_PRIORITY, true),
    ]
}

#[test]
fn evaluates_predicates_sorting_grouping_facets_and_search() {
    let query = Query::new()
        .with_predicate(Predicate::And(vec![
            Predicate::field(FIELD_TITLE, QueryOperator::Contains, QueryValue::Text(SEARCH_TERM.to_string())),
            Predicate::field(FIELD_DONE, QueryOperator::Equals, QueryValue::Bool(false)),
        ]))
        .with_search(SearchTerm::new(vec![FIELD_TITLE.to_string(), FIELD_TAGS.to_string()], SEARCH_TERM))
        .with_date_range(DateRangeClause::new(FIELD_DATE, DateRange::new(START_DATE, END_DATE)))
        .with_sort(SortKey::new(FIELD_PRIORITY, SortDirection::Descending))
        .with_group_by(FIELD_DONE)
        .with_facet(FIELD_TAGS);
    let output = evaluate_records(&surface_schema(QuerySurface::Agenda), &query, &records()).unwrap();
    assert_eq!(output.rows.len(), 1);
    assert_eq!(output.rows[0].index, FIRST_INDEX);
    assert_eq!(output.rows[0].score, FIELD_MATCH_SCORE);
    assert_eq!(output.rows[0].matched_fields, vec![FIELD_TITLE.to_string()]);
    assert_eq!(output.groups.get("false"), Some(&vec![FIRST_INDEX]));
    let tag_facets = output.facets.get(FIELD_TAGS).unwrap();
    assert_eq!(tag_facets.get(TAG_RUST), Some(&1));
    assert_eq!(tag_facets.get(TAG_UI), Some(&1));
}

#[test]
fn compound_or_and_not_predicates_work() {
    let query = Query::new().with_predicate(Predicate::And(vec![
        Predicate::Or(vec![
            Predicate::field(FIELD_TAGS, QueryOperator::Contains, QueryValue::Tags(vec![TAG_DOCS.to_string()])),
            Predicate::field(FIELD_PRIORITY, QueryOperator::GreaterThan, QueryValue::Number(MEDIUM_PRIORITY)),
        ]),
        Predicate::Not(Box::new(Predicate::field(FIELD_DONE, QueryOperator::Equals, QueryValue::Bool(true)))),
    ]));
    let output = evaluate_records(&surface_schema(QuerySurface::Table), &query, &records()).unwrap();
    assert_eq!(output.rows.len(), 2);
    assert_eq!(output.rows[0].index, FIRST_INDEX);
    assert_eq!(output.rows[1].index, SECOND_INDEX);
}

#[test]
fn adapter_schemas_share_predicate_semantics() {
    let query = Query::new().with_predicate(Predicate::field(
        FIELD_TAGS,
        QueryOperator::Contains,
        QueryValue::Tags(vec![TAG_DOCS.to_string()]),
    ));
    let agenda = evaluate_records(&surface_schema(QuerySurface::Agenda), &query, &records()).unwrap();
    let table = evaluate_records(&surface_schema(QuerySurface::Table), &query, &records()).unwrap();
    assert_eq!(agenda.rows, table.rows);
    assert_eq!(agenda.rows.len(), 2);
    assert_eq!(agenda.rows[0].index, SECOND_INDEX);
    assert_eq!(agenda.rows[1].index, THIRD_INDEX);
}

#[test]
fn unknown_field_is_rejected() {
    let query = Query::new().with_predicate(Predicate::field(
        UNKNOWN_FIELD,
        QueryOperator::Equals,
        QueryValue::Text("x".to_string()),
    ));
    let error = evaluate_records(&surface_schema(QuerySurface::Agenda), &query, &records()).unwrap_err();
    assert_eq!(error.code, QueryErrorCode::UnknownField);
    assert_eq!(error.field, UNKNOWN_FIELD);
}

#[test]
fn unsupported_operator_is_rejected() {
    let query =
        Query::new().with_predicate(Predicate::field(FIELD_DONE, QueryOperator::Contains, QueryValue::Bool(false)));
    let error = validate_query(&surface_schema(QuerySurface::Agenda), &query).unwrap_err();
    assert_eq!(error.code, QueryErrorCode::UnsupportedOperator);
    assert_eq!(error.field, FIELD_DONE);
}

#[test]
fn type_mismatch_is_rejected() {
    let query = Query::new().with_predicate(Predicate::field(
        FIELD_PRIORITY,
        QueryOperator::Equals,
        QueryValue::Text("high".to_string()),
    ));
    let error = validate_query(&surface_schema(QuerySurface::Agenda), &query).unwrap_err();
    assert_eq!(error.code, QueryErrorCode::TypeMismatch);
    assert_eq!(error.field, FIELD_PRIORITY);
}

#[test]
fn invalid_date_range_is_rejected() {
    let query = Query::new().with_date_range(DateRangeClause::new(FIELD_DATE, DateRange::new(END_DATE, START_DATE)));
    let error = validate_query(&surface_schema(QuerySurface::Agenda), &query).unwrap_err();
    assert_eq!(error.code, QueryErrorCode::InvalidRange);
    assert_eq!(error.field, FIELD_DATE);
}
