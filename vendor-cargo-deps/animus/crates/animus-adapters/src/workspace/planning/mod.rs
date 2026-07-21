mod graph;
pub(super) mod hash;
mod types;

pub use types::{Issue, Request, Status};

/// Pure workspace plan safe to inspect before host adapter execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    status: Status,
    issue: Issue,
    selected_workspace_id: alloc::string::String,
    parent_workspace_id: Option<alloc::string::String>,
    descriptor_hash: alloc::string::String,
    source_hash: alloc::string::String,
    operation_hash: alloc::string::String,
    required_capabilities: alloc::vec::Vec<super::adapter::Capability>,
    unsupported_capabilities: alloc::vec::Vec<super::adapter::Capability>,
}

impl Plan {
    fn denied(request: Request<'_>, issue: Issue) -> Self {
        let selected = graph::find_descriptor(request.descriptors, request.selected_workspace_id);
        Self::from_parts(
            request.selected_workspace_id,
            selected,
            issue,
            alloc::vec::Vec::new(),
        )
    }

    fn unsupported(
        request: Request<'_>,
        selected: &super::intent::Descriptor,
        capability_gaps: alloc::vec::Vec<super::adapter::Capability>,
    ) -> Self {
        Self::from_parts(
            request.selected_workspace_id,
            Some(selected),
            Issue::UnsupportedCapability,
            capability_gaps,
        )
    }

    fn ready(request: Request<'_>, selected: &super::intent::Descriptor) -> Self {
        Self::from_parts(
            request.selected_workspace_id,
            Some(selected),
            Issue::None,
            alloc::vec::Vec::new(),
        )
    }

    fn from_parts(
        selected_workspace_id: &str,
        selected: Option<&super::intent::Descriptor>,
        issue: Issue,
        capability_gaps: alloc::vec::Vec<super::adapter::Capability>,
    ) -> Self {
        let status = if matches!(issue, Issue::None) {
            Status::Ready
        } else {
            Status::Denied
        };
        let selected_workspace_id = alloc::string::String::from(selected_workspace_id);
        let parent_workspace_id = selected
            .and_then(super::intent::Descriptor::parent_workspace_id)
            .map(alloc::string::String::from);
        let descriptor_hash = selected.map_or_else(
            || hash::parts(&[hash::SCHEMA, "missing", selected_workspace_id.as_str()]),
            hash::descriptor,
        );
        let source_hash = selected.map_or_else(
            || {
                hash::parts(&[
                    hash::SCHEMA,
                    "missing-source",
                    selected_workspace_id.as_str(),
                ])
            },
            |descriptor| hash::parts(&[hash::SCHEMA, "source", descriptor.source_id()]),
        );
        let operation_hash = selected.map_or_else(
            || {
                hash::parts(&[
                    hash::SCHEMA,
                    "missing-operation",
                    selected_workspace_id.as_str(),
                ])
            },
            hash::operation,
        );
        let required_capabilities = selected.map_or_else(alloc::vec::Vec::new, |descriptor| {
            descriptor.required_capabilities().to_vec()
        });
        Self {
            status,
            issue,
            selected_workspace_id,
            parent_workspace_id,
            descriptor_hash,
            source_hash,
            operation_hash,
            required_capabilities,
            unsupported_capabilities: capability_gaps,
        }
    }

    /// Returns the status.
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }

    /// Returns the first blocking issue, or [`Issue::None`].
    #[must_use]
    pub const fn issue(&self) -> Issue {
        self.issue
    }

    /// Returns whether dispatch can continue to the host-owned workspace adapter.
    #[must_use]
    pub const fn dispatch_allowed(&self) -> bool {
        matches!(self.status, Status::Ready)
    }

    /// Returns the selected workspace identity.
    #[must_use]
    pub fn selected_workspace_id(&self) -> &str {
        self.selected_workspace_id.as_str()
    }

    /// Returns the parent workspace identity, if known.
    #[must_use]
    pub fn parent_workspace_id(&self) -> Option<&str> {
        self.parent_workspace_id.as_deref()
    }

    /// Returns the BLAKE3 hash of the normalized selected descriptor.
    #[must_use]
    pub fn descriptor_hash(&self) -> &str {
        self.descriptor_hash.as_str()
    }

    /// Returns the BLAKE3 hash of the source identity label.
    #[must_use]
    pub fn source_hash(&self) -> &str {
        self.source_hash.as_str()
    }

    /// Returns the BLAKE3 hash of the requested operation policy.
    #[must_use]
    pub fn operation_hash(&self) -> &str {
        self.operation_hash.as_str()
    }

    /// Returns required adapter capabilities from the selected descriptor.
    #[must_use]
    pub fn required_capabilities(&self) -> &[super::adapter::Capability] {
        self.required_capabilities.as_slice()
    }

    /// Returns capabilities missing from the selected adapter.
    #[must_use]
    pub fn unsupported_capabilities(&self) -> &[super::adapter::Capability] {
        self.unsupported_capabilities.as_slice()
    }
}

/// Builds a pure workspace plan and fails closed before host execution on invalid input.
#[must_use]
pub fn evaluate(request: Request<'_>) -> Plan {
    debug_assert!(!request.selected_workspace_id.is_empty());
    debug_assert!(
        request
            .descriptors
            .iter()
            .all(|descriptor| !descriptor.workspace_id().is_empty())
    );
    if let Some(issue) = graph_issue(request) {
        return Plan::denied(request, issue);
    }
    let Some(selected) = graph::find_descriptor(request.descriptors, request.selected_workspace_id)
    else {
        return Plan::denied(request, Issue::MissingWorkspace);
    };
    if let Some(issue) = adapter_issue(request.adapter) {
        return Plan::denied(request, issue);
    }
    if let Some(issue) = operation_issue(request, selected) {
        return Plan::denied(request, issue);
    }
    let capability_gaps = graph::missing_capabilities(selected, request.adapter);
    if !capability_gaps.is_empty() {
        return Plan::unsupported(request, selected, capability_gaps);
    }
    Plan::ready(request, selected)
}

fn graph_issue(request: Request<'_>) -> Option<Issue> {
    if graph::has_duplicate_identity(request.descriptors) {
        return Some(Issue::DuplicateWorkspace);
    }
    if graph::has_missing_parent(request.descriptors) {
        return Some(Issue::MissingParent);
    }
    graph::has_parent_cycle(request.descriptors).then_some(Issue::ParentCycle)
}

fn adapter_issue(adapter: &super::adapter::Capabilities) -> Option<Issue> {
    if matches!(adapter.kind(), super::adapter::Kind::Unsupported) {
        return Some(Issue::AdapterUnsupported);
    }
    match adapter.state() {
        super::adapter::State::Available => None,
        super::adapter::State::Unavailable => Some(Issue::AdapterUnavailable),
        super::adapter::State::PartiallyEnforceable => Some(Issue::AdapterPartiallyEnforceable),
    }
}

fn operation_issue(request: Request<'_>, selected: &super::intent::Descriptor) -> Option<Issue> {
    if selected.operation() == super::rules::Operation::Commit {
        if selected.parent_workspace_id().is_none()
            && selected.commit_target() == super::rules::CommitTarget::Parent
        {
            return Some(Issue::MissingParent);
        }
        if graph::has_child(request.descriptors, selected.workspace_id()) {
            return Some(Issue::NonLeafCommit);
        }
    }
    if selected.operation() == super::rules::Operation::Abort
        && graph::has_child(request.descriptors, selected.workspace_id())
    {
        return Some(Issue::NonLeafAbort);
    }
    None
}
