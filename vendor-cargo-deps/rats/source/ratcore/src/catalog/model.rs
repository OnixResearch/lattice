semantic_string_id! {
    /// Stable identity for a shared showcase fixture.
    pub struct FixtureId;
}

semantic_string_id! {
    /// Stable identity for a state within a showcase fixture.
    pub struct FixtureStateId;
}

semantic_string_id! {
    /// Stable identity for an interaction control within a showcase fixture.
    pub struct FixtureControlId;
}

/// Input data or prop key/value for a fixture state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureDatum {
    /// Stable data key.
    pub key: String,
    /// Display or serialized data value.
    pub value: String,
}

impl FixtureDatum {
    /// Creates fixture data.
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Interaction control declared by a fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureControl {
    /// Stable control ID.
    pub id: FixtureControlId,
    /// Human-readable label.
    pub label: String,
}

impl FixtureControl {
    /// Creates a fixture control.
    #[must_use]
    pub fn new(id: impl Into<FixtureControlId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Visual or interaction state for a fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureState {
    /// Stable state ID.
    pub id: FixtureStateId,
    /// Human-readable label.
    pub label: String,
    /// Input data for this state.
    pub data: Vec<FixtureDatum>,
}

impl FixtureState {
    /// Creates a fixture state.
    #[must_use]
    pub fn new(id: impl Into<FixtureStateId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            data: Vec::new(),
        }
    }

    /// Adds state data.
    #[must_use]
    pub fn with_data(mut self, data: FixtureDatum) -> Self {
        self.data.push(data);
        self
    }
}

/// Renderer-neutral showcase fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseFixture {
    /// Stable fixture ID.
    pub id: FixtureId,
    /// Component family label.
    pub family: String,
    /// Human-readable title.
    pub title: String,
    /// State matrix for this fixture.
    pub states: Vec<FixtureState>,
    /// Interaction controls.
    pub controls: Vec<FixtureControl>,
    /// Compatibility renderer support metadata for the default two-family view.
    pub renderers: Vec<RendererSupport>,
    /// Typed ordered renderer cells used by scoped validation.
    pub renderer_cells: Vec<FixtureRendererCell>,
}

impl ShowcaseFixture {
    /// Creates a fixture.
    #[must_use]
    pub fn new(id: impl Into<FixtureId>, family: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            family: family.into(),
            title: title.into(),
            states: Vec::new(),
            controls: Vec::new(),
            renderers: Vec::new(),
            renderer_cells: Vec::new(),
        }
    }

    /// Adds a state.
    #[must_use]
    pub fn with_state(mut self, state: FixtureState) -> Self {
        self.states.push(state);
        self
    }

    /// Adds a control.
    #[must_use]
    pub fn with_control(mut self, control: FixtureControl) -> Self {
        self.controls.push(control);
        self
    }

    /// Adds renderer support metadata.
    #[must_use]
    pub fn with_renderer(mut self, support: RendererSupport) -> Self {
        self.renderer_cells.push(FixtureRendererCell::from(&support));
        self.renderers.push(support);
        self
    }

    /// Adds a typed renderer cell without extending the two-family compatibility view.
    #[must_use]
    pub fn with_renderer_cell(mut self, cell: FixtureRendererCell) -> Self {
        self.renderer_cells.push(cell);
        self
    }
}

/// Stable fixture validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FixtureIssueCode {
    /// Fixture ID is duplicated.
    DuplicateFixtureId,
    /// Fixture ID or title is empty.
    MalformedFixture,
    /// Fixture component family is unsupported.
    UnsupportedFamily,
    /// Fixture has no states.
    MissingState,
    /// State data is malformed.
    InvalidStateData,
    /// Renderer metadata is missing or duplicated.
    UnsupportedRenderer,
    /// Renderer scope or registry policy is invalid.
    InvalidRendererPolicy,
    /// A required scoped renderer cell is missing.
    MissingRendererCell,
    /// A scoped renderer cell is duplicated, unknown, or out of scope.
    InvalidRendererCell,
    /// Parity baseline row is duplicated.
    ParityDuplicateRow,
    /// Parity baseline row is missing required renderer metadata.
    ParityMissingRenderer,
    /// Parity baseline unsupported renderer lacks a reason.
    ParityMissingUnsupportedReason,
    /// Parity baseline references an unknown fixture.
    ParityUnknownFixture,
    /// Parity baseline references a fixture rejected by catalog validation.
    ParityRejectedFixture,
    /// Parity baseline no longer matches fixture coverage.
    ParityStaleCoverage,
    /// Parity baseline row is missing a generated documentation target.
    ParityMissingDocumentationTarget,
}

/// Deterministic fixture validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureIssue {
    /// Stable issue code.
    pub code: FixtureIssueCode,
    /// Fixture ID or field associated with the issue.
    pub subject: String,
    /// Human-readable message.
    pub message: String,
}

impl FixtureIssue {
    pub(super) fn new(code: FixtureIssueCode, subject: impl ToString, message: impl Into<String>) -> Self {
        Self {
            code,
            subject: subject.to_string(),
            message: message.into(),
        }
    }
}

/// Fixture coverage bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FixtureCoverageKind {
    /// Supported by both Subwayrat and Meteorite.
    Shared,
    /// Supported by Subwayrat only.
    TuiOnly,
    /// Supported by Meteorite only.
    MeteoriteOnly,
    /// Unsupported by both renderers.
    Unsupported,
}

/// One ordered scoped fixture coverage cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureCoverageCell {
    /// Nominal renderer family identity.
    pub family: crate::targeting_core::RendererFamilyId,
    /// `Some(true)` for supported, `Some(false)` for unsupported, and `None` for missing.
    pub supported: Option<bool>,
    /// Unsupported or missing reason.
    pub reason: Option<String>,
}

/// Coverage entry for one fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureCoverageEntry {
    /// Fixture ID.
    pub fixture_id: FixtureId,
    /// Compatibility coverage bucket for Subwayrat and Meteorite.
    pub kind: FixtureCoverageKind,
    /// Missing renderer labels or unsupported reasons.
    pub gaps: Vec<String>,
    /// Ordered source-of-truth cells from the active renderer scope.
    pub cells: Vec<FixtureCoverageCell>,
}

/// Validation and coverage result for a fixture set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureSetReport {
    /// Deterministic validation issues.
    pub issues: Vec<FixtureIssue>,
    /// Coverage entries sorted by fixture ID.
    pub coverage: Vec<FixtureCoverageEntry>,
}

impl FixtureSetReport {
    /// Returns true when no validation issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Snapshot evidence status for a renderer in the parity baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FixtureParityEvidenceStatus {
    /// Required docs and snapshot evidence are expected to exist.
    Complete,
    /// Evidence is not present yet and must be reported by validation gates.
    Missing,
    /// Evidence is intentionally unsupported for this renderer.
    Unsupported,
}

/// Renderer-specific parity status for a fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureParityRenderer {
    /// Renderer label.
    pub renderer: ShowcaseRenderer,
    /// Whether the renderer supports this fixture.
    pub supported: bool,
    /// Snapshot/documentation evidence status for this renderer.
    pub evidence: FixtureParityEvidenceStatus,
    /// Required reason when unsupported.
    pub unsupported_reason: Option<String>,
}

mod targets;
pub use targets::*;

mod release;
pub use release::*;
