/// Diagnostic code for pointer route failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerDiagnosticCode {
    /// No routeable target contains the event coordinate.
    NoTarget,
    /// The selected target is hidden.
    HiddenTarget,
    /// The selected target is disabled.
    DisabledTarget,
    /// The renderer supplied an unsupported source, phase, button, unit, or coordinate space.
    UnsupportedKind,
    /// A captured target no longer exists in the hit map.
    StaleTarget,
}

/// Pointer route diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: PointerDiagnosticCode,
    /// Target associated with the diagnostic when available.
    pub target_id: Option<super::shape::PointerTargetId>,
    /// Human-readable message.
    pub message: String,
}

impl PointerDiagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(
        code: PointerDiagnosticCode,
        target_id: Option<super::shape::PointerTargetId>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            target_id,
            message: message.into(),
        }
    }
}

/// Successful pointer interaction class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerInteractionClass {
    /// Activation or primary action.
    Activation,
    /// Context-menu or non-primary action.
    ContextMenu,
    /// Wheel or scroll action.
    WheelScroll,
    /// Hover, enter, leave, or movement preview.
    Hover,
    /// Drag capture began.
    DragStart,
    /// Captured drag updated.
    DragUpdate,
    /// Captured drag committed.
    DragCommit,
    /// Captured drag canceled.
    DragCancel,
    /// Region explicitly passed through to lower hit regions.
    PassThrough,
    /// Modal layer captured the event.
    ModalCapture,
    /// Successful route carries no application mutation intent.
    NoOpDiagnostic,
}

/// Pointer capture lifecycle transition emitted with a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerCaptureTransition {
    /// No capture transition.
    None,
    /// Route starts capture for subsequent events.
    Start,
    /// Route updates an existing capture.
    Update,
    /// Route releases an existing capture.
    Release,
    /// Route cancels an existing capture.
    Cancel,
    /// Modal layer captured the event.
    Modal,
}

/// Successful pointer route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerRoute {
    /// Routed target identifier.
    pub target_id: super::shape::PointerTargetId,
    /// Optional associated action identifier.
    pub action_id: Option<super::shape::PointerActionId>,
    /// Whether the target captured the event as a modal layer.
    pub modal_capture: bool,
    /// Interaction class inferred from normalized event semantics.
    pub interaction: PointerInteractionClass,
    /// Capture lifecycle metadata for drag, release, cancel, and modal paths.
    pub capture: PointerCaptureTransition,
}

/// Route selection outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerRouteOutcome {
    /// Event routed to a target.
    Routed(PointerRoute),
    /// Event failed closed with a diagnostic.
    Diagnostic(PointerDiagnostic),
}

/// Renderer family normalizing events into the pointer router.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerRenderer {
    /// Meteorite or other web renderer.
    Meteorite,
    /// Subwayrat or other terminal renderer.
    Subwayrat,
    /// Ratzilla browser abstraction.
    Ratzilla,
    /// Experimental Iced native adapter.
    Iced,
}

/// Renderer adapter projection for pointer routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerAdapterProjection {
    /// Renderer that supplied the event.
    pub renderer: PointerRenderer,
    /// Shared route selection outcome.
    pub outcome: PointerRouteOutcome,
}
