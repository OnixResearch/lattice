#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-tui-audit nix shell "github:nix-community/fenix?rev=7b33c6466f781cd699fe250c5b69dc4193da67a7#minimal.toolchain" nixpkgs#gcc nixpkgs#clang -c cargo -q -Zscript
---cargo
[package]
name = "build-tui-ux-audit-fixture"
version = "0.1.0"
edition = "2024"

[dependencies]
blake3 = "1.8.3"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
---

use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

const EXPECTED_MISSING_COUNT: usize = 98;
const TUI_PREFIX: &str = "n8nclone.tui_ux.";
const SUBSYSTEM: &str = "tui-ux";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const COVERAGE_PATH: &str = "tests/fixtures/traceability-workflow-execution-promotion-current.json";
const REVIEWED_PATH: &str = "tests/fixtures/traceability-tui-ux-audit-reviewed.json";
const IDS_PATH: &str = "tests/traceability_tui_ux_ids.rs";

const APP: &str = "src/tui/app.rs";
const INPUT: &str = "src/tui/input.rs";
const PANELS: &str = "src/tui/panels.rs";
const MAIN: &str = "src/main.rs";
const CONTROL: &str = "tests/test_control_flow.rs";
const CREDENTIALS: &str = "tests/test_credentials.rs";
const NODE_KIND: &str = "tests/test_node_kind_provider.rs";
const REPLAY: &str = "tests/test_replay.rs";
const RUN_HISTORY: &str = "tests/test_run_history_details.rs";
const TRIGGERS: &str = "tests/test_triggers.rs";
const WEBHOOKS: &str = "tests/webhooks.rs";

const SNAPSHOTS: &[(&str, &str)] = &[
    (APP, "tests/fixtures/traceability-tui-ux-audit-reviewed-src-tui-app.rs.txt"),
    (INPUT, "tests/fixtures/traceability-tui-ux-audit-reviewed-src-tui-input.rs.txt"),
    (PANELS, "tests/fixtures/traceability-tui-ux-audit-reviewed-src-tui-panels.rs.txt"),
    (MAIN, "tests/fixtures/traceability-tui-ux-audit-reviewed-src-main.rs.txt"),
    (CONTROL, "tests/fixtures/traceability-tui-ux-audit-reviewed-test-control-flow.rs.txt"),
    (CREDENTIALS, "tests/fixtures/traceability-tui-ux-audit-reviewed-test-credentials.rs.txt"),
    (NODE_KIND, "tests/fixtures/traceability-tui-ux-audit-reviewed-test-node-kind-provider.rs.txt"),
    (REPLAY, "tests/fixtures/traceability-tui-ux-audit-reviewed-test-replay.rs.txt"),
    (RUN_HISTORY, "tests/fixtures/traceability-tui-ux-audit-reviewed-test-run-history-details.rs.txt"),
    (TRIGGERS, "tests/fixtures/traceability-tui-ux-audit-reviewed-test-triggers.rs.txt"),
    (WEBHOOKS, "tests/fixtures/traceability-tui-ux-audit-reviewed-webhooks.rs.txt"),
];

const PARTIAL_IDS: &[&str] = &[
    "n8nclone.tui_ux.agent_nodes.errors.invalid_target",
    "n8nclone.tui_ux.background_runs.long_running",
    "n8nclone.tui_ux.background_runs.responsive",
    "n8nclone.tui_ux.image_nodes.preview",
    "n8nclone.tui_ux.image_nodes.preview.canvas",
    "n8nclone.tui_ux.keyboard_first",
    "n8nclone.tui_ux.replay_debugger.active_run.stream",
    "n8nclone.tui_ux.replay_debugger.timeline.open",
    "n8nclone.tui_ux.run_history.details.empty_error.storage",
    "n8nclone.tui_ux.run_history.node_outputs.inspect",
    "n8nclone.tui_ux.run_history.node_outputs.missing",
    "n8nclone.tui_ux.run_history.rerun.unavailable",
    "n8nclone.tui_ux.run_history.timeline.show",
    "n8nclone.tui_ux.triggers.manage.error",
    "n8nclone.tui_ux.webhook_listener.cli.start",
    "n8nclone.tui_ux.workspace",
    "n8nclone.tui_ux.workspace.starts",
];

#[derive(Deserialize)]
struct Coverage {
    missing: Vec<String>,
    dangling: Vec<String>,
    valid: bool,
}

#[derive(Serialize)]
struct Report {
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    requirement_id: String,
    subsystem: &'static str,
    classification: &'static str,
    rationale_code: &'static str,
    evidence: Vec<Evidence>,
}

#[derive(Serialize)]
struct Evidence {
    path: &'static str,
    blake3: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("TUI audit fixture failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let coverage: Coverage = serde_json::from_slice(&fs::read(COVERAGE_PATH).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    validate_coverage(&coverage)?;
    let source_hashes = capture_snapshots()?;
    let partial = PARTIAL_IDS.iter().copied().collect::<BTreeSet<_>>();
    if partial.len() != PARTIAL_IDS.len() {
        return Err("partial ID list contains a duplicate".to_string());
    }
    if !partial.iter().all(|identifier| coverage.missing.iter().any(|missing| missing == identifier)) {
        return Err("partial ID is absent from coverage".to_string());
    }

    let rows = coverage
        .missing
        .iter()
        .map(|identifier| row(identifier, &partial, &source_hashes))
        .collect::<Result<Vec<_>, _>>()?;
    let report = serde_json::to_vec_pretty(&Report { rows }).map_err(|error| error.to_string())?;
    fs::write(REVIEWED_PATH, [report.as_slice(), b"\n"].concat()).map_err(|error| error.to_string())?;
    fs::write(IDS_PATH, render_ids(&coverage.missing)).map_err(|error| error.to_string())?;
    Ok(())
}

fn validate_coverage(coverage: &Coverage) -> Result<(), String> {
    if coverage.valid || !coverage.dangling.is_empty() {
        return Err("coverage must remain invalid with no dangling markers".to_string());
    }
    if coverage.missing.len() != EXPECTED_MISSING_COUNT {
        return Err(format!("expected {EXPECTED_MISSING_COUNT} missing IDs"));
    }
    if !coverage.missing.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err("missing IDs must be uniquely sorted".to_string());
    }
    if !coverage.missing.iter().all(|identifier| identifier.starts_with(TUI_PREFIX)) {
        return Err("audit scope contains a non-TUI ID".to_string());
    }
    Ok(())
}

fn capture_snapshots() -> Result<BTreeMap<&'static str, String>, String> {
    let mut hashes = BTreeMap::new();
    for (source, snapshot) in SNAPSHOTS {
        let bytes = fs::read(source).map_err(|error| format!("{source}: {error}"))?;
        if Path::new(snapshot).exists() {
            let existing = fs::read(snapshot).map_err(|error| format!("{snapshot}: {error}"))?;
            if existing != bytes {
                return Err(format!("immutable snapshot differs from current source: {snapshot}"));
            }
        } else {
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(snapshot)
                .map_err(|error| format!("{snapshot}: {error}"))?;
            output.write_all(&bytes).map_err(|error| format!("{snapshot}: {error}"))?;
        }
        hashes.insert(*source, blake3::hash(&bytes).to_hex().to_string());
    }
    Ok(hashes)
}

fn row(
    identifier: &str,
    partial: &BTreeSet<&str>,
    hashes: &BTreeMap<&'static str, String>,
) -> Result<Row, String> {
    let paths = evidence_paths(identifier)?;
    let evidence = paths
        .iter()
        .map(|path| {
            Ok(Evidence {
                path,
                blake3: hashes.get(path).ok_or_else(|| format!("missing source hash for {path}"))?.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let is_partial = partial.contains(identifier);
    Ok(Row {
        requirement_id: identifier.to_string(),
        subsystem: SUBSYSTEM,
        classification: if is_partial { PARTIAL } else { EXACT },
        rationale_code: if is_partial {
            partial_rationale(identifier)?
        } else {
            "assertions.direct-maintained-tui-evidence"
        },
        evidence,
    })
}

fn evidence_paths(identifier: &str) -> Result<&'static [&'static str], String> {
    if identifier.starts_with("n8nclone.tui_ux.add_node.") || identifier.starts_with("n8nclone.tui_ux.agent_nodes.") {
        return Ok(&[APP]);
    }
    if identifier.starts_with("n8nclone.tui_ux.background_runs.") || identifier.starts_with("n8nclone.tui_ux.unsaved.") {
        return Ok(&[APP, PANELS]);
    }
    if identifier.starts_with("n8nclone.tui_ux.command_palette.") {
        return Ok(&[INPUT]);
    }
    if identifier == "n8nclone.tui_ux.control_flow.properties" {
        return Ok(&[CONTROL]);
    }
    if identifier == "n8nclone.tui_ux.credentials.providers.edit" {
        return Ok(&[APP, INPUT]);
    }
    if identifier == "n8nclone.tui_ux.credentials.providers.test" {
        return Ok(&[CREDENTIALS]);
    }
    if identifier.starts_with("n8nclone.tui_ux.empty_state.") || identifier.starts_with("n8nclone.tui_ux.help_footer.") {
        return Ok(&[PANELS]);
    }
    if identifier.starts_with("n8nclone.tui_ux.image_nodes.") {
        return Ok(&[APP, PANELS]);
    }
    if identifier.starts_with("n8nclone.tui_ux.keyboard_first") {
        return Ok(&[INPUT, PANELS]);
    }
    if identifier.starts_with("n8nclone.tui_ux.mouse_wiring.") {
        return Ok(&[INPUT]);
    }
    if identifier == "n8nclone.tui_ux.node_kind_provider" {
        return Ok(&[NODE_KIND, PANELS]);
    }
    if identifier.starts_with("n8nclone.tui_ux.properties.") {
        return Ok(&[APP, INPUT, PANELS]);
    }
    if identifier.starts_with("n8nclone.tui_ux.replay_debugger.") {
        return Ok(&[APP, PANELS, REPLAY]);
    }
    if identifier.starts_with("n8nclone.tui_ux.run_history.") {
        return Ok(&[APP, PANELS, RUN_HISTORY]);
    }
    if identifier.starts_with("n8nclone.tui_ux.triggers.") {
        return Ok(&[APP, TRIGGERS]);
    }
    if identifier == "n8nclone.tui_ux.webhook_listener.cli.start" {
        return Ok(&[MAIN, WEBHOOKS]);
    }
    if identifier.starts_with("n8nclone.tui_ux.workflow_browser.") || identifier.starts_with("n8nclone.tui_ux.workspace") {
        return Ok(&[APP, PANELS]);
    }
    Err(format!("unmapped TUI requirement: {identifier}"))
}

fn partial_rationale(identifier: &str) -> Result<&'static str, String> {
    match identifier {
        "n8nclone.tui_ux.workspace" | "n8nclone.tui_ux.workspace.starts" => {
            Ok("gap.no-terminal-startup-layout-harness")
        }
        "n8nclone.tui_ux.keyboard_first" => Ok("gap.core-keyboard-surface-not-exhaustively-asserted"),
        "n8nclone.tui_ux.background_runs.long_running" | "n8nclone.tui_ux.background_runs.responsive" => {
            Ok("gap.active-render-and-input-loop-unasserted")
        }
        "n8nclone.tui_ux.image_nodes.preview" | "n8nclone.tui_ux.image_nodes.preview.canvas" => {
            Ok("gap.canvas-image-fallback-not-implemented")
        }
        "n8nclone.tui_ux.agent_nodes.errors.invalid_target" => Ok("gap.tui-diagnostic-display-unasserted"),
        "n8nclone.tui_ux.replay_debugger.active_run.stream" => Ok("gap.events-persist-only-after-terminal-state"),
        "n8nclone.tui_ux.replay_debugger.timeline.open" => Ok("gap.historical-replay-open-not-asserted"),
        "n8nclone.tui_ux.run_history.details.empty_error.storage" => Ok("gap.storage-error-shell-not-injectable"),
        "n8nclone.tui_ux.run_history.node_outputs.inspect" => Ok("gap.rendered-node-output-details-unasserted"),
        "n8nclone.tui_ux.run_history.node_outputs.missing" => Ok("gap.selected-node-empty-output-message-unasserted"),
        "n8nclone.tui_ux.run_history.rerun.unavailable" => Ok("gap.unavailable-rerun-path-unasserted"),
        "n8nclone.tui_ux.run_history.timeline.show" => Ok("gap.rendered-timeline-unasserted"),
        "n8nclone.tui_ux.triggers.manage.error" => Ok("gap.recent-trigger-error-not-modeled"),
        "n8nclone.tui_ux.webhook_listener.cli.start" => Ok("gap.start-status-output-unasserted"),
        _ => Err(format!("missing partial rationale: {identifier}")),
    }
}

fn render_ids(ids: &[String]) -> String {
    let mut output = String::from("pub const IDS: &[&str] = &[\n");
    for identifier in ids {
        let suffix = identifier.strip_prefix("n8n").expect("validated TUI identifier starts with n8n");
        output.push_str(&format!("    concat!(\"n8n\", \"{suffix}\"),\n"));
    }
    output.push_str("];\n");
    output
}
