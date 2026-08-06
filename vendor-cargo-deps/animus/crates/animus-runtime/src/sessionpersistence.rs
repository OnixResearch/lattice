//! Injected session-persistence adapter contracts.
//!
//! The functions in this module are deterministic state transformers over safe
//! records. Hosts own filesystem, database, encryption, locking, clock, daemon,
//! and recovery effects.

// Cairn adaptation compatibility module keeps DTOs, validators, and fixtures compact while the change is drained.
#![allow(missing_docs)]
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports,
    path_segment_repetition,
    underscore_in_module_filename,
    dead_code
)]
#![allow(numeric_units, raw_arithmetic_overflow, unbounded_collection_growth)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

const HASH_DOMAIN: &str = "animus.session-persistence.v1";
const HASH_SEPARATOR: &str = "|";
const SUPPORTED_SCHEMA_VERSION: u16 = 1;
const EMPTY_COUNT: u32 = 0;
const MIN_COUNT: u32 = 1;
const MIN_FRESHNESS_SEQUENCE: u64 = 1;
const MAX_LABEL_CHARS: usize = 96;
const HASH_HEX_CHARS: usize = 64;
const GLOBAL_STORE_LABEL: &str = "global-default";
const UNSAFE_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "RAW_MODEL_OUTPUT:",
    "RAW_TOOL_BODY:",
    "PROVIDER_PAYLOAD:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "sk-",
    "/home/",
    "UNBOUNDED_LOG:",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceServiceState {
    Available,
    Missing,
    Unavailable,
    PartialWrite,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionStoreDescriptor {
    pub store_label: String,
    pub schema_version: u16,
    pub service_state: PersistenceServiceState,
    pub global_default: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistenceAdmission {
    pub store: Option<SessionStoreDescriptor>,
    pub session_id: String,
    pub branch_id: String,
    pub redaction_policy: String,
    pub receipt_policy: String,
    pub minimum_snapshot_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeSessionRecord {
    pub schema_version: u16,
    pub session_id: String,
    pub branch_id: String,
    pub parent_record_hash: Option<String>,
    pub record_hash: String,
    pub redaction_class: String,
    pub event_count: u32,
    pub token_count: u32,
    pub status_class: SessionRecordStatus,
    pub adapter_label: String,
    pub replay_sequence: u64,
    pub leak_marker: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionRecordStatus {
    Active,
    Snapshot,
    Restored,
    Compacted,
    Tombstoned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeSessionState {
    pub records: Vec<SafeSessionRecord>,
    pub tombstoned: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplaySummary {
    pub session_id: String,
    pub branch_id: String,
    pub record_count: usize,
    pub terminal_status: SessionRecordStatus,
    pub replay_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactionSummary {
    pub session_id: String,
    pub branch_id: String,
    pub input_records: usize,
    pub summary_hash: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceIssue {
    MissingStore,
    UnavailableBackend,
    PartialWrite,
    GlobalStoreDefault,
    SchemaMismatch,
    UnsafeRecord,
    LeakRejected,
    ReplayGap,
    RestoreConflict,
    TombstoneConflict,
    StaleSnapshot,
    EmptyField,
    InvalidHash,
}

impl PersistenceIssue {
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingStore => "missing-store",
            Self::UnavailableBackend => "unavailable-backend",
            Self::PartialWrite => "partial-write",
            Self::GlobalStoreDefault => "global-store-default",
            Self::SchemaMismatch => "schema-mismatch",
            Self::UnsafeRecord => "unsafe-record",
            Self::LeakRejected => "leak-rejected",
            Self::ReplayGap => "replay-gap",
            Self::RestoreConflict => "restore-conflict",
            Self::TombstoneConflict => "tombstone-conflict",
            Self::StaleSnapshot => "stale-snapshot",
            Self::EmptyField => "empty-field",
            Self::InvalidHash => "invalid-hash",
        }
    }
}

pub type PersistenceResult<T> = Result<T, Vec<PersistenceIssue>>;

pub fn admit_persistence(admission: &PersistenceAdmission) -> PersistenceResult<()> {
    let mut issues = Vec::new();
    validate_label(admission.session_id.as_str(), &mut issues);
    validate_label(admission.branch_id.as_str(), &mut issues);
    validate_label(admission.redaction_policy.as_str(), &mut issues);
    validate_label(admission.receipt_policy.as_str(), &mut issues);
    if admission.minimum_snapshot_sequence < MIN_FRESHNESS_SEQUENCE {
        issues.push(PersistenceIssue::StaleSnapshot);
    }
    match &admission.store {
        None => issues.push(PersistenceIssue::MissingStore),
        Some(store) => validate_store(store, &mut issues),
    }
    result_from_issues(issues, ())
}

pub fn validate_session_record(record: &SafeSessionRecord) -> PersistenceResult<()> {
    let mut issues = Vec::new();
    if record.schema_version != SUPPORTED_SCHEMA_VERSION {
        issues.push(PersistenceIssue::SchemaMismatch);
    }
    validate_label(record.session_id.as_str(), &mut issues);
    validate_label(record.branch_id.as_str(), &mut issues);
    validate_label(record.redaction_class.as_str(), &mut issues);
    validate_label(record.adapter_label.as_str(), &mut issues);
    validate_hash(record.record_hash.as_str(), &mut issues);
    if let Some(parent_hash) = &record.parent_record_hash {
        validate_hash(parent_hash.as_str(), &mut issues);
    }
    if record.event_count == EMPTY_COUNT {
        issues.push(PersistenceIssue::UnsafeRecord);
    }
    if let Some(marker) = &record.leak_marker {
        if UNSAFE_MARKERS.iter().any(|unsafe_marker| marker.contains(unsafe_marker)) {
            issues.push(PersistenceIssue::LeakRejected);
        }
    }
    result_from_issues(issues, ())
}

pub fn append_fake_record(
    admission: &PersistenceAdmission,
    state: &FakeSessionState,
    record: SafeSessionRecord,
) -> PersistenceResult<FakeSessionState> {
    admit_persistence(admission)?;
    validate_session_record(&record)?;
    if state.tombstoned {
        return Err(vec![PersistenceIssue::TombstoneConflict]);
    }
    if matches_store_state(admission, PersistenceServiceState::PartialWrite) {
        return Err(vec![PersistenceIssue::PartialWrite]);
    }
    let mut records = state.records.clone();
    records.push(record);
    Ok(FakeSessionState {
        records,
        tombstoned: false,
    })
}

pub fn replay_fake_session(
    admission: &PersistenceAdmission,
    state: &FakeSessionState,
) -> PersistenceResult<ReplaySummary> {
    admit_persistence(admission)?;
    let filtered = records_for_scope(admission, state)?;
    validate_replay_sequence(&filtered)?;
    let terminal_status = filtered.last().map_or(SessionRecordStatus::Active, |record| record.status_class);
    Ok(ReplaySummary {
        session_id: admission.session_id.clone(),
        branch_id: admission.branch_id.clone(),
        record_count: filtered.len(),
        terminal_status,
        replay_hash: state_hash(&filtered),
    })
}

pub fn restore_fake_snapshot(
    admission: &PersistenceAdmission,
    state: &FakeSessionState,
    snapshot_sequence: u64,
) -> PersistenceResult<ReplaySummary> {
    if snapshot_sequence < admission.minimum_snapshot_sequence {
        return Err(vec![PersistenceIssue::StaleSnapshot]);
    }
    let summary = replay_fake_session(admission, state)?;
    let has_snapshot = state.records.iter().any(|record| {
        record.session_id == admission.session_id
            && record.branch_id == admission.branch_id
            && record.replay_sequence == snapshot_sequence
            && record.status_class == SessionRecordStatus::Snapshot
    });
    if !has_snapshot {
        return Err(vec![PersistenceIssue::RestoreConflict]);
    }
    Ok(summary)
}

pub fn compact_fake_records(
    admission: &PersistenceAdmission,
    state: &FakeSessionState,
) -> PersistenceResult<CompactionSummary> {
    let filtered = records_for_scope(admission, state)?;
    Ok(CompactionSummary {
        session_id: admission.session_id.clone(),
        branch_id: admission.branch_id.clone(),
        input_records: filtered.len(),
        summary_hash: state_hash(&filtered),
    })
}

pub fn tombstone_fake_session(
    admission: &PersistenceAdmission,
    state: &FakeSessionState,
) -> PersistenceResult<FakeSessionState> {
    admit_persistence(admission)?;
    if state.tombstoned {
        return Err(vec![PersistenceIssue::TombstoneConflict]);
    }
    Ok(FakeSessionState {
        records: state.records.clone(),
        tombstoned: true,
    })
}

pub fn session_record_hash(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

const LEDGER_SESSION_HASH_DOMAIN: &str = "animus.ledger-session-adapter.v1";
const LEDGER_RECEIPT_SCHEMA_VERSION: u16 = 1;
const MIN_REPLAY_ENTRY_BUDGET: usize = 1;
const MIN_REPLAY_UNIT_BUDGET: u32 = 1;
const FIRST_LEDGER_SEQUENCE: u64 = 1;
const MAX_SAFE_SUMMARY_CHARS: usize = 160;
const NO_DEFAULT_STORE_NON_CLAIM: &str = "no-default-global-store";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LedgerSessionServiceStatus {
    Available,
    Missing,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LedgerSessionStoreKind {
    InMemory,
    File,
    Database,
    Remote,
    HostDefined,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerSessionStoreDescriptor {
    pub store_label: String,
    pub schema_version: u16,
    pub status: LedgerSessionServiceStatus,
    pub kind: LedgerSessionStoreKind,
    pub global_default: bool,
}

pub trait LedgerSessionService {
    fn descriptor(&self) -> &LedgerSessionStoreDescriptor;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionIdentity {
    pub session_label: String,
    pub session_hash: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LedgerFreshnessClass {
    Current,
    Stale,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionBranchMetadata {
    pub branch_label: String,
    pub branch_hash: String,
    pub parent_branch_hash: Option<String>,
    pub head_sequence: u64,
    pub freshness: LedgerFreshnessClass,
    pub conflicting: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LedgerEntryKind {
    UserMessage,
    AssistantMessage,
    ToolSummary,
    Summary,
    Snapshot,
    MutationRecord,
}

impl LedgerEntryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserMessage => "user_message",
            Self::AssistantMessage => "assistant_message",
            Self::ToolSummary => "tool_summary",
            Self::Summary => "summary",
            Self::Snapshot => "snapshot",
            Self::MutationRecord => "mutation_record",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerEntry {
    pub session_label: String,
    pub branch_label: String,
    pub sequence: u64,
    pub kind: LedgerEntryKind,
    pub safe_summary: String,
    pub summary_units: u32,
    pub redaction_class: String,
    pub freshness: LedgerFreshnessClass,
    pub safe_hash: String,
    pub private_reference: bool,
    pub unsafe_material: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerReplayRequest {
    pub store: Option<LedgerSessionStoreDescriptor>,
    pub identity: SessionIdentity,
    pub branch: SessionBranchMetadata,
    pub max_entries: usize,
    pub max_units: u32,
    pub redaction_policy: String,
    pub receipt_policy: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionRestoreRequest {
    pub replay: LedgerReplayRequest,
    pub target_sequence: u64,
    pub require_current_snapshot: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerMutationRecord {
    pub session_label: String,
    pub branch_label: String,
    pub sequence: u64,
    pub mutation_kind: String,
    pub safe_hash: String,
    pub redaction_class: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LedgerOmissionReason {
    Stale,
    OverBudget,
    PrivateReference,
    UnsafeMaterial,
}

impl LedgerOmissionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stale => "stale",
            Self::OverBudget => "over_budget",
            Self::PrivateReference => "private_reference",
            Self::UnsafeMaterial => "unsafe_material",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerOmittedEntry {
    pub sequence: u64,
    pub reason: LedgerOmissionReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerContextProjection {
    pub sequence: u64,
    pub kind: LedgerEntryKind,
    pub safe_summary: String,
    pub safe_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerSessionReceiptDraft {
    pub schema_version: u16,
    pub session_label: String,
    pub branch_label: String,
    pub sequence_start: u64,
    pub sequence_end: u64,
    pub included_count: usize,
    pub omitted_count: usize,
    pub redaction_class: String,
    pub freshness: LedgerFreshnessClass,
    pub safe_hash: String,
    pub non_claims: Vec<String>,
    pub leak_marker: Option<String>,
}

pub type LedgerSessionReceipt = LedgerSessionReceiptDraft;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerReplayPlan {
    pub session_label: String,
    pub branch_label: String,
    pub projections: Vec<LedgerContextProjection>,
    pub omitted: Vec<LedgerOmittedEntry>,
    pub receipt: LedgerSessionReceiptDraft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LedgerSessionIssue {
    MissingService,
    UnavailableService,
    AmbientFallbackDenied,
    SchemaMismatch,
    EmptyField,
    InvalidHash,
    StaleSnapshot,
    ConflictingBranch,
    MalformedSequenceRange,
    ReplayGap,
    UnsafeEntry,
    LeakRejected,
    MissingNonClaim,
}

impl LedgerSessionIssue {
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingService => "missing-service",
            Self::UnavailableService => "unavailable-service",
            Self::AmbientFallbackDenied => "ambient-fallback-denied",
            Self::SchemaMismatch => "schema-mismatch",
            Self::EmptyField => "empty-field",
            Self::InvalidHash => "invalid-hash",
            Self::StaleSnapshot => "stale-snapshot",
            Self::ConflictingBranch => "conflicting-branch",
            Self::MalformedSequenceRange => "malformed-sequence-range",
            Self::ReplayGap => "replay-gap",
            Self::UnsafeEntry => "unsafe-entry",
            Self::LeakRejected => "leak-rejected",
            Self::MissingNonClaim => "missing-non-claim",
        }
    }
}

pub type LedgerSessionResult<T> = Result<T, Vec<LedgerSessionIssue>>;

pub fn plan_ledger_replay(
    request: &LedgerReplayRequest,
    entries: &[LedgerEntry],
) -> LedgerSessionResult<LedgerReplayPlan> {
    validate_ledger_replay_request(request)?;
    let scoped = scoped_ledger_entries(request, entries)?;
    validate_ledger_sequence(&scoped)?;

    let mut projections = Vec::new();
    let mut omitted = Vec::new();
    let mut used_units = EMPTY_COUNT;
    for entry in scoped {
        if entry.freshness == LedgerFreshnessClass::Stale {
            omitted.push(omitted_entry(entry, LedgerOmissionReason::Stale));
            continue;
        }
        if entry.private_reference {
            omitted.push(omitted_entry(entry, LedgerOmissionReason::PrivateReference));
            continue;
        }
        if entry_has_unsafe_material(entry) {
            omitted.push(omitted_entry(entry, LedgerOmissionReason::UnsafeMaterial));
            continue;
        }
        let next_units = used_units.saturating_add(entry.summary_units);
        if projections.len() >= request.max_entries || next_units > request.max_units {
            omitted.push(omitted_entry(entry, LedgerOmissionReason::OverBudget));
            continue;
        }
        validate_ledger_entry(entry)?;
        used_units = next_units;
        projections.push(LedgerContextProjection {
            sequence: entry.sequence,
            kind: entry.kind,
            safe_summary: entry.safe_summary.clone(),
            safe_hash: entry.safe_hash.clone(),
        });
    }

    if projections.is_empty() {
        return Err(vec![LedgerSessionIssue::ReplayGap]);
    }

    let sequence_start = projections.first().map_or(FIRST_LEDGER_SEQUENCE, |projection| projection.sequence);
    let sequence_end = projections.last().map_or(sequence_start, |projection| projection.sequence);
    let receipt = LedgerSessionReceiptDraft {
        schema_version: LEDGER_RECEIPT_SCHEMA_VERSION,
        session_label: request.identity.session_label.clone(),
        branch_label: request.branch.branch_label.clone(),
        sequence_start,
        sequence_end,
        included_count: projections.len(),
        omitted_count: omitted.len(),
        redaction_class: request.redaction_policy.clone(),
        freshness: request.branch.freshness,
        safe_hash: ledger_session_hash("replay", projections_hash_input(&projections).as_bytes()),
        non_claims: vec![NO_DEFAULT_STORE_NON_CLAIM.to_owned()],
        leak_marker: None,
    };
    validate_ledger_session_receipt(&receipt)?;
    Ok(LedgerReplayPlan {
        session_label: request.identity.session_label.clone(),
        branch_label: request.branch.branch_label.clone(),
        projections,
        omitted,
        receipt,
    })
}

pub fn plan_session_restore(
    request: &SessionRestoreRequest,
    entries: &[LedgerEntry],
) -> LedgerSessionResult<LedgerReplayPlan> {
    if request.target_sequence < FIRST_LEDGER_SEQUENCE {
        return Err(vec![LedgerSessionIssue::MalformedSequenceRange]);
    }
    if request.replay.branch.conflicting {
        return Err(vec![LedgerSessionIssue::ConflictingBranch]);
    }
    if request.require_current_snapshot && request.replay.branch.freshness != LedgerFreshnessClass::Current {
        return Err(vec![LedgerSessionIssue::StaleSnapshot]);
    }
    let has_snapshot = entries.iter().any(|entry| {
        entry.session_label == request.replay.identity.session_label
            && entry.branch_label == request.replay.branch.branch_label
            && entry.sequence == request.target_sequence
            && entry.kind == LedgerEntryKind::Snapshot
            && entry.freshness == LedgerFreshnessClass::Current
    });
    if !has_snapshot {
        return Err(vec![LedgerSessionIssue::StaleSnapshot]);
    }
    plan_ledger_replay(&request.replay, entries)
}

pub fn validate_ledger_session_receipt(draft: &LedgerSessionReceiptDraft) -> LedgerSessionResult<LedgerSessionReceipt> {
    let mut issues = Vec::new();
    if draft.schema_version != LEDGER_RECEIPT_SCHEMA_VERSION {
        issues.push(LedgerSessionIssue::SchemaMismatch);
    }
    validate_ledger_label(draft.session_label.as_str(), &mut issues);
    validate_ledger_label(draft.branch_label.as_str(), &mut issues);
    validate_ledger_label(draft.redaction_class.as_str(), &mut issues);
    validate_ledger_hash(draft.safe_hash.as_str(), &mut issues);
    if draft.sequence_start < FIRST_LEDGER_SEQUENCE || draft.sequence_start > draft.sequence_end {
        issues.push(LedgerSessionIssue::MalformedSequenceRange);
    }
    if !draft.non_claims.iter().any(|claim| claim == NO_DEFAULT_STORE_NON_CLAIM) {
        issues.push(LedgerSessionIssue::MissingNonClaim);
    }
    for non_claim in &draft.non_claims {
        validate_ledger_label(non_claim.as_str(), &mut issues);
    }
    if draft.leak_marker.as_ref().is_some_and(|marker| contains_unsafe_marker(marker.as_str())) {
        issues.push(LedgerSessionIssue::LeakRejected);
    }
    ledger_result_from_issues(issues, draft.clone())
}

pub fn ledger_session_hash(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(LEDGER_SESSION_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn validate_ledger_replay_request(request: &LedgerReplayRequest) -> LedgerSessionResult<()> {
    let mut issues = Vec::new();
    validate_ledger_store(request.store.as_ref(), &mut issues);
    validate_ledger_identity(&request.identity, &mut issues);
    validate_ledger_branch(&request.branch, &mut issues);
    validate_ledger_label(request.redaction_policy.as_str(), &mut issues);
    validate_ledger_label(request.receipt_policy.as_str(), &mut issues);
    if request.max_entries < MIN_REPLAY_ENTRY_BUDGET || request.max_units < MIN_REPLAY_UNIT_BUDGET {
        issues.push(LedgerSessionIssue::ReplayGap);
    }
    ledger_result_from_issues(issues, ())
}

fn validate_ledger_store(store: Option<&LedgerSessionStoreDescriptor>, issues: &mut Vec<LedgerSessionIssue>) {
    match store {
        None => issues.push(LedgerSessionIssue::MissingService),
        Some(descriptor) => {
            validate_ledger_label(descriptor.store_label.as_str(), issues);
            if descriptor.schema_version != LEDGER_RECEIPT_SCHEMA_VERSION {
                issues.push(LedgerSessionIssue::SchemaMismatch);
            }
            match descriptor.status {
                LedgerSessionServiceStatus::Available => {}
                LedgerSessionServiceStatus::Missing => {
                    issues.push(LedgerSessionIssue::MissingService);
                }
                LedgerSessionServiceStatus::Unavailable => {
                    issues.push(LedgerSessionIssue::UnavailableService);
                }
            }
            if descriptor.global_default {
                issues.push(LedgerSessionIssue::AmbientFallbackDenied);
            }
        }
    }
}

fn validate_ledger_identity(identity: &SessionIdentity, issues: &mut Vec<LedgerSessionIssue>) {
    validate_ledger_label(identity.session_label.as_str(), issues);
    validate_ledger_hash(identity.session_hash.as_str(), issues);
}

fn validate_ledger_branch(branch: &SessionBranchMetadata, issues: &mut Vec<LedgerSessionIssue>) {
    validate_ledger_label(branch.branch_label.as_str(), issues);
    validate_ledger_hash(branch.branch_hash.as_str(), issues);
    if let Some(parent) = &branch.parent_branch_hash {
        validate_ledger_hash(parent.as_str(), issues);
    }
    if branch.head_sequence < FIRST_LEDGER_SEQUENCE {
        issues.push(LedgerSessionIssue::MalformedSequenceRange);
    }
    if branch.conflicting {
        issues.push(LedgerSessionIssue::ConflictingBranch);
    }
}

fn validate_ledger_entry(entry: &LedgerEntry) -> LedgerSessionResult<()> {
    let mut issues = Vec::new();
    validate_ledger_label(entry.session_label.as_str(), &mut issues);
    validate_ledger_label(entry.branch_label.as_str(), &mut issues);
    validate_ledger_label(entry.safe_summary.as_str(), &mut issues);
    validate_ledger_label(entry.redaction_class.as_str(), &mut issues);
    validate_ledger_hash(entry.safe_hash.as_str(), &mut issues);
    if entry.sequence < FIRST_LEDGER_SEQUENCE || entry.summary_units < MIN_REPLAY_UNIT_BUDGET {
        issues.push(LedgerSessionIssue::MalformedSequenceRange);
    }
    if entry.safe_summary.len() > MAX_SAFE_SUMMARY_CHARS {
        issues.push(LedgerSessionIssue::UnsafeEntry);
    }
    if entry.private_reference || entry_has_unsafe_material(entry) {
        issues.push(LedgerSessionIssue::UnsafeEntry);
    }
    ledger_result_from_issues(issues, ())
}

fn scoped_ledger_entries<'a>(
    request: &LedgerReplayRequest,
    entries: &'a [LedgerEntry],
) -> LedgerSessionResult<Vec<&'a LedgerEntry>> {
    let mut scoped = entries
        .iter()
        .filter(|entry| {
            entry.session_label == request.identity.session_label && entry.branch_label == request.branch.branch_label
        })
        .collect::<Vec<_>>();
    if scoped.is_empty() {
        return Err(vec![LedgerSessionIssue::ReplayGap]);
    }
    scoped.sort_by_key(|entry| entry.sequence);
    Ok(scoped)
}

fn validate_ledger_sequence(entries: &[&LedgerEntry]) -> LedgerSessionResult<()> {
    let mut expected = FIRST_LEDGER_SEQUENCE;
    for entry in entries {
        if entry.sequence != expected {
            return Err(vec![LedgerSessionIssue::MalformedSequenceRange]);
        }
        expected = expected.saturating_add(FIRST_LEDGER_SEQUENCE);
    }
    Ok(())
}

fn omitted_entry(entry: &LedgerEntry, reason: LedgerOmissionReason) -> LedgerOmittedEntry {
    LedgerOmittedEntry {
        sequence: entry.sequence,
        reason,
    }
}

fn entry_has_unsafe_material(entry: &LedgerEntry) -> bool {
    contains_unsafe_marker(entry.safe_summary.as_str())
        || entry.unsafe_material.as_ref().is_some_and(|value| contains_unsafe_marker(value.as_str()))
}

fn contains_unsafe_marker(value: &str) -> bool {
    UNSAFE_MARKERS.iter().any(|unsafe_marker| value.contains(unsafe_marker))
}

fn projections_hash_input(projections: &[LedgerContextProjection]) -> String {
    projections
        .iter()
        .map(|projection| projection.safe_hash.as_str())
        .collect::<Vec<_>>()
        .join(HASH_SEPARATOR)
}

fn validate_ledger_label(value: &str, issues: &mut Vec<LedgerSessionIssue>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LABEL_CHARS {
        issues.push(LedgerSessionIssue::EmptyField);
    }
    if contains_unsafe_marker(trimmed) {
        issues.push(LedgerSessionIssue::LeakRejected);
    }
}

fn validate_ledger_hash(value: &str, issues: &mut Vec<LedgerSessionIssue>) {
    if value.len() != HASH_HEX_CHARS || !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        issues.push(LedgerSessionIssue::InvalidHash);
    }
}

fn ledger_result_from_issues<T>(mut issues: Vec<LedgerSessionIssue>, value: T) -> LedgerSessionResult<T> {
    issues.sort_by_key(|issue| issue.code());
    issues.dedup();
    if issues.is_empty() { Ok(value) } else { Err(issues) }
}

fn validate_store(store: &SessionStoreDescriptor, issues: &mut Vec<PersistenceIssue>) {
    validate_label(store.store_label.as_str(), issues);
    if store.schema_version != SUPPORTED_SCHEMA_VERSION {
        issues.push(PersistenceIssue::SchemaMismatch);
    }
    match store.service_state {
        PersistenceServiceState::Available => {}
        PersistenceServiceState::Missing => issues.push(PersistenceIssue::MissingStore),
        PersistenceServiceState::Unavailable => issues.push(PersistenceIssue::UnavailableBackend),
        PersistenceServiceState::PartialWrite => issues.push(PersistenceIssue::PartialWrite),
    }
    if store.global_default || store.store_label == GLOBAL_STORE_LABEL {
        issues.push(PersistenceIssue::GlobalStoreDefault);
    }
}

fn matches_store_state(admission: &PersistenceAdmission, state: PersistenceServiceState) -> bool {
    admission.store.as_ref().is_some_and(|store| store.service_state == state)
}

fn records_for_scope<'a>(
    admission: &PersistenceAdmission,
    state: &'a FakeSessionState,
) -> PersistenceResult<Vec<&'a SafeSessionRecord>> {
    admit_persistence(admission)?;
    let records = state
        .records
        .iter()
        .filter(|record| record.session_id == admission.session_id && record.branch_id == admission.branch_id)
        .collect::<Vec<_>>();
    if records.is_empty() {
        return Err(vec![PersistenceIssue::ReplayGap]);
    }
    for record in &records {
        validate_session_record(record)?;
    }
    Ok(records)
}

fn validate_replay_sequence(records: &[&SafeSessionRecord]) -> PersistenceResult<()> {
    for (expected, record) in (MIN_FRESHNESS_SEQUENCE..).zip(records.iter()) {
        if record.replay_sequence != expected {
            return Err(vec![PersistenceIssue::ReplayGap]);
        }
    }
    Ok(())
}

fn state_hash(records: &[&SafeSessionRecord]) -> String {
    let canonical = records.iter().map(|record| record.record_hash.as_str()).collect::<Vec<_>>().join(HASH_SEPARATOR);
    session_record_hash("state", canonical.as_bytes())
}

fn validate_label(value: &str, issues: &mut Vec<PersistenceIssue>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LABEL_CHARS {
        issues.push(PersistenceIssue::EmptyField);
    }
    if UNSAFE_MARKERS.iter().any(|marker| trimmed.contains(marker)) {
        issues.push(PersistenceIssue::UnsafeRecord);
    }
}

fn validate_hash(value: &str, issues: &mut Vec<PersistenceIssue>) {
    if value.len() != HASH_HEX_CHARS || !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        issues.push(PersistenceIssue::InvalidHash);
    }
}

fn result_from_issues<T>(mut issues: Vec<PersistenceIssue>, value: T) -> PersistenceResult<T> {
    issues.sort_by_key(|issue| issue.code());
    issues.dedup();
    if issues.is_empty() { Ok(value) } else { Err(issues) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION_ID: &str = "session.a";
    const BRANCH_ID: &str = "branch.main";
    const REDACTION: &str = "safe-summary-only";
    const RECEIPT_POLICY: &str = "receipt-hash-only";
    const STORE_LABEL: &str = "in-memory-fake-store";
    const ADAPTER_LABEL: &str = "fake-model";
    const FIRST_RECORD_BYTES: &[u8] = b"first safe record";
    const SECOND_RECORD_BYTES: &[u8] = b"second safe record";
    const THIRD_RECORD_BYTES: &[u8] = b"third safe record";
    const EVENT_COUNT: u32 = 2;
    const TOKEN_COUNT: u32 = 5;
    const FIRST_SEQUENCE: u64 = 1;
    const SECOND_SEQUENCE: u64 = 2;
    const THIRD_SEQUENCE: u64 = 3;
    const FOURTH_SEQUENCE: u64 = 4;
    const FIFTH_SEQUENCE: u64 = 5;
    const LEDGER_MAX_REPLAY_ENTRIES: usize = 2;
    const LEDGER_MAX_REPLAY_UNITS: u32 = 4;
    const EXPECTED_LEDGER_OMITTED_COUNT: usize = 3;
    const EMPTY_OMITTED_COUNT: usize = 0;

    #[test]
    fn fake_store_appends_replays_restores_compacts_and_tombstones() {
        // r[verify agent_harness_runtime.session_persistence_adapter.service]
        // r[verify agent_harness_runtime.session_persistence_adapter.fake_store]
        let admission_request = admission(PersistenceServiceState::Available);
        let state = FakeSessionState {
            records: Vec::new(),
            tombstoned: false,
        };
        let first = record(FIRST_SEQUENCE, SessionRecordStatus::Active, FIRST_RECORD_BYTES, None);
        let second = record(
            SECOND_SEQUENCE,
            SessionRecordStatus::Snapshot,
            SECOND_RECORD_BYTES,
            Some(first.record_hash.clone()),
        );
        let third = record(
            THIRD_SEQUENCE,
            SessionRecordStatus::Compacted,
            THIRD_RECORD_BYTES,
            Some(second.record_hash.clone()),
        );

        let state = append_fake_record(&admission_request, &state, first).expect("first append");
        let state = append_fake_record(&admission_request, &state, second).expect("second append");
        let state = append_fake_record(&admission_request, &state, third).expect("third append");
        let replay = replay_fake_session(&admission_request, &state).expect("replay");
        let restored = restore_fake_snapshot(&admission_request, &state, SECOND_SEQUENCE).expect("restore");
        let compacted = compact_fake_records(&admission_request, &state).expect("compact");
        let tombstoned = tombstone_fake_session(&admission_request, &state).expect("tombstone");

        assert_eq!(replay.record_count, usize::try_from(THIRD_SEQUENCE).expect("fits"));
        assert_eq!(replay.terminal_status, SessionRecordStatus::Compacted);
        assert_eq!(restored.session_id, SESSION_ID);
        assert_eq!(compacted.input_records, replay.record_count);
        assert!(tombstoned.tombstoned);
    }

    #[test]
    fn admission_and_records_fail_closed_for_missing_store_and_unsafe_data() {
        // r[verify agent_harness_runtime.session_persistence_adapter.admission]
        // r[verify agent_harness_runtime.session_persistence_adapter.safe_records]
        let mut missing = admission(PersistenceServiceState::Available);
        missing.store = None;
        assert_eq!(admit_persistence(&missing), Err(vec![PersistenceIssue::MissingStore]));

        let global = PersistenceAdmission {
            store: Some(SessionStoreDescriptor {
                store_label: GLOBAL_STORE_LABEL.to_owned(),
                schema_version: SUPPORTED_SCHEMA_VERSION,
                service_state: PersistenceServiceState::Available,
                global_default: true,
            }),
            ..admission(PersistenceServiceState::Available)
        };
        assert!(
            admit_persistence(&global)
                .expect_err("global default")
                .contains(&PersistenceIssue::GlobalStoreDefault)
        );

        let unsafe_record = SafeSessionRecord {
            leak_marker: Some("RAW_PROMPT:secret".to_owned()),
            ..record(FIRST_SEQUENCE, SessionRecordStatus::Active, FIRST_RECORD_BYTES, None)
        };
        assert!(validate_session_record(&unsafe_record).expect_err("leak").contains(&PersistenceIssue::LeakRejected));
    }

    #[test]
    fn replay_restore_and_backend_failures_are_typed() {
        // r[verify agent_harness_runtime.session_persistence_adapter.failures]
        // r[verify agent_harness_runtime.session_persistence_adapter.verification]
        let admission_request = admission(PersistenceServiceState::Available);
        let gap_state = FakeSessionState {
            records: vec![record(
                SECOND_SEQUENCE,
                SessionRecordStatus::Active,
                SECOND_RECORD_BYTES,
                None,
            )],
            tombstoned: false,
        };
        assert_eq!(replay_fake_session(&admission_request, &gap_state), Err(vec![PersistenceIssue::ReplayGap]));
        let no_snapshot_state = FakeSessionState {
            records: vec![record(
                FIRST_SEQUENCE,
                SessionRecordStatus::Active,
                FIRST_RECORD_BYTES,
                None,
            )],
            tombstoned: false,
        };
        assert_eq!(
            restore_fake_snapshot(&admission_request, &no_snapshot_state, FIRST_SEQUENCE),
            Err(vec![PersistenceIssue::RestoreConflict])
        );

        let partial = admission(PersistenceServiceState::PartialWrite);
        let append_result = append_fake_record(
            &partial,
            &FakeSessionState {
                records: Vec::new(),
                tombstoned: false,
            },
            record(FIRST_SEQUENCE, SessionRecordStatus::Active, FIRST_RECORD_BYTES, None),
        );
        assert!(append_result.expect_err("partial write").contains(&PersistenceIssue::PartialWrite));

        let stale = PersistenceAdmission {
            minimum_snapshot_sequence: EMPTY_COUNT.into(),
            ..admission(PersistenceServiceState::Available)
        };
        assert!(admit_persistence(&stale).expect_err("stale").contains(&PersistenceIssue::StaleSnapshot));
    }

    #[test]
    fn ledger_session_replay_restore_and_receipts_are_pure_and_safe() {
        // r[verify agent_harness_runtime.ledger_session_adapter.service]
        // r[verify agent_harness_runtime.ledger_session_adapter.replay]
        // r[verify agent_harness_runtime.ledger_session_adapter.receipts]
        let request = ledger_request(LedgerSessionServiceStatus::Available);
        let entries = ledger_entries();

        let replay = plan_ledger_replay(&request, &entries).expect("replay plan");
        let restore = plan_session_restore(
            &SessionRestoreRequest {
                replay: request.clone(),
                target_sequence: SECOND_SEQUENCE,
                require_current_snapshot: true,
            },
            &entries,
        )
        .expect("restore plan");
        let receipt = validate_ledger_session_receipt(&replay.receipt).expect("safe receipt");

        assert_eq!(replay.projections.len(), LEDGER_MAX_REPLAY_ENTRIES);
        assert_eq!(replay.omitted.len(), EXPECTED_LEDGER_OMITTED_COUNT);
        assert_eq!(restore.branch_label, BRANCH_ID);
        assert_eq!(receipt.included_count, replay.projections.len());
        assert!(replay.omitted.iter().any(|entry| entry.reason == LedgerOmissionReason::Stale));
        assert!(replay.omitted.iter().any(|entry| entry.reason == LedgerOmissionReason::PrivateReference));
        assert!(replay.omitted.iter().any(|entry| entry.reason == LedgerOmissionReason::UnsafeMaterial));
    }

    #[test]
    fn ledger_session_failures_reject_missing_services_and_unsafe_receipts() {
        // r[verify agent_harness_runtime.ledger_session_adapter.service]
        // r[verify agent_harness_runtime.ledger_session_adapter.verification]
        let entries = ledger_entries();
        let mut missing = ledger_request(LedgerSessionServiceStatus::Available);
        missing.store = None;
        assert_eq!(plan_ledger_replay(&missing, &entries), Err(vec![LedgerSessionIssue::MissingService]));

        let mut ambient = ledger_request(LedgerSessionServiceStatus::Available);
        ambient.store = Some(LedgerSessionStoreDescriptor {
            global_default: true,
            ..ledger_store(LedgerSessionServiceStatus::Available)
        });
        assert!(
            plan_ledger_replay(&ambient, &entries)
                .expect_err("ambient fallback denied")
                .contains(&LedgerSessionIssue::AmbientFallbackDenied)
        );

        let mut malformed_entries = ledger_entries();
        malformed_entries.remove(1);
        assert_eq!(
            plan_ledger_replay(&ledger_request(LedgerSessionServiceStatus::Available), &malformed_entries),
            Err(vec![LedgerSessionIssue::MalformedSequenceRange])
        );

        let unsafe_receipt = LedgerSessionReceiptDraft {
            leak_marker: Some("RAW_PROMPT: secret".to_owned()),
            ..safe_ledger_receipt()
        };
        assert!(
            validate_ledger_session_receipt(&unsafe_receipt)
                .expect_err("receipt leak")
                .contains(&LedgerSessionIssue::LeakRejected)
        );
    }

    #[test]
    fn ledger_session_restore_rejects_stale_and_conflicting_branches() {
        // r[verify agent_harness_runtime.ledger_session_adapter.replay]
        // r[verify agent_harness_runtime.ledger_session_adapter.verification]
        let entries = ledger_entries();
        let stale = SessionRestoreRequest {
            replay: LedgerReplayRequest {
                branch: SessionBranchMetadata {
                    freshness: LedgerFreshnessClass::Stale,
                    ..ledger_branch()
                },
                ..ledger_request(LedgerSessionServiceStatus::Available)
            },
            target_sequence: SECOND_SEQUENCE,
            require_current_snapshot: true,
        };
        assert_eq!(plan_session_restore(&stale, &entries), Err(vec![LedgerSessionIssue::StaleSnapshot]));

        let conflicting = SessionRestoreRequest {
            replay: LedgerReplayRequest {
                branch: SessionBranchMetadata {
                    conflicting: true,
                    ..ledger_branch()
                },
                ..ledger_request(LedgerSessionServiceStatus::Available)
            },
            target_sequence: SECOND_SEQUENCE,
            require_current_snapshot: true,
        };
        assert_eq!(plan_session_restore(&conflicting, &entries), Err(vec![LedgerSessionIssue::ConflictingBranch]));
    }

    fn ledger_request(status: LedgerSessionServiceStatus) -> LedgerReplayRequest {
        LedgerReplayRequest {
            store: Some(ledger_store(status)),
            identity: SessionIdentity {
                session_label: SESSION_ID.to_owned(),
                session_hash: ledger_session_hash("session", SESSION_ID.as_bytes()),
            },
            branch: ledger_branch(),
            max_entries: LEDGER_MAX_REPLAY_ENTRIES,
            max_units: LEDGER_MAX_REPLAY_UNITS,
            redaction_policy: REDACTION.to_owned(),
            receipt_policy: RECEIPT_POLICY.to_owned(),
        }
    }

    fn ledger_store(status: LedgerSessionServiceStatus) -> LedgerSessionStoreDescriptor {
        LedgerSessionStoreDescriptor {
            store_label: STORE_LABEL.to_owned(),
            schema_version: LEDGER_RECEIPT_SCHEMA_VERSION,
            status,
            kind: LedgerSessionStoreKind::InMemory,
            global_default: false,
        }
    }

    fn ledger_branch() -> SessionBranchMetadata {
        SessionBranchMetadata {
            branch_label: BRANCH_ID.to_owned(),
            branch_hash: ledger_session_hash("branch", BRANCH_ID.as_bytes()),
            parent_branch_hash: None,
            head_sequence: FIFTH_SEQUENCE,
            freshness: LedgerFreshnessClass::Current,
            conflicting: false,
        }
    }

    fn ledger_entries() -> Vec<LedgerEntry> {
        vec![
            ledger_entry(LedgerEntryFixture {
                sequence: FIRST_SEQUENCE,
                kind: LedgerEntryKind::UserMessage,
                summary: "safe user summary",
                freshness: LedgerFreshnessClass::Current,
                private_reference: false,
                unsafe_material: None,
            }),
            ledger_entry(LedgerEntryFixture {
                sequence: SECOND_SEQUENCE,
                kind: LedgerEntryKind::Snapshot,
                summary: "safe snapshot summary",
                freshness: LedgerFreshnessClass::Current,
                private_reference: false,
                unsafe_material: None,
            }),
            ledger_entry(LedgerEntryFixture {
                sequence: THIRD_SEQUENCE,
                kind: LedgerEntryKind::Summary,
                summary: "stale summary",
                freshness: LedgerFreshnessClass::Stale,
                private_reference: false,
                unsafe_material: None,
            }),
            ledger_entry(LedgerEntryFixture {
                sequence: FOURTH_SEQUENCE,
                kind: LedgerEntryKind::ToolSummary,
                summary: "private reference omitted",
                freshness: LedgerFreshnessClass::Current,
                private_reference: true,
                unsafe_material: None,
            }),
            ledger_entry(LedgerEntryFixture {
                sequence: FIFTH_SEQUENCE,
                kind: LedgerEntryKind::AssistantMessage,
                summary: "unsafe material omitted",
                freshness: LedgerFreshnessClass::Current,
                private_reference: false,
                unsafe_material: Some("RAW_TOOL_BODY: unsafe"),
            }),
        ]
    }

    #[derive(Clone, Copy)]
    struct LedgerEntryFixture<'a> {
        sequence: u64,
        kind: LedgerEntryKind,
        summary: &'a str,
        freshness: LedgerFreshnessClass,
        private_reference: bool,
        unsafe_material: Option<&'a str>,
    }

    fn ledger_entry(fixture: LedgerEntryFixture<'_>) -> LedgerEntry {
        LedgerEntry {
            session_label: SESSION_ID.to_owned(),
            branch_label: BRANCH_ID.to_owned(),
            sequence: fixture.sequence,
            kind: fixture.kind,
            safe_summary: fixture.summary.to_owned(),
            summary_units: MIN_REPLAY_UNIT_BUDGET,
            redaction_class: REDACTION.to_owned(),
            freshness: fixture.freshness,
            safe_hash: ledger_session_hash("entry", fixture.summary.as_bytes()),
            private_reference: fixture.private_reference,
            unsafe_material: fixture.unsafe_material.map(str::to_owned),
        }
    }

    fn safe_ledger_receipt() -> LedgerSessionReceiptDraft {
        LedgerSessionReceiptDraft {
            schema_version: LEDGER_RECEIPT_SCHEMA_VERSION,
            session_label: SESSION_ID.to_owned(),
            branch_label: BRANCH_ID.to_owned(),
            sequence_start: FIRST_SEQUENCE,
            sequence_end: SECOND_SEQUENCE,
            included_count: LEDGER_MAX_REPLAY_ENTRIES,
            omitted_count: EMPTY_OMITTED_COUNT,
            redaction_class: REDACTION.to_owned(),
            freshness: LedgerFreshnessClass::Current,
            safe_hash: ledger_session_hash("receipt", b"safe receipt"),
            non_claims: vec![NO_DEFAULT_STORE_NON_CLAIM.to_owned()],
            leak_marker: None,
        }
    }

    fn admission(state: PersistenceServiceState) -> PersistenceAdmission {
        PersistenceAdmission {
            store: Some(SessionStoreDescriptor {
                store_label: STORE_LABEL.to_owned(),
                schema_version: SUPPORTED_SCHEMA_VERSION,
                service_state: state,
                global_default: false,
            }),
            session_id: SESSION_ID.to_owned(),
            branch_id: BRANCH_ID.to_owned(),
            redaction_policy: REDACTION.to_owned(),
            receipt_policy: RECEIPT_POLICY.to_owned(),
            minimum_snapshot_sequence: FIRST_SEQUENCE,
        }
    }

    fn record(
        sequence: u64,
        status: SessionRecordStatus,
        bytes: &[u8],
        parent_record_hash: Option<String>,
    ) -> SafeSessionRecord {
        SafeSessionRecord {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            session_id: SESSION_ID.to_owned(),
            branch_id: BRANCH_ID.to_owned(),
            parent_record_hash,
            record_hash: session_record_hash("record", bytes),
            redaction_class: REDACTION.to_owned(),
            event_count: EVENT_COUNT,
            token_count: TOKEN_COUNT,
            status_class: status,
            adapter_label: ADAPTER_LABEL.to_owned(),
            replay_sequence: sequence,
            leak_marker: None,
        }
    }
}
