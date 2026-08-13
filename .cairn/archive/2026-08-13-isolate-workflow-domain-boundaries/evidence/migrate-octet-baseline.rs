#!/usr/bin/env -S nix develop . -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.3"
serde_json = "=1.0.149"
---

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde_json::Value;

const EXECUTE_ARGUMENT: &str = "--execute";
const SELF_TEST_ARGUMENT: &str = "--self-test";
const SUCCESS_EXIT_CODE: u8 = 0;
const FAILURE_EXIT_CODE: u8 = 1;
const BASELINE_SCHEMA_VERSION: u64 = 1;
const EXPECTED_OLD_TOTAL: usize = 872;
const EXPECTED_CURRENT_TOTAL: usize = 866;
const EXPECTED_NEW_TOTAL: usize = 45;
const EXPECTED_STALE_TOTAL: usize = 51;
const EXPECTED_NEW_HASH: &str = "b3:dd881425cfbb6501fe4f57eadded4a153e77bceabbc34477b161ba80d73f8bc3";
const EXPECTED_STALE_HASH: &str = "b3:0e4517c9a4d798ce019d4fa4a871520ebd2e8149e62ff59a38d0e047214c1a95";
const AMBIENT_CLOCK_LINT: &str = "ambient_clock";
const TUI_APP_PATH: &str = "src/tui/app.rs";
const DOMAIN_PATH_PREFIX: &str = "src/engine/domain/";
const TUI_ADAPTER_PATH: &str = "src/tui/graph.rs";
const MOVED_CLOCK_ID: &str = "b3:e87e5a93dc780c06";
const REMOVED_CORE_CLOCK_IDS: &[&str] = &["b3:d449a57c2e503dce", "b3:eef4ec0e8b138d8a"];
const EXPECTED_OLD_CLOCK_TOTAL: usize = 7;
const EXPECTED_CURRENT_CLOCK_ID_TOTAL: usize = 6;
const EXPECTED_CURRENT_CLOCK_OCCURRENCES: usize = 7;
const EXPECTED_MOVED_CLOCK_OCCURRENCES: usize = 2;
const PROJECTED_FIELDS: &[&str] = &[
    "stable_id",
    "semantic_id",
    "identity_status",
    "lint",
    "group",
    "severity",
    "file",
    "line",
    "column",
    "message",
    "crate_name",
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::from(SUCCESS_EXIT_CODE),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(FAILURE_EXIT_CODE)
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = env::args().collect::<Vec<_>>();
    if arguments.as_slice().get(1).map(String::as_str) == Some(SELF_TEST_ARGUMENT) {
        self_test()?;
        println!("Octet baseline migration self-test passed");
        return Ok(());
    }
    if arguments.len() != 5 && arguments.len() != 6 {
        return Err(format!(
            "usage: {} OLD_BASELINE RESULTS_JSONL STATUS_JSON OUTPUT [{EXECUTE_ARGUMENT}] | {SELF_TEST_ARGUMENT}",
            arguments[0]
        ));
    }
    let execute = arguments.len() == 6 && arguments[5] == EXECUTE_ARGUMENT;
    if arguments.len() == 6 && !execute {
        return Err(format!("unsupported argument: {}", arguments[5]));
    }

    let old = read_json(Path::new(&arguments[1]))?;
    let result_values = read_json_lines(Path::new(&arguments[2]))?;
    let status = read_json(Path::new(&arguments[3]))?;
    let old_findings = finding_map(findings(&old)?)?;
    let current_findings = finding_map(&result_values)?;
    verify_reviewed_migration(&old_findings, &current_findings, &result_values, &status)?;

    println!(
        "reviewed Octet migration: old={} current={} new={} stale={} lint-category identity growth=0",
        old_findings.len(),
        current_findings.len(),
        EXPECTED_NEW_TOTAL,
        EXPECTED_STALE_TOTAL
    );
    if execute {
        write_baseline(Path::new(&arguments[4]), &old, &status, &current_findings)?;
    }
    Ok(())
}

fn verify_reviewed_migration(
    old: &BTreeMap<String, Value>,
    current: &BTreeMap<String, Value>,
    occurrences: &[Value],
    status: &Value,
) -> Result<(), String> {
    verify_exact_drift(
        &old.keys().cloned().collect(),
        &current.keys().cloned().collect(),
        EXPECTED_NEW_TOTAL,
        EXPECTED_STALE_TOTAL,
        EXPECTED_NEW_HASH,
        EXPECTED_STALE_HASH,
    )?;
    if old.len() != EXPECTED_OLD_TOTAL || current.len() != EXPECTED_CURRENT_TOTAL {
        return Err(format!("unexpected identity totals: old={} current={}", old.len(), current.len()));
    }
    verify_status(status)?;
    verify_no_lint_growth(old, current)?;
    verify_boundary_findings(old, current, occurrences)?;
    Ok(())
}

fn verify_exact_drift(
    old: &BTreeSet<String>,
    current: &BTreeSet<String>,
    expected_new_total: usize,
    expected_stale_total: usize,
    expected_new_hash: &str,
    expected_stale_hash: &str,
) -> Result<(), String> {
    let new = current.difference(old).cloned().collect::<BTreeSet<_>>();
    let stale = old.difference(current).cloned().collect::<BTreeSet<_>>();
    if new.len() != expected_new_total || stale.len() != expected_stale_total {
        return Err(format!("unexpected drift totals: new={} stale={}", new.len(), stale.len()));
    }
    if set_hash(&new) != expected_new_hash || set_hash(&stale) != expected_stale_hash {
        return Err("Octet identity drift differs from the reviewed sets".to_string());
    }
    Ok(())
}

fn verify_status(status: &Value) -> Result<(), String> {
    let baseline = status.get("baseline").ok_or_else(|| "status is missing baseline".to_string())?;
    let current_total = required_u64(baseline, "current_total")? as usize;
    if current_total != EXPECTED_CURRENT_TOTAL {
        return Err(format!("status current_total is {current_total}, expected {EXPECTED_CURRENT_TOTAL}"));
    }
    Ok(())
}

fn verify_no_lint_growth(old: &BTreeMap<String, Value>, current: &BTreeMap<String, Value>) -> Result<(), String> {
    let old_counts = lint_counts(old.values())?;
    let current_counts = lint_counts(current.values())?;
    let growth = current_counts
        .iter()
        .filter(|(lint, count)| **count > old_counts.get(*lint).copied().unwrap_or_default())
        .map(|(lint, count)| format!("{lint}: {} -> {count}", old_counts.get(lint).copied().unwrap_or_default()))
        .collect::<Vec<_>>();
    if !growth.is_empty() {
        return Err(format!("lint-category identity growth: {}", growth.join(", ")));
    }
    Ok(())
}

fn verify_boundary_findings(
    old: &BTreeMap<String, Value>,
    current: &BTreeMap<String, Value>,
    occurrences: &[Value],
) -> Result<(), String> {
    let forbidden = occurrences
        .iter()
        .filter(|finding| {
            let file = string_field(finding, "file").unwrap_or_default();
            let id = string_field(finding, "stable_id").unwrap_or_default();
            !old.contains_key(id) && (file.starts_with(DOMAIN_PATH_PREFIX) || file == TUI_ADAPTER_PATH)
        })
        .count();
    if forbidden != 0 {
        return Err(format!("workflow domain or TUI adapter added {forbidden} Octet occurrences"));
    }

    let old_clock_total = old
        .values()
        .filter(|finding| string_field(finding, "lint").ok() == Some(AMBIENT_CLOCK_LINT))
        .count();
    let old_clock_ids = REMOVED_CORE_CLOCK_IDS.iter().map(|id| (*id).to_string()).collect::<BTreeSet<_>>();
    if !old_clock_ids.is_disjoint(&current.keys().cloned().collect()) {
        return Err("removed core clock identities remain current".to_string());
    }
    let current_clock_ids = current
        .values()
        .filter(|finding| string_field(finding, "lint").ok() == Some(AMBIENT_CLOCK_LINT))
        .count();
    let clock_occurrences = occurrences
        .iter()
        .filter(|finding| string_field(finding, "lint").ok() == Some(AMBIENT_CLOCK_LINT))
        .collect::<Vec<_>>();
    let moved = clock_occurrences
        .iter()
        .filter(|finding| {
            string_field(finding, "stable_id").ok() == Some(MOVED_CLOCK_ID)
                && string_field(finding, "file").ok() == Some(TUI_APP_PATH)
        })
        .count();
    if old_clock_total != EXPECTED_OLD_CLOCK_TOTAL
        || current_clock_ids != EXPECTED_CURRENT_CLOCK_ID_TOTAL
        || clock_occurrences.len() != EXPECTED_CURRENT_CLOCK_OCCURRENCES
        || moved != EXPECTED_MOVED_CLOCK_OCCURRENCES
    {
        return Err(format!(
            "unexpected clock migration: old={old_clock_total} identities={current_clock_ids} occurrences={} moved={moved}",
            clock_occurrences.len()
        ));
    }
    Ok(())
}

fn finding_map(values: &[Value]) -> Result<BTreeMap<String, Value>, String> {
    let mut findings = BTreeMap::new();
    for value in values {
        let id = string_field(value, "stable_id")?.to_string();
        match findings.get(&id) {
            Some(previous) if representative_key(value)? >= representative_key(previous)? => {}
            _ => {
                findings.insert(id, value.clone());
            }
        }
    }
    Ok(findings)
}

fn representative_key(value: &Value) -> Result<(String, u64, u64), String> {
    Ok((
        string_field(value, "file")?.to_string(),
        required_u64(value, "line")?,
        required_u64(value, "column")?,
    ))
}

fn lint_counts<'a>(values: impl Iterator<Item = &'a Value>) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(string_field(value, "lint")?.to_string()).or_default() += 1;
    }
    Ok(counts)
}

fn write_baseline(path: &Path, old: &Value, status: &Value, findings: &BTreeMap<String, Value>) -> Result<(), String> {
    let projected = findings.values().map(project_finding).collect::<Result<Vec<_>, _>>()?;
    let ids = findings.keys().cloned().collect::<BTreeSet<_>>();
    let output = serde_json::json!({
        "schema_version": required_u64(old, "schema_version")?,
        "finding_schema_version": required_u64(old, "finding_schema_version")?,
        "metadata": status.get("metadata").cloned().ok_or_else(|| "status is missing metadata".to_string())?,
        "baseline_hash": set_hash(&ids),
        "findings": projected,
    });
    let mut text =
        serde_json::to_string_pretty(&output).map_err(|error| format!("failed to serialize baseline: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn project_finding(value: &Value) -> Result<Value, String> {
    let mut object = serde_json::Map::new();
    for field in PROJECTED_FIELDS {
        object.insert(
            (*field).to_string(),
            value.get(*field).cloned().ok_or_else(|| format!("finding is missing {field}"))?,
        );
    }
    Ok(Value::Object(object))
}

fn set_hash(ids: &BTreeSet<String>) -> String {
    let payload = serde_json::json!({"schema_version": BASELINE_SCHEMA_VERSION, "stable_ids": ids});
    format!("b3:{}", blake3::hash(payload.to_string().as_bytes()).to_hex())
}

fn findings(value: &Value) -> Result<&Vec<Value>, String> {
    value
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline is missing findings".to_string())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|error| format!("failed to parse {}: {error}", path.display())))
        .collect()
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("value is missing string field {field}"))
}

fn required_u64(value: &Value, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("value is missing integer field {field}"))
}

fn self_test() -> Result<(), String> {
    let old = ["b3:a", "b3:b"].into_iter().map(str::to_string).collect::<BTreeSet<_>>();
    let current = ["b3:b", "b3:c"].into_iter().map(str::to_string).collect::<BTreeSet<_>>();
    let new = ["b3:c".to_string()].into_iter().collect::<BTreeSet<_>>();
    let stale = ["b3:a".to_string()].into_iter().collect::<BTreeSet<_>>();
    verify_exact_drift(&old, &current, 1, 1, &set_hash(&new), &set_hash(&stale))?;
    let changed = ["b3:b", "b3:d"].into_iter().map(str::to_string).collect::<BTreeSet<_>>();
    if verify_exact_drift(&old, &changed, 1, 1, &set_hash(&new), &set_hash(&stale)).is_ok() {
        return Err("negative identity-drift fixture was accepted".to_string());
    }
    Ok(())
}
