/// Borrowed checkpoint identity input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CheckpointDraft<'a> {
    /// Backend-kind label.
    pub backend_kind: &'a str,
    /// Opaque backend checkpoint ID.
    pub checkpoint_id: &'a str,
    /// Safe workspace label.
    pub workspace_label: &'a str,
    /// Safe workspace identity hash.
    pub workspace_hash: &'a str,
    /// Content identity before checkpoint creation.
    pub before_hash: &'a str,
    /// Content identity after checkpoint creation.
    pub after_hash: &'a str,
    /// Bounded changed-file count.
    pub changed_file_count: u32,
    /// Optional session attribution hash.
    pub session_hash: Option<&'a str>,
    /// Optional safe tool label.
    pub tool_label: Option<&'a str>,
    /// Checkpoint lifecycle status label.
    pub status: &'a str,
}

/// VCS-neutral checkpoint identity and safe metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointRecord {
    backend_kind: super::CheckpointBackendKind,
    checkpoint_id: alloc::string::String,
    workspace_label: alloc::string::String,
    workspace_hash: super::CheckpointHash,
    before_hash: super::CheckpointHash,
    after_hash: super::CheckpointHash,
    changed_file_count: u32,
    session_hash: Option<super::CheckpointHash>,
    tool_label: Option<alloc::string::String>,
    status: super::CheckpointStatus,
}

impl CheckpointRecord {
    /// Validates and stores VCS-neutral checkpoint metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::CheckpointError`] when enum labels, required fields,
    /// hashes, optional attribution, or changed-file counts are invalid.
    pub fn new(draft: CheckpointDraft<'_>) -> Result<Self, super::CheckpointError> {
        let backend_kind = super::CheckpointBackendKind::parse(draft.backend_kind)?;
        let checkpoint_id = super::core::required_string(super::CHECKPOINT_ID_FIELD, draft.checkpoint_id)?;
        let workspace_label =
            super::core::required_string(super::CHECKPOINT_WORKSPACE_LABEL_FIELD, draft.workspace_label)?;
        let workspace_hash =
            super::CheckpointHash::new_for_field(super::CHECKPOINT_WORKSPACE_HASH_FIELD, draft.workspace_hash)?;
        let before_hash = super::CheckpointHash::new_for_field(super::CHECKPOINT_BEFORE_HASH_FIELD, draft.before_hash)?;
        let after_hash = super::CheckpointHash::new_for_field(super::CHECKPOINT_AFTER_HASH_FIELD, draft.after_hash)?;
        super::core::validate_changed_file_count(draft.changed_file_count)?;
        let session_hash = super::core::optional_hash(super::CHECKPOINT_SESSION_HASH_FIELD, draft.session_hash)?;
        let tool_label = super::core::optional_string(super::CHECKPOINT_TOOL_LABEL_FIELD, draft.tool_label)?;
        let status = super::CheckpointStatus::parse(draft.status)?;
        Ok(Self {
            backend_kind,
            checkpoint_id,
            workspace_label,
            workspace_hash,
            before_hash,
            after_hash,
            changed_file_count: draft.changed_file_count,
            session_hash,
            tool_label,
            status,
        })
    }

    /// Returns the backend kind.
    #[must_use]
    pub const fn backend_kind(&self) -> super::CheckpointBackendKind {
        self.backend_kind
    }

    /// Returns the opaque checkpoint ID.
    #[must_use]
    pub fn checkpoint_id(&self) -> &str {
        self.checkpoint_id.as_str()
    }

    /// Returns the safe workspace label.
    #[must_use]
    pub fn workspace_label(&self) -> &str {
        self.workspace_label.as_str()
    }

    /// Returns the safe workspace identity hash.
    #[must_use]
    pub const fn workspace_hash(&self) -> &super::CheckpointHash {
        &self.workspace_hash
    }

    /// Returns the pre-checkpoint content hash.
    #[must_use]
    pub const fn before_hash(&self) -> &super::CheckpointHash {
        &self.before_hash
    }

    /// Returns the post-checkpoint content hash.
    #[must_use]
    pub const fn after_hash(&self) -> &super::CheckpointHash {
        &self.after_hash
    }

    /// Returns the bounded changed-file count.
    #[must_use]
    pub const fn changed_file_count(&self) -> u32 {
        self.changed_file_count
    }

    /// Returns optional session attribution hash.
    #[must_use]
    pub const fn session_hash(&self) -> Option<&super::CheckpointHash> {
        self.session_hash.as_ref()
    }

    /// Returns optional safe tool label.
    #[must_use]
    pub fn tool_label(&self) -> Option<&str> {
        self.tool_label.as_deref()
    }

    /// Returns the checkpoint lifecycle status.
    #[must_use]
    pub const fn status(&self) -> super::CheckpointStatus {
        self.status
    }
}
