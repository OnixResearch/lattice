/// Query error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryErrorCode {
    /// Query references a field absent from the schema.
    UnknownField,
    /// Query uses an operator not supported by the field schema.
    UnsupportedOperator,
    /// Query value type does not match the field schema.
    TypeMismatch,
    /// Date range start is greater than end.
    InvalidRange,
}

/// Deterministic query error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError {
    /// Stable error code.
    pub code: QueryErrorCode,
    /// Field related to the error.
    pub field: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl QueryError {
    pub(super) fn new(
        code: QueryErrorCode,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            field: field.into(),
            message: message.into(),
        }
    }
}

use super::*;

/// Validates a query against a schema.
///
/// # Errors
///
/// Returns [`QueryError`] when the query references an unknown field, unsupported
/// operator, mismatched value type, or invalid date range.
pub fn validate_query(schema: &QuerySchema, query: &Query) -> Result<(), QueryError> {
    debug_assert!(query.search_terms.len() <= query.search_terms.capacity());
    debug_assert!(query.sort_keys.len() <= query.sort_keys.capacity());
    validate_predicate(schema, &query.predicate)?;
    for search in &query.search_terms {
        for field in &search.fields {
            let spec = require_field(schema, field)?;
            if !matches!(spec.value_type, FieldType::Text | FieldType::Tags) {
                return Err(QueryError::new(
                    QueryErrorCode::TypeMismatch,
                    field,
                    "search fields must be text or tags",
                ));
            }
        }
    }
    for clause in &query.date_ranges {
        let spec = require_field(schema, &clause.field)?;
        if spec.value_type != FieldType::DateOrdinal {
            return Err(QueryError::new(
                QueryErrorCode::TypeMismatch,
                &clause.field,
                "date range field must be DateOrdinal",
            ));
        }
        if clause.range.start > clause.range.end {
            return Err(QueryError::new(
                QueryErrorCode::InvalidRange,
                &clause.field,
                "date range start must be less than or equal to end",
            ));
        }
    }
    for sort in &query.sort_keys {
        require_field(schema, &sort.field)?;
    }
    if let Some(group_field) = &query.group_by {
        require_field(schema, group_field)?;
    }
    for facet in &query.facets {
        require_field(schema, facet)?;
    }
    Ok(())
}

fn validate_predicate(schema: &QuerySchema, predicate: &Predicate) -> Result<(), QueryError> {
    let mut pending = Vec::with_capacity(1);
    pending.push(predicate);
    debug_assert_eq!(pending.len(), 1);
    debug_assert!(pending.capacity() >= pending.len());
    while let Some(current) = pending.pop() {
        match current {
            Predicate::Always => {}
            Predicate::Field {
                field,
                operator,
                value,
            } => validate_field_predicate(schema, field, *operator, value)?,
            Predicate::And(children) | Predicate::Or(children) => pending.extend(children.iter()),
            Predicate::Not(child) => pending.push(child),
        }
    }
    Ok(())
}

fn validate_field_predicate(
    schema: &QuerySchema,
    field: &str,
    operator: QueryOperator,
    value: &QueryValue,
) -> Result<(), QueryError> {
    let spec = require_field(schema, field)?;
    if !spec.operators.contains(&operator) {
        return Err(QueryError::new(
            QueryErrorCode::UnsupportedOperator,
            field,
            "operator is not supported by field",
        ));
    }
    if spec.value_type != value.field_type() {
        return Err(QueryError::new(
            QueryErrorCode::TypeMismatch,
            field,
            "predicate value type does not match field type",
        ));
    }
    Ok(())
}

fn require_field<'a>(schema: &'a QuerySchema, field: &str) -> Result<&'a FieldSpec, QueryError> {
    schema.field(field).ok_or_else(|| {
        QueryError::new(
            QueryErrorCode::UnknownField,
            field,
            "field is not present in query schema",
        )
    })
}
