mod automation;
mod handoff;
mod inclusion;
mod isolation;
mod planning;
mod receipt;
mod support;

const SOURCE_ID: &str = "repo:animus";
const WORKSPACE_ID: &str = "ws:child";
const PARENT_ID: &str = "ws:parent";
const ROOT_SRC: &str = "src";
const ROOT_DOCS: &str = "docs";
const ADAPTER_LABEL: &str = "branchfs-reference";
const BAD_VALUE: &str = "mystery";
const NUL_WORKSPACE: &str = "ws\0bad";
const NUL_BYTE_INDEX: usize = 2;

fn descriptor_draft() -> crate::SpeculativeWorkspaceDescriptorDraft<'static> {
    crate::SpeculativeWorkspaceDescriptorDraft {
        source_id: SOURCE_ID,
        workspace_id: WORKSPACE_ID,
        parent_workspace_id: Some(PARENT_ID),
        writable_roots: &[ROOT_SRC, ROOT_DOCS],
        file_inclusion: "tracked_only",
        operation: "create",
        commit_target: "parent",
        abort_policy: "discard",
        conflict_policy: "fail",
        cleanup_policy: "required",
        required_capabilities: &["nested_workspaces", "commit_to_parent"],
    }
}

#[test]
fn descriptor_accepts_valid_portable_intent() {
    let descriptor = crate::SpeculativeWorkspaceDescriptor::new(descriptor_draft())
        .expect("valid workspace descriptor should parse");

    assert_eq!(descriptor.source_id(), SOURCE_ID);
    assert_eq!(descriptor.workspace_id(), WORKSPACE_ID);
    assert_eq!(descriptor.parent_workspace_id(), Some(PARENT_ID));
    assert_eq!(
        descriptor.writable_roots(),
        &[
            alloc::string::String::from(ROOT_SRC),
            alloc::string::String::from(ROOT_DOCS)
        ]
    );
    assert_eq!(
        descriptor.file_inclusion(),
        crate::WorkspaceFileInclusionPolicy::TrackedOnly
    );
    assert_eq!(descriptor.operation(), crate::WorkspaceOperation::Create);
    assert_eq!(
        descriptor.commit_target(),
        crate::WorkspaceCommitTarget::Parent
    );
    assert_eq!(
        descriptor.abort_policy(),
        crate::WorkspaceAbortPolicy::Discard
    );
    assert_eq!(
        descriptor.conflict_policy(),
        crate::WorkspaceConflictPolicy::Fail
    );
    assert_eq!(
        descriptor.cleanup_policy(),
        crate::WorkspaceCleanupPolicy::Required
    );
    assert_eq!(
        descriptor.required_capabilities(),
        &[
            crate::WorkspaceCapability::NestedWorkspaces,
            crate::WorkspaceCapability::CommitToParent
        ]
    );
}

#[test]
fn descriptor_rejects_empty_writable_roots() {
    let mut draft = descriptor_draft();
    draft.writable_roots = &[];

    let error = crate::SpeculativeWorkspaceDescriptor::new(draft)
        .expect_err("empty writable roots should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::EmptyCollection {
            field: "workspace_writable_roots",
        }
    );
}

#[test]
fn descriptor_rejects_unknown_capability() {
    let mut draft = descriptor_draft();
    draft.required_capabilities = &[BAD_VALUE];

    let error = crate::SpeculativeWorkspaceDescriptor::new(draft)
        .expect_err("unknown capability should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: "workspace_required_capability",
            value: alloc::string::String::from(BAD_VALUE),
        }
    );
}

#[test]
fn descriptor_rejects_malformed_id() {
    let mut draft = descriptor_draft();
    draft.workspace_id = NUL_WORKSPACE;

    let error = crate::SpeculativeWorkspaceDescriptor::new(draft)
        .expect_err("NUL workspace id should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::NulCharacter {
            field: "workspace_id",
            byte_index: NUL_BYTE_INDEX,
        }
    );
}

#[test]
fn adapter_capabilities_accept_branchfs_like_backend() {
    let capabilities =
        crate::WorkspaceAdapterCapabilities::new(crate::WorkspaceAdapterCapabilitiesDraft {
            adapter_label: ADAPTER_LABEL,
            kind: "branchfs_like",
            state: "available",
            supported_capabilities: &[
                "branchfs_like",
                "nested_workspaces",
                "commit_to_parent",
                "abort",
                "conflict_detection",
                "cleanup_guarantee",
            ],
        })
        .expect("valid adapter capabilities should parse");

    assert_eq!(capabilities.adapter_label(), ADAPTER_LABEL);
    assert_eq!(
        capabilities.kind(),
        crate::WorkspaceAdapterKind::BranchFsLike
    );
    assert_eq!(
        capabilities.state(),
        crate::WorkspaceAdapterState::Available
    );
    assert!(capabilities.supports(crate::WorkspaceCapability::CommitToParent));
    assert!(!capabilities.supports(crate::WorkspaceCapability::VcsWorktree));
}

#[test]
fn adapter_capabilities_reject_empty_supported_capabilities() {
    let error =
        crate::WorkspaceAdapterCapabilities::new(crate::WorkspaceAdapterCapabilitiesDraft {
            adapter_label: ADAPTER_LABEL,
            kind: "unsupported",
            state: "unavailable",
            supported_capabilities: &[],
        })
        .expect_err("empty capability set should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::EmptyCollection {
            field: "workspace_supported_capability",
        }
    );
}

#[test]
fn adapter_outcome_accepts_explicit_failure_state() {
    let outcome = crate::WorkspaceAdapterOutcome::new(crate::WorkspaceAdapterOutcomeDraft {
        kind: "failed_cleanup",
        workspace_id: WORKSPACE_ID,
        issue_code: Some("WORKSPACE_CLEANUP_FAILED"),
    })
    .expect("failed cleanup outcome should parse");

    assert_eq!(
        outcome.kind(),
        crate::WorkspaceAdapterOutcomeKind::FailedCleanup
    );
    assert_eq!(outcome.workspace_id(), WORKSPACE_ID);
    assert_eq!(outcome.issue_code(), Some("WORKSPACE_CLEANUP_FAILED"));
}

#[test]
fn adapter_outcome_rejects_unknown_state() {
    let error = crate::WorkspaceAdapterOutcome::new(crate::WorkspaceAdapterOutcomeDraft {
        kind: BAD_VALUE,
        workspace_id: WORKSPACE_ID,
        issue_code: None,
    })
    .expect_err("unknown outcome should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: "workspace_outcome_kind",
            value: alloc::string::String::from(BAD_VALUE),
        }
    );
}
