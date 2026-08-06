//! Baseline `Ratatui` coverage rows.

const CORE_ROW_COUNT: usize = 4;
const VISUAL_BUILTIN_ROW_COUNT: usize = 4;
const STRUCTURAL_BUILTIN_ROW_COUNT: usize = 3;
const BUILTIN_ROW_COUNT: usize = 7;
const RENDERER_BOUNDARY_ROW_COUNT: usize = 4;
const RENDERER_ROW_COUNT: usize = 11;
const POLICY_ROW_COUNT: usize = 2;

pub(super) fn rows() -> Vec<super::CoverageRow> {
    let mut rows = Vec::with_capacity(super::BASELINE_ROW_COUNT);
    rows.extend(core_rows());
    rows.extend(renderer_rows());
    rows.extend(policy_rows());
    debug_assert_eq!(rows.len(), super::BASELINE_ROW_COUNT);
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn core_rows() -> Vec<super::CoverageRow> {
    let mut rows = Vec::with_capacity(CORE_ROW_COUNT);
    debug_assert!(rows.capacity() >= CORE_ROW_COUNT);
    rows.push(
        super::CoverageRow::new(
            "ratatui.buffer",
            "Buffer, Cell, and area drawing contracts",
            "Subwayrat widget crates",
            super::STATUS_PASS_THROUGH,
            "Keep buffer use renderer-owned and avoid moving buffers into Ratcore.",
        )
        .with_related_surface("Ratcore keeps only renderer-neutral projections.")
        .with_evidence("subwayrat/crates/rat-widgets")
        .with_evidence("docs/renderer-parity-matrix.md"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.layout",
            "Rect, Layout, Constraint, Flex, Margin, Position, and Size",
            "rat-showcase and Subwayrat widget crates",
            super::STATUS_PASS_THROUGH,
            "Add primitive examples before extracting new shared geometry helpers.",
        )
        .with_related_surface("Ratcore may own pure geometry helpers only when renderer-free.")
        .with_evidence("docs/renderer-parity-matrix.md")
        .with_evidence("subwayrat/crates/rat-showcase/src/main.rs"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.style-text-symbols",
            "Style, Color, Text, Line, Span, Masked, and symbol sets",
            "rat-showcase and theme docs",
            super::STATUS_PASS_THROUGH,
            "Publish primitive coverage rows for edge cases and fallback decisions.",
        )
        .with_related_surface("Theme token projection stays in Ratcore; Ratatui styling stays TUI-owned.")
        .with_evidence("docs/theme-token-previews.md")
        .with_evidence("subwayrat/crates/rat-showcase/src/main.rs"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.custom-widgets",
            "Widget and StatefulWidget implementations owned by Subwayrat",
            "Subwayrat widget crates",
            super::STATUS_WRAPPED,
            "Adopt reference-based widget impls where state ownership permits it.",
        )
        .with_related_surface("Compatibility render methods remain imperative shells.")
        .with_evidence("subwayrat/crates/rat-widgets")
        .with_evidence("subwayrat/crates/rat-chrome"),
    );
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn renderer_rows() -> Vec<super::CoverageRow> {
    let mut rows = Vec::with_capacity(RENDERER_ROW_COUNT);
    rows.extend(builtin_rows());
    rows.extend(renderer_boundary_rows());
    debug_assert_eq!(rows.len(), RENDERER_ROW_COUNT);
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn builtin_rows() -> Vec<super::CoverageRow> {
    let mut rows = Vec::with_capacity(BUILTIN_ROW_COUNT);
    rows.extend(visual_builtin_rows());
    rows.extend(structural_builtin_rows());
    debug_assert_eq!(rows.len(), BUILTIN_ROW_COUNT);
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn visual_builtin_rows() -> Vec<super::CoverageRow> {
    let mut rows = Vec::with_capacity(VISUAL_BUILTIN_ROW_COUNT);
    debug_assert!(rows.capacity() >= VISUAL_BUILTIN_ROW_COUNT);
    rows.push(
        super::CoverageRow::new(
            "ratatui.builtin-data-viz",
            "BarChart, Chart, and Sparkline built-ins",
            "rat-showcase coverage matrix",
            super::STATUS_SHOWCASE_ONLY,
            "Add domain chart wrappers only when shared data-view state is accepted.",
        )
        .with_related_surface("Rats documents direct Ratatui use instead of duplicating charts.")
        .with_evidence("docs/ratatui-coverage.md"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.builtin-canvas",
            "Canvas drawing, shapes, and map helpers",
            "rat-showcase coverage matrix",
            super::STATUS_SHOWCASE_ONLY,
            "Keep canvas demos pass-through until a renderer-neutral drawing model is accepted.",
        )
        .with_related_surface("Ratcore may own pure drawing intent only through a future change.")
        .with_evidence("docs/ratatui-coverage.md"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.builtin-calendar",
            "calendar::Monthly date display",
            "rat-showcase coverage matrix",
            super::STATUS_SHOWCASE_ONLY,
            "Keep Subwayrat datepicker behavior separate from direct Monthly demos.",
        )
        .with_related_surface("rat-datepicker remains the value-added interactive calendar.")
        .with_evidence("docs/ratatui-coverage.md")
        .with_evidence("subwayrat/crates/rat-showcase/src/main.rs"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.builtin-progress-scroll",
            "Gauge, LineGauge, Scrollbar, and Sparkline progress indicators",
            "rat-showcase coverage matrix",
            super::STATUS_SHOWCASE_ONLY,
            "Wrap only when shared progress or scroll state exceeds direct Ratatui behavior.",
        )
        .with_related_surface("rat-widgets owns value-added progress and loader components.")
        .with_evidence("docs/ratatui-coverage.md"),
    );
    debug_assert_eq!(rows.len(), VISUAL_BUILTIN_ROW_COUNT);
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn structural_builtin_rows() -> Vec<super::CoverageRow> {
    let mut rows = Vec::with_capacity(STRUCTURAL_BUILTIN_ROW_COUNT);
    debug_assert!(rows.capacity() >= STRUCTURAL_BUILTIN_ROW_COUNT);
    rows.push(
        super::CoverageRow::new(
            "ratatui.builtin-decoration",
            "Block, Clear, Fill, borders, and decoration primitives",
            "rat-showcase and Subwayrat widget crates",
            super::STATUS_PASS_THROUGH,
            "Keep decoration direct unless a chrome wrapper adds shared behavior.",
        )
        .with_related_surface("rat-chrome owns value-added chrome; simple decoration stays direct.")
        .with_evidence("docs/ratatui-coverage.md")
        .with_evidence("subwayrat/crates/rat-showcase/src/main.rs"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.builtin-collections",
            "List, Table, Tabs, and comparison collection widgets",
            "rat-showcase and Subwayrat widget crates",
            super::STATUS_PASS_THROUGH,
            "Keep direct built-ins as comparison points beside value-added Subwayrat widgets.",
        )
        .with_related_surface("rat-table, rat-tree, and tab shell own app-specific behavior.")
        .with_evidence("docs/ratatui-coverage.md")
        .with_evidence("subwayrat/crates/rat-showcase/src/main.rs"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.builtin-branding",
            "RatatuiLogo and RatatuiMascot branding widgets",
            "rat-showcase coverage matrix",
            super::STATUS_SHOWCASE_ONLY,
            "Use branding widgets for demos only; do not make them product dependencies.",
        )
        .with_related_surface("Branding remains showcase/demo content, not Ratcore state.")
        .with_evidence("docs/ratatui-coverage.md"),
    );
    debug_assert_eq!(rows.len(), STRUCTURAL_BUILTIN_ROW_COUNT);
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn renderer_boundary_rows() -> Vec<super::CoverageRow> {
    let mut rows = Vec::with_capacity(RENDERER_BOUNDARY_ROW_COUNT);
    debug_assert!(rows.capacity() >= RENDERER_BOUNDARY_ROW_COUNT);
    rows.push(
        super::CoverageRow::new(
            "ratatui.terminal-lifecycle",
            "Terminal init, restore, draw loop, resize, and cleanup helpers",
            "rat-showcase app shell",
            super::STATUS_RENDERER_OWNED,
            "Evaluate current Ratatui lifecycle helpers before changing terminal setup.",
        )
        .with_related_surface("Ratcore never owns terminal lifecycle or backend handles.")
        .with_evidence("subwayrat/crates/rat-showcase/src/terminal.rs")
        .with_evidence("docs/rat-showcase-shell.md"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.backends-crossterm",
            "Crossterm backend and event integration",
            "app shells and backend adapters",
            super::STATUS_RENDERER_OWNED,
            "Keep backend feature ownership explicit in app shells and adapters.",
        )
        .with_related_surface("Subwayrat reusable crates should remain backend-free.")
        .with_evidence("subwayrat/crates/rat-showcase/Cargo.toml")
        .with_evidence("docs/workspace-validation.md"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.ratzilla-web",
            "Ratzilla browser renderer and web input boundary",
            "rat-ratzilla",
            super::STATUS_RENDERER_OWNED,
            "Keep web smoke evidence fresh as Ratzilla exposes more event classes.",
        )
        .with_related_surface("Meteorite owns the separate Dioxus web renderer.")
        .with_evidence("docs/subwayrat-web-renderer.md")
        .with_evidence("docs/renderer-parity-matrix.md"),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.effects-tachyonfx",
            "Terminal-cell effects after Ratatui rendering",
            "rat-showcase app shell",
            super::STATUS_RENDERER_OWNED,
            "Keep TachyonFX scoped to app shells/adapters; open a parity change before moving animation semantics into shared core.",
        )
        .with_related_surface(
            "Ratcore owns only shared animation intent; Meteorite projects shared IDs through CSS/Dioxus instead of TachyonFX.",
        )
        .with_evidence("docs/animation-parity.md")
        .with_evidence("subwayrat/crates/rat-showcase/src/effects.rs"),
    );
    debug_assert!(rows.len() <= rows.capacity());
    rows
}

fn policy_rows() -> Vec<super::CoverageRow> {
    let mut rows = Vec::with_capacity(POLICY_ROW_COUNT);
    debug_assert!(rows.capacity() >= POLICY_ROW_COUNT);
    rows.push(
        super::CoverageRow::new(
            "ratatui.macros",
            "Ratatui macros and optional convenience APIs",
            "workspace policy",
            super::STATUS_UNSUPPORTED,
            "Approve macro use through a focused feature-policy change if a crate needs it.",
        )
        .with_related_surface("No current Rats public API requires Ratatui macros.")
        .with_reason("No accepted workspace owner or validation evidence yet."),
    );
    rows.push(
        super::CoverageRow::new(
            "ratatui.feature-flags",
            "Optional backend, data, palette, performance, scrolling, and unstable features",
            "workspace policy",
            super::STATUS_NEEDS_EVIDENCE,
            "Define feature policy rows and validation before adding new Ratatui features.",
        )
        .with_related_surface("Feature use must name owner, reason, compatibility, and fallback."),
    );
    debug_assert!(rows.len() <= rows.capacity());
    rows
}
