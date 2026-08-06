use super::*;

/// Meteorite/web accessibility projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AriaProjection {
    /// Active node ID.
    pub active_id: FocusNodeId,
    /// ARIA-like role label.
    pub role: String,
    /// Accessible label.
    pub label: String,
    /// Disabled state.
    pub aria_disabled: bool,
    /// Hidden state.
    pub aria_hidden: bool,
    /// Live-region announcement text.
    pub live_region: String,
}

/// Subwayrat/TUI focus projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiFocusProjection {
    /// Active node ID.
    pub active_id: FocusNodeId,
    /// Focus label.
    pub label: String,
    /// Renderer-neutral style state token.
    pub style_state: String,
    /// Status or announcement text.
    pub status_text: String,
    /// Shortcut hints.
    pub shortcuts: Vec<String>,
}

impl FocusGraph {
    /// Returns the active node as Meteorite/web accessibility metadata.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError`] when the graph is invalid or has no active node.
    pub fn aria_projection(&self) -> Result<AriaProjection, FocusError> {
        let node = self.active_node()?;
        Ok(AriaProjection {
            active_id: node.id.clone(),
            role: node.role.label().to_string(),
            label: node.label.clone(),
            aria_disabled: node.disabled,
            aria_hidden: node.hidden,
            live_region: Self::announcement_for(node),
        })
    }

    /// Returns the active node as Subwayrat/TUI focus metadata.
    ///
    /// # Errors
    ///
    /// Returns [`FocusError`] when the graph is invalid or has no active node.
    pub fn tui_projection(&self) -> Result<TuiFocusProjection, FocusError> {
        let node = self.active_node()?;
        Ok(TuiFocusProjection {
            active_id: node.id.clone(),
            label: node.label.clone(),
            style_state: STYLE_FOCUSED.to_string(),
            status_text: Self::announcement_for(node),
            shortcuts: node.shortcuts.clone(),
        })
    }

    fn active_node(&self) -> Result<&FocusNode, FocusError> {
        self.validate()?;
        let active_id = self
            .active
            .as_ref()
            .ok_or_else(|| FocusError::new(FocusErrorCode::NoEligibleTarget, None, "no active focus node"))?;
        self.nodes.get(active_id).ok_or_else(|| {
            FocusError::new(
                FocusErrorCode::UnknownNode,
                Some(FocusErrorId::Node(active_id.clone())),
                "active focus node is missing",
            )
        })
    }
}
