use super::*;

/// Helper: create a `ViewNode` with a given type tag and optional key.
fn node<T: 'static>(key: Option<&str>) -> ViewNode {
    ViewNode {
        key: key.map(|k| NodeKey(k.to_string())),
        type_tag: std::any::TypeId::of::<T>(),
        state: None,
    }
}

/// Helper: create a `ViewNode` with state.
fn node_with_state<T: 'static>(key: Option<&str>, state: u64) -> ViewNode {
    ViewNode {
        key: key.map(|k| NodeKey(k.to_string())),
        type_tag: std::any::TypeId::of::<T>(),
        state: Some(Box::new(state)),
    }
}

// Dummy types for type_tag discrimination.
struct TypeA;
struct TypeB;

// ── Reconciler tests ─────────────────────────────────────────────────

#[test]
fn keyed_nodes_preserve_state_on_reorder() {
    let old = vec![
        node_with_state::<TypeA>(Some("a"), 42),
        node_with_state::<TypeA>(Some("b"), 99),
    ];
    let new = vec![node::<TypeA>(Some("b")), node::<TypeA>(Some("a"))];

    let result = reconcile(old, new);
    assert_eq!(result.nodes.len(), 2);

    // "b" was second in old (state=99), now first
    let b_state = result.nodes[0].state.as_ref().unwrap();
    assert_eq!(*b_state.downcast_ref::<u64>().unwrap(), 99);

    // "a" was first in old (state=42), now second
    let a_state = result.nodes[1].state.as_ref().unwrap();
    assert_eq!(*a_state.downcast_ref::<u64>().unwrap(), 42);
}

#[test]
fn keyed_node_removed() {
    let old = vec![
        node_with_state::<TypeA>(Some("a"), 1),
        node_with_state::<TypeA>(Some("b"), 2),
    ];
    let new = vec![node::<TypeA>(Some("a"))];

    let result = reconcile(old, new);
    assert_eq!(result.nodes.len(), 1);
    assert_eq!(result.nodes[0].key.as_ref().unwrap().0, "a");
    let state = result.nodes[0].state.as_ref().unwrap();
    assert_eq!(*state.downcast_ref::<u64>().unwrap(), 1);
}

#[test]
fn positional_match_by_type() {
    let old = vec![
        node_with_state::<TypeA>(None, 10),
        node_with_state::<TypeB>(None, 20),
    ];
    let new = vec![node::<TypeA>(None), node::<TypeB>(None)];

    let result = reconcile(old, new);
    assert_eq!(result.nodes.len(), 2);
    assert_eq!(
        *result.nodes[0]
            .state
            .as_ref()
            .unwrap()
            .downcast_ref::<u64>()
            .unwrap(),
        10
    );
    assert_eq!(
        *result.nodes[1]
            .state
            .as_ref()
            .unwrap()
            .downcast_ref::<u64>()
            .unwrap(),
        20
    );
}

#[test]
fn type_mismatch_at_position_creates_new_node() {
    let old = vec![node_with_state::<TypeA>(None, 10)];
    let new = vec![node::<TypeB>(None)];

    let result = reconcile(old, new);
    assert_eq!(result.nodes.len(), 1);
    // No state transfer — type mismatch.
    assert!(result.nodes[0].state.is_none());
}

#[test]
fn appended_node_gets_fresh_state() {
    let old = vec![node_with_state::<TypeA>(Some("a"), 42)];
    let new = vec![node::<TypeA>(Some("a")), node::<TypeA>(Some("b"))];

    let result = reconcile(old, new);
    assert_eq!(result.nodes.len(), 2);
    // "a" preserves state.
    assert_eq!(
        *result.nodes[0]
            .state
            .as_ref()
            .unwrap()
            .downcast_ref::<u64>()
            .unwrap(),
        42
    );
    // "b" is new — no state.
    assert!(result.nodes[1].state.is_none());
}

#[test]
fn reconcile_is_deterministic() {
    let mk_old = || {
        vec![
            node_with_state::<TypeA>(Some("x"), 7),
            node_with_state::<TypeB>(None, 8),
        ]
    };
    let mk_new = || vec![node::<TypeB>(None), node::<TypeA>(Some("x"))];

    let r1 = reconcile(mk_old(), mk_new());
    let r2 = reconcile(mk_old(), mk_new());

    assert_eq!(r1.nodes.len(), r2.nodes.len());
    for (a, b) in r1.nodes.iter().zip(r2.nodes.iter()) {
        assert_eq!(a.key, b.key);
        assert_eq!(a.type_tag, b.type_tag);
        assert_eq!(a.state.is_some(), b.state.is_some());
    }
}

#[test]
fn empty_old_tree() {
    let old = vec![];
    let new = vec![node::<TypeA>(Some("a"))];

    let result = reconcile(old, new);
    assert_eq!(result.nodes.len(), 1);
    assert!(result.nodes[0].state.is_none());
}

#[test]
fn empty_new_tree() {
    let old = vec![node_with_state::<TypeA>(Some("a"), 1)];
    let new = vec![];

    let result = reconcile(old, new);
    assert!(result.nodes.is_empty());
}

// ── Commit tracking tests ────────────────────────────────────────────

#[test]
fn no_commits_when_content_fits() {
    let heights = [5, 5, 5];
    let commits = compute_commits(
        &heights,
        CommitViewport {
            viewport_height: 20,
            scroll_offset: 0,
        },
    );
    assert!(commits.is_empty());
}

#[test]
fn commits_nodes_above_scroll_offset() {
    // 4 nodes of 5 rows each = 20 total.
    // viewport = 10, scroll_offset = 10 means first 10 rows scrolled off.
    let heights = [5, 5, 5, 5];
    let commits = compute_commits(
        &heights,
        CommitViewport {
            viewport_height: 10,
            scroll_offset: 10,
        },
    );
    assert_eq!(commits, vec![0, 1]);
}

#[test]
fn partial_node_not_committed() {
    // Node 0 is 5 rows, node 1 is 5 rows. scroll_offset = 7.
    // Node 0 bottom = 5 <= 7 → committed.
    // Node 1 bottom = 10 > 7 → not committed (partially visible).
    let heights = [5, 5, 5];
    let commits = compute_commits(
        &heights,
        CommitViewport {
            viewport_height: 8,
            scroll_offset: 7,
        },
    );
    assert_eq!(commits, vec![0]);
}

#[test]
fn no_commits_at_zero_scroll() {
    let heights = [5, 5, 5, 5];
    let commits = compute_commits(
        &heights,
        CommitViewport {
            viewport_height: 10,
            scroll_offset: 0,
        },
    );
    assert!(commits.is_empty());
}

#[test]
fn all_nodes_committed() {
    let heights = [3, 3, 3];
    // total = 9, viewport = 5, scroll_offset = 9 → all above viewport.
    let commits = compute_commits(
        &heights,
        CommitViewport {
            viewport_height: 5,
            scroll_offset: 9,
        },
    );
    assert_eq!(commits, vec![0, 1, 2]);
}

#[test]
fn empty_heights() {
    let heights: [u16; 0] = [];
    let commits = compute_commits(
        &heights,
        CommitViewport {
            viewport_height: 10,
            scroll_offset: 0,
        },
    );
    assert!(commits.is_empty());
}
