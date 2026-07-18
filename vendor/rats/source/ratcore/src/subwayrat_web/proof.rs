const HASH_SEPARATOR: &str = "|";

/// Web-renderer check kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebRendererCheckKind {
    /// Pure/shared wasm support.
    CoreWasm,
    /// Reusable Subwayrat widget wasm support.
    WidgetWasm,
    /// Native crossterm shell support.
    NativeShowcase,
    /// Ratzilla web showcase typecheck support.
    WebShowcase,
    /// Linked Ratzilla wasm artifact support.
    WebShowcaseLink,
    /// Browser bundle asset support.
    BrowserBundle,
}

/// Check status for web-renderer evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebRendererCheckStatus {
    /// Check passed.
    Passed,
    /// Check failed.
    Failed,
    /// Path is intentionally unsupported.
    Unsupported,
}

/// Focused validation evidence row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebRendererCheck {
    /// Check kind.
    pub kind: WebRendererCheckKind,
    /// Package or fixture identifier.
    pub package: String,
    /// Exact command or smoke label.
    pub command: String,
    /// Check status.
    pub status: WebRendererCheckStatus,
    /// Blocker or unsupported reason.
    pub blocker: Option<String>,
}

impl WebRendererCheck {
    /// Creates a validation evidence row.
    #[must_use]
    pub fn new(
        kind: WebRendererCheckKind,
        package: impl Into<String>,
        command: impl Into<String>,
        status: WebRendererCheckStatus,
    ) -> Self {
        Self {
            kind,
            package: package.into(),
            command: command.into(),
            status,
            blocker: None,
        }
    }

    /// Adds blocker or unsupported reason text.
    #[must_use]
    pub fn with_blocker(mut self, blocker: impl Into<String>) -> Self {
        self.blocker = Some(blocker.into());
        self
    }
}

/// Subwayrat web renderer evidence bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubwayratWebRendererEvidence {
    /// Focused checks.
    pub checks: Vec<WebRendererCheck>,
    /// Rendered widget or fixture identifiers.
    pub rendered_widget_ids: Vec<String>,
    /// Native-only examples or paths that remain unsupported in the browser.
    pub unsupported_native_paths: Vec<String>,
    /// Deterministic BLAKE3 evidence hash.
    pub evidence_hash: String,
}

/// Builds deterministic web-renderer evidence and hashes it with BLAKE3.
#[must_use]
pub fn build_subwayrat_web_renderer_evidence(
    mut checks: Vec<WebRendererCheck>,
    mut rendered_widget_ids: Vec<String>,
    mut unsupported_native_paths: Vec<String>,
) -> SubwayratWebRendererEvidence {
    checks.sort_by(|left, right| {
        left.package
            .cmp(&right.package)
            .then(left.command.cmp(&right.command))
    });
    rendered_widget_ids.sort();
    unsupported_native_paths.sort();
    let evidence_hash = hash_evidence(&checks, &rendered_widget_ids, &unsupported_native_paths);
    SubwayratWebRendererEvidence {
        checks,
        rendered_widget_ids,
        unsupported_native_paths,
        evidence_hash,
    }
}

fn hash_evidence(
    checks: &[WebRendererCheck],
    widget_ids: &[String],
    unsupported_paths: &[String],
) -> String {
    let mut material = String::new();
    for check in checks {
        material.push_str(&format!(
            "{:?}{HASH_SEPARATOR}{}{HASH_SEPARATOR}{}{HASH_SEPARATOR}{:?}{HASH_SEPARATOR}{:?}\n",
            check.kind, check.package, check.command, check.status, check.blocker
        ));
    }
    for widget_id in widget_ids {
        material.push_str(widget_id);
        material.push('\n');
    }
    for unsupported_path in unsupported_paths {
        material.push_str(unsupported_path);
        material.push('\n');
    }
    blake3::hash(material.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_distinguishes_baseline_web_and_unsupported_paths() {
        let evidence = build_subwayrat_web_renderer_evidence(
            vec![
                WebRendererCheck::new(
                    WebRendererCheckKind::CoreWasm,
                    "ratcore",
                    "cargo check --target wasm32-unknown-unknown -p ratcore",
                    WebRendererCheckStatus::Passed,
                ),
                WebRendererCheck::new(
                    WebRendererCheckKind::WidgetWasm,
                    "rat-widgets",
                    "cargo check --target wasm32-unknown-unknown -p rat-widgets",
                    WebRendererCheckStatus::Failed,
                )
                .with_blocker("crossterm backend selected by default features"),
            ],
            vec!["spinner.basic".to_string(), "tree.basic".to_string()],
            vec!["native.raw-mode".to_string()],
        );

        assert!(
            evidence
                .checks
                .iter()
                .any(|check| check.status == WebRendererCheckStatus::Failed)
        );
        assert_eq!(evidence.rendered_widget_ids[0], "spinner.basic");
        assert!(!evidence.evidence_hash.is_empty());
    }
}
