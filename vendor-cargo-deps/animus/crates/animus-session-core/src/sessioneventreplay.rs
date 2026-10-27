//! Pure bounded reconnect replay over caller-supplied admitted session facts.

#[allow(unused_imports)]
use crate::prelude::*;

#[allow(unused_imports)]
use blake3::Hasher;

const SCHEMA: &str = "animus.session-event-replay.v1";
const POLICY_DOMAIN: &str = "animus/session-event-replay-policy";
const RECEIPT_DOMAIN: &str = "animus/session-event-replay-receipt";
const MAX_IDENTITY_LENGTH: usize = 128;
const HASH_HEX_LENGTH: usize = 64;
const MIN_BOUND: u32 = 1;
const REQUIRED_NON_CLAIMS: [&str; 4] = [
    "not-transcript-truth",
    "not-transport-delivery",
    "not-durable-storage",
    "not-conversation-continuity",
];
const FORBIDDEN_RECEIPT_MARKERS: [&str; 8] = [
    "prompt:",
    "reasoning:",
    "tool arguments",
    "tool result",
    "credential",
    "environment=",
    "/home/",
    "transcript:",
];

macro_rules! identity_type {
    ($name:ident) => {
        #[doc = concat!("Nominal replay identity `", stringify!($name), "`.")]
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Parses a safe nominal identity.
            ///
            /// # Errors
            ///
            /// Returns [`ReplayError::InvalidIdentity`] for empty, oversized, or unsafe text.
            pub fn parse(value: &str) -> Result<Self, ReplayError> {
                if !is_safe_identity(value) {
                    return Err(ReplayError::InvalidIdentity);
                }
                Ok(Self(value.to_string()))
            }

            /// Returns the safe identity text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

identity_type!(ReplaySessionId);
identity_type!(SessionStreamId);
identity_type!(LedgerGenerationId);
identity_type!(ReplayCursorId);
identity_type!(ReplayPolicyId);
identity_type!(RedactionProfileId);
identity_type!(ActiveLeafId);
identity_type!(ReplayReceiptId);

/// Monotonic event sequence. Zero is reserved for the initial offset.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EventSequence(u64);

impl EventSequence {
    /// Creates a nonzero event sequence.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayError::InvalidSequence`] for zero.
    pub const fn new(value: u64) -> Result<Self, ReplayError> {
        if value == 0 {
            return Err(ReplayError::InvalidSequence);
        }
        Ok(Self(value))
    }

    /// Returns the sequence value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Durability class assigned before any client projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionEventClass {
    /// Completed semantic content bound to the admitted ledger.
    DurableSemantic,
    /// Partial streaming delta that is never replayed as completed content.
    LiveOnlyDelta,
    /// Admitted metadata observation without transcript content.
    MetadataObservation,
}

impl SessionEventClass {
    const fn is_replayable(self) -> bool {
        matches!(self, Self::DurableSemantic | Self::MetadataObservation)
    }
}

/// Already-loaded event fact derived from the admitted ledger and runtime order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionEventFact {
    /// Session identity.
    pub session_id: ReplaySessionId,
    /// Stream identity.
    pub stream_id: SessionStreamId,
    /// Ledger generation identity.
    pub generation_id: LedgerGenerationId,
    /// Branch or active-leaf identity.
    pub active_leaf_id: ActiveLeafId,
    /// Monotonic sequence.
    pub sequence: EventSequence,
    /// Event durability class.
    pub class: SessionEventClass,
    /// Safe BLAKE3 event identity.
    pub event_identity: String,
    /// Projected metadata byte count.
    pub projected_bytes: u32,
}

/// Complete reconnect cursor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCursor {
    /// Cursor identity.
    pub cursor_id: ReplayCursorId,
    /// Session identity.
    pub session_id: ReplaySessionId,
    /// Stream identity.
    pub stream_id: SessionStreamId,
    /// Ledger generation identity.
    pub generation_id: LedgerGenerationId,
    /// Branch or active-leaf identity.
    pub active_leaf_id: ActiveLeafId,
    /// Last observed sequence, or zero for an initial cursor.
    pub last_sequence: u64,
    /// Policy identity.
    pub policy_id: ReplayPolicyId,
    /// Redaction profile identity.
    pub redaction_profile_id: RedactionProfileId,
}

/// Generated Nickel replay policy input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayPolicyDraft {
    /// Policy label.
    pub policy_id: String,
    /// Maximum events in one replay response.
    pub maximum_event_count: u32,
    /// Maximum projected bytes in one replay response.
    pub maximum_projected_bytes: u32,
    /// Required redaction profile identity.
    pub redaction_profile_id: String,
    /// Whether stale generation or policy can select an explicit rebuild.
    pub allow_bounded_rebuild: bool,
    /// Required non-claims.
    pub non_claims: Vec<String>,
}

/// Revalidated replay policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayPolicy {
    id: ReplayPolicyId,
    maximum_event_count: u32,
    maximum_projected_bytes: u32,
    redaction_profile_id: RedactionProfileId,
    allow_bounded_rebuild: bool,
    non_claims: Vec<String>,
    identity: String,
}

impl ReplayPolicy {
    /// Revalidates generated policy and derives a BLAKE3 identity.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayError::PolicyRejected`] for missing bounds or non-claims.
    pub fn validate(draft: &ReplayPolicyDraft) -> Result<Self, ReplayError> {
        let id = ReplayPolicyId::parse(draft.policy_id.as_str())?;
        let redaction_profile_id = RedactionProfileId::parse(draft.redaction_profile_id.as_str())?;
        if draft.maximum_event_count < MIN_BOUND || draft.maximum_projected_bytes < MIN_BOUND {
            return Err(ReplayError::PolicyRejected);
        }
        if !has_required_values(&draft.non_claims, &REQUIRED_NON_CLAIMS) {
            return Err(ReplayError::PolicyRejected);
        }
        let event_count = draft.maximum_event_count.to_string();
        let byte_count = draft.maximum_projected_bytes.to_string();
        let rebuild = if draft.allow_bounded_rebuild {
            "rebuild"
        } else {
            "deny"
        };
        let identity = hash_parts(
            POLICY_DOMAIN,
            &[
                SCHEMA,
                id.as_str(),
                redaction_profile_id.as_str(),
                event_count.as_str(),
                byte_count.as_str(),
                rebuild,
            ],
        );
        Ok(Self {
            id,
            maximum_event_count: draft.maximum_event_count,
            maximum_projected_bytes: draft.maximum_projected_bytes,
            redaction_profile_id,
            allow_bounded_rebuild: draft.allow_bounded_rebuild,
            non_claims: draft.non_claims.clone(),
            identity,
        })
    }

    /// Returns the policy identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

/// Stream terminal state supplied by the session owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayStreamState {
    /// More events can arrive.
    Open,
    /// The stream ended normally.
    Completed,
    /// The stream ended after cancellation.
    Cancelled,
    /// The stream ended after a failure.
    Failed,
}

/// Replay planner input over already-loaded facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayRequest<'a> {
    /// Current session identity.
    pub session_id: &'a ReplaySessionId,
    /// Current stream identity.
    pub stream_id: &'a SessionStreamId,
    /// Current ledger generation.
    pub generation_id: &'a LedgerGenerationId,
    /// Current branch or active leaf.
    pub active_leaf_id: &'a ActiveLeafId,
    /// Current redaction profile.
    pub redaction_profile_id: RedactionProfileId,
    /// Optional reconnect cursor.
    pub cursor: Option<&'a ReplayCursor>,
    /// Admitted ledger-derived facts in canonical order.
    pub facts: &'a [SessionEventFact],
    /// Current terminal state.
    pub stream_state: ReplayStreamState,
}

/// Bounded replay outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayOutcome {
    /// Initial replay from the start of the admitted range.
    Initial,
    /// Exact top-up after a cursor offset.
    TopUp,
    /// No later replayable event exists.
    NoOp,
    /// Policy selected an explicit bounded rebuild.
    Rebuild,
    /// A sequence gap exists.
    Gap,
    /// A duplicate sequence exists.
    Duplicate,
    /// Cursor generation is stale and rebuild is denied.
    StaleGeneration,
    /// Cursor branch or active leaf drifted and rebuild is denied.
    BranchDrift,
    /// A policy bound truncated the selected range.
    Truncated,
    /// No later events exist and the stream is terminal.
    Terminal,
}

impl ReplayOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::TopUp => "top_up",
            Self::NoOp => "no_op",
            Self::Rebuild => "rebuild",
            Self::Gap => "gap",
            Self::Duplicate => "duplicate",
            Self::StaleGeneration => "stale_generation",
            Self::BranchDrift => "branch_drift",
            Self::Truncated => "truncated",
            Self::Terminal => "terminal",
        }
    }
}

/// Pure bounded replay plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayPlan {
    /// Replay outcome.
    pub outcome: ReplayOutcome,
    /// Selected replayable events in canonical order.
    pub events: Vec<SessionEventFact>,
    /// Live-only deltas omitted from the candidate range.
    pub omitted_live_delta_count: u32,
    /// Total projected bytes selected.
    pub projected_bytes: u32,
}

/// Client projection surface. It does not own ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayProjectionSurface {
    /// Agent Client Protocol projection.
    Acp,
    /// Headless protocol projection.
    Headless,
    /// Terminal UI projection.
    Tui,
}

/// Transport-neutral projection of one admitted event identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedReplayEvent {
    /// Projection surface.
    pub surface: ReplayProjectionSurface,
    /// Sequence preserved from the plan.
    pub sequence: EventSequence,
    /// Event identity preserved from the plan.
    pub event_identity: String,
    /// Durability class preserved from the plan.
    pub class: SessionEventClass,
}

/// Metadata-only replay receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayReceipt {
    /// Nominal receipt identity.
    pub receipt_id: ReplayReceiptId,
    /// BLAKE3 receipt identity.
    pub identity: String,
    /// Replay outcome.
    pub outcome: ReplayOutcome,
    /// Selected event count.
    pub event_count: u32,
    /// Omitted live-delta count.
    pub omitted_live_delta_count: u32,
    /// Selected byte count.
    pub projected_bytes: u32,
    /// First selected sequence, when present.
    pub first_sequence: Option<EventSequence>,
    /// Last selected sequence, when present.
    pub last_sequence: Option<EventSequence>,
    /// BLAKE3 identity over the exact selected event range.
    pub event_range_identity: String,
}

/// Typed replay failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayError {
    /// Identity text is empty, oversized, or unsafe.
    InvalidIdentity,
    /// Event sequence is zero.
    InvalidSequence,
    /// Generated policy is incomplete or unsafe.
    PolicyRejected,
    /// An event is detached from the requested admitted ledger facts.
    DetachedEvent,
    /// Event identity is not a lowercase BLAKE3 hash.
    InvalidEventIdentity,
    /// Projected byte count is zero.
    InvalidProjectedBytes,
    /// Count conversion exceeded the public bound.
    CountOverflow,
    /// Receipt text contains private material.
    UnsafeReceipt,
}

/// Plans bounded reconnect replay without storage or transport effects.
///
/// # Errors
///
/// Returns typed errors for malformed or detached facts. Duplicate, gap, drift,
/// rebuild, truncation, no-op, and terminal states are explicit plan outcomes.
pub fn plan_session_event_replay(
    policy: &ReplayPolicy,
    request: &ReplayRequest<'_>,
) -> Result<ReplayPlan, ReplayError> {
    validate_facts(request)?;
    if let Some(outcome) = sequence_fault(request.facts) {
        return Ok(empty_plan(outcome));
    }
    if let Some(outcome) = cursor_fence_outcome(policy, request) {
        if outcome == ReplayOutcome::Rebuild {
            let mut rebuild_request = request.clone();
            rebuild_request.cursor = None;
            let mut plan = select_replayable_events(policy, &rebuild_request)?;
            plan.outcome = ReplayOutcome::Rebuild;
            return Ok(plan);
        }
        return Ok(empty_plan(outcome));
    }
    if cursor_offset_has_gap(request) {
        return Ok(empty_plan(ReplayOutcome::Gap));
    }
    select_replayable_events(policy, request)
}

/// Projects one admitted plan to a client surface without changing order.
#[must_use]
pub fn project_replay_plan(
    plan: &ReplayPlan,
    surface: ReplayProjectionSurface,
) -> Vec<ProjectedReplayEvent> {
    plan.events
        .iter()
        .map(|event| ProjectedReplayEvent {
            surface,
            sequence: event.sequence,
            event_identity: event.event_identity.clone(),
            class: event.class,
        })
        .collect()
}

/// Projects a metadata-only receipt.
///
/// # Errors
///
/// Returns a typed error for malformed identities or count overflow.
pub fn project_replay_receipt(
    policy: &ReplayPolicy,
    receipt_id: ReplayReceiptId,
    plan: &ReplayPlan,
) -> Result<ReplayReceipt, ReplayError> {
    debug_assert!(has_required_values(
        &policy.non_claims,
        &REQUIRED_NON_CLAIMS
    ));
    let event_count = u32::try_from(plan.events.len()).map_err(|_| ReplayError::CountOverflow)?;
    let event_count_text = event_count.to_string();
    let omitted_text = plan.omitted_live_delta_count.to_string();
    let bytes_text = plan.projected_bytes.to_string();
    let first_sequence = plan.events.first().map(|event| event.sequence);
    let last_sequence = plan.events.last().map(|event| event.sequence);
    let event_range_identity = hash_event_range(plan);
    let identity = hash_parts(
        RECEIPT_DOMAIN,
        &[
            SCHEMA,
            policy.identity.as_str(),
            receipt_id.as_str(),
            plan.outcome.as_str(),
            event_count_text.as_str(),
            omitted_text.as_str(),
            bytes_text.as_str(),
            event_range_identity.as_str(),
        ],
    );
    Ok(ReplayReceipt {
        receipt_id,
        identity,
        outcome: plan.outcome,
        event_count,
        omitted_live_delta_count: plan.omitted_live_delta_count,
        projected_bytes: plan.projected_bytes,
        first_sequence,
        last_sequence,
        event_range_identity,
    })
}

/// Rejects private material from replay evidence text.
///
/// # Errors
///
/// Returns [`ReplayError::UnsafeReceipt`] for transcript, reasoning, tool,
/// credential, environment, or physical-path content.
pub fn validate_replay_receipt_text(value: &str) -> Result<(), ReplayError> {
    let normalized = value.to_ascii_lowercase();
    if FORBIDDEN_RECEIPT_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Err(ReplayError::UnsafeReceipt);
    }
    Ok(())
}

fn validate_facts(request: &ReplayRequest<'_>) -> Result<(), ReplayError> {
    for fact in request.facts {
        if fact.session_id != *request.session_id
            || fact.stream_id != *request.stream_id
            || fact.generation_id != *request.generation_id
            || fact.active_leaf_id != *request.active_leaf_id
        {
            return Err(ReplayError::DetachedEvent);
        }
        if !is_blake3(fact.event_identity.as_str()) {
            return Err(ReplayError::InvalidEventIdentity);
        }
        if fact.projected_bytes == 0 {
            return Err(ReplayError::InvalidProjectedBytes);
        }
    }
    Ok(())
}

fn cursor_fence_outcome(
    policy: &ReplayPolicy,
    request: &ReplayRequest<'_>,
) -> Option<ReplayOutcome> {
    if request.redaction_profile_id != policy.redaction_profile_id {
        return Some(rebuild_or(policy, ReplayOutcome::BranchDrift));
    }
    let cursor = request.cursor?;
    if cursor.session_id != *request.session_id || cursor.stream_id != *request.stream_id {
        return Some(ReplayOutcome::BranchDrift);
    }
    if cursor.generation_id != *request.generation_id {
        return Some(rebuild_or(policy, ReplayOutcome::StaleGeneration));
    }
    if cursor.active_leaf_id != *request.active_leaf_id {
        return Some(rebuild_or(policy, ReplayOutcome::BranchDrift));
    }
    if cursor.policy_id != policy.id || cursor.redaction_profile_id != request.redaction_profile_id
    {
        return Some(rebuild_or(policy, ReplayOutcome::BranchDrift));
    }
    None
}

fn rebuild_or(policy: &ReplayPolicy, denied: ReplayOutcome) -> ReplayOutcome {
    if policy.allow_bounded_rebuild {
        ReplayOutcome::Rebuild
    } else {
        denied
    }
}

fn cursor_offset_has_gap(request: &ReplayRequest<'_>) -> bool {
    let Some(cursor) = request.cursor else {
        return false;
    };
    if cursor.last_sequence == 0 {
        return false;
    }
    request
        .facts
        .last()
        .is_none_or(|last| cursor.last_sequence > last.sequence.value())
}

fn sequence_fault(facts: &[SessionEventFact]) -> Option<ReplayOutcome> {
    if facts
        .first()
        .is_some_and(|first| first.sequence.value() != 1)
    {
        return Some(ReplayOutcome::Gap);
    }
    for pair in facts.windows(2) {
        let previous = pair[0].sequence.value();
        let current = pair[1].sequence.value();
        if current == previous {
            return Some(ReplayOutcome::Duplicate);
        }
        if current != previous.saturating_add(1) {
            return Some(ReplayOutcome::Gap);
        }
    }
    None
}

fn select_replayable_events(
    policy: &ReplayPolicy,
    request: &ReplayRequest<'_>,
) -> Result<ReplayPlan, ReplayError> {
    let offset = request.cursor.map_or(0, |cursor| cursor.last_sequence);
    let mut events = Vec::new();
    let mut projected_bytes = 0_u32;
    let mut omitted_live_delta_count = 0_u32;
    let mut is_truncated = false;
    for fact in request
        .facts
        .iter()
        .filter(|fact| fact.sequence.value() > offset)
    {
        if !fact.class.is_replayable() {
            omitted_live_delta_count = omitted_live_delta_count
                .checked_add(1)
                .ok_or(ReplayError::CountOverflow)?;
            continue;
        }
        let next_count = u32::try_from(events.len())
            .map_err(|_| ReplayError::CountOverflow)?
            .checked_add(1)
            .ok_or(ReplayError::CountOverflow)?;
        let next_bytes = projected_bytes
            .checked_add(fact.projected_bytes)
            .ok_or(ReplayError::CountOverflow)?;
        if next_count > policy.maximum_event_count || next_bytes > policy.maximum_projected_bytes {
            is_truncated = true;
            break;
        }
        events.push(fact.clone());
        projected_bytes = next_bytes;
    }
    let outcome = classify_selected(request, events.is_empty(), is_truncated);
    Ok(ReplayPlan {
        outcome,
        events,
        omitted_live_delta_count,
        projected_bytes,
    })
}

fn classify_selected(
    request: &ReplayRequest<'_>,
    is_empty: bool,
    is_truncated: bool,
) -> ReplayOutcome {
    if is_truncated {
        return ReplayOutcome::Truncated;
    }
    if is_empty {
        return match request.stream_state {
            ReplayStreamState::Open => ReplayOutcome::NoOp,
            ReplayStreamState::Completed
            | ReplayStreamState::Cancelled
            | ReplayStreamState::Failed => ReplayOutcome::Terminal,
        };
    }
    if request.cursor.is_some() {
        ReplayOutcome::TopUp
    } else {
        ReplayOutcome::Initial
    }
}

fn empty_plan(outcome: ReplayOutcome) -> ReplayPlan {
    ReplayPlan {
        outcome,
        events: Vec::new(),
        omitted_live_delta_count: 0,
        projected_bytes: 0,
    }
}

fn is_safe_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTITY_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn is_blake3(value: &str) -> bool {
    value.len() == HASH_HEX_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn has_required_values(actual: &[String], required: &[&str]) -> bool {
    actual.len() == required.len()
        && required
            .iter()
            .all(|required_value| actual.iter().any(|value| value == required_value))
}

fn hash_event_range(plan: &ReplayPlan) -> String {
    let mut hasher = Hasher::new();
    hasher.update(b"animus/session-event-replay-range");
    for event in &plan.events {
        hasher.update(&[0]);
        hasher.update(event.event_identity.as_bytes());
        hasher.update(&event.sequence.value().to_le_bytes());
        let class = match event.class {
            SessionEventClass::DurableSemantic => b"durable-semantic".as_slice(),
            SessionEventClass::LiveOnlyDelta => b"live-only-delta".as_slice(),
            SessionEventClass::MetadataObservation => b"metadata-observation".as_slice(),
        };
        hasher.update(class);
    }
    hasher.finalize().to_hex().to_string()
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&[0]);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::{
        ActiveLeafId, EventSequence, LedgerGenerationId, REQUIRED_NON_CLAIMS, RedactionProfileId,
        ReplayCursor, ReplayCursorId, ReplayError, ReplayOutcome, ReplayPolicy, ReplayPolicyDraft,
        ReplayProjectionSurface, ReplayReceiptId, ReplayRequest, ReplaySessionId,
        ReplayStreamState, SessionEventClass, SessionEventFact, SessionStreamId,
        plan_session_event_replay, project_replay_plan, project_replay_receipt,
        validate_replay_receipt_text,
    };

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const EVENT_BYTES: u32 = 12;
    const MAX_EVENTS: u32 = 8;
    const MAX_BYTES: u32 = 256;

    fn policy() -> ReplayPolicy {
        ReplayPolicy::validate(&ReplayPolicyDraft {
            policy_id: "policy:replay".to_string(),
            maximum_event_count: MAX_EVENTS,
            maximum_projected_bytes: MAX_BYTES,
            redaction_profile_id: "redaction:safe".to_string(),
            allow_bounded_rebuild: true,
            non_claims: REQUIRED_NON_CLAIMS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        })
        .expect("policy")
    }

    fn fact(sequence: u64, class: SessionEventClass) -> SessionEventFact {
        SessionEventFact {
            session_id: ReplaySessionId::parse("session:one").expect("session"),
            stream_id: SessionStreamId::parse("stream:one").expect("stream"),
            generation_id: LedgerGenerationId::parse("generation:one").expect("generation"),
            active_leaf_id: ActiveLeafId::parse("leaf:one").expect("leaf"),
            sequence: EventSequence::new(sequence).expect("sequence"),
            class,
            event_identity: if sequence == 1 { HASH_A } else { HASH_B }.to_string(),
            projected_bytes: EVENT_BYTES,
        }
    }

    fn cursor(last_sequence: u64) -> ReplayCursor {
        ReplayCursor {
            cursor_id: ReplayCursorId::parse("cursor:one").expect("cursor"),
            session_id: ReplaySessionId::parse("session:one").expect("session"),
            stream_id: SessionStreamId::parse("stream:one").expect("stream"),
            generation_id: LedgerGenerationId::parse("generation:one").expect("generation"),
            active_leaf_id: ActiveLeafId::parse("leaf:one").expect("leaf"),
            last_sequence,
            policy_id: super::ReplayPolicyId::parse("policy:replay").expect("policy id"),
            redaction_profile_id: RedactionProfileId::parse("redaction:safe").expect("redaction"),
        }
    }

    fn request<'a>(
        facts: &'a [SessionEventFact],
        cursor: Option<&'a ReplayCursor>,
    ) -> ReplayRequest<'a> {
        ReplayRequest {
            session_id: &facts[0].session_id,
            stream_id: &facts[0].stream_id,
            generation_id: &facts[0].generation_id,
            active_leaf_id: &facts[0].active_leaf_id,
            redaction_profile_id: cursor.map_or_else(
                || RedactionProfileId::parse("redaction:safe").expect("redaction"),
                |value| value.redaction_profile_id.clone(),
            ),
            cursor,
            facts,
            stream_state: ReplayStreamState::Open,
        }
    }

    #[test]
    fn initial_top_up_noop_and_terminal_replay_are_deterministic() {
        // r[verify agent_harness_runtime.session_event_replay.verification]
        let facts = vec![
            fact(1, SessionEventClass::DurableSemantic),
            fact(2, SessionEventClass::MetadataObservation),
        ];
        let initial =
            plan_session_event_replay(&policy(), &request(&facts, None)).expect("initial");
        let top_cursor = cursor(1);
        let top_up = plan_session_event_replay(&policy(), &request(&facts, Some(&top_cursor)))
            .expect("top up");
        let end_cursor = cursor(2);
        let no_op = plan_session_event_replay(&policy(), &request(&facts, Some(&end_cursor)))
            .expect("no-op");
        let mut terminal_request = request(&facts, Some(&end_cursor));
        terminal_request.stream_state = ReplayStreamState::Completed;
        let terminal = plan_session_event_replay(&policy(), &terminal_request).expect("terminal");

        assert_eq!(initial.outcome, ReplayOutcome::Initial);
        assert_eq!(top_up.outcome, ReplayOutcome::TopUp);
        assert_eq!(top_up.events.len(), 1);
        assert_eq!(no_op.outcome, ReplayOutcome::NoOp);
        assert_eq!(terminal.outcome, ReplayOutcome::Terminal);
    }

    #[test]
    fn live_deltas_are_omitted_and_all_projections_preserve_order() {
        // r[verify agent_harness_runtime.session_event_replay.projection]
        let facts = vec![
            fact(1, SessionEventClass::DurableSemantic),
            fact(2, SessionEventClass::LiveOnlyDelta),
        ];
        let plan = plan_session_event_replay(&policy(), &request(&facts, None)).expect("plan");
        let acp = project_replay_plan(&plan, ReplayProjectionSurface::Acp);
        let headless = project_replay_plan(&plan, ReplayProjectionSurface::Headless);
        let tui = project_replay_plan(&plan, ReplayProjectionSurface::Tui);

        assert_eq!(plan.omitted_live_delta_count, 1);
        assert_eq!(acp[0].event_identity, headless[0].event_identity);
        assert_eq!(headless[0].event_identity, tui[0].event_identity);
    }

    #[test]
    fn duplicate_gap_stale_branch_and_bounds_fail_closed() {
        // r[verify agent_harness_runtime.session_event_replay.verification]
        let duplicate = vec![
            fact(1, SessionEventClass::DurableSemantic),
            fact(1, SessionEventClass::MetadataObservation),
        ];
        let gap = vec![
            fact(1, SessionEventClass::DurableSemantic),
            fact(3, SessionEventClass::MetadataObservation),
        ];
        let mut stale_cursor = cursor(1);
        stale_cursor.generation_id = LedgerGenerationId::parse("generation:old").expect("old");
        let future_cursor = cursor(3);
        let valid = vec![
            fact(1, SessionEventClass::DurableSemantic),
            fact(2, SessionEventClass::MetadataObservation),
        ];
        let mut narrow = ReplayPolicy::validate(&ReplayPolicyDraft {
            policy_id: "policy:replay".to_string(),
            maximum_event_count: 1,
            maximum_projected_bytes: MAX_BYTES,
            redaction_profile_id: "redaction:safe".to_string(),
            allow_bounded_rebuild: true,
            non_claims: REQUIRED_NON_CLAIMS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        })
        .expect("narrow");

        assert_eq!(
            plan_session_event_replay(&policy(), &request(&duplicate, None))
                .expect("duplicate")
                .outcome,
            ReplayOutcome::Duplicate
        );
        assert_eq!(
            plan_session_event_replay(&policy(), &request(&gap, None))
                .expect("gap")
                .outcome,
            ReplayOutcome::Gap
        );
        assert_eq!(
            plan_session_event_replay(&policy(), &request(&gap, Some(&stale_cursor)))
                .expect("gap takes precedence over ambiguous rebuild")
                .outcome,
            ReplayOutcome::Gap
        );
        let rebuilt = plan_session_event_replay(&policy(), &request(&valid, Some(&stale_cursor)))
            .expect("stale");
        assert_eq!(rebuilt.outcome, ReplayOutcome::Rebuild);
        assert_eq!(rebuilt.events.len(), valid.len());
        assert_eq!(
            plan_session_event_replay(&narrow, &request(&valid, None))
                .expect("truncated")
                .outcome,
            ReplayOutcome::Truncated
        );
        assert_eq!(
            plan_session_event_replay(&policy(), &request(&valid, Some(&future_cursor)))
                .expect("future cursor")
                .outcome,
            ReplayOutcome::Gap
        );
        narrow.allow_bounded_rebuild = false;
        assert_eq!(
            plan_session_event_replay(&narrow, &request(&valid, Some(&stale_cursor)))
                .expect("denied stale")
                .outcome,
            ReplayOutcome::StaleGeneration
        );
    }

    #[test]
    fn detached_unsafe_and_receipt_material_are_rejected() {
        // r[verify agent_harness_runtime.session_event_replay.evidence]
        let facts = vec![fact(1, SessionEventClass::DurableSemantic)];
        let plan = plan_session_event_replay(&policy(), &request(&facts, None)).expect("plan");
        let receipt = project_replay_receipt(
            &policy(),
            ReplayReceiptId::parse("receipt:one").expect("receipt id"),
            &plan,
        )
        .expect("receipt");
        let mut detached = facts.clone();
        detached[0].session_id = ReplaySessionId::parse("session:other").expect("other");
        let detached_request = ReplayRequest {
            facts: &detached,
            ..request(&facts, None)
        };
        let mut malformed = facts.clone();
        malformed[0].event_identity = "not-a-hash".to_string();
        let malformed_request = ReplayRequest {
            facts: &malformed,
            ..request(&facts, None)
        };
        let mut drift_cursor = cursor(1);
        drift_cursor.active_leaf_id = ActiveLeafId::parse("leaf:old").expect("old leaf");
        let mut redaction_drift_request = request(&facts, None);
        redaction_drift_request.redaction_profile_id =
            RedactionProfileId::parse("redaction:other").expect("other redaction");
        let mut changed_plan = plan.clone();
        changed_plan.events[0].event_identity = HASH_B.to_string();
        let changed_receipt = project_replay_receipt(
            &policy(),
            ReplayReceiptId::parse("receipt:one").expect("receipt id"),
            &changed_plan,
        )
        .expect("changed receipt");
        let invalid_policy = ReplayPolicy::validate(&ReplayPolicyDraft {
            policy_id: "policy:replay".to_string(),
            maximum_event_count: 0,
            maximum_projected_bytes: MAX_BYTES,
            redaction_profile_id: "redaction:safe".to_string(),
            allow_bounded_rebuild: true,
            non_claims: Vec::new(),
        });

        assert_eq!(receipt.identity.len(), super::HASH_HEX_LENGTH);
        assert_eq!(receipt.event_range_identity.len(), super::HASH_HEX_LENGTH);
        assert_ne!(receipt.identity, changed_receipt.identity);
        assert_eq!(invalid_policy, Err(ReplayError::PolicyRejected));
        assert_eq!(
            plan_session_event_replay(&policy(), &detached_request),
            Err(ReplayError::DetachedEvent)
        );
        assert_eq!(
            plan_session_event_replay(&policy(), &malformed_request),
            Err(ReplayError::InvalidEventIdentity)
        );
        assert_eq!(
            plan_session_event_replay(&policy(), &request(&facts, Some(&drift_cursor)))
                .expect("branch drift")
                .outcome,
            ReplayOutcome::Rebuild
        );
        assert_eq!(
            plan_session_event_replay(&policy(), &redaction_drift_request)
                .expect("redaction drift")
                .outcome,
            ReplayOutcome::Rebuild
        );
        assert_eq!(
            validate_replay_receipt_text("counts classes omissions"),
            Ok(())
        );
        assert_eq!(
            validate_replay_receipt_text("prompt: secret"),
            Err(ReplayError::UnsafeReceipt)
        );
    }
}
