/// Repository-relative file identity for provenance observations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceFileIdentity {
    /// Repository-relative path label, never an absolute path.
    pub relative_path: Text,
    /// Stable file identity supplied by the host or storage backend.
    pub stable_id: Text,
    /// BLAKE3 content hash for the observed content state.
    pub content_hash: Text,
}

/// Borrowed file identity fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceFileIdentityDraft<'a> {
    /// Repository-relative path label, never an absolute path.
    pub relative_path: &'a str,
    /// Stable file identity supplied by the host or storage backend.
    pub stable_id: &'a str,
    /// BLAKE3 content hash for the observed content state.
    pub content_hash: &'a str,
}

impl ProvenanceFileIdentity {
    /// Validates a VCS-neutral file identity.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when labels are unsafe, the path is absolute
    /// or escaping, or the content hash is malformed.
    pub fn validate(draft: &ProvenanceFileIdentityDraft<'_>) -> Result<Self, ProvenanceError> {
        validate_repository_relative_path(draft.relative_path)?;
        Ok(Self {
            relative_path: validate_safe_text("relative_path", draft.relative_path.to_owned())?,
            stable_id: validate_safe_text("stable_file_id", draft.stable_id.to_owned())?,
            content_hash: validate_hash("content_hash", draft.content_hash)?.to_owned(),
        })
    }
}

/// In-memory content observation used by provenance checkpointing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceContentObservation {
    /// Observed file identity.
    pub file: ProvenanceFileIdentity,
    /// Attributed line ranges present in this content state.
    pub line_ranges: Collection<ProvenanceLineRange>,
    /// Bounded byte count supplied by the host.
    pub byte_count: u64,
}

/// Borrowed content observation fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceContentObservationDraft<'a> {
    /// Observed file identity fields.
    pub file: ProvenanceFileIdentityDraft<'a>,
    /// Attributed line ranges present in this content state.
    pub line_ranges: &'a [ProvenanceLineRange],
    /// Bounded byte count supplied by the host.
    pub byte_count: u64,
}

impl ProvenanceContentObservation {
    /// Validates an in-memory content observation.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when file identity, line ranges, or counts are
    /// malformed.
    pub fn validate(
        draft: &ProvenanceContentObservationDraft<'_>,
    ) -> Result<Self, ProvenanceError> {
        validate_count("observation_byte_count", draft.byte_count)?;
        let line_ranges = compress_line_ranges(draft.line_ranges)?;
        Ok(Self {
            file: ProvenanceFileIdentity::validate(&draft.file)?,
            line_ranges,
            byte_count: draft.byte_count,
        })
    }
}

/// Borrowed mutating-tool checkpoint fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceToolCheckpointDraft<'a> {
    /// Safe session identity.
    pub session_id: &'a str,
    /// Safe trace identity.
    pub trace_id: &'a str,
    /// Safe tool name from policy.
    pub tool_name: &'a str,
    /// Canonical mutation kind label.
    pub mutation_kind: &'a str,
    /// Observation before the tool result is applied.
    pub before: ProvenanceContentObservationDraft<'a>,
    /// Observation after the tool result is applied.
    pub after: ProvenanceContentObservationDraft<'a>,
}

/// Validated mutating-tool checkpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceToolCheckpoint {
    /// Safe session identity.
    pub session_id: Text,
    /// Safe trace identity.
    pub trace_id: Text,
    /// Safe tool name from policy.
    pub tool_name: Text,
    /// Canonical mutation kind.
    pub mutation_kind: ProvenanceMutationKind,
    /// Observation before the tool result is applied.
    pub before: ProvenanceContentObservation,
    /// Observation after the tool result is applied.
    pub after: ProvenanceContentObservation,
}

impl ProvenanceToolCheckpoint {
    /// Validates a mutating-tool checkpoint.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when safe labels, observations, or mutation
    /// kind fields are malformed.
    pub fn validate(draft: &ProvenanceToolCheckpointDraft<'_>) -> Result<Self, ProvenanceError> {
        Ok(Self {
            session_id: validate_safe_text("session_id", draft.session_id.to_owned())?,
            trace_id: validate_safe_text("trace_id", draft.trace_id.to_owned())?,
            tool_name: validate_safe_text("tool_name", draft.tool_name.to_owned())?,
            mutation_kind: ProvenanceMutationKind::parse(draft.mutation_kind)?,
            before: ProvenanceContentObservation::validate(&draft.before)?,
            after: ProvenanceContentObservation::validate(&draft.after)?,
        })
    }
}

/// Safe receipt emitted after checkpoint persistence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceToolCheckpointReceipt {
    /// Receipt status label.
    pub status: Text,
    /// Backend kind label.
    pub backend_kind: Text,
    /// Canonical mutation kind.
    pub mutation_kind: ProvenanceMutationKind,
    /// Safe session identity.
    pub session_id: Text,
    /// Safe trace identity.
    pub trace_id: Text,
    /// Before-observation content hash.
    pub before_hash: Text,
    /// After-observation content hash.
    pub after_hash: Text,
    /// Bounded count of observations represented by this receipt.
    pub observation_count: u64,
    /// Deterministic BLAKE3 receipt hash.
    pub receipt_hash: Text,
}

impl ProvenanceToolCheckpointReceipt {
    /// Creates a safe checkpoint receipt.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when backend labels are unsafe.
    pub fn from_checkpoint(
        status: &str,
        backend_kind: &str,
        checkpoint: &ProvenanceToolCheckpoint,
    ) -> Result<Self, ProvenanceError> {
        let status = validate_safe_text("checkpoint_status", status.to_owned())?;
        let backend_kind = validate_safe_text("backend_kind", backend_kind.to_owned())?;
        let receipt_hash = checkpoint_receipt_hash(CheckpointReceiptHashInput {
            status: status.as_str(),
            backend_kind: backend_kind.as_str(),
            mutation_kind: checkpoint.mutation_kind,
            session_id: checkpoint.session_id.as_str(),
            trace_id: checkpoint.trace_id.as_str(),
            before_hash: checkpoint.before.file.content_hash.as_str(),
            after_hash: checkpoint.after.file.content_hash.as_str(),
        });
        Ok(Self {
            status,
            backend_kind,
            mutation_kind: checkpoint.mutation_kind,
            session_id: checkpoint.session_id.clone(),
            trace_id: checkpoint.trace_id.clone(),
            before_hash: checkpoint.before.file.content_hash.clone(),
            after_hash: checkpoint.after.file.content_hash.clone(),
            observation_count: CHECKPOINT_OBSERVATION_COUNT,
            receipt_hash,
        })
    }
}
fn validate_repository_relative_path(path: &str) -> Result<(), ProvenanceError> {
    if path.is_empty() {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::EmptyField,
            "relative_path",
            UNKNOWN_SAFE_FIELD,
            "relative path must not be empty",
        ));
    }
    if path.starts_with(PATH_SEPARATOR) {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::UnsafeMetadata,
            "relative_path",
            "absolute_path",
            "relative path must not be absolute",
        ));
    }
    for segment in path.split(PATH_SEPARATOR) {
        if segment == CURRENT_DIRECTORY_SEGMENT || segment == PARENT_DIRECTORY_SEGMENT {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::UnsafeMetadata,
                "relative_path",
                segment,
                "relative path must not contain dot traversal segments",
            ));
        }
    }
    Ok(())
}
