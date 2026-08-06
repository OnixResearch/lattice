const DEFAULT_PROGRESS_DENOMINATOR: u16 = 100;

/// Drag coordinate in renderer-neutral viewport space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragPoint {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

impl DragPoint {
    /// Creates a drag point.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Rectangle used for previews and drop-zone metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragRect {
    /// Left coordinate.
    pub x: i32,
    /// Top coordinate.
    pub y: i32,
    /// Width.
    pub width: i32,
    /// Height.
    pub height: i32,
}

impl DragRect {
    /// Creates a rectangle.
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true when the rectangle has no positive area.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

/// Kind of drag or resize affordance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragAffordanceKind {
    /// Splitter resize handle.
    Splitter,
    /// Docked panel drag handle.
    Panel,
    /// Tab strip reorder target.
    Tab,
    /// Dock target drop zone.
    DockTarget,
    /// Floating panel movement.
    FloatingPanel,
    /// Window frame resize handle.
    FrameHandle,
}

/// Drag event phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragPhase {
    /// Start a session.
    Start,
    /// Update an existing session.
    Update,
    /// Request preview computation.
    Preview,
    /// Commit an existing session.
    Commit,
    /// Cancel an existing session.
    Cancel,
    /// Renderer reported an unsupported phase.
    Unsupported(String),
}

/// Explicit target metadata for a drag affordance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragTarget {
    /// Stable target identifier.
    pub target_id: String,
    /// Affordance kind.
    pub kind: DragAffordanceKind,
    /// Current target bounds.
    pub bounds: DragRect,
    /// Whether this affordance can be used.
    pub enabled: bool,
    /// Optional diagnostic metadata.
    pub diagnostic: Option<String>,
}

impl DragTarget {
    /// Creates an enabled drag target.
    #[must_use]
    pub fn new(target_id: impl Into<String>, kind: DragAffordanceKind, bounds: DragRect) -> Self {
        Self {
            target_id: target_id.into(),
            kind,
            bounds,
            enabled: true,
            diagnostic: None,
        }
    }

    /// Sets target enablement.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets diagnostic metadata.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}

/// Drop-zone candidate for preview computation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropZoneCandidate {
    /// Stable docking-node identifier.
    pub zone_id: crate::docking::DockNodeId,
    /// Zone bounds.
    pub bounds: DragRect,
    /// Higher priority zones are selected first.
    pub priority: i32,
    /// Whether the candidate accepts the current session.
    pub allowed: bool,
    /// Rejection reason when unavailable.
    pub diagnostic: Option<String>,
}

impl DropZoneCandidate {
    /// Creates an allowed drop-zone candidate.
    #[must_use]
    pub fn new(
        zone_id: impl Into<crate::docking::DockNodeId>,
        bounds: DragRect,
        priority: i32,
    ) -> Self {
        Self {
            zone_id: zone_id.into(),
            bounds,
            priority,
            allowed: true,
            diagnostic: None,
        }
    }

    /// Sets availability.
    #[must_use]
    pub const fn with_allowed(mut self, allowed: bool) -> Self {
        self.allowed = allowed;
        self
    }

    /// Sets diagnostic metadata.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}
