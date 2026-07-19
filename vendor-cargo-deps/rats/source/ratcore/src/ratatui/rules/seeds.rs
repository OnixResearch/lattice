//! Baseline feature-policy rows.

/// Returns the checked-in baseline `Ratatui` feature policy rows.
#[must_use]
pub fn baseline_ratatui_feature_policy_rows() -> Vec<super::types::FeaturePolicyRow> {
    let mut rows = Vec::with_capacity(super::types::BASELINE_POLICY_ROW_COUNT);
    rows.extend(native_rows());
    rows.extend(optional_rows());
    rows.extend(experimental_rows());
    rows
}

fn native_rows() -> Vec<super::types::FeaturePolicyRow> {
    let mut rows = Vec::with_capacity(4);
    debug_assert!(rows.capacity() >= 4);
    rows.push(
        row(RowSeed {
            crate_family: super::types::APP_SHELL_FAMILY,
            feature_name: super::types::FEATURE_CROSSTERM,
            status: super::types::POLICY_APPROVED,
            owner: "rat-showcase owners",
            compatibility_scope: "native terminal app shell only",
            reason: "terminal lifecycle, raw mode, and event dispatch are app-shell side effects",
            fallback: "reusable crates stay backend-free and delegate events through pure models",
        })
        .with_evidence("subwayrat/crates/rat-showcase/Cargo.toml")
        .with_evidence("docs/rat-showcase-shell.md"),
    );
    rows.push(
        row(RowSeed {
            crate_family: super::types::BACKEND_ADAPTER_FAMILY,
            feature_name: super::types::FEATURE_CROSSTERM,
            status: super::types::POLICY_APPROVED,
            owner: "rat-keymap owners",
            compatibility_scope: "native input adapter compatibility",
            reason: "key event type compatibility currently requires Ratatui's crossterm feature",
            fallback: "keep key parsing and formatting pure outside adapter-owned event types",
        })
        .with_evidence("subwayrat/crates/rat-keymap/Cargo.toml")
        .with_evidence("docs/workspace-validation.md"),
    );
    rows.push(
        row(RowSeed {
            crate_family: super::types::REUSABLE_WIDGET_FAMILY,
            feature_name: super::types::FEATURE_CROSSTERM,
            status: super::types::POLICY_FORBIDDEN,
            owner: "workspace policy",
            compatibility_scope: "backend-free native and web-consumed widget crates",
            reason: "terminal backend features leak side effects into reusable widgets",
            fallback: "move backend access to app shells or backend adapters",
        })
        .with_next_action("Open a focused Cairn if a reusable crate needs backend ownership."),
    );
    rows.push(row(RowSeed {
        crate_family: super::types::RATZILLA_WEB_FAMILY,
        feature_name: super::types::FEATURE_CROSSTERM,
        status: super::types::POLICY_FORBIDDEN,
        owner: "rat-ratzilla owners",
        compatibility_scope: "web-consumed wasm profile",
        reason: "crossterm is a native terminal backend and is incompatible with the browser boundary",
        fallback: "use Ratzilla/browser input adapters and Ratcore route normalization",
    })
    .with_next_action("Keep `subwayrat-web` profile reporting browser-boundary diagnostics."));
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn optional_rows() -> Vec<super::types::FeaturePolicyRow> {
    let mut rows = Vec::with_capacity(4);
    debug_assert!(rows.capacity() >= 4);
    rows.push(
        row(RowSeed {
            crate_family: super::types::OPTIONAL_DATA_FAMILY,
            feature_name: super::types::FEATURE_SERDE,
            status: super::types::POLICY_APPROVED,
            owner: "crate-specific owners",
            compatibility_scope: "optional serialization feature only",
            reason: "serialization support is acceptable when a crate owns data exchange or fixtures",
            fallback: "keep serde optional and disabled from the root Ratatui default feature set",
        })
        .with_evidence("subwayrat/crates/rat-nodegraph/Cargo.toml")
        .with_evidence("subwayrat/crates/rat-widgets/Cargo.toml"),
    );
    rows.push(
        row(RowSeed {
            crate_family: super::types::THEME_DOCS_FAMILY,
            feature_name: super::types::FEATURE_PALETTE,
            status: super::types::POLICY_APPROVED,
            owner: "theme-token owners",
            compatibility_scope: "documented palette conversion only",
            reason: "palette conversion may support theme previews when evidence names the consumer",
            fallback: "fall back to checked-in theme tokens when palette conversion is absent",
        })
        .with_evidence("docs/theme-token-previews.md")
        .with_evidence("ratcore::theme_tokens"),
    );
    rows.push(
        row(RowSeed {
            crate_family: super::types::CALENDAR_WIDGET_FAMILY,
            feature_name: super::types::FEATURE_WIDGET_CALENDAR,
            status: super::types::POLICY_APPROVED,
            owner: "rat-datepicker owners",
            compatibility_scope: "calendar widget family only",
            reason: "calendar-specific feature use belongs with date picker and agenda surfaces",
            fallback: "keep date math in Ratcore calendar helpers when renderer-neutral",
        })
        .with_evidence("subwayrat/crates/rat-datepicker")
        .with_evidence("subwayrat/crates/rat-agenda"),
    );
    rows.push(
        row(RowSeed {
            crate_family: super::types::LAYOUT_EXPERIMENT_FAMILY,
            feature_name: super::types::FEATURE_LAYOUT_CACHE,
            status: super::types::POLICY_APPROVED,
            owner: "layout owners",
            compatibility_scope: "measured layout-performance experiments",
            reason: "layout caching is allowed only with a documented measurement or compatibility reason",
            fallback: "fall back to deterministic uncached layout computation",
        })
        .with_evidence("docs/renderer-parity-matrix.md")
        .with_evidence("docs/workspace-validation.md"),
    );
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn experimental_rows() -> Vec<super::types::FeaturePolicyRow> {
    let mut rows = Vec::with_capacity(2);
    debug_assert!(rows.capacity() >= 2);
    rows.push(
        row(RowSeed {
            crate_family: super::types::PRIVATE_EXPERIMENT_FAMILY,
            feature_name: super::types::FEATURE_UNSTABLE,
            status: super::types::POLICY_APPROVED,
            owner: "experiment owner named by future Cairn",
            compatibility_scope: "private experiments only; not public reusable crates",
            reason: "unstable APIs may be explored privately when callers do not depend on them",
            fallback: "public crates require a follow-up Cairn before exposing unstable behavior",
        })
        .with_evidence("docs/ratatui-coverage.md")
        .with_evidence("cairn/changes/define-ratatui-feature-policy"),
    );
    rows.push(row(RowSeed {
        crate_family: super::types::WORKSPACE_POLICY_FAMILY,
        feature_name: super::types::FEATURE_MACROS,
        status: super::types::POLICY_REQUIRES_CHANGE,
        owner: "workspace policy",
        compatibility_scope: "unapproved public API surface",
        reason: "macros add implicit API surface and need an owner decision before adoption",
        fallback: "write explicit builders or local helper functions until a Cairn approves macros",
    })
    .with_next_action("Approve macro use through a focused Cairn with owner and tests."));
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

struct RowSeed {
    crate_family: &'static str,
    feature_name: &'static str,
    status: &'static str,
    owner: &'static str,
    compatibility_scope: &'static str,
    reason: &'static str,
    fallback: &'static str,
}

fn row(seed: RowSeed) -> super::types::FeaturePolicyRow {
    super::types::FeaturePolicyRow::new(super::types::FeaturePolicyRowParts {
        crate_family: seed.crate_family.to_string(),
        feature_name: seed.feature_name.to_string(),
        status: seed.status.to_string(),
        owner: seed.owner.to_string(),
        compatibility_scope: seed.compatibility_scope.to_string(),
        reason: seed.reason.to_string(),
        fallback: seed.fallback.to_string(),
    })
}
