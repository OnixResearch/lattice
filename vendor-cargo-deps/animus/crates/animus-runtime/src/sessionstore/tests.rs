use super::*;

const POLICY_HASH: &str = "cb51ed6a1173d80feeac2e39011cc74ad0a5f51b52df2082b8b5af9643b5e68c";
const STORE_ID: &str = "store.local";
const LOCATION_CAPABILITY: &str = "session.store.reviewed";
const SESSION_ID: &str = "session.durable";
const BRANCH_ID: &str = "branch.main";
const FIRST_LEAF: &str = "entry.assistant";
const SECOND_LEAF: &str = "entry.repair";
const FIRST_SEQUENCE: u64 = 1;
const THIRD_SEQUENCE: u64 = 3;
const EXPECTED_FIRST_FRAME_COUNT: usize = 2;
const EXPECTED_TOTAL_FRAME_COUNT: usize = 4;
const PRIVATE_USER: &[u8] = b"RAW_PROMPT: private user message";
const PRIVATE_ASSISTANT: &[u8] = b"private assistant response";
const PRIVATE_SUMMARY: &[u8] = b"private compaction summary";
const PRIVATE_REPAIR: &[u8] = b"private repaired tool result";
const CORRUPTED_BYTE_MASK: u8 = 1;
const OVER_BUDGET_PAYLOAD_BYTES: usize = 65_537;
const TRUNCATED_COMMIT_BYTES: usize = 10;
const ENVELOPE_LENGTH_OFFSET: usize = 9;
const ENVELOPE_LENGTH_END: usize = 13;
const SCHEMA_OFFSET: usize = 13;
const SCHEMA_NEXT_BYTE_OFFSET: usize = 14;
const UNSUPPORTED_SCHEMA_BYTE: u8 = 2;
const SENSITIVE_CLASS_COUNT: usize = 8;

#[test]
fn generated_policy_and_transaction_identity_repeat() {
    // r[verify agent_harness_runtime.durable_session_store.boundary]
    // r[verify agent_harness_runtime.durable_session_store.identity]
    let input = policy_input();
    let actual_hash = durable_session_store_policy_content_hash(&input).expect("policy hash");
    assert_eq!(actual_hash, POLICY_HASH);
    let policy = validate_durable_session_store_policy(&input).expect("policy");
    let descriptor = descriptor();
    let entries = [
        DurableSessionPrivateEntryDraft {
            entry_id: "entry.user",
            parent_id: None,
            kind: DurableSessionRecordKind::User,
            payload: PRIVATE_USER,
        },
        DurableSessionPrivateEntryDraft {
            entry_id: FIRST_LEAF,
            parent_id: Some("entry.user"),
            kind: DurableSessionRecordKind::Assistant,
            payload: PRIVATE_ASSISTANT,
        },
    ];
    let draft = DurableSessionTransactionDraft {
        descriptor: &descriptor,
        prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
        prior_active_leaf_id: None,
        first_sequence: FIRST_SEQUENCE,
        active_leaf_id: FIRST_LEAF,
        entries: &entries,
    };
    let first = plan_durable_session_transaction(&policy, &draft).expect("first transaction");
    let second = plan_durable_session_transaction(&policy, &draft).expect("second transaction");

    assert_eq!(first, second);
    assert_eq!(first.frames().len(), EXPECTED_FIRST_FRAME_COUNT);
    assert_eq!(first.sidecars().len(), EXPECTED_FIRST_FRAME_COUNT);
    assert!(!format!("{:?}", first.frames()[0]).contains("private user message"));
    assert!(!format!("{:?}", first.sidecars()[0]).contains("private user message"));
}

#[test]
fn transaction_lineage_and_active_leaf_fail_closed() {
    let policy = policy();
    let descriptor = descriptor();
    let valid_entry = [DurableSessionPrivateEntryDraft {
        entry_id: "entry.user",
        parent_id: None,
        kind: DurableSessionRecordKind::User,
        payload: PRIVATE_USER,
    }];
    let base = DurableSessionTransactionDraft {
        descriptor: &descriptor,
        prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
        prior_active_leaf_id: None,
        first_sequence: FIRST_SEQUENCE,
        active_leaf_id: "entry.user",
        entries: &valid_entry,
    };
    assert_eq!(base.entries.len(), 1);
    assert_eq!(base.entries[0].entry_id, base.active_leaf_id);
    assert_issue(
        plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
            active_leaf_id: "entry.missing",
            ..base
        }),
        DurableSessionStoreIssue::IntegrityMismatch,
    );
    let wrong_parent = [DurableSessionPrivateEntryDraft {
        entry_id: "entry.user",
        parent_id: Some("entry.missing"),
        kind: DurableSessionRecordKind::User,
        payload: PRIVATE_USER,
    }];
    assert_issue(
        plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
            entries: &wrong_parent,
            ..base
        }),
        DurableSessionStoreIssue::IntegrityMismatch,
    );
    assert_issue(
        plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
            prior_head_hash: &hash("nonempty-head"),
            ..base
        }),
        DurableSessionStoreIssue::StaleHead,
    );
}

#[test]
fn committed_groups_round_trip_and_restore_exact_leaf() {
    // r[verify agent_harness_runtime.durable_session_store.framing]
    // r[verify agent_harness_runtime.durable_session_store.commit]
    let policy = policy();
    let descriptor = descriptor();
    let first_entries = [
        DurableSessionPrivateEntryDraft {
            entry_id: "entry.user",
            parent_id: None,
            kind: DurableSessionRecordKind::User,
            payload: PRIVATE_USER,
        },
        DurableSessionPrivateEntryDraft {
            entry_id: FIRST_LEAF,
            parent_id: Some("entry.user"),
            kind: DurableSessionRecordKind::Assistant,
            payload: PRIVATE_ASSISTANT,
        },
    ];
    let first = plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
        descriptor: &descriptor,
        prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
        prior_active_leaf_id: None,
        first_sequence: FIRST_SEQUENCE,
        active_leaf_id: FIRST_LEAF,
        entries: &first_entries,
    })
    .expect("first");
    let second_entries = [
        DurableSessionPrivateEntryDraft {
            entry_id: "entry.summary",
            parent_id: Some(FIRST_LEAF),
            kind: DurableSessionRecordKind::Summary,
            payload: PRIVATE_SUMMARY,
        },
        DurableSessionPrivateEntryDraft {
            entry_id: SECOND_LEAF,
            parent_id: Some("entry.summary"),
            kind: DurableSessionRecordKind::Repair,
            payload: PRIVATE_REPAIR,
        },
    ];
    let second = plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
        descriptor: &descriptor,
        prior_head_hash: first.commit().commit_hash(),
        prior_active_leaf_id: Some(FIRST_LEAF),
        first_sequence: THIRD_SEQUENCE,
        active_leaf_id: SECOND_LEAF,
        entries: &second_entries,
    })
    .expect("second");
    assert_eq!(first.frames().len(), EXPECTED_FIRST_FRAME_COUNT);
    assert_eq!(second.frames().len(), EXPECTED_FIRST_FRAME_COUNT);
    let mut journal = encode_durable_session_transaction(&policy, &first).expect("first bytes");
    journal.extend(encode_durable_session_transaction(&policy, &second).expect("second bytes"));
    let recovered = recover_durable_session_log(&policy, &descriptor, journal.as_slice()).expect("recovered");

    assert_two_group_recovery(&recovered, &second);
}

fn assert_two_group_recovery(recovered: &DurableSessionRecoveryReport, second: &DurableSessionTransaction) {
    assert_eq!(recovered.status(), DurableSessionRecoveryStatus::Clean);
    assert_eq!(recovered.committed_frames().len(), EXPECTED_TOTAL_FRAME_COUNT);
    assert_eq!(recovered.active_leaf_id(), Some(SECOND_LEAF));
    assert_eq!(recovered.committed_head_hash(), second.commit().commit_hash());
    assert_eq!(recovered.committed_frames()[0].private_payload(), PRIVATE_USER);
}

#[test]
fn torn_frame_and_uncommitted_group_never_become_visible() {
    // r[verify agent_harness_runtime.durable_session_store.recovery]
    let policy = policy();
    let descriptor = descriptor();
    let entries = [DurableSessionPrivateEntryDraft {
        entry_id: "entry.user",
        parent_id: None,
        kind: DurableSessionRecordKind::User,
        payload: PRIVATE_USER,
    }];
    let transaction = plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
        descriptor: &descriptor,
        prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
        prior_active_leaf_id: None,
        first_sequence: FIRST_SEQUENCE,
        active_leaf_id: "entry.user",
        entries: &entries,
    })
    .expect("transaction");
    let encoded = encode_durable_session_transaction(&policy, &transaction).expect("encoded");
    let commit_start = last_magic_offset(encoded.as_slice());
    let uncommitted = &encoded[..commit_start];
    let recovered = recover_durable_session_log(&policy, &descriptor, uncommitted).expect("uncommitted recovery");
    let torn_end = encoded.len().saturating_sub(TRUNCATED_COMMIT_BYTES);
    let torn = recover_durable_session_log(&policy, &descriptor, &encoded[..torn_end]).expect("torn recovery");

    assert_eq!(recovered.status(), DurableSessionRecoveryStatus::IncompleteTailTransaction);
    assert!(recovered.committed_frames().is_empty());
    assert_eq!(torn.status(), DurableSessionRecoveryStatus::IncompleteTailFrame);
    assert!(torn.committed_frames().is_empty());
    assert_eq!(torn.committed_head_hash(), DURABLE_SESSION_EMPTY_HEAD_HASH);
}

#[test]
fn committed_corruption_sequence_gap_and_schema_fail_closed() {
    let policy = policy();
    let descriptor = descriptor();
    let entries = [DurableSessionPrivateEntryDraft {
        entry_id: "entry.user",
        parent_id: None,
        kind: DurableSessionRecordKind::User,
        payload: PRIVATE_USER,
    }];
    let transaction = plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
        descriptor: &descriptor,
        prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
        prior_active_leaf_id: None,
        first_sequence: FIRST_SEQUENCE,
        active_leaf_id: "entry.user",
        entries: &entries,
    })
    .expect("transaction");
    let encoded = encode_durable_session_transaction(&policy, &transaction).expect("encoded");
    let mut corruption = encoded.clone();
    let payload_offset_bytes = find_subslice(corruption.as_slice(), PRIVATE_USER);
    corruption[payload_offset_bytes] ^= CORRUPTED_BYTE_MASK;
    assert_ne!(corruption, encoded);
    assert_issue(
        recover_durable_session_log(&policy, &descriptor, corruption.as_slice()),
        DurableSessionStoreIssue::IntegrityMismatch,
    );
    let gap = plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
        descriptor: &descriptor,
        prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
        prior_active_leaf_id: None,
        first_sequence: THIRD_SEQUENCE,
        active_leaf_id: "entry.user",
        entries: &entries,
    })
    .expect("gap transaction");
    let gap_bytes = encode_durable_session_transaction(&policy, &gap).expect("gap bytes");
    assert_eq!(gap.frames()[0].sequence(), THIRD_SEQUENCE);
    assert_issue(
        recover_durable_session_log(&policy, &descriptor, gap_bytes.as_slice()),
        DurableSessionStoreIssue::SequenceGap,
    );
    let mut malformed_length_bytes = encoded.clone();
    malformed_length_bytes[ENVELOPE_LENGTH_OFFSET..ENVELOPE_LENGTH_END].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_issue(
        recover_durable_session_log(&policy, &descriptor, malformed_length_bytes.as_slice()),
        DurableSessionStoreIssue::BoundExceeded,
    );
    let mut unsupported_schema_bytes = encoded;
    unsupported_schema_bytes[SCHEMA_OFFSET] = 0;
    unsupported_schema_bytes[SCHEMA_NEXT_BYTE_OFFSET] = UNSUPPORTED_SCHEMA_BYTE;
    assert_issue(
        recover_durable_session_log(&policy, &descriptor, unsupported_schema_bytes.as_slice()),
        DurableSessionStoreIssue::UnsupportedSchema,
    );
}

#[test]
fn descriptor_encryption_and_bounds_fail_closed() {
    let policy = policy();
    let mut required = descriptor();
    required.encryption = DurableSessionEncryptionPosture::ExternalRequired;
    required.encryption_service_available = false;
    assert_eq!(required.encryption, DurableSessionEncryptionPosture::ExternalRequired);
    assert_issue(
        validate_durable_session_store_descriptor(&policy, &required),
        DurableSessionStoreIssue::EncryptionUnavailable,
    );
    let mut ambient = descriptor();
    ambient.location_capability = String::from("/home/private/store");
    assert!(ambient.location_capability.starts_with('/'));
    assert_issue(
        validate_durable_session_store_descriptor(&policy, &ambient),
        DurableSessionStoreIssue::InvalidDescriptor,
    );
    let empty_entries = [];
    assert_issue(
        plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
            descriptor: &descriptor(),
            prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
            prior_active_leaf_id: None,
            first_sequence: FIRST_SEQUENCE,
            active_leaf_id: "entry.user",
            entries: &empty_entries,
        }),
        DurableSessionStoreIssue::BoundExceeded,
    );
    let empty_payload = [DurableSessionPrivateEntryDraft {
        entry_id: "entry.user",
        parent_id: None,
        kind: DurableSessionRecordKind::User,
        payload: &[],
    }];
    assert_issue(
        plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
            descriptor: &descriptor(),
            prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
            prior_active_leaf_id: None,
            first_sequence: FIRST_SEQUENCE,
            active_leaf_id: "entry.user",
            entries: &empty_payload,
        }),
        DurableSessionStoreIssue::BoundExceeded,
    );
}

#[test]
fn over_budget_payload_bytes_fail_closed() {
    let policy = policy();
    let over_budget_payload_bytes = vec![0; OVER_BUDGET_PAYLOAD_BYTES];
    let over_budget_entry_drafts = [DurableSessionPrivateEntryDraft {
        entry_id: "entry.user",
        parent_id: None,
        kind: DurableSessionRecordKind::User,
        payload: over_budget_payload_bytes.as_slice(),
    }];
    assert_eq!(over_budget_payload_bytes.len(), OVER_BUDGET_PAYLOAD_BYTES);
    assert_eq!(over_budget_entry_drafts.len(), 1);
    assert_issue(
        plan_durable_session_transaction(&policy, &DurableSessionTransactionDraft {
            descriptor: &descriptor(),
            prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
            prior_active_leaf_id: None,
            first_sequence: FIRST_SEQUENCE,
            active_leaf_id: "entry.user",
            entries: &over_budget_entry_drafts,
        }),
        DurableSessionStoreIssue::BoundExceeded,
    );
}

#[test]
fn receipts_are_metadata_only_and_reject_private_classes() {
    // r[verify agent_harness_runtime.durable_session_store.receipts]
    // r[verify agent_harness_runtime.durable_session_store.verification]
    let policy = policy();
    let descriptor = descriptor();
    let clean = DurableSessionStoreReceiptInput {
        policy: &policy,
        descriptor: &descriptor,
        operation: DurableSessionStoreOperation::Commit,
        durability: DurableSessionDurability::SyncAllEachCommit,
        status: "committed",
        prior_head_hash: DURABLE_SESSION_EMPTY_HEAD_HASH,
        new_head_hash: &hash("head"),
        committed_frame_count: 1,
        recovery_status: DurableSessionRecoveryStatus::Clean,
        lock_status: "exclusive",
        sensitive: DurableSessionStoreSensitiveDraft::default(),
    };
    let receipt = project_durable_session_store_receipt(&clean).expect("receipt");
    let rendered = format!("{receipt:?}");
    assert_eq!(receipt.durability, DurableSessionDurability::SyncAllEachCommit);
    assert!(!rendered.contains("private user message"));
    assert!(receipt.non_claim.contains("not encryption"));
    let mut invalid_descriptor = descriptor.clone();
    invalid_descriptor.location_capability = String::from("/ambient/path");
    assert_issue(
        project_durable_session_store_receipt(&DurableSessionStoreReceiptInput {
            descriptor: &invalid_descriptor,
            ..clean
        }),
        DurableSessionStoreIssue::InvalidDescriptor,
    );
    let sensitive = sensitive_drafts();
    assert_eq!(sensitive.len(), SENSITIVE_CLASS_COUNT);
    for draft in sensitive {
        assert_issue(
            project_durable_session_store_receipt(&DurableSessionStoreReceiptInput {
                sensitive: draft,
                ..clean
            }),
            DurableSessionStoreIssue::ReceiptLeak,
        );
    }
}

fn policy() -> DurableSessionStorePolicy {
    validate_durable_session_store_policy(&policy_input()).expect("policy")
}

fn policy_input() -> DurableSessionStorePolicyInput {
    DurableSessionStorePolicyInput {
        schema: String::from(DURABLE_SESSION_STORE_POLICY_SCHEMA),
        policy_id: String::from("local-sync-each-commit"),
        max_frame_payload_bytes: 65_536,
        max_transaction_frames: 64,
        max_file_bytes: 67_108_864,
        file_mode: 0o600,
        durability: String::from("sync_all_each_commit"),
        locking: String::from("exclusive_create_new"),
        private_data: String::from("replay_private"),
        encryption_postures: vec![String::from("absent_explicit"), String::from("external_required")],
        tail_recovery: String::from("ignore_uncommitted"),
        committed_corruption: String::from("reject"),
        receipt_fields: vec![
            String::from("store_identity"),
            String::from("session_identity"),
            String::from("branch_identity"),
            String::from("head_identity"),
            String::from("operation"),
            String::from("durability"),
            String::from("counts"),
            String::from("lock_status"),
            String::from("recovery_status"),
            String::from("redaction"),
            String::from("non_claim"),
        ],
        policy_hash: String::from(POLICY_HASH),
    }
}

fn descriptor() -> DurableSessionStoreDescriptor {
    DurableSessionStoreDescriptor {
        store_id: String::from(STORE_ID),
        location_capability: String::from(LOCATION_CAPABILITY),
        session_id: String::from(SESSION_ID),
        branch_id: String::from(BRANCH_ID),
        policy_hash: String::from(POLICY_HASH),
        durability: DurableSessionDurability::SyncAllEachCommit,
        locking: DurableSessionLockPolicy::ExclusiveCreateNew,
        private_data: DurableSessionPrivacyPosture::ReplayPrivate,
        encryption: DurableSessionEncryptionPosture::AbsentExplicit,
        encryption_service_available: false,
        receipt_policy: String::from("metadata.only"),
    }
}

fn hash(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

fn last_magic_offset(bytes: &[u8]) -> usize {
    bytes.windows(8).rposition(|window| window == b"ANIMSJ01").expect("commit record magic")
}

fn find_subslice(bytes: &[u8], needle: &[u8]) -> usize {
    bytes.windows(needle.len()).position(|window| window == needle).expect("private payload bytes")
}

fn sensitive_drafts() -> [DurableSessionStoreSensitiveDraft<'static>; SENSITIVE_CLASS_COUNT] {
    [
        DurableSessionStoreSensitiveDraft {
            private_payload: Some(PRIVATE_USER),
            ..DurableSessionStoreSensitiveDraft::default()
        },
        DurableSessionStoreSensitiveDraft {
            tool_result: Some(PRIVATE_REPAIR),
            ..DurableSessionStoreSensitiveDraft::default()
        },
        DurableSessionStoreSensitiveDraft {
            provider_payload: Some("PROVIDER_PAYLOAD: private"),
            ..DurableSessionStoreSensitiveDraft::default()
        },
        DurableSessionStoreSensitiveDraft {
            credential: Some("API_KEY=private"),
            ..DurableSessionStoreSensitiveDraft::default()
        },
        DurableSessionStoreSensitiveDraft {
            private_path: Some("/home/private/store"),
            ..DurableSessionStoreSensitiveDraft::default()
        },
        DurableSessionStoreSensitiveDraft {
            raw_frame: Some(PRIVATE_ASSISTANT),
            ..DurableSessionStoreSensitiveDraft::default()
        },
        DurableSessionStoreSensitiveDraft {
            lock_dump: Some("lock owner private"),
            ..DurableSessionStoreSensitiveDraft::default()
        },
        DurableSessionStoreSensitiveDraft {
            raw_error: Some("unbounded private error"),
            ..DurableSessionStoreSensitiveDraft::default()
        },
    ]
}

fn assert_issue<T: std::fmt::Debug>(result: Result<T, DurableSessionStoreError>, issue: DurableSessionStoreIssue) {
    assert_eq!(result.expect_err("expected durable-store rejection").issue(), issue);
}
