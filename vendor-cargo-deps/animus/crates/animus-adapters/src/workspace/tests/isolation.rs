#![allow(unknown_lints, path_segment_repetition)]

use super::support::*;

const SNAPSHOT_ID: &str = "snap:child-001";
const MANIFEST_HASH: &str = "blake3:workspace-manifest";
const RESTORE_HINT: &str = "restore via host adapter snapshot";
const KIND_JUJUTSU: &str = "jujutsu_workspace";
const CAP_JUJUTSU: &str = "jujutsu_workspace";
const CAP_VCS_WORKTREE: &str = "vcs_worktree";
const ADAPTER_LABEL: &str = "jj-workspace-adapter";
const STATE_AVAILABLE: &str = "available";
const EXPECTED_WRITABLE_ROOT_COUNT: usize = 2;

#[test]
fn isolated_workspace_accepts_jujutsu_capabilities_without_git_worktree() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.isolated_workspace]
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.isolated_workspace.vcs_neutral]
    let descriptor = descriptor(WORKSPACE_ID, Some(PARENT_ID), OP_CREATE, &[CAP_JUJUTSU]);
    let restore_metadata = restore_metadata();
    let adapter = jujutsu_adapter(&[CAP_JUJUTSU, CAP_NESTED, CAP_COMMIT, CAP_CLEANUP]);

    let workspace = crate::IsolatedTaskWorkspace::new(crate::IsolatedTaskWorkspaceDraft {
        descriptor: &descriptor,
        snapshot_id: SNAPSHOT_ID,
        restore_metadata: &restore_metadata,
        adapter: &adapter,
    })
    .expect("Jujutsu-oriented adapter should not require Git worktree capability");

    assert_eq!(workspace.source_id(), SOURCE_ID);
    assert_eq!(workspace.workspace_id(), WORKSPACE_ID);
    assert_eq!(workspace.snapshot_id(), SNAPSHOT_ID);
    assert_eq!(
        workspace.adapter_kind(),
        crate::WorkspaceAdapterKind::JujutsuWorkspace
    );
    assert!(
        !workspace
            .adapter_capabilities()
            .contains(&crate::WorkspaceCapability::VcsWorktree)
    );
    assert_eq!(
        workspace.writable_roots().len(),
        EXPECTED_WRITABLE_ROOT_COUNT
    );
    assert_eq!(workspace.restore_metadata().manifest_hash(), MANIFEST_HASH);
}

#[test]
fn isolated_workspace_rejects_git_only_requirement_when_adapter_lacks_it() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.isolated_workspace.vcs_neutral]
    let descriptor = descriptor(
        WORKSPACE_ID,
        Some(PARENT_ID),
        OP_CREATE,
        &[CAP_VCS_WORKTREE],
    );
    let restore_metadata = restore_metadata();
    let adapter = jujutsu_adapter(&[CAP_JUJUTSU, CAP_NESTED, CAP_CLEANUP]);
    assert!(!adapter.supports(crate::WorkspaceCapability::VcsWorktree));

    let error = crate::IsolatedTaskWorkspace::new(crate::IsolatedTaskWorkspaceDraft {
        descriptor: &descriptor,
        snapshot_id: SNAPSHOT_ID,
        restore_metadata: &restore_metadata,
        adapter: &adapter,
    })
    .expect_err("Git-only capability must not be assumed for Jujutsu workspaces");

    assert_eq!(
        error,
        crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: "workspace_required_capability",
            value: alloc::string::String::from(CAP_VCS_WORKTREE),
        }
    );
}

#[test]
fn isolated_workspace_rejects_mismatched_restore_snapshot() {
    let descriptor = descriptor(WORKSPACE_ID, Some(PARENT_ID), OP_CREATE, &[CAP_JUJUTSU]);
    let restore_metadata =
        crate::WorkspaceRestoreMetadata::new(crate::WorkspaceRestoreMetadataDraft {
            snapshot_id: "snap:other",
            manifest_hash: MANIFEST_HASH,
            restore_hint: RESTORE_HINT,
        })
        .expect("restore metadata fixture should parse");
    let adapter = jujutsu_adapter(&[CAP_JUJUTSU, CAP_NESTED, CAP_CLEANUP]);
    assert_ne!(restore_metadata.snapshot_id(), SNAPSHOT_ID);

    let error = crate::IsolatedTaskWorkspace::new(crate::IsolatedTaskWorkspaceDraft {
        descriptor: &descriptor,
        snapshot_id: SNAPSHOT_ID,
        restore_metadata: &restore_metadata,
        adapter: &adapter,
    })
    .expect_err("snapshot mismatch should fail closed");

    assert!(matches!(
        error,
        crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: "workspace_snapshot_id",
            ..
        }
    ));
}

fn restore_metadata() -> crate::WorkspaceRestoreMetadata {
    crate::WorkspaceRestoreMetadata::new(crate::WorkspaceRestoreMetadataDraft {
        snapshot_id: SNAPSHOT_ID,
        manifest_hash: MANIFEST_HASH,
        restore_hint: RESTORE_HINT,
    })
    .expect("restore metadata fixture should parse")
}

fn jujutsu_adapter(
    supported_capabilities: &'static [&'static str],
) -> crate::WorkspaceAdapterCapabilities {
    crate::WorkspaceAdapterCapabilities::new(crate::WorkspaceAdapterCapabilitiesDraft {
        adapter_label: ADAPTER_LABEL,
        kind: KIND_JUJUTSU,
        state: STATE_AVAILABLE,
        supported_capabilities,
    })
    .expect("Jujutsu adapter fixture should parse")
}
