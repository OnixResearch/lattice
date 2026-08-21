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
const EXPECTED_OLD_TOTAL: usize = 855;
const EXPECTED_NEW_TOTAL: usize = 16;
const EXPECTED_STALE_TOTAL: usize = 24;
const EXPECTED_IMPROVEMENT_TOTAL: usize = EXPECTED_STALE_TOTAL - EXPECTED_NEW_TOTAL;
const EXPECTED_MIGRATED_TOTAL: usize = EXPECTED_OLD_TOTAL - EXPECTED_IMPROVEMENT_TOTAL;
const PROJECTED_FIELDS: [&str; 11] = [
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
const SEMANTIC_FIELDS: [&str; 6] = ["lint", "group", "severity", "file", "message", "crate_name"];

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
    let results = read_json_lines(Path::new(&arguments[2]))?;
    let status = read_json(Path::new(&arguments[3]))?;
    let migrated = migrate_checked(&old, &results, &status)?;
    println!(
        "reviewed Octet migration: old={} new-identities={} stale-identities={} improvements={} migrated={} semantic-growth=0",
        EXPECTED_OLD_TOTAL,
        EXPECTED_NEW_TOTAL,
        EXPECTED_STALE_TOTAL,
        EXPECTED_IMPROVEMENT_TOTAL,
        migrated.len(),
    );
    if execute {
        write_baseline(Path::new(&arguments[4]), &old, &status, &migrated)?;
    }
    Ok(())
}

fn migrate_checked(old: &Value, results: &[Value], status: &Value) -> Result<BTreeMap<String, Value>, String> {
    let baseline = status.get("baseline").ok_or_else(|| "status is missing baseline".to_string())?;
    let new_ids = string_set(baseline, "new_finding_ids")?;
    let stale_ids = string_set(baseline, "stale_finding_ids")?;
    if new_ids.len() != EXPECTED_NEW_TOTAL || stale_ids.len() != EXPECTED_STALE_TOTAL {
        return Err(format!("unexpected identity counts: new={} stale={}", new_ids.len(), stale_ids.len()));
    }
    let old_map = finding_map(findings(old)?)?;
    let current_map = finding_map(results)?;
    if old_map.len() != EXPECTED_OLD_TOTAL {
        return Err(format!("old baseline total is {}, expected {EXPECTED_OLD_TOTAL}", old_map.len()));
    }
    let migrated = migrate(old_map, &current_map, &new_ids, &stale_ids)?;
    if migrated.len() != EXPECTED_MIGRATED_TOTAL {
        return Err(format!("migrated total is {}, expected {EXPECTED_MIGRATED_TOTAL}", migrated.len()));
    }
    Ok(migrated)
}

fn migrate(
    mut old: BTreeMap<String, Value>,
    current: &BTreeMap<String, Value>,
    new_ids: &BTreeSet<String>,
    stale_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, Value>, String> {
    let stale = stale_ids
        .iter()
        .map(|id| old.remove(id).ok_or_else(|| format!("missing stale finding {id}")))
        .collect::<Result<Vec<_>, _>>()?;
    let new = new_ids
        .iter()
        .map(|id| current.get(id).cloned().ok_or_else(|| format!("missing new finding {id}")))
        .collect::<Result<Vec<_>, _>>()?;
    verify_no_semantic_growth(&stale, &new)?;
    for finding in new {
        old.insert(string_field(&finding, "stable_id")?.to_string(), finding);
    }
    Ok(old)
}

fn verify_no_semantic_growth(stale: &[Value], new: &[Value]) -> Result<(), String> {
    let mut stale_counts = BTreeMap::<String, usize>::new();
    for finding in stale {
        let key = semantic_key(finding)?;
        *stale_counts.entry(key).or_default() += 1;
    }
    for finding in new {
        let key = semantic_key(finding)?;
        let count = stale_counts.get_mut(&key).ok_or_else(|| format!("new semantic finding has no stale peer: {key}"))?;
        if *count == 0 {
            return Err(format!("new semantic finding exceeds stale peers: {key}"));
        }
        *count -= 1;
    }
    let improvements = stale_counts.values().copied().sum::<usize>();
    if improvements != stale.len().saturating_sub(new.len()) {
        return Err("semantic identity accounting is inconsistent".to_string());
    }
    Ok(())
}

fn semantic_key(finding: &Value) -> Result<String, String> {
    let lint = string_field(finding, "lint")?;
    SEMANTIC_FIELDS
        .iter()
        .map(|field| {
            if *field == "message" {
                semantic_message(lint, string_field(finding, field)?)
            } else {
                string_field(finding, field).map(str::to_string)
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("\u{1f}"))
}

fn semantic_message(lint: &str, message: &str) -> Result<String, String> {
    match lint {
        "excessive_file_length" => Ok("source file exceeds the reviewed length limit".to_string()),
        "assertion_density" => message
            .split_once(" has ")
            .map(|(subject, _details)| format!("{subject} has reviewed assertion-density debt"))
            .ok_or_else(|| "assertion-density message lacks a subject".to_string()),
        _ => Ok(message.to_string()),
    }
}

fn finding_map(values: &[Value]) -> Result<BTreeMap<String, Value>, String> {
    values
        .iter()
        .map(|value| Ok((string_field(value, "stable_id")?.to_string(), value.clone())))
        .collect()
}

fn write_baseline(
    path: &Path,
    old: &Value,
    status: &Value,
    findings: &BTreeMap<String, Value>,
) -> Result<(), String> {
    let projected = findings.values().map(project).collect::<Result<Vec<_>, _>>()?;
    let identities = findings.keys().cloned().collect::<BTreeSet<_>>();
    let output = serde_json::json!({
        "schema_version": integer_field(old, "schema_version")?,
        "finding_schema_version": integer_field(old, "finding_schema_version")?,
        "metadata": status.get("metadata").cloned().ok_or_else(|| "status is missing metadata".to_string())?,
        "baseline_hash": baseline_hash(&identities),
        "findings": projected,
    });
    let mut text = serde_json::to_string_pretty(&output).map_err(|error| format!("serialize baseline: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

fn project(value: &Value) -> Result<Value, String> {
    let mut object = serde_json::Map::new();
    for field in PROJECTED_FIELDS {
        object.insert(field.to_string(), value.get(field).cloned().ok_or_else(|| format!("missing {field}"))?);
    }
    Ok(Value::Object(object))
}

fn baseline_hash(ids: &BTreeSet<String>) -> String {
    let payload = serde_json::json!({"schema_version": BASELINE_SCHEMA_VERSION, "stable_ids": ids});
    format!("b3:{}", blake3::hash(payload.to_string().as_bytes()).to_hex())
}

fn self_test() -> Result<(), String> {
    let stale_a = fixture("old-a", "same");
    let stale_b = fixture("old-b", "removed");
    let new_a = fixture("new-a", "same");
    let old = finding_map(&[stale_a.clone(), stale_b])?;
    let current = finding_map(std::slice::from_ref(&new_a))?;
    let migrated = migrate(
        old,
        &current,
        &BTreeSet::from(["new-a".to_string()]),
        &BTreeSet::from(["old-a".to_string(), "old-b".to_string()]),
    )?;
    if migrated.len() != 1 || !migrated.contains_key("new-a") {
        return Err("positive migration fixture failed".to_string());
    }
    let changed = fixture("new-b", "changed");
    let changed_map = finding_map(std::slice::from_ref(&changed))?;
    if migrate(
        finding_map(std::slice::from_ref(&stale_a))?,
        &changed_map,
        &BTreeSet::from(["new-b".to_string()]),
        &BTreeSet::from(["old-a".to_string()]),
    )
    .is_ok()
    {
        return Err("negative semantic-growth fixture was accepted".to_string());
    }
    Ok(())
}

fn fixture(id: &str, message: &str) -> Value {
    serde_json::json!({
        "stable_id": id,
        "semantic_id": id,
        "identity_status": "computed",
        "lint": "lint",
        "group": "group",
        "severity": "warning",
        "file": "file.rs",
        "line": 1,
        "column": 1,
        "message": message,
        "crate_name": "crate",
    })
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|error| format!("parse {}: {error}", path.display())))
        .collect()
}

fn findings(value: &Value) -> Result<&Vec<Value>, String> {
    value.get("findings").and_then(Value::as_array).ok_or_else(|| "baseline is missing findings".to_string())
}

fn string_set(value: &Value, field: &str) -> Result<BTreeSet<String>, String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("value is missing array {field}"))?
        .iter()
        .map(|item| item.as_str().map(str::to_string).ok_or_else(|| format!("{field} contains non-string")))
        .collect()
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("value is missing string {field}"))
}

fn integer_field(value: &Value, field: &str) -> Result<u64, String> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| format!("value is missing integer {field}"))
}
