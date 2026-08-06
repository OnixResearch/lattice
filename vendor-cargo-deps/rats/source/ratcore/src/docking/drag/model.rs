/// Drag session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragSessionStatus {
    /// Session is active.
    Active,
    /// Session has a preview.
    Previewing,
    /// Session committed.
    Committed,
    /// Session canceled.
    Canceled,
}

/// Pure drag session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragSession {
    /// Target identifier.
    pub target_id: String,
    /// Target kind.
    pub kind: DragAffordanceKind,
    /// Start coordinate.
    pub start: DragPoint,
    /// Last observed coordinate.
    pub current: DragPoint,
    /// Current status.
    pub status: DragSessionStatus,
}

/// Temporary compatibility name for the canonical non-collapsing split ratio.
///
/// New code uses [`crate::docking::SplitRatio`]; this alias is scheduled for
/// removal at the next semver-breaking Ratcore release.
pub type ResizeRatio = crate::docking::SplitRatio;

/// Commit operation emitted by an accepted drag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragDockOperation {
    /// Resize a splitter with a ratio.
    ResizeSplitter(ResizeRatio),
    /// Reorder a tab before or after another target.
    ReorderTab {
        /// Moved tab identifier.
        tab_id: String,
        /// Destination tab or group identifier.
        destination_id: String,
    },
    /// Dock a panel into a zone.
    DockPanel {
        /// Canonical panel identifier.
        panel_id: crate::docking::PanelId,
        /// Canonical docking-node identifier.
        zone_id: crate::docking::DockNodeId,
    },
    /// Move a floating panel to a new origin.
    MoveFloatingPanel {
        /// Canonical panel identifier.
        panel_id: crate::docking::PanelId,
        /// New origin.
        origin: DragPoint,
    },
    /// Resize a frame to a new rectangle.
    ResizeFrame {
        /// Canonical panel identifier.
        panel_id: crate::docking::PanelId,
        /// New frame rectangle.
        frame: DragRect,
    },
}

/// Renderer-neutral docking intent description emitted by drag commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragDockIntent {
    /// Renderer that originated the drag.
    pub renderer: DragRenderer,
    /// Docking operation to apply in the application reducer.
    pub operation: DragDockOperation,
}

/// Renderer family originating a drag session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragRenderer {
    /// Meteorite/Dioxus renderer.
    Meteorite,
    /// Subwayrat/ratatui renderer.
    Subwayrat,
}

/// Drag event supplied to the reducer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragEvent {
    /// Event phase.
    pub phase: DragPhase,
    /// Target identifier associated with the event.
    pub target_id: String,
    /// Event coordinate.
    pub position: DragPoint,
}

impl DragEvent {
    /// Creates a drag event.
    #[must_use]
    pub fn new(phase: DragPhase, target_id: impl Into<String>, position: DragPoint) -> Self {
        Self {
            phase,
            target_id: target_id.into(),
            position,
        }
    }
}

/// Declarative drag preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragPreview {
    /// Target associated with the session.
    pub target_id: String,
    /// Ordered candidates after deterministic ranking.
    pub candidates: Vec<DropZoneCandidate>,
    /// Selected candidate when available.
    pub selected_zone_id: Option<crate::docking::DockNodeId>,
    /// Ghost rectangle to render.
    pub ghost: DragRect,
    /// Whether the selected target is allowed.
    pub allowed: bool,
    /// Preview diagnostic when no allowed target exists.
    pub diagnostic: Option<DragDiagnostic>,
}

/// Drag diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragDiagnosticCode {
    /// Update, preview, or commit arrived without a valid start.
    MissingStart,
    /// Target is absent from explicit target metadata.
    StaleTarget,
    /// Renderer supplied an unsupported phase.
    UnsupportedPhase,
    /// Target is disabled.
    DisabledAffordance,
    /// Commit request is malformed.
    InvalidCommit,
    /// Resize constraints reject the requested size.
    ImpossibleResize,
    /// Drag was canceled.
    Canceled,
}

/// Drag diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: DragDiagnosticCode,
    /// Target associated with the diagnostic.
    pub target_id: Option<String>,
    /// Human-readable message.
    pub message: String,
}

impl DragDiagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(
        code: DragDiagnosticCode,
        target_id: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            target_id,
            message: message.into(),
        }
    }
}

/// Reducer outcome for one drag event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragTransition {
    /// Next session state when the event keeps one alive.
    pub session: Option<DragSession>,
    /// Diagnostic when the event failed closed.
    pub diagnostic: Option<DragDiagnostic>,
}

/// Commit request for an active drag session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragCommitRequest {
    /// Renderer that originated the session.
    pub renderer: DragRenderer,
    /// Session to commit.
    pub session: DragSession,
    /// Operation proposed by preview or application policy.
    pub operation: DragDockOperation,
}

/// Commit outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DragCommitOutcome {
    /// Accepted docking intent.
    pub intent: Option<DragDockIntent>,
    /// Diagnostic when the commit was rejected.
    pub diagnostic: Option<DragDiagnostic>,
}
