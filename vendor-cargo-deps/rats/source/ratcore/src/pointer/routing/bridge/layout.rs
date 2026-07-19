/// Renderer-produced hit region derived from visible geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPointerRegion {
    /// Stable target identifier.
    pub target_id: super::super::shape::PointerTargetId,
    /// Rendered bounds.
    pub bounds: super::super::shape::PointerRect,
    /// Stacking order.
    pub z_order: i32,
    /// Whether the region is visible.
    pub visible: bool,
    /// Whether the region is enabled.
    pub enabled: bool,
    /// Capture policy.
    pub capture: super::super::area::HitCapturePolicy,
    /// Optional action identifier.
    pub action_id: Option<super::super::shape::PointerActionId>,
    /// Optional diagnostic.
    pub diagnostic: Option<String>,
}

impl RenderedPointerRegion {
    /// Creates a rendered pointer region.
    #[must_use]
    pub fn new(
        target_id: impl Into<super::super::shape::PointerTargetId>,
        bounds: super::super::shape::PointerRect,
        z_order: i32,
    ) -> Self {
        Self {
            target_id: target_id.into(),
            bounds,
            z_order,
            visible: true,
            enabled: true,
            capture: super::super::area::HitCapturePolicy::Capture,
            action_id: None,
            diagnostic: None,
        }
    }

    /// Sets visibility.
    #[must_use]
    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets enablement.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets capture policy.
    #[must_use]
    pub const fn with_capture(mut self, capture: super::super::area::HitCapturePolicy) -> Self {
        self.capture = capture;
        self
    }

    /// Sets action identifier.
    #[must_use]
    pub fn with_action(
        mut self,
        action_id: impl Into<super::super::shape::PointerActionId>,
    ) -> Self {
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

/// Derives a shared hit map from renderer geometry projections.
#[must_use]
pub fn derive_pointer_hit_map(regions: Vec<RenderedPointerRegion>) -> super::super::area::HitMap {
    super::super::area::HitMap::new(
        regions
            .into_iter()
            .map(rendered_region_to_hit_region)
            .collect(),
    )
}

fn rendered_region_to_hit_region(region: RenderedPointerRegion) -> super::super::area::HitRegion {
    let mut hit_region =
        super::super::area::HitRegion::new(region.target_id, region.bounds, region.z_order)
            .with_visible(region.visible)
            .with_enabled(region.enabled)
            .with_capture(region.capture);
    if let Some(action_id) = region.action_id {
        hit_region = hit_region.with_action(action_id);
    }
    if let Some(diagnostic) = region.diagnostic {
        hit_region = hit_region.with_diagnostic(diagnostic);
    }
    hit_region
}
