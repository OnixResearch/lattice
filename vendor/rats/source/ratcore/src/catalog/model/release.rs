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

/// Release-facing cross-renderer fixture parity row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParityRow {
    /// Fixture ID.
    pub fixture_id: String,
    /// Component family label expected for the fixture.
    pub family: String,
    /// Generated documentation page path or stable target.
    pub documentation_target: String,
    /// Renderer support and evidence statuses.
    pub renderers: Vec<FixtureParityRenderer>,
}

impl FixtureParityRow {
    /// Creates a parity baseline row.
    #[must_use]
    pub fn new(
        fixture_id: impl Into<String>,
        family: impl Into<String>,
        documentation_target: impl Into<String>,
    ) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            family: family.into(),
            documentation_target: documentation_target.into(),
            renderers: Vec::new(),
        }
    }

    /// Adds renderer parity metadata.
    #[must_use]
    pub fn with_renderer(mut self, renderer: FixtureParityRenderer) -> Self {
        self.renderers.push(renderer);
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
