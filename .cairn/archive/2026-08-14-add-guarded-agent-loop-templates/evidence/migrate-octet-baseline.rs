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
const EXPECTED_OLD_TOTAL: usize = 856;
const EXPECTED_CURRENT_TOTAL: usize = 856;
const EXPECTED_NEW_TOTAL: usize = 1;
const EXPECTED_STALE_TOTAL: usize = 1;
const EXPECTED_PAIRED_TOTAL: usize = 1;
const EXPECTED_REPAIRED_TOTAL: usize = 0;
const EXPECTED_NEW_HASH: &str = "b3:0d6a54edc74b22e60eb4d28dffa444fee472ee573eed197664f80b1d06653189";
const EXPECTED_STALE_HASH: &str = "b3:818c56b4c38398bdf5360888ec54e229d78cb5625dbd1489e829eafcf8bc1a6f";
const REPAIRED_FINDINGS: &[&str] = &[];
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
    if arguments.get(1).map(String::as_str) == Some(SELF_TEST_ARGUMENT) {
        self_test()?;
        println!("guarded-agent Octet migration self-test passed");
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
    let occurrences = read_json_lines(Path::new(&arguments[2]))?;
    let status = read_json(Path::new(&arguments[3]))?;
    let old_findings = finding_map(findings(&old)?)?;
    let current_findings = finding_map(&occurrences)?;
    verify_reviewed_migration(&old_findings, &current_findings, &status)?;

    println!(
        "reviewed guarded-agent Octet migration: old={} current={} paired={} repaired={} lint-category growth=0",
        old_findings.len(),
        current_findings.len(),
        EXPECTED_PAIRED_TOTAL,
        EXPECTED_REPAIRED_TOTAL
    );
    if execute {
        write_baseline(Path::new(&arguments[4]), &old, &status, &current_findings)?;
    }
    Ok(())
}

fn verify_reviewed_migration(
    old: &BTreeMap<String, Value>,
    current: &BTreeMap<String, Value>,
    status: &Value,
) -> Result<(), String> {
    let old_ids = old.keys().cloned().collect::<BTreeSet<_>>();
    let current_ids = current.keys().cloned().collect::<BTreeSet<_>>();
    let new_ids = current_ids.difference(&old_ids).cloned().collect::<BTreeSet<_>>();
    let stale_ids = old_ids.difference(&current_ids).cloned().collect::<BTreeSet<_>>();
    verify_exact_drift(&new_ids, &stale_ids)?;
    if old.len() != EXPECTED_OLD_TOTAL || current.len() != EXPECTED_CURRENT_TOTAL {
        return Err(format!("unexpected identity totals: old={} current={}", old.len(), current.len()));
    }
    verify_status(status)?;
    verify_no_lint_growth(old, current)?;
    verify_semantic_drift(old, current, &new_ids, &stale_ids)?;
    Ok(())
}

fn verify_exact_drift(new_ids: &BTreeSet<String>, stale_ids: &BTreeSet<String>) -> Result<(), String> {
    if new_ids.len() != EXPECTED_NEW_TOTAL || stale_ids.len() != EXPECTED_STALE_TOTAL {
        return Err(format!("unexpected drift totals: new={} stale={}", new_ids.len(), stale_ids.len()));
    }
    if set_hash(new_ids) != EXPECTED_NEW_HASH || set_hash(stale_ids) != EXPECTED_STALE_HASH {
        return Err("Octet identity drift differs from the reviewed sets".to_string());
    }
    Ok(())
}

fn verify_status(status: &Value) -> Result<(), String> {
    let baseline = status.get("baseline").ok_or_else(|| "status is missing baseline".to_string())?;
    let checks = [
        ("accepted_total", EXPECTED_OLD_TOTAL),
        ("current_total", EXPECTED_CURRENT_TOTAL),
        ("new_total", EXPECTED_NEW_TOTAL),
        ("stale_total", EXPECTED_STALE_TOTAL),
    ];
    for (field, expected) in checks {
        let observed = required_u64(baseline, field)? as usize;
        if observed != expected {
            return Err(format!("status {field} is {observed}, expected {expected}"));
        }
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

fn verify_semantic_drift(
    old: &BTreeMap<String, Value>,
    current: &BTreeMap<String, Value>,
    new_ids: &BTreeSet<String>,
    stale_ids: &BTreeSet<String>,
) -> Result<(), String> {
    let stale_groups = semantic_groups(stale_ids.iter().map(|id| &old[id]))?;
    let new_groups = semantic_groups(new_ids.iter().map(|id| &current[id]))?;
    let keys = stale_groups.keys().chain(new_groups.keys()).cloned().collect::<BTreeSet<_>>();
    let mut paired_total = 0usize;
    let mut unpaired_new = BTreeSet::new();
    let mut unpaired_stale = BTreeSet::new();
    for key in keys {
        let stale = stale_groups.get(&key).map(Vec::as_slice).unwrap_or_default();
        let new = new_groups.get(&key).map(Vec::as_slice).unwrap_or_default();
        let paired = stale.len().min(new.len());
        paired_total = paired_total.saturating_add(paired);
        unpaired_stale.extend(stale[paired..].iter().cloned());
        unpaired_new.extend(new[paired..].iter().cloned());
    }
    let expected_repaired = REPAIRED_FINDINGS.iter().map(|identity| (*identity).to_string()).collect::<BTreeSet<_>>();
    if paired_total != EXPECTED_PAIRED_TOTAL || !unpaired_new.is_empty() || unpaired_stale != expected_repaired {
        return Err(format!(
            "semantic drift mismatch: paired={paired_total} unpaired_new={unpaired_new:?} unpaired_stale={unpaired_stale:?}"
        ));
    }
    Ok(())
}

fn semantic_groups<'a>(values: impl Iterator<Item = &'a Value>) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for value in values {
        let key = format!(
            "{}\t{}\t{}",
            string_field(value, "lint")?,
            string_field(value, "file")?,
            normalize_digits(string_field(value, "message")?)
        );
        groups.entry(key).or_default().push(string_field(value, "stable_id")?.to_string());
    }
    Ok(groups)
}

fn normalize_digits(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut reading_digits = false;
    for character in value.chars() {
        if character.is_ascii_digit() {
            if !reading_digits {
                normalized.push('#');
            }
            reading_digits = true;
        } else {
            reading_digits = false;
            normalized.push(character);
        }
    }
    normalized
}

fn finding_map(values: &[Value]) -> Result<BTreeMap<String, Value>, String> {
    let mut output = BTreeMap::new();
    for value in values {
        let id = string_field(value, "stable_id")?.to_string();
        match output.get(&id) {
            Some(previous) if representative_key(value)? >= representative_key(previous)? => {}
            _ => {
                output.insert(id, value.clone());
            }
        }
    }
    Ok(output)
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
    let mut text = serde_json::to_string_pretty(&output)
        .map_err(|error| format!("failed to serialize baseline: {error}"))?;
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

fn fixture_finding(id: &str, lint: &str, file: &str, message: &str) -> Value {
    serde_json::json!({
        "stable_id": id,
        "lint": lint,
        "file": file,
        "message": message,
        "line": 1,
        "column": 1,
    })
}

fn self_test() -> Result<(), String> {
    let fixed_id = "b3:fixed";
    let old_values = [
        fixture_finding(fixed_id, "growth", "src/example.rs", "collection grows"),
        fixture_finding("b3:old", "length", "src/example.rs", "function has 80 lines"),
    ];
    let current_values = [fixture_finding(
        "b3:new",
        "length",
        "src/example.rs",
        "function has 96 lines",
    )];
    let old = finding_map(&old_values)?;
    let current = finding_map(&current_values)?;
    let old_ids = old.keys().cloned().collect::<BTreeSet<_>>();
    let current_ids = current.keys().cloned().collect::<BTreeSet<_>>();
    let new_ids = current_ids.difference(&old_ids).cloned().collect::<BTreeSet<_>>();
    let stale_ids = old_ids.difference(&current_ids).cloned().collect::<BTreeSet<_>>();
    let stale_groups = semantic_groups(stale_ids.iter().map(|id| &old[id]))?;
    let new_groups = semantic_groups(new_ids.iter().map(|id| &current[id]))?;
    if stale_groups.keys().filter(|key| new_groups.contains_key(*key)).count() != 1 {
        return Err("positive semantic pairing fixture failed".to_string());
    }
    let changed = [fixture_finding(
        "b3:new",
        "different",
        "src/example.rs",
        "unrelated finding",
    )];
    let changed = finding_map(&changed)?;
    let changed_groups = semantic_groups(changed.values())?;
    if stale_groups.keys().any(|key| changed_groups.contains_key(key)) {
        return Err("negative semantic pairing fixture was accepted".to_string());
    }
    Ok(())
}
