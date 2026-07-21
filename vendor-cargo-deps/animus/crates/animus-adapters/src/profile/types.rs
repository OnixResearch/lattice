/// Host trust decision for project-local profiles.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProjectTrustMode {
    /// Project profiles are omitted from prompt composition.
    #[default]
    DenyProjectPolicy,
    /// Trusted project profiles may contribute to prompt composition.
    AdmitTrustedProjectPolicy,
}

/// Policy descriptor boundary relative to runtime enforcement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnforcementBoundary {
    /// Descriptor is prompt/configuration guidance only.
    GuidanceOnly,
}

/// Runtime guard decision required for capability enforcement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeGuardRequirement {
    /// A typed runtime capability guard must still enforce the request.
    Required,
}

/// Prompt segment attributed to a policy profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptSegment {
    pub(super) provenance: super::Text,
    pub(super) text: super::Text,
}

impl PromptSegment {
    /// Returns safe policy provenance/source attribution.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }

    /// Returns bounded prompt text.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}

/// Diagnostic emitted while composing policy profiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionDiagnostic {
    pub(super) kind: DiagnosticKind,
    pub(super) provenance: super::Text,
}

impl CompositionDiagnostic {
    /// Returns the diagnostic kind.
    #[must_use]
    pub const fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    /// Returns safe policy provenance/source attribution.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }
}

/// Diagnostic class emitted while composing policy profiles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticKind {
    /// Project-local profile was omitted because host trust did not admit it.
    ProjectPolicyOmitted,
    /// Project-local side-effect rule was defeated by a global safety rule.
    GlobalSafetyPrecedence,
}

/// Result of deterministic policy profile composition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionReport {
    pub(super) segments: super::Collection<PromptSegment>,
    pub(super) diagnostics: super::Collection<CompositionDiagnostic>,
}

impl CompositionReport {
    /// Returns prompt segments in deterministic precedence order.
    #[must_use]
    pub fn segments(&self) -> &[PromptSegment] {
        self.segments.as_slice()
    }

    /// Returns safe composition diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[CompositionDiagnostic] {
        self.diagnostics.as_slice()
    }
}

/// Errors raised while validating policy profiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Generic adapter field contract error.
    Contract(crate::AdapterContractError),
    /// Descriptor value was not one of the supported labels.
    UnsupportedValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: super::Text,
    },
    /// Priority was outside the supported range.
    PriorityOutOfRange {
        /// Minimum supported priority.
        min: u16,
        /// Maximum supported priority.
        max: u16,
        /// Actual priority.
        actual: u16,
    },
    /// Collection exceeded its bounded item ceiling.
    TooManyItems {
        /// Boundary collection field name.
        field: &'static str,
        /// Maximum supported item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// Prompt hygiene validation failed.
    PromptHygiene {
        /// Specific prompt hygiene issue.
        issue: PromptHygieneIssue,
    },
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

/// Prompt hygiene issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptHygieneIssue {
    /// Success criteria were absent.
    MissingSuccessCriteria,
    /// Validation rules were absent.
    MissingValidationRule,
    /// Stop rules were absent.
    MissingStopRule,
    /// Process-heavy script text was detected.
    ProcessHeavy,
    /// Unbounded prompt text was detected.
    UnboundedInstruction,
    /// Stale model-specific guidance was detected.
    StaleModelGuidance,
    /// Prompt text exceeded the bounded budget.
    PromptTooLong,
}
