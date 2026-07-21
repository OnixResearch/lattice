pub(super) const SOURCE_ID: &str = "repo:animus";
pub(super) const WORKSPACE_ID: &str = "ws:child";
pub(super) const PARENT_ID: &str = "ws:parent";
pub(super) const ROOT_SRC: &str = "src";
pub(super) const ROOT_DOCS: &str = "docs";
pub(super) const ROOT_PARENT_ID: &str = "ws:root";
pub(super) const NO_CAPS: &[&str] = &[];
pub(super) const CAP_NESTED: &str = "nested_workspaces";
pub(super) const CAP_COMMIT: &str = "commit_to_parent";
pub(super) const CAP_ABORT: &str = "abort";
pub(super) const CAP_CONFLICT: &str = "conflict_detection";
pub(super) const CAP_CLEANUP: &str = "cleanup_guarantee";
pub(super) const CAP_BRANCHFS: &str = "branchfs_like";
pub(super) const OP_CREATE: &str = "create";
pub(super) const OP_COMMIT: &str = "commit";
pub(super) const OP_ABORT: &str = "abort";
pub(super) const SAFE_MANIFEST_LABEL: &str = "manifest:bounded-counts";
pub(super) const OUTCOME_COMMITTED: &str = "committed";
pub(super) const OUTCOME_ABORTED: &str = "aborted";

const ADAPTER_LABEL: &str = "branchfs-reference";
const KIND_BRANCHFS: &str = "branchfs_like";
const KIND_PLACEHOLDER: &str = "unsupported";
const STATE_AVAILABLE: &str = "available";
const STATE_UNAVAILABLE: &str = "unavailable";
const STATE_PARTIAL: &str = "partially_enforceable";

pub(super) fn descriptor(
    workspace_id: &'static str,
    parent_workspace_id: Option<&'static str>,
    operation: &'static str,
    required_capabilities: &'static [&'static str],
) -> crate::SpeculativeWorkspaceDescriptor {
    let draft = crate::SpeculativeWorkspaceDescriptorDraft {
        source_id: SOURCE_ID,
        workspace_id,
        parent_workspace_id,
        writable_roots: &[ROOT_SRC, ROOT_DOCS],
        file_inclusion: "tracked_only",
        operation,
        commit_target: "parent",
        abort_policy: "discard",
        conflict_policy: "fail",
        cleanup_policy: "required",
        required_capabilities,
    };
    crate::SpeculativeWorkspaceDescriptor::new(draft).expect("descriptor fixture should parse")
}

pub(super) fn branchfs_available(
    supported_capabilities: &'static [&'static str],
) -> crate::WorkspaceAdapterCapabilities {
    crate::WorkspaceAdapterCapabilities::new(crate::WorkspaceAdapterCapabilitiesDraft {
        adapter_label: ADAPTER_LABEL,
        kind: KIND_BRANCHFS,
        state: STATE_AVAILABLE,
        supported_capabilities,
    })
    .expect("available BranchFS-like fixture should parse")
}

pub(super) fn branchfs_unavailable() -> crate::WorkspaceAdapterCapabilities {
    crate::WorkspaceAdapterCapabilities::new(crate::WorkspaceAdapterCapabilitiesDraft {
        adapter_label: ADAPTER_LABEL,
        kind: KIND_BRANCHFS,
        state: STATE_UNAVAILABLE,
        supported_capabilities: &[CAP_BRANCHFS],
    })
    .expect("unavailable BranchFS-like fixture should parse")
}

pub(super) fn branchfs_partial() -> crate::WorkspaceAdapterCapabilities {
    crate::WorkspaceAdapterCapabilities::new(crate::WorkspaceAdapterCapabilitiesDraft {
        adapter_label: ADAPTER_LABEL,
        kind: KIND_BRANCHFS,
        state: STATE_PARTIAL,
        supported_capabilities: &[CAP_BRANCHFS],
    })
    .expect("partial BranchFS-like fixture should parse")
}

pub(super) fn placeholder_available() -> crate::WorkspaceAdapterCapabilities {
    crate::WorkspaceAdapterCapabilities::new(crate::WorkspaceAdapterCapabilitiesDraft {
        adapter_label: ADAPTER_LABEL,
        kind: KIND_PLACEHOLDER,
        state: STATE_AVAILABLE,
        supported_capabilities: &[CAP_BRANCHFS],
    })
    .expect("placeholder adapter fixture should parse")
}
