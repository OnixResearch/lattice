//! Pure validation, projection, disposition, and recovery-control logic.

type AdmissionRequest = super::data::AdmissionRequest;
type Boundary = super::data::Boundary;
type CommitObservation = super::data::CommitObservation;
type CommitOutcome = super::data::CommitOutcome;
type CompletedDisposition = super::data::CompletedDisposition;
type CorruptionClass = super::data::CorruptionClass;
type DispositionDecision = super::data::DispositionDecision;
type DispositionKind = super::data::DispositionKind;
type DispositionPlan = super::data::DispositionPlan;
type DispositionRequest = super::data::DispositionRequest;
type Entry = super::data::Entry;
type EntryDraft = super::data::EntryDraft;
type EntryIdentity = super::data::EntryIdentity;
type Error = super::data::Error;
type Inspection = super::data::Inspection;
type InspectionEntry = super::data::InspectionEntry;
type Issue = super::data::Issue;
type PlanIdentity = super::data::PlanIdentity;
type Policy = super::data::Policy;
type Projection = super::data::Projection;
type RecoveryControlPlan = super::data::RecoveryControlPlan;
type RecoveryControlRequest = super::data::RecoveryControlRequest;
type RecoveryState = super::data::RecoveryState;
type StartupFacts = super::data::StartupFacts;

const PROJECTION_DOMAIN: &str = "animus.pending-entry-recovery.projection.v1";
const DISPOSITION_DOMAIN: &str = "animus.pending-entry-recovery.disposition.v1";
const CONTROL_DOMAIN: &str = "animus.pending-entry-recovery.control.v1";
const HASH_SEPARATOR: &[u8] = b"\0";
const FIRST_SEQUENCE: u64 = 1;
const MAX_METADATA_BYTES_USIZE: usize = 256;
const BLAKE3_HEX_LENGTH_USIZE: usize = 64;

#[derive(Clone, Copy)]
struct DispositionHashFacts<'a> {
    id: &'a super::data::DispositionIdentity,
    projection: &'a PlanIdentity,
    boundary: &'a Boundary,
    kind: DispositionKind,
    selected: &'a [EntryIdentity],
}

#[derive(Clone, Copy)]
struct HashBounds {
    entries: usize,
    attachments: usize,
}

/// Validates finite policy limits.
///
/// # Errors
///
/// Returns a bounded limit issue for zero or unsupported limits.
pub fn validate_policy(policy: &Policy) -> Result<(), Error> {
    let is_entry_count_valid = policy.entries > 0 && policy.entries <= super::data::MAX_ENTRY_COUNT;
    let is_attachment_count_valid =
        policy.attachments_per_entry <= super::data::MAX_ATTACHMENT_COUNT;
    if !is_entry_count_valid {
        return Err(Error::new(Issue::EntryLimit));
    }
    if policy.entry_bytes.value() == 0 || policy.total_bytes == 0 {
        return Err(Error::new(Issue::EntryByteLimit));
    }
    if !is_attachment_count_valid {
        return Err(Error::new(Issue::AttachmentLimit));
    }
    Ok(())
}

/// Validates one owner-supplied entry without loading its body.
///
/// # Errors
///
/// Returns a bounded metadata, identity, scope, generation, or limit issue.
// r[impl agent_harness_runtime.pending_input_vault.contents]
pub fn validate_entry(
    policy: &Policy,
    boundary: &Boundary,
    draft: EntryDraft,
) -> Result<Entry, Error> {
    validate_policy(policy)?;
    validate_boundary(boundary)?;
    validate_metadata(draft.id.as_str())?;
    validate_metadata(draft.provenance.as_str())?;
    validate_blake3(draft.body_identity.as_str())?;
    if draft.conversation != boundary.conversation {
        return Err(Error::new(Issue::CrossConversation));
    }
    if draft.generation != boundary.generation {
        return Err(Error::new(Issue::MixedGeneration));
    }
    if draft.sequence.value() == 0 {
        return Err(Error::new(Issue::MissingSequence));
    }
    if draft.body_bytes.value() > policy.entry_bytes.value() {
        return Err(Error::new(Issue::EntryByteLimit));
    }
    let attachment_count = count_u32(draft.attachments.len())?;
    if attachment_count > policy.attachments_per_entry {
        return Err(Error::new(Issue::AttachmentLimit));
    }
    validate_attachments(draft.attachments.as_slice())?;
    debug_assert_eq!(draft.conversation, boundary.conversation);
    debug_assert!(draft.body_bytes.value() <= policy.entry_bytes.value());
    Ok(Entry {
        id: draft.id,
        conversation: draft.conversation,
        generation: draft.generation,
        sequence: draft.sequence,
        class: draft.class,
        body_identity: draft.body_identity,
        body_bytes: draft.body_bytes,
        attachments: draft.attachments,
        provenance: draft.provenance,
    })
}

/// Projects supplied durable facts into one deterministic startup state.
///
/// # Errors
///
/// Returns a bounded boundary or policy issue. Entry corruption becomes
/// `BlockedCorrupt` and never yields a partial recoverable suffix.
// r[impl agent_harness_runtime.pending_input_vault.boundary]
// r[impl agent_harness_runtime.pending_input_vault.recovery]
pub fn project_startup(policy: &Policy, facts: &StartupFacts) -> Result<Projection, Error> {
    validate_policy(policy)?;
    validate_boundary(&facts.boundary)?;
    let corruption = classify_corruption(policy, facts);
    let state = select_state(facts, corruption)?;
    let entries = if matches!(state, RecoveryState::BlockedCorrupt(_)) {
        Vec::new()
    } else {
        facts.entries.clone()
    };
    let identity = projection_identity(policy, facts, &state);
    debug_assert!(!identity.as_str().is_empty());
    Ok(Projection {
        boundary: facts.boundary.clone(),
        entries,
        state,
        identity,
    })
}

/// Returns bounded metadata for explicit operator review.
///
/// # Errors
///
/// Returns invalid state when the projection does not permit inspection.
pub fn inspect_projection(projection: &Projection) -> Result<Inspection, Error> {
    let is_permitted = matches!(
        projection.state,
        RecoveryState::ReviewRequired
            | RecoveryState::CleanupRequired
            | RecoveryState::BlockedCorrupt(_)
    );
    if !is_permitted {
        return Err(Error::new(Issue::InvalidState));
    }
    let mut entries = Vec::with_capacity(projection.entries.len());
    for entry in &projection.entries {
        entries.push(InspectionEntry {
            id: entry.id.clone(),
            sequence: entry.sequence,
            class: entry.class,
            body_identity: entry.body_identity.clone(),
            body_bytes: entry.body_bytes,
            attachment_count: count_u32(entry.attachments.len())?,
            provenance: entry.provenance.clone(),
        });
    }
    debug_assert_eq!(entries.len(), projection.entries.len());
    debug_assert!(entries.iter().all(|entry| entry.sequence.value() > 0));
    Ok(Inspection {
        identity: projection.identity.clone(),
        state: projection.state.clone(),
        entries,
    })
}

/// Reports whether ordinary ingress can resume.
#[must_use]
pub const fn ordinary_ingress_allowed(state: &RecoveryState) -> bool {
    matches!(
        state,
        RecoveryState::NoVault | RecoveryState::DispositionComplete
    )
}

/// Plans one generation-fenced restore or discard transaction.
///
/// # Errors
///
/// Returns stale, conflicting, unauthorized, or invalid-state before mutation.
// r[impl agent_harness_runtime.pending_input_vault.transactions]
pub fn plan_disposition(
    projection: &Projection,
    request: &DispositionRequest,
    prior: Option<&CompletedDisposition>,
) -> Result<DispositionDecision, Error> {
    validate_request_scope(projection, request)?;
    if let Some(completed) = prior {
        return repeated_or_conflicting(projection, request, completed);
    }
    validate_new_request(projection, request)?;
    let selected = canonical_selection(projection, request.selected.as_slice())?;
    if selected != request.selected {
        return Err(Error::new(Issue::Conflict));
    }
    let plan_identity = disposition_identity(request, selected.as_slice());
    let plan = DispositionPlan {
        id: request.id.clone(),
        plan_identity,
        projection: request.projection.clone(),
        boundary: request.boundary.clone(),
        expected_generation: request.expected_generation,
        selected,
        kind: request.kind,
    };
    validate_disposition_plan(projection, &plan)?;
    debug_assert_eq!(plan.boundary, projection.boundary);
    debug_assert!(!plan.selected.is_empty());
    Ok(DispositionDecision::Planned(plan))
}

/// Validates one exact disposition plan before a host mutation.
///
/// # Errors
///
/// Returns stale, conflicting, or invalid-state for a forged or drifted plan.
pub fn validate_disposition_plan(
    projection: &Projection,
    plan: &DispositionPlan,
) -> Result<(), Error> {
    if plan.projection != projection.identity
        || plan.boundary != projection.boundary
        || plan.expected_generation != projection.boundary.generation
    {
        return Err(Error::new(Issue::Stale));
    }
    if !disposition_kind_permitted(&projection.state, plan.kind) {
        return Err(Error::new(Issue::InvalidState));
    }
    validate_metadata(plan.id.as_str())?;
    let selected = canonical_selection(projection, plan.selected.as_slice())?;
    if selected != plan.selected {
        return Err(Error::new(Issue::Conflict));
    }
    let expected = hash_disposition(DispositionHashFacts {
        id: &plan.id,
        projection: &plan.projection,
        boundary: &plan.boundary,
        kind: plan.kind,
        selected: plan.selected.as_slice(),
    });
    if expected != plan.plan_identity {
        return Err(Error::new(Issue::Conflict));
    }
    debug_assert_eq!(selected, plan.selected);
    debug_assert_eq!(expected, plan.plan_identity);
    Ok(())
}

/// Validates retained completion evidence before cleanup or restart use.
///
/// # Errors
///
/// Returns a bounded issue for forged, malformed, or inconsistent evidence.
pub fn validate_completed(completed: &CompletedDisposition) -> Result<(), Error> {
    validate_boundary(&completed.boundary)?;
    validate_metadata(completed.id.as_str())?;
    validate_plan_identity(completed.projection.as_str())?;
    if completed.cleanup == super::data::CleanupState::NotRequired {
        return Err(Error::new(Issue::Conflict));
    }
    validate_completed_selection(completed)?;
    let expected = hash_disposition(DispositionHashFacts {
        id: &completed.id,
        projection: &completed.projection,
        boundary: &completed.boundary,
        kind: completed.kind,
        selected: completed.selected.as_slice(),
    });
    if expected != completed.plan_identity {
        return Err(Error::new(Issue::Conflict));
    }
    debug_assert_eq!(expected, completed.plan_identity);
    Ok(())
}

/// Reduces exact commit evidence into completed, retryable, or corrupt state.
///
/// # Errors
///
/// Returns stale when host evidence does not bind the exact plan.
pub fn reduce_commit(
    projection: &Projection,
    plan: &DispositionPlan,
    observation: &CommitObservation,
) -> Result<CommitOutcome, Error> {
    validate_disposition_plan(projection, plan)?;
    if observation.plan_identity != plan.plan_identity
        || observation.generation != plan.expected_generation
    {
        return Err(Error::new(Issue::Stale));
    }
    debug_assert_eq!(observation.plan_identity, plan.plan_identity);
    debug_assert_eq!(observation.generation, plan.expected_generation);
    match observation.status {
        super::data::CommitStatus::Failed => Ok(CommitOutcome::ReviewRequired),
        super::data::CommitStatus::Uncertain => Ok(CommitOutcome::BlockedCorrupt(
            CorruptionClass::UncertainCommit,
        )),
        super::data::CommitStatus::Committed => {
            let is_retention_exact =
                observation.restore_material_retained == (plan.kind == DispositionKind::Restore);
            if !is_retention_exact {
                return Err(Error::new(Issue::Conflict));
            }
            let admission_requests = admission_requests(projection, plan)?;
            let completed = CompletedDisposition {
                id: plan.id.clone(),
                plan_identity: plan.plan_identity.clone(),
                projection: plan.projection.clone(),
                boundary: plan.boundary.clone(),
                selected: plan.selected.clone(),
                kind: plan.kind,
                admission_requests,
                restore_material_retained: observation.restore_material_retained,
                cleanup: super::data::CleanupState::Required,
            };
            validate_completed(&completed)?;
            Ok(CommitOutcome::Completed(Box::new(completed)))
        }
    }
}

/// Plans a separately authorized recovery-only control.
///
/// # Errors
///
/// Returns unauthorized, stale, or invalid-state without granting ingress.
// r[impl agent_harness_runtime.pending_input_vault.corruption]
pub fn plan_recovery_control(
    projection: &Projection,
    request: &RecoveryControlRequest,
) -> Result<RecoveryControlPlan, Error> {
    if !request.authorized {
        return Err(Error::new(Issue::Unauthorized));
    }
    if !matches!(projection.state, RecoveryState::BlockedCorrupt(_)) {
        return Err(Error::new(Issue::InvalidState));
    }
    if request.boundary != projection.boundary || request.projection != projection.identity {
        return Err(Error::new(Issue::Stale));
    }
    debug_assert!(request.authorized);
    debug_assert_eq!(request.projection, projection.identity);
    let plan_identity =
        hash_control_fields(&projection.identity, &projection.boundary, request.kind);
    let plan = RecoveryControlPlan {
        boundary: request.boundary.clone(),
        projection: request.projection.clone(),
        kind: request.kind,
        plan_identity,
    };
    validate_recovery_plan(&plan)?;
    Ok(plan)
}

/// Validates one recovery-only plan before a host effect.
///
/// # Errors
///
/// Returns a bounded issue for malformed or forged plan metadata.
pub fn validate_recovery_plan(plan: &RecoveryControlPlan) -> Result<(), Error> {
    validate_boundary(&plan.boundary)?;
    validate_plan_identity(plan.projection.as_str())?;
    let expected = hash_control_fields(&plan.projection, &plan.boundary, plan.kind);
    if expected != plan.plan_identity {
        return Err(Error::new(Issue::Conflict));
    }
    debug_assert_eq!(expected, plan.plan_identity);
    Ok(())
}

/// Validates one explicit conversation and store boundary before effects.
///
/// # Errors
///
/// Returns a bounded missing or unsafe metadata issue.
pub fn validate_boundary(boundary: &Boundary) -> Result<(), Error> {
    for value in [
        boundary.id.as_str(),
        boundary.conversation.as_str(),
        boundary.branch.as_str(),
        boundary.store.as_str(),
        boundary.authority.as_str(),
        boundary.policy.as_str(),
    ] {
        validate_metadata(value)?;
    }
    if boundary.generation.value() == 0 {
        return Err(Error::new(Issue::MissingBoundary));
    }
    Ok(())
}

pub(super) fn validate_metadata(value: &str) -> Result<(), Error> {
    let length_bytes = u32::try_from(value.len()).map_err(|_| Error::new(Issue::UnsafeMetadata))?;
    if value.is_empty() || length_bytes > super::data::MAX_METADATA_BYTES {
        return Err(Error::new(Issue::UnsafeMetadata));
    }
    let has_safe_characters = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'));
    if !has_safe_characters {
        return Err(Error::new(Issue::UnsafeMetadata));
    }
    Ok(())
}

fn validate_blake3(value: &str) -> Result<(), Error> {
    if !is_blake3(value) {
        return Err(Error::new(Issue::InvalidBodyIdentity));
    }
    Ok(())
}

fn validate_plan_identity(value: &str) -> Result<(), Error> {
    if !is_blake3(value) {
        return Err(Error::new(Issue::Conflict));
    }
    Ok(())
}

pub(super) fn is_blake3(value: &str) -> bool {
    let Ok(length_bytes) = u32::try_from(value.len()) else {
        return false;
    };
    length_bytes == super::data::BLAKE3_HEX_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_attachments(attachments: &[super::data::AttachmentIdentity]) -> Result<(), Error> {
    let mut seen = std::collections::BTreeSet::new();
    for attachment in attachments {
        validate_blake3(attachment.as_str())?;
        if !seen.insert(attachment.as_str()) {
            return Err(Error::new(Issue::Conflict));
        }
    }
    Ok(())
}

fn classify_corruption(policy: &Policy, facts: &StartupFacts) -> Option<CorruptionClass> {
    if facts.uncertain_commit
        || facts
            .completed
            .as_ref()
            .is_some_and(|completed| validate_completed(completed).is_err())
    {
        return Some(CorruptionClass::UncertainCommit);
    }
    let Ok(entry_count) = u32::try_from(facts.entries.len()) else {
        return Some(CorruptionClass::OverBudget);
    };
    if entry_count > policy.entries {
        return Some(CorruptionClass::OverBudget);
    }
    classify_entry_corruption(policy, &facts.boundary, facts.entries.as_slice())
}

fn classify_entry_corruption(
    policy: &Policy,
    boundary: &Boundary,
    entries: &[Entry],
) -> Option<CorruptionClass> {
    let mut expected_sequence = FIRST_SEQUENCE;
    let mut total_bytes = 0_u64;
    let mut seen = std::collections::BTreeSet::new();
    for entry in entries {
        if entry.conversation != boundary.conversation {
            return Some(CorruptionClass::CrossConversation);
        }
        if entry.generation != boundary.generation {
            return Some(CorruptionClass::MixedGeneration);
        }
        if !seen.insert(entry.id.as_str()) {
            return Some(CorruptionClass::Duplicate);
        }
        let sequence = entry.sequence.value();
        if sequence < expected_sequence {
            return Some(CorruptionClass::Reordered);
        }
        if sequence > expected_sequence {
            return Some(CorruptionClass::MissingSequence);
        }
        if entry.body_bytes.value() > policy.entry_bytes.value() {
            return Some(CorruptionClass::OverBudget);
        }
        if validate_entry_material(policy, entry).is_err() {
            return Some(CorruptionClass::Malformed);
        }
        total_bytes = match total_bytes.checked_add(u64::from(entry.body_bytes.value())) {
            Some(value) => value,
            None => return Some(CorruptionClass::OverBudget),
        };
        if total_bytes > policy.total_bytes {
            return Some(CorruptionClass::OverBudget);
        }
        expected_sequence = match expected_sequence.checked_add(1) {
            Some(value) => value,
            None => return Some(CorruptionClass::OverBudget),
        };
    }
    debug_assert!(total_bytes <= policy.total_bytes);
    debug_assert!(expected_sequence >= FIRST_SEQUENCE);
    None
}

fn validate_entry_material(policy: &Policy, entry: &Entry) -> Result<(), Error> {
    validate_metadata(entry.id.as_str())?;
    validate_metadata(entry.provenance.as_str())?;
    validate_blake3(entry.body_identity.as_str())?;
    let attachment_count = count_u32(entry.attachments.len())?;
    if attachment_count > policy.attachments_per_entry {
        return Err(Error::new(Issue::AttachmentLimit));
    }
    validate_attachments(entry.attachments.as_slice())
}

fn select_state(
    facts: &StartupFacts,
    corruption: Option<CorruptionClass>,
) -> Result<RecoveryState, Error> {
    if let Some(class) = corruption {
        return Ok(RecoveryState::BlockedCorrupt(class));
    }
    debug_assert!(corruption.is_none());
    debug_assert!(facts.boundary.generation.value() > 0);
    if let Some(completed) = facts.completed.as_ref() {
        validate_completed_boundary(completed, &facts.boundary)?;
        return Ok(match facts.cleanup {
            super::data::CleanupState::Required | super::data::CleanupState::Failed => {
                RecoveryState::CleanupRequired
            }
            super::data::CleanupState::NotRequired | super::data::CleanupState::Complete => {
                RecoveryState::DispositionComplete
            }
        });
    }
    if facts.entries.is_empty() {
        return Ok(RecoveryState::NoVault);
    }
    Ok(RecoveryState::ReviewRequired)
}

fn validate_completed_boundary(
    completed: &CompletedDisposition,
    boundary: &Boundary,
) -> Result<(), Error> {
    validate_completed(completed)?;
    if completed.boundary != *boundary {
        return Err(Error::new(Issue::Stale));
    }
    Ok(())
}

fn validate_request_scope(
    projection: &Projection,
    request: &DispositionRequest,
) -> Result<(), Error> {
    if !request.authorized {
        return Err(Error::new(Issue::Unauthorized));
    }
    if request.boundary != projection.boundary
        || request.expected_generation != projection.boundary.generation
    {
        return Err(Error::new(Issue::Stale));
    }
    validate_metadata(request.id.as_str())?;
    debug_assert!(request.authorized);
    debug_assert_eq!(request.boundary, projection.boundary);
    Ok(())
}

fn validate_new_request(
    projection: &Projection,
    request: &DispositionRequest,
) -> Result<(), Error> {
    if request.projection != projection.identity {
        return Err(Error::new(Issue::Stale));
    }
    if !disposition_kind_permitted(&projection.state, request.kind) {
        return Err(Error::new(Issue::InvalidState));
    }
    debug_assert_eq!(request.projection, projection.identity);
    Ok(())
}

const fn disposition_kind_permitted(state: &RecoveryState, kind: DispositionKind) -> bool {
    match state {
        RecoveryState::ReviewRequired => {
            matches!(kind, DispositionKind::Restore | DispositionKind::Discard)
        }
        RecoveryState::BlockedCorrupt(_)
        | RecoveryState::NoVault
        | RecoveryState::DispositionPending
        | RecoveryState::DispositionComplete
        | RecoveryState::CleanupRequired => false,
    }
}

fn canonical_selection(
    projection: &Projection,
    selected: &[EntryIdentity],
) -> Result<Vec<EntryIdentity>, Error> {
    let selected_count =
        u32::try_from(selected.len()).map_err(|_| Error::new(Issue::EntryLimit))?;
    if selected.is_empty() || selected_count > super::data::MAX_ENTRY_COUNT {
        return Err(Error::new(Issue::Conflict));
    }
    let mut requested = std::collections::BTreeSet::new();
    for entry in selected {
        if !requested.insert(entry.as_str()) {
            return Err(Error::new(Issue::Conflict));
        }
    }
    let mut canonical = Vec::with_capacity(selected.len());
    for entry in &projection.entries {
        if requested.contains(entry.id.as_str()) {
            canonical.push(entry.id.clone());
        }
    }
    if canonical.len() != selected.len() {
        return Err(Error::new(Issue::Conflict));
    }
    debug_assert_eq!(canonical.len(), selected.len());
    debug_assert!(!canonical.is_empty());
    Ok(canonical)
}

fn repeated_or_conflicting(
    projection: &Projection,
    request: &DispositionRequest,
    completed: &CompletedDisposition,
) -> Result<DispositionDecision, Error> {
    validate_completed_boundary(completed, &projection.boundary)?;
    let is_repeat_state = matches!(
        projection.state,
        RecoveryState::ReviewRequired
            | RecoveryState::CleanupRequired
            | RecoveryState::DispositionComplete
    );
    if !is_repeat_state {
        return Err(Error::new(Issue::InvalidState));
    }
    let is_binding_exact = request.id == completed.id
        && request.projection == completed.projection
        && request.boundary == completed.boundary
        && request.expected_generation == completed.boundary.generation
        && request.selected == completed.selected
        && request.kind == completed.kind;
    if !is_binding_exact {
        return Err(Error::new(Issue::Conflict));
    }
    let expected = disposition_identity(request, request.selected.as_slice());
    if expected != completed.plan_identity {
        return Err(Error::new(Issue::Conflict));
    }
    debug_assert_eq!(expected, completed.plan_identity);
    Ok(DispositionDecision::Repeated(completed.clone()))
}

fn validate_completed_selection(completed: &CompletedDisposition) -> Result<(), Error> {
    let selected_count =
        u32::try_from(completed.selected.len()).map_err(|_| Error::new(Issue::EntryLimit))?;
    let admission_count = u32::try_from(completed.admission_requests.len())
        .map_err(|_| Error::new(Issue::EntryLimit))?;
    if completed.selected.is_empty()
        || selected_count > super::data::MAX_ENTRY_COUNT
        || admission_count > super::data::MAX_ENTRY_COUNT
    {
        return Err(Error::new(Issue::Conflict));
    }
    let mut selected = std::collections::BTreeSet::new();
    for entry in &completed.selected {
        validate_metadata(entry.as_str())?;
        if !selected.insert(entry.as_str()) {
            return Err(Error::new(Issue::Conflict));
        }
    }
    debug_assert!(!selected.is_empty());
    debug_assert!(selected_count <= super::data::MAX_ENTRY_COUNT);
    match completed.kind {
        DispositionKind::Restore => {
            if !completed.restore_material_retained {
                return Err(Error::new(Issue::Conflict));
            }
            validate_admission_requests(completed)
        }
        DispositionKind::Discard => {
            if completed.restore_material_retained || !completed.admission_requests.is_empty() {
                return Err(Error::new(Issue::Conflict));
            }
            Ok(())
        }
    }
}

fn validate_admission_requests(completed: &CompletedDisposition) -> Result<(), Error> {
    if completed.admission_requests.len() != completed.selected.len() {
        return Err(Error::new(Issue::Conflict));
    }
    for (request, selected) in completed
        .admission_requests
        .iter()
        .zip(completed.selected.iter())
    {
        if request.entry != *selected
            || request.conversation != completed.boundary.conversation
            || request.generation != completed.boundary.generation
        {
            return Err(Error::new(Issue::Conflict));
        }
        validate_blake3(request.body_identity.as_str())?;
    }
    Ok(())
}

fn admission_requests(
    projection: &Projection,
    plan: &DispositionPlan,
) -> Result<Vec<AdmissionRequest>, Error> {
    if plan.kind != DispositionKind::Restore {
        return Ok(Vec::new());
    }
    let mut requests = Vec::with_capacity(plan.selected.len());
    for selected in &plan.selected {
        let entry = projection
            .entries
            .iter()
            .find(|candidate| candidate.id == *selected)
            .ok_or_else(|| Error::new(Issue::Conflict))?;
        requests.push(AdmissionRequest {
            entry: entry.id.clone(),
            conversation: entry.conversation.clone(),
            generation: entry.generation,
            class: entry.class,
            body_identity: entry.body_identity.clone(),
        });
    }
    debug_assert_eq!(requests.len(), plan.selected.len());
    debug_assert!(
        requests
            .iter()
            .all(|request| request.generation == plan.expected_generation)
    );
    Ok(requests)
}

fn projection_identity(
    policy: &Policy,
    facts: &StartupFacts,
    state: &RecoveryState,
) -> PlanIdentity {
    let mut hasher = blake3::Hasher::new();
    update_hash(&mut hasher, PROJECTION_DOMAIN);
    hash_boundary(&mut hasher, &facts.boundary);
    update_hash(&mut hasher, state.as_str());
    if let RecoveryState::BlockedCorrupt(class) = state {
        update_hash(&mut hasher, class.as_str());
    }
    update_hash(&mut hasher, facts.entries.len().to_string().as_str());
    let bounds = hash_bounds(policy);
    for entry in facts.entries.iter().take(bounds.entries) {
        hash_observed_entry(&mut hasher, entry, bounds);
    }
    if let Some(value) = facts.completed.as_ref() {
        hash_observed_completion(&mut hasher, value, bounds);
    }
    PlanIdentity::new(hasher.finalize().to_hex().to_string())
}

fn disposition_identity(request: &DispositionRequest, selected: &[EntryIdentity]) -> PlanIdentity {
    hash_disposition(DispositionHashFacts {
        id: &request.id,
        projection: &request.projection,
        boundary: &request.boundary,
        kind: request.kind,
        selected,
    })
}

fn hash_disposition(input: DispositionHashFacts<'_>) -> PlanIdentity {
    let mut hasher = blake3::Hasher::new();
    update_hash(&mut hasher, DISPOSITION_DOMAIN);
    update_hash(&mut hasher, input.id.as_str());
    update_hash(&mut hasher, input.projection.as_str());
    hash_boundary(&mut hasher, input.boundary);
    update_hash(&mut hasher, input.kind.as_str());
    for entry in input.selected {
        update_hash(&mut hasher, entry.as_str());
    }
    PlanIdentity::new(hasher.finalize().to_hex().to_string())
}

fn hash_boundary(hasher: &mut blake3::Hasher, boundary: &Boundary) {
    for value in [
        boundary.id.as_str(),
        boundary.conversation.as_str(),
        boundary.branch.as_str(),
        boundary.store.as_str(),
        boundary.authority.as_str(),
        boundary.policy.as_str(),
    ] {
        update_hash(hasher, value);
    }
    update_hash(hasher, boundary.generation.value().to_string().as_str());
}

fn hash_observed_entry(hasher: &mut blake3::Hasher, entry: &Entry, bounds: HashBounds) {
    debug_assert!(bounds.entries > 0);
    debug_assert!(bounds.attachments <= BLAKE3_HEX_LENGTH_USIZE);
    update_observed_hash(hasher, entry.id.as_str(), MAX_METADATA_BYTES_USIZE);
    update_observed_hash(
        hasher,
        entry.conversation.as_str(),
        MAX_METADATA_BYTES_USIZE,
    );
    update_hash(hasher, entry.class.as_str());
    update_observed_hash(
        hasher,
        entry.body_identity.as_str(),
        BLAKE3_HEX_LENGTH_USIZE,
    );
    update_observed_hash(hasher, entry.provenance.as_str(), MAX_METADATA_BYTES_USIZE);
    update_hash(hasher, entry.generation.value().to_string().as_str());
    update_hash(hasher, entry.sequence.value().to_string().as_str());
    update_hash(hasher, entry.body_bytes.value().to_string().as_str());
    update_hash(hasher, entry.attachments.len().to_string().as_str());
    for attachment in entry.attachments.iter().take(bounds.attachments) {
        update_observed_hash(hasher, attachment.as_str(), BLAKE3_HEX_LENGTH_USIZE);
    }
}

fn hash_observed_completion(
    hasher: &mut blake3::Hasher,
    completed: &CompletedDisposition,
    bounds: HashBounds,
) {
    debug_assert!(bounds.entries > 0);
    debug_assert!(bounds.entries <= MAX_METADATA_BYTES_USIZE);
    update_observed_hash(hasher, completed.id.as_str(), MAX_METADATA_BYTES_USIZE);
    update_observed_hash(
        hasher,
        completed.plan_identity.as_str(),
        BLAKE3_HEX_LENGTH_USIZE,
    );
    update_observed_hash(
        hasher,
        completed.projection.as_str(),
        BLAKE3_HEX_LENGTH_USIZE,
    );
    hash_observed_boundary(hasher, &completed.boundary);
    update_hash(hasher, completed.kind.as_str());
    update_hash(hasher, completed.cleanup.as_str());
    update_hash(
        hasher,
        if completed.restore_material_retained {
            "restore-material-retained"
        } else {
            "restore-material-not-retained"
        },
    );
    update_hash(hasher, completed.selected.len().to_string().as_str());
    for selected in completed.selected.iter().take(bounds.entries) {
        update_observed_hash(hasher, selected.as_str(), MAX_METADATA_BYTES_USIZE);
    }
    update_hash(
        hasher,
        completed.admission_requests.len().to_string().as_str(),
    );
    for request in completed.admission_requests.iter().take(bounds.entries) {
        update_observed_hash(hasher, request.entry.as_str(), MAX_METADATA_BYTES_USIZE);
        update_observed_hash(
            hasher,
            request.conversation.as_str(),
            MAX_METADATA_BYTES_USIZE,
        );
        update_hash(hasher, request.generation.value().to_string().as_str());
        update_hash(hasher, request.class.as_str());
        update_observed_hash(
            hasher,
            request.body_identity.as_str(),
            BLAKE3_HEX_LENGTH_USIZE,
        );
    }
}

fn hash_observed_boundary(hasher: &mut blake3::Hasher, boundary: &Boundary) {
    for value in [
        boundary.id.as_str(),
        boundary.conversation.as_str(),
        boundary.branch.as_str(),
        boundary.store.as_str(),
        boundary.authority.as_str(),
        boundary.policy.as_str(),
    ] {
        update_observed_hash(hasher, value, MAX_METADATA_BYTES_USIZE);
    }
    update_hash(hasher, boundary.generation.value().to_string().as_str());
}

fn update_observed_hash(hasher: &mut blake3::Hasher, value: &str, limit: usize) {
    update_hash(hasher, value.len().to_string().as_str());
    let bytes = value.as_bytes();
    let observed_length_bytes = bytes.len().min(limit);
    hasher.update(&bytes[..observed_length_bytes]);
    hasher.update(HASH_SEPARATOR);
}

fn hash_bounds(policy: &Policy) -> HashBounds {
    HashBounds {
        entries: usize::try_from(policy.entries).unwrap_or(MAX_METADATA_BYTES_USIZE),
        attachments: usize::try_from(policy.attachments_per_entry)
            .unwrap_or(BLAKE3_HEX_LENGTH_USIZE)
            .saturating_add(1),
    }
}

fn hash_control_fields(
    projection: &PlanIdentity,
    boundary: &Boundary,
    kind: super::data::RecoveryControlKind,
) -> PlanIdentity {
    let mut hasher = blake3::Hasher::new();
    update_hash(&mut hasher, CONTROL_DOMAIN);
    update_hash(&mut hasher, projection.as_str());
    hash_boundary(&mut hasher, boundary);
    update_hash(&mut hasher, kind.label());
    PlanIdentity::new(hasher.finalize().to_hex().to_string())
}

fn update_hash(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update(HASH_SEPARATOR);
}

fn count_u32(length: usize) -> Result<u32, Error> {
    u32::try_from(length).map_err(|_| Error::new(Issue::EntryLimit))
}

trait RecoveryControlLabel {
    fn label(self) -> &'static str;
}

impl RecoveryControlLabel for super::data::RecoveryControlKind {
    fn label(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::QuarantineExport => "quarantine-export",
            Self::RepairImport => "repair-import",
            Self::DestructiveDiscard => "destructive-discard",
        }
    }
}
