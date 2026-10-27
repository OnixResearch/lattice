use super::*;

const CHECKPOINT_ID: &str = "checkpoint-001";
const WORKSPACE_LABEL: &str = "workspace-main";
const TOOL_LABEL: &str = "edit";
const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const CHANGED_FILE_COUNT: u32 = 2;

fn hash(value: &str) -> CheckpointHash {
    CheckpointHash::new(value).expect("hash should validate")
}

fn draft(backend_kind: &'static str) -> CheckpointDraft<'static> {
    CheckpointDraft {
        backend_kind,
        checkpoint_id: CHECKPOINT_ID,
        workspace_label: WORKSPACE_LABEL,
        workspace_hash: HASH_A,
        before_hash: HASH_B,
        after_hash: HASH_C,
        changed_file_count: CHANGED_FILE_COUNT,
        session_hash: Some(HASH_D),
        tool_label: Some(TOOL_LABEL),
        status: "created",
    }
}

fn record() -> CheckpointRecord {
    CheckpointRecord::new(draft("jj")).expect("checkpoint should validate")
}

#[test]
fn dto_accepts_backend_neutral_checkpoint_identities() {
    let cases = [
        ("jj", CheckpointBackendKind::Jj),
        ("git", CheckpointBackendKind::Git),
        ("in_memory", CheckpointBackendKind::InMemory),
        (
            "workspace_isolation",
            CheckpointBackendKind::WorkspaceIsolation,
        ),
    ];
    for (label, expected_kind) in cases {
        let checkpoint = CheckpointRecord::new(draft(label)).expect("checkpoint should validate");
        assert_eq!(checkpoint.backend_kind(), expected_kind);
        assert_eq!(checkpoint.checkpoint_id(), CHECKPOINT_ID);
        assert_eq!(checkpoint.workspace_label(), WORKSPACE_LABEL);
        assert_eq!(checkpoint.workspace_hash().as_str(), HASH_A);
        assert_eq!(checkpoint.before_hash().as_str(), HASH_B);
        assert_eq!(checkpoint.after_hash().as_str(), HASH_C);
        assert_eq!(checkpoint.changed_file_count(), CHANGED_FILE_COUNT);
        assert_eq!(
            checkpoint.session_hash().map(CheckpointHash::as_str),
            Some(HASH_D)
        );
        assert_eq!(checkpoint.tool_label(), Some(TOOL_LABEL));
        assert_eq!(checkpoint.status(), CheckpointStatus::Created);
    }
}

#[test]
fn dto_rejects_unknown_backend_and_invalid_metadata() {
    let unknown_backend_diagnostic =
        CheckpointRecord::new(draft("svn")).expect_err("unknown backend should fail");
    assert!(
        matches!(unknown_backend_diagnostic, CheckpointError::UnsupportedValue { field, .. } if field == CHECKPOINT_BACKEND_FIELD.as_str())
    );
    let too_many_files_diagnostic = CheckpointRecord::new(CheckpointDraft {
        changed_file_count: MAX_CHANGED_FILE_COUNT.saturating_add(1),
        ..draft("git")
    })
    .expect_err("oversized changed-file count should fail");
    assert!(matches!(
        too_many_files_diagnostic,
        CheckpointError::ChangedFileCountTooLarge { .. }
    ));
    let bad_hash_diagnostic = CheckpointRecord::new(CheckpointDraft {
        workspace_hash: "abc",
        ..draft("git")
    })
    .expect_err("short hash should fail");
    assert!(
        matches!(bad_hash_diagnostic, CheckpointError::AdapterContract(crate::AdapterContractError::InvalidEvidenceHash { field, .. }) if field == CHECKPOINT_WORKSPACE_HASH_FIELD.as_str())
    );
}

#[test]
fn strict_protected_mutation_requires_injected_service() {
    let missing = evaluate_protected_mutation(ProtectedMutationInput {
        policy: CheckpointPolicy::Strict,
        protection: MutationProtection::Protected,
        service_state: CheckpointServiceState::Missing,
    });
    assert!(!missing.dispatch_allowed());
    assert!(!missing.checkpoint_required());
    assert_eq!(
        missing.failure_class(),
        CheckpointFailureClass::MissingService
    );
    let available = evaluate_protected_mutation(ProtectedMutationInput {
        policy: CheckpointPolicy::Strict,
        protection: MutationProtection::Protected,
        service_state: CheckpointServiceState::Injected,
    });
    assert!(available.dispatch_allowed());
    assert!(available.checkpoint_required());
    assert_eq!(available.failure_class(), CheckpointFailureClass::None);
}

#[test]
fn best_effort_records_checkpoint_failure_without_blocking_dispatch() {
    let decision = evaluate_protected_mutation(ProtectedMutationInput {
        policy: CheckpointPolicy::BestEffort,
        protection: MutationProtection::Protected,
        service_state: CheckpointServiceState::Unavailable,
    });
    assert!(decision.dispatch_allowed());
    assert!(!decision.checkpoint_required());
    assert_eq!(
        decision.failure_class(),
        CheckpointFailureClass::Unavailable
    );
}

#[test]
fn rollback_requires_confirmation_before_restore() {
    let checkpoint = record();
    let expected = hash(HASH_C);
    let validation = validate_rollback(RollbackValidationInput {
        target: &checkpoint,
        confirmation: RollbackConfirmationStatus::Missing,
        expected_current_hash: &expected,
        observed_current_hash: &expected,
    });
    assert!(!validation.restore_allowed());
    assert_eq!(validation.status(), CheckpointStatus::Blocked);
    assert_eq!(
        validation.failure_class(),
        CheckpointFailureClass::UnconfirmedRollback
    );
}

#[test]
fn rollback_rejects_stale_workspace_before_restore() {
    let checkpoint = record();
    let expected = hash(HASH_C);
    let observed = hash(HASH_D);
    let validation = validate_rollback(RollbackValidationInput {
        target: &checkpoint,
        confirmation: RollbackConfirmationStatus::Approved,
        expected_current_hash: &expected,
        observed_current_hash: &observed,
    });
    assert!(!validation.restore_allowed());
    assert_eq!(validation.status(), CheckpointStatus::Blocked);
    assert_eq!(
        validation.failure_class(),
        CheckpointFailureClass::StaleWorkspace
    );
}

#[test]
fn rollback_allows_confirmed_fresh_target() {
    let checkpoint = record();
    let expected = hash(HASH_C);
    let validation = validate_rollback(RollbackValidationInput {
        target: &checkpoint,
        confirmation: RollbackConfirmationStatus::Approved,
        expected_current_hash: &expected,
        observed_current_hash: &expected,
    });
    assert!(validation.restore_allowed());
    assert_eq!(validation.status(), CheckpointStatus::RolledBack);
    assert_eq!(validation.failure_class(), CheckpointFailureClass::None);
}

#[test]
fn receipt_records_safe_metadata_and_stable_hash() {
    let workspace_hash = hash(HASH_A);
    let before_hash = hash(HASH_B);
    let after_hash = hash(HASH_C);
    let draft = CheckpointReceiptDraft {
        operation: CheckpointOperation::Rollback,
        status: CheckpointStatus::RolledBack,
        backend_kind: CheckpointBackendKind::Jj,
        checkpoint_id: CHECKPOINT_ID,
        workspace_hash: &workspace_hash,
        before_hash: Some(&before_hash),
        after_hash: Some(&after_hash),
        changed_file_count: CHANGED_FILE_COUNT,
        confirmation: RollbackConfirmationStatus::Approved,
        failure_class: CheckpointFailureClass::None,
    };
    let receipt = CheckpointReceipt::new(draft).expect("receipt should validate");
    let repeated = CheckpointReceipt::new(draft).expect("receipt should validate again");
    assert_eq!(receipt.operation(), CheckpointOperation::Rollback);
    assert_eq!(receipt.status(), CheckpointStatus::RolledBack);
    assert_eq!(receipt.backend_kind(), CheckpointBackendKind::Jj);
    assert_eq!(receipt.checkpoint_id(), CHECKPOINT_ID);
    assert_eq!(receipt.workspace_hash().as_str(), HASH_A);
    assert_eq!(
        receipt.before_hash().map(CheckpointHash::as_str),
        Some(HASH_B)
    );
    assert_eq!(
        receipt.after_hash().map(CheckpointHash::as_str),
        Some(HASH_C)
    );
    assert_eq!(receipt.changed_file_count(), CHANGED_FILE_COUNT);
    assert_eq!(receipt.confirmation(), RollbackConfirmationStatus::Approved);
    assert_eq!(receipt.failure_class(), CheckpointFailureClass::None);
    assert_eq!(receipt.receipt_hash(), repeated.receipt_hash());
}

#[test]
fn receipt_rejects_leak_markers() {
    assert_eq!(
        validate_checkpoint_receipt_text("operation=rollback status=blocked"),
        Ok(())
    );
    let cases = [
        ("RAW_DIFF:\n-old\n+new", CheckpointReceiptLeak::RawDiff),
        (
            "RAW_FILE: complete source",
            CheckpointReceiptLeak::RawFileBody,
        ),
        ("API_KEY=secret", CheckpointReceiptLeak::Credential),
        (
            "ENV_DUMP:\nTOKEN=value",
            CheckpointReceiptLeak::EnvironmentDump,
        ),
        (
            "files: /home/alice/project/src/lib.rs",
            CheckpointReceiptLeak::FullPathList,
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(validate_checkpoint_receipt_text(text), Err(expected));
        assert!(!expected.as_str().is_empty());
    }
}
