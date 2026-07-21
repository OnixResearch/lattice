use super::support::*;

#[test]
fn duplicate_entry_id_fails_closed() {
    let snapshot = super::super::Snapshot::new(duplicate_nodes(), DUPLICATE_ID, alloc::vec![])
        .expect("snapshot should build");

    assert_eq!(
        super::super::replay_snapshot(&snapshot).expect_err("duplicate should fail"),
        super::super::Error::DuplicateEntryId {
            entry_id: alloc::string::String::from(DUPLICATE_ID),
        }
    );
}

#[test]
fn missing_parent_fails_closed() {
    let snapshot = super::super::Snapshot::new(missing_parent_nodes(), ASSISTANT_ID, alloc::vec![])
        .expect("snapshot should build");

    assert_eq!(
        super::super::replay_snapshot(&snapshot).expect_err("missing parent should fail"),
        super::super::Error::MissingParent {
            entry_id: alloc::string::String::from(ASSISTANT_ID),
            parent_id: alloc::string::String::from(MISSING_PARENT_ID),
        }
    );
}

#[test]
fn parent_cycle_fails_closed() {
    let snapshot = super::super::Snapshot::new(cycle_nodes(), CYCLE_A_ID, alloc::vec![])
        .expect("snapshot should build");

    assert!(matches!(
        super::super::replay_snapshot(&snapshot).expect_err("cycle should fail"),
        super::super::Error::ParentCycle { .. }
    ));
}

#[test]
fn invalid_active_leaf_fails_closed() {
    let snapshot = super::super::Snapshot::new(branch_nodes(), INVALID_ACTIVE_ID, alloc::vec![])
        .expect("snapshot should build");

    assert_eq!(
        super::super::replay_snapshot(&snapshot).expect_err("invalid active should fail"),
        super::super::Error::InvalidActiveLeaf {
            active_leaf: alloc::string::String::from(INVALID_ACTIVE_ID),
        }
    );
}

#[test]
fn bad_label_target_fails_closed() {
    let label = super::super::Label::new(INVALID_ACTIVE_ID, NAV_LABEL).expect("label");
    let snapshot = super::super::Snapshot::new(branch_nodes(), SIBLING_ID, alloc::vec![label])
        .expect("snapshot should build");

    assert_eq!(
        super::super::replay_snapshot(&snapshot).expect_err("bad label should fail"),
        super::super::Error::MissingLabelTarget {
            target_id: alloc::string::String::from(INVALID_ACTIVE_ID),
        }
    );
}

#[test]
fn orphan_tool_result_fails_closed() {
    let snapshot = super::super::Snapshot::new(orphan_tool_nodes(), TOOL_ID, alloc::vec![])
        .expect("snapshot should build");

    assert_eq!(
        super::super::replay_snapshot(&snapshot).expect_err("orphan tool should fail"),
        super::super::Error::OrphanToolResult {
            entry_id: alloc::string::String::from(TOOL_ID),
        }
    );
}

#[test]
fn malformed_summary_boundary_fails_closed() {
    let snapshot = super::super::Snapshot::new(summary_root_nodes(), SUMMARY_ID, alloc::vec![])
        .expect("snapshot should build");

    assert_eq!(
        super::super::replay_snapshot(&snapshot).expect_err("summary should fail"),
        super::super::Error::MalformedSummaryBoundary {
            entry_id: alloc::string::String::from(SUMMARY_ID),
        }
    );
}

fn duplicate_nodes() -> alloc::vec::Vec<super::super::Node> {
    alloc::vec![
        super::super::Node::root(
            super::super::Entry::user(DUPLICATE_ID, USER_TEXT).expect("entry")
        ),
        super::super::Node::root(
            super::super::Entry::assistant(DUPLICATE_ID, ASSISTANT_TEXT).expect("entry"),
        ),
    ]
}

fn missing_parent_nodes() -> alloc::vec::Vec<super::super::Node> {
    alloc::vec![
        super::super::Node::root(super::super::Entry::user(USER_ID, USER_TEXT).expect("entry")),
        super::super::Node::child(
            super::super::Entry::assistant(ASSISTANT_ID, ASSISTANT_TEXT).expect("entry"),
            MISSING_PARENT_ID,
        )
        .expect("node should build"),
    ]
}

fn cycle_nodes() -> alloc::vec::Vec<super::super::Node> {
    alloc::vec![
        super::super::Node::child(
            super::super::Entry::user(CYCLE_A_ID, USER_TEXT).expect("entry"),
            CYCLE_B_ID,
        )
        .expect("node should build"),
        super::super::Node::child(
            super::super::Entry::assistant(CYCLE_B_ID, ASSISTANT_TEXT).expect("entry"),
            CYCLE_A_ID,
        )
        .expect("node should build"),
    ]
}

fn orphan_tool_nodes() -> alloc::vec::Vec<super::super::Node> {
    alloc::vec![super::super::Node::root(
        super::super::Entry::tool(TOOL_ID, TOOL_LABEL, TOOL_TEXT).expect("tool entry should build"),
    )]
}

fn summary_root_nodes() -> alloc::vec::Vec<super::super::Node> {
    alloc::vec![super::super::Node::root(
        super::super::Entry::summary(SUMMARY_ID, SUMMARY_TEXT).expect("summary entry should build"),
    )]
}
