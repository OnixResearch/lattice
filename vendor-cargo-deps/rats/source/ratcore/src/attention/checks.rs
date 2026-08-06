use super::*;

impl FocusGraph {
    /// Validates the graph shape and active node.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError`] for unknown scopes, missing labels, disabled active
    /// nodes, hidden active nodes, and modal-scope inconsistencies.
    pub fn validate(&self) -> Result<(), FocusError> {
        for node in self.nodes.values() {
            if !self.scopes.contains_key(&node.scope) {
                return Err(FocusError::new(
                    FocusErrorCode::UnknownScope,
                    Some(FocusErrorId::Scope(node.scope.clone())),
                    "node references unknown focus scope",
                ));
            }
            if node.label.trim().is_empty() {
                return Err(FocusError::new(
                    FocusErrorCode::MissingLabel,
                    Some(FocusErrorId::Node(node.id.clone())),
                    "focus node label must not be empty",
                ));
            }
        }
        if let Some(active_id) = &self.active {
            let active = self.nodes.get(active_id).ok_or_else(|| {
                FocusError::new(
                    FocusErrorCode::UnknownNode,
                    Some(FocusErrorId::Node(active_id.clone())),
                    "active focus node is missing",
                )
            })?;
            if active.disabled {
                return Err(FocusError::new(
                    FocusErrorCode::ActiveDisabled,
                    Some(FocusErrorId::Node(active_id.clone())),
                    "active focus node is disabled",
                ));
            }
            if active.hidden {
                return Err(FocusError::new(
                    FocusErrorCode::ActiveHidden,
                    Some(FocusErrorId::Node(active_id.clone())),
                    "active focus node is hidden",
                ));
            }
        }
        if let Some(scope_id) = &self.modal_scope
            && !self.scopes.contains_key(scope_id)
        {
            return Err(FocusError::new(
                FocusErrorCode::UnknownScope,
                Some(FocusErrorId::Scope(scope_id.clone())),
                "modal focus scope is missing",
            ));
        }
        Ok(())
    }
}
