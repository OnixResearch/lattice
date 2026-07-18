const EMBED_BACKEND_DOM: &str = "dom";
const EMBED_LIFECYCLE_MOUNT: &str = "mount";
const EMBED_LIFECYCLE_REDRAW: &str = "redraw";
const EMBED_LIFECYCLE_RESIZE: &str = "resize";
const EMBED_LIFECYCLE_UNMOUNT: &str = "unmount";
const DEFAULT_FOCUS_POLICY: &str = "host-focus-enters-terminal";
const DEFAULT_INPUT_POLICY: &str = "ratzilla-owns-terminal-input";
const DEFAULT_RESIZE_POLICY: &str = "host-resize-observer";
const MOUNT_KEY_SEPARATOR: &str = "#";
const EMBED_ISSUE_BOUND: usize = 7;
const KNOWN_EMBED_BACKENDS: &[&str] = &[EMBED_BACKEND_DOM];
const KNOWN_EMBED_LIFECYCLES: &[&str] = &[
    EMBED_LIFECYCLE_MOUNT,
    EMBED_LIFECYCLE_REDRAW,
    EMBED_LIFECYCLE_RESIZE,
    EMBED_LIFECYCLE_UNMOUNT,
];

/// Browser backend used by the web showcase shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebBackend {
    /// Ratzilla ratatui web backend.
    Ratzilla,
}

/// Declarative Ratzilla web showcase shell descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebShowcaseShellDescriptor {
    /// Backend.
    pub backend: WebBackend,
    /// Rendered widget identifiers.
    pub widget_ids: Vec<String>,
    /// Browser asset entry point.
    pub asset_entrypoint: String,
    /// Whether crossterm terminal lifecycle is required.
    pub requires_crossterm: bool,
}

/// Backend-neutral crate profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendNeutralWidgetProfile {
    /// Crate name.
    pub crate_name: String,
    /// Whether library dependencies enable ratatui default backend features.
    pub ratatui_default_features_enabled: bool,
    /// Whether the native shell explicitly opts into crossterm.
    pub native_shell_opts_into_crossterm: bool,
}

/// Backend-neutral profile diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendNeutralDiagnostic {
    /// Crate name.
    pub crate_name: String,
    /// Human-readable diagnostic.
    pub message: String,
}

/// Creates a Ratzilla web showcase shell descriptor.
#[must_use]
pub fn build_ratzilla_web_showcase_shell(
    widget_ids: Vec<String>,
    asset_entrypoint: impl Into<String>,
) -> WebShowcaseShellDescriptor {
    WebShowcaseShellDescriptor {
        backend: WebBackend::Ratzilla,
        widget_ids,
        asset_entrypoint: asset_entrypoint.into(),
        requires_crossterm: false,
    }
}

/// Lifecycle diagnostic for an embedded Ratzilla mount request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmbeddedRatzillaDiagnosticCode {
    /// Host ID is empty.
    MissingHostId,
    /// Fixture or widget ID is empty.
    MissingFixtureId,
    /// Backend label is not supported by the current embedded shell.
    UnsupportedBackend,
    /// Initial host size is empty.
    ZeroSizedHost,
    /// A mount already owns the same host key.
    DuplicateMountKey,
    /// Shared fixture catalog does not contain the requested fixture.
    UnsupportedFixture,
    /// Lifecycle operation is not supported by the current shell.
    UnsupportedLifecycle,
}

/// Deterministic embedded Ratzilla mount diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRatzillaDiagnostic {
    /// Stable diagnostic code.
    pub code: EmbeddedRatzillaDiagnosticCode,
    /// Rejected field or operation.
    pub subject: String,
    /// Human-readable diagnostic text.
    pub message: String,
}

impl EmbeddedRatzillaDiagnostic {
    fn new(
        code: EmbeddedRatzillaDiagnosticCode,
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

/// Shared fixture facts used by an embedded Ratzilla surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRatzillaFixtureFacts {
    /// Shared fixture ID.
    pub fixture_id: String,
    /// Shared fixture label.
    pub label: String,
    /// Browser renderer label that owns the fixture.
    pub renderer_label: String,
    /// Support status copied from the shared browser catalog.
    pub support_status: String,
    /// Evidence text copied from the shared browser catalog.
    pub evidence_text: String,
    /// Launch or help context copied from the shared browser catalog.
    pub launch_context: String,
}

impl EmbeddedRatzillaFixtureFacts {
    /// Converts a shared browser fixture row into embedded-surface facts.
    #[must_use]
    pub fn from_browser_fixture_row(
        row: &crate::catalog_core::BrowserFixtureNavigationRow,
    ) -> Self {
        Self {
            fixture_id: row.fixture_id.clone(),
            label: row.label.clone(),
            renderer_label: row.renderer_label.clone(),
            support_status: row.support_status.clone(),
            evidence_text: row.evidence_text.clone(),
            launch_context: row.launch_context.clone(),
        }
    }
}

/// Data-first descriptor for a Meteorite-hosted Ratzilla terminal surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRatzillaMountDescriptor {
    /// Stable host element ID supplied by Meteorite.
    pub host_id: String,
    /// Shared fixture or widget ID selected for embedding.
    pub fixture_id: String,
    /// Backend label. The current supported value is `dom`.
    pub backend: String,
    /// Initial width in terminal cells or host pixels, depending on the shell.
    pub initial_width: u32,
    /// Initial height in terminal cells or host pixels, depending on the shell.
    pub initial_height: u32,
    /// Resize policy label.
    pub resize_policy: String,
    /// Focus policy label.
    pub focus_policy: String,
    /// Input policy label.
    pub input_policy: String,
    /// Lifecycle operation label.
    pub lifecycle: String,
}

impl EmbeddedRatzillaMountDescriptor {
    /// Creates a mount descriptor with deterministic browser-shell defaults.
    #[must_use]
    pub fn new(
        host_id: impl Into<String>,
        fixture_id: impl Into<String>,
        initial_width: u32,
        initial_height: u32,
    ) -> Self {
        Self {
            host_id: host_id.into(),
            fixture_id: fixture_id.into(),
            backend: EMBED_BACKEND_DOM.to_string(),
            initial_width,
            initial_height,
            resize_policy: DEFAULT_RESIZE_POLICY.to_string(),
            focus_policy: DEFAULT_FOCUS_POLICY.to_string(),
            input_policy: DEFAULT_INPUT_POLICY.to_string(),
            lifecycle: EMBED_LIFECYCLE_MOUNT.to_string(),
        }
    }

    /// Sets the backend label.
    #[must_use]
    pub fn with_backend(mut self, backend: impl Into<String>) -> Self {
        self.backend = backend.into();
        self
    }

    /// Sets the lifecycle operation label.
    #[must_use]
    pub fn with_lifecycle(mut self, lifecycle: impl Into<String>) -> Self {
        self.lifecycle = lifecycle.into();
        self
    }
}

/// Evaluated embedded Ratzilla mount plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedRatzillaMountPlan {
    /// Whether the descriptor can be executed without mutating unrelated state.
    pub accepted: bool,
    /// Stable host-based mount key.
    pub mount_key: String,
    /// Host ID supplied by Meteorite.
    pub host_id: String,
    /// Selected fixture ID.
    pub fixture_id: String,
    /// Backend label.
    pub backend: String,
    /// Lifecycle operation label.
    pub lifecycle: String,
    /// Shared fixture facts when the selected fixture is supported.
    pub fixture_facts: Option<EmbeddedRatzillaFixtureFacts>,
    /// Deterministic diagnostics.
    pub diagnostics: Vec<EmbeddedRatzillaDiagnostic>,
}

impl EmbeddedRatzillaMountPlan {
    /// Returns true when the plan has no diagnostics and can be executed by the shell.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        self.accepted && self.diagnostics.is_empty()
    }
}

/// Evaluates an embedded Ratzilla mount descriptor against catalog facts and active mounts.
#[must_use]
pub fn evaluate_embedded_ratzilla_mount(
    descriptor: &EmbeddedRatzillaMountDescriptor,
    fixture_facts: &[EmbeddedRatzillaFixtureFacts],
    active_mount_keys: &[String],
) -> EmbeddedRatzillaMountPlan {
    let mut diagnostics = Vec::with_capacity(EMBED_ISSUE_BOUND);
    let mount_key = embedded_mount_key(descriptor.host_id.as_str());
    if descriptor.host_id.trim().is_empty() {
        diagnostics.push(EmbeddedRatzillaDiagnostic::new(
            EmbeddedRatzillaDiagnosticCode::MissingHostId,
            "host_id",
            "embedded Ratzilla host ID is required",
        ));
    }
    if descriptor.fixture_id.trim().is_empty() {
        diagnostics.push(EmbeddedRatzillaDiagnostic::new(
            EmbeddedRatzillaDiagnosticCode::MissingFixtureId,
            "fixture_id",
            "embedded Ratzilla fixture or widget ID is required",
        ));
    }
    if !KNOWN_EMBED_BACKENDS.contains(&descriptor.backend.as_str()) {
        diagnostics.push(EmbeddedRatzillaDiagnostic::new(
            EmbeddedRatzillaDiagnosticCode::UnsupportedBackend,
            "backend",
            format!(
                "unsupported embedded Ratzilla backend `{}`",
                descriptor.backend
            ),
        ));
    }
    if !KNOWN_EMBED_LIFECYCLES.contains(&descriptor.lifecycle.as_str()) {
        diagnostics.push(EmbeddedRatzillaDiagnostic::new(
            EmbeddedRatzillaDiagnosticCode::UnsupportedLifecycle,
            "lifecycle",
            format!(
                "unsupported embedded Ratzilla lifecycle `{}`",
                descriptor.lifecycle
            ),
        ));
    }
    if descriptor.initial_width == 0 || descriptor.initial_height == 0 {
        diagnostics.push(EmbeddedRatzillaDiagnostic::new(
            EmbeddedRatzillaDiagnosticCode::ZeroSizedHost,
            "initial_size",
            "embedded Ratzilla host must have non-zero initial size",
        ));
    }
    if active_mount_keys.iter().any(|key| key == &mount_key) {
        diagnostics.push(EmbeddedRatzillaDiagnostic::new(
            EmbeddedRatzillaDiagnosticCode::DuplicateMountKey,
            "host_id",
            "embedded Ratzilla host is already mounted",
        ));
    }
    let fixture_facts = fixture_facts
        .iter()
        .find(|facts| facts.fixture_id == descriptor.fixture_id)
        .cloned();
    if fixture_facts.is_none() {
        diagnostics.push(EmbeddedRatzillaDiagnostic::new(
            EmbeddedRatzillaDiagnosticCode::UnsupportedFixture,
            "fixture_id",
            format!(
                "embedded Ratzilla fixture `{}` is not supported",
                descriptor.fixture_id
            ),
        ));
    }
    EmbeddedRatzillaMountPlan {
        accepted: diagnostics.is_empty(),
        mount_key,
        host_id: descriptor.host_id.clone(),
        fixture_id: descriptor.fixture_id.clone(),
        backend: descriptor.backend.clone(),
        lifecycle: descriptor.lifecycle.clone(),
        fixture_facts,
        diagnostics,
    }
}

/// Creates the stable host-based mount key for an embedded Ratzilla surface.
#[must_use]
pub fn embedded_mount_key(host_id: &str) -> String {
    format!("ratzilla{MOUNT_KEY_SEPARATOR}{host_id}")
}

/// Classifies whether a reusable widget crate stays backend-neutral.
#[must_use]
pub fn classify_backend_neutral_widget(
    profile: &BackendNeutralWidgetProfile,
) -> Vec<BackendNeutralDiagnostic> {
    let mut diagnostics = Vec::new();
    if profile.ratatui_default_features_enabled {
        diagnostics.push(BackendNeutralDiagnostic {
            crate_name: profile.crate_name.clone(),
            message: "library crate enables ratatui default backend features".to_string(),
        });
    }
    if !profile.native_shell_opts_into_crossterm {
        diagnostics.push(BackendNeutralDiagnostic {
            crate_name: profile.crate_name.clone(),
            message: "native shell does not explicitly opt into crossterm".to_string(),
        });
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMBED_HOST_ID: &str = "meteorite-ratzilla-host";
    const FIXTURE_CHROME_SHELL: &str = "chrome.shell";
    const FIXTURE_CHROME_LABEL: &str = "Chrome shell";
    const RENDERER_RATZILLA: &str = "subwayrat-ratzilla";
    const SUPPORT_COMPLETE: &str = "complete";
    const EVIDENCE_WEB_RENDERER: &str = "docs/subwayrat-web-renderer.md";
    const LAUNCH_TRUNK_SERVE: &str = "trunk serve";
    const EMBED_WIDTH: u32 = 80;
    const EMBED_HEIGHT: u32 = 24;
    const ZERO_SIZE: u32 = 0;
    const UNSUPPORTED_BACKEND: &str = "webgl";
    const UNSUPPORTED_LIFECYCLE: &str = "hydrate";
    const MISSING_FIXTURE: &str = "missing.fixture";
    const BACKEND_NEUTRAL_DIAGNOSTIC_COUNT: usize = 2;

    #[test]
    fn ratzilla_shell_descriptor_is_browser_shell_only() {
        let shell =
            build_ratzilla_web_showcase_shell(vec!["spinner.basic".to_string()], "index.html");

        assert_eq!(shell.backend, WebBackend::Ratzilla);
        assert!(!shell.requires_crossterm);
    }

    #[test]
    fn embedded_mount_accepts_catalog_fixture_and_dom_backend() {
        let facts = vec![EmbeddedRatzillaFixtureFacts {
            fixture_id: FIXTURE_CHROME_SHELL.to_string(),
            label: FIXTURE_CHROME_LABEL.to_string(),
            renderer_label: RENDERER_RATZILLA.to_string(),
            support_status: SUPPORT_COMPLETE.to_string(),
            evidence_text: EVIDENCE_WEB_RENDERER.to_string(),
            launch_context: LAUNCH_TRUNK_SERVE.to_string(),
        }];
        let descriptor = EmbeddedRatzillaMountDescriptor::new(
            EMBED_HOST_ID,
            FIXTURE_CHROME_SHELL,
            EMBED_WIDTH,
            EMBED_HEIGHT,
        );

        let plan = evaluate_embedded_ratzilla_mount(&descriptor, &facts, &[]);

        assert!(plan.is_accepted(), "diagnostics: {:?}", plan.diagnostics);
        assert_eq!(plan.backend, EMBED_BACKEND_DOM);
        assert_eq!(
            plan.fixture_facts.expect("fixture facts").label,
            FIXTURE_CHROME_LABEL
        );
    }

    #[test]
    fn embedded_mount_rejects_missing_duplicate_and_unsupported_state() {
        let descriptor =
            EmbeddedRatzillaMountDescriptor::new("", MISSING_FIXTURE, ZERO_SIZE, EMBED_HEIGHT)
                .with_backend(UNSUPPORTED_BACKEND)
                .with_lifecycle(UNSUPPORTED_LIFECYCLE);
        let active = vec![embedded_mount_key("")];

        let plan = evaluate_embedded_ratzilla_mount(&descriptor, &[], &active);

        assert!(!plan.is_accepted());
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == EmbeddedRatzillaDiagnosticCode::MissingHostId)
        );
        assert!(plan.diagnostics.iter().any(
            |diagnostic| diagnostic.code == EmbeddedRatzillaDiagnosticCode::UnsupportedBackend
        ));
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == EmbeddedRatzillaDiagnosticCode::ZeroSizedHost)
        );
        assert!(
            plan.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code
                    == EmbeddedRatzillaDiagnosticCode::DuplicateMountKey)
        );
        assert!(plan.diagnostics.iter().any(
            |diagnostic| diagnostic.code == EmbeddedRatzillaDiagnosticCode::UnsupportedFixture
        ));
        assert!(
            plan.diagnostics.iter().any(|diagnostic| diagnostic.code
                == EmbeddedRatzillaDiagnosticCode::UnsupportedLifecycle)
        );
    }

    #[test]
    fn backend_neutral_profile_reports_library_and_shell_gaps() {
        let diagnostics = classify_backend_neutral_widget(&BackendNeutralWidgetProfile {
            crate_name: "rat-widgets".to_string(),
            ratatui_default_features_enabled: true,
            native_shell_opts_into_crossterm: false,
        });

        assert_eq!(diagnostics.len(), BACKEND_NEUTRAL_DIAGNOSTIC_COUNT);
    }
}
