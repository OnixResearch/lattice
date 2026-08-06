pub(super) fn validate_links(
    snapshot: &super::super::SemanticSnapshot,
    nodes: &super::structure::NodeMap<'_>,
    focus_graph: Option<&super::super::super::FocusGraph>,
) -> Vec<super::super::SemanticIssue> {
    let mut seen = std::collections::BTreeSet::new();
    let mut issues = Vec::with_capacity(snapshot.focus_links.len());
    for link in &snapshot.focus_links {
        if !seen.insert(link.focus.clone()) {
            issues.push(super::issue(
                super::super::SemanticIssueCode::DuplicateFocusLink,
                link.focus.as_str(),
                "focus identity is linked more than once",
            ));
        }
        issues.extend(link_target_issues(link, nodes, focus_graph));
    }
    issues
}

fn link_target_issues(
    link: &super::super::SemanticFocusLink,
    nodes: &super::structure::NodeMap<'_>,
    focus_graph: Option<&super::super::super::FocusGraph>,
) -> Vec<super::super::SemanticIssue> {
    let mut issues = Vec::with_capacity(2);
    let semantic = nodes.get(&link.semantic);
    if semantic.is_none() {
        issues.push(super::issue(
            super::super::SemanticIssueCode::UnknownNode,
            link.semantic.as_str(),
            "focus link references a missing semantic node",
        ));
    }
    if let Some(graph) = focus_graph {
        issues.extend(focus_node_issues(link, semantic.copied(), graph));
    }
    issues
}

fn focus_node_issues(
    link: &super::super::SemanticFocusLink,
    semantic: Option<&super::super::SemanticNode>,
    graph: &super::super::super::FocusGraph,
) -> Vec<super::super::SemanticIssue> {
    let Some(focus) = graph.node(&link.focus) else {
        return vec![super::issue(
            super::super::SemanticIssueCode::MissingFocusNode,
            link.focus.as_str(),
            "semantic focus link references a missing focus node",
        )];
    };
    if semantic.is_some_and(|node| !roles_are_compatible(&focus.role, node.role)) {
        vec![super::issue(
            super::super::SemanticIssueCode::IncompatibleFocusRole,
            link.focus.as_str(),
            "focus and semantic roles are incompatible",
        )]
    } else {
        Vec::new()
    }
}

fn roles_are_compatible(focus: &super::super::super::FocusRole, semantic: super::super::SemanticRole) -> bool {
    match focus {
        super::super::super::FocusRole::Button => semantic == super::super::SemanticRole::Button,
        super::super::super::FocusRole::MenuItem => semantic == super::super::SemanticRole::MenuItem,
        super::super::super::FocusRole::Dialog => {
            matches!(semantic, super::super::SemanticRole::Dialog | super::super::SemanticRole::Button)
        }
        super::super::super::FocusRole::TreeItem => semantic == super::super::SemanticRole::TreeItem,
        super::super::super::FocusRole::ListItem => matches!(
            semantic,
            super::super::SemanticRole::MenuItem
                | super::super::SemanticRole::TreeItem
                | super::super::SemanticRole::Tab
        ),
        super::super::super::FocusRole::Panel => matches!(
            semantic,
            super::super::SemanticRole::Application
                | super::super::SemanticRole::Dialog
                | super::super::SemanticRole::Group
        ),
        super::super::super::FocusRole::TableCell
        | super::super::super::FocusRole::FormField
        | super::super::super::FocusRole::Custom(_) => false,
    }
}
