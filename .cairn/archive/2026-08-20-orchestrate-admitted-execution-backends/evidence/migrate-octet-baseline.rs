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
const EXPECTED_BASELINE_TOTAL: usize = 855;
const EXPECTED_NEW_IDS: [&str; 2] = ["b3:43d20409c09d7255", "b3:7f07f4ebfec11018"];
const EXPECTED_STALE_IDS: [&str; 2] = ["b3:212d875e0c3257f7", "b3:c3943ee3932075c6"];
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
        println!("Octet identity migration self-test passed");
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
    let migrated = migrate(&old, &results, &status)?;
    println!(
        "reviewed Octet identity migration: total={} new={} stale={} semantic-growth=0",
        migrated.len(),
        EXPECTED_NEW_IDS.len(),
        EXPECTED_STALE_IDS.len()
    );
    if execute {
        write_baseline(Path::new(&arguments[4]), &old, &status, &migrated)?;
    }
    Ok(())
}

fn migrate(old: &Value, results: &[Value], status: &Value) -> Result<BTreeMap<String, Value>, String> {
    let mut old_map = finding_map(findings(old)?)?;
    let current_map = finding_map(results)?;
    let expected_new = expected_set(&EXPECTED_NEW_IDS);
    let expected_stale = expected_set(&EXPECTED_STALE_IDS);
    verify_status(status, &expected_new, &expected_stale)?;
    if old_map.len() != EXPECTED_BASELINE_TOTAL {
        return Err(format!("old baseline total is {}, expected {EXPECTED_BASELINE_TOTAL}", old_map.len()));
    }

    let stale = EXPECTED_STALE_IDS
        .iter()
        .map(|id| old_map.remove(*id).ok_or_else(|| format!("missing stale baseline identity {id}")))
        .collect::<Result<Vec<_>, _>>()?;
    let new = EXPECTED_NEW_IDS
        .iter()
        .map(|id| current_map.get(*id).cloned().ok_or_else(|| format!("missing replacement identity {id}")))
        .collect::<Result<Vec<_>, _>>()?;
    verify_semantic_equivalence(&stale, &new)?;
    for finding in new {
        old_map.insert(string_field(&finding, "stable_id")?.to_string(), finding);
    }
    if old_map.len() != EXPECTED_BASELINE_TOTAL {
        return Err("identity migration changed the accepted finding total".to_string());
    }
    Ok(old_map)
}

fn verify_status(status: &Value, expected_new: &BTreeSet<String>, expected_stale: &BTreeSet<String>) -> Result<(), String> {
    let baseline = status.get("baseline").ok_or_else(|| "status is missing baseline".to_string())?;
    let new = string_set(baseline, "new_finding_ids")?;
    let stale = string_set(baseline, "stale_finding_ids")?;
    if &new != expected_new || &stale != expected_stale {
        return Err(format!("Octet identity drift differs: new={new:?} stale={stale:?}"));
    }
    Ok(())
}

fn verify_semantic_equivalence(stale: &[Value], new: &[Value]) -> Result<(), String> {
    let stale_keys = stale.iter().map(semantic_key).collect::<Result<BTreeSet<_>, _>>()?;
    let new_keys = new.iter().map(semantic_key).collect::<Result<BTreeSet<_>, _>>()?;
    if stale_keys != new_keys {
        return Err("replacement findings differ in lint, file, message, severity, or crate".to_string());
    }
    Ok(())
}

fn semantic_key(finding: &Value) -> Result<String, String> {
    SEMANTIC_FIELDS
        .iter()
        .map(|field| string_field(finding, field).map(str::to_string))
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("\u{1f}"))
}

fn finding_map(values: &[Value]) -> Result<BTreeMap<String, Value>, String> {
    let mut map = BTreeMap::new();
    for value in values {
        let id = string_field(value, "stable_id")?.to_string();
        match map.get(&id) {
            Some(previous) if representative_key(value)? >= representative_key(previous)? => {}
            _ => {
                map.insert(id, value.clone());
            }
        }
    }
    Ok(map)
}

fn representative_key(value: &Value) -> Result<(String, u64, u64), String> {
    Ok((
        string_field(value, "file")?.to_string(),
        integer_field(value, "line")?,
        integer_field(value, "column")?,
    ))
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
        object.insert(
            field.to_string(),
            value.get(field).cloned().ok_or_else(|| format!("finding is missing {field}"))?,
        );
    }
    Ok(Value::Object(object))
}

fn expected_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
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

fn baseline_hash(ids: &BTreeSet<String>) -> String {
    let payload = serde_json::json!({"schema_version": BASELINE_SCHEMA_VERSION, "stable_ids": ids});
    format!("b3:{}", blake3::hash(payload.to_string().as_bytes()).to_hex())
}

fn findings(value: &Value) -> Result<&Vec<Value>, String> {
    value.get("findings").and_then(Value::as_array).ok_or_else(|| "baseline is missing findings".to_string())
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("value is missing string {field}"))
}

fn integer_field(value: &Value, field: &str) -> Result<u64, String> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| format!("value is missing integer {field}"))
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

fn self_test() -> Result<(), String> {
    let old_a = fixture("b3:212d875e0c3257f7", "excessive_file_length");
    let old_b = fixture("b3:c3943ee3932075c6", "underscore_in_module_filename");
    let new_a = fixture("b3:43d20409c09d7255", "excessive_file_length");
    let new_b = fixture("b3:7f07f4ebfec11018", "underscore_in_module_filename");
    let mut fillers = Vec::new();
    for index in 0..(EXPECTED_BASELINE_TOTAL - EXPECTED_STALE_IDS.len()) {
        fillers.push(fixture(&format!("b3:{index:016x}"), "existing"));
    }
    let mut old_findings = fillers.clone();
    old_findings.extend([old_a, old_b]);
    let old = serde_json::json!({"schema_version": 1, "finding_schema_version": 2, "findings": old_findings});
    let mut results = fillers;
    results.extend([new_a, new_b]);
    let status = serde_json::json!({
        "metadata": {},
        "baseline": {
            "new_finding_ids": EXPECTED_NEW_IDS,
            "stale_finding_ids": EXPECTED_STALE_IDS,
        }
    });
    let migrated = migrate(&old, &results, &status)?;
    if migrated.len() != EXPECTED_BASELINE_TOTAL || !EXPECTED_NEW_IDS.iter().all(|id| migrated.contains_key(*id)) {
        return Err("positive migration self-test failed".to_string());
    }
    let invalid_status = serde_json::json!({
        "baseline": {"new_finding_ids": [], "stale_finding_ids": EXPECTED_STALE_IDS}
    });
    if migrate(&old, &results, &invalid_status).is_ok() {
        return Err("negative migration self-test did not reject identity drift".to_string());
    }
    Ok(())
}

fn fixture(id: &str, lint: &str) -> Value {
    serde_json::json!({
        "stable_id": id,
        "semantic_id": id,
        "identity_status": "computed",
        "lint": lint,
        "group": "tigerstyle::structure",
        "severity": "warning",
        "file": "src/workflow_format.rs",
        "line": 1,
        "column": 1,
        "message": lint,
        "crate_name": "lattice",
    })
}
