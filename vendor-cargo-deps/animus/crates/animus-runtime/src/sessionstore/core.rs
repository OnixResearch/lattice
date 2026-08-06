//! Pure durable-session policy, framing, transaction, and sidecar planning.

type DurableSessionCommit = super::data::DurableSessionCommit;
type DurableSessionEncryptionPosture = super::data::DurableSessionEncryptionPosture;
type DurableSessionFrame = super::data::DurableSessionFrame;
type DurableSessionSafeSidecar = super::data::DurableSessionSafeSidecar;
type DurableSessionStoreDescriptor = super::data::DurableSessionStoreDescriptor;
type DurableSessionStoreError = super::data::DurableSessionStoreError;
type DurableSessionStoreIssue = super::data::DurableSessionStoreIssue;
type DurableSessionStorePolicy = super::data::DurableSessionStorePolicy;
type DurableSessionStorePolicyInput = super::data::DurableSessionStorePolicyInput;
type DurableSessionTransaction = super::data::DurableSessionTransaction;
type DurableSessionTransactionDraft<'a> = super::data::DurableSessionTransactionDraft<'a>;

const BLAKE3_HEX_CHARS: usize = super::data::BLAKE3_HEX_CHARS;
const MAX_FILE_BYTES: u64 = super::data::MAX_FILE_BYTES;
const MAX_FRAME_PAYLOAD_BYTES: u32 = super::data::MAX_FRAME_PAYLOAD_BYTES;
const MAX_TRANSACTION_FRAMES: u32 = super::data::MAX_TRANSACTION_FRAMES;
const MIN_COUNT: u32 = super::data::MIN_COUNT;
const PRIVATE_FILE_MODE: u32 = super::data::PRIVATE_FILE_MODE;
const SUPPORTED_FRAME_SCHEMA: u16 = super::data::SUPPORTED_FRAME_SCHEMA;
const POLICY_SCHEMA: &str = super::data::DURABLE_SESSION_STORE_POLICY_SCHEMA;
const POLICY_HASH_DOMAIN: &str = super::data::DURABLE_SESSION_STORE_POLICY_HASH_DOMAIN;
const FRAME_HASH_DOMAIN: &str = super::data::DURABLE_SESSION_FRAME_HASH_DOMAIN;
const TRANSACTION_HASH_DOMAIN: &str = super::data::DURABLE_SESSION_TRANSACTION_HASH_DOMAIN;
const COMMIT_HASH_DOMAIN: &str = super::data::DURABLE_SESSION_COMMIT_HASH_DOMAIN;
const SIDECAR_HASH_DOMAIN: &str = super::data::DURABLE_SESSION_SIDECAR_HASH_DOMAIN;
const REQUIRED_DURABILITY: &str = "sync_all_each_commit";
const REQUIRED_LOCKING: &str = "exclusive_create_new";
const REQUIRED_PRIVATE_DATA: &str = "replay_private";
const REQUIRED_TAIL_RECOVERY: &str = "ignore_uncommitted";
const REQUIRED_CORRUPTION: &str = "reject";
const REQUIRED_ENCRYPTION_POSTURES: &[&str] = &["absent_explicit", "external_required"];
const REQUIRED_RECEIPT_FIELDS: &[&str] = &[
    "store_identity",
    "session_identity",
    "branch_identity",
    "head_identity",
    "operation",
    "durability",
    "counts",
    "lock_status",
    "recovery_status",
    "redaction",
    "non_claim",
];

#[derive(Clone, Copy)]
pub(super) struct StoreField(pub(super) &'static str);

const FIELD_POLICY: StoreField = StoreField("durable_session_store_policy");
const FIELD_DESCRIPTOR: StoreField = StoreField("durable_session_store_descriptor");
const FIELD_FRAME: StoreField = StoreField("durable_session_frame");
const FIELD_TRANSACTION: StoreField = StoreField("durable_session_transaction");

/// Validates generated durable-session store policy and canonical identity.
///
/// # Errors
///
/// Returns [`DurableSessionStoreError`] when shape, bounds, postures, fields, or
/// canonical policy identity differ.
// r[impl agent_harness_runtime.durable_session_store.boundary]
// r[impl agent_harness_runtime.durable_session_store.commit]
pub fn validate_durable_session_store_policy(
    input: &DurableSessionStorePolicyInput,
) -> Result<DurableSessionStorePolicy, DurableSessionStoreError> {
    validate_policy_shape(input)?;
    let encryption_postures = input
        .encryption_postures
        .iter()
        .map(|label| {
            DurableSessionEncryptionPosture::parse(label.as_str())
                .ok_or_else(|| error(DurableSessionStoreIssue::InvalidPolicy, FIELD_POLICY))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected_hash = durable_session_store_policy_content_hash(input)?;
    if expected_hash != input.policy_hash {
        return Err(error(DurableSessionStoreIssue::StalePolicy, FIELD_POLICY));
    }
    let policy = DurableSessionStorePolicy {
        policy_id: input.policy_id.clone(),
        max_frame_payload_bytes: input.max_frame_payload_bytes,
        max_transaction_frames: input.max_transaction_frames,
        max_file_bytes: input.max_file_bytes,
        file_mode: input.file_mode,
        encryption_postures,
        receipt_fields: input.receipt_fields.clone(),
        policy_hash: input.policy_hash.clone(),
    };
    debug_assert_eq!(policy.file_mode, PRIVATE_FILE_MODE);
    debug_assert_eq!(policy.encryption_postures.len(), REQUIRED_ENCRYPTION_POSTURES.len());
    Ok(policy)
}

/// Computes canonical generated-policy identity.
///
/// # Errors
///
/// Returns [`DurableSessionStoreError`] if a canonical part length cannot be
/// represented by the fixed-width wire identity.
pub fn durable_session_store_policy_content_hash(
    input: &DurableSessionStorePolicyInput,
) -> Result<String, DurableSessionStoreError> {
    let frame_bytes = input.max_frame_payload_bytes.to_string();
    let transaction_frames = input.max_transaction_frames.to_string();
    let file_bytes = input.max_file_bytes.to_string();
    let file_mode = input.file_mode.to_string();
    let mut parts = vec![
        input.schema.as_bytes(),
        input.policy_id.as_bytes(),
        frame_bytes.as_bytes(),
        transaction_frames.as_bytes(),
        file_bytes.as_bytes(),
        file_mode.as_bytes(),
        input.durability.as_bytes(),
        input.locking.as_bytes(),
        input.private_data.as_bytes(),
        input.tail_recovery.as_bytes(),
        input.committed_corruption.as_bytes(),
    ];
    parts.extend(input.encryption_postures.iter().map(String::as_bytes));
    parts.extend(input.receipt_fields.iter().map(String::as_bytes));
    debug_assert!(!POLICY_HASH_DOMAIN.is_empty());
    let policy_hash = hash_parts(POLICY_HASH_DOMAIN, parts.as_slice())?;
    debug_assert_eq!(policy_hash.len(), BLAKE3_HEX_CHARS);
    Ok(policy_hash)
}

/// Validates an explicit non-ambient store descriptor.
///
/// # Errors
///
/// Returns [`DurableSessionStoreError`] when labels, policy binding, or
/// encryption admission are unsafe or unsupported.
// r[impl agent_harness_runtime.durable_session_store.boundary]
// r[impl agent_harness_runtime.durable_session_store.private_journal]
pub fn validate_durable_session_store_descriptor(
    policy: &DurableSessionStorePolicy,
    descriptor: &DurableSessionStoreDescriptor,
) -> Result<(), DurableSessionStoreError> {
    safe_label(descriptor.store_id.as_str(), FIELD_DESCRIPTOR)?;
    safe_label(descriptor.location_capability.as_str(), FIELD_DESCRIPTOR)?;
    safe_label(descriptor.session_id.as_str(), FIELD_DESCRIPTOR)?;
    safe_label(descriptor.branch_id.as_str(), FIELD_DESCRIPTOR)?;
    safe_label(descriptor.receipt_policy.as_str(), FIELD_DESCRIPTOR)?;
    lowercase_blake3(descriptor.policy_hash.as_str(), FIELD_DESCRIPTOR)?;
    if descriptor.policy_hash != policy.policy_hash {
        return Err(error(DurableSessionStoreIssue::InvalidDescriptor, FIELD_DESCRIPTOR));
    }
    if !policy.admits_encryption(descriptor.encryption) {
        return Err(error(DurableSessionStoreIssue::InvalidDescriptor, FIELD_DESCRIPTOR));
    }
    if descriptor.encryption == DurableSessionEncryptionPosture::ExternalRequired
        && !descriptor.encryption_service_available
    {
        return Err(error(DurableSessionStoreIssue::EncryptionUnavailable, FIELD_DESCRIPTOR));
    }
    debug_assert!(!descriptor.location_capability.is_empty());
    debug_assert_eq!(descriptor.policy_hash, policy.policy_hash);
    Ok(())
}

/// Plans one bounded private frame group and logical active-leaf commit.
///
/// # Errors
///
/// Returns [`DurableSessionStoreError`] for invalid descriptors, stale or
/// malformed identities, empty or over-budget groups, payload bounds, or
/// sequence overflow.
// r[impl agent_harness_runtime.durable_session_store.framing]
// r[impl agent_harness_runtime.durable_session_store.identity]
// r[impl agent_harness_runtime.durable_session_store.commit]
pub fn plan_durable_session_transaction(
    policy: &DurableSessionStorePolicy,
    draft: &DurableSessionTransactionDraft<'_>,
) -> Result<DurableSessionTransaction, DurableSessionStoreError> {
    validate_durable_session_store_descriptor(policy, draft.descriptor)?;
    lowercase_blake3(draft.prior_head_hash, FIELD_TRANSACTION)?;
    safe_label(draft.active_leaf_id, FIELD_TRANSACTION)?;
    let frame_count = u32::try_from(draft.entries.len())
        .map_err(|_error| error(DurableSessionStoreIssue::BoundExceeded, FIELD_TRANSACTION))?;
    if !(MIN_COUNT..=policy.max_transaction_frames).contains(&frame_count) {
        return Err(error(DurableSessionStoreIssue::BoundExceeded, FIELD_TRANSACTION));
    }
    validate_transaction_lineage(draft)?;
    if draft.first_sequence == 0 {
        return Err(error(DurableSessionStoreIssue::SequenceGap, FIELD_TRANSACTION));
    }
    let mut frames = Vec::with_capacity(draft.entries.len());
    let mut sidecars = Vec::with_capacity(draft.entries.len());
    let mut sequence = draft.first_sequence;
    let mut previous_hash = String::from(draft.prior_head_hash);
    for entry in draft.entries {
        let frame = build_frame(&FrameBuildInput {
            policy,
            descriptor: draft.descriptor,
            sequence,
            previous_frame_hash: previous_hash.as_str(),
            entry,
        })?;
        sidecars.push(project_sidecar(&frame)?);
        previous_hash.clone_from(&frame.frame_hash);
        frames.push(frame);
        sequence = sequence
            .checked_add(1)
            .ok_or_else(|| error(DurableSessionStoreIssue::SequenceGap, FIELD_TRANSACTION))?;
    }
    let commit = build_commit(&CommitBuildInput {
        descriptor: draft.descriptor,
        prior_head_hash: draft.prior_head_hash,
        active_leaf_id: draft.active_leaf_id,
        frames: frames.as_slice(),
    })?;
    debug_assert_eq!(frames.len(), draft.entries.len());
    debug_assert_eq!(sidecars.len(), frames.len());
    Ok(DurableSessionTransaction {
        frames,
        sidecars,
        commit,
    })
}

struct FrameBuildInput<'a> {
    policy: &'a DurableSessionStorePolicy,
    descriptor: &'a DurableSessionStoreDescriptor,
    sequence: u64,
    previous_frame_hash: &'a str,
    entry: &'a super::data::DurableSessionPrivateEntryDraft<'a>,
}

fn build_frame(input: &FrameBuildInput<'_>) -> Result<DurableSessionFrame, DurableSessionStoreError> {
    safe_label(input.entry.entry_id, FIELD_FRAME)?;
    if let Some(parent_id) = input.entry.parent_id {
        safe_label(parent_id, FIELD_FRAME)?;
    }
    lowercase_blake3(input.previous_frame_hash, FIELD_FRAME)?;
    let payload_bytes = u32::try_from(input.entry.payload.len())
        .map_err(|_error| error(DurableSessionStoreIssue::BoundExceeded, FIELD_FRAME))?;
    if !(MIN_COUNT..=input.policy.max_frame_payload_bytes).contains(&payload_bytes) {
        return Err(error(DurableSessionStoreIssue::BoundExceeded, FIELD_FRAME));
    }
    let payload_hash = blake3::hash(input.entry.payload).to_hex().to_string();
    let sequence = input.sequence.to_string();
    let kind = input.entry.kind.as_str();
    let parent = input.entry.parent_id.unwrap_or("");
    let frame_hash = hash_parts(FRAME_HASH_DOMAIN, &[
        input.descriptor.session_id.as_bytes(),
        input.descriptor.branch_id.as_bytes(),
        sequence.as_bytes(),
        input.entry.entry_id.as_bytes(),
        kind.as_bytes(),
        parent.as_bytes(),
        payload_bytes.to_string().as_bytes(),
        payload_hash.as_bytes(),
        input.previous_frame_hash.as_bytes(),
    ])?;
    debug_assert!(!payload_hash.is_empty());
    debug_assert_eq!(u32::try_from(input.entry.payload.len()), Ok(payload_bytes));
    Ok(DurableSessionFrame {
        schema_version: SUPPORTED_FRAME_SCHEMA,
        sequence: input.sequence,
        branch_id: input.descriptor.branch_id.clone(),
        entry_id: String::from(input.entry.entry_id),
        parent_id: input.entry.parent_id.map(String::from),
        kind: input.entry.kind,
        payload: input.entry.payload.to_vec(),
        payload_hash,
        previous_frame_hash: String::from(input.previous_frame_hash),
        frame_hash,
    })
}

struct CommitBuildInput<'a> {
    descriptor: &'a DurableSessionStoreDescriptor,
    prior_head_hash: &'a str,
    active_leaf_id: &'a str,
    frames: &'a [DurableSessionFrame],
}

fn build_commit(input: &CommitBuildInput<'_>) -> Result<DurableSessionCommit, DurableSessionStoreError> {
    debug_assert!(!input.frames.is_empty());
    debug_assert!(!input.active_leaf_id.is_empty());
    let frame_hashes = input.frames.iter().map(|frame| frame.frame_hash.clone()).collect::<Vec<_>>();
    let mut transaction_parts = vec![
        input.descriptor.session_id.as_bytes(),
        input.descriptor.branch_id.as_bytes(),
        input.prior_head_hash.as_bytes(),
        input.active_leaf_id.as_bytes(),
    ];
    transaction_parts.extend(frame_hashes.iter().map(String::as_bytes));
    let transaction_hash = hash_parts(TRANSACTION_HASH_DOMAIN, transaction_parts.as_slice())?;
    let commit_hash = hash_parts(COMMIT_HASH_DOMAIN, &[
        transaction_hash.as_bytes(),
        input.prior_head_hash.as_bytes(),
        input.active_leaf_id.as_bytes(),
    ])?;
    Ok(DurableSessionCommit {
        transaction_hash,
        prior_head_hash: String::from(input.prior_head_hash),
        active_leaf_id: String::from(input.active_leaf_id),
        frame_hashes,
        commit_hash,
    })
}

fn project_sidecar(frame: &DurableSessionFrame) -> Result<DurableSessionSafeSidecar, DurableSessionStoreError> {
    let payload_bytes = u32::try_from(frame.payload.len())
        .map_err(|_error| error(DurableSessionStoreIssue::BoundExceeded, FIELD_FRAME))?;
    let sequence = frame.sequence.to_string();
    let payload_bytes_text = payload_bytes.to_string();
    let sidecar_hash = hash_parts(SIDECAR_HASH_DOMAIN, &[
        sequence.as_bytes(),
        frame.entry_id.as_bytes(),
        frame.kind.as_str().as_bytes(),
        payload_bytes_text.as_bytes(),
        frame.payload_hash.as_bytes(),
        frame.frame_hash.as_bytes(),
        super::data::REDACTION_MARKER.as_bytes(),
    ])?;
    debug_assert!(payload_bytes > 0);
    debug_assert!(!sidecar_hash.is_empty());
    Ok(DurableSessionSafeSidecar {
        sequence: frame.sequence,
        entry_id: frame.entry_id.clone(),
        kind: frame.kind,
        payload_bytes,
        payload_hash: frame.payload_hash.clone(),
        frame_hash: frame.frame_hash.clone(),
        sidecar_hash,
        redaction_marker: String::from(super::data::REDACTION_MARKER),
    })
}

fn validate_transaction_lineage(draft: &DurableSessionTransactionDraft<'_>) -> Result<(), DurableSessionStoreError> {
    let is_empty_head = draft.prior_head_hash == super::data::DURABLE_SESSION_EMPTY_HEAD_HASH;
    if is_empty_head != draft.prior_active_leaf_id.is_none() {
        return Err(error(DurableSessionStoreIssue::StaleHead, FIELD_TRANSACTION));
    }
    if let Some(prior_active_leaf_id) = draft.prior_active_leaf_id {
        safe_label(prior_active_leaf_id, FIELD_TRANSACTION)?;
    }
    let mut expected_parent = draft.prior_active_leaf_id;
    let mut seen_entry_ids = Vec::with_capacity(draft.entries.len());
    for entry in draft.entries {
        safe_label(entry.entry_id, FIELD_TRANSACTION)?;
        if seen_entry_ids.iter().any(|entry_id| entry_id == &entry.entry_id) || entry.parent_id != expected_parent {
            return Err(error(DurableSessionStoreIssue::IntegrityMismatch, FIELD_TRANSACTION));
        }
        seen_entry_ids.push(entry.entry_id);
        expected_parent = Some(entry.entry_id);
    }
    if expected_parent != Some(draft.active_leaf_id) {
        return Err(error(DurableSessionStoreIssue::IntegrityMismatch, FIELD_TRANSACTION));
    }
    debug_assert_eq!(seen_entry_ids.len(), draft.entries.len());
    debug_assert_eq!(expected_parent, Some(draft.active_leaf_id));
    Ok(())
}

fn validate_policy_shape(input: &DurableSessionStorePolicyInput) -> Result<(), DurableSessionStoreError> {
    let has_required_shape = input.schema == POLICY_SCHEMA
        && input.durability == REQUIRED_DURABILITY
        && input.locking == REQUIRED_LOCKING
        && input.private_data == REQUIRED_PRIVATE_DATA;
    let has_required_recovery =
        input.tail_recovery == REQUIRED_TAIL_RECOVERY && input.committed_corruption == REQUIRED_CORRUPTION;
    if !has_required_shape || !has_required_recovery {
        return Err(error(DurableSessionStoreIssue::InvalidPolicy, FIELD_POLICY));
    }
    safe_label(input.policy_id.as_str(), FIELD_POLICY)?;
    lowercase_blake3(input.policy_hash.as_str(), FIELD_POLICY)?;
    let has_frame_bound = (MIN_COUNT..=MAX_FRAME_PAYLOAD_BYTES).contains(&input.max_frame_payload_bytes);
    let has_transaction_bound = (MIN_COUNT..=MAX_TRANSACTION_FRAMES).contains(&input.max_transaction_frames);
    let has_file_bound = u64::from(MIN_COUNT) <= input.max_file_bytes && input.max_file_bytes <= MAX_FILE_BYTES;
    if !has_frame_bound || !has_transaction_bound || !has_file_bound {
        return Err(error(DurableSessionStoreIssue::InvalidPolicy, FIELD_POLICY));
    }
    if input.file_mode != PRIVATE_FILE_MODE {
        return Err(error(DurableSessionStoreIssue::InvalidPolicy, FIELD_POLICY));
    }
    validate_exact_labels(input.encryption_postures.as_slice(), REQUIRED_ENCRYPTION_POSTURES)?;
    validate_exact_labels(input.receipt_fields.as_slice(), REQUIRED_RECEIPT_FIELDS)?;
    debug_assert!(input.max_file_bytes >= u64::from(input.max_frame_payload_bytes));
    debug_assert_eq!(input.file_mode, PRIVATE_FILE_MODE);
    Ok(())
}

fn validate_exact_labels(actual: &[String], expected: &[&str]) -> Result<(), DurableSessionStoreError> {
    let has_exact_labels =
        actual.len() == expected.len() && actual.iter().map(String::as_str).eq(expected.iter().copied());
    if !has_exact_labels {
        return Err(error(DurableSessionStoreIssue::InvalidPolicy, FIELD_POLICY));
    }
    Ok(())
}

pub(super) fn hash_parts(domain: &str, parts: &[&[u8]]) -> Result<String, DurableSessionStoreError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        let part_bytes = u64::try_from(part.len())
            .map_err(|_error| error(DurableSessionStoreIssue::BoundExceeded, FIELD_TRANSACTION))?;
        let part_bytes_wire = part_bytes.to_be_bytes();
        hasher.update(super::data::HASH_SEPARATOR);
        hasher.update(&part_bytes_wire);
        hasher.update(part);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

pub(super) fn safe_label(value: &str, field: StoreField) -> Result<(), DurableSessionStoreError> {
    const MAX_SAFE_LABEL_BYTES: usize = 256;
    let is_label_bounded = !value.is_empty() && value.len() <= MAX_SAFE_LABEL_BYTES;
    let has_safe_chars = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ':' | '-'));
    if !is_label_bounded || !has_safe_chars {
        return Err(error(DurableSessionStoreIssue::InvalidDescriptor, field));
    }
    Ok(())
}

pub(super) fn lowercase_blake3(value: &str, field: StoreField) -> Result<(), DurableSessionStoreError> {
    let has_exact_width = value.len() == BLAKE3_HEX_CHARS;
    let has_lowercase_hex = value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !has_exact_width || !has_lowercase_hex {
        return Err(error(DurableSessionStoreIssue::IntegrityMismatch, field));
    }
    Ok(())
}

fn error(issue: DurableSessionStoreIssue, field: StoreField) -> DurableSessionStoreError {
    DurableSessionStoreError::new(issue, field.0)
}
