use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

const EXECUTE_ARG: &str = "--execute";
const BASELINE_SCHEMA_VERSION: u64 = 1;
const DIAGNOSTIC_LIMIT: usize = 20;
const SEMANTIC_FIELDS: &[&str] = &[
    "identity_status",
    "lint",
    "group",
    "severity",
    "file",
    "message",
    "crate_name",
];
const EXCESSIVE_FILE_LENGTH_LINT: &str = "excessive_file_length";
const ASSERTION_DENSITY_LINT: &str = "assertion_density";
const FUNCTION_LENGTH_LINT: &str = "function_length";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Migration<'a> {
    findings: &'a [Value],
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

#[derive(Debug, Serialize, PartialEq, Eq)]
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

// r[impl lattice.repository_validation.octet_baseline_migration]
pub fn run(args: &[String]) -> Result<(), String> {
    if !(7..=8).contains(&args.len()) {
        return Err(
            "usage: octet-baseline-migrate OLD_BASELINE REFERENCE_RESULTS_JSONL REFERENCE_STATUS_JSON RESULTS_JSONL STATUS_JSON OUTPUT [--execute]".to_string()
        );
    }
    let execute = args.len() == 8;
    if execute && args[7] != EXECUTE_ARG {
        return Err(format!("unsupported argument: {}", args[7]));
    }
    let old = read_json(Path::new(&args[1]))?;
    let reference_findings = read_json_lines(Path::new(&args[2]))?;
    let reference_status = read_json(Path::new(&args[3]))?;
    let current_findings = read_json_lines(Path::new(&args[4]))?;
    let status = read_json(Path::new(&args[5]))?;
    if grouped(&reference_findings)? != grouped(&current_findings)? {
        return Err("complete raw warning multisets differ; refusing to rebaseline".to_string());
    }
    let reference = candidate_baseline(&old, reference_findings, &reference_status)?;
    let candidate = candidate_baseline(&old, current_findings, &status)?;
    if grouped(findings(&old)?)? != grouped(findings(&reference)?)? {
        return Err("reference observation does not preserve the accepted warning facts".to_string());
    }
    let migration = migrate(&old, &candidate)?;
    print_summary(&migration);
    if !migration.unmatched_old.is_empty() || !migration.unmatched_candidate.is_empty() {
        return Err(
            "semantic finding multisets differ outside the selected bounded policy; refusing to rebaseline".to_string()
        );
    }
    if execute {
        write_migrated_baseline(Path::new(&args[6]), &old, &status, migration.findings)?;
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
    if status.pointer("/cargo_process_exit/classification").and_then(Value::as_str) != Some("success")
        || status.pointer("/cargo_process_exit/code").and_then(Value::as_u64) != Some(0)
    {
        return Err("candidate observation did not complete successfully".to_string());
    }
    if u64::try_from(findings.len()).ok() != Some(required_u64(status, "total_findings")?) {
        return Err("candidate observation is incomplete: results count differs from status".to_string());
    }
    let old_metadata = old.get("metadata").ok_or("old baseline is missing metadata")?;
    let metadata = status.get("metadata").ok_or("status is missing metadata")?;
    for field in [
        "tool_name",
        "tool_version",
        "rustc_version",
        "toolchain",
        "profile_name",
        "profile_hash",
        "config_hash",
    ] {
        if required_string(old_metadata, field)? != required_string(metadata, field)? {
            return Err(format!("candidate metadata differs: {field}"));
        }
    }
    let config_hash = metadata.get("config_hash");
    let mut unique = BTreeMap::new();
    for finding in findings {
        if finding.get("config_hash") != config_hash {
            return Err("raw finding config hash differs from observation metadata".to_string());
        }
        project_finding(&finding)?;
        let stable_id = required_string(&finding, "stable_id")?;
        unique.entry(stable_id).or_insert(finding);
    }
    let findings = unique.into_values().collect::<Vec<_>>();
    let schema_version = old
        .get("schema_version")
        .cloned()
        .ok_or_else(|| "old baseline is missing schema_version".to_string())?;
    let finding_schema_version = old
        .get("finding_schema_version")
        .cloned()
        .ok_or_else(|| "old baseline is missing finding_schema_version".to_string())?;
    Ok(serde_json::json!({
        "schema_version": schema_version,
        "finding_schema_version": finding_schema_version,
        "metadata": metadata,
        "baseline_hash": "",
        "findings": findings,
    }))
}

fn write_migrated_baseline(path: &Path, old: &Value, status: &Value, findings: &[Value]) -> Result<(), String> {
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
    serde_json::to_string(&parts).map_err(|error| error.to_string())
}

fn normalized_message(lint: &str, message: &str) -> String {
    match lint {
        EXCESSIVE_FILE_LENGTH_LINT => {
            normalize_count(message, "source file is ", " lines (limit: ", ")").unwrap_or_else(|| message.to_string())
        }
        ASSERTION_DENSITY_LINT => normalize_density(message).unwrap_or_else(|| message.to_string()),
        FUNCTION_LENGTH_LINT => message
            .split_once(" is ")
            .and_then(|(function, rest)| {
                normalize_count(rest, "", " lines (limit: ", ")").map(|count| format!("{function} is {count}"))
            })
            .unwrap_or_else(|| message.to_string()),
        "ambiguous_params" => normalize_ambiguous_params_message(message).unwrap_or_else(|| message.to_string()),
        "numeric_units" => normalize_numeric_units_message(message).unwrap_or_else(|| message.to_string()),
        _ => message.to_string(),
    }
}

// Ignore only the observed source-line count; configured limits remain warning facts.
fn normalize_count(message: &str, prefix: &str, middle: &str, suffix: &str) -> Option<String> {
    let (count, limit) = message.strip_prefix(prefix)?.strip_suffix(suffix)?.split_once(middle)?;
    count.parse::<u64>().ok()?;
    limit.parse::<u64>().ok()?;
    Some(format!("{prefix}<lines>{middle}{limit}{suffix}"))
}

fn normalize_density(message: &str) -> Option<String> {
    let (function_and_assertions, rest) = message.split_once(" assertion(s) in ")?;
    let (function, assertions) = function_and_assertions.rsplit_once(" has ")?;
    assertions.parse::<u64>().ok()?;
    let count = normalize_count(rest, "", " lines (need ", "+)")?;
    Some(format!("{function} has {assertions} assertion(s) in {count}"))
}

// The reviewed Octet transition added/removed only `consecutive`, not count or type.
fn normalize_ambiguous_params_message(message: &str) -> Option<String> {
    let suffix = " parameters are easy to swap by accident";
    let (count, type_part) = message.strip_suffix(suffix)?.split_once(' ')?;
    if count.parse::<usize>().ok()? == 0 {
        return None;
    }
    let type_part = type_part.strip_prefix("consecutive ").unwrap_or(type_part);
    let type_name = type_part.strip_prefix('`')?.strip_suffix('`')?;
    if type_name.is_empty() || type_name.contains('`') {
        return None;
    }
    Some(format!("{count} `{type_name}`{suffix}"))
}

fn normalize_numeric_units_message(message: &str) -> Option<String> {
    let old_suffix = " looks like a quantity but has no unit suffix";
    let new_suffix = " looks like a raw numeric quantity but has no unit suffix";
    let name_part = message.strip_suffix(old_suffix).or_else(|| message.strip_suffix(new_suffix))?;
    let name = name_part.strip_prefix('`')?.strip_suffix('`')?;
    if name.is_empty() || name.contains('`') {
        return None;
    }
    Some(format!("`{name}`{old_suffix}"))
}

fn grouped(findings: &[Value]) -> Result<BTreeMap<String, usize>, String> {
    let mut groups = BTreeMap::<String, usize>::new();
    for finding in findings {
        *groups.entry(semantic_key(finding)?).or_default() += 1;
    }
    Ok(groups)
}

fn migrate<'a>(old: &Value, candidate: &'a Value) -> Result<Migration<'a>, String> {
    let old_findings = findings(old)?;
    let candidate_findings = findings(candidate)?;
    let old_groups = grouped(old_findings)?;
    let candidate_groups = grouped(candidate_findings)?;
    let mut unmatched_old = Vec::new();
    let mut unmatched_candidate = Vec::new();
    for (key, old_count) in &old_groups {
        let candidate_count = candidate_groups.get(key).copied().unwrap_or(0);
        if candidate_count != *old_count {
            unmatched_old.push(format!("count mismatch old={old_count} candidate={candidate_count} key={key}"));
        }
    }
    for (key, count) in &candidate_groups {
        if !old_groups.contains_key(key) {
            unmatched_candidate.push(format!("new count={count} key={key}"));
        }
    }
    Ok(Migration {
        findings: candidate_findings,
        old_count: old_findings.len(),
        candidate_count: candidate_findings.len(),
        unmatched_old,
        unmatched_candidate,
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

fn print_summary(migration: &Migration) {
    println!(
        "old={} candidate={} unmatched_old={} unmatched_candidate={}",
        migration.old_count,
        migration.candidate_count,
        migration.unmatched_old.len(),
        migration.unmatched_candidate.len()
    );
    println!("Complete raw warning multiplicities and canonical first-per-stable-ID baseline facts are preserved.");
    for diagnostic in migration.unmatched_old.iter().take(DIAGNOSTIC_LIMIT) {
        println!("OLD: {diagnostic}");
    }
    for diagnostic in migration.unmatched_candidate.iter().take(DIAGNOSTIC_LIMIT) {
        println!("NEW: {diagnostic}");
    }
}
