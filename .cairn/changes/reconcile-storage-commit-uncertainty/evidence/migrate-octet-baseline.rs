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
const ROOT_SCOPE: &str = "root";
const CORE_SCOPE: &str = "core";
const ROOT_BASELINE_TOTAL: usize = 846;
const ROOT_CONTROL_EXTRA_TOTAL: usize = 232;
const ROOT_CURRENT_EXTRA_TOTAL: usize = 231;
const CORE_BASELINE_TOTAL: usize = 23;
const CORE_CONTROL_EXTRA_TOTAL: usize = 8;
const CORE_CURRENT_EXTRA_TOTAL: usize = 8;
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

#[derive(Clone, Copy)]
struct MigrationExpectations {
    baseline_total: usize,
    control_extra_total: usize,
    current_extra_total: usize,
}

impl MigrationExpectations {
    fn improvement_total(self) -> usize {
        self.control_extra_total.saturating_sub(self.current_extra_total)
    }
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
    let arguments = env::args().collect::<Vec<_>>();
    if arguments.get(1).map(String::as_str) == Some(SELF_TEST_ARGUMENT) {
        self_test()?;
        println!("Octet baseline migration self-test passed");
        return Ok(());
    }
    if arguments.len() != 7 && arguments.len() != 8 {
        return Err(format!(
            "usage: {} SCOPE OLD_BASELINE CURRENT_RESULTS CONTROL_RESULTS STATUS OUTPUT [{EXECUTE_ARGUMENT}] | {SELF_TEST_ARGUMENT}",
            arguments[0]
        ));
    }
    let execute = arguments.len() == 8 && arguments[7] == EXECUTE_ARGUMENT;
    if arguments.len() == 8 && !execute {
        return Err(format!("unsupported argument: {}", arguments[7]));
    }
    let scope = expectations(&arguments[1])?;
    let old = read_json(Path::new(&arguments[2]))?;
    let current = read_json_lines(Path::new(&arguments[3]))?;
    let control = read_json_lines(Path::new(&arguments[4]))?;
    let status = read_json(Path::new(&arguments[5]))?;
    let migrated = migrate_checked(&old, &current, &control, scope)?;
    println!(
        "reviewed Octet migration: baseline={} migrated={} control-extra={} current-extra={} improvements={} baseline-growth=0",
        scope.baseline_total,
        migrated.len(),
        scope.control_extra_total,
        scope.current_extra_total,
        scope.improvement_total()
    );
    if execute {
        write_baseline(Path::new(&arguments[6]), &old, &status, &migrated)?;
    }
    Ok(())
}

fn migrate_checked(
    old: &Value,
    current: &[Value],
    control: &[Value],
    scope: MigrationExpectations,
) -> Result<Vec<Value>, String> {
    let old_findings = findings(old)?;
    if old_findings.len() != scope.baseline_total {
        return Err(format!("old baseline total is {}, expected {}", old_findings.len(), scope.baseline_total));
    }
    let (migrated, current_extra) = match_baseline(old_findings, current)?;
    let (_control_migrated, control_extra) = match_baseline(old_findings, control)?;
    if migrated.len() != scope.baseline_total {
        return Err(format!("migrated total is {}, expected {}", migrated.len(), scope.baseline_total));
    }
    if control_extra.len() != scope.control_extra_total {
        return Err(format!("control extra total is {}, expected {}", control_extra.len(), scope.control_extra_total));
    }
    if current_extra.len() != scope.current_extra_total {
        return Err(format!("current extra total is {}, expected {}", current_extra.len(), scope.current_extra_total));
    }
    verify_strict_improvement(&control_extra, &current_extra, scope.improvement_total())?;
    Ok(migrated)
}

fn match_baseline(old: &[Value], current: &[Value]) -> Result<(Vec<Value>, Vec<Value>), String> {
    let old_counts = semantic_counts(old)?;
    let mut current_by_key = group_by_semantic_key(current)?;
    let mut migrated = Vec::<Value>::with_capacity(old.len());
    let mut migrated_ids = BTreeSet::<String>::new();
    let mut extras = Vec::<Value>::new();
    for (key, required_count) in old_counts {
        let mut candidates = current_by_key
            .remove(&key)
            .ok_or_else(|| format!("baseline finding has no current semantic peer: {key}"))?;
        candidates.sort_by(|left, right| {
            let left_id = left.get("stable_id").and_then(Value::as_str).unwrap_or_default();
            let right_id = right.get("stable_id").and_then(Value::as_str).unwrap_or_default();
            left_id.cmp(right_id)
        });
        let mut selected_count = 0_usize;
        for candidate in candidates {
            let identity = string_field(&candidate, "stable_id")?.to_string();
            let can_select = selected_count < required_count && !migrated_ids.contains(&identity);
            if can_select {
                migrated_ids.insert(identity);
                migrated.push(candidate);
                selected_count =
                    selected_count.checked_add(1).ok_or_else(|| "selected finding count overflow".to_string())?;
            } else {
                extras.push(candidate);
            }
        }
        if selected_count != required_count {
            return Err(format!(
                "baseline semantic peers are not distinct: key={key} selected={selected_count} required={required_count}"
            ));
        }
    }
    extras.extend(current_by_key.into_values().flatten());
    Ok((migrated, extras))
}

fn verify_strict_improvement(
    control: &[Value],
    current: &[Value],
    expected_improvement_total: usize,
) -> Result<(), String> {
    let mut control_counts = semantic_counts(control)?;
    for finding in current {
        let key = semantic_key(finding)?;
        let count = control_counts
            .get_mut(&key)
            .ok_or_else(|| format!("current semantic growth has no control peer: {key}"))?;
        if *count == 0 {
            return Err(format!("current semantic growth exceeds control peers: {key}"));
        }
        *count -= 1;
    }
    let improvements = control_counts.values().copied().sum::<usize>();
    if improvements != expected_improvement_total {
        return Err(format!("improvement total is {improvements}, expected {expected_improvement_total}"));
    }
    Ok(())
}

fn group_by_semantic_key(values: &[Value]) -> Result<BTreeMap<String, Vec<Value>>, String> {
    let mut groups = BTreeMap::<String, Vec<Value>>::new();
    for value in values {
        groups.entry(semantic_key(value)?).or_default().push(value.clone());
    }
    Ok(groups)
}

fn semantic_counts(values: &[Value]) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        let count = counts.entry(semantic_key(value)?).or_default();
        *count = count.checked_add(1).ok_or_else(|| "semantic finding count overflow".to_string())?;
    }
    Ok(counts)
}

fn semantic_key(finding: &Value) -> Result<String, String> {
    let lint = string_field(finding, "lint")?;
    let message = semantic_message(lint, string_field(finding, "message")?)?;
    Ok([
        lint.to_string(),
        string_field(finding, "group")?.to_string(),
        string_field(finding, "severity")?.to_string(),
        string_field(finding, "file")?.to_string(),
        message,
        string_field(finding, "crate_name")?.to_string(),
    ]
    .join("\u{1f}"))
}

fn semantic_message(lint: &str, message: &str) -> Result<String, String> {
    match lint {
        "excessive_file_length" => Ok("source file exceeds the reviewed length limit".to_string()),
        "module_file_count" => Ok("module directory exceeds the reviewed file-count limit".to_string()),
        "assertion_density" => message
            .split_once(" has ")
            .map(|(subject, _)| format!("{subject} has reviewed assertion-density debt"))
            .ok_or_else(|| "assertion-density message lacks a subject".to_string()),
        _ => Ok(message.to_string()),
    }
}

fn write_baseline(path: &Path, old: &Value, status: &Value, findings: &[Value]) -> Result<(), String> {
    let projected = findings.iter().map(project).collect::<Result<Vec<_>, _>>()?;
    let identities = findings
        .iter()
        .map(|finding| string_field(finding, "stable_id").map(str::to_string))
        .collect::<Result<BTreeSet<_>, _>>()?;
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
    let payload = serde_json::json!({
        "schema_version": BASELINE_SCHEMA_VERSION,
        "stable_ids": ids,
    });
    format!("b3:{}", blake3::hash(payload.to_string().as_bytes()).to_hex())
}

fn self_test() -> Result<(), String> {
    let old = vec![fixture(
        "old",
        "module_file_count",
        "module directory `src` contains 16 Rust source files (limit: 7)",
    )];
    let current = vec![
        fixture("new", "module_file_count", "module directory `src` contains 17 Rust source files (limit: 7)"),
        fixture("extra", "non_trait_imports", "pre-existing warning"),
    ];
    let (migrated, extras) = match_baseline(&old, &current)?;
    let migrated_new = migrated.iter().any(|finding| string_field(finding, "stable_id") == Ok("new"));
    if migrated.len() != 1 || extras.len() != 1 || !migrated_new {
        return Err("positive migration fixture failed".to_string());
    }
    let growth = vec![fixture("growth", "numeric_units", "new semantic finding")];
    if verify_strict_improvement(&[], &growth, 0).is_ok() {
        return Err("negative semantic-growth fixture was accepted".to_string());
    }
    Ok(())
}

fn expectations(scope: &str) -> Result<MigrationExpectations, String> {
    match scope {
        ROOT_SCOPE => Ok(MigrationExpectations {
            baseline_total: ROOT_BASELINE_TOTAL,
            control_extra_total: ROOT_CONTROL_EXTRA_TOTAL,
            current_extra_total: ROOT_CURRENT_EXTRA_TOTAL,
        }),
        CORE_SCOPE => Ok(MigrationExpectations {
            baseline_total: CORE_BASELINE_TOTAL,
            control_extra_total: CORE_CONTROL_EXTRA_TOTAL,
            current_extra_total: CORE_CURRENT_EXTRA_TOTAL,
        }),
        _ => Err(format!("unsupported migration scope: {scope}")),
    }
}

fn fixture(id: &str, lint: &str, message: &str) -> Value {
    serde_json::json!({
        "stable_id": id,
        "semantic_id": id,
        "identity_status": "computed",
        "lint": lint,
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
    value
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline is missing findings".to_string())
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("value is missing string {field}"))
}

fn integer_field(value: &Value, field: &str) -> Result<u64, String> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| format!("value is missing integer {field}"))
}
