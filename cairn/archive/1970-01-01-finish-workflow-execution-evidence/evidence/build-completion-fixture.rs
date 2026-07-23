#!/usr/bin/env -S nix develop . -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.3"
serde_json = "=1.0.149"
---

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde_json::Value;

const EXECUTE_ARG: &str = "--execute";
const SELF_TEST_ARG: &str = "--self-test";
const FAILURE_EXIT_CODE: u8 = 1;
const SELF_TEST_ARG_COUNT: usize = 2;
const EXECUTE_ARG_COUNT: usize = 3;
const COVERAGE_ARG_INDEX: usize = 2;
const EXPECTED_CHANGED_ROWS: usize = 4;
const EXPECTED_SELECTED_ROWS: usize = 83;
const AUDIT_PATH: &str = "tests/fixtures/traceability-workflow-execution-audit-reviewed.json";
const OUTPUT_PATH: &str = "tests/fixtures/traceability-workflow-execution-completion-reviewed.json";
const COVERAGE_OUTPUT_PATH: &str = "tests/fixtures/traceability-workflow-execution-completion-current.json";
const RATS_SOURCE_PATH: &str = "tests/test_rats_source_closure.rs";
const RATS_SNAPSHOT_PATH: &str =
    "tests/fixtures/traceability-workflow-execution-completion-reviewed-rats-source-closure.rs.txt";
const SUBWORKFLOW_SOURCE_PATH: &str = "tests/test_subworkflow_execution.rs";
const SUBWORKFLOW_SNAPSHOT_PATH: &str =
    "tests/fixtures/traceability-workflow-execution-completion-reviewed-subworkflow-execution.rs.txt";
const EXACT: &str = "exact_maintained_evidence";
const RATIONALE: &str = "assertions.completed-maintained-boundary";
const SUBSYSTEM: &str = "workflow-execution";
const COMPLETED_ROWS: &[(&str, &str)] = &[
    ("n8nclone.workflow_execution.rats_dependency", RATS_SOURCE_PATH),
    ("n8nclone.workflow_execution.rats_dependency.scenario.source-completeness", RATS_SOURCE_PATH),
    ("n8nclone.workflow_execution.subworkflows.events.correlation", SUBWORKFLOW_SOURCE_PATH),
    ("n8nclone.workflow_execution.subworkflows.invocation.outputs", SUBWORKFLOW_SOURCE_PATH),
];

fn main() -> ExitCode {
    match run(&std::env::args().collect::<Vec<_>>()) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(FAILURE_EXIT_CODE)
        }
    }
}

fn run(arguments: &[String]) -> Result<String, String> {
    if arguments.len() == SELF_TEST_ARG_COUNT && arguments[1] == SELF_TEST_ARG {
        self_test()?;
        return Ok("workflow-execution completion fixture self-test passed".to_string());
    }
    if arguments.len() != EXECUTE_ARG_COUNT || arguments[1] != EXECUTE_ARG {
        return Err(format!("usage: {} {SELF_TEST_ARG} | {EXECUTE_ARG} COVERAGE_JSON", arguments[0]));
    }
    execute(Path::new(&arguments[COVERAGE_ARG_INDEX]))?;
    Ok(format!("wrote {OUTPUT_PATH} and {COVERAGE_OUTPUT_PATH}"))
}

fn execute(coverage_path: &Path) -> Result<(), String> {
    let mut report = read_json(Path::new(AUDIT_PATH))?;
    let rats_bytes = fs::read(RATS_SOURCE_PATH).map_err(|error| format!("read {RATS_SOURCE_PATH}: {error}"))?;
    let subworkflow_bytes =
        fs::read(SUBWORKFLOW_SOURCE_PATH).map_err(|error| format!("read {SUBWORKFLOW_SOURCE_PATH}: {error}"))?;
    let evidence = [
        (RATS_SOURCE_PATH, blake3::hash(&rats_bytes).to_hex().to_string()),
        (SUBWORKFLOW_SOURCE_PATH, blake3::hash(&subworkflow_bytes).to_hex().to_string()),
    ];
    complete_rows(&mut report, &evidence)?;
    validate_report(&report)?;
    write_json(Path::new(OUTPUT_PATH), &report)?;
    fs::write(RATS_SNAPSHOT_PATH, rats_bytes).map_err(|error| format!("write {RATS_SNAPSHOT_PATH}: {error}"))?;
    fs::write(SUBWORKFLOW_SNAPSHOT_PATH, subworkflow_bytes)
        .map_err(|error| format!("write {SUBWORKFLOW_SNAPSHOT_PATH}: {error}"))?;
    fs::copy(coverage_path, COVERAGE_OUTPUT_PATH)
        .map(|_| ())
        .map_err(|error| format!("copy coverage receipt: {error}"))
}

fn complete_rows(report: &mut Value, evidence: &[(&str, String)]) -> Result<(), String> {
    let rows = report
        .get_mut("rows")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "audit report omitted rows".to_string())?;
    let allowlist = COMPLETED_ROWS.iter().map(|(id, _)| *id).collect::<BTreeSet<_>>();
    let mut changed = BTreeSet::new();
    for row in rows {
        let Some(requirement_id) = row.get("requirement_id").and_then(Value::as_str).map(str::to_string) else {
            return Err("audit row omitted requirement_id".to_string());
        };
        let Some((_, evidence_path)) = COMPLETED_ROWS.iter().find(|(id, _)| *id == requirement_id) else {
            continue;
        };
        let evidence_hash = evidence
            .iter()
            .find_map(|(path, hash)| (*path == *evidence_path).then_some(hash))
            .ok_or_else(|| format!("missing evidence hash for {evidence_path}"))?;
        let object = row.as_object_mut().ok_or_else(|| "audit row is not an object".to_string())?;
        object.insert("classification".to_string(), Value::String(EXACT.to_string()));
        object.insert("rationale_code".to_string(), Value::String(RATIONALE.to_string()));
        object.insert("evidence".to_string(), serde_json::json!([{ "path": evidence_path, "blake3": evidence_hash }]));
        changed.insert(requirement_id);
    }
    if changed.len() != EXPECTED_CHANGED_ROWS || changed != allowlist.into_iter().map(str::to_string).collect() {
        return Err(format!("completion changed the wrong rows: {changed:?}"));
    }
    Ok(())
}

fn validate_report(report: &Value) -> Result<(), String> {
    let rows = report.get("rows").and_then(Value::as_array).ok_or_else(|| "completion omitted rows".to_string())?;
    let selected = rows
        .iter()
        .filter(|row| row.get("subsystem").and_then(Value::as_str) == Some(SUBSYSTEM))
        .collect::<Vec<_>>();
    if selected.len() != EXPECTED_SELECTED_ROWS {
        return Err(format!("expected {EXPECTED_SELECTED_ROWS} workflow-execution rows, found {}", selected.len()));
    }
    if selected.iter().any(|row| {
        row.get("classification").and_then(Value::as_str) != Some(EXACT)
            || row.get("evidence").and_then(Value::as_array).is_none_or(Vec::is_empty)
    }) {
        return Err("completion retained non-exact or empty evidence rows".to_string());
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("decode {}: {error}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| format!("encode {}: {error}", path.display()))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn self_test() -> Result<(), String> {
    let mut report = serde_json::json!({
        "rows": COMPLETED_ROWS.iter().map(|(id, _)| serde_json::json!({
            "requirement_id": id,
            "subsystem": SUBSYSTEM,
            "classification": "partial_evidence_only",
            "rationale_code": "gap.test",
            "evidence": [{ "path": "old", "blake3": "old" }]
        })).collect::<Vec<_>>()
    });
    let evidence = [
        (RATS_SOURCE_PATH, "rats-hash".to_string()),
        (SUBWORKFLOW_SOURCE_PATH, "subworkflow-hash".to_string()),
    ];
    complete_rows(&mut report, &evidence)?;
    let rows = report["rows"].as_array().ok_or_else(|| "self-test rows missing".to_string())?;
    if rows.len() != EXPECTED_CHANGED_ROWS
        || rows.iter().any(|row| row["classification"] != EXACT || row["rationale_code"] != RATIONALE)
    {
        return Err("positive completion transform failed".to_string());
    }

    let mut missing = serde_json::json!({ "rows": [] });
    if complete_rows(&mut missing, &evidence).is_ok() {
        return Err("negative completion transform accepted missing allowlisted rows".to_string());
    }
    Ok(())
}
