#!/usr/bin/env -S nix develop . -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.3"
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.149"
---

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde::Serialize;
use serde_json::Value;

const EXECUTE_ARG: &str = "--execute";
const SELF_TEST_ARG: &str = "--self-test";
const BASELINE_SCHEMA_VERSION: u64 = 1;
const SUCCESS_EXIT_CODE: u8 = 0;
const FAILURE_EXIT_CODE: u8 = 1;
const SELF_TEST_ARG_COUNT: usize = 2;
const DRY_RUN_ARG_COUNT: usize = 5;
const EXECUTE_ARG_COUNT: usize = 6;
const DIAGNOSTIC_LIMIT: usize = 20;
const SEMANTIC_FIELDS: &[&str] = &["identity_status", "lint", "group", "severity", "file", "message"];
const EXCESSIVE_FILE_LENGTH_LINT: &str = "excessive_file_length";
const ASSERTION_DENSITY_LINT: &str = "assertion_density";
const FUNCTION_LENGTH_LINT: &str = "function_length";
const NORMALIZED_FILE_LENGTH_MESSAGE: &str = "source file exceeds configured line limit";
const NORMALIZED_ASSERTION_DENSITY_SUFFIX: &str = " has insufficient assertion density";
const NORMALIZED_FUNCTION_LENGTH_SUFFIX: &str = " exceeds configured function length";
const ALLOWED_RESOLVED_LINT: &str = "ignored_result";
const ALLOWED_RESOLVED_FILE: &str = "src/tui/app.rs";
const ALLOWED_RESOLVED_MESSAGE: &str = "`Result` value silently discarded with `let _ = ...`";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Migration {
    findings: Vec<Value>,
    old_count: usize,
    candidate_count: usize,
    unmatched_old: Vec<String>,
    unmatched_candidate: Vec<String>,
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
    let args: Vec<String> = env::args().collect();
    if args.len() == SELF_TEST_ARG_COUNT && args[1] == SELF_TEST_ARG {
        self_test()?;
        println!("Octet baseline migration self-test passed");
        return Ok(());
    }
    if args.len() != DRY_RUN_ARG_COUNT && args.len() != EXECUTE_ARG_COUNT {
        return Err(format!(
            "usage: {} OLD_BASELINE RESULTS_JSONL STATUS_JSON OUTPUT [{EXECUTE_ARG}] | {SELF_TEST_ARG}",
            args[0]
        ));
    }
    let execute = args.len() == EXECUTE_ARG_COUNT && args[5] == EXECUTE_ARG;
    if args.len() == EXECUTE_ARG_COUNT && !execute {
        return Err(format!("unsupported argument: {}", args[5]));
    }

    let old = read_json(Path::new(&args[1]))?;
    let current_findings = read_json_lines(Path::new(&args[2]))?;
    let status = read_json(Path::new(&args[3]))?;
    let candidate = candidate_baseline(&old, current_findings, &status)?;
    let migration = migrate(&old, &candidate)?;
    print_summary(&migration);
    if !migration_is_accepted(&migration) {
        return Err(
            "semantic finding sets differ outside the exact resolved-finding allowlist; refusing to rebaseline"
                .to_string(),
        );
    }

    if execute {
        write_migrated_baseline(Path::new(&args[4]), &old, &status, migration.findings)?;
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

fn candidate_baseline(old: &Value, findings: Vec<Value>, status: &Value) -> Result<Value, String> {
    let schema_version = old
        .get("schema_version")
        .cloned()
        .ok_or_else(|| "old baseline is missing schema_version".to_string())?;
    let finding_schema_version = old
        .get("finding_schema_version")
        .cloned()
        .ok_or_else(|| "old baseline is missing finding_schema_version".to_string())?;
    let metadata = status.get("metadata").cloned().ok_or_else(|| "status is missing metadata".to_string())?;
    Ok(serde_json::json!({
        "schema_version": schema_version,
        "finding_schema_version": finding_schema_version,
        "metadata": metadata,
        "baseline_hash": "",
        "findings": findings,
    }))
}

fn write_migrated_baseline(path: &Path, old: &Value, status: &Value, findings: Vec<Value>) -> Result<(), String> {
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

fn project_finding(value: &Value) -> Result<BaselineFinding, String> {
    Ok(BaselineFinding {
        stable_id: required_string(value, "stable_id")?,
        semantic_id: required_string(value, "semantic_id")?,
        identity_status: required_string(value, "identity_status")?,
        lint: required_string(value, "lint")?,
        group: required_string(value, "group")?,
        severity: required_string(value, "severity")?,
        file: required_string(value, "file")?,
        line: required_u64(value, "line")?,
        column: required_u64(value, "column")?,
        message: required_string(value, "message")?,
        crate_name: required_string(value, "crate_name")?,
    })
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

fn findings(value: &Value) -> Result<&Vec<Value>, String> {
    value
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline is missing a findings array".to_string())
}

fn semantic_key(finding: &Value) -> Result<String, String> {
    let lint = finding
        .get("lint")
        .and_then(Value::as_str)
        .ok_or_else(|| "finding is missing string field lint".to_string())?;
    let mut parts = Vec::with_capacity(SEMANTIC_FIELDS.len());
    for field in SEMANTIC_FIELDS {
        let value = finding
            .get(*field)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("finding is missing string field {field}"))?;
        if *field == "message" {
            parts.push(normalized_message(lint, value));
        } else {
            parts.push(value.to_string());
        }
    }
    Ok(parts.join("\u{1f}"))
}

fn normalized_message(lint: &str, message: &str) -> String {
    if lint == EXCESSIVE_FILE_LENGTH_LINT {
        return NORMALIZED_FILE_LENGTH_MESSAGE.to_string();
    }
    if lint == ASSERTION_DENSITY_LINT
        && let Some((function, _)) = message.split_once(" has ")
    {
        return format!("{function}{NORMALIZED_ASSERTION_DENSITY_SUFFIX}");
    }
    if lint == FUNCTION_LENGTH_LINT
        && let Some((function, _)) = message.split_once(" is ")
    {
        return format!("{function}{NORMALIZED_FUNCTION_LENGTH_SUFFIX}");
    }
    message.to_string()
}

fn grouped(findings: &[Value]) -> Result<BTreeMap<String, Vec<Value>>, String> {
    let mut groups = BTreeMap::<String, Vec<Value>>::new();
    for finding in findings {
        groups.entry(semantic_key(finding)?).or_default().push(finding.clone());
    }
    for values in groups.values_mut() {
        values.sort_by_key(|finding| {
            (
                finding.get("line").and_then(Value::as_u64).unwrap_or_default(),
                finding.get("column").and_then(Value::as_u64).unwrap_or_default(),
            )
        });
    }
    Ok(groups)
}

fn migrate(old: &Value, candidate: &Value) -> Result<Migration, String> {
    let old_findings = findings(old)?;
    let candidate_findings = findings(candidate)?;
    let old_groups = grouped(old_findings)?;
    let candidate_groups = grouped(candidate_findings)?;
    let mut unmatched_old = Vec::new();
    let mut unmatched_candidate = Vec::new();

    for (key, old_values) in &old_groups {
        match candidate_groups.get(key) {
            Some(candidate_values) if candidate_values.len() == old_values.len() => {}
            Some(candidate_values) => unmatched_old.push(format!(
                "count mismatch old={} candidate={} key={key}",
                old_values.len(),
                candidate_values.len()
            )),
            None => unmatched_old.push(format!("missing candidate key={key}")),
        }
    }
    for (key, candidate_values) in &candidate_groups {
        if !old_groups.contains_key(key) {
            unmatched_candidate.push(format!("new count={} key={key}", candidate_values.len()));
        }
    }

    let mut migrated = Vec::with_capacity(old_findings.len());
    for finding in candidate_findings {
        let key = semantic_key(finding)?;
        let has_matching_count = old_groups
            .get(&key)
            .zip(candidate_groups.get(&key))
            .is_some_and(|(old_values, candidate_values)| old_values.len() == candidate_values.len());
        if has_matching_count {
            migrated.push(finding.clone());
        }
    }
    Ok(Migration {
        findings: migrated,
        old_count: old_findings.len(),
        candidate_count: candidate_findings.len(),
        unmatched_old,
        unmatched_candidate,
    })
}

fn migration_is_accepted(migration: &Migration) -> bool {
    if !migration.unmatched_candidate.is_empty() {
        return false;
    }
    if migration.unmatched_old.is_empty() {
        return true;
    }
    migration.unmatched_old.len() == 1
        && migration.unmatched_old[0].contains(ALLOWED_RESOLVED_LINT)
        && migration.unmatched_old[0].contains(ALLOWED_RESOLVED_FILE)
        && migration.unmatched_old[0].contains(ALLOWED_RESOLVED_MESSAGE)
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

fn print_summary(migration: &Migration) {
    println!(
        "old={} candidate={} matched={} unmatched_old={} unmatched_candidate={}",
        migration.old_count,
        migration.candidate_count,
        migration.findings.len(),
        migration.unmatched_old.len(),
        migration.unmatched_candidate.len()
    );
    for diagnostic in migration.unmatched_old.iter().take(DIAGNOSTIC_LIMIT) {
        println!("OLD: {diagnostic}");
    }
    for diagnostic in migration.unmatched_candidate.iter().take(DIAGNOSTIC_LIMIT) {
        println!("NEW: {diagnostic}");
    }
}

fn finding(stable_id: &str, crate_name: &str, message: &str) -> Value {
    serde_json::json!({
        "stable_id": stable_id,
        "semantic_id": stable_id,
        "identity_status": "computed",
        "lint": "demo_lint",
        "group": "demo::group",
        "severity": "error",
        "file": "src/lib.rs",
        "line": 1,
        "column": 1,
        "message": message,
        "crate_name": crate_name,
    })
}

fn finding_with_lint(stable_id: &str, crate_name: &str, lint: &str, file: &str, message: &str) -> Value {
    let mut value = finding(stable_id, crate_name, message);
    value["lint"] = Value::String(lint.to_string());
    value["file"] = Value::String(file.to_string());
    value
}

fn file_length_finding(stable_id: &str, crate_name: &str, message: &str) -> Value {
    finding_with_lint(stable_id, crate_name, EXCESSIVE_FILE_LENGTH_LINT, "src/lib.rs", message)
}

fn self_test() -> Result<(), String> {
    let old = serde_json::json!({"findings": [finding("b3:old", "n8nclone", "same finding")]});
    let candidate = serde_json::json!({"findings": [finding("b3:new", "lattice", "same finding")]});
    let migration = migrate(&old, &candidate)?;
    if migration.findings.len() != 1 || !migration.unmatched_old.is_empty() || !migration.unmatched_candidate.is_empty()
    {
        return Err("positive crate-identity migration did not match".to_string());
    }

    let drift = serde_json::json!({"findings": [finding("b3:new", "lattice", "changed finding")]});
    let rejected = migrate(&old, &drift)?;
    if rejected.unmatched_old.is_empty() || rejected.unmatched_candidate.is_empty() {
        return Err("negative semantic drift was not rejected".to_string());
    }

    let old_length = serde_json::json!({
        "findings": [file_length_finding("b3:old-length", "n8nclone", "source file is 10 lines (limit: 5)")]
    });
    let new_length = serde_json::json!({
        "findings": [file_length_finding("b3:new-length", "lattice", "source file is 12 lines (limit: 5)")]
    });
    let length_migration = migrate(&old_length, &new_length)?;
    if length_migration.findings.len() != 1
        || !length_migration.unmatched_old.is_empty()
        || !length_migration.unmatched_candidate.is_empty()
    {
        return Err("file-length identity migration did not normalize line-count drift".to_string());
    }

    let old_density = serde_json::json!({
        "findings": [finding_with_lint(
            "b3:old-density",
            "n8nclone",
            ASSERTION_DENSITY_LINT,
            "src/node_kind_provider.rs",
            "function `validate_config_for_kind` has 0 assertion(s) in 73 lines (need 2+)",
        )]
    });
    let new_density = serde_json::json!({
        "findings": [finding_with_lint(
            "b3:new-density",
            "lattice",
            ASSERTION_DENSITY_LINT,
            "src/node_kind_provider.rs",
            "function `validate_config_for_kind` has 0 assertion(s) in 76 lines (need 2+)",
        )]
    });
    let density_migration = migrate(&old_density, &new_density)?;
    if density_migration.findings.len() != 1
        || !density_migration.unmatched_old.is_empty()
        || !density_migration.unmatched_candidate.is_empty()
    {
        return Err("assertion-density identity migration did not normalize line-count drift".to_string());
    }

    let retained = finding("b3:retained", "lattice", "retained finding");
    let resolved = finding_with_lint(
        "b3:resolved",
        "lattice",
        ALLOWED_RESOLVED_LINT,
        ALLOWED_RESOLVED_FILE,
        ALLOWED_RESOLVED_MESSAGE,
    );
    let resolved_old = serde_json::json!({"findings": [retained.clone(), resolved]});
    let resolved_candidate = serde_json::json!({"findings": [retained]});
    let resolved_migration = migrate(&resolved_old, &resolved_candidate)?;
    if !migration_is_accepted(&resolved_migration) || resolved_migration.findings.len() != 1 {
        return Err("exact resolved-finding allowlist was not accepted".to_string());
    }

    let unapproved_old = serde_json::json!({"findings": [finding("b3:unapproved", "lattice", "removed finding")]});
    let empty_candidate = serde_json::json!({"findings": []});
    let unapproved_migration = migrate(&unapproved_old, &empty_candidate)?;
    if migration_is_accepted(&unapproved_migration) {
        return Err("unapproved resolved finding was accepted".to_string());
    }
    Ok(())
}
