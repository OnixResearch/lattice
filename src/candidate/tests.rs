use super::*;

fn receipt() -> CandidateReceipt {
    CandidateReceipt {
        schema: CANDIDATE_RECEIPT_SCHEMA.to_string(),
        work_id: "work-1".to_string(),
        attempt_id: "attempt-1".to_string(),
        fence: "7".to_string(),
        workspace_id: "workspace-1".to_string(),
        base_identity: "a".repeat(64),
        candidate_identity: Some("b".repeat(64)),
        diff_identity: Some("c".repeat(64)),
        changed_paths: vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
        disposition: CandidateDisposition::Produced,
        blockers: Vec::new(),
    }
}

#[test]
fn produced_candidate_requires_complete_bindings() {
    let receipt = receipt();
    assert_eq!(receipt.validate(), Ok(()));
    assert!(receipt.is_usable_candidate());
}

#[test]
fn exit_zero_without_candidate_is_not_usable() {
    let mut receipt = receipt();
    receipt.candidate_identity = None;
    assert_eq!(receipt.validate(), Err(CandidateReceiptError::MissingCandidate));
    assert!(!receipt.is_usable_candidate());
}

#[test]
fn malformed_identities_and_paths_are_rejected() {
    let mut identity = receipt();
    identity.base_identity = "A".repeat(64);
    assert_eq!(identity.validate(), Err(CandidateReceiptError::InvalidIdentity));

    let mut absolute = receipt();
    absolute.changed_paths = vec!["/etc/passwd".to_string()];
    assert_eq!(absolute.validate(), Err(CandidateReceiptError::InvalidChangedPath));

    let mut escaping = receipt();
    escaping.changed_paths = vec!["src/../secret".to_string()];
    assert_eq!(escaping.validate(), Err(CandidateReceiptError::InvalidChangedPath));

    let mut unordered = receipt();
    unordered.changed_paths = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
    assert_eq!(unordered.validate(), Err(CandidateReceiptError::ChangedPathsNotCanonical));

    let mut duplicate = receipt();
    duplicate.changed_paths = vec!["src/lib.rs".to_string(), "src/lib.rs".to_string()];
    assert_eq!(duplicate.validate(), Err(CandidateReceiptError::ChangedPathsNotCanonical));
}

#[test]
fn non_success_dispositions_remain_precise() {
    let mut blocked = receipt();
    blocked.disposition = CandidateDisposition::Blocked;
    blocked.candidate_identity = None;
    blocked.diff_identity = None;
    blocked.changed_paths.clear();
    assert_eq!(blocked.validate(), Err(CandidateReceiptError::MissingBlocker));
    blocked.blockers = vec!["base is stale".to_string()];
    assert_eq!(blocked.validate(), Ok(()));
    assert!(!blocked.is_usable_candidate());

    let mut unknown = receipt();
    unknown.disposition = CandidateDisposition::Unknown;
    unknown.candidate_identity = None;
    unknown.diff_identity = None;
    unknown.changed_paths.clear();
    assert_eq!(unknown.validate(), Ok(()));
    assert!(!unknown.is_usable_candidate());
}

#[test]
fn receipt_json_round_trips_only_after_admission() {
    let receipt = receipt();
    let json = receipt.to_json().expect("valid receipt serializes");
    assert_eq!(CandidateReceipt::from_json(&json), Ok(receipt.clone()));

    let mut invalid = receipt;
    invalid.changed_paths = vec!["../escape".to_string()];
    assert_eq!(invalid.to_json(), Err(CandidateReceiptError::InvalidChangedPath));
    assert_eq!(CandidateReceipt::from_json("{not-json"), Err(CandidateReceiptError::InvalidJson));
}

#[test]
fn snapshot_receipt_captures_changed_added_and_deleted_paths() {
    let base = std::collections::BTreeMap::from([
        ("src/lib.rs".to_string(), b"old".to_vec()),
        ("src/removed.rs".to_string(), b"gone".to_vec()),
    ]);
    let candidate = std::collections::BTreeMap::from([
        ("src/lib.rs".to_string(), b"new".to_vec()),
        ("src/added.rs".to_string(), b"added".to_vec()),
    ]);
    let receipt = produced_receipt_from_snapshots(
        Bindings {
            work_id: "work-1",
            attempt_id: "attempt-1",
            fence: "7",
            workspace_id: "workspace-1",
        },
        &base,
        &candidate,
    )
    .expect("snapshot receipt derives");

    assert_eq!(receipt.changed_paths, vec!["src/added.rs", "src/lib.rs", "src/removed.rs"]);
    assert_eq!(receipt.disposition, CandidateDisposition::Produced);
    assert!(receipt.validate().is_ok());
    assert!(receipt.is_usable_candidate());

    let repeated = produced_receipt_from_snapshots(
        Bindings {
            work_id: "work-1",
            attempt_id: "attempt-1",
            fence: "7",
            workspace_id: "workspace-1",
        },
        &base,
        &candidate,
    )
    .expect("snapshot receipt repeats");
    assert_eq!(receipt, repeated);
}

#[test]
fn snapshot_receipt_rejects_no_change_and_invalid_paths() {
    let base = std::collections::BTreeMap::from([("src/lib.rs".to_string(), b"same".to_vec())]);
    assert_eq!(
        produced_receipt_from_snapshots(
            Bindings {
                work_id: "work-1",
                attempt_id: "attempt-1",
                fence: "7",
                workspace_id: "workspace-1"
            },
            &base,
            &base,
        ),
        Err(CandidateReceiptError::NoCandidateChanges)
    );

    let invalid = std::collections::BTreeMap::from([("../escape.rs".to_string(), b"x".to_vec())]);
    assert_eq!(
        produced_receipt_from_snapshots(
            Bindings {
                work_id: "work-1",
                attempt_id: "attempt-1",
                fence: "7",
                workspace_id: "workspace-1"
            },
            &base,
            &invalid,
        ),
        Err(CandidateReceiptError::InvalidChangedPath)
    );
}

#[test]
fn independent_verification_rejects_crossed_or_stale_receipts() {
    let base = std::collections::BTreeMap::from([("src/lib.rs".to_string(), b"old".to_vec())]);
    let candidate = std::collections::BTreeMap::from([("src/lib.rs".to_string(), b"new".to_vec())]);
    let receipt = produced_receipt_from_snapshots(
        Bindings {
            work_id: "work-1",
            attempt_id: "attempt-1",
            fence: "7",
            workspace_id: "workspace-1",
        },
        &base,
        &candidate,
    )
    .expect("receipt derives");

    assert_eq!(
        verify_receipt_from_snapshots(
            &receipt,
            Bindings {
                work_id: "work-1",
                attempt_id: "attempt-1",
                fence: "7",
                workspace_id: "workspace-1"
            },
            &base,
            &candidate,
        ),
        Ok(())
    );
    assert_eq!(
        verify_receipt_from_snapshots(
            &receipt,
            Bindings {
                work_id: "work-1",
                attempt_id: "attempt-2",
                fence: "7",
                workspace_id: "workspace-1"
            },
            &base,
            &candidate,
        ),
        Err(CandidateReceiptError::BindingMismatch)
    );
    let changed = std::collections::BTreeMap::from([("src/lib.rs".to_string(), b"different".to_vec())]);
    assert_eq!(
        verify_receipt_from_snapshots(
            &receipt,
            Bindings {
                work_id: "work-1",
                attempt_id: "attempt-1",
                fence: "7",
                workspace_id: "workspace-1"
            },
            &base,
            &changed,
        ),
        Err(CandidateReceiptError::ReceiptMismatch)
    );
}
