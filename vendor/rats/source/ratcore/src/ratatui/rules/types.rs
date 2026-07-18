//! Feature-rule data types.

pub(super) const POLICY_APPROVED: &str = "approved";
pub(super) const POLICY_FORBIDDEN: &str = "forbidden";
pub(super) const POLICY_REQUIRES_CHANGE: &str = "requires-change";
pub(super) const APP_SHELL_FAMILY: &str = "app-shell";
pub(super) const BACKEND_ADAPTER_FAMILY: &str = "backend-adapter";
pub(super) const REUSABLE_WIDGET_FAMILY: &str = "reusable-widget";
pub(super) const RATZILLA_WEB_FAMILY: &str = "ratzilla-web";
pub(super) const OPTIONAL_DATA_FAMILY: &str = "optional-data";
pub(super) const THEME_DOCS_FAMILY: &str = "theme-docs";
pub(super) const CALENDAR_WIDGET_FAMILY: &str = "calendar-widget";
pub(super) const LAYOUT_EXPERIMENT_FAMILY: &str = "layout-experiment";
pub(super) const PRIVATE_EXPERIMENT_FAMILY: &str = "private-experiment";
pub(super) const WORKSPACE_POLICY_FAMILY: &str = "workspace-policy";
pub(super) const FEATURE_CROSSTERM: &str = "crossterm";
pub(super) const FEATURE_SERDE: &str = "serde";
pub(super) const FEATURE_PALETTE: &str = "palette";
pub(super) const FEATURE_WIDGET_CALENDAR: &str = "widget-calendar";
pub(super) const FEATURE_LAYOUT_CACHE: &str = "layout-cache";
pub(super) const FEATURE_UNSTABLE: &str = "unstable";
pub(super) const FEATURE_MACROS: &str = "macros";
pub(super) const WEB_INCOMPATIBLE_FEATURES: &[&str] = &[FEATURE_CROSSTERM];
pub(super) const KNOWN_POLICY_STATUSES: &[&str] =
    &[POLICY_APPROVED, POLICY_FORBIDDEN, POLICY_REQUIRES_CHANGE];
pub(super) const BASELINE_POLICY_ROW_COUNT: usize = 10;

/// Whether a `Ratatui` feature use participates in web/Ratzilla builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureUseTarget {
    /// Native-only package or app-shell use.
    NativeOnly,
    /// Package participates in a web-consumed validation profile.
    WebConsumed,
}

/// Constructor parts for one checked-in `Ratatui` feature policy row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeaturePolicyRowParts {
    /// Crate family or policy family the row applies to.
    pub crate_family: String,
    /// Ratatui feature name.
    pub feature_name: String,
    /// Policy status: `approved`, `forbidden`, or `requires-change`.
    pub status: String,
    /// Human owner for the feature decision.
    pub owner: String,
    /// Compatibility scope, such as native-only, web-safe, or private experiment.
    pub compatibility_scope: String,
    /// Reason for approving, forbidding, or staging the feature.
    pub reason: String,
    /// Fallback or fail-closed behavior when the feature is absent or forbidden.
    pub fallback: String,
}

/// One checked-in `Ratatui` feature policy row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeaturePolicyRow {
    /// Crate family or policy family the row applies to.
    pub crate_family: String,
    /// Ratatui feature name.
    pub feature_name: String,
    /// Policy status: `approved`, `forbidden`, or `requires-change`.
    pub status: String,
    /// Human owner for the feature decision.
    pub owner: String,
    /// Compatibility scope, such as native-only, web-safe, or private experiment.
    pub compatibility_scope: String,
    /// Reason for approving, forbidding, or staging the feature.
    pub reason: String,
    /// Evidence paths or commands supporting approved use.
    pub evidence: Vec<String>,
    /// Fallback or fail-closed behavior when the feature is absent or forbidden.
    pub fallback: String,
    /// Required follow-up for rows that are not accepted implementation policy yet.
    pub next_action: String,
}

impl FeaturePolicyRow {
    /// Creates a feature policy row with required policy fields.
    #[must_use]
    pub fn new(parts: FeaturePolicyRowParts) -> Self {
        Self {
            crate_family: parts.crate_family,
            feature_name: parts.feature_name,
            status: parts.status,
            owner: parts.owner,
            compatibility_scope: parts.compatibility_scope,
            reason: parts.reason,
            evidence: Vec::new(),
            fallback: parts.fallback,
            next_action: String::new(),
        }
    }

    /// Adds evidence for an approved policy row.
    #[must_use]
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    /// Adds a next action for rows that require a future Cairn change.
    #[must_use]
    pub fn with_next_action(mut self, next_action: impl Into<String>) -> Self {
        self.next_action = next_action.into();
        self
    }
}

/// One observed `Ratatui` feature use reported by a manifest or metadata check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureUse {
    /// Crate name using the feature.
    pub crate_name: String,
    /// Crate family used to match policy rows.
    pub crate_family: String,
    /// Ratatui feature name.
    pub feature_name: String,
    /// Native-only or web-consumed target classification.
    pub target: FeatureUseTarget,
}

impl FeatureUse {
    /// Creates a feature-use report row.
    #[must_use]
    pub fn new(
        crate_name: impl Into<String>,
        crate_family: impl Into<String>,
        feature_name: impl Into<String>,
        target: FeatureUseTarget,
    ) -> Self {
        Self {
            crate_name: crate_name.into(),
            crate_family: crate_family.into(),
            feature_name: feature_name.into(),
            target,
        }
    }
}

/// Stable feature-policy diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeaturePolicyIssueCode {
    /// Policy row key is duplicated.
    DuplicatePolicyKey,
    /// Required policy or feature-use field is empty.
    MissingRequiredField,
    /// Policy status is unknown.
    UnknownStatus,
    /// Approved policy row lacks supporting evidence.
    MissingEvidence,
    /// Policy row lacks a reason.
    MissingReason,
    /// Policy row lacks fallback behavior.
    MissingFallback,
    /// Staged policy row lacks a next action.
    MissingNextAction,
    /// Feature use has no accepted policy row.
    UnapprovedFeatureUse,
    /// Feature use matches an explicit forbidden policy row.
    ForbiddenFeatureUse,
    /// Web-consumed crate uses a feature classified as web-incompatible.
    WebIncompatibleFeatureUse,
}

/// Deterministic feature-policy validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeaturePolicyIssue {
    /// Stable diagnostic code.
    pub code: FeaturePolicyIssueCode,
    /// Affected policy or crate key.
    pub key: String,
    /// Affected field or crate name.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl FeaturePolicyIssue {
    pub(super) fn new(
        code: FeaturePolicyIssueCode,
        key: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            key: key.into(),
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Validation report for policy rows or feature uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeaturePolicyReport {
    /// Deterministic validation issues.
    pub issues: Vec<FeaturePolicyIssue>,
    /// Policy rows sorted by stable key.
    pub rows: Vec<FeaturePolicyRow>,
}

impl FeaturePolicyReport {
    /// Returns true when no feature-policy issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}
