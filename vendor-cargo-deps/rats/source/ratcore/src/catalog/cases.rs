//! Showcase edge-case catalog validation.

pub(super) const KIND_INVALID: Kind = Kind::Invalid;
pub(super) const KIND_MALFORMED: Kind = Kind::Malformed;
pub(super) const KIND_UNSUPPORTED: Kind = Kind::Unsupported;

/// Edge-case state family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// Normal happy-path state.
    HappyPath,
    /// Empty data state.
    Empty,
    /// Boundary-size or boundary-value state.
    Boundary,
    /// Disabled action state.
    Disabled,
    /// Invalid user input state.
    Invalid,
    /// Malformed fixture or serialized data state.
    Malformed,
    /// Oversized content state.
    Oversized,
    /// Unsupported renderer capability state.
    Unsupported,
}

/// Renderer visibility for one case state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Renderer {
    /// Renderer label.
    pub renderer: super::ShowcaseRenderer,
    /// Whether the state is visible in that renderer.
    pub visible: bool,
    /// Required reason when hidden or unsupported.
    pub reason: Option<String>,
}

impl Renderer {
    /// Marks a state visible for a renderer.
    #[must_use]
    pub fn visible(renderer: super::ShowcaseRenderer) -> Self {
        Self {
            renderer,
            visible: true,
            reason: None,
        }
    }

    /// Marks a state unsupported with a reason.
    #[must_use]
    pub fn unsupported(renderer: super::ShowcaseRenderer, reason: impl Into<String>) -> Self {
        Self {
            renderer,
            visible: false,
            reason: Some(reason.into()),
        }
    }
}

/// One named edge state for a fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// Stable state ID.
    pub id: super::FixtureStateId,
    /// Display label.
    pub label: String,
    /// State kind.
    pub kind: Kind,
    /// Expected diagnostics for invalid, malformed, or unsupported states.
    pub diagnostics: Vec<String>,
    /// Renderer visibility metadata.
    pub renderers: Vec<Renderer>,
}

impl State {
    /// Creates an edge state.
    #[must_use]
    pub fn new(id: impl Into<super::FixtureStateId>, label: impl Into<String>, kind: Kind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            diagnostics: Vec::new(),
            renderers: Vec::new(),
        }
    }

    /// Adds an expected diagnostic.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostics.push(diagnostic.into());
        self
    }

    /// Adds renderer visibility metadata.
    #[must_use]
    pub fn with_renderer(mut self, renderer: Renderer) -> Self {
        self.renderers.push(renderer);
        self
    }
}

/// Edge-case catalog entry for one shared fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    /// Stable fixture ID from the shared catalog.
    pub fixture_id: super::FixtureId,
    /// Coverage or parity reference proving the base fixture is current.
    pub coverage_reference: String,
    /// Edge states for the fixture.
    pub states: Vec<State>,
}

impl Fixture {
    /// Creates a catalog entry.
    #[must_use]
    pub fn new(fixture_id: impl Into<super::FixtureId>, coverage_reference: impl Into<String>) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            coverage_reference: coverage_reference.into(),
            states: Vec::new(),
        }
    }

    /// Adds an edge state.
    #[must_use]
    pub fn with_state(mut self, state: State) -> Self {
        self.states.push(state);
        self
    }
}

/// Stable validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueCode {
    /// Fixture ID is duplicated.
    DuplicateFixtureId,
    /// State ID is duplicated within a fixture.
    DuplicateStateId,
    /// Required fixture or state metadata is missing.
    MissingRequiredField,
    /// Invalid, malformed, or unsupported state lacks diagnostics.
    MissingDiagnostic,
    /// State lacks renderer metadata.
    MissingRendererSupport,
    /// Unsupported renderer declaration lacks a reason.
    MissingUnsupportedReason,
    /// Entry references a fixture outside accepted coverage.
    StaleCoverageReference,
}

/// Deterministic validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Stable issue code.
    pub code: IssueCode,
    /// Fixture ID associated with the issue.
    pub fixture_id: super::FixtureId,
    /// State ID, renderer label, or field associated with the issue.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl Issue {
    pub(super) fn new(
        code: IssueCode,
        fixture_id: impl Into<super::FixtureId>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            fixture_id: fixture_id.into(),
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Validation report for showcase edge cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// Deterministic validation issues.
    pub issues: Vec<Issue>,
    /// Fixtures sorted by stable fixture ID.
    pub fixtures: Vec<Fixture>,
}

impl Report {
    /// Returns true when no validation issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

mod checks;
pub use checks::validate;
mod projection;
pub use projection::EdgeDisplay;
pub use projection::EdgeDisplayRow;
pub use projection::build_edge_display;
pub use projection::built_in_edge_display;
pub use projection::built_in_edge_fixtures;

#[cfg(test)]
#[path = "cases/tests.rs"]
mod tests;
