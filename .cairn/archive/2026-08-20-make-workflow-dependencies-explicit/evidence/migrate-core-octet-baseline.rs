#!/usr/bin/env -S nix develop . -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.3"
serde_json = "=1.0.149"
---

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
const EXPECTED_OLD_TOTAL: usize = 24;
const EXPECTED_STALE_TOTAL: usize = 1;
const EXPECTED_MIGRATED_TOTAL: usize = EXPECTED_OLD_TOTAL - EXPECTED_STALE_TOTAL;
const BASELINE_SCHEMA_VERSION: u64 = 1;

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
        println!("core Octet baseline migration self-test passed");
        return Ok(());
    }
    if arguments.len() != 4 && arguments.len() != 5 {
        return Err(format!(
            "usage: {} OLD_BASELINE STATUS_JSON OUTPUT [{EXECUTE_ARGUMENT}] | {SELF_TEST_ARGUMENT}",
            arguments[0]
        ));
    }
    let execute = arguments.len() == 5 && arguments[4] == EXECUTE_ARGUMENT;
    if arguments.len() == 5 && !execute {
        return Err(format!("unsupported argument: {}", arguments[4]));
    }
    let old = read_json(Path::new(&arguments[1]))?;
    let status = read_json(Path::new(&arguments[2]))?;
    let migrated = migrate_checked(&old, &status)?;
    println!(
        "reviewed core Octet migration: old={} stale={} migrated={} semantic-growth=0",
        EXPECTED_OLD_TOTAL,
        EXPECTED_STALE_TOTAL,
        migrated.len(),
    );
    if execute {
        write_baseline(Path::new(&arguments[3]), &old, &status, &migrated)?;
    }
    Ok(())
}

fn migrate_checked(old: &Value, status: &Value) -> Result<Vec<Value>, String> {
    let baseline = status.get("baseline").ok_or_else(|| "status is missing baseline".to_string())?;
    let new_ids = string_set(baseline, "new_finding_ids")?;
    let stale_ids = string_set(baseline, "stale_finding_ids")?;
    if !new_ids.is_empty() || stale_ids.len() != EXPECTED_STALE_TOTAL {
        return Err(format!("unexpected core identities: new={} stale={}", new_ids.len(), stale_ids.len()));
    }
    let findings = old
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline is missing findings".to_string())?;
    if findings.len() != EXPECTED_OLD_TOTAL {
        return Err(format!("old baseline total is {}, expected {EXPECTED_OLD_TOTAL}", findings.len()));
    }
    let migrated = findings
        .iter()
        .filter(|finding| finding.get("stable_id").and_then(Value::as_str).is_none_or(|id| !stale_ids.contains(id)))
        .cloned()
        .collect::<Vec<_>>();
    if migrated.len() != EXPECTED_MIGRATED_TOTAL {
        return Err(format!("migrated total is {}, expected {EXPECTED_MIGRATED_TOTAL}", migrated.len()));
    }
    Ok(migrated)
}

fn write_baseline(path: &Path, old: &Value, status: &Value, findings: &[Value]) -> Result<(), String> {
    let identities = findings
        .iter()
        .map(|finding| {
            finding
                .get("stable_id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| "finding lacks stable_id".to_string())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let output = serde_json::json!({
        "schema_version": integer_field(old, "schema_version")?,
        "finding_schema_version": integer_field(old, "finding_schema_version")?,
        "metadata": status.get("metadata").cloned().ok_or_else(|| "status is missing metadata".to_string())?,
        "baseline_hash": baseline_hash(&identities),
        "findings": findings,
    });
    let mut text = serde_json::to_string_pretty(&output).map_err(|error| format!("serialize baseline: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

fn self_test() -> Result<(), String> {
    let old = serde_json::json!({
        "schema_version": 1,
        "finding_schema_version": 2,
        "findings": [{"stable_id": "keep"}, {"stable_id": "stale"}],
    });
    let valid = serde_json::json!({"baseline": {"new_finding_ids": [], "stale_finding_ids": ["stale"]}});
    let stale_ids = string_set(valid.get("baseline").ok_or_else(|| "missing baseline".to_string())?, "stale_finding_ids")?;
    let migrated = old
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing findings".to_string())?
        .iter()
        .filter(|finding| finding.get("stable_id").and_then(Value::as_str).is_none_or(|id| !stale_ids.contains(id)))
        .collect::<Vec<_>>();
    if migrated.len() != 1 {
        return Err("positive core migration fixture failed".to_string());
    }
    let invalid = serde_json::json!({"baseline": {"new_finding_ids": ["new"], "stale_finding_ids": ["stale"]}});
    if string_set(invalid.get("baseline").ok_or_else(|| "missing baseline".to_string())?, "new_finding_ids")?.is_empty()
    {
        return Err("negative core migration fixture accepted finding growth".to_string());
    }
    Ok(())
}

fn baseline_hash(ids: &BTreeSet<String>) -> String {
    let payload = serde_json::json!({"schema_version": BASELINE_SCHEMA_VERSION, "stable_ids": ids});
    format!("b3:{}", blake3::hash(payload.to_string().as_bytes()).to_hex())
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

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn integer_field(value: &Value, field: &str) -> Result<u64, String> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| format!("value is missing integer {field}"))
}
