use super::*;

/// Pure focus graph.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FocusGraph {
    pub(super) scopes: std::collections::BTreeMap<String, FocusScope>,
    pub(super) nodes: std::collections::BTreeMap<String, FocusNode>,
    pub(super) order: Vec<String>,
    pub(super) active: Option<String>,
    pub(super) modal_scope: Option<String>,
    pub(super) restore_stack: Vec<String>,
}

impl FocusGraph {
    /// Creates an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces a scope.
    #[must_use]
    pub fn with_scope(mut self, scope: FocusScope) -> Self {
        self.scopes.insert(scope.id.clone(), scope);
        self
    }

    /// Adds or replaces a focus node.
    #[must_use]
    pub fn with_node(mut self, node: FocusNode) -> Self {
        if !self.order.iter().any(|id| id == &node.id) {
            self.order.push(node.id.clone());
        }
        self.nodes.insert(node.id.clone(), node);
        self
    }

    /// Returns the active node ID.
    #[must_use]
    pub fn active_id(&self) -> Option<&str> {
        self.active.as_deref()
    }

    /// Returns the active modal scope ID.
    #[must_use]
    pub fn modal_scope(&self) -> Option<&str> {
        self.modal_scope.as_deref()
    }

    /// Returns a node by ID.
    #[must_use]
    pub fn node(&self, id: &str) -> Option<&FocusNode> {
        self.nodes.get(id)
    }

    /// Returns a scope by ID.
    #[must_use]
    pub fn scope(&self, id: &str) -> Option<&FocusScope> {
        self.scopes.get(id)
    }

    /// Activates the first eligible node in the current traversal scope.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError::NoEligibleTarget`] when no eligible node exists, or
    /// another validation error when the graph is invalid.
    pub fn activate_first(mut self) -> Result<Self, FocusError> {
        self.validate()?;
        let target = self.eligible_ids().into_iter().next().ok_or_else(|| {
            FocusError::new(
                FocusErrorCode::NoEligibleTarget,
                None,
                "no eligible focus target",
            )
        })?;
        self.active = Some(target);
        Ok(self)
    }

    /// Focuses a specific node if it is eligible and inside any active modal trap.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError`] when the target is missing, disabled, hidden, or
    /// outside the active modal scope.
    pub fn focus_node(mut self, id: &str) -> Result<(Self, FocusTransition), FocusError> {
        self.validate()?;
        let target = self.nodes.get(id).ok_or_else(|| {
            FocusError::new(
                FocusErrorCode::UnknownNode,
                Some(id.to_string()),
                "focus target is missing",
            )
        })?;
        if !target.eligible() {
            return Err(FocusError::new(
                FocusErrorCode::NoEligibleTarget,
                Some(id.to_string()),
                "focus target is disabled or hidden",
            ));
        }
        if !self.node_allowed_by_modal(target) {
            return Err(FocusError::new(
                FocusErrorCode::ModalEscape,
                Some(id.to_string()),
                "focus target is outside active modal scope",
            ));
        }
        let previous = self.active.clone();
        self.active = Some(id.to_string());
        let transition = self.transition(previous);
        Ok((self, transition))
    }

    /// Enters a modal scope and focuses the first eligible node inside it.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError`] when the scope is missing or has no eligible target.
    pub fn enter_modal(mut self, scope_id: &str) -> Result<(Self, FocusTransition), FocusError> {
        self.validate()?;
        if !self.scopes.contains_key(scope_id) {
            return Err(FocusError::new(
                FocusErrorCode::UnknownScope,
                Some(scope_id.to_string()),
                "modal scope is missing",
            ));
        }
        if let Some(active) = &self.active {
            self.restore_stack.push(active.clone());
        }
        self.modal_scope = Some(scope_id.to_string());
        let previous = self.active.clone();
        let target = self.eligible_ids().into_iter().next().ok_or_else(|| {
            FocusError::new(
                FocusErrorCode::NoEligibleTarget,
                Some(scope_id.to_string()),
                "modal scope has no eligible focus target",
            )
        })?;
        self.active = Some(target);
        let transition = self.transition(previous);
        Ok((self, transition))
    }

    /// Leaves the active modal scope and restores prior focus when possible.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError::NoEligibleTarget`] when no restore target and no
    /// fallback eligible node exist.
    pub fn leave_modal(mut self) -> Result<(Self, FocusTransition), FocusError> {
        self.modal_scope = None;
        let previous = self.active.clone();
        let restore = self.restore_stack.pop();
        let target = restore
            .filter(|id| self.nodes.get(id).is_some_and(FocusNode::eligible))
            .or_else(|| self.eligible_ids().into_iter().next())
            .ok_or_else(|| {
                FocusError::new(
                    FocusErrorCode::NoEligibleTarget,
                    None,
                    "no restore focus target",
                )
            })?;
        self.active = Some(target);
        let transition = self.transition(previous);
        Ok((self, transition))
    }

    pub(super) fn transition(&self, previous: Option<String>) -> FocusTransition {
        FocusTransition {
            previous,
            active: self.active.clone(),
            announcement: self
                .active
                .as_ref()
                .and_then(|id| self.nodes.get(id))
                .map(Self::announcement_for),
        }
    }

    pub(super) fn announcement_for(node: &FocusNode) -> String {
        format!("{}{}{}", node.role.label(), STATUS_SEPARATOR, node.label)
    }
}
