#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-credential-octet-migration-target nix shell "github:nix-community/fenix?rev=4e49ef62eedaa5c149d62b63b3f53844e1cc45d7#complete.toolchain" nixpkgs#gcc nixpkgs#clang nixpkgs#mold -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.3"
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.149"
---

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
const EXPECTED_OLD_COUNT: usize = 1163;
const EXPECTED_CURRENT_COUNT: usize = 1162;
const WORKFLOW_FORMAT_PATH: &str = "src/workflow_format.rs";
const ASSERTION_DENSITY_LINT: &str = "assertion_density";
const RESOLVED_FUNCTION: &str = "portable_config_to_node_credentials";
const RESOLVED_OLD_ID: &str = "b3:7e68c4333e08f1b6";

#[derive(Debug, Clone, Copy)]
struct Replacement {
    old_id: &'static str,
    new_id: &'static str,
}

const REPLACEMENTS: &[Replacement] = &[
    Replacement {
        old_id: "b3:89dd760ba38a770f",
        new_id: "b3:ba3db39e4d4104db",
    },
    Replacement {
        old_id: "b3:1d324a6e565587d3",
        new_id: "b3:8b42e355acc3d372",
    },
    Replacement {
        old_id: "b3:8c5cc5f0f75ac680",
        new_id: "b3:fae970a39dccf654",
    },
    Replacement {
        old_id: "b3:09bf4440ac412549",
        new_id: "b3:12241547ca6aac0a",
    },
    Replacement {
        old_id: "b3:bb3bac2c4b48a040",
        new_id: "b3:4da36e4ff44a51df",
    },
    Replacement {
        old_id: "b3:bddd79e8de8719ae",
        new_id: "b3:2972b9933407d71d",
    },
];

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
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments.len() == SELF_TEST_ARGUMENT_COUNT && arguments[1] == SELF_TEST_ARGUMENT {
        self_test()?;
        println!("credential Octet baseline migration self-test passed");
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
    let current = read_json_lines(Path::new(&arguments[2]))?;
    let status = read_json(Path::new(&arguments[3]))?;
    let old_findings = findings(&old)?;
    let issues = validate_migration(old_findings, &current, &status, REPLACEMENTS, RESOLVED_OLD_ID)?;
    if !issues.is_empty() {
        return Err(format!("Octet migration refused: {}", issues.join("; ")));
    }
    println!(
        "old={} current={} replacements={} resolved=1",
        old_findings.len(),
        current.len(),
        REPLACEMENTS.len()
    );
    if execute {
        write_baseline(Path::new(&arguments[4]), &old, &status, current)?;
    }
    Ok(())
}

fn validate_migration(
    old: &[Value],
    current: &[Value],
    status: &Value,
    replacements: &[Replacement],
    resolved_id: &str,
) -> Result<Vec<String>, String> {
    let mut issues = Vec::new();
    let old_ids = finding_ids(old)?;
    let current_ids = finding_ids(current)?;
    let replacement_old = replacements.iter().map(|replacement| replacement.old_id.to_string()).collect::<BTreeSet<_>>();
    let replacement_new = replacements.iter().map(|replacement| replacement.new_id.to_string()).collect::<BTreeSet<_>>();
    let mut expected_stale = replacement_old.clone();
    expected_stale.insert(resolved_id.to_string());
    let expected_new = replacement_new.clone();
    let observed_stale = status_ids(status, "stale_finding_ids")?;
    let observed_new = status_ids(status, "new_finding_ids")?;

    if old.len() != EXPECTED_OLD_COUNT {
        issues.push(format!("old count {} != {EXPECTED_OLD_COUNT}", old.len()));
    }
    if current.len() != EXPECTED_CURRENT_COUNT {
        issues.push(format!("current count {} != {EXPECTED_CURRENT_COUNT}", current.len()));
    }
    if observed_stale != expected_stale {
        issues.push("status stale set differs from exact allowlist".to_string());
    }
    if observed_new != expected_new {
        issues.push("status new set differs from exact allowlist".to_string());
    }

    let mut expected_current = old_ids.clone();
    for stale_id in &expected_stale {
        expected_current.remove(stale_id);
    }
    expected_current.extend(expected_new);
    if current_ids != expected_current {
        issues.push("current finding IDs are not the exact allowlisted transformation".to_string());
    }

    for replacement in replacements {
        let old_finding = match finding_by_id(old, replacement.old_id) {
            Ok(finding) => finding,
            Err(error) => {
                issues.push(error);
                continue;
            }
        };
        let new_finding = match finding_by_id(current, replacement.new_id) {
            Ok(finding) => finding,
            Err(error) => {
                issues.push(error);
                continue;
            }
        };
        if semantic_shape(old_finding)? != semantic_shape(new_finding)? {
            issues.push(format!("semantic shape changed for {}", replacement.old_id));
        }
    }

    let resolved = match finding_by_id(old, resolved_id) {
        Ok(finding) => finding,
        Err(error) => {
            issues.push(error);
            return Ok(issues);
        }
    };
    if required_string(resolved, "lint")? != ASSERTION_DENSITY_LINT
        || required_string(resolved, "file")? != WORKFLOW_FORMAT_PATH
        || !required_string(resolved, "message")?.contains(RESOLVED_FUNCTION)
    {
        issues.push("resolved finding does not match the reviewed assertion-density debt".to_string());
    }
    if current.iter().any(|finding| {
        finding.get("lint").and_then(Value::as_str) == Some(ASSERTION_DENSITY_LINT)
            && finding.get("file").and_then(Value::as_str) == Some(WORKFLOW_FORMAT_PATH)
            && finding
                .get("message")
                .and_then(Value::as_str)
                .is_some_and(|message| message.contains(RESOLVED_FUNCTION))
    }) {
        issues.push("resolved assertion-density finding remains current".to_string());
    }

    issues.sort();
    issues.dedup();
    Ok(issues)
}

fn semantic_shape(finding: &Value) -> Result<Vec<String>, String> {
    Ok(vec![
        required_string(finding, "identity_status")?,
        required_string(finding, "lint")?,
        required_string(finding, "group")?,
        required_string(finding, "severity")?,
        required_string(finding, "file")?,
        normalize_numeric_runs(&required_string(finding, "message")?),
        required_string(finding, "crate_name")?,
    ])
}

fn normalize_numeric_runs(message: &str) -> String {
    let mut output = String::with_capacity(message.len());
    let mut in_numeric_run = false;
    for character in message.chars() {
        if character.is_ascii_digit() {
            if !in_numeric_run {
                output.push('#');
            }
            in_numeric_run = true;
        } else {
            in_numeric_run = false;
            output.push(character);
        }
    }
    output
}

fn finding_ids(findings: &[Value]) -> Result<BTreeSet<String>, String> {
    findings.iter().map(|finding| required_string(finding, "stable_id")).collect()
}

fn finding_by_id<'a>(findings: &'a [Value], stable_id: &str) -> Result<&'a Value, String> {
    let matches = findings
        .iter()
        .filter(|finding| finding.get("stable_id").and_then(Value::as_str) == Some(stable_id))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [finding] => Ok(*finding),
        [] => Err(format!("missing finding {stable_id}")),
        _ => Err(format!("reviewed finding ID is ambiguous: {stable_id}")),
    }
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

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|error| format!("invalid JSONL: {error}")))
        .collect()
}

fn findings(value: &Value) -> Result<&[Value], String> {
    value
        .get("findings")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| "baseline is missing findings".to_string())
}

fn write_baseline(path: &Path, old: &Value, status: &Value, mut findings: Vec<Value>) -> Result<(), String> {
    findings.sort_by_key(location_key);
    let projected = findings.iter().map(project_finding).collect::<Result<Vec<_>, _>>()?;
    let stable_ids = projected.iter().map(|finding| finding.stable_id.clone()).collect::<Vec<_>>();
    let output = BaselineOutput {
        schema_version: required_u64(old, "schema_version")?,
        finding_schema_version: required_u64(old, "finding_schema_version")?,
        metadata: status.get("metadata").cloned().ok_or_else(|| "status is missing metadata".to_string())?,
        baseline_hash: baseline_hash(&stable_ids),
        findings: projected,
    };
    let mut text = serde_json::to_string_pretty(&output).map_err(|error| format!("serialize baseline: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

fn location_key(finding: &Value) -> (String, u64, u64, String) {
    (
        finding.get("file").and_then(Value::as_str).unwrap_or_default().to_string(),
        finding.get("line").and_then(Value::as_u64).unwrap_or_default(),
        finding.get("column").and_then(Value::as_u64).unwrap_or_default(),
        finding.get("stable_id").and_then(Value::as_str).unwrap_or_default().to_string(),
    )
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

fn demo_finding(id: &str, lint: &str, message: &str) -> Value {
    serde_json::json!({
        "stable_id": id,
        "semantic_id": id,
        "identity_status": "computed",
        "lint": lint,
        "group": "demo::group",
        "severity": "error",
        "file": "src/demo.rs",
        "line": 1,
        "column": 1,
        "message": message,
        "crate_name": "lattice",
    })
}

fn self_test() -> Result<(), String> {
    let pair = Replacement {
        old_id: "b3:old",
        new_id: "b3:new",
    };
    let old = vec![
        demo_finding(pair.old_id, "function_length", "function `demo` is 10 lines"),
        demo_finding("b3:resolved", ASSERTION_DENSITY_LINT, "function `portable_config_to_node_credentials` has 0 assertions"),
    ];
    let current = vec![demo_finding(pair.new_id, "function_length", "function `demo` is 11 lines")];
    let status = serde_json::json!({
        "baseline": {
            "stale_finding_ids": [pair.old_id, "b3:resolved"],
            "new_finding_ids": [pair.new_id],
        }
    });
    let positive = validate_migration_with_counts(&old, &current, &status, &[pair], "b3:resolved", old.len(), current.len())?;
    if !positive.is_empty() {
        return Err(format!("positive self-test failed: {positive:?}"));
    }

    let mut unexpected = current.clone();
    unexpected.push(demo_finding("b3:unexpected", "demo_lint", "unexpected"));
    let negative = validate_migration_with_counts(
        &old,
        &unexpected,
        &status,
        &[pair],
        "b3:resolved",
        old.len(),
        unexpected.len(),
    )?;
    if !negative.iter().any(|issue| issue.contains("exact allowlisted transformation")) {
        return Err(format!("negative self-test accepted unexpected finding: {negative:?}"));
    }
    Ok(())
}

fn validate_migration_with_counts(
    old: &[Value],
    current: &[Value],
    status: &Value,
    replacements: &[Replacement],
    resolved_id: &str,
    expected_old_count: usize,
    expected_current_count: usize,
) -> Result<Vec<String>, String> {
    let mut issues = validate_migration_core(old, current, status, replacements, resolved_id)?;
    if old.len() != expected_old_count {
        issues.push("self-test old count mismatch".to_string());
    }
    if current.len() != expected_current_count {
        issues.push("self-test current count mismatch".to_string());
    }
    issues.sort();
    issues.dedup();
    Ok(issues)
}

fn validate_migration_core(
    old: &[Value],
    current: &[Value],
    status: &Value,
    replacements: &[Replacement],
    resolved_id: &str,
) -> Result<Vec<String>, String> {
    let old_ids = finding_ids(old)?;
    let current_ids = finding_ids(current)?;
    let replacement_old = replacements.iter().map(|replacement| replacement.old_id.to_string()).collect::<BTreeSet<_>>();
    let replacement_new = replacements.iter().map(|replacement| replacement.new_id.to_string()).collect::<BTreeSet<_>>();
    let mut expected_stale = replacement_old.clone();
    expected_stale.insert(resolved_id.to_string());
    let observed_stale = status_ids(status, "stale_finding_ids")?;
    let observed_new = status_ids(status, "new_finding_ids")?;
    let mut issues = Vec::new();
    if observed_stale != expected_stale {
        issues.push("status stale set differs from exact allowlist".to_string());
    }
    if observed_new != replacement_new {
        issues.push("status new set differs from exact allowlist".to_string());
    }
    let mut expected_current = old_ids;
    for stale_id in expected_stale {
        expected_current.remove(&stale_id);
    }
    expected_current.extend(replacement_new);
    if current_ids != expected_current {
        issues.push("current finding IDs are not the exact allowlisted transformation".to_string());
    }
    for replacement in replacements {
        if let (Ok(old_finding), Ok(new_finding)) = (
            finding_by_id(old, replacement.old_id),
            finding_by_id(current, replacement.new_id),
        ) && semantic_shape(old_finding)? != semantic_shape(new_finding)?
        {
            issues.push(format!("semantic shape changed for {}", replacement.old_id));
        }
    }
    issues.sort();
    issues.dedup();
    Ok(issues)
}
