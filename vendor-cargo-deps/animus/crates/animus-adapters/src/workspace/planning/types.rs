/// Workspace planning status before host adapter execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// The workspace request is safe to hand to a host-owned adapter.
    Ready,
    /// The workspace request must fail closed before model, tool, or process dispatch.
    Denied,
}

impl Status {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Denied => "denied",
        }
    }
}

/// Pure workspace planning issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Issue {
    /// No issue was found.
    None,
    /// The request did not name a descriptor in the graph.
    MissingWorkspace,
    /// Two descriptors used the same workspace identity.
    DuplicateWorkspace,
    /// A descriptor named a parent absent from the graph, or a parent commit lacked a parent.
    MissingParent,
    /// Parent links form a cycle.
    ParentCycle,
    /// A commit operation targeted a workspace with child workspaces.
    NonLeafCommit,
    /// An abort operation targeted a workspace with child workspaces.
    NonLeafAbort,
    /// The selected adapter is unavailable.
    AdapterUnavailable,
    /// The selected adapter reports only partial enforcement.
    AdapterPartiallyEnforceable,
    /// The selected adapter is an unsupported placeholder.
    AdapterUnsupported,
    /// The selected adapter lacks one or more required capabilities.
    UnsupportedCapability,
}

impl Issue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::MissingWorkspace => "missing_workspace",
            Self::DuplicateWorkspace => "duplicate_workspace",
            Self::MissingParent => "missing_parent",
            Self::ParentCycle => "parent_cycle",
            Self::NonLeafCommit => "non_leaf_commit",
            Self::NonLeafAbort => "non_leaf_abort",
            Self::AdapterUnavailable => "adapter_unavailable",
            Self::AdapterPartiallyEnforceable => "adapter_partially_enforceable",
            Self::AdapterUnsupported => "adapter_unsupported",
            Self::UnsupportedCapability => "unsupported_capability",
        }
    }
}

/// Pure planning request over already validated descriptor and adapter DTOs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request<'a> {
    /// Candidate workspace descriptor graph.
    pub descriptors: &'a [crate::workspace::intent::Descriptor],
    /// Workspace identity selected for the requested operation.
    pub selected_workspace_id: &'a str,
    /// Host-advertised adapter capability DTO.
    pub adapter: &'a crate::workspace::adapter::Capabilities,
}
