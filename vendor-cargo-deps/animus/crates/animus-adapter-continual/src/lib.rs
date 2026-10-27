//! Optional `no_std` continual-harness refinement contracts.
//!
//! This crate owns only pure proposal validation, bounded admission, snapshot
//! planning, rollback planning, explicit promotion planning, and metadata-only
//! receipt projection. Hosts own trajectory review, snapshot storage, prompt
//! composition, durable state, and all filesystem, VCS, process, network,
//! credential, clock, and persistence authority.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use core::cmp::Ordering;

/// The reviewed continual-harness contract schema.
pub const SCHEMA: &str = "animus.continual-harness.v1";
/// The reviewed Prime Agent continual-harness cohort.
pub const REVIEWED_COHORT: &str = "prime-agent-continual-harness";
/// The reviewed cohort license.
pub const REVIEWED_LICENSE: &str = "MIT";
/// The BLAKE3 policy identity domain.
pub const POLICY_DOMAIN: &str = "animus.continual-harness-policy.v1";
/// The BLAKE3 supplemental-state identity domain.
pub const STATE_DOMAIN: &str = "animus.continual-harness-state.v1";
/// The BLAKE3 snapshot identity domain.
pub const SNAPSHOT_DOMAIN: &str = "animus.continual-harness-snapshot.v1";
/// The BLAKE3 receipt identity domain.
pub const RECEIPT_DOMAIN: &str = "animus.continual-harness-receipt.v1";
/// The BLAKE3 hexadecimal identity length.
pub const BLAKE3_HEX_CHARS: usize = 64;
/// The largest accepted key length.
pub const MAX_KEY_BYTES: usize = 256;
/// The largest accepted evidence identifier length.
pub const MAX_EVIDENCE_ID_BYTES: usize = 256;
/// The largest accepted evidence hash length.
pub const MAX_EVIDENCE_HASH_BYTES: usize = BLAKE3_HEX_CHARS;
/// The smallest number of required policy non-claims.
pub const MIN_NON_CLAIMS: usize = 8;

const HASH_SEPARATOR: u8 = 0;
const MIN_CONTENT_BYTES: usize = 1;

/// One of the four supplemental refinement target classes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TargetClass {
    /// A supplemental prompt fragment, never the immutable base prompt.
    SupplementalPrompt,
    /// A bounded session-local memory entry.
    Memory,
    /// A supplemental skill description, never a repo-owned skill package.
    SkillDescription,
    /// A reusable subagent specification, never an accepted spec.
    SubagentSpecification,
    /// The immutable base system prompt, always rejected.
    BaseSystemPrompt,
    /// A repo-owned skill package, always rejected.
    RepoOwnedSkill,
    /// An accepted specification, always rejected.
    AcceptedSpec,
    /// A policy file, always rejected.
    PolicyFile,
}

impl TargetClass {
    /// Returns the stable metadata label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SupplementalPrompt => "supplemental_prompt",
            Self::Memory => "memory",
            Self::SkillDescription => "skill_description",
            Self::SubagentSpecification => "subagent_specification",
            Self::BaseSystemPrompt => "base_system_prompt",
            Self::RepoOwnedSkill => "repo_owned_skill",
            Self::AcceptedSpec => "accepted_spec",
            Self::PolicyFile => "policy_file",
        }
    }

    /// Returns whether the class is part of the admitted supplemental set.
    #[must_use]
    pub const fn is_admitted(self) -> bool {
        matches!(
            self,
            Self::SupplementalPrompt
                | Self::Memory
                | Self::SkillDescription
                | Self::SubagentSpecification
        )
    }
}

/// One permitted refinement operation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Operation {
    /// Create a new keyed entry.
    Add,
    /// Replace an existing keyed entry.
    Update,
    /// Remove an existing keyed entry.
    Retract,
}

impl Operation {
    /// Returns the stable metadata label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Update => "update",
            Self::Retract => "retract",
        }
    }
}

/// A typed reference into the host-owned session ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRef {
    /// Stable ledger entry identifier, not the entry body.
    pub ledger_entry_id: String,
    /// BLAKE3 identity of the referenced evidence.
    pub evidence_hash: String,
}

impl EvidenceRef {
    /// Creates a bounded evidence reference.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MalformedEvidence`] for empty or oversized fields.
    pub fn new(ledger_entry_id: &str, evidence_hash: &str) -> Result<Self, Error> {
        validate_bounded_text(ledger_entry_id, MAX_EVIDENCE_ID_BYTES)?;
        validate_hash(evidence_hash, MAX_EVIDENCE_HASH_BYTES)?;
        Ok(Self {
            ledger_entry_id: ledger_entry_id.into(),
            evidence_hash: evidence_hash.into(),
        })
    }
}

/// A typed refinement proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    /// Exactly one admitted or rejected target class.
    pub target_class: TargetClass,
    /// Exactly one add, update, or retract operation.
    pub operation: Operation,
    /// Stable supplemental-state key.
    pub key: String,
    /// New content for add and update; absent for retract.
    pub content: Option<String>,
    /// At least one typed session-ledger evidence reference is required.
    pub evidence: Vec<EvidenceRef>,
    /// Proposal-level promotion is never permitted.
    pub requests_promotion: bool,
}

/// Borrowed input used to construct a refinement proposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProposalDraft<'a> {
    /// Exactly one admitted or rejected target class.
    pub target_class: TargetClass,
    /// Exactly one add, update, or retract operation.
    pub operation: Operation,
    /// Stable supplemental-state key.
    pub key: &'a str,
    /// New content for add and update; absent for retract.
    pub content: Option<&'a str>,
    /// Typed session-ledger evidence references.
    pub evidence: &'a [EvidenceRef],
}

impl Proposal {
    /// Creates a proposal from a borrowed draft.
    #[must_use]
    pub fn new(draft: ProposalDraft<'_>) -> Self {
        Self {
            target_class: draft.target_class,
            operation: draft.operation,
            key: draft.key.into(),
            content: draft.content.map(String::from),
            evidence: draft.evidence.to_vec(),
            requests_promotion: false,
        }
    }
}

/// One admitted supplemental-state entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateEntry {
    /// The admitted target class.
    pub target_class: TargetClass,
    /// Stable entry key.
    pub key: String,
    /// Supplemental content owned by the session host.
    pub content: String,
}

/// Canonically ordered supplemental session state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplementalState {
    /// Entries sorted by target class and key.
    pub entries: Vec<StateEntry>,
}

impl SupplementalState {
    /// Creates and validates canonical supplemental state.
    ///
    /// # Errors
    ///
    /// Returns an error for unsupported targets, malformed keys, empty content,
    /// duplicate keys, or non-canonical ordering.
    pub fn new(mut entries: Vec<StateEntry>) -> Result<Self, Error> {
        for entry in &entries {
            validate_state_entry(entry)?;
        }
        entries.sort_by(compare_entries);
        for pair in entries.windows(2) {
            if same_entry_key(&pair[0], &pair[1]) {
                return Err(Error::Conflict {
                    target_class: pair[1].target_class,
                    key: pair[1].key.clone(),
                });
            }
        }
        Ok(Self { entries })
    }

    /// Returns the deterministic state identity.
    #[must_use]
    pub fn identity(&self) -> String {
        digest(STATE_DOMAIN, &canonical_state(self))
    }

    /// Returns the canonical byte count used by policy bounds.
    #[must_use]
    pub fn content_bytes(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| entry.key.len() + entry.content.len())
            .sum()
    }
}

/// Nickel-exported policy input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDraft {
    /// Reviewed source cohort label.
    pub cohort: String,
    /// Exact reviewed source revision.
    pub source_revision: String,
    /// Maximum content bytes for one proposal entry.
    pub max_content_bytes: usize,
    /// Maximum retained state entries.
    pub max_state_entries: usize,
    /// Maximum retained state content bytes.
    pub max_state_bytes: usize,
    /// Maximum retained snapshots.
    pub max_snapshots: usize,
    /// Maximum snapshot state bytes.
    pub max_snapshot_bytes: usize,
    /// Required metadata-only receipt fields.
    pub receipt_fields: Vec<String>,
    /// Required receipt non-claims.
    pub non_claims: Vec<String>,
}

/// Validated continual-harness policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    /// Reviewed source cohort label.
    pub cohort: String,
    /// Exact reviewed source revision.
    pub source_revision: String,
    /// Maximum content bytes for one proposal entry.
    pub max_content_bytes: usize,
    /// Maximum retained state entries.
    pub max_state_entries: usize,
    /// Maximum retained state content bytes.
    pub max_state_bytes: usize,
    /// Maximum retained snapshots.
    pub max_snapshots: usize,
    /// Maximum snapshot state bytes.
    pub max_snapshot_bytes: usize,
    /// Required metadata-only receipt fields.
    pub receipt_fields: Vec<String>,
    /// Required receipt non-claims.
    pub non_claims: Vec<String>,
    /// Domain-separated policy identity.
    pub identity: String,
}

impl Policy {
    /// Validates a Nickel-exported policy and derives its identity.
    ///
    /// # Errors
    ///
    /// Returns a bounded typed policy error when a source, bound, field, or
    /// non-claim is missing or unsafe.
    pub fn new(draft: PolicyDraft) -> Result<Self, Error> {
        validate_bounded_text(&draft.cohort, MAX_KEY_BYTES)?;
        validate_bounded_text(&draft.source_revision, MAX_KEY_BYTES)?;
        if draft.cohort != REVIEWED_COHORT || draft.source_revision.is_empty() {
            return Err(Error::PolicyRejected);
        }
        if draft.max_content_bytes < MIN_CONTENT_BYTES
            || draft.max_state_entries < MIN_CONTENT_BYTES
            || draft.max_state_bytes < MIN_CONTENT_BYTES
            || draft.max_snapshots < MIN_CONTENT_BYTES
            || draft.max_snapshot_bytes < MIN_CONTENT_BYTES
        {
            return Err(Error::PolicyRejected);
        }
        if draft.receipt_fields.is_empty() || draft.non_claims.len() < MIN_NON_CLAIMS {
            return Err(Error::PolicyRejected);
        }
        for value in draft.receipt_fields.iter().chain(draft.non_claims.iter()) {
            validate_bounded_text(value, MAX_KEY_BYTES)?;
        }
        let identity_material = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}",
            draft.cohort,
            draft.source_revision,
            draft.max_content_bytes,
            draft.max_state_entries,
            draft.max_state_bytes,
            draft.max_snapshots,
            draft.max_snapshot_bytes,
            join(&draft.receipt_fields),
            join(&draft.non_claims),
        );
        Ok(Self {
            cohort: draft.cohort,
            source_revision: draft.source_revision,
            max_content_bytes: draft.max_content_bytes,
            max_state_entries: draft.max_state_entries,
            max_state_bytes: draft.max_state_bytes,
            max_snapshots: draft.max_snapshots,
            max_snapshot_bytes: draft.max_snapshot_bytes,
            receipt_fields: draft.receipt_fields,
            non_claims: draft.non_claims,
            identity: digest(POLICY_DOMAIN, &identity_material),
        })
    }
}

/// A pre-update snapshot owned by the host after the pure plan is applied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Snapshot {
    /// Exact pre-update supplemental state.
    pub state: SupplementalState,
    /// Domain-separated identity over the pre-update state.
    pub identity: String,
    /// State byte count used for retention bounds.
    pub state_bytes: usize,
}

impl Snapshot {
    /// Creates a deterministic snapshot from state.
    #[must_use]
    pub fn from_state(state: &SupplementalState) -> Self {
        let state_bytes = state.content_bytes();
        let identity = digest(SNAPSHOT_DOMAIN, &canonical_state(state));
        Self {
            state: state.clone(),
            identity,
            state_bytes,
        }
    }
}

/// A pure admission result. The host must record `snapshot` before applying
/// `next_state`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionPlan {
    /// The pre-update snapshot that must be recorded first.
    pub snapshot: Snapshot,
    /// The state after the admitted operation.
    pub next_state: SupplementalState,
    /// Metadata-only update receipt.
    pub receipt: Receipt,
}

/// Session-local state and retained snapshots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
    /// Current session-local supplemental state.
    pub state: SupplementalState,
    /// Retained admitted snapshots, oldest first.
    pub snapshots: Vec<Snapshot>,
}

impl Session {
    /// Creates a session with no retained snapshots.
    #[must_use]
    pub fn new(state: SupplementalState) -> Self {
        Self {
            state,
            snapshots: Vec::new(),
        }
    }

    /// Applies an admission plan without performing host I/O.
    ///
    /// # Errors
    ///
    /// Rejects stale plans, oversized snapshots, or invalid retention bounds.
    pub fn apply(&self, plan: &AdmissionPlan, policy: &Policy) -> Result<Self, Error> {
        validate_policy_identity(policy)?;
        if plan.snapshot.state != self.state {
            return Err(Error::StalePlan);
        }
        if plan.snapshot.state_bytes > policy.max_snapshot_bytes {
            return Err(Error::SnapshotTooLarge);
        }
        if plan.next_state.entries.len() > policy.max_state_entries
            || plan.next_state.content_bytes() > policy.max_state_bytes
        {
            return Err(Error::StateLimitExceeded);
        }
        let mut snapshots = self.snapshots.clone();
        snapshots.push(plan.snapshot.clone());
        while snapshots.len() > policy.max_snapshots {
            snapshots.remove(0);
        }
        Ok(Self {
            state: plan.next_state.clone(),
            snapshots,
        })
    }

    /// Plans exact rollback to one retained snapshot.
    ///
    /// # Errors
    ///
    /// Rejects unknown, evicted, forged, or oversized snapshot identities.
    pub fn rollback(
        &self,
        snapshot_identity: &str,
        policy: &Policy,
    ) -> Result<RollbackPlan, Error> {
        validate_policy_identity(policy)?;
        validate_hash(snapshot_identity, BLAKE3_HEX_CHARS)?;
        let snapshot = self
            .snapshots
            .iter()
            .find(|candidate| candidate.identity == snapshot_identity)
            .ok_or(Error::UnknownSnapshot)?;
        if Snapshot::from_state(&snapshot.state) != *snapshot {
            return Err(Error::ForgedSnapshot);
        }
        let receipt = Receipt::new(
            policy,
            ReceiptInput {
                target_class: TargetClass::Memory,
                operation: Operation::Update,
                status: ReceiptStatus::RolledBack,
                snapshot_identity: snapshot.identity.as_str(),
                state_identity: snapshot.state.identity().as_str(),
                item_count: snapshot.state.entries.len(),
                content_bytes: snapshot.state.content_bytes(),
            },
        )?;
        Ok(RollbackPlan {
            state: snapshot.state.clone(),
            snapshot_identity: snapshot.identity.clone(),
            receipt,
        })
    }
}

/// A pure rollback result for the host to apply to session-local state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackPlan {
    /// Exact retained state to restore.
    pub state: SupplementalState,
    /// Retained snapshot identity selected by the host.
    pub snapshot_identity: String,
    /// Metadata-only rollback receipt.
    pub receipt: Receipt,
}

/// A durable destination that can be selected only by an explicit host action.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DurableScope {
    /// Project-owned durable state.
    Project,
    /// Personal durable state.
    Personal,
}

impl DurableScope {
    /// Returns the stable metadata label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Personal => "personal",
        }
    }
}

/// Host-owned request to promote selected session state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionRequest {
    /// Explicit host authorization for the promotion action.
    pub explicit_host_action: bool,
    /// Durable destination selected by the host.
    pub destination: DurableScope,
}

/// Pure promotion plan. It does not write durable state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionPlan {
    /// Source session-state identity.
    pub state_identity: String,
    /// Explicit durable destination.
    pub destination: DurableScope,
    /// Metadata-only promotion receipt.
    pub receipt: Receipt,
}

/// Plans an explicit host promotion without performing the promotion.
///
/// # Errors
///
/// Returns [`Error::ImplicitPromotion`] unless the host supplied explicit
/// promotion authority.
pub fn plan_promotion(
    session: &Session,
    request: &PromotionRequest,
    policy: &Policy,
) -> Result<PromotionPlan, Error> {
    validate_policy_identity(policy)?;
    if !request.explicit_host_action {
        return Err(Error::ImplicitPromotion);
    }
    let state_identity = session.state.identity();
    let receipt = Receipt::new(
        policy,
        ReceiptInput {
            target_class: TargetClass::Memory,
            operation: Operation::Update,
            status: ReceiptStatus::PromotionPlanned,
            snapshot_identity: state_identity.as_str(),
            state_identity: state_identity.as_str(),
            item_count: session.state.entries.len(),
            content_bytes: session.state.content_bytes(),
        },
    )?;
    Ok(PromotionPlan {
        state_identity,
        destination: request.destination,
        receipt,
    })
}

/// Receipt status for refinement history.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReceiptStatus {
    /// A proposal was admitted and planned.
    Admitted,
    /// An admitted snapshot was selected for rollback.
    RolledBack,
    /// An explicit host promotion was planned.
    PromotionPlanned,
}

impl ReceiptStatus {
    /// Returns the stable metadata label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::RolledBack => "rolled_back",
            Self::PromotionPlanned => "promotion_planned",
        }
    }
}

/// Metadata-only refinement history receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    /// Policy identity.
    pub policy_identity: String,
    /// Reviewed source cohort.
    pub cohort: String,
    /// Target class without target content.
    pub target_class: TargetClass,
    /// Operation class.
    pub operation: Operation,
    /// Receipt outcome.
    pub status: ReceiptStatus,
    /// Snapshot identity, never snapshot content.
    pub snapshot_identity: String,
    /// Resulting state identity.
    pub state_identity: String,
    /// Bounded item count.
    pub item_count: usize,
    /// Bounded content byte count.
    pub content_bytes: usize,
    /// Required non-claims.
    pub non_claims: Vec<String>,
    /// Domain-separated receipt identity.
    pub identity: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReceiptInput<'a> {
    target_class: TargetClass,
    operation: Operation,
    status: ReceiptStatus,
    snapshot_identity: &'a str,
    state_identity: &'a str,
    item_count: usize,
    content_bytes: usize,
}

impl Receipt {
    fn new(policy: &Policy, input: ReceiptInput<'_>) -> Result<Self, Error> {
        let ReceiptInput {
            target_class,
            operation,
            status,
            snapshot_identity,
            state_identity,
            item_count,
            content_bytes,
        } = input;
        validate_policy_identity(policy)?;
        validate_hash(snapshot_identity, BLAKE3_HEX_CHARS)?;
        validate_hash(state_identity, BLAKE3_HEX_CHARS)?;
        if !target_class.is_admitted() {
            return Err(Error::UnsupportedTarget { target_class });
        }
        if item_count > policy.max_state_entries || content_bytes > policy.max_state_bytes {
            return Err(Error::StateLimitExceeded);
        }
        let identity_material = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            policy.identity,
            target_class.as_str(),
            operation.as_str(),
            status.as_str(),
            snapshot_identity,
            state_identity,
            item_count,
            content_bytes,
        );
        Ok(Self {
            policy_identity: policy.identity.clone(),
            cohort: policy.cohort.clone(),
            target_class,
            operation,
            status,
            snapshot_identity: snapshot_identity.into(),
            state_identity: state_identity.into(),
            item_count,
            content_bytes,
            non_claims: policy.non_claims.clone(),
            identity: digest(RECEIPT_DOMAIN, &identity_material),
        })
    }
}

/// Unsafe material classes that a receipt must never contain.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UnsafeReceiptField {
    /// Raw lesson body.
    LessonBody,
    /// Prompt text.
    PromptText,
    /// Memory contents.
    MemoryContents,
    /// Skill body.
    SkillBody,
    /// Trajectory span.
    TrajectorySpan,
    /// Model improvement claim.
    ImprovementClaim,
    /// Correctness or generalization claim.
    CorrectnessClaim,
    /// Authority-widening claim.
    AuthorityWideningClaim,
}

/// Borrowed input used to construct a receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptDraftInput<'a> {
    /// Target class.
    pub target_class: TargetClass,
    /// Operation class.
    pub operation: Operation,
    /// Receipt status.
    pub status: ReceiptStatus,
    /// Snapshot identity.
    pub snapshot_identity: &'a str,
    /// Resulting state identity.
    pub state_identity: &'a str,
    /// Bounded item count.
    pub item_count: usize,
    /// Bounded content byte count.
    pub content_bytes: usize,
}

/// Receipt projection input. It has no content fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptDraft {
    /// Target class.
    pub target_class: TargetClass,
    /// Operation class.
    pub operation: Operation,
    /// Receipt status.
    pub status: ReceiptStatus,
    /// Snapshot identity.
    pub snapshot_identity: String,
    /// Resulting state identity.
    pub state_identity: String,
    /// Bounded item count.
    pub item_count: usize,
    /// Bounded content byte count.
    pub content_bytes: usize,
    /// Test and host boundary marker for prohibited material.
    pub unsafe_fields: Vec<UnsafeReceiptField>,
}

impl ReceiptDraft {
    /// Creates a safe receipt draft with no content fields.
    #[must_use]
    pub fn new(input: ReceiptDraftInput<'_>) -> Self {
        Self {
            target_class: input.target_class,
            operation: input.operation,
            status: input.status,
            snapshot_identity: input.snapshot_identity.into(),
            state_identity: input.state_identity.into(),
            item_count: input.item_count,
            content_bytes: input.content_bytes,
            unsafe_fields: Vec::new(),
        }
    }

    /// Adds a prohibited material marker for negative conformance tests.
    #[must_use]
    pub fn with_unsafe_field(mut self, field: UnsafeReceiptField) -> Self {
        self.unsafe_fields.push(field);
        self
    }
}

/// Validates and projects a metadata-only receipt draft.
///
/// # Errors
///
/// Rejects prohibited material, unsupported targets, malformed identities, and
/// policy or bound mismatches without echoing any sensitive material.
pub fn project_receipt(draft: &ReceiptDraft, policy: &Policy) -> Result<Receipt, Error> {
    if !draft.unsafe_fields.is_empty() {
        return Err(Error::ReceiptContainsContent);
    }
    Receipt::new(
        policy,
        ReceiptInput {
            target_class: draft.target_class,
            operation: draft.operation,
            status: draft.status,
            snapshot_identity: draft.snapshot_identity.as_str(),
            state_identity: draft.state_identity.as_str(),
            item_count: draft.item_count,
            content_bytes: draft.content_bytes,
        },
    )
}

/// Admits one validated proposal against current session-local state.
///
/// # Errors
///
/// Returns a typed fail-closed error before any state transition, snapshot
/// storage, receipt projection, or host effect.
pub fn admit(
    proposal: &Proposal,
    state: &SupplementalState,
    policy: &Policy,
) -> Result<AdmissionPlan, Error> {
    validate_policy_identity(policy)?;
    validate_proposal(proposal, policy)?;
    if state.entries.len() > policy.max_state_entries
        || state.content_bytes() > policy.max_state_bytes
    {
        return Err(Error::StateLimitExceeded);
    }
    let mut entries = state.entries.clone();
    let existing = state
        .entries
        .iter()
        .position(|entry| entry.target_class == proposal.target_class && entry.key == proposal.key);
    match proposal.operation {
        Operation::Add => {
            if existing.is_some() {
                return Err(Error::Conflict {
                    target_class: proposal.target_class,
                    key: proposal.key.clone(),
                });
            }
            entries.push(StateEntry {
                target_class: proposal.target_class,
                key: proposal.key.clone(),
                content: proposal.content.clone().ok_or(Error::MissingContent)?,
            });
        }
        Operation::Update => {
            let index = existing.ok_or_else(|| Error::MissingEntry {
                target_class: proposal.target_class,
                key: proposal.key.clone(),
            })?;
            entries[index].content = proposal.content.clone().ok_or(Error::MissingContent)?;
        }
        Operation::Retract => {
            let index = existing.ok_or_else(|| Error::MissingEntry {
                target_class: proposal.target_class,
                key: proposal.key.clone(),
            })?;
            entries.remove(index);
        }
    }
    let next_state = SupplementalState::new(entries)?;
    if next_state.entries.len() > policy.max_state_entries
        || next_state.content_bytes() > policy.max_state_bytes
    {
        return Err(Error::StateLimitExceeded);
    }
    let snapshot = Snapshot::from_state(state);
    if snapshot.state_bytes > policy.max_snapshot_bytes {
        return Err(Error::SnapshotTooLarge);
    }
    let receipt = Receipt::new(
        policy,
        ReceiptInput {
            target_class: proposal.target_class,
            operation: proposal.operation,
            status: ReceiptStatus::Admitted,
            snapshot_identity: snapshot.identity.as_str(),
            state_identity: next_state.identity().as_str(),
            item_count: next_state.entries.len(),
            content_bytes: next_state.content_bytes(),
        },
    )?;
    Ok(AdmissionPlan {
        snapshot,
        next_state,
        receipt,
    })
}

/// Typed fail-closed contract errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The generated policy failed validation.
    PolicyRejected,
    /// A generated policy identity was malformed.
    PolicyIdentityInvalid,
    /// A target class is outside the four admitted classes.
    UnsupportedTarget {
        /// Rejected class.
        target_class: TargetClass,
    },
    /// A bounded text field is empty or oversized.
    InvalidText {
        /// Field label.
        field: &'static str,
    },
    /// Evidence is missing or malformed.
    MalformedEvidence,
    /// Proposal content is missing for add or update.
    MissingContent,
    /// State entry is absent for update or retract.
    MissingEntry {
        /// Target class.
        target_class: TargetClass,
        /// Entry key.
        key: String,
    },
    /// The proposal conflicts with existing state.
    Conflict {
        /// Target class.
        target_class: TargetClass,
        /// Entry key.
        key: String,
    },
    /// A bound would be exceeded.
    StateLimitExceeded,
    /// A snapshot would exceed its byte bound.
    SnapshotTooLarge,
    /// A plan was built against a different current state.
    StalePlan,
    /// The selected snapshot is not retained.
    UnknownSnapshot,
    /// The selected snapshot identity does not match its state.
    ForgedSnapshot,
    /// A proposal attempted self-promotion or ambient persistence.
    ImplicitPromotion,
    /// A receipt draft included prohibited material.
    ReceiptContainsContent,
    /// A BLAKE3 identity was malformed.
    InvalidHash,
}

fn validate_policy_identity(policy: &Policy) -> Result<(), Error> {
    validate_hash(policy.identity.as_str(), BLAKE3_HEX_CHARS)
        .map_err(|_| Error::PolicyIdentityInvalid)
}

fn validate_proposal(proposal: &Proposal, policy: &Policy) -> Result<(), Error> {
    if !proposal.target_class.is_admitted() {
        return Err(Error::UnsupportedTarget {
            target_class: proposal.target_class,
        });
    }
    if proposal.requests_promotion {
        return Err(Error::ImplicitPromotion);
    }
    validate_bounded_text(proposal.key.as_str(), MAX_KEY_BYTES)
        .map_err(|_| Error::InvalidText { field: "key" })?;
    if proposal.evidence.is_empty() {
        return Err(Error::MalformedEvidence);
    }
    for evidence in &proposal.evidence {
        EvidenceRef::new(
            evidence.ledger_entry_id.as_str(),
            evidence.evidence_hash.as_str(),
        )?;
    }
    if let Some(content) = proposal.content.as_deref() {
        if content.is_empty() || content.len() > policy.max_content_bytes {
            return Err(Error::StateLimitExceeded);
        }
    } else if proposal.operation != Operation::Retract {
        return Err(Error::MissingContent);
    }
    if proposal.operation == Operation::Retract && proposal.content.is_some() {
        return Err(Error::MissingContent);
    }
    Ok(())
}

fn validate_state_entry(entry: &StateEntry) -> Result<(), Error> {
    if !entry.target_class.is_admitted() {
        return Err(Error::UnsupportedTarget {
            target_class: entry.target_class,
        });
    }
    validate_bounded_text(entry.key.as_str(), MAX_KEY_BYTES)
        .map_err(|_| Error::InvalidText { field: "key" })?;
    if entry.content.is_empty() {
        return Err(Error::InvalidText { field: "content" });
    }
    Ok(())
}

fn validate_bounded_text(value: &str, maximum: usize) -> Result<(), Error> {
    if value.is_empty() || value.len() > maximum {
        return Err(Error::InvalidText { field: "text" });
    }
    Ok(())
}

fn validate_hash(value: &str, maximum: usize) -> Result<(), Error> {
    if value.len() != maximum || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::InvalidHash);
    }
    Ok(())
}

fn compare_entries(left: &StateEntry, right: &StateEntry) -> Ordering {
    left.target_class
        .cmp(&right.target_class)
        .then_with(|| left.key.cmp(&right.key))
}

fn same_entry_key(left: &StateEntry, right: &StateEntry) -> bool {
    left.target_class == right.target_class && left.key == right.key
}

fn canonical_state(state: &SupplementalState) -> String {
    let mut result = String::new();
    for entry in &state.entries {
        result.push_str(entry.target_class.as_str());
        result.push('|');
        result.push_str(entry.key.as_str());
        result.push('|');
        result.push_str(entry.content.as_str());
        result.push('|');
    }
    result
}

fn digest(domain: &str, material: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[HASH_SEPARATOR]);
    hasher.update(material.as_bytes());
    format!("{}", hasher.finalize().to_hex())
}

fn join(values: &[String]) -> String {
    let mut result = String::new();
    for value in values {
        result.push_str(value.as_str());
        result.push('|');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    const CONTENT: &str = "retain bounded lesson";
    const EMPTY_STATE_BYTES: usize = 0;
    const EVIDENCE_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const STATE_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const SOURCE_REVISION: &str = "prime-agent@reviewed-continual-harness";
    const MAX_CONTENT_BYTES: usize = 128;
    const MAX_STATE_ENTRIES: usize = 8;
    const MAX_STATE_BYTES: usize = 512;
    const MAX_SNAPSHOTS: usize = 2;
    const MAX_SNAPSHOT_BYTES: usize = 512;

    fn policy() -> Policy {
        Policy::new(PolicyDraft {
            cohort: REVIEWED_COHORT.into(),
            source_revision: SOURCE_REVISION.into(),
            max_content_bytes: MAX_CONTENT_BYTES,
            max_state_entries: MAX_STATE_ENTRIES,
            max_state_bytes: MAX_STATE_BYTES,
            max_snapshots: MAX_SNAPSHOTS,
            max_snapshot_bytes: MAX_SNAPSHOT_BYTES,
            receipt_fields: vec![
                "policy_identity".into(),
                "target_class".into(),
                "operation".into(),
                "snapshot_identity".into(),
                "state_identity".into(),
                "status".into(),
            ],
            non_claims: vec![
                "not-lesson-correctness".into(),
                "not-model-improvement".into(),
                "not-generalization".into(),
                "not-base-prompt-mutation".into(),
                "not-automatic-promotion".into(),
                "not-cross-session-consistency".into(),
                "not-production-readiness".into(),
                "not-release-eligibility".into(),
            ],
        })
        .expect("valid policy")
    }

    fn evidence() -> EvidenceRef {
        EvidenceRef::new("ledger-entry-1", EVIDENCE_HASH).expect("valid evidence")
    }

    fn empty_state() -> SupplementalState {
        SupplementalState::new(Vec::new()).expect("empty state")
    }

    fn proposal(target_class: TargetClass, operation: Operation, key: &str) -> Proposal {
        Proposal::new(ProposalDraft {
            target_class,
            operation,
            key,
            content: Some(CONTENT),
            evidence: &[evidence()],
        })
    }

    #[test]
    fn admitted_update_snapshots_before_state_transition() {
        let policy = policy();
        let state = empty_state();
        let plan = admit(
            &proposal(TargetClass::Memory, Operation::Add, "lesson-1"),
            &state,
            &policy,
        )
        .expect("admitted proposal");
        assert_eq!(plan.snapshot.state, state);
        assert_eq!(plan.next_state.entries.len(), MIN_CONTENT_BYTES);
        assert_eq!(plan.receipt.status, ReceiptStatus::Admitted);
        let session = Session::new(state)
            .apply(&plan, &policy)
            .expect("apply plan");
        assert_eq!(session.snapshots.len(), MIN_CONTENT_BYTES);
        assert_eq!(session.state, plan.next_state);
    }

    #[test]
    fn rollback_restores_exact_retained_state() {
        let policy = policy();
        let state = empty_state();
        let first = admit(
            &proposal(TargetClass::Memory, Operation::Add, "lesson-1"),
            &state,
            &policy,
        )
        .expect("first plan");
        let session = Session::new(state)
            .apply(&first, &policy)
            .expect("first apply");
        let rollback = session
            .rollback(first.snapshot.identity.as_str(), &policy)
            .expect("rollback plan");
        assert_eq!(rollback.state, empty_state());
        assert_eq!(rollback.snapshot_identity, first.snapshot.identity);
        assert_eq!(rollback.receipt.status, ReceiptStatus::RolledBack);
    }

    #[test]
    fn explicit_host_promotion_is_planned_without_persistence() {
        let policy = policy();
        let session = Session::new(empty_state());
        let plan = plan_promotion(
            &session,
            &PromotionRequest {
                explicit_host_action: true,
                destination: DurableScope::Project,
            },
            &policy,
        )
        .expect("explicit promotion");
        assert_eq!(plan.destination, DurableScope::Project);
        assert_eq!(plan.receipt.status, ReceiptStatus::PromotionPlanned);
    }

    #[test]
    fn repeated_inputs_have_deterministic_policy_and_state_identities() {
        let first = policy();
        let second = policy();
        assert_eq!(first.identity, second.identity);
        assert_eq!(empty_state().identity(), empty_state().identity());
    }

    #[test]
    fn rejected_base_prompt_and_implicit_promotion_fail_before_effect() {
        let policy = policy();
        let state = empty_state();
        assert!(matches!(
            admit(
                &proposal(TargetClass::BaseSystemPrompt, Operation::Add, "base"),
                &state,
                &policy,
            ),
            Err(Error::UnsupportedTarget { .. })
        ));
        let mut implicit = proposal(TargetClass::Memory, Operation::Add, "lesson-1");
        implicit.requests_promotion = true;
        assert_eq!(
            admit(&implicit, &state, &policy),
            Err(Error::ImplicitPromotion)
        );
        assert_eq!(state, empty_state());
    }

    #[test]
    fn rejected_evidence_free_conflict_and_bound_violation_fail_closed() {
        let policy = policy();
        let state = empty_state();
        let mut evidence_free = proposal(TargetClass::Memory, Operation::Add, "lesson-1");
        evidence_free.evidence.clear();
        assert_eq!(
            admit(&evidence_free, &state, &policy),
            Err(Error::MalformedEvidence)
        );
        let first = admit(
            &proposal(TargetClass::Memory, Operation::Add, "lesson-1"),
            &state,
            &policy,
        )
        .expect("first plan");
        let current = Session::new(state.clone())
            .apply(&first, &policy)
            .expect("apply");
        assert!(matches!(
            admit(
                &proposal(TargetClass::Memory, Operation::Add, "lesson-1"),
                &current.state,
                &policy,
            ),
            Err(Error::Conflict { .. })
        ));
        let oversized = "x";
        let mut too_large = proposal(TargetClass::Memory, Operation::Add, "lesson-2");
        too_large.content = Some(
            alloc::string::String::from(oversized).repeat(MAX_CONTENT_BYTES + MIN_CONTENT_BYTES),
        );
        assert_eq!(
            admit(&too_large, &state, &policy),
            Err(Error::StateLimitExceeded)
        );
    }

    #[test]
    fn forged_unknown_and_evicted_snapshots_fail_closed() {
        let policy = policy();
        let session = Session::new(empty_state());
        assert_eq!(
            session.rollback(STATE_HASH, &policy),
            Err(Error::UnknownSnapshot)
        );
        let mut forged = Snapshot::from_state(&empty_state());
        forged.identity = STATE_HASH.into();
        let session = Session {
            state: empty_state(),
            snapshots: vec![forged],
        };
        assert_eq!(
            session.rollback(STATE_HASH, &policy),
            Err(Error::ForgedSnapshot)
        );
    }

    #[test]
    fn receipt_projection_rejects_content_and_overclaim_markers() {
        let policy = policy();
        let draft = ReceiptDraft::new(ReceiptDraftInput {
            target_class: TargetClass::Memory,
            operation: Operation::Add,
            status: ReceiptStatus::Admitted,
            snapshot_identity: STATE_HASH,
            state_identity: STATE_HASH,
            item_count: MIN_CONTENT_BYTES,
            content_bytes: CONTENT.len(),
        })
        .with_unsafe_field(UnsafeReceiptField::MemoryContents)
        .with_unsafe_field(UnsafeReceiptField::ImprovementClaim);
        assert_eq!(
            project_receipt(&draft, &policy),
            Err(Error::ReceiptContainsContent)
        );
        let safe = ReceiptDraft::new(ReceiptDraftInput {
            target_class: TargetClass::Memory,
            operation: Operation::Add,
            status: ReceiptStatus::Admitted,
            snapshot_identity: STATE_HASH,
            state_identity: STATE_HASH,
            item_count: MIN_CONTENT_BYTES,
            content_bytes: CONTENT.len(),
        });
        assert!(project_receipt(&safe, &policy).is_ok());
    }

    #[test]
    fn invalid_policy_fails_closed() {
        let mut draft = PolicyDraft {
            cohort: REVIEWED_COHORT.into(),
            source_revision: SOURCE_REVISION.into(),
            max_content_bytes: MAX_CONTENT_BYTES,
            max_state_entries: MAX_STATE_ENTRIES,
            max_state_bytes: MAX_STATE_BYTES,
            max_snapshots: MAX_SNAPSHOTS,
            max_snapshot_bytes: MAX_SNAPSHOT_BYTES,
            receipt_fields: vec!["policy_identity".into()],
            non_claims: Vec::new(),
        };
        assert_eq!(Policy::new(draft.clone()), Err(Error::PolicyRejected));
        draft.cohort = "unreviewed".into();
        draft.non_claims = vec!["not-a-claim".into(); MIN_NON_CLAIMS];
        assert_eq!(Policy::new(draft), Err(Error::PolicyRejected));
    }

    #[test]
    fn rejected_retract_content_is_not_accepted() {
        let policy = policy();
        let state = empty_state();
        let mut retract = proposal(TargetClass::Memory, Operation::Retract, "lesson-1");
        retract.content = Some(CONTENT.into());
        assert_eq!(admit(&retract, &state, &policy), Err(Error::MissingContent));
    }

    #[test]
    fn no_ambient_promotion_is_rejected() {
        let policy = policy();
        let request = PromotionRequest {
            explicit_host_action: false,
            destination: DurableScope::Personal,
        };
        assert_eq!(
            plan_promotion(&Session::new(empty_state()), &request, &policy),
            Err(Error::ImplicitPromotion)
        );
    }

    #[test]
    fn empty_state_size_is_named_and_stable() {
        assert_eq!(empty_state().content_bytes(), EMPTY_STATE_BYTES);
    }
}
