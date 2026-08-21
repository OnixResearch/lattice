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
        return Err("usage: analyze-octet-drift.rs BASELINE RESULTS_JSONL STATUS_JSON".to_string());
    };
    let baseline = read_json(baseline_path)?;
    let old = baseline
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline findings are missing".to_string())?;
    let current = read_json_lines(results_path)?;
    let status = read_json(status_path)?;
    let stale_ids = status
        .pointer("/baseline/stale_finding_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| "stale finding identities are missing".to_string())?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let new_ids = status
        .pointer("/baseline/new_finding_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| "new finding identities are missing".to_string())?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let mut old_counts = BTreeMap::<String, usize>::new();
    for finding in old {
        *old_counts.entry(semantic_key(finding)?).or_default() += 1;
    }
    let mut unmatched = Vec::new();
    for finding in &current {
        if !new_ids.contains(field(finding, "stable_id")?) {
            continue;
        }
        let key = semantic_key(finding)?;
        match old_counts.get_mut(&key) {
            Some(count) if *count > 0 => *count -= 1,
            _ => unmatched.push(finding),
        }
    }
    println!(
        "old={} current={} new={} unmatched-new={}",
        old.len(),
        current.len(),
        new_ids.len(),
        unmatched.len()
    );
    for finding in unmatched {
        println!(
            "unmatched\t{}\t{}\t{}\t{}",
            field(finding, "lint")?,
            field(finding, "file")?,
            finding.get("line").and_then(Value::as_u64).unwrap_or_default(),
            field(finding, "message")?,
        );
    }
    for finding in &current {
        if new_ids.contains(field(finding, "stable_id")?) {
            print_finding("new", finding)?;
        }
    }
    for finding in old {
        if stale_ids.contains(field(finding, "stable_id")?) {
            print_finding("stale", finding)?;
        }
    }
    Ok(())
}

fn print_finding(prefix: &str, finding: &Value) -> Result<(), String> {
    println!(
        "{prefix}\t{}\t{}\t{}\t{}",
        field(finding, "lint")?,
        field(finding, "file")?,
        finding.get("line").and_then(Value::as_u64).unwrap_or_default(),
        field(finding, "message")?,
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
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("finding field is missing: {name}"))
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
