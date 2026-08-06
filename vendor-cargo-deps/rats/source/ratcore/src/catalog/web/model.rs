/// Browser showcase evidence status for a catalog entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserShowcaseEvidenceStatus {
    /// Package, bundle, and browser smoke evidence are all recorded.
    Complete,
    /// Some browser evidence is recorded while known gaps remain visible.
    Partial,
    /// The browser entry is intentionally unsupported in this repo.
    Unsupported,
}

impl BrowserShowcaseEvidenceStatus {
    /// Stable display label for the evidence status.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Complete => super::STATUS_COMPLETE,
            Self::Partial => super::STATUS_PARTIAL,
            Self::Unsupported => super::STATUS_UNSUPPORTED,
        }
    }
}

/// Renderer-neutral browser showcase catalog entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserShowcaseEntry {
    /// Stable browser showcase entry ID.
    pub id: String,
    /// Human-readable showcase title.
    pub title: String,
    /// Compatibility renderer label shown to users.
    pub renderer_label: String,
    /// Nominal concrete browser target identity.
    pub renderer_target: crate::targeting_core::RendererTargetId,
    /// Cargo package or app package name.
    pub package_name: String,
    /// Repository-relative crate path.
    pub crate_path: String,
    /// Command that launches the browser showcase locally.
    pub launch_command: String,
    /// Command that builds the browser bundle.
    pub bundle_command: String,
    /// Root validation profile or evidence label.
    pub validation_profile: String,
    /// Shared fixture IDs supported by this browser showcase.
    pub supported_fixture_ids: Vec<super::super::FixtureId>,
    /// Repository evidence links backing the status.
    pub evidence_links: Vec<String>,
    /// Browser evidence status.
    pub evidence_status: BrowserShowcaseEvidenceStatus,
    /// Unsupported or incomplete-evidence reasons kept visible to users.
    pub unsupported_reasons: Vec<String>,
}

impl BrowserShowcaseEntry {
    /// Creates a browser showcase catalog entry with required identity fields.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        renderer_label: impl Into<String>,
        package_name: impl Into<String>,
        crate_path: impl Into<String>,
    ) -> Self {
        let renderer_label = renderer_label.into();
        let renderer_target = super::legacy_browser_target(&renderer_label);
        Self {
            id: id.into(),
            title: title.into(),
            renderer_label,
            renderer_target,
            package_name: package_name.into(),
            crate_path: crate_path.into(),
            launch_command: String::new(),
            bundle_command: String::new(),
            validation_profile: String::new(),
            supported_fixture_ids: Vec::new(),
            evidence_links: Vec::new(),
            evidence_status: BrowserShowcaseEvidenceStatus::Partial,
            unsupported_reasons: Vec::new(),
        }
    }

    /// Adds launch and bundle commands.
    #[must_use]
    pub fn with_commands(mut self, launch_command: impl Into<String>, bundle_command: impl Into<String>) -> Self {
        self.launch_command = launch_command.into();
        self.bundle_command = bundle_command.into();
        self
    }

    /// Adds the validation profile label.
    #[must_use]
    pub fn with_validation_profile(mut self, validation_profile: impl Into<String>) -> Self {
        self.validation_profile = validation_profile.into();
        self
    }

    /// Adds supported shared fixture IDs.
    #[must_use]
    pub fn with_supported_fixture_ids(mut self, fixture_ids: &[&str]) -> Self {
        self.supported_fixture_ids =
            fixture_ids.iter().map(|fixture_id| super::super::FixtureId::new(*fixture_id)).collect();
        self
    }

    /// Adds evidence links.
    #[must_use]
    pub fn with_evidence_links(mut self, links: &[&str]) -> Self {
        self.evidence_links = links.iter().map(|link| (*link).to_string()).collect();
        self
    }

    /// Sets the evidence status.
    #[must_use]
    pub fn with_evidence_status(mut self, status: BrowserShowcaseEvidenceStatus) -> Self {
        self.evidence_status = status;
        self
    }

    /// Adds unsupported or incomplete-evidence reasons.
    #[must_use]
    pub fn with_unsupported_reasons(mut self, reasons: &[&str]) -> Self {
        self.unsupported_reasons = reasons.iter().map(|reason| (*reason).to_string()).collect();
        self
    }
}

/// Browser showcase validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BrowserShowcaseIssueCode {
    /// Browser showcase entry ID is duplicated.
    DuplicateEntryId,
    /// Required text field is missing.
    MissingRequiredField,
    /// Launch or bundle command is missing.
    MissingCommand,
    /// Supported fixture list is empty or references an unknown fixture.
    InvalidFixtureReference,
    /// Complete evidence status lacks evidence links.
    CompleteEvidenceWithoutLinks,
    /// Unsupported evidence status lacks a visible reason.
    UnsupportedWithoutReason,
    /// Browser renderer registry or scope is invalid.
    InvalidRendererScope,
    /// Browser target is unknown or outside the browser scope.
    InvalidRendererTarget,
    /// Compatibility label does not map to the nominal target.
    MismatchedRendererLabel,
}

/// Deterministic browser showcase validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserShowcaseIssue {
    /// Stable issue code.
    pub code: BrowserShowcaseIssueCode,
    /// Entry ID associated with the issue.
    pub entry_id: String,
    /// Field associated with the issue.
    pub field: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl BrowserShowcaseIssue {
    pub(super) fn new(
        code: BrowserShowcaseIssueCode,
        entry_id: impl Into<String>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            entry_id: entry_id.into(),
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Browser showcase index with validation issues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserShowcaseIndex {
    /// Browser showcase entries sorted by entry ID.
    pub entries: Vec<BrowserShowcaseEntry>,
    /// Deterministic validation issues.
    pub issues: Vec<BrowserShowcaseIssue>,
}

impl BrowserShowcaseIndex {
    /// Returns true when the index has no validation issues.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Display row for browser fixture navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserFixtureNavigationRow {
    /// Shared fixture ID.
    pub fixture_id: super::super::FixtureId,
    /// Shared fixture label.
    pub label: String,
    /// Browser renderer label.
    pub renderer_label: String,
    /// Evidence status label.
    pub support_status: String,
    /// Evidence links or visible gap reasons.
    pub evidence_text: String,
    /// Launch or help context from the browser catalog.
    pub launch_context: String,
}

/// Browser fixture navigation projection with diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserFixtureNavigation {
    /// Navigation rows sorted by fixture ID.
    pub rows: Vec<BrowserFixtureNavigationRow>,
    /// Deterministic validation issues.
    pub issues: Vec<BrowserShowcaseIssue>,
}

impl BrowserFixtureNavigation {
    /// Returns true when navigation has no diagnostics.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}
