//! Renderer-neutral query, filtering, sorting, grouping, and facet logic.
//!
//! Queries are structured data first. Text query parsing can be layered above
//! this module later, but renderers and widgets can already build typed queries
//! directly and evaluate them against schema-backed records.
//!
//! # Example
//!
//! ```
//! use std::collections::BTreeMap;
//! use ratcore::query::{
//!     FieldSpec, FieldType, Predicate, Query, QueryOperator, QuerySchema,
//!     QueryValue, SortDirection, SortKey, evaluate_records,
//! };
//!
//! let schema = QuerySchema::new(vec![
//!     FieldSpec::new("title", FieldType::Text),
//!     FieldSpec::new("priority", FieldType::Number),
//! ]);
//! let mut row = BTreeMap::new();
//! row.insert("title".to_string(), QueryValue::Text("Ship docs".to_string()));
//! row.insert("priority".to_string(), QueryValue::Number(1));
//! let query = Query::new()
//!     .with_predicate(Predicate::field(
//!         "title",
//!         QueryOperator::Contains,
//!         QueryValue::Text("docs".to_string()),
//!     ))
//!     .with_sort(SortKey::new("priority", SortDirection::Ascending));
//! let result = evaluate_records(&schema, &query, &[row]).unwrap();
//! assert_eq!(result.rows.len(), 1);
//! ```

const EMPTY_SEARCH_SCORE: u16 = 0;
const FIELD_MATCH_SCORE: u16 = 1;
const DEFAULT_GROUP_KEY: &str = "";

mod atoms;
mod checks;
mod matching;
mod parts;
mod runtime;

pub use atoms::FieldSpec;
pub use atoms::FieldType;
pub use atoms::QueryOperator;
pub use atoms::QuerySchema;
pub use atoms::QuerySurface;
pub use atoms::QueryValue;
pub use atoms::surface_schema;
pub use checks::QueryError;
pub use checks::QueryErrorCode;
pub use checks::validate_query;
pub use parts::DateRange;
pub use parts::DateRangeClause;
pub use parts::Predicate;
pub use parts::Query;
pub use parts::SearchTerm;
pub use parts::SortDirection;
pub use parts::SortKey;
pub use runtime::QueryEvaluation;
pub use runtime::QueryRecord;
pub use runtime::QueryRow;
pub use runtime::evaluate_records;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
