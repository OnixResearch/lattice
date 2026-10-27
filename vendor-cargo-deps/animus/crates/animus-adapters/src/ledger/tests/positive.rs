use super::support::*;

#[test]
fn session_tree_replay_follows_active_branch_and_excludes_sibling() {
    let snapshot = branch_snapshot(SIBLING_ID).expect("snapshot should build");

    let replayed = super::super::replay_snapshot(&snapshot).expect("branch replay should succeed");
    let context = replayed.context();

    assert_eq!(context.len(), BRANCH_CONTEXT_ITEMS);
    assert_eq!(context[FIRST_CONTEXT_INDEX].content(), USER_TEXT);
    assert_eq!(context[SECOND_CONTEXT_INDEX].content(), SIBLING_TEXT);
    assert!(context.iter().all(|item| item.content() != ASSISTANT_TEXT));
}

#[test]
fn session_tree_replay_preserves_tool_result_order() {
    let snapshot = tool_snapshot().expect("tool snapshot should build");

    let replayed = super::super::replay_snapshot(&snapshot).expect("tool replay should succeed");
    let context = replayed.context();

    assert_eq!(context.len(), TOOL_CONTEXT_ITEMS);
    assert_eq!(
        context[FIRST_CONTEXT_INDEX].role(),
        super::super::ContextRole::User
    );
    assert_eq!(
        context[SECOND_CONTEXT_INDEX].role(),
        super::super::ContextRole::Assistant
    );
    assert_eq!(
        context[THIRD_CONTEXT_INDEX].role(),
        super::super::ContextRole::Tool
    );
    assert_eq!(context[THIRD_CONTEXT_INDEX].content(), TOOL_TEXT);
}

#[test]
fn summary_entry_replaces_older_model_visible_span() {
    let snapshot = summary_snapshot().expect("summary snapshot should build");

    let replayed = super::super::replay_snapshot(&snapshot).expect("summary replay should succeed");
    let context = replayed.context();

    assert_eq!(context.len(), SUMMARY_CONTEXT_ITEMS);
    assert_eq!(
        context[FIRST_CONTEXT_INDEX].role(),
        super::super::ContextRole::System
    );
    assert_eq!(context[FIRST_CONTEXT_INDEX].content(), SUMMARY_TEXT);
    assert_eq!(context[SECOND_CONTEXT_INDEX].content(), LATER_TEXT);
    assert!(context.iter().all(|item| item.content() != USER_TEXT));
    assert!(context.iter().all(|item| item.content() != ASSISTANT_TEXT));
}

#[test]
fn labels_are_metadata_only_by_default() {
    let label = super::super::Label::new(SIBLING_ID, NAV_LABEL).expect("label should build");
    let snapshot = super::super::Snapshot::new(branch_nodes(), SIBLING_ID, alloc::vec![label])
        .expect("snapshot should build");

    let replayed = super::super::replay_snapshot(&snapshot).expect("label replay should succeed");

    assert_eq!(replayed.labels().len(), EXPECTED_ONE_USIZE);
    assert_eq!(replayed.labels()[FIRST_CONTEXT_INDEX].text(), NAV_LABEL);
    assert!(
        replayed
            .context()
            .iter()
            .all(|item| !item.content().contains(NAV_LABEL))
    );
    assert_eq!(
        snapshot
            .labels_for(replayed.labels()[FIRST_CONTEXT_INDEX].target_id())
            .count(),
        EXPECTED_ONE_USIZE
    );
}
