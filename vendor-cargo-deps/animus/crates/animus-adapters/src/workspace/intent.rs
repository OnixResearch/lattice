const SOURCE_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_source_id");
const WORKSPACE_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_id");
const PARENT_WORKSPACE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("parent_workspace_id");
const WRITABLE_ROOT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_writable_root");
const WRITABLE_ROOTS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_writable_roots");

/// Validated speculative workspace descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Descriptor {
    source_id: alloc::string::String,
    workspace_id: alloc::string::String,
    parent_workspace_id: Option<alloc::string::String>,
    writable_roots: alloc::vec::Vec<alloc::string::String>,
    file_inclusion: super::rules::FileInclusionPolicy,
    operation: super::rules::Operation,
    commit_target: super::rules::CommitTarget,
    abort_policy: super::rules::AbortPolicy,
    conflict_policy: super::rules::ConflictPolicy,
    cleanup_policy: super::rules::CleanupPolicy,
    required_capabilities: alloc::vec::Vec<super::adapter::Capability>,
}

impl Descriptor {
    /// Validates a speculative workspace descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when fields are empty, malformed,
    /// unknown, or the descriptor lacks writable roots.
    pub fn new(draft: DescriptorDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        crate::validation::validate_required_adapter_field(SOURCE_ID_FIELD, draft.source_id)?;
        crate::validation::validate_required_adapter_field(WORKSPACE_ID_FIELD, draft.workspace_id)?;
        let parent_workspace_id = super::text::optional_field(PARENT_WORKSPACE_ID_FIELD, draft.parent_workspace_id)?;
        let writable_roots =
            super::text::required_texts(WRITABLE_ROOTS_FIELD, WRITABLE_ROOT_FIELD, draft.writable_roots)?;
        let file_inclusion = super::rules::FileInclusionPolicy::parse(draft.file_inclusion)?;
        let operation = super::rules::Operation::parse(draft.operation)?;
        let commit_target = super::rules::CommitTarget::parse(draft.commit_target)?;
        let abort_policy = super::rules::AbortPolicy::parse(draft.abort_policy)?;
        let conflict_policy = super::rules::ConflictPolicy::parse(draft.conflict_policy)?;
        let cleanup_policy = super::rules::CleanupPolicy::parse(draft.cleanup_policy)?;
        let required_capabilities =
            super::text::collect_capabilities(draft.required_capabilities, super::adapter::Capability::parse)?;

        Ok(Self {
            source_id: alloc::string::String::from(draft.source_id),
            workspace_id: alloc::string::String::from(draft.workspace_id),
            parent_workspace_id,
            writable_roots,
            file_inclusion,
            operation,
            commit_target,
            abort_policy,
            conflict_policy,
            cleanup_policy,
            required_capabilities,
        })
    }

    /// Returns the source identity label.
    #[must_use]
    pub fn source_id(&self) -> &str {
        self.source_id.as_str()
    }

    /// Returns the workspace identity label.
    #[must_use]
    pub fn workspace_id(&self) -> &str {
        self.workspace_id.as_str()
    }

    /// Returns the parent workspace identity, if any.
    #[must_use]
    pub fn parent_workspace_id(&self) -> Option<&str> {
        self.parent_workspace_id.as_deref()
    }

    /// Returns portable writable-root labels.
    #[must_use]
    pub fn writable_roots(&self) -> &[alloc::string::String] {
        self.writable_roots.as_slice()
    }

    /// Returns ignored/local-file inclusion policy.
    #[must_use]
    pub const fn file_inclusion(&self) -> super::rules::FileInclusionPolicy {
        self.file_inclusion
    }

    /// Returns the requested operation.
    #[must_use]
    pub const fn operation(&self) -> super::rules::Operation {
        self.operation
    }

    /// Returns the commit target.
    #[must_use]
    pub const fn commit_target(&self) -> super::rules::CommitTarget {
        self.commit_target
    }

    /// Returns the abort policy.
    #[must_use]
    pub const fn abort_policy(&self) -> super::rules::AbortPolicy {
        self.abort_policy
    }

    /// Returns the conflict policy.
    #[must_use]
    pub const fn conflict_policy(&self) -> super::rules::ConflictPolicy {
        self.conflict_policy
    }

    /// Returns the cleanup policy.
    #[must_use]
    pub const fn cleanup_policy(&self) -> super::rules::CleanupPolicy {
        self.cleanup_policy
    }

    /// Returns required adapter capabilities.
    #[must_use]
    pub fn required_capabilities(&self) -> &[super::adapter::Capability] {
        self.required_capabilities.as_slice()
    }
}

/// Borrowed speculative workspace descriptor input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DescriptorDraft<'a> {
    /// Portable source identity.
    pub source_id: &'a str,
    /// Portable workspace identity.
    pub workspace_id: &'a str,
    /// Optional parent workspace identity.
    pub parent_workspace_id: Option<&'a str>,
    /// Portable writable-root labels.
    pub writable_roots: &'a [&'a str],
    /// Ignored/local-file inclusion policy.
    pub file_inclusion: &'a str,
    /// Requested operation.
    pub operation: &'a str,
    /// Commit target.
    pub commit_target: &'a str,
    /// Abort policy.
    pub abort_policy: &'a str,
    /// Conflict policy.
    pub conflict_policy: &'a str,
    /// Cleanup policy.
    pub cleanup_policy: &'a str,
    /// Required adapter capability labels.
    pub required_capabilities: &'a [&'a str],
}

pub(super) const WORKSPACE_ID_FIELD_NAME: crate::validation::FieldName = WORKSPACE_ID_FIELD;
