//! Renderer-neutral knowledge workflow adapter core.
//!
//! This module normalizes application-owned document, heading, tag, backlink,
//! agenda, capture, search, and graph data into panel-ready models. It performs
//! deterministic validation only; applications remain responsible for storage,
//! commits, indexing, and filesystem writes.

const PANEL_OUTLINE: &str = "outline";
const PANEL_CAPTURE: &str = "capture";
const PANEL_TAGS: &str = "tags";
const PANEL_BACKLINKS: &str = "backlinks";
const PANEL_AGENDA: &str = "agenda";
const PANEL_MARKDOWN: &str = "markdown";
const PANEL_SEARCH: &str = "search";
const PANEL_GRAPH: &str = "graph";
const TAG_PREFIX: char = '#';

mod assembly;
pub use assembly::{KnowledgePanels, KnowledgeWorkflowBuild, build_knowledge_workflow};

/// Normalized document supplied by an application adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeDocument {
    /// Stable document ID.
    pub id: String,
    /// Display title.
    pub title: String,
    /// Raw or rendered body text owned by the application.
    pub body: String,
}

impl KnowledgeDocument {
    /// Creates a normalized document.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body: body.into(),
        }
    }
}

/// Normalized heading record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeHeading {
    /// Owning document ID.
    pub document_id: String,
    /// Heading text.
    pub title: String,
    /// Nesting level, starting at one.
    pub level: u8,
}

impl KnowledgeHeading {
    /// Creates a heading.
    #[must_use]
    pub fn new(document_id: impl Into<String>, title: impl Into<String>, level: u8) -> Self {
        Self {
            document_id: document_id.into(),
            title: title.into(),
            level,
        }
    }
}

/// Normalized tag assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeTag {
    /// Owning document ID.
    pub document_id: String,
    /// Tag label, including the `#` prefix.
    pub label: String,
}

impl KnowledgeTag {
    /// Creates a tag assignment.
    #[must_use]
    pub fn new(document_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            label: label.into(),
        }
    }
}

/// Normalized backlink edge between documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeBacklink {
    /// Source document ID.
    pub source_id: String,
    /// Target document ID.
    pub target_id: String,
}

impl KnowledgeBacklink {
    /// Creates a backlink edge.
    #[must_use]
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
        }
    }
}

/// Normalized agenda item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeAgendaItem {
    /// Owning document ID.
    pub document_id: String,
    /// Item title.
    pub title: String,
    /// Inclusive start date in `YYYY-MM-DD` form.
    pub start: String,
    /// Optional inclusive end date in `YYYY-MM-DD` form.
    pub end: Option<String>,
}

impl KnowledgeAgendaItem {
    /// Creates an agenda item.
    #[must_use]
    pub fn new(
        document_id: impl Into<String>,
        title: impl Into<String>,
        start: impl Into<String>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            title: title.into(),
            start: start.into(),
            end: None,
        }
    }

    /// Sets an end date.
    #[must_use]
    pub fn with_end(mut self, end: impl Into<String>) -> Self {
        self.end = Some(end.into());
        self
    }
}

/// Application-owned capture target status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeCaptureTarget {
    /// Stable capture target ID.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Whether this target can currently accept capture content.
    pub writable: bool,
}

impl KnowledgeCaptureTarget {
    /// Creates a capture target.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, writable: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            writable,
        }
    }
}

/// Search index record supplied by the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeSearchRecord {
    /// Owning document ID.
    pub document_id: String,
    /// Searchable title or snippet.
    pub text: String,
}

impl KnowledgeSearchRecord {
    /// Creates a search record.
    #[must_use]
    pub fn new(document_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            text: text.into(),
        }
    }
}

/// Graph edge between knowledge records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeGraphEdge {
    /// Source node ID.
    pub source_id: String,
    /// Target node ID.
    pub target_id: String,
}

impl KnowledgeGraphEdge {
    /// Creates a graph edge.
    #[must_use]
    pub fn new(source_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            target_id: target_id.into(),
        }
    }
}

/// Application adapter boundary for supplying knowledge workspace data.
pub trait KnowledgeWorkspaceAdapter {
    /// Returns a normalized snapshot owned by the application adapter.
    fn workspace_snapshot(&self) -> KnowledgeWorkspaceInput;
}

/// Adapter snapshot supplied by an application.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KnowledgeWorkspaceInput {
    /// Documents in the workspace.
    pub documents: Vec<KnowledgeDocument>,
    /// Heading records.
    pub headings: Vec<KnowledgeHeading>,
    /// Tag assignments.
    pub tags: Vec<KnowledgeTag>,
    /// Backlink edges.
    pub backlinks: Vec<KnowledgeBacklink>,
    /// Agenda items.
    pub agenda: Vec<KnowledgeAgendaItem>,
    /// Capture targets.
    pub captures: Vec<KnowledgeCaptureTarget>,
    /// Search records.
    pub search: Vec<KnowledgeSearchRecord>,
    /// Graph edges.
    pub graph: Vec<KnowledgeGraphEdge>,
}

/// Stable knowledge diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KnowledgeIssueCode {
    /// No documents were supplied.
    EmptyDocuments,
    /// A document ID is duplicated.
    DuplicateDocument,
    /// A heading is malformed.
    MalformedHeading,
    /// A tag label is invalid.
    InvalidTag,
    /// A link references a missing document.
    MissingLinkTarget,
    /// An agenda date range is invalid.
    BadDateRange,
    /// A capture target cannot accept writes.
    FailedCaptureTarget,
    /// Search data is required but empty.
    EmptySearch,
}

/// Deterministic knowledge diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeIssue {
    /// Stable issue code.
    pub code: KnowledgeIssueCode,
    /// Affected record ID or logical field.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl KnowledgeIssue {
    fn new(
        code: KnowledgeIssueCode,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            subject: subject.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
