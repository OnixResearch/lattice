pub(super) const USER_ID: &str = "user-1";
pub(super) const ASSISTANT_ID: &str = "assistant-1";
pub(super) const TOOL_ID: &str = "tool-1";
pub(super) const SUMMARY_ID: &str = "summary-1";
pub(super) const USAGE_ID: &str = "usage-1";
pub(super) const RECEIPT_ID: &str = "receipt-1";
pub(super) const UNSUPPORTED_ID: &str = "unsupported-1";
pub(super) const ADAPTER_ID: &str = "adapter-1";
pub(super) const SIBLING_ID: &str = "assistant-2";
pub(super) const LATER_ID: &str = "user-2";
pub(super) const DUPLICATE_ID: &str = "duplicate-1";
pub(super) const MISSING_PARENT_ID: &str = "missing-parent";
pub(super) const CYCLE_A_ID: &str = "cycle-a";
pub(super) const CYCLE_B_ID: &str = "cycle-b";
pub(super) const INVALID_ACTIVE_ID: &str = "invalid-active";
pub(super) const SESSION_LABEL: &str = "session-a";
pub(super) const TOOL_LABEL: &str = "search";
pub(super) const NAV_LABEL: &str = "follow-up branch";
pub(super) const USER_TEXT: &str = "user asks a question";
pub(super) const ASSISTANT_TEXT: &str = "assistant drafts an answer";
pub(super) const SIBLING_TEXT: &str = "assistant explores sibling path";
pub(super) const LATER_TEXT: &str = "user asks after compaction";
pub(super) const TOOL_TEXT: &str = "tool result summary";
pub(super) const SUMMARY_TEXT: &str = "bounded prior context";
pub(super) const UNSUPPORTED_KIND: &str = "opaque-entry";
pub(super) const ADAPTER_KIND: &str = "adapter-owned-entry";
pub(super) const INPUT_TOKENS: u32 = 5;
pub(super) const OUTPUT_TOKENS: u32 = 8;
pub(super) const HASH_TEXT: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const LINEAR_CONTEXT_ITEMS: usize = 4;
pub(super) const LINEAR_ENTRY_COUNT: u32 = 6;
pub(super) const LINEAR_CONTEXT_COUNT: u32 = 4;
pub(super) const BRANCH_CONTEXT_ITEMS: usize = 2;
pub(super) const SUMMARY_CONTEXT_ITEMS: usize = 2;
pub(super) const TOOL_CONTEXT_ITEMS: usize = 3;
pub(super) const EXPECTED_ONE_U32: u32 = 1;
pub(super) const EXPECTED_ONE_USIZE: usize = 1;
pub(super) const FIRST_CONTEXT_INDEX: usize = 0;
pub(super) const SECOND_CONTEXT_INDEX: usize = 1;
pub(super) const THIRD_CONTEXT_INDEX: usize = 2;
pub(super) const FOURTH_CONTEXT_INDEX: usize = 3;

pub(super) fn branch_snapshot(
    active_leaf: &str,
) -> Result<super::super::Snapshot, crate::AdapterContractError> {
    super::super::Snapshot::new(branch_nodes(), active_leaf, alloc::vec![])
}

pub(super) fn branch_nodes() -> alloc::vec::Vec<super::super::Node> {
    alloc::vec![
        super::super::Node::root(super::super::Entry::user(USER_ID, USER_TEXT).expect("entry")),
        super::super::Node::child(
            super::super::Entry::assistant(ASSISTANT_ID, ASSISTANT_TEXT).expect("entry"),
            USER_ID,
        )
        .expect("node should build"),
        super::super::Node::child(
            super::super::Entry::assistant(SIBLING_ID, SIBLING_TEXT).expect("entry"),
            USER_ID,
        )
        .expect("node should build"),
    ]
}

pub(super) fn tool_snapshot() -> Result<super::super::Snapshot, crate::AdapterContractError> {
    super::super::Snapshot::new(tool_nodes(), TOOL_ID, alloc::vec![])
}

pub(super) fn summary_snapshot() -> Result<super::super::Snapshot, crate::AdapterContractError> {
    super::super::Snapshot::new(summary_nodes(), LATER_ID, alloc::vec![])
}

pub(super) fn sample_receipt() -> crate::SafeReceipt {
    let hash = crate::ArtifactHash::new(HASH_TEXT).expect("hash should parse");
    let labels = ["ledger-replay"];
    let hashes = [hash];
    let redactions = [crate::ReceiptRedaction::PayloadHashOnly];
    crate::SafeReceipt::new(
        crate::SafeReceiptDraft::new(
            crate::ReceiptStatus::Completed,
            crate::ReceiptIssueCode::ProvenanceObservation,
            crate::ReceiptOperation::ProvenanceObservation,
        )
        .with_labels(&labels)
        .with_artifact_hashes(&hashes)
        .with_redactions(&redactions),
    )
    .expect("sample receipt should build")
}

fn tool_nodes() -> alloc::vec::Vec<super::super::Node> {
    alloc::vec![
        super::super::Node::root(super::super::Entry::user(USER_ID, USER_TEXT).expect("entry")),
        super::super::Node::child(
            super::super::Entry::assistant(ASSISTANT_ID, ASSISTANT_TEXT).expect("entry"),
            USER_ID,
        )
        .expect("node should build"),
        super::super::Node::child(
            super::super::Entry::tool(TOOL_ID, TOOL_LABEL, TOOL_TEXT).expect("entry"),
            ASSISTANT_ID,
        )
        .expect("node should build"),
    ]
}

fn summary_nodes() -> alloc::vec::Vec<super::super::Node> {
    alloc::vec![
        super::super::Node::root(super::super::Entry::user(USER_ID, USER_TEXT).expect("entry")),
        super::super::Node::child(
            super::super::Entry::assistant(ASSISTANT_ID, ASSISTANT_TEXT).expect("entry"),
            USER_ID,
        )
        .expect("node should build"),
        super::super::Node::child(
            super::super::Entry::summary(SUMMARY_ID, SUMMARY_TEXT).expect("entry"),
            ASSISTANT_ID,
        )
        .expect("node should build"),
        super::super::Node::child(
            super::super::Entry::user(LATER_ID, LATER_TEXT).expect("entry"),
            SUMMARY_ID,
        )
        .expect("node should build"),
    ]
}
