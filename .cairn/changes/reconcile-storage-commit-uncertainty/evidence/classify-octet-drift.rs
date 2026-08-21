#!/usr/bin/env -S nix develop . -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
serde_json = "=1.0.149"
---

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

use serde_json::Value;

const SUCCESS_EXIT_CODE: u8 = 0;
const FAILURE_EXIT_CODE: u8 = 1;

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
    let [_, baseline_path, results_path, status_path] = arguments.as_slice() else {
        return Err("usage: classify-octet-drift.rs BASELINE RESULTS_JSONL STATUS_JSON".to_string());
    };
    let baseline = read_json(baseline_path)?;
    let old = baseline
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline findings are missing".to_string())?;
    let current = read_json_lines(results_path)?;
    let status = read_json(status_path)?;
    let stale_ids = string_set(&status, "/baseline/stale_finding_ids")?;
    let new_ids = string_set(&status, "/baseline/new_finding_ids")?;
    let stale = old.iter().filter(|finding| stable_id_is_in(finding, &stale_ids)).collect::<Vec<_>>();
    let new = current.iter().filter(|finding| stable_id_is_in(finding, &new_ids)).collect::<Vec<_>>();
    let stale_by_key = group_by_semantic_key(&stale)?;
    let new_by_key = group_by_semantic_key(&new)?;
    let all_keys = stale_by_key.keys().chain(new_by_key.keys()).cloned().collect::<BTreeSet<_>>();
    let mut matched_count = 0_usize;
    let mut unmatched_new = Vec::new();
    let mut unmatched_stale = Vec::new();
    for key in all_keys {
        let old_group = stale_by_key.get(&key).cloned().unwrap_or_default();
        let new_group = new_by_key.get(&key).cloned().unwrap_or_default();
        let pair_count = old_group.len().min(new_group.len());
        matched_count =
            matched_count.checked_add(pair_count).ok_or_else(|| "matched finding count overflow".to_string())?;
        unmatched_stale.extend(old_group.into_iter().skip(pair_count));
        unmatched_new.extend(new_group.into_iter().skip(pair_count));
    }
    let retained_count = current.len().saturating_sub(new.len());
    println!(
        "old={} current={} retained={} new={} stale={} semantic-pairs={} unmatched-new={} unmatched-stale={}",
        old.len(),
        current.len(),
        retained_count,
        new.len(),
        stale.len(),
        matched_count,
        unmatched_new.len(),
        unmatched_stale.len()
    );
    print_lint_counts("unmatched-new-lint", &unmatched_new)?;
    print_lint_counts("unmatched-stale-lint", &unmatched_stale)?;
    for finding in unmatched_stale {
        print_finding("unmatched-stale", finding)?;
    }
    for finding in unmatched_new {
        print_finding("unmatched-new", finding)?;
    }
    Ok(())
}

fn string_set(value: &Value, pointer: &str) -> Result<BTreeSet<String>, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("string array is missing: {pointer}"))?
        .iter()
        .map(|item| item.as_str().map(str::to_string).ok_or_else(|| format!("non-string array item: {pointer}")))
        .collect()
}

fn stable_id_is_in(finding: &Value, identities: &BTreeSet<String>) -> bool {
    finding
        .get("stable_id")
        .and_then(Value::as_str)
        .is_some_and(|identity| identities.contains(identity))
}

fn group_by_semantic_key<'a>(findings: &[&'a Value]) -> Result<BTreeMap<String, Vec<&'a Value>>, String> {
    let mut groups = BTreeMap::<String, Vec<&Value>>::new();
    for finding in findings {
        groups.entry(semantic_key(finding)?).or_default().push(finding);
    }
    Ok(groups)
}

fn print_lint_counts(prefix: &str, findings: &[&Value]) -> Result<(), String> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for finding in findings {
        let count = counts.entry(field(finding, "lint")?).or_default();
        *count = count.checked_add(1).ok_or_else(|| "lint finding count overflow".to_string())?;
    }
    for (lint, count) in counts {
        println!("{prefix}\t{lint}\t{count}");
    }
    Ok(())
}

fn print_finding(prefix: &str, finding: &Value) -> Result<(), String> {
    println!(
        "{prefix}\t{}\t{}\t{}\t{}",
        field(finding, "lint")?,
        field(finding, "file")?,
        finding.get("line").and_then(Value::as_u64).unwrap_or_default(),
        field(finding, "message")?
    );
    Ok(())
}

fn semantic_key(finding: &Value) -> Result<String, String> {
    let lint = field(finding, "lint")?;
    let message = match lint {
        "excessive_file_length" => "source file exceeds the reviewed length limit".to_string(),
        "assertion_density" => field(finding, "message")?
            .split_once(" has ")
            .map(|(subject, _)| format!("{subject} has reviewed assertion-density debt"))
            .ok_or_else(|| "assertion-density message lacks subject".to_string())?,
        _ => field(finding, "message")?.to_string(),
    };
    Ok([
        lint.to_string(),
        field(finding, "group")?.to_string(),
        field(finding, "severity")?.to_string(),
        field(finding, "file")?.to_string(),
        message,
        field(finding, "crate_name")?.to_string(),
    ]
    .join("\u{1f}"))
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value.get(name).and_then(Value::as_str).ok_or_else(|| format!("finding field is missing: {name}"))
}

fn read_json(path: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {path}: {error}"))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {path}: {error}"))
}

fn read_json_lines(path: &str) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {path}: {error}"))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|error| format!("parse {path}: {error}")))
        .collect()
}
