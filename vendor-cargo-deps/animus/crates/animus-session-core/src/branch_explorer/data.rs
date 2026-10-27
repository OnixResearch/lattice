// Public explorer DTO names preserve their owning branch-explorer namespace.
#![allow(
    unknown_lints,
    bool_naming,
    compound_condition,
    non_trait_imports,
    path_segment_repetition
)]
#![allow(missing_docs)]
#![allow(
    clippy::missing_errors_doc,
    clippy::struct_excessive_bools,
    clippy::struct_field_names
)]

#[allow(unused_imports)]
use crate::prelude::*;

#[allow(unused_imports)]
use crate::branching::SessionBranchOperation;
use crate::conversation_projection::ConversationProjectionSnapshot;

/// Generated policy schema.
pub const BRANCH_EXPLORER_POLICY_SCHEMA: &str = "animus.session-branch-explorer.policy.v1";
/// Projection schema.
pub const BRANCH_EXPLORER_PROJECTION_SCHEMA: &str = "animus.session-branch-explorer.projection.v1";
/// Receipt schema.
pub const BRANCH_EXPLORER_RECEIPT_SCHEMA: &str = "animus.session-branch-explorer.receipt.v1";
/// Domain for policy identities.
pub const BRANCH_EXPLORER_POLICY_DOMAIN: &str = "animus.session-branch-explorer.policy.v1";
/// Domain for operation identities.
pub const BRANCH_EXPLORER_OPERATION_DOMAIN: &str = "animus.session-branch-explorer.operation.v1";
/// Domain for complete generation identities.
pub const BRANCH_EXPLORER_GENERATION_DOMAIN: &str = "animus.session-branch-explorer.generation.v1";
/// Domain for projection identities.
pub const BRANCH_EXPLORER_PROJECTION_DOMAIN: &str = "animus.session-branch-explorer.projection.v1";
/// Domain for receipts.
pub const BRANCH_EXPLORER_RECEIPT_DOMAIN: &str = "animus.session-branch-explorer.receipt.v1";

/// Stable policy input exported from Nickel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchExplorerPolicyInput {
    pub schema: String,
    pub max_nodes: u32,
    pub max_depth: u32,
    pub max_label_bytes: u32,
    pub max_labels_per_node: u32,
    pub max_ordered_children: u32,
    pub max_comparison_suffix: u32,
    pub max_safe_references: u32,
    pub redaction_first: bool,
    pub one_replay_parent: bool,
    pub annotations_are_sidecars: bool,
    pub external_state_required: bool,
    pub destructive_actions_allowed: bool,
    pub receipt_metadata_only: bool,
    pub non_claims: Vec<String>,
}

/// Validated branch explorer policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchExplorerPolicy {
    pub schema: String,
    pub policy_id: String,
    pub max_nodes: u32,
    pub max_depth: u32,
    pub max_label_bytes: u32,
    pub max_labels_per_node: u32,
    pub max_ordered_children: u32,
    pub max_comparison_suffix: u32,
    pub max_safe_references: u32,
    pub non_claims: Vec<String>,
}

/// Redaction-safe ledger entry class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BranchEntryClass {
    User,
    Assistant,
    Tool,
    Steering,
    Compaction,
    Repair,
    System,
}

impl BranchEntryClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
            Self::Steering => "steering",
            Self::Compaction => "compaction",
            Self::Repair => "repair",
            Self::System => "system",
        }
    }
}

/// Supplied external-world state classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalStatePosture {
    ReadOnly,
    SharedWorld,
    Isolated,
    Unknown,
}

impl ExternalStatePosture {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::SharedWorld => "shared_world",
            Self::Isolated => "isolated",
            Self::Unknown => "unknown",
        }
    }
}

/// One redaction-safe ledger node fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchLedgerNodeInput {
    pub entry_id: String,
    pub parent_id: Option<String>,
    /// A non-empty value proves an unsupported multiple-parent attempt.
    pub extra_parent_id: Option<String>,
    pub entry_class: BranchEntryClass,
    pub storage_sequence: u32,
    pub claimed_leaf: bool,
    pub artifact_reference_ids: Vec<String>,
    pub effect_receipt_ids: Vec<String>,
    pub model_profile_id: Option<String>,
    pub usage_units: Option<u64>,
}

/// One bookmark sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchBookmarkInput {
    pub target_entry_id: String,
    pub label: Option<String>,
}

/// One safe label sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchLabelInput {
    pub target_entry_id: String,
    pub label: String,
}

/// Partial sibling presentation order sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchSiblingOrderInput {
    pub parent_entry_id: String,
    pub ordered_child_ids: Vec<String>,
}

/// Sidecar annotations bound to one ledger revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchAnnotationsInput {
    pub revision_id: String,
    pub bookmarks: Vec<BranchBookmarkInput>,
    pub labels: Vec<BranchLabelInput>,
    pub sibling_orders: Vec<BranchSiblingOrderInput>,
}

/// Complete in-memory source facts for projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchExplorerSourceInput {
    pub session_id: String,
    pub revision_id: String,
    pub ledger_generation: u64,
    pub active_leaf_id: String,
    pub nodes: Vec<BranchLedgerNodeInput>,
    pub annotations: BranchAnnotationsInput,
    pub external_state: Option<ExternalStatePosture>,
    pub isolation_evidence_id: Option<String>,
}

/// One projected redaction-safe node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchProjectionNode {
    pub entry_id: String,
    pub parent_id: Option<String>,
    pub entry_class: BranchEntryClass,
    pub storage_sequence: u32,
    pub depth: u32,
    pub child_count: u32,
    pub is_leaf: bool,
    pub is_active_path: bool,
    pub is_bookmarked: bool,
    pub labels: Vec<String>,
    pub artifact_reference_ids: Vec<String>,
    pub effect_receipt_ids: Vec<String>,
    pub model_profile_id: Option<String>,
    pub usage_units: Option<u64>,
}

/// Explicit bounded projection facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchProjectionTruncation {
    pub omitted_node_count: u32,
    pub node_limit: u32,
}

/// Canonical branch topology projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchProjectionSnapshot {
    pub schema: String,
    pub projection_id: String,
    pub policy_id: String,
    pub session_id: String,
    pub revision_id: String,
    pub ledger_generation: u64,
    pub active_leaf_id: String,
    pub external_state: ExternalStatePosture,
    pub isolation_evidence_id: Option<String>,
    pub nodes: Vec<BranchProjectionNode>,
    pub presentation_order: Vec<String>,
    pub truncation: BranchProjectionTruncation,
}

/// Planned explorer operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchExplorerOperation {
    Inspect,
    ActivateLeaf,
    ForkFromNode,
}

impl BranchExplorerOperation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::ActivateLeaf => "activate_leaf",
            Self::ForkFromNode => "fork_from_node",
        }
    }
}

/// Planner result class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchPlanStatus {
    Admitted,
    Blocked,
}

impl BranchPlanStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Blocked => "blocked",
        }
    }
}

/// Pure host mutation or inspection plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchOperationPlan {
    pub operation_id: String,
    pub operation: BranchExplorerOperation,
    pub status: BranchPlanStatus,
    pub session_id: String,
    pub expected_revision_id: String,
    pub expected_active_leaf_id: String,
    pub target_entry_id: String,
    pub policy_id: String,
    pub external_state: ExternalStatePosture,
    pub continuation_independent: bool,
    pub isolation_evidence_id: Option<String>,
    pub append_only_branch_operation: Option<SessionBranchOperation>,
    pub diagnostic: Option<BranchExplorerIssue>,
}

/// Selected canonical read-view binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchSelectedRead {
    pub branch_projection_id: String,
    pub conversation_projection_id: String,
    pub session_id: String,
    pub revision_id: String,
    pub selected_entry_id: String,
}

impl BranchSelectedRead {
    /// Bind one conversation projection to the same selected branch revision.
    pub fn bind(
        branch: &BranchProjectionSnapshot,
        selected_entry_id: &str,
        conversation: &ConversationProjectionSnapshot,
    ) -> Result<Self, BranchExplorerError> {
        let selected_exists = branch
            .nodes
            .iter()
            .any(|node| node.entry_id == selected_entry_id);
        let exact_session = conversation.cursor.session_id == branch.session_id;
        let exact_revision = conversation.cursor.generation == branch.ledger_generation;
        let exact_leaf = conversation.cursor.leaf_hash == selected_entry_id;
        if !selected_exists || !exact_session || !exact_revision || !exact_leaf {
            return Err(BranchExplorerError::new(
                BranchExplorerIssue::StaleRevision,
                "selected_read",
            ));
        }
        Ok(Self {
            branch_projection_id: branch.projection_id.clone(),
            conversation_projection_id: conversation.snapshot_hash.clone(),
            session_id: branch.session_id.clone(),
            revision_id: branch.revision_id.clone(),
            selected_entry_id: String::from(selected_entry_id),
        })
    }
}

/// Exact bounded branch comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchComparison {
    pub comparison_id: String,
    pub session_id: String,
    pub revision_id: String,
    pub common_ancestor_id: String,
    pub left_target_id: String,
    pub right_target_id: String,
    pub left_suffix_ids: Vec<String>,
    pub right_suffix_ids: Vec<String>,
    pub left_class_counts: Vec<(BranchEntryClass, u32)>,
    pub right_class_counts: Vec<(BranchEntryClass, u32)>,
    pub left_usage_units: Option<u64>,
    pub right_usage_units: Option<u64>,
    pub unavailable_facts: Vec<String>,
    pub truncated: bool,
}

/// Complete facts required to identify one generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchGenerationInput {
    pub source_revision_id: String,
    pub fork_point_id: String,
    pub model_profile_id: String,
    pub parameter_profile_id: String,
    pub tool_plan_id: String,
    pub effect_receipt_ids: Vec<String>,
    pub response_artifact_id: String,
}

/// Non-destructive sibling equivalence sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchSiblingEquivalence {
    pub equivalence_id: String,
    pub parent_entry_id: String,
    pub left_leaf_id: String,
    pub right_leaf_id: String,
    pub generation_id: String,
    pub branches_preserved: bool,
}

/// Stable validation issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BranchExplorerIssue {
    InvalidPolicy,
    EmptyTree,
    DuplicateEntry,
    DuplicateSequence,
    MissingParent,
    MultipleParents,
    Cycle,
    FalseLeaf,
    UnknownTarget,
    UnsafeLabel,
    StaleRevision,
    InvalidSiblingOrder,
    BoundExceeded,
    NonLeafActivation,
    MissingExternalState,
    MissingIsolationEvidence,
    CrossSession,
    MissingCommonAncestor,
    IncompleteGenerationIdentity,
    NotSiblingLeaves,
    GenerationMismatch,
    DestructiveAction,
    ReceiptLeak,
    Overclaim,
}

/// Bounded safe error without source material.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchExplorerError {
    issue: BranchExplorerIssue,
    field: &'static str,
}

impl BranchExplorerError {
    #[must_use]
    pub(crate) const fn new(issue: BranchExplorerIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    #[must_use]
    pub const fn issue(&self) -> BranchExplorerIssue {
        self.issue
    }

    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for BranchExplorerError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "branch explorer field `{}` failed with {:?}",
            self.field, self.issue
        )
    }
}

impl core::error::Error for BranchExplorerError {}

/// Metadata-only receipt input.
#[derive(Clone, Debug)]
pub struct BranchExplorerReceiptInput<'a> {
    pub policy: &'a BranchExplorerPolicy,
    pub projection: &'a BranchProjectionSnapshot,
    pub operation: Option<&'a BranchOperationPlan>,
    pub comparison: Option<&'a BranchComparison>,
    pub equivalence: Option<&'a BranchSiblingEquivalence>,
    pub tui_action_id: Option<&'a str>,
    pub sensitive: BranchExplorerSensitiveDraft<'a>,
    pub claims: BranchExplorerClaimDraft,
}

/// Private fields that are forbidden in receipts.
#[derive(Clone, Copy, Debug, Default)]
pub struct BranchExplorerSensitiveDraft<'a> {
    pub transcript_text: Option<&'a str>,
    pub reasoning: Option<&'a str>,
    pub tool_body: Option<&'a str>,
    pub provider_payload: Option<&'a str>,
    pub credential: Option<&'a str>,
    pub environment_value: Option<&'a str>,
    pub physical_path: Option<&'a str>,
    pub raw_ui_state: Option<&'a str>,
}

/// Claims forbidden at this bounded evidence layer.
#[derive(Clone, Copy, Debug, Default)]
pub struct BranchExplorerClaimDraft {
    pub rollback: bool,
    pub world_isolation: bool,
    pub semantic_quality: bool,
    pub completion: bool,
    pub production_readiness: bool,
    pub release_eligibility: bool,
}

/// Metadata-only explorer receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchExplorerReceipt {
    pub schema: String,
    pub receipt_id: String,
    pub policy_id: String,
    pub projection_id: String,
    pub session_id: String,
    pub revision_id: String,
    pub node_count: u32,
    pub omitted_node_count: u32,
    pub operation_class: Option<String>,
    pub result_class: Option<String>,
    pub comparison_id: Option<String>,
    pub equivalence_id: Option<String>,
    pub tui_action_id: Option<String>,
    pub external_state: ExternalStatePosture,
    pub redaction: String,
    pub non_claims: Vec<String>,
}

/// Read-only projection source port. Implementations remain host-owned.
pub trait BranchProjectionPort {
    type Failure;

    fn load_branch_source(&self) -> Result<BranchExplorerSourceInput, Self::Failure>;
}

/// Annotation persistence port. Reusable logic does not implement it.
pub trait BranchAnnotationPort {
    type Failure;

    fn compare_and_set_annotations(
        &mut self,
        expected_revision_id: &str,
        annotations: &BranchAnnotationsInput,
    ) -> Result<(), Self::Failure>;
}

/// Active-leaf compare-and-set port.
pub trait BranchSelectionPort {
    type Failure;

    fn compare_and_set_active_leaf(
        &mut self,
        expected_revision_id: &str,
        expected_active_leaf_id: &str,
        target_leaf_id: &str,
    ) -> Result<(), Self::Failure>;
}

/// Append-only continuation dispatch port.
pub trait BranchContinuationPort {
    type Failure;

    fn dispatch_append_only_fork(
        &mut self,
        plan: &BranchOperationPlan,
    ) -> Result<(), Self::Failure>;
}

/// Evidence persistence port.
pub trait BranchEvidencePort {
    type Failure;

    fn record_branch_receipt(
        &mut self,
        receipt: &BranchExplorerReceipt,
    ) -> Result<(), Self::Failure>;
}
