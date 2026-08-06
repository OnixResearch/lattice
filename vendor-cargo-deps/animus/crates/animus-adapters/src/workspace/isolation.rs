#![allow(unknown_lints, path_segment_repetition)]

const SNAPSHOT_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_snapshot_id");
const RESTORE_MANIFEST_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_restore_manifest_hash");
const RESTORE_HINT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_restore_hint");
const REQUIRED_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_required_capability");
const ADAPTER_KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_adapter_kind");
const ADAPTER_STATE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_adapter_state");

/// Restore metadata preserved before isolated workspace cleanup or handoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreMetadata {
    snapshot_id: alloc::string::String,
    manifest_hash: alloc::string::String,
    restore_hint: alloc::string::String,
}

impl RestoreMetadata {
    /// Validates bounded restore metadata.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when any field is empty, malformed,
    /// or too long for adapter-boundary metadata.
    pub fn new(draft: RestoreMetadataDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let snapshot_id = crate::validation::normalized_adapter_field(SNAPSHOT_ID_FIELD, draft.snapshot_id)?;
        let manifest_hash =
            crate::validation::normalized_adapter_field(RESTORE_MANIFEST_HASH_FIELD, draft.manifest_hash)?;
        let restore_hint = crate::validation::normalized_adapter_field(RESTORE_HINT_FIELD, draft.restore_hint)?;
        Ok(Self {
            snapshot_id,
            manifest_hash,
            restore_hint,
        })
    }

    /// Returns the snapshot identity used for restore.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        self.snapshot_id.as_str()
    }

    /// Returns the safe manifest hash for restore evidence.
    #[must_use]
    pub fn manifest_hash(&self) -> &str {
        self.manifest_hash.as_str()
    }

    /// Returns bounded host-owned restore guidance.
    #[must_use]
    pub fn restore_hint(&self) -> &str {
        self.restore_hint.as_str()
    }
}

/// Borrowed restore metadata input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreMetadataDraft<'a> {
    /// Snapshot identity preserved by the host adapter.
    pub snapshot_id: &'a str,
    /// Safe manifest hash for restore evidence.
    pub manifest_hash: &'a str,
    /// Bounded restore hint, not raw file contents.
    pub restore_hint: &'a str,
}

/// Validated VCS-neutral isolated task workspace DTO.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IsolatedTaskWorkspace {
    descriptor: super::intent::Descriptor,
    snapshot_id: alloc::string::String,
    restore_metadata: RestoreMetadata,
    adapter_label: alloc::string::String,
    adapter_kind: super::adapter::Kind,
    adapter_capabilities: alloc::vec::Vec<super::adapter::Capability>,
}

impl IsolatedTaskWorkspace {
    /// Validates a VCS-neutral isolated task workspace against adapter capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when snapshot metadata is malformed,
    /// the adapter is unavailable or unsupported, or a required capability is absent.
    pub fn new(draft: IsolatedTaskWorkspaceDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let snapshot_id = crate::validation::normalized_adapter_field(SNAPSHOT_ID_FIELD, draft.snapshot_id)?;
        reject_snapshot_mismatch(SnapshotMatchInput {
            snapshot_id: snapshot_id.as_str(),
            restore_snapshot_id: draft.restore_metadata.snapshot_id(),
        })?;
        reject_unavailable_adapter(draft.adapter)?;
        reject_missing_capabilities(draft.descriptor, draft.adapter)?;
        Ok(Self {
            descriptor: draft.descriptor.clone(),
            snapshot_id,
            restore_metadata: draft.restore_metadata.clone(),
            adapter_label: alloc::string::String::from(draft.adapter.adapter_label()),
            adapter_kind: draft.adapter.kind(),
            adapter_capabilities: draft.adapter.supported_capabilities().to_vec(),
        })
    }

    /// Returns the source identity label.
    #[must_use]
    pub fn source_id(&self) -> &str {
        self.descriptor.source_id()
    }

    /// Returns the isolated workspace identity label.
    #[must_use]
    pub fn workspace_id(&self) -> &str {
        self.descriptor.workspace_id()
    }

    /// Returns portable writable-root labels.
    #[must_use]
    pub fn writable_roots(&self) -> &[alloc::string::String] {
        self.descriptor.writable_roots()
    }

    /// Returns the snapshot identity for the isolated workspace.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        self.snapshot_id.as_str()
    }

    /// Returns restore metadata preserved by the host adapter.
    #[must_use]
    pub const fn restore_metadata(&self) -> &RestoreMetadata {
        &self.restore_metadata
    }

    /// Returns the adapter label used to create the workspace.
    #[must_use]
    pub fn adapter_label(&self) -> &str {
        self.adapter_label.as_str()
    }

    /// Returns the adapter kind used to create the workspace.
    #[must_use]
    pub const fn adapter_kind(&self) -> super::adapter::Kind {
        self.adapter_kind
    }

    /// Returns advertised adapter capabilities used for validation.
    #[must_use]
    pub fn adapter_capabilities(&self) -> &[super::adapter::Capability] {
        self.adapter_capabilities.as_slice()
    }
}

/// Borrowed isolated task workspace input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IsolatedTaskWorkspaceDraft<'a> {
    /// Validated portable workspace descriptor.
    pub descriptor: &'a super::intent::Descriptor,
    /// Snapshot identity for the isolated run.
    pub snapshot_id: &'a str,
    /// Restore metadata preserved before cleanup or handoff.
    pub restore_metadata: &'a RestoreMetadata,
    /// Host-advertised workspace adapter capabilities.
    pub adapter: &'a super::adapter::Capabilities,
}

#[derive(Clone, Copy)]
struct SnapshotMatchInput<'a> {
    snapshot_id: &'a str,
    restore_snapshot_id: &'a str,
}

fn reject_snapshot_mismatch(input: SnapshotMatchInput<'_>) -> Result<(), crate::AdapterContractError> {
    if input.snapshot_id == input.restore_snapshot_id {
        return Ok(());
    }
    Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
        field: SNAPSHOT_ID_FIELD.as_str(),
        value: alloc::string::String::from(input.restore_snapshot_id),
    })
}

fn reject_unavailable_adapter(adapter: &super::adapter::Capabilities) -> Result<(), crate::AdapterContractError> {
    if matches!(adapter.kind(), super::adapter::Kind::Unsupported) {
        return Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: ADAPTER_KIND_FIELD.as_str(),
            value: alloc::string::String::from(adapter.kind().as_str()),
        });
    }
    if matches!(adapter.state(), super::adapter::State::Available) {
        return Ok(());
    }
    Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
        field: ADAPTER_STATE_FIELD.as_str(),
        value: alloc::string::String::from(adapter.state().as_str()),
    })
}

fn reject_missing_capabilities(
    descriptor: &super::intent::Descriptor,
    adapter: &super::adapter::Capabilities,
) -> Result<(), crate::AdapterContractError> {
    for capability in descriptor.required_capabilities() {
        if !adapter.supports(*capability) {
            return Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
                field: REQUIRED_CAPABILITY_FIELD.as_str(),
                value: alloc::string::String::from(capability.as_str()),
            });
        }
    }
    Ok(())
}
