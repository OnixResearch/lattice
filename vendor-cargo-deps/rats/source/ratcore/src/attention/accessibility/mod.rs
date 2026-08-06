//! Bounded renderer-neutral accessibility semantics.
//!
//! Semantic snapshots describe accepted application projections. They are not a
//! second domain model and contain no renderer geometry or platform objects.

semantic_string_id! {
    /// Stable accessibility semantic node identity.
    pub struct SemanticNodeId;
}

semantic_string_id! {
    /// Stable semantic surface identity.
    pub struct SemanticSurfaceId;
}

/// Stable diagnostic codes for semantic validation, updates, routing, and evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticIssueCode {
    /// A required identity is empty.
    EmptyIdentity,
    /// A node identity occurs more than once.
    DuplicateNode,
    /// A surface identity occurs more than once.
    DuplicateSurface,
    /// A surface root is missing.
    MissingRoot,
    /// A surface has an invalid additional root.
    InvalidRoot,
    /// A referenced surface is missing.
    MissingSurface,
    /// A referenced parent is missing.
    MissingParent,
    /// Parent links contain a cycle.
    ParentCycle,
    /// Siblings use the same explicit order.
    DuplicateOrder,
    /// A relationship target is missing.
    MissingRelation,
    /// Relationships contain a cycle.
    RelationCycle,
    /// A relationship occurs more than once.
    DuplicateRelation,
    /// A required accessible name is missing.
    MissingName,
    /// State is incompatible with the semantic role.
    IncompatibleState,
    /// Value metadata is malformed or incompatible.
    InvalidValue,
    /// Collection metadata is malformed or incompatible.
    InvalidCollection,
    /// An action is incompatible with the semantic role.
    IncompatibleAction,
    /// An action has no valid application target.
    MissingActionTarget,
    /// Announcement metadata is malformed.
    InvalidAnnouncement,
    /// A focus identity occurs in more than one link.
    DuplicateFocusLink,
    /// A linked focus node is missing.
    MissingFocusNode,
    /// A focus and semantic role cannot be linked.
    IncompatibleFocusRole,
    /// An update generation is stale.
    StaleGeneration,
    /// A replacement generation does not advance.
    InvalidGeneration,
    /// An update targets the same node more than once.
    DuplicateOperation,
    /// An update references an unknown node.
    UnknownNode,
    /// An action request targets a hidden node.
    HiddenNode,
    /// An action request targets a disabled node.
    DisabledNode,
    /// An action is not exposed by the node.
    UnsupportedAction,
    /// An action value is missing or malformed.
    InvalidActionValue,
    /// Renderer support is absent without an explicit reason.
    MissingSupportReason,
    /// Evidence generation does not match its snapshot.
    StaleEvidence,
    /// Semantic evidence is mislabeled as platform runtime proof.
    MisclassifiedEvidence,
}

/// Deterministic semantic diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemanticIssue {
    /// Stable issue code.
    pub code: SemanticIssueCode,
    /// Stable node, surface, focus, fixture, or request subject.
    pub subject: String,
    /// Human-readable explanation.
    pub message: String,
}

impl SemanticIssue {
    pub(super) fn new(code: SemanticIssueCode, subject: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Ordered semantic validation result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SemanticValidation {
    /// Stable ordered diagnostics.
    pub issues: Vec<SemanticIssue>,
}

impl SemanticValidation {
    /// Returns true when no issue rejected the candidate.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

mod fixtures;
mod output;
mod schema;
mod state;
mod validation;

pub use fixtures::SemanticEvidence;
pub use fixtures::SemanticEvidenceKind;
pub use fixtures::menu_dialog_semantic_fixture;
pub use fixtures::semantic_fixture_evidence;
pub use fixtures::semantic_snapshot_hash;
pub use fixtures::tree_semantic_fixture;
pub use fixtures::validate_semantic_evidence;
pub use output::NativeSemanticJoinKey;
pub use output::SemanticFeature;
pub use output::SemanticProjection;
pub use output::SemanticProjectionNode;
pub use output::SemanticRendererTarget;
pub use output::SemanticSupport;
pub use output::SemanticSupportLevel;
pub use output::project_semantic_summary;
pub use output::semantic_support;
pub use schema::SemanticAction;
pub use schema::SemanticAnnouncement;
pub use schema::SemanticChecked;
pub use schema::SemanticCollection;
pub use schema::SemanticCurrent;
pub use schema::SemanticFocusLink;
pub use schema::SemanticGeneration;
pub use schema::SemanticNode;
pub use schema::SemanticPoliteness;
pub use schema::SemanticProperty;
pub use schema::SemanticRange;
pub use schema::SemanticRelation;
pub use schema::SemanticRelationKind;
pub use schema::SemanticRole;
pub use schema::SemanticSnapshot;
pub use schema::SemanticState;
pub use schema::SemanticStateKind;
pub use schema::SemanticSurface;
pub use schema::SemanticValue;
pub use state::SemanticActionIntent;
pub use state::SemanticActionOutcome;
pub use state::SemanticActionRequest;
pub use state::SemanticActionValue;
pub use state::SemanticChange;
pub use state::SemanticTree;
pub use state::SemanticUpdate;
pub use state::SemanticUpdateOperation;
pub use state::route_semantic_action;
pub use validation::validate_semantic_snapshot;

#[cfg(test)]
mod tests;
