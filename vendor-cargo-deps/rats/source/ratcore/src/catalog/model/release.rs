use super::*;

impl FixtureParityRenderer {
    /// Marks a renderer as supported in the parity baseline.
    #[must_use]
    pub fn supported(renderer: ShowcaseRenderer, evidence: FixtureParityEvidenceStatus) -> Self {
        Self {
            renderer,
            supported: true,
            evidence,
            unsupported_reason: None,
        }
    }

    /// Marks a renderer as unsupported in the parity baseline.
    #[must_use]
    pub fn unsupported(renderer: ShowcaseRenderer, reason: impl Into<String>) -> Self {
        Self {
            renderer,
            supported: false,
            evidence: FixtureParityEvidenceStatus::Unsupported,
            unsupported_reason: Some(reason.into()),
        }
    }
}

/// Typed source-of-truth parity cell for one renderer family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParityCell {
    /// Nominal renderer family identity.
    pub family: crate::targeting_core::RendererFamilyId,
    /// Whether the renderer supports the fixture.
    pub supported: bool,
    /// Snapshot/documentation evidence status.
    pub evidence: FixtureParityEvidenceStatus,
    /// Required reason when unsupported.
    pub unsupported_reason: Option<String>,
}

impl FixtureParityCell {
    /// Creates a supported typed parity cell.
    #[must_use]
    pub fn supported(
        family: impl Into<crate::targeting_core::RendererFamilyId>,
        evidence: FixtureParityEvidenceStatus,
    ) -> Self {
        Self {
            family: family.into(),
            supported: true,
            evidence,
            unsupported_reason: None,
        }
    }

    /// Creates an unsupported typed parity cell.
    #[must_use]
    pub fn unsupported(family: impl Into<crate::targeting_core::RendererFamilyId>, reason: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            supported: false,
            evidence: FixtureParityEvidenceStatus::Unsupported,
            unsupported_reason: Some(reason.into()),
        }
    }
}

impl From<&FixtureParityRenderer> for FixtureParityCell {
    fn from(renderer: &FixtureParityRenderer) -> Self {
        Self {
            family: renderer.renderer.family_id(),
            supported: renderer.supported,
            evidence: renderer.evidence,
            unsupported_reason: renderer.unsupported_reason.clone(),
        }
    }
}

/// Release-facing cross-renderer fixture parity row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParityRow {
    /// Fixture ID.
    pub fixture_id: FixtureId,
    /// Component family label expected for the fixture.
    pub family: String,
    /// Generated documentation page path or stable target.
    pub documentation_target: String,
    /// Compatibility renderer support and evidence statuses.
    pub renderers: Vec<FixtureParityRenderer>,
    /// Typed ordered renderer cells used by scoped validation.
    pub renderer_cells: Vec<FixtureParityCell>,
}

impl FixtureParityRow {
    /// Creates a parity baseline row.
    #[must_use]
    pub fn new(
        fixture_id: impl Into<FixtureId>,
        family: impl Into<String>,
        documentation_target: impl Into<String>,
    ) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            family: family.into(),
            documentation_target: documentation_target.into(),
            renderers: Vec::new(),
            renderer_cells: Vec::new(),
        }
    }

    /// Adds renderer parity metadata.
    #[must_use]
    pub fn with_renderer(mut self, renderer: FixtureParityRenderer) -> Self {
        self.renderer_cells.push(FixtureParityCell::from(&renderer));
        self.renderers.push(renderer);
        self
    }

    /// Adds a typed renderer parity cell outside the compatibility view.
    #[must_use]
    pub fn with_renderer_cell(mut self, cell: FixtureParityCell) -> Self {
        self.renderer_cells.push(cell);
        self
    }
}

/// Validation result for a cross-renderer fixture parity baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParityReport {
    /// Deterministic validation issues.
    pub issues: Vec<FixtureIssue>,
    /// Baseline rows sorted by fixture ID.
    pub rows: Vec<FixtureParityRow>,
}

impl FixtureParityReport {
    /// Returns true when no parity validation issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}
