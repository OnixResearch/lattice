mod content;
mod focus;
mod structure;

/// Validates one complete semantic snapshot and optional focus graph links.
#[must_use]
pub fn validate_semantic_snapshot(
    snapshot: &super::SemanticSnapshot,
    focus_graph: Option<&super::super::FocusGraph>,
) -> super::SemanticValidation {
    let (surfaces, mut issues) = structure::validate_surfaces(snapshot);
    let (nodes, mut node_issues) = structure::validate_nodes(snapshot, &surfaces);
    issues.append(&mut node_issues);
    issues.append(&mut structure::validate_parentage(snapshot, &nodes));
    issues.append(&mut structure::validate_relations(snapshot, &nodes));
    issues.append(&mut focus::validate_links(snapshot, &nodes, focus_graph));
    issues.sort();
    issues.dedup();
    super::SemanticValidation { issues }
}

pub(super) fn issue(
    code: super::SemanticIssueCode,
    subject: impl Into<String>,
    message: &'static str,
) -> super::SemanticIssue {
    super::SemanticIssue::new(code, subject, message)
}
