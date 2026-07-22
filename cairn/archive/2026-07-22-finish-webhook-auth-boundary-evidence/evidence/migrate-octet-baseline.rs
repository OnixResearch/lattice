#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-webhook-octet-migration-target nix shell "github:nix-community/fenix?rev=4e49ef62eedaa5c149d62b63b3f53844e1cc45d7#complete.toolchain" nixpkgs#gcc nixpkgs#clang nixpkgs#mold -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.3"
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.149"
---

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde::Serialize;
use serde_json::Value;

const EXECUTE_ARGUMENT: &str = "--execute";
const SELF_TEST_ARGUMENT: &str = "--self-test";
const BASELINE_SCHEMA_VERSION: u64 = 1;
const SUCCESS_EXIT_CODE: u8 = 0;
const FAILURE_EXIT_CODE: u8 = 1;
const SELF_TEST_ARGUMENT_COUNT: usize = 2;
const DRY_RUN_ARGUMENT_COUNT: usize = 5;
const EXECUTE_ARGUMENT_COUNT: usize = 6;
const DIAGNOSTIC_LIMIT: usize = 100;
const SEMANTIC_FIELDS: &[&str] = &["identity_status", "lint", "group", "file", "message"];
const EXCESSIVE_FILE_LENGTH_LINT: &str = "excessive_file_length";
const MODULE_FILE_COUNT_LINT: &str = "module_file_count";
const NORMALIZED_FILE_LENGTH_MESSAGE: &str = "source file exceeds configured line limit";
const NORMALIZED_MODULE_FILE_COUNT_MESSAGE: &str = "module directory exceeds configured file limit";
const REVIEWED_OLD_IDS: &[&str] = &["b3:0abea2ee24e78611", "b3:bc87db66c7c830c1"];
const REVIEWED_NEW_IDS: &[&str] = &["b3:10eeb86059d6ba71", "b3:6a7c653f247cd2af"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Migration {
    matched: Vec<Value>,
    stale: Vec<Value>,
    new: Vec<Value>,
    old_count: usize,
    candidate_count: usize,
}

#[derive(Debug, Serialize)]
struct BaselineOutput {
    schema_version: u64,
    finding_schema_version: u64,
    metadata: Value,
    baseline_hash: String,
    findings: Vec<BaselineFinding>,
}

#[derive(Debug, Serialize)]
struct BaselineFinding {
    stable_id: String,
    semantic_id: String,
    identity_status: String,
    lint: String,
    group: String,
    severity: String,
    file: String,
    line: u64,
    column: u64,
    message: String,
    crate_name: String,
}

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
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() == SELF_TEST_ARGUMENT_COUNT && arguments[1] == SELF_TEST_ARGUMENT {
        self_test()?;
        println!("Octet baseline migration self-test passed");
        return Ok(());
    }
    if arguments.len() != DRY_RUN_ARGUMENT_COUNT && arguments.len() != EXECUTE_ARGUMENT_COUNT {
        return Err(format!(
            "usage: {} OLD_BASELINE RESULTS_JSONL STATUS_JSON OUTPUT [{EXECUTE_ARGUMENT}] | {SELF_TEST_ARGUMENT}",
            arguments[0]
        ));
    }
    let execute = arguments.len() == EXECUTE_ARGUMENT_COUNT && arguments[5] == EXECUTE_ARGUMENT;
    if arguments.len() == EXECUTE_ARGUMENT_COUNT && !execute {
        return Err(format!("unsupported argument: {}", arguments[5]));
    }

    let old = read_json(Path::new(&arguments[1]))?;
    let candidate_findings = read_json_lines(Path::new(&arguments[2]))?;
    let status = read_json(Path::new(&arguments[3]))?;
    let old_findings = findings(&old)?;
    let migration = migrate(old_findings, &candidate_findings)?;
    print_summary(&migration)?;
    if !migration.stale.is_empty() || !migration.new.is_empty() {
        return Err("semantic finding multiset changed; refusing to rebaseline".to_string());
    }
    validate_status_delta(&status)?;
    if execute {
        write_baseline(Path::new(&arguments[4]), &old, &status, migration.matched)?;
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line)
                .map_err(|error| format!("failed to parse JSONL from {}: {error}", path.display()))
        })
        .collect()
}

fn findings(value: &Value) -> Result<&[Value], String> {
    value
        .get("findings")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| "baseline is missing a findings array".to_string())
}

fn semantic_key(finding: &Value) -> Result<String, String> {
    let lint = required_string(finding, "lint")?;
    let mut parts = Vec::with_capacity(SEMANTIC_FIELDS.len());
    for field in SEMANTIC_FIELDS {
        let value = required_string(finding, field)?;
        let normalized = if *field == "message" {
            normalized_message(&lint, &value)
        } else {
            value
        };
        parts.push(normalized);
    }
    Ok(parts.join("\u{1f}"))
}

fn normalized_message(lint: &str, message: &str) -> String {
    match lint {
        EXCESSIVE_FILE_LENGTH_LINT => NORMALIZED_FILE_LENGTH_MESSAGE.to_string(),
        MODULE_FILE_COUNT_LINT => NORMALIZED_MODULE_FILE_COUNT_MESSAGE.to_string(),
        _ => message.to_string(),
    }
}

fn validate_status_delta(status: &Value) -> Result<(), String> {
    let observed_stale = status_ids(status, "stale_finding_ids")?;
    let observed_new = status_ids(status, "new_finding_ids")?;
    let expected_stale = REVIEWED_OLD_IDS.iter().map(|identifier| (*identifier).to_string()).collect();
    let expected_new = REVIEWED_NEW_IDS.iter().map(|identifier| (*identifier).to_string()).collect();
    if observed_stale != expected_stale || observed_new != expected_new {
        return Err("Octet status differs from the exact reviewed webhook replacements".to_string());
    }
    Ok(())
}

fn status_ids(status: &Value, field: &str) -> Result<BTreeSet<String>, String> {
    status
        .get("baseline")
        .and_then(|baseline| baseline.get(field))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("status is missing baseline.{field}"))?
        .iter()
        .map(|value| value.as_str().map(str::to_string).ok_or_else(|| format!("{field} contains a non-string")))
        .collect()
}

fn grouped(values: &[Value]) -> Result<BTreeMap<String, Vec<Value>>, String> {
    let mut groups = BTreeMap::<String, Vec<Value>>::new();
    for value in values {
        groups.entry(semantic_key(value)?).or_default().push(value.clone());
    }
    for group in groups.values_mut() {
        group.sort_by_key(location_key);
    }
    Ok(groups)
}

fn location_key(finding: &Value) -> (String, u64, u64, String) {
    (
        finding.get("file").and_then(Value::as_str).unwrap_or_default().to_string(),
        finding.get("line").and_then(Value::as_u64).unwrap_or_default(),
        finding.get("column").and_then(Value::as_u64).unwrap_or_default(),
        finding.get("stable_id").and_then(Value::as_str).unwrap_or_default().to_string(),
    )
}

fn migrate(old: &[Value], candidate: &[Value]) -> Result<Migration, String> {
    let old_groups = grouped(old)?;
    let candidate_groups = grouped(candidate)?;
    let mut matched = Vec::new();
    let mut stale = Vec::new();
    let mut new = Vec::new();

    for (key, old_values) in &old_groups {
        let candidate_values = candidate_groups.get(key).map(Vec::as_slice).unwrap_or_default();
        let matched_count = old_values.len().min(candidate_values.len());
        matched.extend_from_slice(&candidate_values[..matched_count]);
        stale.extend_from_slice(&old_values[matched_count..]);
    }
    for (key, candidate_values) in &candidate_groups {
        let old_count = old_groups.get(key).map_or(0, Vec::len);
        if candidate_values.len() > old_count {
            new.extend_from_slice(&candidate_values[old_count..]);
        }
    }
    matched.sort_by_key(location_key);
    stale.sort_by_key(location_key);
    new.sort_by_key(location_key);
    Ok(Migration {
        matched,
        stale,
        new,
        old_count: old.len(),
        candidate_count: candidate.len(),
    })
}

fn print_summary(migration: &Migration) -> Result<(), String> {
    println!(
        "old={} candidate={} matched={} stale={} new={}",
        migration.old_count,
        migration.candidate_count,
        migration.matched.len(),
        migration.stale.len(),
        migration.new.len()
    );
    for finding in migration.stale.iter().take(DIAGNOSTIC_LIMIT) {
        println!("STALE: {}", diagnostic(finding)?);
    }
    for finding in migration.new.iter().take(DIAGNOSTIC_LIMIT) {
        println!("NEW: {}", diagnostic(finding)?);
    }
    Ok(())
}

fn diagnostic(finding: &Value) -> Result<String, String> {
    Ok(format!(
        "{} {}:{} {}",
        required_string(finding, "lint")?,
        required_string(finding, "file")?,
        required_u64(finding, "line")?,
        required_string(finding, "message")?
    ))
}

fn write_baseline(path: &Path, old: &Value, status: &Value, findings: Vec<Value>) -> Result<(), String> {
    let projected: Vec<BaselineFinding> = findings.iter().map(project_finding).collect::<Result<_, _>>()?;
    let stable_ids: Vec<String> = projected.iter().map(|finding| finding.stable_id.clone()).collect();
    let output = BaselineOutput {
        schema_version: required_u64(old, "schema_version")?,
        finding_schema_version: required_u64(old, "finding_schema_version")?,
        metadata: status.get("metadata").cloned().ok_or_else(|| "status is missing metadata".to_string())?,
        baseline_hash: baseline_hash(&stable_ids),
        findings: projected,
    };
    let mut text = serde_json::to_string_pretty(&output)
        .map_err(|error| format!("failed to serialize migrated baseline: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn project_finding(finding: &Value) -> Result<BaselineFinding, String> {
    Ok(BaselineFinding {
        stable_id: required_string(finding, "stable_id")?,
        semantic_id: required_string(finding, "semantic_id")?,
        identity_status: required_string(finding, "identity_status")?,
        lint: required_string(finding, "lint")?,
        group: required_string(finding, "group")?,
        severity: required_string(finding, "severity")?,
        file: required_string(finding, "file")?,
        line: required_u64(finding, "line")?,
        column: required_u64(finding, "column")?,
        message: required_string(finding, "message")?,
        crate_name: required_string(finding, "crate_name")?,
    })
}

fn baseline_hash(stable_ids: &[String]) -> String {
    let mut sorted_ids = stable_ids.to_vec();
    sorted_ids.sort();
    sorted_ids.dedup();
    let payload = serde_json::json!({
        "schema_version": BASELINE_SCHEMA_VERSION,
        "stable_ids": sorted_ids,
    });
    format!("b3:{}", blake3::hash(payload.to_string().as_bytes()).to_hex())
}

fn required_string(value: &Value, field: &str) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("value is missing string field {field}"))
}

fn required_u64(value: &Value, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("value is missing integer field {field}"))
}

fn finding(stable_id: &str, severity: &str, line: u64, message: &str) -> Value {
    serde_json::json!({
        "stable_id": stable_id,
        "semantic_id": stable_id,
        "identity_status": "computed",
        "lint": "demo_lint",
        "group": "demo::group",
        "severity": severity,
        "file": "src/lib.rs",
        "line": line,
        "column": 1,
        "message": message,
        "crate_name": "lattice",
    })
}

fn self_test() -> Result<(), String> {
    let old = vec![finding("b3:old", "warning", 1, "same finding")];
    let candidate = vec![finding("b3:new", "error", 2, "same finding")];
    let severity_migration = migrate(&old, &candidate)?;
    if severity_migration.matched.len() != 1
        || !severity_migration.stale.is_empty()
        || !severity_migration.new.is_empty()
    {
        return Err("positive severity-only migration did not match".to_string());
    }

    let additional = vec![
        finding("b3:new", "error", 2, "same finding"),
        finding("b3:added", "error", 3, "same finding"),
    ];
    let rejected = migrate(&old, &additional)?;
    if rejected.new.len() != 1 || !rejected.stale.is_empty() {
        return Err("negative additional finding was not isolated".to_string());
    }

    let removed = migrate(&old, &[])?;
    if removed.stale.len() != 1 || !removed.new.is_empty() {
        return Err("resolved finding was not classified as stale".to_string());
    }

    let positive_status = serde_json::json!({
        "baseline": {
            "stale_finding_ids": REVIEWED_OLD_IDS,
            "new_finding_ids": REVIEWED_NEW_IDS,
        }
    });
    validate_status_delta(&positive_status)?;
    let negative_status = serde_json::json!({
        "baseline": {
            "stale_finding_ids": REVIEWED_OLD_IDS,
            "new_finding_ids": ["b3:unexpected"],
        }
    });
    if validate_status_delta(&negative_status).is_ok() {
        return Err("negative status delta was accepted".to_string());
    }
    Ok(())
}
