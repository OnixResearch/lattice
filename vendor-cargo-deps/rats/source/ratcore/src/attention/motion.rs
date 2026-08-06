use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Next,
    Previous,
}

impl FocusGraph {
    /// Moves to the next eligible node.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError::NoEligibleTarget`] when traversal scope has no
    /// eligible nodes, or another validation error when the graph is invalid.
    pub fn move_next(mut self) -> Result<(Self, FocusTransition), FocusError> {
        self.move_by(Direction::Next)
    }

    /// Moves to the previous eligible node.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError::NoEligibleTarget`] when traversal scope has no
    /// eligible nodes, or another validation error when the graph is invalid.
    pub fn move_previous(mut self) -> Result<(Self, FocusTransition), FocusError> {
        self.move_by(Direction::Previous)
    }

    fn move_by(&mut self, direction: Direction) -> Result<(Self, FocusTransition), FocusError> {
        self.validate()?;
        let eligible = self.eligible_ids();
        if eligible.is_empty() {
            return Err(FocusError::new(FocusErrorCode::NoEligibleTarget, None, "no eligible focus target"));
        }
        let previous = self.active.clone();
        let current_index = previous.as_ref().and_then(|id| eligible.iter().position(|candidate| candidate == id));
        let next_index = next_index(current_index, eligible.len(), direction);
        self.active = Some(eligible[next_index].clone());
        let transition = self.transition(previous);
        Ok((self.clone(), transition))
    }

    pub(super) fn eligible_ids(&self) -> Vec<FocusNodeId> {
        self.order
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .filter(|node| node.eligible())
            .filter(|node| self.node_allowed_by_modal(node))
            .map(|node| node.id.clone())
            .collect()
    }

    pub(super) fn node_allowed_by_modal(&self, node: &FocusNode) -> bool {
        self.modal_scope.as_ref().is_none_or(|scope_id| self.scope_contains(scope_id, &node.scope))
    }

    fn scope_contains(&self, ancestor: &FocusScopeId, child: &FocusScopeId) -> bool {
        if ancestor == child {
            return true;
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut cursor = Some(child.clone());
        while let Some(scope_id) = cursor.as_ref() {
            if !seen.insert(scope_id.clone()) {
                return false;
            }
            let Some(scope) = self.scopes.get(scope_id) else {
                return false;
            };
            if scope.parent.as_ref() == Some(ancestor) {
                return true;
            }
            cursor.clone_from(&scope.parent);
        }
        false
    }
}

fn next_index(current_index: Option<usize>, len: usize, direction: Direction) -> usize {
    if len == 0 {
        return 0;
    }
    let Some(index) = current_index else {
        return 0;
    };
    debug_assert!(len > 0);
    debug_assert!(len.saturating_sub(1) < len);
    match direction {
        Direction::Next => match index.saturating_add(1).checked_rem(len) {
            Some(next) => next,
            None => 0,
        },
        Direction::Previous => {
            if index == 0 {
                len.saturating_sub(1)
            } else {
                index.saturating_sub(1)
            }
        }
    }
}
