const SUPPORTED_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_supported_capability");
const ADAPTER_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_adapter_label");
const ADAPTER_KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_adapter_kind");
const ADAPTER_STATE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("workspace_adapter_state");

const ADAPTER_KIND_BRANCHFS_LIKE: &str = "branchfs_like";
const ADAPTER_KIND_JUJUTSU_WORKSPACE: &str = "jujutsu_workspace";
const ADAPTER_KIND_VCS_WORKTREE: &str = "vcs_worktree";
const ADAPTER_KIND_TEMPDIR_SNAPSHOT: &str = "tempdir_snapshot";
const ADAPTER_KIND_UNSUPPORTED: &str = "unsupported";
const ADAPTER_STATE_AVAILABLE: &str = "available";
const ADAPTER_STATE_UNAVAILABLE: &str = "unavailable";
const ADAPTER_STATE_PARTIAL: &str = "partially_enforceable";

/// Backend kind advertised by a workspace adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// BranchFS-like copy-on-write backend.
    BranchFsLike,
    /// Jujutsu workspace backend.
    JujutsuWorkspace,
    /// Generic VCS worktree backend.
    VcsWorktree,
    /// Temporary snapshot directory backend.
    TempdirSnapshot,
    /// Unsupported backend placeholder.
    Unsupported,
}

impl Kind {
    /// Parses a workspace adapter kind.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(ADAPTER_KIND_FIELD, value, |normalized| match normalized {
            ADAPTER_KIND_BRANCHFS_LIKE => Some(Self::BranchFsLike),
            ADAPTER_KIND_JUJUTSU_WORKSPACE => Some(Self::JujutsuWorkspace),
            ADAPTER_KIND_VCS_WORKTREE => Some(Self::VcsWorktree),
            ADAPTER_KIND_TEMPDIR_SNAPSHOT => Some(Self::TempdirSnapshot),
            ADAPTER_KIND_UNSUPPORTED => Some(Self::Unsupported),
            _ => None,
        })
    }

    /// Returns the stable adapter-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BranchFsLike => ADAPTER_KIND_BRANCHFS_LIKE,
            Self::JujutsuWorkspace => ADAPTER_KIND_JUJUTSU_WORKSPACE,
            Self::VcsWorktree => ADAPTER_KIND_VCS_WORKTREE,
            Self::TempdirSnapshot => ADAPTER_KIND_TEMPDIR_SNAPSHOT,
            Self::Unsupported => ADAPTER_KIND_UNSUPPORTED,
        }
    }
}

/// Availability state advertised by a workspace adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    /// Adapter can enforce its advertised capabilities.
    Available,
    /// Adapter is unavailable on the current host.
    Unavailable,
    /// Adapter can enforce only part of the requested policy.
    PartiallyEnforceable,
}

impl State {
    /// Parses a workspace adapter state.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(ADAPTER_STATE_FIELD, value, |normalized| match normalized {
            ADAPTER_STATE_AVAILABLE => Some(Self::Available),
            ADAPTER_STATE_UNAVAILABLE => Some(Self::Unavailable),
            ADAPTER_STATE_PARTIAL => Some(Self::PartiallyEnforceable),
            _ => None,
        })
    }

    /// Returns the stable adapter-state label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => ADAPTER_STATE_AVAILABLE,
            Self::Unavailable => ADAPTER_STATE_UNAVAILABLE,
            Self::PartiallyEnforceable => ADAPTER_STATE_PARTIAL,
        }
    }
}

/// Validated workspace adapter capability advertisement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capabilities {
    adapter_label: alloc::string::String,
    kind: Kind,
    state: State,
    supported: alloc::vec::Vec<Capability>,
}

impl Capabilities {
    /// Validates a workspace adapter capability advertisement.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when fields are empty, malformed,
    /// unknown, or the supported capability set is empty.
    pub fn new(draft: CapabilitiesDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        crate::validation::validate_required_adapter_field(ADAPTER_LABEL_FIELD, draft.adapter_label)?;
        let kind = Kind::parse(draft.kind)?;
        let state = State::parse(draft.state)?;
        let advertised = super::text::collect_capabilities(draft.supported_capabilities, |value| {
            Capability::parse_for_field(SUPPORTED_CAPABILITY_FIELD, value)
        })?;
        if advertised.is_empty() {
            return Err(crate::AdapterContractError::EmptyCollection {
                field: SUPPORTED_CAPABILITY_FIELD.as_str(),
            });
        }

        Ok(Self {
            adapter_label: alloc::string::String::from(draft.adapter_label),
            kind,
            state,
            supported: advertised,
        })
    }

    /// Returns the adapter label.
    #[must_use]
    pub fn adapter_label(&self) -> &str {
        self.adapter_label.as_str()
    }

    /// Returns the adapter kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Returns the adapter availability state.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// Returns advertised capabilities.
    #[must_use]
    pub fn supported_capabilities(&self) -> &[Capability] {
        self.supported.as_slice()
    }

    /// Returns whether this adapter advertises `capability`.
    #[must_use]
    pub fn supports(&self, capability: Capability) -> bool {
        self.supported.contains(&capability)
    }
}

/// Borrowed workspace adapter capability input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilitiesDraft<'a> {
    /// Adapter label for receipts and diagnostics.
    pub adapter_label: &'a str,
    /// Backend kind label.
    pub kind: &'a str,
    /// Availability state label.
    pub state: &'a str,
    /// Advertised capability labels.
    pub supported_capabilities: &'a [&'a str],
}

const REQUIRED_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_required_capability");

const CAPABILITY_BRANCHFS_LIKE: &str = "branchfs_like";
const CAPABILITY_JUJUTSU_WORKSPACE: &str = "jujutsu_workspace";
const CAPABILITY_VCS_WORKTREE: &str = "vcs_worktree";
const CAPABILITY_TEMPDIR_SNAPSHOT: &str = "tempdir_snapshot";
const CAPABILITY_NESTED_WORKSPACES: &str = "nested_workspaces";
const CAPABILITY_COMMIT_TO_PARENT: &str = "commit_to_parent";
const CAPABILITY_ABORT: &str = "abort";
const CAPABILITY_CONFLICT_DETECTION: &str = "conflict_detection";
const CAPABILITY_CLEANUP_GUARANTEE: &str = "cleanup_guarantee";

/// Capability class for speculative workspace adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// BranchFS-like copy-on-write branch behavior.
    BranchFsLike,
    /// Jujutsu workspace behavior without requiring Git worktrees.
    JujutsuWorkspace,
    /// Generic VCS worktree behavior.
    VcsWorktree,
    /// Temporary snapshot directory behavior.
    TempdirSnapshot,
    /// Nested workspace parentage.
    NestedWorkspaces,
    /// Commit-to-parent behavior.
    CommitToParent,
    /// Abort or discard behavior.
    Abort,
    /// Conflict detection before commit.
    ConflictDetection,
    /// Cleanup outcome guarantee.
    CleanupGuarantee,
}

impl Capability {
    /// Parses a workspace capability label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(REQUIRED_CAPABILITY_FIELD, value, Self::from_normalized)
    }

    fn parse_for_field(
        field_name: crate::validation::FieldName,
        value: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(field_name, value, Self::from_normalized)
    }

    fn from_normalized(normalized: &str) -> Option<Self> {
        match normalized {
            CAPABILITY_BRANCHFS_LIKE => Some(Self::BranchFsLike),
            CAPABILITY_JUJUTSU_WORKSPACE => Some(Self::JujutsuWorkspace),
            CAPABILITY_VCS_WORKTREE => Some(Self::VcsWorktree),
            CAPABILITY_TEMPDIR_SNAPSHOT => Some(Self::TempdirSnapshot),
            CAPABILITY_NESTED_WORKSPACES => Some(Self::NestedWorkspaces),
            CAPABILITY_COMMIT_TO_PARENT => Some(Self::CommitToParent),
            CAPABILITY_ABORT => Some(Self::Abort),
            CAPABILITY_CONFLICT_DETECTION => Some(Self::ConflictDetection),
            CAPABILITY_CLEANUP_GUARANTEE => Some(Self::CleanupGuarantee),
            _ => None,
        }
    }

    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BranchFsLike => CAPABILITY_BRANCHFS_LIKE,
            Self::JujutsuWorkspace => CAPABILITY_JUJUTSU_WORKSPACE,
            Self::VcsWorktree => CAPABILITY_VCS_WORKTREE,
            Self::TempdirSnapshot => CAPABILITY_TEMPDIR_SNAPSHOT,
            Self::NestedWorkspaces => CAPABILITY_NESTED_WORKSPACES,
            Self::CommitToParent => CAPABILITY_COMMIT_TO_PARENT,
            Self::Abort => CAPABILITY_ABORT,
            Self::ConflictDetection => CAPABILITY_CONFLICT_DETECTION,
            Self::CleanupGuarantee => CAPABILITY_CLEANUP_GUARANTEE,
        }
    }
}
