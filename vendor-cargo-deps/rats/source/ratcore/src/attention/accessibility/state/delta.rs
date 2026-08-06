/// Accepted semantic tree state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticTree {
    accepted: super::super::SemanticSnapshot,
}

/// Result of a full replacement or incremental update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticChange {
    /// Whether the candidate replaced accepted state.
    pub accepted: bool,
    /// Generation that remains accepted after the operation.
    pub generation: super::super::SemanticGeneration,
    /// Stable ordered rejection diagnostics.
    pub issues: Vec<super::super::SemanticIssue>,
}

impl SemanticTree {
    /// Validates and accepts an initial complete snapshot.
    ///
    /// # Errors
    ///
    /// Returns ordered validation issues when the initial snapshot is invalid.
    pub fn new(
        initial: super::super::SemanticSnapshot,
        focus_graph: Option<&super::super::super::FocusGraph>,
    ) -> Result<Self, super::super::SemanticValidation> {
        let validation = super::super::validate_semantic_snapshot(&initial, focus_graph);
        if validation.is_valid() {
            Ok(Self { accepted: initial })
        } else {
            Err(validation)
        }
    }

    /// Returns the current accepted complete snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &super::super::SemanticSnapshot {
        &self.accepted
    }

    /// Atomically replaces the accepted complete snapshot.
    pub fn replace(
        &mut self,
        candidate: super::super::SemanticSnapshot,
        focus_graph: Option<&super::super::super::FocusGraph>,
    ) -> SemanticChange {
        if candidate.generation <= self.accepted.generation {
            return rejected(
                self.accepted.generation,
                super::super::SemanticIssue::new(
                    super::super::SemanticIssueCode::InvalidGeneration,
                    candidate.generation.value().to_string(),
                    "full semantic snapshot generation must advance",
                ),
            );
        }
        self.accept_candidate(candidate, focus_graph)
    }

    /// Atomically applies generation-checked node operations.
    pub fn apply(
        &mut self,
        update: &SemanticUpdate,
        focus_graph: Option<&super::super::super::FocusGraph>,
    ) -> SemanticChange {
        if let Some(issue) = generation_issue(self.accepted.generation, update) {
            return rejected(self.accepted.generation, issue);
        }
        let mut candidate = self.accepted.clone();
        candidate.generation = update.generation;
        let operation_issues = apply_operations(&mut candidate, &update.operations);
        if operation_issues.is_empty() {
            self.accept_candidate(candidate, focus_graph)
        } else {
            SemanticChange {
                accepted: false,
                generation: self.accepted.generation,
                issues: operation_issues,
            }
        }
    }

    fn accept_candidate(
        &mut self,
        candidate: super::super::SemanticSnapshot,
        focus_graph: Option<&super::super::super::FocusGraph>,
    ) -> SemanticChange {
        let validation = super::super::validate_semantic_snapshot(&candidate, focus_graph);
        if !validation.is_valid() {
            return SemanticChange {
                accepted: false,
                generation: self.accepted.generation,
                issues: validation.issues,
            };
        }
        self.accepted = candidate;
        SemanticChange {
            accepted: true,
            generation: self.accepted.generation,
            issues: Vec::new(),
        }
    }
}

/// One atomic semantic update batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticUpdate {
    /// Generation observed by the producer.
    pub base_generation: super::super::SemanticGeneration,
    /// Generation to publish after acceptance.
    pub generation: super::super::SemanticGeneration,
    /// Ordered node operations.
    pub operations: Vec<SemanticUpdateOperation>,
}

/// One node operation in an update batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticUpdateOperation {
    /// Add a node that does not exist.
    Add(super::super::SemanticNode),
    /// Replace a node that exists.
    Replace(super::super::SemanticNode),
    /// Change parent and sibling order.
    Reorder {
        /// Existing node identity.
        node: super::super::SemanticNodeId,
        /// New optional parent.
        parent: Option<super::super::SemanticNodeId>,
        /// New sibling order.
        order: u64,
    },
    /// Remove an existing node.
    Remove(super::super::SemanticNodeId),
}

impl SemanticUpdateOperation {
    fn node_id(&self) -> &super::super::SemanticNodeId {
        match self {
            Self::Add(node) | Self::Replace(node) => &node.id,
            Self::Reorder { node, .. } | Self::Remove(node) => node,
        }
    }
}

fn generation_issue(
    accepted: super::super::SemanticGeneration,
    update: &SemanticUpdate,
) -> Option<super::super::SemanticIssue> {
    if update.base_generation != accepted {
        return Some(super::super::SemanticIssue::new(
            super::super::SemanticIssueCode::StaleGeneration,
            update.base_generation.value().to_string(),
            "semantic update base generation is stale",
        ));
    }
    (update.generation <= update.base_generation).then(|| {
        super::super::SemanticIssue::new(
            super::super::SemanticIssueCode::InvalidGeneration,
            update.generation.value().to_string(),
            "semantic update generation must advance",
        )
    })
}

fn apply_operations(
    candidate: &mut super::super::SemanticSnapshot,
    operations: &[SemanticUpdateOperation],
) -> Vec<super::super::SemanticIssue> {
    let mut issues = Vec::with_capacity(operations.len());
    let mut targets = std::collections::BTreeSet::new();
    for operation in operations {
        if !targets.insert(operation.node_id().clone()) {
            issues.push(super::super::SemanticIssue::new(
                super::super::SemanticIssueCode::DuplicateOperation,
                operation.node_id().as_str(),
                "semantic update targets a node more than once",
            ));
        } else if let Err(issue) = apply_operation(candidate, operation) {
            issues.push(issue);
        }
    }
    issues.sort();
    issues
}

fn apply_operation(
    candidate: &mut super::super::SemanticSnapshot,
    operation: &SemanticUpdateOperation,
) -> Result<(), super::super::SemanticIssue> {
    match operation {
        SemanticUpdateOperation::Add(node) => add_node(candidate, node),
        SemanticUpdateOperation::Replace(node) => replace_node(candidate, node),
        SemanticUpdateOperation::Reorder { node, parent, order } => {
            reorder_node(candidate, node, parent.as_ref(), *order)
        }
        SemanticUpdateOperation::Remove(node) => remove_node(candidate, node),
    }
}

fn add_node(
    candidate: &mut super::super::SemanticSnapshot,
    node: &super::super::SemanticNode,
) -> Result<(), super::super::SemanticIssue> {
    if candidate.node(&node.id).is_some() {
        return Err(operation_issue(
            super::super::SemanticIssueCode::DuplicateNode,
            &node.id,
            "semantic update adds an existing node",
        ));
    }
    candidate.nodes.push(node.clone());
    Ok(())
}

fn replace_node(
    candidate: &mut super::super::SemanticSnapshot,
    node: &super::super::SemanticNode,
) -> Result<(), super::super::SemanticIssue> {
    let index = node_index(candidate, &node.id)?;
    candidate.nodes[index] = node.clone();
    Ok(())
}

fn reorder_node(
    candidate: &mut super::super::SemanticSnapshot,
    node: &super::super::SemanticNodeId,
    parent: Option<&super::super::SemanticNodeId>,
    order: u64,
) -> Result<(), super::super::SemanticIssue> {
    let index = node_index(candidate, node)?;
    candidate.nodes[index].parent = parent.cloned();
    candidate.nodes[index].order = order;
    Ok(())
}

fn remove_node(
    candidate: &mut super::super::SemanticSnapshot,
    node: &super::super::SemanticNodeId,
) -> Result<(), super::super::SemanticIssue> {
    let index = node_index(candidate, node)?;
    candidate.nodes.remove(index);
    Ok(())
}

fn node_index(
    candidate: &super::super::SemanticSnapshot,
    node: &super::super::SemanticNodeId,
) -> Result<usize, super::super::SemanticIssue> {
    candidate.nodes.iter().position(|candidate_node| candidate_node.id == *node).ok_or_else(|| {
        operation_issue(
            super::super::SemanticIssueCode::UnknownNode,
            node,
            "semantic update references an unknown node",
        )
    })
}

fn operation_issue(
    code: super::super::SemanticIssueCode,
    node: &super::super::SemanticNodeId,
    message: &'static str,
) -> super::super::SemanticIssue {
    super::super::SemanticIssue::new(code, node.as_str(), message)
}

fn rejected(generation: super::super::SemanticGeneration, issue: super::super::SemanticIssue) -> SemanticChange {
    SemanticChange {
        accepted: false,
        generation,
        issues: vec![issue],
    }
}
