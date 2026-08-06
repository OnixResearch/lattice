pub(super) type SurfaceMap<'a> = std::collections::BTreeMap<&'a str, &'a super::super::SemanticSurface>;
pub(super) type NodeMap<'a> = std::collections::BTreeMap<super::super::SemanticNodeId, &'a super::super::SemanticNode>;

pub(super) fn validate_surfaces(
    snapshot: &super::super::SemanticSnapshot,
) -> (SurfaceMap<'_>, Vec<super::super::SemanticIssue>) {
    let mut surfaces = SurfaceMap::new();
    let mut issues = Vec::with_capacity(snapshot.surfaces.len());
    for surface in &snapshot.surfaces {
        issues.extend(surface_identity_issues(surface));
        if surfaces.insert(surface.id.as_str(), surface).is_some() {
            issues.push(super::issue(
                super::super::SemanticIssueCode::DuplicateSurface,
                surface.id.as_str(),
                "surface identity is duplicated",
            ));
        }
    }
    (surfaces, issues)
}

fn surface_identity_issues(surface: &super::super::SemanticSurface) -> Vec<super::super::SemanticIssue> {
    let is_identity_empty = surface.id.as_str().trim().is_empty() || surface.root.as_str().trim().is_empty();
    if is_identity_empty {
        vec![super::issue(
            super::super::SemanticIssueCode::EmptyIdentity,
            surface.id.as_str(),
            "surface identity and root must not be empty",
        )]
    } else {
        Vec::new()
    }
}

pub(super) fn validate_nodes<'a>(
    snapshot: &'a super::super::SemanticSnapshot,
    surfaces: &SurfaceMap<'_>,
) -> (NodeMap<'a>, Vec<super::super::SemanticIssue>) {
    let mut nodes = NodeMap::new();
    let mut issues = Vec::with_capacity(snapshot.nodes.len());
    for node in &snapshot.nodes {
        issues.extend(node_shape_issues(node, surfaces));
        issues.extend(super::content::node_issues(node));
        if nodes.insert(node.id.clone(), node).is_some() {
            issues.push(super::issue(
                super::super::SemanticIssueCode::DuplicateNode,
                node.id.as_str(),
                "semantic node identity is duplicated",
            ));
        }
    }
    issues.extend(root_issues(surfaces, &nodes));
    (nodes, issues)
}

fn node_shape_issues(node: &super::super::SemanticNode, surfaces: &SurfaceMap<'_>) -> Vec<super::super::SemanticIssue> {
    let mut issues = Vec::with_capacity(2);
    if node.id.as_str().trim().is_empty() {
        issues.push(super::issue(
            super::super::SemanticIssueCode::EmptyIdentity,
            node.id.as_str(),
            "semantic node identity must not be empty",
        ));
    }
    if !surfaces.contains_key(node.surface.as_str()) {
        issues.push(super::issue(
            super::super::SemanticIssueCode::MissingSurface,
            node.id.as_str(),
            "semantic node references a missing surface",
        ));
    }
    issues
}

fn root_issues(surfaces: &SurfaceMap<'_>, nodes: &NodeMap<'_>) -> Vec<super::super::SemanticIssue> {
    surfaces
        .values()
        .filter(|surface| {
            nodes.get(&surface.root).is_none_or(|root| root.parent.is_some() || root.surface != surface.id)
        })
        .map(|surface| {
            super::issue(
                super::super::SemanticIssueCode::MissingRoot,
                surface.id.as_str(),
                "surface root is missing or invalid",
            )
        })
        .collect()
}

pub(super) fn validate_parentage(
    snapshot: &super::super::SemanticSnapshot,
    nodes: &NodeMap<'_>,
) -> Vec<super::super::SemanticIssue> {
    let roots = surface_roots(snapshot);
    let mut orders = std::collections::BTreeSet::new();
    let mut issues = Vec::with_capacity(nodes.len());
    for node in nodes.values() {
        issues.extend(parent_reference_issues(node, nodes, &roots, &mut orders));
        if parent_cycle_exists(node, nodes) {
            issues.push(super::issue(
                super::super::SemanticIssueCode::ParentCycle,
                node.id.as_str(),
                "semantic parent links contain a cycle",
            ));
        }
    }
    issues
}

fn surface_roots(
    snapshot: &super::super::SemanticSnapshot,
) -> std::collections::BTreeSet<&super::super::SemanticNodeId> {
    snapshot.surfaces.iter().map(|surface| &surface.root).collect()
}

fn parent_reference_issues(
    node: &super::super::SemanticNode,
    nodes: &NodeMap<'_>,
    roots: &std::collections::BTreeSet<&super::super::SemanticNodeId>,
    orders: &mut std::collections::BTreeSet<(super::super::SemanticSurfaceId, super::super::SemanticNodeId, u64)>,
) -> Vec<super::super::SemanticIssue> {
    let mut issues = Vec::with_capacity(2);
    if node.parent.is_none() && !roots.contains(&node.id) {
        issues.push(super::issue(
            super::super::SemanticIssueCode::InvalidRoot,
            node.id.as_str(),
            "only a declared surface root may omit its parent",
        ));
    }
    if let Some(parent) = &node.parent {
        issues.extend(parent_target_issues(node, parent, nodes, orders));
    }
    issues
}

fn parent_target_issues(
    node: &super::super::SemanticNode,
    parent: &super::super::SemanticNodeId,
    nodes: &NodeMap<'_>,
    orders: &mut std::collections::BTreeSet<(super::super::SemanticSurfaceId, super::super::SemanticNodeId, u64)>,
) -> Vec<super::super::SemanticIssue> {
    let mut issues = Vec::with_capacity(2);
    if !nodes.contains_key(parent) {
        issues.push(super::issue(
            super::super::SemanticIssueCode::MissingParent,
            node.id.as_str(),
            "semantic parent is missing",
        ));
    }
    if !orders.insert((node.surface.clone(), parent.clone(), node.order)) {
        issues.push(super::issue(
            super::super::SemanticIssueCode::DuplicateOrder,
            node.id.as_str(),
            "siblings must use distinct order values",
        ));
    }
    issues
}

fn parent_cycle_exists(node: &super::super::SemanticNode, nodes: &NodeMap<'_>) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    let mut cursor = node.parent.as_ref();
    while let Some(parent) = cursor {
        if !seen.insert(parent.clone()) {
            return true;
        }
        cursor = nodes.get(parent).and_then(|candidate| candidate.parent.as_ref());
    }
    false
}

pub(super) fn validate_relations(
    snapshot: &super::super::SemanticSnapshot,
    nodes: &NodeMap<'_>,
) -> Vec<super::super::SemanticIssue> {
    let mut issues = Vec::with_capacity(snapshot.nodes.len());
    for node in &snapshot.nodes {
        issues.extend(relation_record_issues(node, nodes));
        if relation_cycle_exists(node, nodes) {
            issues.push(super::issue(
                super::super::SemanticIssueCode::RelationCycle,
                node.id.as_str(),
                "semantic relationships contain a cycle",
            ));
        }
    }
    issues
}

fn relation_record_issues(node: &super::super::SemanticNode, nodes: &NodeMap<'_>) -> Vec<super::super::SemanticIssue> {
    let mut seen = std::collections::BTreeSet::new();
    node.relations
        .iter()
        .flat_map(|relation| {
            [
                missing_relation_issue(node, relation, nodes),
                duplicate_relation_issue(node, relation, &mut seen),
            ]
            .into_iter()
            .flatten()
        })
        .collect()
}

fn missing_relation_issue(
    node: &super::super::SemanticNode,
    relation: &super::super::SemanticRelation,
    nodes: &NodeMap<'_>,
) -> Option<super::super::SemanticIssue> {
    (!nodes.contains_key(&relation.target)).then(|| {
        super::issue(
            super::super::SemanticIssueCode::MissingRelation,
            node.id.as_str(),
            "semantic relationship target is missing",
        )
    })
}

fn duplicate_relation_issue(
    node: &super::super::SemanticNode,
    relation: &super::super::SemanticRelation,
    seen: &mut std::collections::BTreeSet<super::super::SemanticRelation>,
) -> Option<super::super::SemanticIssue> {
    (!seen.insert(relation.clone())).then(|| {
        super::issue(
            super::super::SemanticIssueCode::DuplicateRelation,
            node.id.as_str(),
            "semantic relationship is duplicated",
        )
    })
}

fn relation_cycle_exists(node: &super::super::SemanticNode, nodes: &NodeMap<'_>) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    let mut pending = relation_targets(node);
    while let Some(current) = pending.pop() {
        if current == node.id {
            return true;
        }
        if seen.insert(current.clone()) {
            pending.extend(
                nodes
                    .get(&current)
                    .into_iter()
                    .flat_map(|target| target.relations.iter().map(|relation| relation.target.clone())),
            );
        }
    }
    false
}

fn relation_targets(node: &super::super::SemanticNode) -> Vec<super::super::SemanticNodeId> {
    node.relations.iter().map(|relation| relation.target.clone()).collect()
}
