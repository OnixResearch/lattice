//! Renderer-neutral docking and panel layout core.
//!
//! The core stores panel trees, tabs, splits, floating panels, focus, geometry
//! constraints, versioned snapshots, and renderer-neutral docking intents.
//! Renderers own hit-testing and visuals; they submit intents and consume the
//! resulting logical layout and panel regions.
//!
//! # Example
//!
//! ```
//! use ratcore::docking::{
//!     DockAxis, DockIntent, DockLayout, DockOperation, DockRect, DockRenderer,
//!     SplitRatio, apply_intent,
//! };
//!
//! const SIDEBAR_RATIO_NUMERATOR: u16 = 1;
//! const SIDEBAR_RATIO_DENOMINATOR: u16 = 3;
//! const VIEWPORT_WIDTH: u16 = 120;
//! const VIEWPORT_HEIGHT: u16 = 40;
//!
//! let layout = DockLayout::new("editor");
//! let layout = apply_intent(
//!     layout,
//!     DockIntent::new(
//!         DockRenderer::Tui,
//!         DockOperation::SplitFocused {
//!             panel: ratcore::docking::PanelId::new("sidebar"),
//!             axis: DockAxis::Horizontal,
//!             ratio: SplitRatio::new(SIDEBAR_RATIO_NUMERATOR, SIDEBAR_RATIO_DENOMINATOR).unwrap(),
//!         },
//!     ),
//! ).unwrap();
//! let regions = layout.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT));
//! assert!(regions.contains_key("editor"));
//! assert!(regions.contains_key("sidebar"));
//! ```

const CURRENT_LAYOUT_VERSION: u16 = 1;
const DEFAULT_FLOAT_X: u16 = 0;
const DEFAULT_FLOAT_Y: u16 = 0;
const DEFAULT_FLOAT_WIDTH: u16 = 40;
const DEFAULT_FLOAT_HEIGHT: u16 = 12;
const DEFAULT_MIN_WIDTH: u16 = 1;
const DEFAULT_MIN_HEIGHT: u16 = 1;

mod identity;
mod proportion;
mod state;
mod tree;
pub use identity::DockInsertionIndex;
pub use identity::DockNodeId;
pub use identity::LegacyZoneName;
pub use identity::PanelId;
pub use proportion::SplitRatio;
pub use state::DockLayout;
pub use state::DockLayoutSnapshot;
pub use state::FloatingPanel;
pub use tree::DockNode;

/// Split axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockAxis {
    /// Horizontal split creates left/right children.
    Horizontal,
    /// Vertical split creates top/bottom children.
    Vertical,
}

/// Minimum geometry constraints for a panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelConstraints {
    /// Minimum width.
    pub min_width: u16,
    /// Minimum height.
    pub min_height: u16,
}

impl PanelConstraints {
    /// Creates constraints.
    #[must_use]
    pub fn new(min_width: u16, min_height: u16) -> Self {
        Self { min_width, min_height }
    }
}

impl Default for PanelConstraints {
    fn default() -> Self {
        Self::new(DEFAULT_MIN_WIDTH, DEFAULT_MIN_HEIGHT)
    }
}

/// Logical rectangle for panel region projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockRect {
    /// Left coordinate.
    pub x: u16,
    /// Top coordinate.
    pub y: u16,
    /// Width.
    pub width: u16,
    /// Height.
    pub height: u16,
}

impl DockRect {
    /// Creates a rectangle.
    #[must_use]
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }
}

/// Renderer producing a docking intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockRenderer {
    /// Meteorite or other web renderer.
    Web,
    /// Subwayrat or other terminal renderer.
    Tui,
}

/// Renderer-neutral docking operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockOperation {
    /// Focus an existing panel.
    Focus(PanelId),
    /// Split the focused panel.
    SplitFocused {
        /// New panel ID.
        panel: PanelId,
        /// Split axis.
        axis: DockAxis,
        /// Split ratio.
        ratio: SplitRatio,
    },
    /// Add a tab to the focused panel.
    TabFocused {
        /// New panel ID.
        panel: PanelId,
    },
    /// Float a new panel.
    Float(FloatingPanel),
    /// Resize the root split.
    ResizeRoot(SplitRatio),
}

/// Renderer-neutral docking intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockIntent {
    /// Renderer that produced the intent.
    pub renderer: DockRenderer,
    /// Operation to apply.
    pub operation: DockOperation,
}

impl DockIntent {
    /// Creates an intent.
    #[must_use]
    pub fn new(renderer: DockRenderer, operation: DockOperation) -> Self {
        Self { renderer, operation }
    }
}

/// Applies a renderer-neutral intent to a layout.
///
/// # Errors
///
/// Returns [`DockingError`] when the operation references unknown panels,
/// duplicates panel IDs, has invalid geometry, or cannot apply to the layout.
pub fn apply_intent(layout: DockLayout, intent: DockIntent) -> Result<DockLayout, DockingError> {
    match intent.operation {
        DockOperation::Focus(panel) => layout.focus_panel(&panel),
        DockOperation::SplitFocused { panel, axis, ratio } => layout.split_focused(panel, axis, ratio),
        DockOperation::TabFocused { panel } => layout.tab_focused(panel),
        DockOperation::Float(floating) => layout.float_panel(floating),
        DockOperation::ResizeRoot(ratio) => layout.resize_root_split(ratio),
    }
}

/// Docking validation and operation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockingError {
    /// Split ratio is invalid.
    InvalidRatio,
    /// Panel does not exist.
    UnknownPanel(PanelId),
    /// Panel already exists.
    DuplicatePanel(PanelId),
    /// No panel is focused.
    NoFocus,
    /// Root node is not a split.
    NotSplitRoot,
    /// Snapshot version is unsupported.
    UnsupportedVersion(u16),
    /// Tab group is malformed.
    InvalidTabs(PanelId),
    /// Panel constraints conflict with valid layout geometry.
    ConflictingConstraints(PanelId),
    /// Public or persisted insertion position cannot fit the host index type.
    InvalidInsertionIndex(u128),
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
