/// Floating panel state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloatingPanel {
    /// Floating panel ID.
    pub panel: super::PanelId,
    /// Floating panel rectangle.
    pub rect: super::DockRect,
}

impl FloatingPanel {
    /// Creates a floating panel with the default rectangle.
    #[must_use]
    pub fn new(panel: impl Into<super::PanelId>) -> Self {
        Self {
            panel: panel.into(),
            rect: super::DockRect::new(
                super::DEFAULT_FLOAT_X,
                super::DEFAULT_FLOAT_Y,
                super::DEFAULT_FLOAT_WIDTH,
                super::DEFAULT_FLOAT_HEIGHT,
            ),
        }
    }

    /// Sets the floating rectangle.
    #[must_use]
    pub fn with_rect(mut self, rect: super::DockRect) -> Self {
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
    pub root: super::DockNode,
    /// Floating panels.
    pub floating: Vec<FloatingPanel>,
    /// Focused panel.
    pub focus: Option<super::PanelId>,
    /// Panel constraints.
    pub constraints: std::collections::BTreeMap<super::PanelId, super::PanelConstraints>,
}

impl DockLayout {
    /// Creates a layout with a single root panel.
    #[must_use]
    pub fn new(root_panel: impl Into<super::PanelId>) -> Self {
        let panel = root_panel.into();
        Self {
            version: super::CURRENT_LAYOUT_VERSION,
            root: super::DockNode::leaf(panel.clone()),
            floating: Vec::new(),
            focus: Some(panel),
            constraints: std::collections::BTreeMap::new(),
        }
    }

    /// Returns all docked and floating panels in deterministic order.
    #[must_use]
    pub fn panel_ids(&self) -> Vec<super::PanelId> {
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
    /// Returns [`super::DockingError::UnknownPanel`] when the panel is not in the layout.
    pub fn focus_panel(mut self, panel: &super::PanelId) -> Result<Self, super::DockingError> {
        if !self.contains_panel(panel) {
            return Err(super::DockingError::UnknownPanel(panel.clone()));
        }
        self.root.activate_tab(panel);
        self.focus = Some(panel.clone());
        Ok(self)
    }

    /// Splits the focused panel and inserts `new_panel` as the second child.
    ///
    /// # Errors
    ///
    /// Returns [`super::DockingError`] when there is no focus, the focused panel is
    /// unknown, the new panel already exists, or the ratio is invalid.
    pub fn split_focused(
        mut self,
        new_panel: impl Into<super::PanelId>,
        axis: super::DockAxis,
        ratio: super::SplitRatio,
    ) -> Result<Self, super::DockingError> {
        let new_panel = new_panel.into();
        self.require_new_panel(&new_panel)?;
        let focus = self.focus.clone().ok_or(super::DockingError::NoFocus)?;
        let replacement = super::DockNode::Split {
            axis,
            ratio,
            first: Box::new(super::DockNode::leaf(focus.clone())),
            second: Box::new(super::DockNode::leaf(new_panel.clone())),
        };
        if !self.root.replace_panel(&focus, replacement) {
            return Err(super::DockingError::UnknownPanel(focus));
        }
        self.focus = Some(new_panel);
        Ok(self)
    }

    /// Tabs `new_panel` with the focused panel.
    ///
    /// # Errors
    ///
    /// Returns [`super::DockingError`] when there is no focus, the focused panel is
    /// unknown, or the new panel already exists.
    pub fn tab_focused(mut self, new_panel: impl Into<super::PanelId>) -> Result<Self, super::DockingError> {
        let new_panel = new_panel.into();
        self.require_new_panel(&new_panel)?;
        let focus = self.focus.clone().ok_or(super::DockingError::NoFocus)?;
        if !self.root.add_tab(&focus, new_panel.clone()) {
            return Err(super::DockingError::UnknownPanel(focus));
        }
        self.focus = Some(new_panel);
        Ok(self)
    }

    /// Adds or replaces a floating panel.
    ///
    /// # Errors
    ///
    /// Returns [`super::DockingError::DuplicatePanel`] when the panel already exists.
    pub fn float_panel(mut self, floating: FloatingPanel) -> Result<Self, super::DockingError> {
        self.require_new_panel(&floating.panel)?;
        self.focus = Some(floating.panel.clone());
        self.floating.push(floating);
        Ok(self)
    }

    /// Resizes the root split.
    ///
    /// # Errors
    ///
    /// Returns [`super::DockingError::NotSplitRoot`] when the root is not a split.
    pub fn resize_root_split(mut self, ratio: super::SplitRatio) -> Result<Self, super::DockingError> {
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
    /// Returns [`super::DockingError`] for unsupported versions, duplicate panel IDs,
    /// missing focused panels, invalid tab groups, or impossible constraints.
    pub fn restore(snapshot: DockLayoutSnapshot) -> Result<Self, super::DockingError> {
        if snapshot.version != super::CURRENT_LAYOUT_VERSION {
            return Err(super::DockingError::UnsupportedVersion(snapshot.version));
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
    /// Returns [`super::DockingError`] for duplicate panels, invalid tabs, missing focus,
    /// or impossible constraints.
    pub fn validate(&self) -> Result<(), super::DockingError> {
        validate_node(&self.root)?;
        let panels = self.panel_ids();
        let unique = panels.iter().collect::<std::collections::BTreeSet<_>>();
        if unique.len() != panels.len() {
            return Err(super::DockingError::DuplicatePanel(super::PanelId::new("duplicate panel id")));
        }
        if let Some(focus) = &self.focus
            && !self.contains_panel(focus)
        {
            return Err(super::DockingError::UnknownPanel(focus.clone()));
        }
        for (panel, constraints) in &self.constraints {
            if !self.contains_panel(panel) {
                return Err(super::DockingError::UnknownPanel(panel.clone()));
            }
            if constraints.min_width == 0 || constraints.min_height == 0 {
                return Err(super::DockingError::ConflictingConstraints(panel.clone()));
            }
        }
        Ok(())
    }

    /// Projects logical panel rectangles for a viewport.
    #[must_use]
    pub fn project_regions(
        &self,
        area: super::DockRect,
    ) -> std::collections::BTreeMap<super::PanelId, super::DockRect> {
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

    fn contains_panel(&self, panel: &super::PanelId) -> bool {
        self.root.contains_panel(panel) || self.floating.iter().any(|floating| &floating.panel == panel)
    }

    fn require_new_panel(&self, panel: &super::PanelId) -> Result<(), super::DockingError> {
        if self.contains_panel(panel) {
            return Err(super::DockingError::DuplicatePanel(panel.clone()));
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
    pub root: super::DockNode,
    /// Floating panels.
    pub floating: Vec<FloatingPanel>,
    /// Focused panel.
    pub focus: Option<super::PanelId>,
    /// Panel constraints.
    pub constraints: std::collections::BTreeMap<super::PanelId, super::PanelConstraints>,
}

fn validate_node(node: &super::DockNode) -> Result<(), super::DockingError> {
    let mut pending = Vec::with_capacity(1);
    pending.push(node);
    debug_assert_eq!(pending.len(), 1);
    debug_assert!(pending.capacity() >= pending.len());
    while let Some(current) = pending.pop() {
        match current {
            super::DockNode::Leaf(panel) => {
                if panel.is_empty() {
                    return Err(super::DockingError::UnknownPanel(panel.clone()));
                }
            }
            super::DockNode::Split { first, second, .. } => {
                pending.push(first);
                pending.push(second);
            }
            super::DockNode::Tabs { panels, active } => {
                if panels.is_empty() || !panels.iter().any(|panel| panel == active) {
                    return Err(super::DockingError::InvalidTabs(active.clone()));
                }
            }
        }
    }
    Ok(())
}
