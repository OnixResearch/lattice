//! Deterministic bounded durable-session wire framing and recovery.

type DurableSessionCommit = super::data::DurableSessionCommit;
type DurableSessionFrame = super::data::DurableSessionFrame;
type DurableSessionRecoveryReport = super::data::DurableSessionRecoveryReport;
type DurableSessionRecoveryStatus = super::data::DurableSessionRecoveryStatus;
type DurableSessionRecordKind = super::data::DurableSessionRecordKind;
type DurableSessionStoreDescriptor = super::data::DurableSessionStoreDescriptor;
type DurableSessionStoreError = super::data::DurableSessionStoreError;
type DurableSessionStoreIssue = super::data::DurableSessionStoreIssue;
type DurableSessionStorePolicy = super::data::DurableSessionStorePolicy;
type DurableSessionTransaction = super::data::DurableSessionTransaction;

const MAGIC: &[u8; 8] = b"ANIMSJ01";
const RECORD_FRAME: u8 = 1;
const RECORD_COMMIT: u8 = 2;
const ENVELOPE_HEADER_BYTES: usize = 13;
const HASH_BYTES: usize = super::data::BLAKE3_HEX_CHARS;
const MAX_LABEL_BYTES: usize = 256;
const FIELD_WIRE: super::core::StoreField = super::core::StoreField("durable_session_wire");

struct WireBuffer {
    bytes: Vec<u8>,
}

impl WireBuffer {
    const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn reserve(&mut self, additional_bytes: usize) {
        self.bytes.reserve(additional_bytes);
    }

    fn extend_from_slice(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn push(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Encodes an ordered frame group followed by its logical commit frame.
///
/// # Errors
///
/// Returns [`DurableSessionStoreError`] when labels, counts, payload sizes, or
/// output bytes exceed the validated policy.
// r[impl agent_harness_runtime.durable_session_store.framing]
// r[impl agent_harness_runtime.durable_session_store.identity]
pub fn encode_durable_session_transaction(
    policy: &DurableSessionStorePolicy,
    transaction: &DurableSessionTransaction,
) -> Result<Vec<u8>, DurableSessionStoreError> {
    let mut output = WireBuffer::new();
    for frame in &transaction.frames {
        let body = encode_frame_body(frame)?;
        append_record(&mut output, RECORD_FRAME, body.as_slice())?;
    }
    let commit_body = encode_commit_body(&transaction.commit)?;
    append_record(&mut output, RECORD_COMMIT, commit_body.as_slice())?;
    let output_bytes =
        u64::try_from(output.len()).map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
    if output_bytes > policy.max_file_bytes {
        return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
    }
    debug_assert!(!output.is_empty());
    debug_assert_eq!(&output.as_slice()[..MAGIC.len()], MAGIC);
    Ok(output.into_bytes())
}

/// Recovers the validated committed prefix of a bounded journal byte slice.
///
/// # Errors
///
/// Returns [`DurableSessionStoreError`] for complete malformed records,
/// unsupported schema, sequence gaps, stale heads, identity mismatch, or file
/// bounds. Only an incomplete final record or complete uncommitted final group
/// is ignored.
// r[impl agent_harness_runtime.durable_session_store.recovery]
// r[impl agent_harness_runtime.durable_session_store.commit]
pub fn recover_durable_session_log(
    policy: &DurableSessionStorePolicy,
    descriptor: &DurableSessionStoreDescriptor,
    bytes: &[u8],
) -> Result<DurableSessionRecoveryReport, DurableSessionStoreError> {
    super::core::validate_durable_session_store_descriptor(policy, descriptor)?;
    let journal_bytes =
        u64::try_from(bytes.len()).map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
    if journal_bytes > policy.max_file_bytes {
        return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
    }
    debug_assert!(journal_bytes <= policy.max_file_bytes);
    debug_assert_eq!(descriptor.policy_hash, policy.policy_hash);
    if bytes.is_empty() {
        return Ok(empty_report());
    }
    let mut state = RecoveryState::new();
    while state.offset < bytes.len() {
        let record_start = state.offset;
        let remaining = bytes.len().saturating_sub(state.offset);
        if remaining < ENVELOPE_HEADER_BYTES {
            state.mark_incomplete_tail(record_start, bytes.len())?;
            break;
        }
        let magic_end = state
            .offset
            .checked_add(MAGIC.len())
            .ok_or_else(|| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
        if bytes.get(state.offset..magic_end) != Some(MAGIC.as_slice()) {
            return Err(wire_error(DurableSessionStoreIssue::MidFileCorruption));
        }
        state.offset = magic_end;
        let record_type = bytes[state.offset];
        state.offset = state.offset.saturating_add(1);
        let body_bytes = read_u32_at(bytes, &mut state.offset)?;
        if u64::from(body_bytes) > policy.max_file_bytes {
            return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
        }
        let body_len =
            usize::try_from(body_bytes).map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
        let Some(body_end) = state.offset.checked_add(body_len) else {
            return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
        };
        if body_end > bytes.len() {
            state.mark_incomplete_tail(record_start, bytes.len())?;
            break;
        }
        let body = &bytes[state.offset..body_end];
        state.offset = body_end;
        match record_type {
            RECORD_FRAME => {
                let frame = decode_frame_body(policy, descriptor, body)?;
                state.accept_frame(frame, record_start)?;
            }
            RECORD_COMMIT => {
                let commit = decode_commit_body(body)?;
                state.accept_commit(descriptor, commit)?;
            }
            _other => return Err(wire_error(DurableSessionStoreIssue::MidFileCorruption)),
        }
    }
    state.finish(bytes.len())
}

struct RecoveryState {
    offset: usize,
    committed_frames: Vec<DurableSessionFrame>,
    pending_frames: Vec<DurableSessionFrame>,
    pending_start: Option<usize>,
    active_leaf_id: Option<String>,
    head_hash: String,
    next_sequence: u64,
    status: DurableSessionRecoveryStatus,
    ignored_tail_bytes: u64,
}

impl RecoveryState {
    fn new() -> Self {
        Self {
            offset: 0,
            committed_frames: Vec::new(),
            pending_frames: Vec::new(),
            pending_start: None,
            active_leaf_id: None,
            head_hash: String::from(super::data::DURABLE_SESSION_EMPTY_HEAD_HASH),
            next_sequence: 1,
            status: DurableSessionRecoveryStatus::Clean,
            ignored_tail_bytes: 0,
        }
    }

    fn accept_frame(
        &mut self,
        frame: DurableSessionFrame,
        record_start: usize,
    ) -> Result<(), DurableSessionStoreError> {
        if frame.sequence != self.next_sequence {
            return Err(wire_error(DurableSessionStoreIssue::SequenceGap));
        }
        let expected_previous =
            self.pending_frames.last().map_or(self.head_hash.as_str(), |previous| previous.frame_hash.as_str());
        if frame.previous_frame_hash != expected_previous {
            return Err(wire_error(DurableSessionStoreIssue::IntegrityMismatch));
        }
        let expected_parent = self
            .pending_frames
            .last()
            .map(|previous| previous.entry_id.as_str())
            .or(self.active_leaf_id.as_deref());
        let is_entry_duplicate = self
            .committed_frames
            .iter()
            .chain(self.pending_frames.iter())
            .any(|existing| existing.entry_id == frame.entry_id);
        if frame.parent_id.as_deref() != expected_parent || is_entry_duplicate {
            return Err(wire_error(DurableSessionStoreIssue::IntegrityMismatch));
        }
        if self.pending_start.is_none() {
            self.pending_start = Some(record_start);
        }
        self.next_sequence =
            self.next_sequence.checked_add(1).ok_or_else(|| wire_error(DurableSessionStoreIssue::SequenceGap))?;
        self.pending_frames.push(frame);
        Ok(())
    }

    fn accept_commit(
        &mut self,
        descriptor: &DurableSessionStoreDescriptor,
        commit: DurableSessionCommit,
    ) -> Result<(), DurableSessionStoreError> {
        if self.pending_frames.is_empty() {
            return Err(wire_error(DurableSessionStoreIssue::StaleHead));
        }
        validate_commit(descriptor, self.head_hash.as_str(), &self.pending_frames, &commit)?;
        self.head_hash = commit.commit_hash;
        self.active_leaf_id = Some(commit.active_leaf_id);
        self.committed_frames.append(&mut self.pending_frames);
        self.pending_start = None;
        self.status = DurableSessionRecoveryStatus::Clean;
        Ok(())
    }

    fn mark_incomplete_tail(&mut self, record_start: usize, file_end: usize) -> Result<(), DurableSessionStoreError> {
        let ignored_start = self.pending_start.unwrap_or(record_start);
        self.ignored_tail_bytes = u64::try_from(file_end.saturating_sub(ignored_start))
            .map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
        self.pending_frames.clear();
        self.status = DurableSessionRecoveryStatus::IncompleteTailFrame;
        self.offset = file_end;
        Ok(())
    }

    fn finish(mut self, file_end: usize) -> Result<DurableSessionRecoveryReport, DurableSessionStoreError> {
        if !self.pending_frames.is_empty() {
            let pending_start = self.pending_start.unwrap_or(file_end);
            self.ignored_tail_bytes = u64::try_from(file_end.saturating_sub(pending_start))
                .map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
            self.pending_frames.clear();
            self.status = DurableSessionRecoveryStatus::IncompleteTailTransaction;
        }
        debug_assert!(self.pending_frames.is_empty());
        debug_assert_eq!(self.head_hash.len(), HASH_BYTES);
        Ok(DurableSessionRecoveryReport {
            status: self.status,
            committed_frames: self.committed_frames,
            active_leaf_id: self.active_leaf_id,
            committed_head_hash: self.head_hash,
            ignored_tail_bytes: self.ignored_tail_bytes,
        })
    }
}

fn empty_report() -> DurableSessionRecoveryReport {
    DurableSessionRecoveryReport {
        status: DurableSessionRecoveryStatus::Empty,
        committed_frames: Vec::new(),
        active_leaf_id: None,
        committed_head_hash: String::from(super::data::DURABLE_SESSION_EMPTY_HEAD_HASH),
        ignored_tail_bytes: 0,
    }
}

fn encode_frame_body(frame: &DurableSessionFrame) -> Result<Vec<u8>, DurableSessionStoreError> {
    let mut body = WireBuffer::new();
    body.extend_from_slice(&frame.schema_version.to_be_bytes());
    body.extend_from_slice(&frame.sequence.to_be_bytes());
    body.push(frame.kind.wire_code());
    push_text(&mut body, frame.branch_id.as_str())?;
    push_text(&mut body, frame.entry_id.as_str())?;
    push_optional_text(&mut body, frame.parent_id.as_deref())?;
    let payload_bytes =
        u32::try_from(frame.payload.len()).map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
    body.extend_from_slice(&payload_bytes.to_be_bytes());
    body.extend_from_slice(frame.payload.as_slice());
    push_hash(&mut body, frame.payload_hash.as_str())?;
    push_hash(&mut body, frame.previous_frame_hash.as_str())?;
    push_hash(&mut body, frame.frame_hash.as_str())?;
    Ok(body.into_bytes())
}

fn decode_frame_body(
    policy: &DurableSessionStorePolicy,
    descriptor: &DurableSessionStoreDescriptor,
    body: &[u8],
) -> Result<DurableSessionFrame, DurableSessionStoreError> {
    let mut cursor = Cursor::new(body);
    let schema_version = cursor.read_u16()?;
    if schema_version != super::data::SUPPORTED_FRAME_SCHEMA {
        return Err(wire_error(DurableSessionStoreIssue::UnsupportedSchema));
    }
    let sequence = cursor.read_u64()?;
    let kind = DurableSessionRecordKind::from_wire_code(cursor.read_u8()?)
        .ok_or_else(|| wire_error(DurableSessionStoreIssue::InvalidFrame))?;
    let branch_id = cursor.read_text()?;
    let entry_id = cursor.read_text()?;
    let parent_text = cursor.read_text()?;
    let parent_id = if parent_text.is_empty() {
        None
    } else {
        Some(parent_text)
    };
    let payload_bytes = cursor.read_u32()?;
    if payload_bytes == 0 || payload_bytes > policy.max_frame_payload_bytes {
        return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
    }
    let payload = cursor.read_bytes(payload_bytes)?.to_vec();
    let payload_hash = cursor.read_hash()?;
    let previous_frame_hash = cursor.read_hash()?;
    let frame_hash = cursor.read_hash()?;
    cursor.finish()?;
    let frame = DurableSessionFrame {
        schema_version,
        sequence,
        branch_id,
        entry_id,
        parent_id,
        kind,
        payload,
        payload_hash,
        previous_frame_hash,
        frame_hash,
    };
    validate_frame(descriptor, &frame)?;
    debug_assert_eq!(frame.schema_version, super::data::SUPPORTED_FRAME_SCHEMA);
    debug_assert_eq!(frame.branch_id, descriptor.branch_id);
    Ok(frame)
}

fn encode_commit_body(commit: &DurableSessionCommit) -> Result<Vec<u8>, DurableSessionStoreError> {
    let mut body = WireBuffer::new();
    body.extend_from_slice(&super::data::SUPPORTED_FRAME_SCHEMA.to_be_bytes());
    push_hash(&mut body, commit.transaction_hash.as_str())?;
    push_hash(&mut body, commit.prior_head_hash.as_str())?;
    push_text(&mut body, commit.active_leaf_id.as_str())?;
    let frame_count = u32::try_from(commit.frame_hashes.len())
        .map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
    body.extend_from_slice(&frame_count.to_be_bytes());
    for frame_hash in &commit.frame_hashes {
        push_hash(&mut body, frame_hash.as_str())?;
    }
    push_hash(&mut body, commit.commit_hash.as_str())?;
    Ok(body.into_bytes())
}

fn decode_commit_body(body: &[u8]) -> Result<DurableSessionCommit, DurableSessionStoreError> {
    let mut cursor = Cursor::new(body);
    let schema_version = cursor.read_u16()?;
    if schema_version != super::data::SUPPORTED_FRAME_SCHEMA {
        return Err(wire_error(DurableSessionStoreIssue::UnsupportedSchema));
    }
    let transaction_hash = cursor.read_hash()?;
    let prior_head_hash = cursor.read_hash()?;
    let active_leaf_id = cursor.read_text()?;
    let frame_count = cursor.read_u32()?;
    if frame_count == 0 || frame_count > super::data::MAX_TRANSACTION_FRAMES {
        return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
    }
    let frame_hash_count =
        usize::try_from(frame_count).map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
    let mut frame_hashes = Vec::with_capacity(frame_hash_count);
    for _frame_index in 0..frame_count {
        frame_hashes.push(cursor.read_hash()?);
    }
    let commit_hash = cursor.read_hash()?;
    cursor.finish()?;
    debug_assert_eq!(schema_version, super::data::SUPPORTED_FRAME_SCHEMA);
    debug_assert_eq!(u32::try_from(frame_hashes.len()), Ok(frame_count));
    Ok(DurableSessionCommit {
        transaction_hash,
        prior_head_hash,
        active_leaf_id,
        frame_hashes,
        commit_hash,
    })
}

fn validate_frame(
    descriptor: &DurableSessionStoreDescriptor,
    frame: &DurableSessionFrame,
) -> Result<(), DurableSessionStoreError> {
    super::core::safe_label(frame.branch_id.as_str(), FIELD_WIRE)?;
    super::core::safe_label(frame.entry_id.as_str(), FIELD_WIRE)?;
    if let Some(parent_id) = &frame.parent_id {
        super::core::safe_label(parent_id.as_str(), FIELD_WIRE)?;
    }
    if frame.branch_id != descriptor.branch_id {
        return Err(wire_error(DurableSessionStoreIssue::IntegrityMismatch));
    }
    let expected_payload_hash = blake3::hash(frame.payload.as_slice()).to_hex().to_string();
    if expected_payload_hash != frame.payload_hash {
        return Err(wire_error(DurableSessionStoreIssue::IntegrityMismatch));
    }
    let sequence = frame.sequence.to_string();
    let parent = frame.parent_id.as_deref().unwrap_or("");
    let payload_bytes = frame.payload.len().to_string();
    let expected_frame_hash = super::core::hash_parts(super::data::DURABLE_SESSION_FRAME_HASH_DOMAIN, &[
        descriptor.session_id.as_bytes(),
        descriptor.branch_id.as_bytes(),
        sequence.as_bytes(),
        frame.entry_id.as_bytes(),
        frame.kind.as_str().as_bytes(),
        parent.as_bytes(),
        payload_bytes.as_bytes(),
        frame.payload_hash.as_bytes(),
        frame.previous_frame_hash.as_bytes(),
    ])?;
    if expected_frame_hash != frame.frame_hash {
        return Err(wire_error(DurableSessionStoreIssue::IntegrityMismatch));
    }
    debug_assert_eq!(expected_payload_hash, frame.payload_hash);
    debug_assert_eq!(expected_frame_hash, frame.frame_hash);
    Ok(())
}

fn validate_commit(
    descriptor: &DurableSessionStoreDescriptor,
    current_head_hash: &str,
    frames: &[DurableSessionFrame],
    commit: &DurableSessionCommit,
) -> Result<(), DurableSessionStoreError> {
    if commit.prior_head_hash != current_head_hash {
        return Err(wire_error(DurableSessionStoreIssue::StaleHead));
    }
    let actual_hashes = frames.iter().map(|frame| frame.frame_hash.as_str()).collect::<Vec<_>>();
    if actual_hashes.as_slice() != commit.frame_hashes.as_slice() {
        return Err(wire_error(DurableSessionStoreIssue::IntegrityMismatch));
    }
    super::core::safe_label(commit.active_leaf_id.as_str(), FIELD_WIRE)?;
    if frames.last().map(|frame| frame.entry_id.as_str()) != Some(commit.active_leaf_id.as_str()) {
        return Err(wire_error(DurableSessionStoreIssue::IntegrityMismatch));
    }
    let mut transaction_parts = vec![
        descriptor.session_id.as_bytes(),
        descriptor.branch_id.as_bytes(),
        current_head_hash.as_bytes(),
        commit.active_leaf_id.as_bytes(),
    ];
    transaction_parts.extend(actual_hashes.iter().map(|hash| hash.as_bytes()));
    let expected_transaction_hash =
        super::core::hash_parts(super::data::DURABLE_SESSION_TRANSACTION_HASH_DOMAIN, transaction_parts.as_slice())?;
    let expected_commit_hash = super::core::hash_parts(super::data::DURABLE_SESSION_COMMIT_HASH_DOMAIN, &[
        expected_transaction_hash.as_bytes(),
        current_head_hash.as_bytes(),
        commit.active_leaf_id.as_bytes(),
    ])?;
    if expected_transaction_hash != commit.transaction_hash || expected_commit_hash != commit.commit_hash {
        return Err(wire_error(DurableSessionStoreIssue::IntegrityMismatch));
    }
    debug_assert_eq!(expected_transaction_hash, commit.transaction_hash);
    debug_assert_eq!(expected_commit_hash, commit.commit_hash);
    Ok(())
}

fn append_record(output: &mut WireBuffer, record_type: u8, body: &[u8]) -> Result<(), DurableSessionStoreError> {
    let body_bytes = u32::try_from(body.len()).map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
    output.reserve(ENVELOPE_HEADER_BYTES.saturating_add(body.len()));
    output.extend_from_slice(MAGIC);
    output.push(record_type);
    output.extend_from_slice(&body_bytes.to_be_bytes());
    output.extend_from_slice(body);
    Ok(())
}

fn push_text(output: &mut WireBuffer, value: &str) -> Result<(), DurableSessionStoreError> {
    super::core::safe_label(value, FIELD_WIRE)?;
    let text_bytes =
        u16::try_from(value.len()).map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
    output.extend_from_slice(&text_bytes.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn push_optional_text(output: &mut WireBuffer, value: Option<&str>) -> Result<(), DurableSessionStoreError> {
    let Some(value) = value else {
        output.extend_from_slice(&0_u16.to_be_bytes());
        return Ok(());
    };
    push_text(output, value)
}

fn push_hash(output: &mut WireBuffer, value: &str) -> Result<(), DurableSessionStoreError> {
    super::core::lowercase_blake3(value, FIELD_WIRE)?;
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn read_u32_at(bytes: &[u8], offset: &mut usize) -> Result<u32, DurableSessionStoreError> {
    let Some(end) = offset.checked_add(core::mem::size_of::<u32>()) else {
        return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
    };
    let raw = bytes.get(*offset..end).ok_or_else(|| wire_error(DurableSessionStoreIssue::IncompleteTail))?;
    let array = <[u8; core::mem::size_of::<u32>()]>::try_from(raw)
        .map_err(|_error| wire_error(DurableSessionStoreIssue::MidFileCorruption))?;
    *offset = end;
    Ok(u32::from_be_bytes(array))
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, DurableSessionStoreError> {
        let value =
            *self.bytes.get(self.offset).ok_or_else(|| wire_error(DurableSessionStoreIssue::MidFileCorruption))?;
        self.offset = self.offset.saturating_add(1);
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, DurableSessionStoreError> {
        let raw = self.read_exact(core::mem::size_of::<u16>())?;
        let array = <[u8; core::mem::size_of::<u16>()]>::try_from(raw)
            .map_err(|_error| wire_error(DurableSessionStoreIssue::MidFileCorruption))?;
        Ok(u16::from_be_bytes(array))
    }

    fn read_u32(&mut self) -> Result<u32, DurableSessionStoreError> {
        let raw = self.read_exact(core::mem::size_of::<u32>())?;
        let array = <[u8; core::mem::size_of::<u32>()]>::try_from(raw)
            .map_err(|_error| wire_error(DurableSessionStoreIssue::MidFileCorruption))?;
        Ok(u32::from_be_bytes(array))
    }

    fn read_u64(&mut self) -> Result<u64, DurableSessionStoreError> {
        let raw = self.read_exact(core::mem::size_of::<u64>())?;
        let array = <[u8; core::mem::size_of::<u64>()]>::try_from(raw)
            .map_err(|_error| wire_error(DurableSessionStoreIssue::MidFileCorruption))?;
        Ok(u64::from_be_bytes(array))
    }

    fn read_text(&mut self) -> Result<String, DurableSessionStoreError> {
        let text_bytes = usize::from(self.read_u16()?);
        if text_bytes > MAX_LABEL_BYTES {
            return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
        }
        let bytes = self.read_exact(text_bytes)?;
        let value =
            core::str::from_utf8(bytes).map_err(|_error| wire_error(DurableSessionStoreIssue::MidFileCorruption))?;
        if !value.is_empty() {
            super::core::safe_label(value, FIELD_WIRE)?;
        }
        Ok(String::from(value))
    }

    fn read_hash(&mut self) -> Result<String, DurableSessionStoreError> {
        let bytes = self.read_exact(HASH_BYTES)?;
        let value =
            core::str::from_utf8(bytes).map_err(|_error| wire_error(DurableSessionStoreIssue::IntegrityMismatch))?;
        super::core::lowercase_blake3(value, FIELD_WIRE)?;
        Ok(String::from(value))
    }

    fn read_bytes(&mut self, count: u32) -> Result<&'a [u8], DurableSessionStoreError> {
        let count = usize::try_from(count).map_err(|_error| wire_error(DurableSessionStoreIssue::BoundExceeded))?;
        self.read_exact(count)
    }

    fn read_exact(&mut self, count: usize) -> Result<&'a [u8], DurableSessionStoreError> {
        let Some(end) = self.offset.checked_add(count) else {
            return Err(wire_error(DurableSessionStoreIssue::BoundExceeded));
        };
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| wire_error(DurableSessionStoreIssue::MidFileCorruption))?;
        self.offset = end;
        Ok(value)
    }

    fn finish(self) -> Result<(), DurableSessionStoreError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(wire_error(DurableSessionStoreIssue::MidFileCorruption))
        }
    }
}

fn wire_error(issue: DurableSessionStoreIssue) -> DurableSessionStoreError {
    DurableSessionStoreError::new(issue, FIELD_WIRE.0)
}
