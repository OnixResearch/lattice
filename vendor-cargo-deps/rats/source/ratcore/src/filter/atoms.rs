/// Supported field value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldType {
    /// Free text field.
    Text,
    /// Signed integer number field.
    Number,
    /// Boolean field.
    Bool,
    /// Date represented as a caller-owned ordinal.
    DateOrdinal,
    /// Tag/list field.
    Tags,
}

/// Query value used by predicates, records, facets, and sorting.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryValue {
    /// Text value.
    Text(String),
    /// Signed integer value.
    Number(i64),
    /// Boolean value.
    Bool(bool),
    /// Date ordinal value.
    DateOrdinal(i32),
    /// Tag/list value.
    Tags(Vec<String>),
}

impl QueryValue {
    /// Returns the field type for this value.
    #[must_use]
    pub fn field_type(&self) -> FieldType {
        match self {
            Self::Text(_) => FieldType::Text,
            Self::Number(_) => FieldType::Number,
            Self::Bool(_) => FieldType::Bool,
            Self::DateOrdinal(_) => FieldType::DateOrdinal,
            Self::Tags(_) => FieldType::Tags,
        }
    }

    pub(super) fn text_contains(&self, needle: &str) -> bool {
        let needle_lower = needle.to_lowercase();
        match self {
            Self::Text(value) => value.to_lowercase().contains(&needle_lower),
            Self::Tags(values) => values.iter().any(|value| value.to_lowercase().contains(&needle_lower)),
            Self::Number(_) | Self::Bool(_) | Self::DateOrdinal(_) => false,
        }
    }

    pub(super) fn text_starts_with(&self, prefix: &str) -> bool {
        let prefix_lower = prefix.to_lowercase();
        match self {
            Self::Text(value) => value.to_lowercase().starts_with(&prefix_lower),
            Self::Tags(values) => values.iter().any(|value| value.to_lowercase().starts_with(&prefix_lower)),
            Self::Number(_) | Self::Bool(_) | Self::DateOrdinal(_) => false,
        }
    }

    pub(super) fn facet_values(&self) -> Vec<String> {
        match self {
            Self::Text(value) => vec![value.clone()],
            Self::Number(value) => vec![value.to_string()],
            Self::Bool(value) => vec![value.to_string()],
            Self::DateOrdinal(value) => vec![value.to_string()],
            Self::Tags(values) => values.clone(),
        }
    }
}

/// Supported field predicate operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QueryOperator {
    /// Field equals value.
    Equals,
    /// Field does not equal value.
    NotEquals,
    /// Text or tags contain value.
    Contains,
    /// Text or tags start with value.
    StartsWith,
    /// Field is greater than value.
    GreaterThan,
    /// Field is less than value.
    LessThan,
}

/// Field schema and supported operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    /// Stable field name.
    pub name: String,
    /// Field value type.
    pub value_type: FieldType,
    /// Operators supported by the field.
    pub operators: std::collections::BTreeSet<QueryOperator>,
}

impl FieldSpec {
    /// Creates a field schema with default operators for the value type.
    #[must_use]
    pub fn new(name: impl Into<String>, value_type: FieldType) -> Self {
        Self {
            name: name.into(),
            value_type,
            operators: default_operators(value_type),
        }
    }

    /// Overrides supported operators.
    #[must_use]
    pub fn with_operators(mut self, operators: std::collections::BTreeSet<QueryOperator>) -> Self {
        self.operators = operators;
        self
    }
}

/// Query schema for a data source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuerySchema {
    fields: std::collections::BTreeMap<String, FieldSpec>,
}

impl QuerySchema {
    /// Creates a schema from field specs. Later duplicate names replace earlier ones.
    #[must_use]
    pub fn new(fields: Vec<FieldSpec>) -> Self {
        Self {
            fields: fields.into_iter().map(|field| (field.name.clone(), field)).collect(),
        }
    }

    /// Returns a field spec by name.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&FieldSpec> {
        self.fields.get(name)
    }

    /// Returns all fields in deterministic name order.
    #[must_use]
    pub fn fields(&self) -> Vec<&FieldSpec> {
        self.fields.values().collect()
    }
}

fn default_operators(value_type: FieldType) -> std::collections::BTreeSet<QueryOperator> {
    match value_type {
        FieldType::Text | FieldType::Tags => [
            QueryOperator::Equals,
            QueryOperator::NotEquals,
            QueryOperator::Contains,
            QueryOperator::StartsWith,
        ]
        .into_iter()
        .collect(),
        FieldType::Number | FieldType::DateOrdinal => [
            QueryOperator::Equals,
            QueryOperator::NotEquals,
            QueryOperator::GreaterThan,
            QueryOperator::LessThan,
        ]
        .into_iter()
        .collect(),
        FieldType::Bool => [QueryOperator::Equals, QueryOperator::NotEquals].into_iter().collect(),
    }
}

/// Common Rats data surfaces with conventional field schemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuerySurface {
    /// Agenda item data.
    Agenda,
    /// Table row data.
    Table,
    /// Tag index data.
    Tags,
    /// Backlink data.
    Backlinks,
    /// Branch list data.
    Branches,
    /// Fuzzy candidate data.
    Fuzzy,
    /// Command palette action data.
    CommandPalette,
}

/// Returns a conventional schema for a Rats data surface.
#[must_use]
pub fn surface_schema(surface: QuerySurface) -> QuerySchema {
    match surface {
        QuerySurface::Agenda | QuerySurface::Table => QuerySchema::new(vec![
            FieldSpec::new("title", FieldType::Text),
            FieldSpec::new("tags", FieldType::Tags),
            FieldSpec::new("date", FieldType::DateOrdinal),
            FieldSpec::new("priority", FieldType::Number),
            FieldSpec::new("done", FieldType::Bool),
        ]),
        QuerySurface::Tags => QuerySchema::new(vec![
            FieldSpec::new("tag", FieldType::Text),
            FieldSpec::new("count", FieldType::Number),
        ]),
        QuerySurface::Backlinks => QuerySchema::new(vec![
            FieldSpec::new("source", FieldType::Text),
            FieldSpec::new("target", FieldType::Text),
            FieldSpec::new("tags", FieldType::Tags),
        ]),
        QuerySurface::Branches => QuerySchema::new(vec![
            FieldSpec::new("name", FieldType::Text),
            FieldSpec::new("depth", FieldType::Number),
        ]),
        QuerySurface::Fuzzy => QuerySchema::new(vec![
            FieldSpec::new("text", FieldType::Text),
            FieldSpec::new("score", FieldType::Number),
        ]),
        QuerySurface::CommandPalette => QuerySchema::new(vec![
            FieldSpec::new("label", FieldType::Text),
            FieldSpec::new("category", FieldType::Text),
            FieldSpec::new("shortcut", FieldType::Text),
            FieldSpec::new("enabled", FieldType::Bool),
        ]),
    }
}
