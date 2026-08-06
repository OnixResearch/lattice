//! Subwayrat structured semantic summaries.

const TERMINAL_LIMITATION: &str =
    "terminal summaries are not a native accessibility tree or screen-reader runtime receipt";

/// Checked Subwayrat semantic summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSemanticProjection {
    /// Stable role, name, state, relationship, and action summaries.
    pub nodes: Vec<TerminalSemanticNode>,
    /// Structured live/status announcements.
    pub announcements: Vec<String>,
    /// Shared validation diagnostics.
    pub issues: Vec<ratcore::focus::SemanticIssue>,
    /// Explicit platform limitation.
    pub limitation: String,
}

/// One terminal-friendly semantic node summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSemanticNode {
    /// Stable shared semantic identity.
    pub id: ratcore::focus::SemanticNodeId,
    /// Bounded shared role name.
    pub role: String,
    /// Accessible name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Stable state summaries.
    pub states: Vec<String>,
    /// Stable relationship summaries.
    pub relations: Vec<String>,
    /// Actions visible in shared state but unsupported as terminal accessibility events.
    pub actions: Vec<ratcore::focus::SemanticAction>,
}

/// Projects a semantic snapshot into terminal role/focus/status summaries.
#[must_use]
pub fn project_terminal_semantics(snapshot: &ratcore::focus::SemanticSnapshot) -> TerminalSemanticProjection {
    let summary =
        ratcore::focus::project_semantic_summary(snapshot, ratcore::focus::SemanticRendererTarget::SubwayratTerminal);
    if !summary.issues.is_empty() {
        return invalid_projection(summary.issues);
    }
    TerminalSemanticProjection {
        nodes: summary.nodes.into_iter().map(terminal_node).collect(),
        announcements: announcements(snapshot),
        issues: Vec::new(),
        limitation: TERMINAL_LIMITATION.to_string(),
    }
}

fn invalid_projection(issues: Vec<ratcore::focus::SemanticIssue>) -> TerminalSemanticProjection {
    TerminalSemanticProjection {
        nodes: Vec::new(),
        announcements: Vec::new(),
        issues,
        limitation: TERMINAL_LIMITATION.to_string(),
    }
}

fn terminal_node(node: ratcore::focus::SemanticProjectionNode) -> TerminalSemanticNode {
    TerminalSemanticNode {
        id: node.id,
        role: node.role,
        name: node.name,
        description: node.description,
        states: node.states,
        relations: relation_summaries(node.relations),
        actions: node.actions,
    }
}

fn relation_summaries(relations: Vec<ratcore::focus::SemanticRelation>) -> Vec<String> {
    relations.into_iter().map(|relation| format!("{:?}={}", relation.kind, relation.target)).collect()
}

fn announcements(snapshot: &ratcore::focus::SemanticSnapshot) -> Vec<String> {
    snapshot
        .nodes
        .iter()
        .filter_map(|node| node.announcement.as_ref())
        .map(|announcement| announcement.message.clone())
        .collect()
}

#[cfg(test)]
mod tests;
