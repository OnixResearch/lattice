//! Pure drag and resize affordance model for docking shells.
//!
//! This module turns normalized drag phases, explicit targets, and drop zones
//! into deterministic session state, previews, and commit intents. Renderers own
//! pointer capture and drawing; application reducers own applying accepted
//! docking intents to real layout state.

use std::cmp::Reverse;

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
    /// Stable zone identifier.
    pub zone_id: String,
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
    pub fn new(zone_id: impl Into<String>, bounds: DragRect, priority: i32) -> Self {
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

/// Ratio used by resize commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeRatio {
    /// Numerator.
    pub numerator: u16,
    /// Denominator.
    pub denominator: u16,
}

impl ResizeRatio {
    /// Creates a resize ratio.
    ///
    /// # Errors
    ///
    /// Returns [`DragDiagnostic`] when the denominator is zero or the ratio
    /// would collapse either side of the resize.
    pub fn new(numerator: u16, denominator: u16) -> Result<Self, DragDiagnostic> {
        if denominator == 0 || numerator == 0 || numerator >= denominator {
            return Err(DragDiagnostic::new(
                DragDiagnosticCode::ImpossibleResize,
                None,
                "resize ratio would collapse a pane",
            ));
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }
}

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
        /// Panel identifier.
        panel_id: String,
        /// Zone identifier.
        zone_id: String,
    },
    /// Move a floating panel to a new origin.
    MoveFloatingPanel {
        /// Panel identifier.
        panel_id: String,
        /// New origin.
        origin: DragPoint,
    },
    /// Resize a frame to a new rectangle.
    ResizeFrame {
        /// Panel identifier.
        panel_id: String,
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
    pub selected_zone_id: Option<String>,
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

/// Reduces a drag event into session state without mutating layout.
#[must_use]
pub fn reduce_drag_resize_session(
    previous: Option<&DragSession>,
    event: &DragEvent,
    targets: &[DragTarget],
) -> DragTransition {
    if let DragPhase::Unsupported(phase) = &event.phase {
        return rejected(
            previous.cloned(),
            DragDiagnostic::new(
                DragDiagnosticCode::UnsupportedPhase,
                Some(event.target_id.clone()),
                format!("unsupported drag phase: {phase}"),
            ),
        );
    }
    match event.phase {
        DragPhase::Start => start_session(event, targets),
        DragPhase::Update | DragPhase::Preview => update_session(previous, event),
        DragPhase::Cancel => cancel_session(previous, event),
        DragPhase::Commit => commit_session_transition(previous, event),
        DragPhase::Unsupported(_) => unreachable!("unsupported drag phase handled before match"),
    }
}

/// Computes preview metadata from a session and explicit drop zones.
#[must_use]
pub fn compute_drag_resize_preview(
    session: &DragSession,
    zones: &[DropZoneCandidate],
) -> DragPreview {
    let mut candidates = zones.to_vec();
    candidates.sort_by_key(|candidate| (Reverse(candidate.priority), candidate.zone_id.clone()));
    let selected = candidates.iter().find(|candidate| candidate.allowed);
    let diagnostic = selected.is_none().then(|| {
        DragDiagnostic::new(
            DragDiagnosticCode::InvalidCommit,
            Some(session.target_id.clone()),
            "no allowed drop zone for drag preview",
        )
    });
    DragPreview {
        target_id: session.target_id.clone(),
        selected_zone_id: selected.map(|candidate| candidate.zone_id.clone()),
        ghost: selected.map_or(
            DragRect::new(session.current.x, session.current.y, 0, 0),
            |candidate| candidate.bounds,
        ),
        allowed: selected.is_some(),
        candidates,
        diagnostic,
    }
}

/// Validates and converts an accepted session into a docking intent description.
#[must_use]
pub fn commit_drag_resize(request: DragCommitRequest) -> DragCommitOutcome {
    if request.session.status == DragSessionStatus::Canceled {
        return DragCommitOutcome {
            intent: None,
            diagnostic: Some(DragDiagnostic::new(
                DragDiagnosticCode::Canceled,
                Some(request.session.target_id),
                "cannot commit a canceled drag session",
            )),
        };
    }
    if request.session.status != DragSessionStatus::Committed {
        return DragCommitOutcome {
            intent: None,
            diagnostic: Some(DragDiagnostic::new(
                DragDiagnosticCode::InvalidCommit,
                Some(request.session.target_id),
                "drag session was not committed by the reducer",
            )),
        };
    }
    let diagnostic = validate_commit_operation(&request.operation);
    if let Some(diagnostic) = diagnostic {
        return DragCommitOutcome {
            intent: None,
            diagnostic: Some(diagnostic),
        };
    }
    DragCommitOutcome {
        intent: Some(DragDockIntent {
            renderer: request.renderer,
            operation: request.operation,
        }),
        diagnostic: None,
    }
}

/// Creates a progress ratio from percent-like input for callers that expose sliders.
///
/// # Errors
///
/// Returns [`DragDiagnostic`] when the value is outside the accepted non-edge range.
pub fn resize_ratio_from_percent(percent: u16) -> Result<ResizeRatio, DragDiagnostic> {
    ResizeRatio::new(percent, DEFAULT_PROGRESS_DENOMINATOR)
}

fn start_session(event: &DragEvent, targets: &[DragTarget]) -> DragTransition {
    let Some(target) = targets
        .iter()
        .find(|target| target.target_id == event.target_id)
    else {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::StaleTarget,
                Some(event.target_id.clone()),
                "drag target is absent",
            ),
        );
    };
    if !target.enabled {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::DisabledAffordance,
                Some(target.target_id.clone()),
                target
                    .diagnostic
                    .clone()
                    .unwrap_or_else(|| "drag target is disabled".to_string()),
            ),
        );
    }
    DragTransition {
        session: Some(DragSession {
            target_id: target.target_id.clone(),
            kind: target.kind,
            start: event.position,
            current: event.position,
            status: DragSessionStatus::Active,
        }),
        diagnostic: None,
    }
}

fn update_session(previous: Option<&DragSession>, event: &DragEvent) -> DragTransition {
    let Some(session) = previous else {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::MissingStart,
                Some(event.target_id.clone()),
                "drag update arrived without a start",
            ),
        );
    };
    if session.target_id != event.target_id {
        return rejected(
            Some(session.clone()),
            DragDiagnostic::new(
                DragDiagnosticCode::StaleTarget,
                Some(event.target_id.clone()),
                "drag update target does not match session target",
            ),
        );
    }
    let status = if event.phase == DragPhase::Preview {
        DragSessionStatus::Previewing
    } else {
        DragSessionStatus::Active
    };
    DragTransition {
        session: Some(DragSession {
            current: event.position,
            status,
            ..session.clone()
        }),
        diagnostic: None,
    }
}

fn cancel_session(previous: Option<&DragSession>, event: &DragEvent) -> DragTransition {
    let Some(session) = previous else {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::MissingStart,
                Some(event.target_id.clone()),
                "drag cancel arrived without a start",
            ),
        );
    };
    DragTransition {
        session: Some(DragSession {
            status: DragSessionStatus::Canceled,
            ..session.clone()
        }),
        diagnostic: Some(DragDiagnostic::new(
            DragDiagnosticCode::Canceled,
            Some(session.target_id.clone()),
            "drag session canceled",
        )),
    }
}

fn commit_session_transition(previous: Option<&DragSession>, event: &DragEvent) -> DragTransition {
    let Some(session) = previous else {
        return rejected(
            None,
            DragDiagnostic::new(
                DragDiagnosticCode::MissingStart,
                Some(event.target_id.clone()),
                "drag commit arrived without a start",
            ),
        );
    };
    if session.target_id != event.target_id {
        return rejected(
            Some(session.clone()),
            DragDiagnostic::new(
                DragDiagnosticCode::StaleTarget,
                Some(event.target_id.clone()),
                "drag commit target does not match session target",
            ),
        );
    }
    DragTransition {
        session: Some(DragSession {
            current: event.position,
            status: DragSessionStatus::Committed,
            ..session.clone()
        }),
        diagnostic: None,
    }
}

fn validate_commit_operation(operation: &DragDockOperation) -> Option<DragDiagnostic> {
    match operation {
        DragDockOperation::ResizeSplitter(ratio) => {
            ResizeRatio::new(ratio.numerator, ratio.denominator).err()
        }
        DragDockOperation::ReorderTab {
            tab_id,
            destination_id,
        } => validate_non_empty_pair(tab_id, destination_id),
        DragDockOperation::DockPanel { panel_id, zone_id } => {
            validate_non_empty_pair(panel_id, zone_id)
        }
        DragDockOperation::MoveFloatingPanel { panel_id, .. }
        | DragDockOperation::ResizeFrame { panel_id, .. } => validate_non_empty(panel_id),
    }
}

fn validate_non_empty_pair(left: &str, right: &str) -> Option<DragDiagnostic> {
    validate_non_empty(left).or_else(|| validate_non_empty(right))
}

fn validate_non_empty(value: &str) -> Option<DragDiagnostic> {
    value.trim().is_empty().then(|| {
        DragDiagnostic::new(
            DragDiagnosticCode::InvalidCommit,
            None,
            "commit operation contains an empty identifier",
        )
    })
}

fn rejected(session: Option<DragSession>, diagnostic: DragDiagnostic) -> DragTransition {
    DragTransition {
        session,
        diagnostic: Some(diagnostic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const START_X: i32 = 4;
    const START_Y: i32 = 6;
    const UPDATE_X: i32 = 10;
    const UPDATE_Y: i32 = 12;
    const RECT_X: i32 = 0;
    const RECT_Y: i32 = 0;
    const RECT_WIDTH: i32 = 20;
    const RECT_HEIGHT: i32 = 10;
    const HIGH_PRIORITY: i32 = 10;
    const LOW_PRIORITY: i32 = 1;
    const VALID_RESIZE_PERCENT: u16 = 40;

    fn point() -> DragPoint {
        DragPoint::new(START_X, START_Y)
    }

    fn updated_point() -> DragPoint {
        DragPoint::new(UPDATE_X, UPDATE_Y)
    }

    fn rect() -> DragRect {
        DragRect::new(RECT_X, RECT_Y, RECT_WIDTH, RECT_HEIGHT)
    }

    fn target() -> DragTarget {
        DragTarget::new("splitter", DragAffordanceKind::Splitter, rect())
    }

    #[test]
    fn drag_session_start_update_preview_and_commit_are_pure() {
        let start = DragEvent::new(DragPhase::Start, "splitter", point());
        let transition = reduce_drag_resize_session(None, &start, &[target()]);
        let Some(session) = transition.session else {
            panic!("start should create a session");
        };
        let update = DragEvent::new(DragPhase::Update, "splitter", updated_point());
        let updated = reduce_drag_resize_session(Some(&session), &update, &[target()]);
        let Some(updated_session) = updated.session else {
            panic!("update should keep a session");
        };
        let commit = DragEvent::new(DragPhase::Commit, "splitter", updated_point());
        let committed = reduce_drag_resize_session(Some(&updated_session), &commit, &[target()]);

        assert!(transition.diagnostic.is_none());
        assert_eq!(updated_session.current, updated_point());
        assert_eq!(
            committed.session.map(|session| session.status),
            Some(DragSessionStatus::Committed)
        );
    }

    #[test]
    fn stale_disabled_missing_and_unsupported_events_fail_closed() {
        let disabled_target = target().with_enabled(false);
        let start_disabled = DragEvent::new(DragPhase::Start, "splitter", point());
        let missing_update = DragEvent::new(DragPhase::Update, "splitter", point());
        let unsupported = DragEvent::new(
            DragPhase::Unsupported("hover-drag".to_string()),
            "splitter",
            point(),
        );
        let stale = DragEvent::new(DragPhase::Start, "missing", point());

        let disabled = reduce_drag_resize_session(None, &start_disabled, &[disabled_target]);
        let missing = reduce_drag_resize_session(None, &missing_update, &[target()]);
        let unsupported = reduce_drag_resize_session(None, &unsupported, &[target()]);
        let stale = reduce_drag_resize_session(None, &stale, &[target()]);

        assert!(matches!(
            disabled.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::DisabledAffordance,
                ..
            })
        ));
        assert!(matches!(
            missing.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::MissingStart,
                ..
            })
        ));
        assert!(matches!(
            unsupported.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::UnsupportedPhase,
                ..
            })
        ));
        assert!(matches!(
            stale.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::StaleTarget,
                ..
            })
        ));
    }

    #[test]
    fn preview_ranks_stable_candidates_and_rejects_empty_set() {
        let session = DragSession {
            target_id: "panel".to_string(),
            kind: DragAffordanceKind::Panel,
            start: point(),
            current: updated_point(),
            status: DragSessionStatus::Previewing,
        };
        let preview = compute_drag_resize_preview(
            &session,
            &[
                DropZoneCandidate::new("right", rect(), LOW_PRIORITY),
                DropZoneCandidate::new("left", rect(), HIGH_PRIORITY),
            ],
        );
        let rejected = compute_drag_resize_preview(
            &session,
            &[DropZoneCandidate::new("blocked", rect(), HIGH_PRIORITY).with_allowed(false)],
        );

        assert_eq!(preview.selected_zone_id, Some("left".to_string()));
        assert!(preview.allowed);
        assert!(!rejected.allowed);
        assert!(matches!(
            rejected.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::InvalidCommit,
                ..
            })
        ));
    }

    #[test]
    fn commit_emits_intent_or_rejects_invalid_and_canceled_sessions() {
        let ratio = resize_ratio_from_percent(VALID_RESIZE_PERCENT)
            .expect("named test percent should be a valid resize ratio");
        let committed_session = DragSession {
            target_id: "splitter".to_string(),
            kind: DragAffordanceKind::Splitter,
            start: point(),
            current: updated_point(),
            status: DragSessionStatus::Committed,
        };
        let accepted = commit_drag_resize(DragCommitRequest {
            renderer: DragRenderer::Subwayrat,
            session: committed_session.clone(),
            operation: DragDockOperation::ResizeSplitter(ratio),
        });
        let canceled = commit_drag_resize(DragCommitRequest {
            renderer: DragRenderer::Meteorite,
            session: DragSession {
                status: DragSessionStatus::Canceled,
                ..committed_session
            },
            operation: DragDockOperation::DockPanel {
                panel_id: "panel".to_string(),
                zone_id: "left".to_string(),
            },
        });

        assert!(accepted.intent.is_some());
        assert!(accepted.diagnostic.is_none());
        assert!(matches!(
            canceled.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::Canceled,
                ..
            })
        ));
    }

    #[test]
    fn impossible_resize_commit_is_rejected_without_intent() {
        let committed_session = DragSession {
            target_id: "splitter".to_string(),
            kind: DragAffordanceKind::Splitter,
            start: point(),
            current: updated_point(),
            status: DragSessionStatus::Committed,
        };
        let impossible_ratio = ResizeRatio {
            numerator: DEFAULT_PROGRESS_DENOMINATOR,
            denominator: DEFAULT_PROGRESS_DENOMINATOR,
        };

        let rejected = commit_drag_resize(DragCommitRequest {
            renderer: DragRenderer::Subwayrat,
            session: committed_session,
            operation: DragDockOperation::ResizeSplitter(impossible_ratio),
        });

        assert!(rejected.intent.is_none());
        assert!(matches!(
            rejected.diagnostic,
            Some(DragDiagnostic {
                code: DragDiagnosticCode::ImpossibleResize,
                ..
            })
        ));
    }
}
