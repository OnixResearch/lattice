#![allow(unknown_lints, path_segment_repetition)]

const HANDOFF_THREAD_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_thread_id");
const HANDOFF_DIRECTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_direction");
const HANDOFF_SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_source_workspace_id");
const HANDOFF_TARGET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_target_workspace_id");
const HANDOFF_SNAPSHOT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_snapshot_id");
const HANDOFF_INCLUDED_RESOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_included_resource");
const HANDOFF_INCLUDED_RESOURCES_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_included_resources");
const HANDOFF_CONFLICT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_conflict_status");
const HANDOFF_RESTORE_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("handoff_restore_manifest_hash");

const DIRECTION_FOREGROUND_TO_ISOLATED: &str = "foreground_to_isolated";
const DIRECTION_ISOLATED_TO_FOREGROUND: &str = "isolated_to_foreground";
const CONFLICT_CLEAR: &str = "clear";
const CONFLICT_DIRTY_STATE: &str = "dirty_state_conflict";

/// Direction for a snapshot-based thread handoff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffDirection {
    /// Move a foreground thread into an isolated workspace context.
    ForegroundToIsolated,
    /// Move an isolated workspace thread back to the foreground context.
    IsolatedToForeground,
}

impl HandoffDirection {
    /// Parses a handoff direction label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(
            HANDOFF_DIRECTION_FIELD,
            value,
            |normalized| match normalized {
                DIRECTION_FOREGROUND_TO_ISOLATED => Some(Self::ForegroundToIsolated),
                DIRECTION_ISOLATED_TO_FOREGROUND => Some(Self::IsolatedToForeground),
                _ => None,
            },
        )
    }

    /// Returns the stable direction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ForegroundToIsolated => DIRECTION_FOREGROUND_TO_ISOLATED,
            Self::IsolatedToForeground => DIRECTION_ISOLATED_TO_FOREGROUND,
        }
    }
}

/// Handoff conflict status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffConflictStatus {
    /// No conflict blocks the handoff.
    Clear,
    /// Dirty tracked state conflicts block the handoff.
    DirtyStateConflict,
}

impl HandoffConflictStatus {
    /// Parses a handoff conflict status.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(
            HANDOFF_CONFLICT_FIELD,
            value,
            |normalized| match normalized {
                CONFLICT_CLEAR => Some(Self::Clear),
                CONFLICT_DIRTY_STATE => Some(Self::DirtyStateConflict),
                _ => None,
            },
        )
    }

    /// Returns the stable conflict-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clear => CONFLICT_CLEAR,
            Self::DirtyStateConflict => CONFLICT_DIRTY_STATE,
        }
    }
}

/// Snapshot record for moving a thread between foreground and isolated contexts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotHandoffRecord {
    thread_id: alloc::string::String,
    direction: HandoffDirection,
    source_workspace_id: alloc::string::String,
    target_workspace_id: alloc::string::String,
    snapshot_id: alloc::string::String,
    included_resources: alloc::vec::Vec<alloc::string::String>,
    conflict_status: HandoffConflictStatus,
    restore_manifest_hash: alloc::string::String,
}

impl SnapshotHandoffRecord {
    /// Validates a snapshot handoff record.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when required labels are empty,
    /// unsupported, or the included resource manifest is empty.
    pub fn new(draft: SnapshotHandoffRecordDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let thread_id =
            crate::validation::normalized_adapter_field(HANDOFF_THREAD_FIELD, draft.thread_id)?;
        let direction = HandoffDirection::parse(draft.direction)?;
        let source_workspace_id = crate::validation::normalized_adapter_field(
            HANDOFF_SOURCE_FIELD,
            draft.source_workspace_id,
        )?;
        let target_workspace_id = crate::validation::normalized_adapter_field(
            HANDOFF_TARGET_FIELD,
            draft.target_workspace_id,
        )?;
        let snapshot_id =
            crate::validation::normalized_adapter_field(HANDOFF_SNAPSHOT_FIELD, draft.snapshot_id)?;
        let included_resources = super::text::required_texts(
            HANDOFF_INCLUDED_RESOURCES_FIELD,
            HANDOFF_INCLUDED_RESOURCE_FIELD,
            draft.included_resources,
        )?;
        let conflict_status = HandoffConflictStatus::parse(draft.conflict_status)?;
        let restore_manifest_hash = crate::validation::normalized_adapter_field(
            HANDOFF_RESTORE_HASH_FIELD,
            draft.restore_manifest_hash,
        )?;
        Ok(Self {
            thread_id,
            direction,
            source_workspace_id,
            target_workspace_id,
            snapshot_id,
            included_resources,
            conflict_status,
            restore_manifest_hash,
        })
    }

    /// Returns whether the handoff can proceed without manual review.
    #[must_use]
    pub const fn handoff_allowed(&self) -> bool {
        matches!(self.conflict_status, HandoffConflictStatus::Clear)
    }

    /// Returns the thread identity.
    #[must_use]
    pub fn thread_id(&self) -> &str {
        self.thread_id.as_str()
    }

    /// Returns the handoff direction.
    #[must_use]
    pub const fn direction(&self) -> HandoffDirection {
        self.direction
    }

    /// Returns the source workspace identity.
    #[must_use]
    pub fn source_workspace_id(&self) -> &str {
        self.source_workspace_id.as_str()
    }

    /// Returns the target workspace identity.
    #[must_use]
    pub fn target_workspace_id(&self) -> &str {
        self.target_workspace_id.as_str()
    }

    /// Returns the snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        self.snapshot_id.as_str()
    }

    /// Returns the bounded included resource manifest.
    #[must_use]
    pub fn included_resources(&self) -> &[alloc::string::String] {
        self.included_resources.as_slice()
    }

    /// Returns the conflict status.
    #[must_use]
    pub const fn conflict_status(&self) -> HandoffConflictStatus {
        self.conflict_status
    }

    /// Returns the restore manifest hash kept for review.
    #[must_use]
    pub fn restore_manifest_hash(&self) -> &str {
        self.restore_manifest_hash.as_str()
    }
}

/// Borrowed snapshot handoff input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotHandoffRecordDraft<'a> {
    /// Thread identity being moved.
    pub thread_id: &'a str,
    /// Handoff direction label.
    pub direction: &'a str,
    /// Source workspace identity.
    pub source_workspace_id: &'a str,
    /// Target workspace identity.
    pub target_workspace_id: &'a str,
    /// Snapshot identity for this handoff attempt.
    pub snapshot_id: &'a str,
    /// Bounded included resource labels or hashes.
    pub included_resources: &'a [&'a str],
    /// Conflict status label.
    pub conflict_status: &'a str,
    /// Restore manifest hash for manual review.
    pub restore_manifest_hash: &'a str,
}
