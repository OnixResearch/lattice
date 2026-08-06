const LOCAL_CONFIG_LABEL: &str = "local-config:dev-shell";
const SECRET_LABEL: &str = "secret:api-token";
const PUBLIC_LABEL: &str = "fixture:readme";
const PROVENANCE: &str = "operator-allowlist";
const REDACTION_LOCAL_CONFIG: &str = "local_config";
const REDACTION_SECRET: &str = "secret";
const REDACTION_PUBLIC: &str = "public";
const APPROVED: &str = "approved";
const DENIED: &str = "denied";
const APPROVED_ENTRY_COUNT: usize = 1;

#[derive(Clone, Copy)]
struct AllowlistFixture {
    file_label: &'static str,
    redaction_class: &'static str,
    approval: &'static str,
}

#[test]
fn allowlisted_ignored_file_materializes_with_safe_provenance() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.ignored_file_include]
    let entry = allowlist_entry(AllowlistFixture {
        file_label: LOCAL_CONFIG_LABEL,
        redaction_class: REDACTION_LOCAL_CONFIG,
        approval: APPROVED,
    });
    let decision =
        crate::plan_workspace_ignored_file_materialization(crate::WorkspaceIgnoredFileMaterializationRequest {
            file_label: LOCAL_CONFIG_LABEL,
            classification: crate::WorkspaceIgnoredFileRedactionClass::LocalConfig,
            allowlist: alloc::slice::from_ref(&entry),
        })
        .expect("safe allowlisted materialization should plan");

    assert_eq!(decision.file_label(), LOCAL_CONFIG_LABEL);
    assert_eq!(decision.status(), crate::WorkspaceIgnoredFileMaterializationStatus::Materialized);
    assert_eq!(decision.denial_reason(), crate::WorkspaceIgnoredFileDenialReason::None);
    assert_eq!(decision.provenance(), Some(PROVENANCE));
    assert_eq!(alloc::slice::from_ref(&entry).len(), APPROVED_ENTRY_COUNT);
}

#[test]
fn ignored_secret_is_not_copied_without_allowlist_and_redaction_approval() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.ignored_file_include.secrets]
    let missing_allowlist =
        crate::plan_workspace_ignored_file_materialization(crate::WorkspaceIgnoredFileMaterializationRequest {
            file_label: SECRET_LABEL,
            classification: crate::WorkspaceIgnoredFileRedactionClass::Secret,
            allowlist: &[],
        })
        .expect("missing allowlist should produce copy-denied evidence");
    let denied_secret = allowlist_entry(AllowlistFixture {
        file_label: SECRET_LABEL,
        redaction_class: REDACTION_SECRET,
        approval: DENIED,
    });
    let secret_without_approval =
        crate::plan_workspace_ignored_file_materialization(crate::WorkspaceIgnoredFileMaterializationRequest {
            file_label: SECRET_LABEL,
            classification: crate::WorkspaceIgnoredFileRedactionClass::Secret,
            allowlist: &[denied_secret],
        })
        .expect("denied secret should produce copy-denied evidence");

    assert_eq!(missing_allowlist.status(), crate::WorkspaceIgnoredFileMaterializationStatus::CopyDenied);
    assert_eq!(missing_allowlist.denial_reason(), crate::WorkspaceIgnoredFileDenialReason::MissingAllowlist);
    assert_eq!(secret_without_approval.status(), crate::WorkspaceIgnoredFileMaterializationStatus::CopyDenied);
    assert_eq!(
        secret_without_approval.denial_reason(),
        crate::WorkspaceIgnoredFileDenialReason::SecretWithoutApproval
    );
    assert!(secret_without_approval.provenance().is_none());
}

#[test]
fn mismatched_redaction_class_denies_copy() {
    let public_entry = allowlist_entry(AllowlistFixture {
        file_label: PUBLIC_LABEL,
        redaction_class: REDACTION_PUBLIC,
        approval: APPROVED,
    });
    let decision =
        crate::plan_workspace_ignored_file_materialization(crate::WorkspaceIgnoredFileMaterializationRequest {
            file_label: PUBLIC_LABEL,
            classification: crate::WorkspaceIgnoredFileRedactionClass::Secret,
            allowlist: &[public_entry],
        })
        .expect("redaction mismatch should still return safe decision evidence");

    assert_eq!(decision.denial_reason(), crate::WorkspaceIgnoredFileDenialReason::RedactionRejected);
}

fn allowlist_entry(fixture: AllowlistFixture) -> crate::WorkspaceIgnoredFileAllowlistEntry {
    crate::WorkspaceIgnoredFileAllowlistEntry::new(crate::WorkspaceIgnoredFileAllowlistEntryDraft {
        file_label: fixture.file_label,
        provenance: PROVENANCE,
        redaction_class: fixture.redaction_class,
        approval: fixture.approval,
    })
    .expect("allowlist entry fixture should parse")
}
