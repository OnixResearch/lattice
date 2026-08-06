use super::*;

const POLICY_HASH: &str = "828fed5de19fe77a8e7a7050778113872d686343cd2a997070cf5e30930284db";
const SESSION_ID: &str = "session.redacted";
const ROOT_ID: &str = "entry.root";
const USER_ID: &str = "entry.user";
const ASSISTANT_ID: &str = "entry.assistant";
const SIBLING_ID: &str = "entry.sibling";
const COMPACTION_ID: &str = "entry.compaction";
const REPAIR_ID: &str = "entry.repair";
const EXPECTED_ENTRY_COUNT: usize = 6;
const SINGLETON_ENTRY_COUNT: usize = 1;
const SENSITIVE_DRAFT_COUNT: usize = 9;
const BOUNDED_ENTRY_COUNT: u32 = 5;
const EXPECTED_ACTIVE_PATH_COUNT: u32 = 5;
const EXPECTED_BRANCH_POINT_COUNT: u32 = 1;
const EXPECTED_BRANCH_CHILD_COUNT: u32 = 2;
const FIRST_SEQUENCE: u32 = 1;
const SECOND_SEQUENCE: u32 = 2;
const THIRD_SEQUENCE: u32 = 3;
const FOURTH_SEQUENCE: u32 = 4;
const FIFTH_SEQUENCE: u32 = 5;
const SIXTH_SEQUENCE: u32 = 6;
const SIBLING_INDEX: usize = 3;
const REPAIR_INDEX: usize = 5;
const POLICY_MAX_ENTRIES: u32 = 256;
const POLICY_MAX_DEPTH: u32 = 32;
const POLICY_MAX_SUMMARY_CHARS: u32 = 160;
const POLICY_MAX_ARTIFACT_BYTES: u32 = 1_048_576;
const PRIVATE_PROMPT: &str = "RAW_PROMPT: private";
const PRIVATE_TOOL_BODY: &str = "RAW_TOOL_BODY: private";
const PRIVATE_REASONING: &str = "HIDDEN_REASONING: private";
const PRIVATE_CREDENTIAL: &str = "API_KEY=private";
const PRIVATE_PROVIDER: &str = "PROVIDER_PAYLOAD: private";
const PRIVATE_PATH: &str = "/home/private/session";
const PRIVATE_ENVIRONMENT: &str = "ENVIRONMENT: SECRET=value";
const PRIVATE_ERROR: &str = "RAW_ERROR: unbounded provider failure";
const PRIVATE_CUSTOM_ENTRY: &str = "PRIVATE_CUSTOM_ENTRY: private";
const INJECTION_SUMMARY: &str = "safe <script src='https://remote.invalid'>status</script>";

#[test]
fn generated_policy_and_branched_projection_are_deterministic() {
    // r[verify agent_harness_runtime.session_tree_export.policy]
    // r[verify agent_harness_runtime.session_tree_export.verification]
    // r[verify agent_harness_runtime.session_tree_export.source]
    // r[verify agent_harness_runtime.session_tree_export.projection]
    let input = policy_input();
    let actual_hash = session_tree_export_policy_content_hash(&input);
    assert_eq!(actual_hash, POLICY_HASH);
    let policy = validate_session_tree_export_policy(&input).expect("valid policy");
    let first = project_session_tree_export(&policy, &source()).expect("first projection");
    let second = project_session_tree_export(&policy, &source()).expect("second projection");

    assert_eq!(first, second);
    assert_eq!(first.entries().len(), EXPECTED_ENTRY_COUNT);
    assert_eq!(first.active_path_count(), EXPECTED_ACTIVE_PATH_COUNT);
    assert_eq!(first.branch_point_count(), EXPECTED_BRANCH_POINT_COUNT);
    assert!(first.entries()[0].is_active_path());
    assert!(!first.entries()[SIBLING_INDEX].is_active_path());
    assert_eq!(first.entries()[1].child_count(), EXPECTED_BRANCH_CHILD_COUNT);
}

#[test]
fn html_and_lines_escape_injection_and_repeat_byte_for_byte() {
    // r[verify agent_harness_runtime.session_tree_export.html]
    // r[verify agent_harness_runtime.session_tree_export.determinism]
    let policy = policy();
    let projection = project_session_tree_export(&policy, &source()).expect("projection");
    let html = render_session_tree_export(&policy, &projection, SessionTreeExportFormat::Html).expect("html");
    let repeated =
        render_session_tree_export(&policy, &projection, SessionTreeExportFormat::Html).expect("repeated html");
    let lines =
        render_session_tree_export(&policy, &projection, SessionTreeExportFormat::SafeLines).expect("safe lines");
    let html_text = std::str::from_utf8(html.bytes()).expect("html utf8");
    let lines_text = std::str::from_utf8(lines.bytes()).expect("lines utf8");

    assert_eq!(html, repeated);
    assert!(html_text.contains("<h2>Topology</h2>"));
    assert!(html_text.contains("<h2>Storage order</h2>"));
    assert!(html_text.contains("parent=<code>entry.user</code>"));
    assert!(html_text.contains("&lt;script"));
    assert!(!html_text.contains("<script"));
    assert!(!html_text.contains("<style"));
    assert!(!html_text.contains("href="));
    assert!(lines_text.contains("schema\tanimus.session-tree-export-lines.v1"));
    assert!(!lines_text.contains(PRIVATE_PROMPT));
    assert_ne!(html.artifact_hash(), lines.artifact_hash());
}

#[test]
fn linear_empty_branch_and_safe_tool_status_project() {
    let policy = policy();
    let empty_source = singleton_source(SessionTreeExportEntryKind::SystemStatus);
    let empty_projection = project_session_tree_export(&policy, &empty_source).expect("empty selected branch");
    let source = singleton_source(SessionTreeExportEntryKind::ToolStatus);
    let projection = project_session_tree_export(&policy, &source).expect("linear projection");
    let artifact =
        render_session_tree_export(&policy, &projection, SessionTreeExportFormat::SafeLines).expect("linear artifact");

    assert_eq!(empty_projection.entries().len(), SINGLETON_ENTRY_COUNT);
    assert_eq!(empty_projection.branch_point_count(), 0);
    assert_eq!(projection.entries().len(), SINGLETON_ENTRY_COUNT);
    assert_eq!(projection.active_path_count(), FIRST_SEQUENCE);
    assert_eq!(projection.branch_point_count(), 0);
    assert!(std::str::from_utf8(artifact.bytes()).expect("safe lines").contains("tool_status"));
}

#[test]
fn malformed_topology_and_stale_leaf_fail_closed() {
    let policy = policy();
    let mut duplicate = source();
    duplicate.entries[1].entry_id = String::from(ROOT_ID);
    let mut missing_parent = source();
    missing_parent.entries[1].parent_id = Some(String::from("entry.missing"));
    let mut cycle = source();
    cycle.entries[0].parent_id = Some(String::from(REPAIR_ID));
    let mut stale_leaf = source();
    stale_leaf.active_leaf_id = String::from("entry.stale");
    let mut stale_repair = source();
    stale_repair.entries[REPAIR_INDEX].repair_target_id = Some(String::from("entry.stale"));

    assert_eq!(duplicate.entries.len(), EXPECTED_ENTRY_COUNT);
    assert_ne!(stale_leaf.active_leaf_id, REPAIR_ID);
    assert_issue(project_session_tree_export(&policy, &duplicate), SessionTreeExportIssue::DuplicateEntry);
    assert_issue(project_session_tree_export(&policy, &missing_parent), SessionTreeExportIssue::MissingParent);
    assert_issue(project_session_tree_export(&policy, &cycle), SessionTreeExportIssue::InvalidSource);
    assert_issue(project_session_tree_export(&policy, &stale_leaf), SessionTreeExportIssue::StaleActiveLeaf);
    assert_issue(project_session_tree_export(&policy, &stale_repair), SessionTreeExportIssue::StaleRepairTarget);
}

#[test]
fn unsafe_metadata_bounds_and_unsupported_kind_fail_closed() {
    let policy = policy();
    let mut private = source();
    private.entries[0].safe_summary = String::from(PRIVATE_PROMPT);
    let mut invalid_hash = source();
    invalid_hash.entries[0].entry_hash = String::from("invalid");
    let mut restrictive_input = policy_input();
    restrictive_input.allowed_entry_kinds = vec![String::from("system_status")];
    restrictive_input.policy_hash = session_tree_export_policy_content_hash(&restrictive_input);
    let restrictive = validate_session_tree_export_policy(&restrictive_input).expect("restrictive");

    assert!(private.entries[0].safe_summary.contains(PRIVATE_PROMPT));
    assert_ne!(invalid_hash.entries[0].entry_hash.len(), super::data::BLAKE3_HEX_CHARS);
    assert_issue(project_session_tree_export(&policy, &private), SessionTreeExportIssue::InvalidSource);
    assert_issue(project_session_tree_export(&policy, &invalid_hash), SessionTreeExportIssue::InvalidSource);
    assert_issue(project_session_tree_export(&restrictive, &source()), SessionTreeExportIssue::InvalidSource);
    let mut bounded_input = policy_input();
    bounded_input.max_entries = BOUNDED_ENTRY_COUNT;
    bounded_input.policy_hash = session_tree_export_policy_content_hash(&bounded_input);
    let bounded = validate_session_tree_export_policy(&bounded_input).expect("bounded policy");
    assert_issue(project_session_tree_export(&bounded, &source()), SessionTreeExportIssue::BoundExceeded);
    let mut shallow_input = policy_input();
    shallow_input.max_depth = FIRST_SEQUENCE;
    shallow_input.policy_hash = session_tree_export_policy_content_hash(&shallow_input);
    let shallow = validate_session_tree_export_policy(&shallow_input).expect("shallow policy");
    assert_issue(project_session_tree_export(&shallow, &source()), SessionTreeExportIssue::BoundExceeded);
    let mut summary_policy_input = policy_input();
    summary_policy_input.max_summary_chars = FIRST_SEQUENCE;
    summary_policy_input.policy_hash = session_tree_export_policy_content_hash(&summary_policy_input);
    let summary_policy = validate_session_tree_export_policy(&summary_policy_input).expect("summary policy");
    assert_issue(project_session_tree_export(&summary_policy, &source()), SessionTreeExportIssue::InvalidSource);
    let mut custom_kind_input = policy_input();
    custom_kind_input.allowed_entry_kinds = vec![String::from("custom_private")];
    custom_kind_input.policy_hash = session_tree_export_policy_content_hash(&custom_kind_input);
    assert_issue(validate_session_tree_export_policy(&custom_kind_input), SessionTreeExportIssue::InvalidPolicy);
}

#[test]
fn every_sensitive_summary_class_is_rejected() {
    let policy = policy();
    let private_values = [
        PRIVATE_PROMPT,
        PRIVATE_TOOL_BODY,
        PRIVATE_REASONING,
        PRIVATE_CREDENTIAL,
        PRIVATE_PROVIDER,
        PRIVATE_PATH,
        PRIVATE_ENVIRONMENT,
        PRIVATE_ERROR,
        PRIVATE_CUSTOM_ENTRY,
    ];
    assert_eq!(private_values.len(), SENSITIVE_DRAFT_COUNT);
    assert!(!policy.policy_hash().is_empty());
    for private_value in private_values {
        let mut unsafe_source = singleton_source(SessionTreeExportEntryKind::SystemStatus);
        unsafe_source.entries[0].safe_summary = String::from(private_value);
        assert_issue(project_session_tree_export(&policy, &unsafe_source), SessionTreeExportIssue::InvalidSource);
    }
}

#[test]
fn artifact_budget_and_format_policy_fail_closed() {
    let mut tiny_html_input = policy_input();
    tiny_html_input.max_html_bytes = 1;
    tiny_html_input.policy_hash = session_tree_export_policy_content_hash(&tiny_html_input);
    let tiny_html_policy = validate_session_tree_export_policy(&tiny_html_input).expect("tiny html policy");
    let projection = project_session_tree_export(&tiny_html_policy, &source()).expect("projection");
    assert_eq!(tiny_html_policy.artifact_byte_limit(SessionTreeExportFormat::Html), 1);
    assert!(tiny_html_policy.formats.contains(&SessionTreeExportFormat::Html));
    assert_issue(
        render_session_tree_export(&tiny_html_policy, &projection, SessionTreeExportFormat::Html),
        SessionTreeExportIssue::BoundExceeded,
    );

    let mut lines_only_input = policy_input();
    lines_only_input.formats = vec![String::from("safe_lines")];
    lines_only_input.policy_hash = session_tree_export_policy_content_hash(&lines_only_input);
    let lines_only = validate_session_tree_export_policy(&lines_only_input).expect("lines only");
    let projection = project_session_tree_export(&lines_only, &source()).expect("projection");
    assert_issue(
        render_session_tree_export(&lines_only, &projection, SessionTreeExportFormat::Html),
        SessionTreeExportIssue::UnsupportedFormat,
    );
}

#[test]
fn receipt_is_metadata_only_and_rejects_every_private_class() {
    // r[verify agent_harness_runtime.session_tree_export.receipts]
    let policy = policy();
    let projection = project_session_tree_export(&policy, &source()).expect("projection");
    let artifact = render_session_tree_export(&policy, &projection, SessionTreeExportFormat::Html).expect("artifact");
    let receipt = project_session_tree_export_receipt(&SessionTreeExportReceiptInput {
        policy: &policy,
        projection: &projection,
        artifact: &artifact,
        write: write_evidence(&artifact),
        sensitive: SessionTreeExportSensitiveDraft::default(),
    })
    .expect("receipt");
    let rendered = format!("{receipt:?}");

    assert_eq!(receipt.artifact_hash, artifact.artifact_hash());
    assert_eq!(receipt.artifact_bytes, artifact.byte_count());
    assert_eq!(receipt.entry_count, u32::try_from(EXPECTED_ENTRY_COUNT).expect("bounded expected entry count"));
    assert_eq!(receipt.active_path_count, EXPECTED_ACTIVE_PATH_COUNT);
    assert!(!rendered.contains(PRIVATE_PROMPT));
    for sensitive in sensitive_drafts() {
        assert_issue(
            project_session_tree_export_receipt(&SessionTreeExportReceiptInput {
                policy: &policy,
                projection: &projection,
                artifact: &artifact,
                write: write_evidence(&artifact),
                sensitive,
            }),
            SessionTreeExportIssue::ReceiptLeak,
        );
    }
    let mut incomplete_write = write_evidence(&artifact);
    incomplete_write.synchronized = false;
    assert_issue(
        project_session_tree_export_receipt(&SessionTreeExportReceiptInput {
            policy: &policy,
            projection: &projection,
            artifact: &artifact,
            write: incomplete_write,
            sensitive: SessionTreeExportSensitiveDraft::default(),
        }),
        SessionTreeExportIssue::InvalidWrite,
    );
    let mut failed_write = write_evidence(&artifact);
    failed_write.status = "failed";
    assert_issue(
        project_session_tree_export_receipt(&SessionTreeExportReceiptInput {
            policy: &policy,
            projection: &projection,
            artifact: &artifact,
            write: failed_write,
            sensitive: SessionTreeExportSensitiveDraft::default(),
        }),
        SessionTreeExportIssue::InvalidWrite,
    );
}

fn policy() -> SessionTreeExportPolicy {
    validate_session_tree_export_policy(&policy_input()).expect("policy")
}

fn policy_input() -> SessionTreeExportPolicyInput {
    SessionTreeExportPolicyInput {
        schema: String::from(SESSION_TREE_EXPORT_POLICY_SCHEMA),
        policy_id: String::from("redacted-session-tree"),
        max_entries: POLICY_MAX_ENTRIES,
        max_depth: POLICY_MAX_DEPTH,
        max_summary_chars: POLICY_MAX_SUMMARY_CHARS,
        max_html_bytes: POLICY_MAX_ARTIFACT_BYTES,
        max_safe_lines_bytes: POLICY_MAX_ARTIFACT_BYTES,
        allowed_entry_kinds: vec![
            String::from("user_omitted"),
            String::from("assistant_omitted"),
            String::from("tool_status"),
            String::from("compaction"),
            String::from("repair"),
            String::from("branch_summary"),
            String::from("system_status"),
        ],
        formats: vec![String::from("html"), String::from("safe_lines")],
        branch_visibility: String::from("all_redacted"),
        tool_metadata_posture: String::from("status_only"),
        custom_entry_posture: String::from("reject"),
        receipt_fields: vec![
            String::from("source_identity"),
            String::from("projection_identity"),
            String::from("artifact_identity"),
            String::from("counts"),
            String::from("format"),
            String::from("byte_count"),
            String::from("redaction"),
            String::from("write_status"),
            String::from("non_claim"),
        ],
        content_posture: String::from("metadata_only"),
        path_posture: String::from("omit"),
        malformed_input: String::from("reject"),
        scripts: String::from("forbidden"),
        remote_resources: String::from("forbidden"),
        policy_hash: String::from(POLICY_HASH),
    }
}

fn singleton_source(kind: SessionTreeExportEntryKind) -> SessionTreeExportSourceInput {
    SessionTreeExportSourceInput {
        session_id: String::from(SESSION_ID),
        snapshot_hash: hash("singleton-snapshot"),
        replay_hash: hash("singleton-replay"),
        active_leaf_id: String::from(ROOT_ID),
        entries: vec![entry(ROOT_ID, None, FIRST_SEQUENCE, kind)],
        safe_receipt_hashes: Vec::new(),
    }
}

fn source() -> SessionTreeExportSourceInput {
    let mut entries = vec![
        entry(ROOT_ID, None, FIRST_SEQUENCE, SessionTreeExportEntryKind::SystemStatus),
        entry(USER_ID, Some(ROOT_ID), SECOND_SEQUENCE, SessionTreeExportEntryKind::UserOmitted),
        entry(ASSISTANT_ID, Some(USER_ID), THIRD_SEQUENCE, SessionTreeExportEntryKind::AssistantOmitted),
        entry(SIBLING_ID, Some(USER_ID), FOURTH_SEQUENCE, SessionTreeExportEntryKind::BranchSummary),
        entry(COMPACTION_ID, Some(ASSISTANT_ID), FIFTH_SEQUENCE, SessionTreeExportEntryKind::Compaction),
        entry(REPAIR_ID, Some(COMPACTION_ID), SIXTH_SEQUENCE, SessionTreeExportEntryKind::Repair),
    ];
    entries[SIBLING_INDEX].safe_summary = String::from(INJECTION_SUMMARY);
    let source = SessionTreeExportSourceInput {
        session_id: String::from(SESSION_ID),
        snapshot_hash: hash("snapshot"),
        replay_hash: hash("replay"),
        active_leaf_id: String::from(REPAIR_ID),
        entries,
        safe_receipt_hashes: vec![hash("receipt")],
    };
    debug_assert_eq!(source.entries.len(), EXPECTED_ENTRY_COUNT);
    debug_assert_eq!(source.active_leaf_id, REPAIR_ID);
    source
}

fn entry(
    entry_id: &str,
    parent_id: Option<&str>,
    storage_sequence: u32,
    kind: SessionTreeExportEntryKind,
) -> SessionTreeExportEntryInput {
    let (safe_summary, omitted_field_count) = match kind {
        SessionTreeExportEntryKind::UserOmitted => ("user body omitted", 1),
        SessionTreeExportEntryKind::AssistantOmitted => ("assistant body omitted", 1),
        SessionTreeExportEntryKind::ToolStatus => ("tool status", 0),
        SessionTreeExportEntryKind::Compaction => ("compacted prior context", 0),
        SessionTreeExportEntryKind::Repair => ("repaired interrupted tool result", 0),
        SessionTreeExportEntryKind::BranchSummary => ("branch summary", 0),
        SessionTreeExportEntryKind::SystemStatus => ("session started", 0),
    };
    let repair_target_id = if kind == SessionTreeExportEntryKind::Repair {
        parent_id.map(String::from)
    } else {
        None
    };
    let entry = SessionTreeExportEntryInput {
        entry_id: String::from(entry_id),
        parent_id: parent_id.map(String::from),
        storage_sequence,
        kind,
        repair_target_id,
        safe_summary: String::from(safe_summary),
        entry_hash: hash(entry_id),
        status: String::from("accepted"),
        omitted_field_count,
    };
    debug_assert!(!entry.entry_id.is_empty());
    debug_assert!(entry.storage_sequence >= FIRST_SEQUENCE);
    entry
}

fn hash(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

fn write_evidence(artifact: &SessionTreeExportArtifact) -> SessionTreeExportWriteEvidence<'_> {
    SessionTreeExportWriteEvidence {
        destination_capability: "export.reviewed",
        allow_overwrite: false,
        file_mode: super::data::PRIVATE_FILE_MODE,
        max_bytes: artifact.byte_count(),
        artifact_hash: artifact.artifact_hash(),
        bytes_written: artifact.byte_count(),
        committed: true,
        synchronized: true,
        status: "committed",
    }
}

fn sensitive_drafts() -> [SessionTreeExportSensitiveDraft<'static>; SENSITIVE_DRAFT_COUNT] {
    [
        SessionTreeExportSensitiveDraft {
            raw_content: Some(PRIVATE_PROMPT),
            ..SessionTreeExportSensitiveDraft::default()
        },
        SessionTreeExportSensitiveDraft {
            tool_body: Some(PRIVATE_TOOL_BODY),
            ..SessionTreeExportSensitiveDraft::default()
        },
        SessionTreeExportSensitiveDraft {
            reasoning: Some(PRIVATE_REASONING),
            ..SessionTreeExportSensitiveDraft::default()
        },
        SessionTreeExportSensitiveDraft {
            credential: Some(PRIVATE_CREDENTIAL),
            ..SessionTreeExportSensitiveDraft::default()
        },
        SessionTreeExportSensitiveDraft {
            provider_payload: Some(PRIVATE_PROVIDER),
            ..SessionTreeExportSensitiveDraft::default()
        },
        SessionTreeExportSensitiveDraft {
            private_path: Some(PRIVATE_PATH),
            ..SessionTreeExportSensitiveDraft::default()
        },
        SessionTreeExportSensitiveDraft {
            environment: Some(PRIVATE_ENVIRONMENT),
            ..SessionTreeExportSensitiveDraft::default()
        },
        SessionTreeExportSensitiveDraft {
            raw_error: Some(PRIVATE_ERROR),
            ..SessionTreeExportSensitiveDraft::default()
        },
        SessionTreeExportSensitiveDraft {
            private_custom_entry: Some(PRIVATE_CUSTOM_ENTRY),
            ..SessionTreeExportSensitiveDraft::default()
        },
    ]
}

fn assert_issue<T: std::fmt::Debug>(result: Result<T, SessionTreeExportError>, issue: SessionTreeExportIssue) {
    assert_eq!(result.expect_err("expected export rejection").issue(), issue);
}
