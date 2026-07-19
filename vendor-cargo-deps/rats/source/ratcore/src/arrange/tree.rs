use super::*;

/// Docking tree node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockNode {
    /// Single panel leaf.
    Leaf(PanelId),
    /// Split with two child nodes.
    Split {
        /// Split axis.
        axis: DockAxis,
        /// First-child ratio.
        ratio: SplitRatio,
        /// First child.
        first: Box<DockNode>,
        /// Second child.
        second: Box<DockNode>,
    },
    /// Tab group.
    Tabs {
        /// Panels in tab order.
        panels: Vec<PanelId>,
        /// Active panel ID.
        active: PanelId,
    },
}

impl DockNode {
    /// Creates a leaf node.
    #[must_use]
    pub fn leaf(panel: impl Into<PanelId>) -> Self {
        Self::Leaf(panel.into())
    }

    pub(super) fn contains_panel(&self, panel: &str) -> bool {
        match self {
            Self::Leaf(id) => id == panel,
            Self::Split { first, second, .. } => {
                first.contains_panel(panel) || second.contains_panel(panel)
            }
            Self::Tabs { panels, .. } => panels.iter().any(|id| id == panel),
        }
    }

    pub(super) fn collect_panels(&self, panels: &mut impl Extend<PanelId>) {
        match self {
            Self::Leaf(id) => panels.extend(std::iter::once(id.clone())),
            Self::Split { first, second, .. } => {
                first.collect_panels(panels);
                second.collect_panels(panels);
            }
            Self::Tabs {
                panels: tab_panels, ..
            } => panels.extend(tab_panels.iter().cloned()),
        }
    }

    pub(super) fn replace_panel(&mut self, target: &str, replacement: DockNode) -> bool {
        match self {
            Self::Leaf(id) if id == target => {
                *self = replacement;
                true
            }
            Self::Leaf(_) | Self::Tabs { .. } => false,
            Self::Split { first, second, .. } => {
                first.replace_panel(target, replacement.clone())
                    || second.replace_panel(target, replacement)
            }
        }
    }

    pub(super) fn add_tab(&mut self, target: &str, panel: PanelId) -> bool {
        match self {
            Self::Leaf(id) if id == target => {
                *self = Self::Tabs {
                    panels: vec![id.clone(), panel.clone()],
                    active: panel,
                };
                true
            }
            Self::Tabs { panels, active } if panels.iter().any(|id| id == target) => {
                panels.push(panel.clone());
                *active = panel;
                true
            }
            Self::Leaf(_) | Self::Tabs { .. } => false,
            Self::Split { first, second, .. } => {
                first.add_tab(target, panel.clone()) || second.add_tab(target, panel)
            }
        }
    }

    pub(super) fn activate_tab(&mut self, panel: &str) -> bool {
        match self {
            Self::Leaf(id) => id == panel,
            Self::Tabs { panels, active } if panels.iter().any(|id| id == panel) => {
                *active = panel.to_string();
                true
            }
            Self::Tabs { .. } => false,
            Self::Split { first, second, .. } => {
                first.activate_tab(panel) || second.activate_tab(panel)
            }
        }
    }

    pub(super) fn resize_root(&mut self, ratio: SplitRatio) -> Result<(), DockingError> {
        match self {
            Self::Split { ratio: current, .. } => {
                *current = ratio;
                Ok(())
            }
            Self::Leaf(_) | Self::Tabs { .. } => Err(DockingError::NotSplitRoot),
        }
    }

    pub(super) fn project_regions(
        &self,
        area: DockRect,
        regions: &mut std::collections::BTreeMap<PanelId, DockRect>,
    ) {
        match self {
            Self::Leaf(panel) => {
                regions.insert(panel.clone(), area);
            }
            Self::Tabs { panels, active } => {
                for panel in panels {
                    if panel == active {
                        regions.insert(panel.clone(), area);
                    }
                }
            }
            Self::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                let (first_area, second_area) = split_area(area, *axis, *ratio);
                first.project_regions(first_area, regions);
                second.project_regions(second_area, regions);
            }
        }
    }
}

fn split_area(area: DockRect, axis: DockAxis, ratio: SplitRatio) -> (DockRect, DockRect) {
    match axis {
        DockAxis::Horizontal => {
            let first_width = ratio.first_len(area.width);
            let second_width = area.width.saturating_sub(first_width);
            (
                DockRect::new(area.x, area.y, first_width, area.height),
                DockRect::new(
                    area.x.saturating_add(first_width),
                    area.y,
                    second_width,
                    area.height,
                ),
            )
        }
        DockAxis::Vertical => {
            let first_height = ratio.first_len(area.height);
            let second_height = area.height.saturating_sub(first_height);
            (
                DockRect::new(area.x, area.y, area.width, first_height),
                DockRect::new(
                    area.x,
                    area.y.saturating_add(first_height),
                    area.width,
                    second_height,
                ),
            )
        }
    }
}
