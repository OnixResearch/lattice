/// Hit-region capture behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitCapturePolicy {
    /// Region captures matching events.
    Capture,
    /// Region is transparent and allows lower regions to be considered.
    PassThrough,
    /// Region captures events as a modal layer even when no child is hit.
    ModalCapture,
}

/// Immutable hit-testable region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitRegion {
    /// Stable target identifier.
    pub target_id: super::shape::PointerTargetId,
    /// Region bounds.
    pub bounds: super::shape::PointerRect,
    /// Higher values win when regions overlap.
    pub z_order: i32,
    /// Whether the region participates in hit testing.
    pub visible: bool,
    /// Whether the region can emit an action.
    pub enabled: bool,
    /// Capture behavior.
    pub capture: HitCapturePolicy,
    /// Optional action emitted by successful routes.
    pub action_id: Option<super::shape::PointerActionId>,
    /// Optional diagnostic metadata supplied by the renderer.
    pub diagnostic: Option<String>,
}

impl HitRegion {
    /// Creates an enabled visible capturing hit region.
    #[must_use]
    pub fn new(
        target_id: impl Into<super::shape::PointerTargetId>,
        bounds: super::shape::PointerRect,
        z_order: i32,
    ) -> Self {
        Self {
            target_id: target_id.into(),
            bounds,
            z_order,
            visible: true,
            enabled: true,
            capture: HitCapturePolicy::Capture,
            action_id: None,
            diagnostic: None,
        }
    }

    /// Sets region visibility.
    #[must_use]
    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets region enablement.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets capture policy.
    #[must_use]
    pub const fn with_capture(mut self, capture: HitCapturePolicy) -> Self {
        self.capture = capture;
        self
    }

    /// Sets the associated action ID.
    #[must_use]
    pub fn with_action(mut self, action_id: impl Into<super::shape::PointerActionId>) -> Self {
        self.action_id = Some(action_id.into());
        self
    }

    /// Sets diagnostic metadata.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}

/// Immutable collection of hit regions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HitMap {
    /// Regions in renderer-supplied order.
    pub regions: Vec<HitRegion>,
}

impl HitMap {
    /// Creates a hit map.
    #[must_use]
    pub fn new(regions: Vec<HitRegion>) -> Self {
        Self { regions }
    }

    /// Finds a region by target identifier.
    #[must_use]
    pub fn target(&self, target_id: &str) -> Option<&HitRegion> {
        self.regions
            .iter()
            .find(|region| region.target_id == target_id)
    }
}
