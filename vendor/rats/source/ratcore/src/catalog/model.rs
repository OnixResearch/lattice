/// Renderer label supported by shared showcase fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShowcaseRenderer {
    /// Subwayrat terminal renderer.
    Subwayrat,
    /// Meteorite web renderer.
    Meteorite,
}

impl ShowcaseRenderer {
    /// Stable renderer label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Subwayrat => "subwayrat",
            Self::Meteorite => "meteorite",
        }
    }
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
    pub id: String,
    /// Human-readable label.
    pub label: String,
}

impl FixtureControl {
    /// Creates a fixture control.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
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
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Input data for this state.
    pub data: Vec<FixtureDatum>,
}

impl FixtureState {
    /// Creates a fixture state.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
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

/// Renderer support metadata for one fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererSupport {
    /// Renderer label.
    pub renderer: ShowcaseRenderer,
    /// Whether the renderer supports this fixture.
    pub supported: bool,
    /// Optional reason when unsupported.
    pub reason: Option<String>,
}

impl RendererSupport {
    /// Marks a renderer as supported.
    #[must_use]
    pub fn supported(renderer: ShowcaseRenderer) -> Self {
        Self {
            renderer,
            supported: true,
            reason: None,
        }
    }

    /// Marks a renderer as unsupported with a reason.
    #[must_use]
    pub fn unsupported(renderer: ShowcaseRenderer, reason: impl Into<String>) -> Self {
        Self {
            renderer,
            supported: false,
            reason: Some(reason.into()),
        }
    }
}

/// Renderer-neutral showcase fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowcaseFixture {
    /// Stable fixture ID.
    pub id: String,
    /// Component family label.
    pub family: String,
    /// Human-readable title.
    pub title: String,
    /// State matrix for this fixture.
    pub states: Vec<FixtureState>,
    /// Interaction controls.
    pub controls: Vec<FixtureControl>,
    /// Renderer support metadata.
    pub renderers: Vec<RendererSupport>,
}

impl ShowcaseFixture {
    /// Creates a fixture.
    #[must_use]
    pub fn new(id: impl Into<String>, family: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            family: family.into(),
            title: title.into(),
            states: Vec::new(),
            controls: Vec::new(),
            renderers: Vec::new(),
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
        self.renderers.push(support);
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
    pub(super) fn new(
        code: FixtureIssueCode,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            subject: subject.into(),
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

/// Coverage entry for one fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureCoverageEntry {
    /// Fixture ID.
    pub fixture_id: String,
    /// Coverage bucket.
    pub kind: FixtureCoverageKind,
    /// Missing renderer labels or unsupported reasons.
    pub gaps: Vec<String>,
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

mod release;
pub use release::*;
