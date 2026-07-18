use super::*;

/// Floating panel state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloatingPanel {
    /// Floating panel ID.
    pub panel: PanelId,
    /// Floating panel rectangle.
    pub rect: DockRect,
}

impl FloatingPanel {
    /// Creates a floating panel with the default rectangle.
    #[must_use]
    pub fn new(panel: impl Into<PanelId>) -> Self {
        Self {
            panel: panel.into(),
            rect: DockRect::new(
                DEFAULT_FLOAT_X,
                DEFAULT_FLOAT_Y,
                DEFAULT_FLOAT_WIDTH,
                DEFAULT_FLOAT_HEIGHT,
            ),
        }
    }

    /// Sets the floating rectangle.
    #[must_use]
    pub fn with_rect(mut self, rect: DockRect) -> Self {
        self.rect = rect;
        self
    }
}

/// Docking layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockLayout {
    /// Layout schema version.
    pub version: u16,
    /// Root panel tree.
    pub root: DockNode,
    /// Floating panels.
    pub floating: Vec<FloatingPanel>,
    /// Focused panel.
    pub focus: Option<PanelId>,
    /// Panel constraints.
    pub constraints: std::collections::BTreeMap<PanelId, PanelConstraints>,
}

impl DockLayout {
    /// Creates a layout with a single root panel.
    #[must_use]
    pub fn new(root_panel: impl Into<PanelId>) -> Self {
        let panel = root_panel.into();
        Self {
            version: CURRENT_LAYOUT_VERSION,
            root: DockNode::leaf(panel.clone()),
            floating: Vec::new(),
            focus: Some(panel),
            constraints: std::collections::BTreeMap::new(),
        }
    }

    /// Returns all docked and floating panels in deterministic order.
    #[must_use]
    pub fn panel_ids(&self) -> Vec<PanelId> {
        let mut panels = Vec::new();
        self.root.collect_panels(&mut panels);
        panels.extend(self.floating.iter().map(|floating| floating.panel.clone()));
        panels.sort();
        panels
    }

    /// Focuses an existing panel.
    ///
    /// # Errors
    ///
    /// Returns [`DockingError::UnknownPanel`] when the panel is not in the layout.
    pub fn focus_panel(mut self, panel: &str) -> Result<Self, DockingError> {
        if !self.contains_panel(panel) {
            return Err(DockingError::UnknownPanel(panel.to_string()));
        }
        self.root.activate_tab(panel);
        self.focus = Some(panel.to_string());
        Ok(self)
    }

    /// Splits the focused panel and inserts `new_panel` as the second child.
    ///
    /// # Errors
    ///
    /// Returns [`DockingError`] when there is no focus, the focused panel is
    /// unknown, the new panel already exists, or the ratio is invalid.
    pub fn split_focused(
        mut self,
        new_panel: impl Into<PanelId>,
        axis: DockAxis,
        ratio: SplitRatio,
    ) -> Result<Self, DockingError> {
        let new_panel = new_panel.into();
        self.require_new_panel(&new_panel)?;
        let focus = self.focus.clone().ok_or(DockingError::NoFocus)?;
        let replacement = DockNode::Split {
            axis,
            ratio,
            first: Box::new(DockNode::leaf(focus.clone())),
            second: Box::new(DockNode::leaf(new_panel.clone())),
        };
        if !self.root.replace_panel(&focus, replacement) {
            return Err(DockingError::UnknownPanel(focus));
        }
        self.focus = Some(new_panel);
        Ok(self)
    }

    /// Tabs `new_panel` with the focused panel.
    ///
    /// # Errors
    ///
    /// Returns [`DockingError`] when there is no focus, the focused panel is
    /// unknown, or the new panel already exists.
    pub fn tab_focused(mut self, new_panel: impl Into<PanelId>) -> Result<Self, DockingError> {
        let new_panel = new_panel.into();
        self.require_new_panel(&new_panel)?;
        let focus = self.focus.clone().ok_or(DockingError::NoFocus)?;
        if !self.root.add_tab(&focus, new_panel.clone()) {
            return Err(DockingError::UnknownPanel(focus));
        }
        self.focus = Some(new_panel);
        Ok(self)
    }

    /// Adds or replaces a floating panel.
    ///
    /// # Errors
    ///
    /// Returns [`DockingError::DuplicatePanel`] when the panel already exists.
    pub fn float_panel(mut self, floating: FloatingPanel) -> Result<Self, DockingError> {
        self.require_new_panel(&floating.panel)?;
        self.focus = Some(floating.panel.clone());
        self.floating.push(floating);
        Ok(self)
    }

    /// Resizes the root split.
    ///
    /// # Errors
    ///
    /// Returns [`DockingError::NotSplitRoot`] when the root is not a split.
    pub fn resize_root_split(mut self, ratio: SplitRatio) -> Result<Self, DockingError> {
        self.root.resize_root(ratio)?;
        Ok(self)
    }

    /// Creates a versioned snapshot.
    #[must_use]
    pub fn snapshot(&self) -> DockLayoutSnapshot {
        DockLayoutSnapshot {
            version: self.version,
            root: self.root.clone(),
            floating: self.floating.clone(),
            focus: self.focus.clone(),
            constraints: self.constraints.clone(),
        }
    }

    /// Restores a layout from a snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`DockingError`] for unsupported versions, duplicate panel IDs,
    /// missing focused panels, invalid tab groups, or impossible constraints.
    pub fn restore(snapshot: DockLayoutSnapshot) -> Result<Self, DockingError> {
        if snapshot.version != CURRENT_LAYOUT_VERSION {
            return Err(DockingError::UnsupportedVersion(snapshot.version));
        }
        let layout = Self {
            version: snapshot.version,
            root: snapshot.root,
            floating: snapshot.floating,
            focus: snapshot.focus,
            constraints: snapshot.constraints,
        };
        layout.validate()?;
        Ok(layout)
    }

    /// Validates the layout.
    ///
    /// # Errors
    ///
    /// Returns [`DockingError`] for duplicate panels, invalid tabs, missing focus,
    /// or impossible constraints.
    pub fn validate(&self) -> Result<(), DockingError> {
        validate_node(&self.root)?;
        let panels = self.panel_ids();
        let unique = panels.iter().collect::<std::collections::BTreeSet<_>>();
        if unique.len() != panels.len() {
            return Err(DockingError::DuplicatePanel(
                "duplicate panel id".to_string(),
            ));
        }
        if let Some(focus) = &self.focus
            && !self.contains_panel(focus)
        {
            return Err(DockingError::UnknownPanel(focus.clone()));
        }
        for (panel, constraints) in &self.constraints {
            if !self.contains_panel(panel) {
                return Err(DockingError::UnknownPanel(panel.clone()));
            }
            if constraints.min_width == 0 || constraints.min_height == 0 {
                return Err(DockingError::ConflictingConstraints(panel.clone()));
            }
        }
        Ok(())
    }

    /// Projects logical panel rectangles for a viewport.
    #[must_use]
    pub fn project_regions(&self, area: DockRect) -> std::collections::BTreeMap<PanelId, DockRect> {
        let mut regions = std::collections::BTreeMap::new();
        self.root.project_regions(area, &mut regions);
        let floating_region_bound = regions.len().saturating_add(self.floating.len());
        for floating in &self.floating {
            if regions.len() >= floating_region_bound {
                break;
            }
            regions.insert(floating.panel.clone(), floating.rect);
        }
        regions
    }

    fn contains_panel(&self, panel: &str) -> bool {
        self.root.contains_panel(panel)
            || self.floating.iter().any(|floating| floating.panel == panel)
    }

    fn require_new_panel(&self, panel: &str) -> Result<(), DockingError> {
        if self.contains_panel(panel) {
            return Err(DockingError::DuplicatePanel(panel.to_string()));
        }
        Ok(())
    }
}

/// Versioned docking snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockLayoutSnapshot {
    /// Snapshot schema version.
    pub version: u16,
    /// Root tree.
    pub root: DockNode,
    /// Floating panels.
    pub floating: Vec<FloatingPanel>,
    /// Focused panel.
    pub focus: Option<PanelId>,
    /// Panel constraints.
    pub constraints: std::collections::BTreeMap<PanelId, PanelConstraints>,
}

fn validate_node(node: &DockNode) -> Result<(), DockingError> {
    let mut pending = Vec::with_capacity(1);
    pending.push(node);
    debug_assert_eq!(pending.len(), 1);
    debug_assert!(pending.capacity() >= pending.len());
    while let Some(current) = pending.pop() {
        match current {
            DockNode::Leaf(panel) => {
                if panel.is_empty() {
                    return Err(DockingError::UnknownPanel(panel.clone()));
                }
            }
            DockNode::Split { first, second, .. } => {
                pending.push(first);
                pending.push(second);
            }
            DockNode::Tabs { panels, active } => {
                if panels.is_empty() || !panels.iter().any(|panel| panel == active) {
                    return Err(DockingError::InvalidTabs(active.clone()));
                }
            }
        }
    }
    Ok(())
}
