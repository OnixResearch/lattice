#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-tui-completion nix shell "github:nix-community/fenix?rev=7b33c6466f781cd699fe250c5b69dc4193da67a7#minimal.toolchain" nixpkgs#gcc nixpkgs#clang -c cargo -q -Zscript
---cargo
[package]
name = "build-tui-ux-completion-fixture"
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

const EXPECTED_ROW_COUNT: usize = 98;
const EXPECTED_PARTIAL_COUNT: usize = 17;
const BLAKE3_HEX_LENGTH: usize = 64;
const SUBSYSTEM: &str = "tui-ux";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const AUDIT_PATH: &str = "tests/fixtures/traceability-tui-ux-audit-reviewed.json";
const OUTPUT_PATH: &str = "tests/fixtures/traceability-tui-ux-completion-reviewed.json";

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
const COMPLETION: &str = "tests/test_tui_ux_completion.rs";

const SNAPSHOTS: &[(&str, &str)] = &[
    (APP, "tests/fixtures/traceability-tui-ux-completion-reviewed-src-tui-app.rs.txt"),
    (INPUT, "tests/fixtures/traceability-tui-ux-completion-reviewed-src-tui-input.rs.txt"),
    (PANELS, "tests/fixtures/traceability-tui-ux-completion-reviewed-src-tui-panels.rs.txt"),
    (MAIN, "tests/fixtures/traceability-tui-ux-completion-reviewed-src-main.rs.txt"),
    (CONTROL, "tests/fixtures/traceability-tui-ux-completion-reviewed-test-control-flow.rs.txt"),
    (CREDENTIALS, "tests/fixtures/traceability-tui-ux-completion-reviewed-test-credentials.rs.txt"),
    (NODE_KIND, "tests/fixtures/traceability-tui-ux-completion-reviewed-test-node-kind-provider.rs.txt"),
    (REPLAY, "tests/fixtures/traceability-tui-ux-completion-reviewed-test-replay.rs.txt"),
    (RUN_HISTORY, "tests/fixtures/traceability-tui-ux-completion-reviewed-test-run-history-details.rs.txt"),
    (TRIGGERS, "tests/fixtures/traceability-tui-ux-completion-reviewed-test-triggers.rs.txt"),
    (WEBHOOKS, "tests/fixtures/traceability-tui-ux-completion-reviewed-webhooks.rs.txt"),
    (COMPLETION, "tests/fixtures/traceability-tui-ux-completion-reviewed-test-tui-ux-completion.rs.txt"),
];

const COMPLETION_TEST_IDS: &[&str] = &[
    "n8nclone.tui_ux.background_runs.long_running",
    "n8nclone.tui_ux.background_runs.responsive",
    "n8nclone.tui_ux.image_nodes.preview",
    "n8nclone.tui_ux.image_nodes.preview.canvas",
    "n8nclone.tui_ux.keyboard_first",
    "n8nclone.tui_ux.replay_debugger.active_run.stream",
    "n8nclone.tui_ux.workspace",
    "n8nclone.tui_ux.workspace.starts",
];

#[derive(Debug, Clone, Deserialize)]
struct AuditReport {
    rows: Vec<AuditRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct AuditRow {
    requirement_id: String,
    subsystem: String,
    classification: String,
    rationale_code: String,
    evidence: Vec<AuditEvidence>,
}

#[derive(Debug, Clone, Deserialize)]
struct AuditEvidence {
    path: String,
    blake3: String,
}

#[derive(Serialize)]
struct CompletionReport {
    rows: Vec<CompletionRow>,
}

#[derive(Serialize)]
struct CompletionRow {
    requirement_id: String,
    subsystem: &'static str,
    classification: &'static str,
    rationale_code: &'static str,
    evidence: Vec<CompletionEvidence>,
}

#[derive(Serialize)]
struct CompletionEvidence {
    path: String,
    blake3: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("TUI completion fixture failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let audit: AuditReport = serde_json::from_slice(&fs::read(AUDIT_PATH).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    validate_audit(&audit)?;
    let hashes = capture_snapshots()?;
    let completion_test_ids = COMPLETION_TEST_IDS.iter().copied().collect::<BTreeSet<_>>();
    if completion_test_ids.len() != COMPLETION_TEST_IDS.len() {
        return Err("completion-test ID list contains a duplicate".to_string());
    }

    let mut rows = Vec::with_capacity(audit.rows.len());
    for audit_row in audit.rows {
        let was_partial = audit_row.classification == PARTIAL;
        let mut paths = audit_row.evidence.into_iter().map(|evidence| evidence.path).collect::<Vec<_>>();
        if completion_test_ids.contains(audit_row.requirement_id.as_str()) {
            paths.push(COMPLETION.to_string());
        }
        paths.sort();
        paths.dedup();
        let evidence = paths
            .into_iter()
            .map(|path| {
                Ok(CompletionEvidence {
                    blake3: hashes.get(path.as_str()).ok_or_else(|| format!("missing source hash for {path}"))?.clone(),
                    path,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        rows.push(CompletionRow {
            requirement_id: audit_row.requirement_id,
            subsystem: SUBSYSTEM,
            classification: EXACT,
            rationale_code: if was_partial {
                "assertions.completed-maintained-tui-evidence"
            } else {
                "assertions.direct-maintained-tui-evidence"
            },
            evidence,
        });
    }
    let bytes = serde_json::to_vec_pretty(&CompletionReport { rows }).map_err(|error| error.to_string())?;
    fs::write(OUTPUT_PATH, [bytes.as_slice(), b"\n"].concat()).map_err(|error| error.to_string())?;
    Ok(())
}

fn validate_audit(audit: &AuditReport) -> Result<(), String> {
    if audit.rows.len() != EXPECTED_ROW_COUNT {
        return Err(format!("expected {EXPECTED_ROW_COUNT} audit rows"));
    }
    let mut ids = BTreeSet::new();
    let mut partial_count = 0;
    for row in &audit.rows {
        if row.subsystem != SUBSYSTEM || !ids.insert(row.requirement_id.as_str()) {
            return Err("audit rows contain wrong subsystem or duplicate ID".to_string());
        }
        if row.classification == PARTIAL {
            partial_count += 1;
            if !row.rationale_code.starts_with("gap.") {
                return Err("partial audit rationale is not gap-bounded".to_string());
            }
        } else if row.classification != EXACT {
            return Err("audit row has unsupported classification".to_string());
        }
        if row.evidence.is_empty()
            || row.evidence.iter().any(|evidence| evidence.path.is_empty() || evidence.blake3.len() != BLAKE3_HEX_LENGTH)
        {
            return Err("audit row has incomplete evidence".to_string());
        }
    }
    if partial_count != EXPECTED_PARTIAL_COUNT {
        return Err(format!("expected {EXPECTED_PARTIAL_COUNT} partial rows"));
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
