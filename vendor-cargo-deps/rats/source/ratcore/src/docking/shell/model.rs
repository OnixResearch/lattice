const DEFAULT_BORDER_THICKNESS: u16 = 1;
const DEFAULT_HEADER_HEIGHT: u16 = 1;
const DEFAULT_CONTROL_WIDTH: u16 = 3;
const DEFAULT_CONTROL_GAP: u16 = 1;
const DEFAULT_RESIZE_AFFORDANCE: u16 = 1;
const UNTITLED_PANEL: &str = "(untitled)";
const ZERO_DIMENSION: u16 = 0;

/// Temporary compatibility name for the canonical docking panel identifier.
///
/// New code uses [`crate::docking::PanelId`]; this alias is scheduled for
/// removal at the next semver-breaking Ratcore release.
pub type ShellPanelId = crate::docking::PanelId;

/// Canonical registry action identifier used by shell controls.
pub type ShellActionId = crate::action_registry::ActionId;

/// Rectangle in renderer-neutral shell coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellRect {
    /// Left coordinate.
    pub x: u16,
    /// Top coordinate.
    pub y: u16,
    /// Width.
    pub width: u16,
    /// Height.
    pub height: u16,
}

impl ShellRect {
    /// Creates a rectangle.
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true when either dimension is zero.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.width == ZERO_DIMENSION || self.height == ZERO_DIMENSION
    }
}

/// Declarative window control kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellControlKind {
    /// Close control.
    Close,
    /// Minimize control.
    Minimize,
    /// Maximize or restore control.
    Maximize,
    /// Pin control.
    Pin,
    /// Float control.
    Float,
    /// Dock control.
    Dock,
    /// Renderer/application specific control.
    Custom(String),
}

/// Declarative shell control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellControl {
    /// Stable control identifier.
    pub id: String,
    /// Control kind.
    pub kind: ShellControlKind,
    /// Action or typed intent target.
    pub action_id: Option<ShellActionId>,
    /// Whether the control can be activated.
    pub enabled: bool,
    /// Optional diagnostic metadata.
    pub diagnostic: Option<String>,
}

impl ShellControl {
    /// Creates an enabled control with an action target.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        kind: ShellControlKind,
        action_id: impl Into<ShellActionId>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            action_id: Some(action_id.into()),
            enabled: true,
            diagnostic: None,
        }
    }

    /// Creates a control without an action target for validation tests or diagnostics.
    #[must_use]
    pub fn without_action(id: impl Into<String>, kind: ShellControlKind) -> Self {
        Self {
            id: id.into(),
            kind,
            action_id: None,
            enabled: true,
            diagnostic: None,
        }
    }

    /// Sets enablement.
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

/// Renderer-neutral shell record for a panel or floating window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelShell {
    /// Stable panel identifier.
    pub panel_id: ShellPanelId,
    /// Display title.
    pub title: String,
    /// Whether this panel is active.
    pub active: bool,
    /// Z-order supplied by the owning application.
    pub z_order: i32,
    /// Whether the shell should be visible.
    pub visible: bool,
    /// Window controls.
    pub controls: Vec<ShellControl>,
    /// Frame bounds supplied by layout or caller measurement.
    pub frame: ShellRect,
    /// Caller-owned content slot identifier.
    pub client_slot_id: String,
}

impl PanelShell {
    /// Creates a visible inactive panel shell.
    #[must_use]
    pub fn new(
        panel_id: impl Into<ShellPanelId>,
        title: impl Into<String>,
        frame: ShellRect,
    ) -> Self {
        let panel_id = panel_id.into();
        Self {
            client_slot_id: panel_id.as_str().to_string(),
            panel_id,
            title: title.into(),
            active: false,
            z_order: 0,
            visible: true,
            controls: Vec::new(),
            frame,
        }
    }

    /// Sets active state.
    #[must_use]
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets z-order.
    #[must_use]
    pub const fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Sets visibility.
    #[must_use]
    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets controls.
    #[must_use]
    pub fn with_controls(mut self, controls: Vec<ShellControl>) -> Self {
        self.controls = controls;
        self
    }

    /// Sets caller-owned client slot identifier.
    #[must_use]
    pub fn with_client_slot(mut self, client_slot_id: impl Into<String>) -> Self {
        self.client_slot_id = client_slot_id.into();
        self
    }
}
