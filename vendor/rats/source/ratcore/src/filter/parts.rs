use super::*;

/// Predicate tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    /// Always matches.
    Always,
    /// Field predicate.
    Field {
        /// Field name.
        field: String,
        /// Operator.
        operator: QueryOperator,
        /// Comparison value.
        value: QueryValue,
    },
    /// All child predicates must match.
    And(Vec<Predicate>),
    /// At least one child predicate must match.
    Or(Vec<Predicate>),
    /// Child predicate must not match.
    Not(Box<Predicate>),
}

impl Predicate {
    /// Creates a field predicate.
    #[must_use]
    pub fn field(field: impl Into<String>, operator: QueryOperator, value: QueryValue) -> Self {
        Self::Field {
            field: field.into(),
            operator,
            value,
        }
    }
}

/// Search term over one or more fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTerm {
    /// Fields to search.
    pub fields: Vec<String>,
    /// Term to find.
    pub term: String,
}

impl SearchTerm {
    /// Creates a search term.
    #[must_use]
    pub fn new(fields: Vec<String>, term: impl Into<String>) -> Self {
        Self {
            fields,
            term: term.into(),
        }
    }
}

/// Date range over a date-ordinal field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    /// Inclusive start date ordinal.
    pub start: i32,
    /// Inclusive end date ordinal.
    pub end: i32,
}

impl DateRange {
    /// Creates a date range.
    #[must_use]
    pub fn new(start: i32, end: i32) -> Self {
        Self { start, end }
    }

    pub(super) fn contains(self, value: i32) -> bool {
        self.start <= value && value <= self.end
    }
}

/// Query date range clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRangeClause {
    /// Date field name.
    pub field: String,
    /// Inclusive date range.
    pub range: DateRange,
}

impl DateRangeClause {
    /// Creates a date range clause.
    #[must_use]
    pub fn new(field: impl Into<String>, range: DateRange) -> Self {
        Self {
            field: field.into(),
            range,
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order.
    Ascending,
    /// Descending order.
    Descending,
}

/// Sort key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortKey {
    /// Field name.
    pub field: String,
    /// Sort direction.
    pub direction: SortDirection,
}

impl SortKey {
    /// Creates a sort key.
    #[must_use]
    pub fn new(field: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            field: field.into(),
            direction,
        }
    }
}

/// Structured query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    /// Predicate tree.
    pub predicate: Predicate,
    /// Search terms.
    pub search_terms: Vec<SearchTerm>,
    /// Date range clauses.
    pub date_ranges: Vec<DateRangeClause>,
    /// Sort keys.
    pub sort_keys: Vec<SortKey>,
    /// Optional group field.
    pub group_by: Option<String>,
    /// Facet fields.
    pub facets: Vec<String>,
}

impl Query {
    /// Creates an empty query that matches all records.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the predicate tree.
    #[must_use]
    pub fn with_predicate(mut self, predicate: Predicate) -> Self {
        self.predicate = predicate;
        self
    }

    /// Adds a search term.
    #[must_use]
    pub fn with_search(mut self, search: SearchTerm) -> Self {
        self.search_terms.push(search);
        self
    }

    /// Adds a date range clause.
    #[must_use]
    pub fn with_date_range(mut self, date_range: DateRangeClause) -> Self {
        self.date_ranges.push(date_range);
        self
    }

    /// Adds a sort key.
    #[must_use]
    pub fn with_sort(mut self, sort: SortKey) -> Self {
        self.sort_keys.push(sort);
        self
    }

    /// Sets a group field.
    #[must_use]
    pub fn with_group_by(mut self, field: impl Into<String>) -> Self {
        self.group_by = Some(field.into());
        self
    }

    /// Adds a facet field.
    #[must_use]
    pub fn with_facet(mut self, field: impl Into<String>) -> Self {
        self.facets.push(field.into());
        self
    }
}

impl Default for Query {
    fn default() -> Self {
        Self {
            predicate: Predicate::Always,
            search_terms: Vec::new(),
            date_ranges: Vec::new(),
            sort_keys: Vec::new(),
            group_by: None,
            facets: Vec::new(),
        }
    }
}
