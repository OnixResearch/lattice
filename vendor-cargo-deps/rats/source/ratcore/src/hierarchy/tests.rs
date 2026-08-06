use super::*;

const fn node(value: u32) -> NodeId {
    NodeId::new(value)
}

fn sample_tree() -> SimpleTree {
    // root(0)
    //   ├─ a(1)
    //   │  ├─ a1(3)
    //   │  └─ a2(4)
    //   └─ b(2)
    //      └─ b1(5)
    SimpleTree::new(vec![
        (node(0), None, "root".into()),
        (node(1), Some(node(0)), "a".into()),
        (node(2), Some(node(0)), "b".into()),
        (node(3), Some(node(1)), "a1".into()),
        (node(4), Some(node(1)), "a2".into()),
        (node(5), Some(node(2)), "b1".into()),
    ])
}

#[test]
fn simple_tree_roots() {
    let tree = sample_tree();
    assert_eq!(tree.root_count(), 1);
    assert_eq!(tree.root(0), node(0));
}

#[test]
fn simple_tree_children() {
    let tree = sample_tree();
    assert_eq!(tree.child_count(node(0)), 2);
    assert_eq!(tree.child(node(0), 0), node(1));
    assert_eq!(tree.child(node(0), 1), node(2));
    assert_eq!(tree.child_count(node(1)), 2);
    assert_eq!(tree.child_count(node(5)), 0); // leaf
}

#[test]
fn simple_tree_labels() {
    let tree = sample_tree();
    assert_eq!(tree.node_label(node(0)), "root");
    assert_eq!(tree.node_label(node(3)), "a1");
}

#[test]
fn simple_tree_parent() {
    let tree = sample_tree();
    assert_eq!(tree.parent(node(0)), None);
    assert_eq!(tree.parent(node(1)), Some(node(0)));
    assert_eq!(tree.parent(node(3)), Some(node(1)));
}

#[test]
fn simple_tree_icon_default() {
    let tree = sample_tree();
    assert_eq!(tree.node_icon(node(0)), None);
}

#[test]
fn simple_tree_multiple_roots() {
    let tree = SimpleTree::new(vec![
        (node(0), None, "root-a".into()),
        (node(1), None, "root-b".into()),
        (node(2), Some(node(0)), "child".into()),
    ]);
    assert_eq!(tree.root_count(), 2);
    assert_eq!(tree.root(0), node(0));
    assert_eq!(tree.root(1), node(1));
    assert_eq!(tree.node_label(node(0)), "root-a");
    assert_eq!(tree.node_label(node(1)), "root-b");
}

#[test]
fn visible_rows_all_collapsed() {
    let tree = sample_tree();
    let expanded = std::collections::BTreeSet::new();
    let rows = compute_visible_rows(&tree, &expanded);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].node_id, node(0));
    assert_eq!(rows[0].depth, 0);
    assert!(rows[0].has_children);
    assert!(!rows[0].is_expanded);
    assert!(rows[0].is_last_sibling);
}

#[test]
fn visible_rows_expand_root() {
    let tree = sample_tree();
    let expanded = std::collections::BTreeSet::from([node(0)]);
    let rows = compute_visible_rows(&tree, &expanded);

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].node_id, node(0));
    assert!(rows[0].is_expanded);
    assert_eq!(rows[1].node_id, node(1));
    assert_eq!(rows[1].depth, 1);
    assert!(!rows[1].is_last_sibling);
    assert!(rows[1].has_children);
    assert_eq!(rows[2].node_id, node(2));
    assert_eq!(rows[2].depth, 1);
    assert!(rows[2].is_last_sibling);
}

#[test]
fn visible_rows_nested_expand() {
    let tree = sample_tree();
    let expanded = std::collections::BTreeSet::from([node(0), node(1)]);
    let rows = compute_visible_rows(&tree, &expanded);

    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].node_id, node(0));
    assert_eq!(rows[1].node_id, node(1));
    assert!(rows[1].is_expanded);
    assert_eq!(rows[2].node_id, node(3)); // a1
    assert_eq!(rows[2].depth, 2);
    assert!(!rows[2].is_last_sibling);
    assert_eq!(rows[3].node_id, node(4)); // a2
    assert_eq!(rows[3].depth, 2);
    assert!(rows[3].is_last_sibling);
    assert_eq!(rows[4].node_id, node(2)); // b
    assert_eq!(rows[4].depth, 1);
}

#[test]
fn visible_rows_collapse_hides_descendants() {
    let tree = sample_tree();

    let expanded_all = std::collections::BTreeSet::from([node(0), node(1), node(2)]);
    let rows_all = compute_visible_rows(&tree, &expanded_all);
    assert_eq!(rows_all.len(), 6);

    let expanded_partial = std::collections::BTreeSet::from([node(0), node(2)]);
    let rows_partial = compute_visible_rows(&tree, &expanded_partial);
    assert_eq!(rows_partial.len(), 4);
    let ids: Vec<NodeId> = rows_partial.iter().map(|row| row.node_id).collect();
    assert_eq!(ids, vec![node(0), node(1), node(2), node(5)]);
}

#[test]
fn visible_rows_multiple_roots() {
    let tree = SimpleTree::new(vec![(node(0), None, "r1".into()), (node(1), None, "r2".into())]);
    let rows = compute_visible_rows(&tree, &std::collections::BTreeSet::new());
    assert_eq!(rows.len(), 2);
    assert!(!rows[0].is_last_sibling);
    assert!(rows[1].is_last_sibling);
}

#[test]
fn visible_rows_empty_tree() {
    let tree = SimpleTree::new(vec![]);
    let rows = compute_visible_rows(&tree, &std::collections::BTreeSet::new());
    assert!(rows.is_empty());
}

#[test]
fn visible_rows_expand_leaf_is_noop() {
    let tree = sample_tree();
    let expanded = std::collections::BTreeSet::from([node(0), node(1), node(3)]);
    let rows = compute_visible_rows(&tree, &expanded);
    let expected = std::collections::BTreeSet::from([node(0), node(1)]);
    let rows_expected = compute_visible_rows(&tree, &expected);
    assert_eq!(rows.len(), rows_expected.len());
}

#[test]
fn ancestors_last_tracking() {
    let tree = sample_tree();
    let expanded = std::collections::BTreeSet::from([node(0), node(1)]);
    let rows = compute_visible_rows(&tree, &expanded);

    assert!(rows[0].ancestors_last.is_empty());
    assert_eq!(rows[1].ancestors_last, vec![true]);
    assert_eq!(rows[2].ancestors_last, vec![true, false]);
    assert_eq!(rows[3].ancestors_last, vec![true, false]);
    assert_eq!(rows[4].ancestors_last, vec![true]);
}
