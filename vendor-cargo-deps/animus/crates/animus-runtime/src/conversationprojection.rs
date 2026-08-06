//! Pure canonical conversation-view projection and cache reconciliation.
//!
//! The admitted ledger remains authoritative. This module accepts only safe,
//! already-classified source facts. Hosts own storage, cache mutation, transport,
//! clocks, UI rendering, and ledger access.

// This archived Cairn adaptation keeps explicit projection DTOs and bounded reconciliation branches.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    compound_condition,
    function_length,
    nested_conditionals,
    numeric_units,
    path_segment_repetition,
    unbounded_collection_growth
)]

/// Projection schema.
pub const CONVERSATION_PROJECTION_SCHEMA: &str = "animus.conversation-projection.v1";
const POLICY_DOMAIN: &str = "animus.conversation-projection-policy.v1";
const STREAM_DOMAIN: &str = "animus.conversation-projection-stream.v1";
const ITEM_DOMAIN: &str = "animus.conversation-projection-item.v1";
const CURSOR_DOMAIN: &str = "animus.conversation-projection-cursor.v1";
const SNAPSHOT_DOMAIN: &str = "animus.conversation-projection-snapshot.v1";
const ACTION_DOMAIN: &str = "animus.conversation-projection-action.v1";
const RECEIPT_DOMAIN: &str = "animus.conversation-projection-receipt.v1";
const HASH_CHARS: usize = 64;
const MAX_LABEL_CHARS: usize = 128;
const SEPARATOR: &str = "\u{1f}";
const FIELD_POLICY: &str = "conversation_policy";
const FIELD_SOURCE: &str = "conversation_source";
const FIELD_CURSOR: &str = "conversation_cursor";
const FIELD_STREAM: &str = "conversation_stream";
const FIELD_RECEIPT: &str = "conversation_receipt";

/// Generated policy input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationProjectionPolicyInput {
    /// Schema.
    pub schema: String,
    /// Safe policy label.
    pub policy_id: String,
    /// Maximum source entries.
    pub max_entries: u32,
    /// Maximum visible parts in one folded item.
    pub max_parts_per_item: u32,
    /// Required reasoning posture.
    pub reasoning_posture: String,
    /// Required tool posture.
    pub tool_posture: String,
    /// Required cursor posture.
    pub cursor_posture: String,
    /// Required cache posture.
    pub cache_posture: String,
    /// Required receipt fields.
    pub receipt_fields: Vec<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
    /// Source policy hash.
    pub policy_hash: String,
}

/// Validated projection policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationProjectionPolicy {
    input: ConversationProjectionPolicyInput,
    identity: String,
}

impl ConversationProjectionPolicy {
    /// Returns the domain-separated identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

/// Safe source entry class.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationSourceKind {
    User,
    Assistant,
    Steering,
    Tool,
    System,
    Unsupported,
}
/// One safe source entry. Raw content is absent by construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationSourceEntry {
    /// Exact ledger sequence.
    pub sequence: u64,
    /// Canonical entry identity.
    pub entry_hash: String,
    /// Entry class.
    pub kind: ConversationSourceKind,
    /// Admitted redacted visible-part identity.
    pub visible_part_hash: String,
    /// Active-turn identity for assistant or steering facts.
    pub turn_hash: Option<String>,
}

/// One admitted source branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationSourceSnapshot {
    /// Session label.
    pub session_id: String,
    /// Branch label.
    pub branch_id: String,
    /// Selected leaf identity.
    pub leaf_hash: String,
    /// Ledger generation.
    pub generation: u64,
    /// Optional exact active-turn identity.
    pub active_turn_hash: Option<String>,
    /// Ordered entries.
    pub entries: Vec<ConversationSourceEntry>,
}

/// Projected view item class.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationViewItemKind {
    User,
    Assistant,
    Steering,
    Tool,
    System,
}
impl ConversationViewItemKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Steering => "steering",
            Self::Tool => "tool",
            Self::System => "system",
        }
    }
}

/// Redaction-safe view item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationViewItem {
    /// Stable item identity.
    pub item_hash: String,
    /// Item class.
    pub kind: ConversationViewItemKind,
    /// First source sequence.
    pub first_sequence: u64,
    /// Ordered canonical source identities.
    pub source_hashes: Vec<String>,
    /// Ordered admitted visible-part identities.
    pub visible_part_hashes: Vec<String>,
    /// Optional folded turn identity.
    pub turn_hash: Option<String>,
}

/// Exact projection cursor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationProjectionCursor {
    /// Stream identity.
    pub stream_hash: String,
    /// Policy identity.
    pub policy_hash: String,
    /// Session label.
    pub session_id: String,
    /// Branch label.
    pub branch_id: String,
    /// Selected leaf identity.
    pub leaf_hash: String,
    /// Source generation.
    pub generation: u64,
    /// Last source entry identity.
    pub last_entry_hash: String,
    /// Last projected sequence.
    pub last_sequence: u64,
    /// Optional active-turn identity.
    pub active_turn_hash: Option<String>,
    /// Cursor identity.
    pub cursor_hash: String,
}

/// Complete snapshot-first projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationProjectionSnapshot {
    /// Stream identity.
    pub stream_hash: String,
    /// Projection items.
    pub items: Vec<ConversationViewItem>,
    /// Exact cursor.
    pub cursor: ConversationProjectionCursor,
    /// Number of assistant source entries folded into existing items.
    pub fold_count: u32,
    /// Steering marker count.
    pub steering_count: u32,
    /// Snapshot identity.
    pub snapshot_hash: String,
}

/// Projection mutation class.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationProjectionMutation {
    Snapshot,
    Append,
    ReplaceFoldedAssistant,
    HealMetadata,
    Unsupported,
}
impl ConversationProjectionMutation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Snapshot => "snapshot",
            Self::Append => "append",
            Self::ReplaceFoldedAssistant => "replace_folded_assistant",
            Self::HealMetadata => "heal_metadata",
            Self::Unsupported => "unsupported",
        }
    }
}

/// One exact cache action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationProjectionAction {
    /// Stream identity.
    pub stream_hash: String,
    /// Source generation.
    pub generation: u64,
    /// Projection sequence.
    pub sequence: u64,
    /// Mutation class.
    pub mutation: ConversationProjectionMutation,
    /// Target item identity.
    pub item_hash: String,
    /// Action identity.
    pub action_hash: String,
}

/// Reconciliation plan class.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationProjectionPlanKind {
    Snapshot,
    Delta,
    Noop,
    Rebuild,
    Blocked,
}

/// Pure cache reconciliation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationProjectionPlan {
    /// Plan class.
    pub kind: ConversationProjectionPlanKind,
    /// Planned actions.
    pub actions: Vec<ConversationProjectionAction>,
    /// Resulting cursor.
    pub cursor: ConversationProjectionCursor,
    /// Plan identity.
    pub plan_hash: String,
}

/// Cache application status.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationCacheStatus {
    Applied,
    AlreadyApplied,
    Partial,
    Conflict,
    Unavailable,
}

/// Host cache evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationCacheEvidence {
    /// Plan identity.
    pub plan_hash: String,
    /// Applied action identities.
    pub action_hashes: Vec<String>,
    /// Resulting cursor identity.
    pub cursor_hash: String,
    /// Cache status.
    pub status: ConversationCacheStatus,
}

/// Implementation-free cache port.
pub trait ConversationProjectionCache {
    /// Applies an exact projection plan.
    fn apply(&mut self, plan: &ConversationProjectionPlan) -> ConversationCacheEvidence;
}

/// Legacy heal input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationLegacyHealDraft {
    /// Existing item identity.
    pub item_hash: String,
    /// Candidate exact source identities.
    pub candidate_source_hashes: Vec<String>,
    /// Whether visible content would change.
    pub changes_visible_content: bool,
    /// Whether an active turn can still mutate the item.
    pub active_turn_race: bool,
}

/// Sensitive receipt input that must be absent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConversationProjectionSensitiveDraft<'a> {
    /// Transcript text.
    pub transcript: Option<&'a str>,
    /// Reasoning.
    pub reasoning: Option<&'a str>,
    /// Tool body.
    pub tool_body: Option<&'a str>,
    /// Provider payload.
    pub provider_payload: Option<&'a str>,
    /// Credential.
    pub credential: Option<&'a str>,
    /// Environment value.
    pub environment: Option<&'a str>,
    /// Physical path.
    pub path: Option<&'a str>,
    /// Raw UI state.
    pub ui_state: Option<&'a str>,
    /// Unsupported truth or delivery claim.
    pub unsupported_claim: Option<&'a str>,
}

/// Metadata-only projection receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationProjectionReceipt {
    /// Receipt identity.
    pub receipt_hash: String,
    /// Plan class.
    pub plan_kind: ConversationProjectionPlanKind,
    /// Item count.
    pub item_count: u32,
    /// Stable non-claim.
    pub non_claim: &'static str,
}

/// Projection issue.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationProjectionIssue {
    InvalidPolicy,
    InvalidSource,
    StaleCursor,
    InvalidStream,
    CacheConflict,
    AmbiguousHeal,
    ReceiptLeak,
}

/// Safe projection error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversationProjectionError {
    issue: ConversationProjectionIssue,
    field: &'static str,
}
impl ConversationProjectionError {
    const fn new(issue: ConversationProjectionIssue, field: &'static str) -> Self {
        Self { issue, field }
    }
    /// Returns the issue class.
    #[must_use]
    pub const fn issue(&self) -> ConversationProjectionIssue {
        self.issue
    }
}
impl core::fmt::Display for ConversationProjectionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "conversation projection rejected {}", self.field)
    }
}
impl std::error::Error for ConversationProjectionError {}

/// Validates generated projection policy.
///
/// # Errors
/// Returns a safe error for unsupported or unbounded policy.
// r[impl agent_harness_runtime.conversation_view_projection.boundary]
// r[impl agent_harness_runtime.conversation_view_projection.identity]
pub fn validate_conversation_projection_policy(
    input: &ConversationProjectionPolicyInput,
) -> Result<ConversationProjectionPolicy, ConversationProjectionError> {
    const REQUIRED_RECEIPT: [&str; 5] = ["source", "stream", "cursor", "counts", "non_claims"];
    const REQUIRED_NON_CLAIMS: [&str; 6] = [
        "transcript_truth",
        "ui_delivery",
        "cache_durability",
        "renderer_correctness",
        "task_completion",
        "production_readiness",
    ];
    if input.schema != CONVERSATION_PROJECTION_SCHEMA
        || input.max_entries == 0
        || input.max_parts_per_item == 0
        || input.reasoning_posture != "hidden"
        || input.tool_posture != "hash_only"
        || input.cursor_posture != "revision_fenced"
        || input.cache_posture != "disposable"
        || !contains_all(&input.receipt_fields, &REQUIRED_RECEIPT)
        || !contains_all(&input.non_claims, &REQUIRED_NON_CLAIMS)
    {
        return Err(error(ConversationProjectionIssue::InvalidPolicy, FIELD_POLICY));
    }
    safe_label(&input.policy_id, FIELD_POLICY)?;
    hash_value(&input.policy_hash, FIELD_POLICY)?;
    let mut parts = vec![
        input.schema.clone(),
        input.policy_id.clone(),
        input.max_entries.to_string(),
        input.max_parts_per_item.to_string(),
        input.reasoning_posture.clone(),
        input.tool_posture.clone(),
        input.cursor_posture.clone(),
        input.cache_posture.clone(),
        input.policy_hash.clone(),
    ];
    parts.extend(input.receipt_fields.iter().cloned());
    parts.extend(input.non_claims.iter().cloned());
    Ok(ConversationProjectionPolicy {
        input: input.clone(),
        identity: hash_parts(POLICY_DOMAIN, &parts),
    })
}

/// Projects one complete snapshot before any incremental update.
///
/// # Errors
/// Returns a safe error for malformed, unordered, duplicate, unsupported, or overbound source
/// facts.
// r[impl agent_harness_runtime.conversation_view_projection.stream]
// r[impl agent_harness_runtime.conversation_view_projection.active_turn]
pub fn project_conversation_snapshot(
    policy: &ConversationProjectionPolicy,
    source: &ConversationSourceSnapshot,
) -> Result<ConversationProjectionSnapshot, ConversationProjectionError> {
    validate_source(policy, source)?;
    let stream_hash = hash_parts(STREAM_DOMAIN, &[
        policy.identity.clone(),
        source.session_id.clone(),
        source.branch_id.clone(),
        source.leaf_hash.clone(),
        source.generation.to_string(),
        source.active_turn_hash.clone().unwrap_or_default(),
    ]);
    let mut items = Vec::<ConversationViewItem>::new();
    let mut fold_count = 0_u32;
    let mut steering_count = 0_u32;
    for entry in &source.entries {
        if entry.kind == ConversationSourceKind::Assistant {
            if let Some(turn) = entry.turn_hash.as_deref() {
                if let Some(existing) = items.iter_mut().rev().find(|item| {
                    item.kind == ConversationViewItemKind::Assistant && item.turn_hash.as_deref() == Some(turn)
                }) {
                    let part_bound = usize::try_from(policy.input.max_parts_per_item)
                        .map_err(|_| error(ConversationProjectionIssue::InvalidPolicy, FIELD_POLICY))?;
                    if existing.visible_part_hashes.len() >= part_bound {
                        return Err(error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE));
                    }
                    existing.source_hashes.push(entry.entry_hash.clone());
                    existing.visible_part_hashes.push(entry.visible_part_hash.clone());
                    existing.item_hash = item_hash(
                        policy,
                        ConversationViewItemKind::Assistant,
                        existing.first_sequence,
                        &existing.source_hashes,
                        existing.turn_hash.as_deref(),
                    );
                    fold_count = fold_count
                        .checked_add(1)
                        .ok_or_else(|| error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE))?;
                    continue;
                }
            }
        }
        let kind = map_kind(entry.kind)?;
        if kind == ConversationViewItemKind::Steering {
            steering_count = steering_count
                .checked_add(1)
                .ok_or_else(|| error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE))?;
        }
        let source_hashes = vec![entry.entry_hash.clone()];
        items.push(ConversationViewItem {
            item_hash: item_hash(policy, kind, entry.sequence, &source_hashes, entry.turn_hash.as_deref()),
            kind,
            first_sequence: entry.sequence,
            source_hashes,
            visible_part_hashes: vec![entry.visible_part_hash.clone()],
            turn_hash: entry.turn_hash.clone(),
        });
    }
    let last = source
        .entries
        .last()
        .ok_or_else(|| error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE))?;
    let cursor = make_cursor(policy, source, stream_hash.clone(), last.entry_hash.clone(), last.sequence);
    let mut snapshot_parts = vec![
        stream_hash.clone(),
        cursor.cursor_hash.clone(),
        fold_count.to_string(),
        steering_count.to_string(),
    ];
    snapshot_parts.extend(items.iter().map(|item| item.item_hash.clone()));
    Ok(ConversationProjectionSnapshot {
        stream_hash,
        items,
        cursor,
        fold_count,
        steering_count,
        snapshot_hash: hash_parts(SNAPSHOT_DOMAIN, &snapshot_parts),
    })
}

/// Plans exact no-op, delta, or rebuild reconciliation.
///
/// # Errors
/// Returns a safe error only when the current source itself is invalid.
// r[impl agent_harness_runtime.conversation_view_projection.rebuild]
pub fn plan_conversation_top_up(
    policy: &ConversationProjectionPolicy,
    source: &ConversationSourceSnapshot,
    cursor: Option<&ConversationProjectionCursor>,
) -> Result<ConversationProjectionPlan, ConversationProjectionError> {
    let snapshot = project_conversation_snapshot(policy, source)?;
    let Some(cursor) = cursor else {
        return Ok(snapshot_plan(&snapshot, ConversationProjectionPlanKind::Snapshot));
    };
    if !cursor_matches_source(policy, source, cursor) {
        return Ok(snapshot_plan(&snapshot, ConversationProjectionPlanKind::Rebuild));
    }
    let anchor = source
        .entries
        .iter()
        .position(|entry| entry.entry_hash == cursor.last_entry_hash && entry.sequence == cursor.last_sequence);
    let Some(anchor) = anchor else {
        return Ok(snapshot_plan(&snapshot, ConversationProjectionPlanKind::Rebuild));
    };
    let remaining = source.entries.iter().skip(anchor.saturating_add(1)).collect::<Vec<_>>();
    if remaining.is_empty() {
        return Ok(ConversationProjectionPlan {
            kind: ConversationProjectionPlanKind::Noop,
            actions: Vec::new(),
            cursor: snapshot.cursor.clone(),
            plan_hash: hash_parts(ACTION_DOMAIN, &["noop".to_string(), cursor.cursor_hash.clone()]),
        });
    }
    let actions = remaining
        .iter()
        .map(|entry| {
            let item = snapshot
                .items
                .iter()
                .find(|item| item.source_hashes.contains(&entry.entry_hash))
                .ok_or_else(|| error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE))?;
            let mutation = if item.source_hashes.len() > 1 {
                ConversationProjectionMutation::ReplaceFoldedAssistant
            } else {
                ConversationProjectionMutation::Append
            };
            Ok(action(&snapshot.stream_hash, source.generation, entry.sequence, mutation, &item.item_hash))
        })
        .collect::<Result<Vec<_>, ConversationProjectionError>>()?;
    Ok(plan_from_actions(ConversationProjectionPlanKind::Delta, actions, snapshot.cursor))
}

/// Validates snapshot-first update order and lineage.
///
/// # Errors
/// Returns a safe error for incremental-before-snapshot, sequence gaps, stale revision,
/// cross-stream updates, or unsupported mutations.
#[allow(clippy::too_many_arguments)]
pub fn validate_conversation_update(
    snapshot_accepted: bool,
    expected_stream_hash: &str,
    expected_generation: u64,
    expected_sequence: u64,
    action: &ConversationProjectionAction,
) -> Result<(), ConversationProjectionError> {
    if !snapshot_accepted
        || action.stream_hash != expected_stream_hash
        || action.generation != expected_generation
        || action.sequence != expected_sequence
        || matches!(
            action.mutation,
            ConversationProjectionMutation::Snapshot | ConversationProjectionMutation::Unsupported
        )
    {
        return Err(error(ConversationProjectionIssue::InvalidStream, FIELD_STREAM));
    }
    hash_value(&action.item_hash, FIELD_STREAM)?;
    hash_value(&action.action_hash, FIELD_STREAM)
}

/// Plans conservative metadata-only healing from one exact join.
///
/// # Errors
/// Returns a safe error for ambiguous, visible-content-changing, or active-turn-racing healing.
pub fn plan_conversation_legacy_heal(
    stream_hash: &str,
    generation: u64,
    sequence: u64,
    draft: &ConversationLegacyHealDraft,
) -> Result<ConversationProjectionAction, ConversationProjectionError> {
    hash_value(stream_hash, FIELD_STREAM)?;
    hash_value(&draft.item_hash, FIELD_STREAM)?;
    if draft.candidate_source_hashes.len() != 1 || draft.changes_visible_content || draft.active_turn_race {
        return Err(error(ConversationProjectionIssue::AmbiguousHeal, FIELD_SOURCE));
    }
    hash_value(&draft.candidate_source_hashes[0], FIELD_SOURCE)?;
    Ok(action(
        stream_hash,
        generation,
        sequence,
        ConversationProjectionMutation::HealMetadata,
        &draft.item_hash,
    ))
}

/// Validates exact cache evidence.
///
/// # Errors
/// Returns a safe error for partial, conflicting, unavailable, or mismatched cache evidence.
pub fn validate_conversation_cache_evidence(
    plan: &ConversationProjectionPlan,
    evidence: &ConversationCacheEvidence,
) -> Result<(), ConversationProjectionError> {
    let expected = plan.actions.iter().map(|action| action.action_hash.clone()).collect::<Vec<_>>();
    if evidence.plan_hash != plan.plan_hash
        || evidence.cursor_hash != plan.cursor.cursor_hash
        || !matches!(evidence.status, ConversationCacheStatus::Applied | ConversationCacheStatus::AlreadyApplied)
        || (!evidence.action_hashes.is_empty() && evidence.action_hashes != expected)
    {
        return Err(error(ConversationProjectionIssue::CacheConflict, FIELD_CURSOR));
    }
    Ok(())
}

/// Projects metadata-only evidence.
///
/// # Errors
/// Returns a safe error when private material or unsupported claims are supplied.
// r[impl agent_harness_runtime.conversation_view_projection.evidence]
#[allow(clippy::too_many_arguments)]
pub fn project_conversation_projection_receipt(
    policy: &ConversationProjectionPolicy,
    snapshot: &ConversationProjectionSnapshot,
    plan: &ConversationProjectionPlan,
    evidence: &ConversationCacheEvidence,
    sensitive: ConversationProjectionSensitiveDraft<'_>,
) -> Result<ConversationProjectionReceipt, ConversationProjectionError> {
    if [
        sensitive.transcript,
        sensitive.reasoning,
        sensitive.tool_body,
        sensitive.provider_payload,
        sensitive.credential,
        sensitive.environment,
        sensitive.path,
        sensitive.ui_state,
        sensitive.unsupported_claim,
    ]
    .into_iter()
    .any(|value| value.is_some())
    {
        return Err(error(ConversationProjectionIssue::ReceiptLeak, FIELD_RECEIPT));
    }
    validate_conversation_cache_evidence(plan, evidence)?;
    let item_count = u32::try_from(snapshot.items.len())
        .map_err(|_| error(ConversationProjectionIssue::InvalidSource, FIELD_RECEIPT))?;
    let non_claim = "projection does not prove transcript truth, UI delivery, cache durability, renderer correctness, task completion, or production readiness";
    let receipt_hash = hash_parts(RECEIPT_DOMAIN, &[
        policy.identity.clone(),
        snapshot.snapshot_hash.clone(),
        plan.plan_hash.clone(),
        evidence.cursor_hash.clone(),
        item_count.to_string(),
        format!("{:?}", plan.kind),
        non_claim.to_string(),
    ]);
    Ok(ConversationProjectionReceipt {
        receipt_hash,
        plan_kind: plan.kind,
        item_count,
        non_claim,
    })
}

fn validate_source(
    policy: &ConversationProjectionPolicy,
    source: &ConversationSourceSnapshot,
) -> Result<(), ConversationProjectionError> {
    safe_label(&source.session_id, FIELD_SOURCE)?;
    safe_label(&source.branch_id, FIELD_SOURCE)?;
    hash_value(&source.leaf_hash, FIELD_SOURCE)?;
    if let Some(turn) = &source.active_turn_hash {
        hash_value(turn, FIELD_SOURCE)?;
    }
    let bound = usize::try_from(policy.input.max_entries)
        .map_err(|_| error(ConversationProjectionIssue::InvalidPolicy, FIELD_POLICY))?;
    if source.entries.is_empty() || source.entries.len() > bound {
        return Err(error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE));
    }
    let mut hashes = std::collections::BTreeSet::new();
    let mut prior = None;
    for entry in &source.entries {
        hash_value(&entry.entry_hash, FIELD_SOURCE)?;
        hash_value(&entry.visible_part_hash, FIELD_SOURCE)?;
        if !hashes.insert(entry.entry_hash.as_str())
            || prior.is_some_and(|value| entry.sequence <= value)
            || entry.kind == ConversationSourceKind::Unsupported
        {
            return Err(error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE));
        }
        if matches!(entry.kind, ConversationSourceKind::Assistant | ConversationSourceKind::Steering)
            && entry.turn_hash.is_none()
        {
            return Err(error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE));
        }
        if let Some(turn) = &entry.turn_hash {
            hash_value(turn, FIELD_SOURCE)?;
        }
        prior = Some(entry.sequence);
    }
    Ok(())
}

fn cursor_matches_source(
    policy: &ConversationProjectionPolicy,
    source: &ConversationSourceSnapshot,
    cursor: &ConversationProjectionCursor,
) -> bool {
    cursor.policy_hash == policy.identity
        && cursor.session_id == source.session_id
        && cursor.branch_id == source.branch_id
        && cursor.leaf_hash == source.leaf_hash
        && cursor.generation == source.generation
        && cursor.active_turn_hash == source.active_turn_hash
        && cursor.cursor_hash
            == hash_parts(CURSOR_DOMAIN, &[
                cursor.stream_hash.clone(),
                cursor.policy_hash.clone(),
                cursor.session_id.clone(),
                cursor.branch_id.clone(),
                cursor.leaf_hash.clone(),
                cursor.generation.to_string(),
                cursor.last_entry_hash.clone(),
                cursor.last_sequence.to_string(),
                cursor.active_turn_hash.clone().unwrap_or_default(),
            ])
}
#[allow(clippy::too_many_arguments)]
fn make_cursor(
    policy: &ConversationProjectionPolicy,
    source: &ConversationSourceSnapshot,
    stream_hash: String,
    last_entry_hash: String,
    last_sequence: u64,
) -> ConversationProjectionCursor {
    let parts = vec![
        stream_hash.clone(),
        policy.identity.clone(),
        source.session_id.clone(),
        source.branch_id.clone(),
        source.leaf_hash.clone(),
        source.generation.to_string(),
        last_entry_hash.clone(),
        last_sequence.to_string(),
        source.active_turn_hash.clone().unwrap_or_default(),
    ];
    ConversationProjectionCursor {
        stream_hash,
        policy_hash: policy.identity.clone(),
        session_id: source.session_id.clone(),
        branch_id: source.branch_id.clone(),
        leaf_hash: source.leaf_hash.clone(),
        generation: source.generation,
        last_entry_hash,
        last_sequence,
        active_turn_hash: source.active_turn_hash.clone(),
        cursor_hash: hash_parts(CURSOR_DOMAIN, &parts),
    }
}
#[allow(clippy::too_many_arguments)]
fn item_hash(
    policy: &ConversationProjectionPolicy,
    kind: ConversationViewItemKind,
    first_sequence: u64,
    sources: &[String],
    turn: Option<&str>,
) -> String {
    let mut parts = vec![
        policy.identity.clone(),
        kind.as_str().to_string(),
        first_sequence.to_string(),
        turn.unwrap_or_default().to_string(),
    ];
    parts.extend(sources.iter().cloned());
    hash_parts(ITEM_DOMAIN, &parts)
}
#[allow(clippy::too_many_arguments)]
fn action(
    stream: &str,
    generation: u64,
    sequence: u64,
    mutation: ConversationProjectionMutation,
    item_hash: &str,
) -> ConversationProjectionAction {
    let parts = vec![
        stream.to_string(),
        generation.to_string(),
        sequence.to_string(),
        mutation.as_str().to_string(),
        item_hash.to_string(),
    ];
    ConversationProjectionAction {
        stream_hash: stream.to_string(),
        generation,
        sequence,
        mutation,
        item_hash: item_hash.to_string(),
        action_hash: hash_parts(ACTION_DOMAIN, &parts),
    }
}
fn snapshot_plan(
    snapshot: &ConversationProjectionSnapshot,
    kind: ConversationProjectionPlanKind,
) -> ConversationProjectionPlan {
    let action = action(
        &snapshot.stream_hash,
        snapshot.cursor.generation,
        0,
        ConversationProjectionMutation::Snapshot,
        &snapshot.snapshot_hash,
    );
    plan_from_actions(kind, vec![action], snapshot.cursor.clone())
}
fn plan_from_actions(
    kind: ConversationProjectionPlanKind,
    actions: Vec<ConversationProjectionAction>,
    cursor: ConversationProjectionCursor,
) -> ConversationProjectionPlan {
    let mut parts = vec![format!("{kind:?}"), cursor.cursor_hash.clone()];
    parts.extend(actions.iter().map(|action| action.action_hash.clone()));
    ConversationProjectionPlan {
        kind,
        actions,
        cursor,
        plan_hash: hash_parts(ACTION_DOMAIN, &parts),
    }
}
fn map_kind(kind: ConversationSourceKind) -> Result<ConversationViewItemKind, ConversationProjectionError> {
    match kind {
        ConversationSourceKind::User => Ok(ConversationViewItemKind::User),
        ConversationSourceKind::Assistant => Ok(ConversationViewItemKind::Assistant),
        ConversationSourceKind::Steering => Ok(ConversationViewItemKind::Steering),
        ConversationSourceKind::Tool => Ok(ConversationViewItemKind::Tool),
        ConversationSourceKind::System => Ok(ConversationViewItemKind::System),
        ConversationSourceKind::Unsupported => Err(error(ConversationProjectionIssue::InvalidSource, FIELD_SOURCE)),
    }
}
fn contains_all(values: &[String], required: &[&str]) -> bool {
    required.iter().all(|required| values.iter().any(|value| value == required))
}
fn hash_parts(domain: &str, parts: &[String]) -> String {
    let mut material = domain.to_string();
    for part in parts {
        material.push_str(SEPARATOR);
        material.push_str(part);
    }
    blake3::hash(material.as_bytes()).to_hex().to_string()
}
fn hash_value(value: &str, field: &'static str) -> Result<(), ConversationProjectionError> {
    if value.len() != HASH_CHARS || !value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')) {
        return Err(error(ConversationProjectionIssue::InvalidSource, field));
    }
    Ok(())
}
fn safe_label(value: &str, field: &'static str) -> Result<(), ConversationProjectionError> {
    if value.is_empty()
        || value.len() > MAX_LABEL_CHARS
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(error(ConversationProjectionIssue::InvalidSource, field));
    }
    Ok(())
}
const fn error(issue: ConversationProjectionIssue, field: &'static str) -> ConversationProjectionError {
    ConversationProjectionError::new(issue, field)
}

#[cfg(test)]
mod tests {
    use super::*;
    const GENERATION: u64 = 3;
    const FIRST: u64 = 1;
    const SECOND: u64 = 2;
    const THIRD: u64 = 3;
    const FOURTH: u64 = 4;
    #[test]
    fn snapshot_folds_assistant_turn_and_preserves_steering() {
        let projection = project_conversation_snapshot(&policy(), &source()).expect("snapshot");
        assert_eq!(projection.items.len(), 3);
        assert_eq!(projection.fold_count, 1);
        assert_eq!(projection.steering_count, 1);
        assert_eq!(projection.items[1].kind, ConversationViewItemKind::Assistant);
        assert_eq!(projection.items[1].source_hashes.len(), 2);
    }
    #[test]
    fn equal_visible_content_has_distinct_source_identity() {
        let mut source = source();
        source.entries[0].visible_part_hash = source.entries[1].visible_part_hash.clone();
        let projection = project_conversation_snapshot(&policy(), &source).expect("projection");
        assert_ne!(projection.items[0].item_hash, projection.items[1].item_hash);
    }
    #[test]
    fn absent_cursor_starts_snapshot_and_exact_cursor_is_noop() {
        let policy = policy();
        let source = source();
        let snapshot = project_conversation_snapshot(&policy, &source).expect("snapshot");
        assert_eq!(
            plan_conversation_top_up(&policy, &source, None).expect("plan").kind,
            ConversationProjectionPlanKind::Snapshot
        );
        assert_eq!(
            plan_conversation_top_up(&policy, &source, Some(&snapshot.cursor)).expect("noop").kind,
            ConversationProjectionPlanKind::Noop
        );
    }
    #[test]
    fn stale_generation_leaf_and_policy_require_rebuild() {
        let policy = policy();
        let source = source();
        let mut cursor = project_conversation_snapshot(&policy, &source).expect("snapshot").cursor;
        cursor.generation += 1;
        assert_eq!(
            plan_conversation_top_up(&policy, &source, Some(&cursor)).expect("rebuild").kind,
            ConversationProjectionPlanKind::Rebuild
        );
    }
    #[test]
    fn snapshot_first_and_exact_sequence_are_required() {
        let snapshot = project_conversation_snapshot(&policy(), &source()).expect("snapshot");
        let action = action(
            &snapshot.stream_hash,
            GENERATION,
            FOURTH,
            ConversationProjectionMutation::Append,
            &snapshot.items[0].item_hash,
        );
        assert_eq!(
            validate_conversation_update(false, &snapshot.stream_hash, GENERATION, FOURTH, &action)
                .expect_err("before snapshot")
                .issue(),
            ConversationProjectionIssue::InvalidStream
        );
        assert!(validate_conversation_update(true, &snapshot.stream_hash, GENERATION, FOURTH, &action).is_ok());
    }
    #[test]
    fn corrupt_duplicate_reversed_and_unsupported_sources_fail() {
        let mut duplicate = source();
        duplicate.entries[1].entry_hash = duplicate.entries[0].entry_hash.clone();
        let mut reversed = source();
        reversed.entries[1].sequence = FIRST;
        let mut unsupported = source();
        unsupported.entries[1].kind = ConversationSourceKind::Unsupported;
        for candidate in [duplicate, reversed, unsupported] {
            assert_eq!(
                project_conversation_snapshot(&policy(), &candidate).expect_err("invalid").issue(),
                ConversationProjectionIssue::InvalidSource
            );
        }
    }
    #[test]
    fn exact_legacy_heal_is_metadata_only() {
        let snapshot = project_conversation_snapshot(&policy(), &source()).expect("snapshot");
        let draft = ConversationLegacyHealDraft {
            item_hash: snapshot.items[0].item_hash.clone(),
            candidate_source_hashes: vec![snapshot.items[0].source_hashes[0].clone()],
            changes_visible_content: false,
            active_turn_race: false,
        };
        assert_eq!(
            plan_conversation_legacy_heal(&snapshot.stream_hash, GENERATION, FOURTH, &draft)
                .expect("heal")
                .mutation,
            ConversationProjectionMutation::HealMetadata
        );
    }
    #[test]
    fn ambiguous_or_visible_legacy_heal_fails() {
        let snapshot = project_conversation_snapshot(&policy(), &source()).expect("snapshot");
        let draft = ConversationLegacyHealDraft {
            item_hash: snapshot.items[0].item_hash.clone(),
            candidate_source_hashes: vec![hash("one"), hash("two")],
            changes_visible_content: true,
            active_turn_race: false,
        };
        assert_eq!(
            plan_conversation_legacy_heal(&snapshot.stream_hash, GENERATION, FOURTH, &draft)
                .expect_err("ambiguous")
                .issue(),
            ConversationProjectionIssue::AmbiguousHeal
        );
    }
    #[test]
    fn cache_evidence_is_exact_and_idempotent() {
        let plan = plan_conversation_top_up(&policy(), &source(), None).expect("plan");
        let hashes = plan.actions.iter().map(|action| action.action_hash.clone()).collect();
        let evidence = ConversationCacheEvidence {
            plan_hash: plan.plan_hash.clone(),
            action_hashes: hashes,
            cursor_hash: plan.cursor.cursor_hash.clone(),
            status: ConversationCacheStatus::AlreadyApplied,
        };
        assert!(validate_conversation_cache_evidence(&plan, &evidence).is_ok());
    }
    #[test]
    fn partial_and_conflicting_cache_evidence_fails() {
        let plan = plan_conversation_top_up(&policy(), &source(), None).expect("plan");
        for status in [
            ConversationCacheStatus::Partial,
            ConversationCacheStatus::Conflict,
            ConversationCacheStatus::Unavailable,
        ] {
            let evidence = ConversationCacheEvidence {
                plan_hash: plan.plan_hash.clone(),
                action_hashes: Vec::new(),
                cursor_hash: plan.cursor.cursor_hash.clone(),
                status,
            };
            assert_eq!(
                validate_conversation_cache_evidence(&plan, &evidence).expect_err("blocked").issue(),
                ConversationProjectionIssue::CacheConflict
            );
        }
    }
    #[test]
    fn receipt_is_stable_and_metadata_only() {
        let policy = policy();
        let snapshot = project_conversation_snapshot(&policy, &source()).expect("snapshot");
        let plan = plan_conversation_top_up(&policy, &source(), None).expect("plan");
        let evidence = passing_evidence(&plan);
        let first = project_conversation_projection_receipt(
            &policy,
            &snapshot,
            &plan,
            &evidence,
            ConversationProjectionSensitiveDraft::default(),
        )
        .expect("receipt");
        let second = project_conversation_projection_receipt(
            &policy,
            &snapshot,
            &plan,
            &evidence,
            ConversationProjectionSensitiveDraft::default(),
        )
        .expect("receipt");
        assert_eq!(first, second);
        assert!(first.non_claim.contains("does not prove"));
    }
    #[test]
    fn receipt_rejects_private_material_and_overclaims() {
        let policy = policy();
        let snapshot = project_conversation_snapshot(&policy, &source()).expect("snapshot");
        let plan = plan_conversation_top_up(&policy, &source(), None).expect("plan");
        let evidence = passing_evidence(&plan);
        let sensitive = ConversationProjectionSensitiveDraft {
            transcript: Some("private"),
            ..ConversationProjectionSensitiveDraft::default()
        };
        assert_eq!(
            project_conversation_projection_receipt(&policy, &snapshot, &plan, &evidence, sensitive)
                .expect_err("leak")
                .issue(),
            ConversationProjectionIssue::ReceiptLeak
        );
    }
    fn passing_evidence(plan: &ConversationProjectionPlan) -> ConversationCacheEvidence {
        ConversationCacheEvidence {
            plan_hash: plan.plan_hash.clone(),
            action_hashes: plan.actions.iter().map(|action| action.action_hash.clone()).collect(),
            cursor_hash: plan.cursor.cursor_hash.clone(),
            status: ConversationCacheStatus::Applied,
        }
    }
    fn policy() -> ConversationProjectionPolicy {
        validate_conversation_projection_policy(&ConversationProjectionPolicyInput {
            schema: CONVERSATION_PROJECTION_SCHEMA.to_string(),
            policy_id: "canonical-conversation".to_string(),
            max_entries: 64,
            max_parts_per_item: 8,
            reasoning_posture: "hidden".to_string(),
            tool_posture: "hash_only".to_string(),
            cursor_posture: "revision_fenced".to_string(),
            cache_posture: "disposable".to_string(),
            receipt_fields: ["source", "stream", "cursor", "counts", "non_claims"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            non_claims: [
                "transcript_truth",
                "ui_delivery",
                "cache_durability",
                "renderer_correctness",
                "task_completion",
                "production_readiness",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            policy_hash: hash("policy"),
        })
        .expect("policy")
    }
    fn source() -> ConversationSourceSnapshot {
        let turn = hash("turn");
        ConversationSourceSnapshot {
            session_id: "session.one".to_string(),
            branch_id: "branch.one".to_string(),
            leaf_hash: hash("leaf"),
            generation: GENERATION,
            active_turn_hash: Some(turn.clone()),
            entries: vec![
                entry(FIRST, "user", ConversationSourceKind::User, None),
                entry(SECOND, "assistant-one", ConversationSourceKind::Assistant, Some(turn.clone())),
                entry(THIRD, "steering", ConversationSourceKind::Steering, Some(turn.clone())),
                entry(FOURTH, "assistant-two", ConversationSourceKind::Assistant, Some(turn)),
            ],
        }
    }
    fn entry(
        sequence: u64,
        label: &str,
        kind: ConversationSourceKind,
        turn_hash: Option<String>,
    ) -> ConversationSourceEntry {
        ConversationSourceEntry {
            sequence,
            entry_hash: hash(label),
            kind,
            visible_part_hash: hash(&format!("visible:{label}")),
            turn_hash,
        }
    }
    fn hash(value: &str) -> String {
        blake3::hash(value.as_bytes()).to_hex().to_string()
    }
}
