#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lti nix shell "github:nix-community/fenix?rev=7b33c6466f781cd699fe250c5b69dc4193da67a7#minimal.toolchain" nixpkgs#nickel nixpkgs#gcc nixpkgs#clang -c cargo -q -Zscript
---cargo
[package]
name = "traceability-inventory"
version = "0.1.0"
edition = "2024"

[dependencies]
blake3 = "1.8.3"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
---

#[path = "traceability_inventory/core.rs"]
mod core;

use core::CoverageReceipt;
use core::InventoryReport;
use core::ReviewPolicy;
use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;

const COMMAND_GENERATE: &str = "generate";
const COMMAND_CHECK: &str = "check";
const COMMAND_HASH: &str = "hash";
const EXPECTED_ARGUMENT_COUNT: usize = 2;
const COMMAND_ARGUMENT_INDEX: usize = 0;
const COVERAGE_ARGUMENT_INDEX: usize = 1;
const FAILURE_EXIT_CODE: u8 = 1;
const KIBIBYTE_BYTES: u64 = 1_024;
const MAX_INPUT_MEBIBYTES: u64 = 4;
const MAX_INPUT_BYTES: u64 = MAX_INPUT_MEBIBYTES * KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const NICKEL_PROGRAM: &str = "nickel";
const POLICY_PATH: &str = "cairn-policy/traceability-inventory.ncl";
const OUTPUT_PATH: &str = "cairn-policy/generated/traceability-debt-inventory.json";
const TEMP_OUTPUT_PATH: &str = "cairn-policy/generated/traceability-debt-inventory.json.tmp";

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("traceability inventory failed: {error}");
            ExitCode::from(FAILURE_EXIT_CODE)
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    if args.len() != EXPECTED_ARGUMENT_COUNT {
        return Err(format!("expected `{COMMAND_GENERATE}|{COMMAND_CHECK}|{COMMAND_HASH} <coverage-json>`"));
    }
    let coverage_path = safe_input_path(&args[COVERAGE_ARGUMENT_INDEX])?;
    let coverage = load_coverage(coverage_path)?;
    match args[COMMAND_ARGUMENT_INDEX].as_str() {
        COMMAND_HASH => Ok(core::missing_set_hash(&coverage.missing)),
        COMMAND_GENERATE => {
            let report = current_report(&coverage)?;
            write_report(&report)?;
            Ok(format!("wrote {OUTPUT_PATH} ({})", report.inventory_hash))
        }
        COMMAND_CHECK => {
            let expected = report_bytes(&current_report(&coverage)?)?;
            let actual = read_bounded(Path::new(OUTPUT_PATH))?;
            if actual != expected {
                return Err("generated inventory is stale; run the generate command".to_string());
            }
            let report: InventoryReport =
                serde_json::from_slice(&actual).map_err(|_| "generated inventory JSON is malformed".to_string())?;
            let issues = core::verify_report(&report);
            if !issues.is_empty() {
                return Err(format!("generated inventory is invalid: {}", issue_codes(&issues)));
            }
            Ok(format!("checked {OUTPUT_PATH} ({})", report.inventory_hash))
        }
        _ => Err(format!("expected `{COMMAND_GENERATE}|{COMMAND_CHECK}|{COMMAND_HASH} <coverage-json>`")),
    }
}

fn current_report(coverage: &CoverageReceipt) -> Result<InventoryReport, String> {
    let policy = load_review_policy()?;
    let observed_evidence = load_observed_evidence(&policy)?;
    let report = core::build_inventory(coverage, &policy, &observed_evidence);
    if !report.valid {
        return Err(format!("inventory inputs are invalid: {}", issue_codes(&report.issues)));
    }
    let verification_issues = core::verify_report(&report);
    if !verification_issues.is_empty() {
        return Err(format!("inventory report is invalid: {}", issue_codes(&verification_issues)));
    }
    Ok(report)
}

fn load_coverage(path: &Path) -> Result<CoverageReceipt, String> {
    let bytes = read_bounded(path)?;
    serde_json::from_slice(&bytes).map_err(|_| "coverage JSON is malformed".to_string())
}

fn load_review_policy() -> Result<ReviewPolicy, String> {
    let output = Command::new(NICKEL_PROGRAM)
        .args(["export", "--format", "json", POLICY_PATH])
        .output()
        .map_err(|_| "Nickel review policy export could not start".to_string())?;
    if !output.status.success() {
        return Err("Nickel review policy export failed".to_string());
    }
    if u64::try_from(output.stdout.len()).map_or(true, |length| length > MAX_INPUT_BYTES) {
        return Err("Nickel review policy export exceeded the input bound".to_string());
    }
    serde_json::from_slice(&output.stdout).map_err(|_| "Nickel review policy export was not valid JSON".to_string())
}

fn load_observed_evidence(policy: &ReviewPolicy) -> Result<BTreeMap<String, String>, String> {
    let references = core::expected_evidence(policy)
        .map_err(|issues| format!("review policy evidence is invalid: {}", issue_codes(&issues)))?;
    let mut observed = BTreeMap::new();
    for reference in references {
        let path = Path::new(&reference.path);
        let Ok(metadata) = fs::symlink_metadata(path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES {
            continue;
        }
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        observed.insert(reference.path, blake3::hash(&bytes).to_hex().to_string());
    }
    Ok(observed)
}

fn report_bytes(report: &InventoryReport) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(report).map_err(|_| "inventory JSON serialization failed".to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_report(report: &InventoryReport) -> Result<(), String> {
    let bytes = report_bytes(report)?;
    let temporary_path = Path::new(TEMP_OUTPUT_PATH);
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary_path)
        .map_err(|_| "temporary inventory output already exists or cannot be created".to_string())?;
    let write_result = file
        .write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "temporary inventory output could not be persisted".to_string());
    if write_result.is_err() {
        drop(file);
        let _ = fs::remove_file(temporary_path);
        return write_result;
    }
    drop(file);
    fs::rename(temporary_path, OUTPUT_PATH).map_err(|_| {
        let _ = fs::remove_file(temporary_path);
        "inventory output could not be atomically replaced".to_string()
    })
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "inventory input is unavailable".to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_INPUT_BYTES {
        return Err("inventory input is not a bounded regular file".to_string());
    }
    fs::read(path).map_err(|_| "inventory input could not be read".to_string())
}

fn safe_input_path(value: &str) -> Result<&Path, String> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err("coverage path must be a normalized repository-relative path".to_string());
    }
    Ok(path)
}

fn issue_codes(issues: &[core::InventoryIssue]) -> String {
    issues.iter().map(|issue| issue.code.as_str()).collect::<Vec<_>>().join(",")
}
