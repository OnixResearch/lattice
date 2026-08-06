#[cfg(test)]
const KEY_ENTER: &str = "Enter";
const ACTION_ACTIVATE: &str = "activate";
const ACTION_CANCEL: &str = "cancel";
const ACTION_POINTER_ROUTE: &str = "pointer.route";
const GRID_ORIGIN: i32 = 0;

#[path = "events/normalization.rs"]
mod normalization;
pub use normalization::RatzillaKeyInput;
pub use normalization::adapt_ratzilla_key_input;
pub use normalization::normalize_ratzilla_key_input;

/// Ratzilla mouse input normalized at the shell boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatzillaMouseInput {
    /// Grid x coordinate.
    pub x: i32,
    /// Grid y coordinate.
    pub y: i32,
    /// Mouse button number supplied by the shell.
    pub button: i32,
    /// Ratzilla mouse event kind supplied by the shell.
    pub kind: crate::pointer_input_routing::RatzillaMouseKind,
    /// Active pointer modifiers.
    pub modifiers: crate::pointer_input_routing::PointerModifiers,
}

impl RatzillaMouseInput {
    /// Creates a mouse input.
    #[must_use]
    pub const fn new(x: i32, y: i32, button: i32) -> Self {
        Self {
            x,
            y,
            button,
            kind: crate::pointer_input_routing::RatzillaMouseKind::ButtonDown,
            modifiers: crate::pointer_input_routing::PointerModifiers {
                ctrl: false,
                alt: false,
                shift: false,
                meta: false,
            },
        }
    }

    /// Sets the Ratzilla event kind.
    #[must_use]
    pub fn with_kind(mut self, kind: crate::pointer_input_routing::RatzillaMouseKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets modifier metadata.
    #[must_use]
    pub const fn with_modifiers(mut self, modifiers: crate::pointer_input_routing::PointerModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }
}

/// Web grid bounds for input normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebGridBounds {
    /// Width in terminal cells.
    pub width: i32,
    /// Height in terminal cells.
    pub height: i32,
}

impl WebGridBounds {
    /// Creates grid bounds.
    #[must_use]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

/// Web input diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebInputDiagnosticCode {
    /// Key has no action mapping.
    UnsupportedKey,
    /// Mouse button has no action mapping.
    UnknownMouseButton,
    /// Grid coordinate is outside the terminal surface.
    OutOfBoundsCoordinate,
    /// Modifier combination is not supported by the web shell.
    UnsupportedModifierCombination,
    /// Uncommitted composition cannot become a shortcut.
    CompositionExcluded,
    /// Repeated key input is excluded by policy.
    RepeatExcluded,
}

/// Web input diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebInputDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: WebInputDiagnosticCode,
    /// Human-readable message.
    pub message: String,
}

impl WebInputDiagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(code: WebInputDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Platform-neutral input adapter outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebInputOutcome {
    /// Shared action identifier when mapped.
    pub action_id: Option<String>,
    /// Diagnostic when unsupported.
    pub diagnostic: Option<WebInputDiagnostic>,
    /// Pointer modifier metadata when the outcome came from a mouse event.
    pub pointer_modifiers: Option<crate::pointer_input_routing::PointerModifiers>,
}

fn action_outcome(action_id: &str) -> WebInputOutcome {
    WebInputOutcome {
        action_id: Some(action_id.to_string()),
        diagnostic: None,
        pointer_modifiers: None,
    }
}

fn diagnostic_outcome(
    code: WebInputDiagnosticCode,
    message: impl Into<String>,
    pointer_modifiers: Option<crate::pointer_input_routing::PointerModifiers>,
) -> WebInputOutcome {
    WebInputOutcome {
        action_id: None,
        diagnostic: Some(WebInputDiagnostic::new(code, message)),
        pointer_modifiers,
    }
}

/// Adapts a Ratzilla mouse input into a platform-neutral pointer action.
#[must_use]
pub fn adapt_ratzilla_mouse_input(input: RatzillaMouseInput, bounds: WebGridBounds) -> WebInputOutcome {
    debug_assert!(bounds.width >= GRID_ORIGIN);
    debug_assert!(bounds.height >= GRID_ORIGIN);
    if input.x < 0 || input.y < 0 || input.x >= bounds.width || input.y >= bounds.height {
        return diagnostic_outcome(
            WebInputDiagnosticCode::OutOfBoundsCoordinate,
            "mouse coordinate is outside the web terminal grid",
            Some(input.modifiers),
        );
    }
    let normalized = crate::pointer_input_routing::normalize_ratzilla_mouse_input(
        crate::pointer_input_routing::RatzillaPointerInput::new(
            crate::pointer_input_routing::PointerPoint::new(input.x, input.y),
            input.kind,
            input.button,
        )
        .with_modifiers(input.modifiers),
    );
    let Ok(event) = normalized else {
        return diagnostic_outcome(
            WebInputDiagnosticCode::UnknownMouseButton,
            "mouse event is not mapped by the web shell",
            Some(input.modifiers),
        );
    };
    shared_pointer_outcome(&event, bounds)
}

fn shared_pointer_outcome(
    event: &crate::pointer_input_routing::PointerEvent,
    bounds: WebGridBounds,
) -> WebInputOutcome {
    debug_assert!(bounds.width >= GRID_ORIGIN);
    debug_assert!(bounds.height >= GRID_ORIGIN);
    let hit_map = crate::pointer_input_routing::derive_pointer_hit_map(vec![
        crate::pointer_input_routing::RenderedPointerRegion::new(
            ACTION_POINTER_ROUTE,
            crate::pointer_input_routing::PointerRect::new(GRID_ORIGIN, GRID_ORIGIN, bounds.width, bounds.height),
            GRID_ORIGIN,
        )
        .with_action(ACTION_POINTER_ROUTE),
    ]);
    match crate::pointer_input_routing::route_pointer_event(event, &hit_map) {
        crate::pointer_input_routing::PointerRouteOutcome::Routed(route) => WebInputOutcome {
            action_id: route.action_id.map(|action_id| action_id.to_string()),
            diagnostic: None,
            pointer_modifiers: Some(event.modifiers),
        },
        crate::pointer_input_routing::PointerRouteOutcome::Diagnostic(diagnostic) => {
            diagnostic_outcome(WebInputDiagnosticCode::UnknownMouseButton, diagnostic.message, Some(event.modifiers))
        }
    }
}

#[cfg(test)]
#[path = "events/tests.rs"]
mod tests;
