use super::*;

/// Queryable record interface.
pub trait QueryRecord {
    /// Gets a field value by name.
    fn get_value(&self, field: &str) -> Option<&QueryValue>;
}

impl QueryRecord for std::collections::BTreeMap<String, QueryValue> {
    fn get_value(&self, field: &str) -> Option<&QueryValue> {
        self.get(field)
    }
}

/// Matched row metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRow {
    /// Original record index.
    pub index: usize,
    /// Search score from matched search fields.
    pub score: u16,
    /// Fields that matched search terms.
    pub matched_fields: Vec<String>,
    /// Optional group key.
    pub group_key: Option<String>,
}

/// Query evaluation output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryEvaluation {
    /// Matching rows in deterministic output order.
    pub rows: Vec<QueryRow>,
    /// Group key to original record indices.
    pub groups: std::collections::BTreeMap<String, Vec<usize>>,
    /// Facet field to facet value counts.
    pub facets: std::collections::BTreeMap<String, std::collections::BTreeMap<String, usize>>,
}

/// Evaluates records with a schema-backed structured query.
///
/// # Errors
///
/// Returns [`QueryError`] when validation finds unknown fields, unsupported
/// operators, type mismatches, or invalid date ranges.
pub fn evaluate_records<R: QueryRecord>(
    schema: &QuerySchema,
    query: &Query,
    records: &[R],
) -> Result<QueryEvaluation, QueryError> {
    validate_query(schema, query)?;
    let mut rows = Vec::with_capacity(records.len());
    debug_assert!(rows.capacity() >= records.len());
    debug_assert!(rows.len() <= records.len());
    for (index, record) in records.iter().enumerate() {
        if !matching::predicate_matches(record, &query.predicate) {
            continue;
        }
        if !matching::date_ranges_match(record, &query.date_ranges) {
            continue;
        }
        let (search_matches, score) = matching::search_matches(record, &query.search_terms);
        if !search_matches {
            continue;
        }
        let group_key = query
            .group_by
            .as_ref()
            .and_then(|field| record.get_value(field))
            .map(group_key_for_value);
        rows.push(QueryRow {
            index,
            score,
            matched_fields: matching::matched_fields(record, &query.search_terms),
            group_key,
        });
    }
    debug_assert!(rows.len() <= records.len());
    debug_assert!(rows.iter().all(|row| row.index < records.len()));
    sort_rows(&mut rows, records, &query.sort_keys);
    Ok(QueryEvaluation {
        groups: collect_groups(&rows),
        facets: collect_facets(&rows, records, &query.facets),
        rows,
    })
}

fn sort_rows<R: QueryRecord>(rows: &mut [QueryRow], records: &[R], sort_keys: &[SortKey]) {
    rows.sort_by(|left, right| {
        for sort in sort_keys {
            let left_value = records[left.index].get_value(&sort.field);
            let right_value = records[right.index].get_value(&sort.field);
            let ordering = left_value.cmp(&right_value);
            let ordering = match sort.direction {
                SortDirection::Ascending => ordering,
                SortDirection::Descending => ordering.reverse(),
            };
            if ordering != std::cmp::Ordering::Equal {
                return ordering;
            }
        }
        left.index.cmp(&right.index)
    });
}

fn group_key_for_value(value: &QueryValue) -> String {
    value
        .facet_values()
        .first()
        .cloned()
        .unwrap_or_else(|| DEFAULT_GROUP_KEY.to_string())
}

fn collect_groups(rows: &[QueryRow]) -> std::collections::BTreeMap<String, Vec<usize>> {
    let mut groups: std::collections::BTreeMap<String, Vec<usize>> =
        std::collections::BTreeMap::new();
    for row in rows {
        if let Some(group_key) = &row.group_key {
            groups.entry(group_key.clone()).or_default().push(row.index);
        }
    }
    groups
}

fn collect_facets<R: QueryRecord>(
    rows: &[QueryRow],
    records: &[R],
    facet_fields: &[String],
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, usize>> {
    facet_fields
        .iter()
        .map(|field| {
            let mut counts = std::collections::BTreeMap::new();
            for row in rows {
                if let Some(value) = records[row.index].get_value(field) {
                    for facet_value in value.facet_values() {
                        *counts.entry(facet_value).or_insert(0) += 1;
                    }
                }
            }
            (field.clone(), counts)
        })
        .collect()
}
