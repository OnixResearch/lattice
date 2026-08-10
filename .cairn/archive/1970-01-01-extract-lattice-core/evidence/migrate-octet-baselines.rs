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

use serde::{Deserialize, Serialize};
use serde_json::Value;

const EXECUTE_ARG: &str = "--execute";
const SELF_TEST_ARG: &str = "--self-test";
const SUCCESS_EXIT_CODE: u8 = 0;
const FAILURE_EXIT_CODE: u8 = 1;
const SELF_TEST_ARG_COUNT: usize = 2;
const EXECUTE_ARG_COUNT: usize = 11;
const EXPECTED_OLD_ROOT_RECORDS: usize = 1141;
const EXPECTED_OLD_ROOT_IDENTITIES: usize = 890;
const EXPECTED_NEW_ROOT_IDENTITIES: usize = 872;
const EXPECTED_CORE_IDENTITIES: usize = 24;
const EXPECTED_REMOVED_ROOT_IDENTITIES: usize = 18;
const EXPECTED_OLD_MOVED_OCCURRENCES: usize = 33;
const EXPECTED_CORE_OCCURRENCES: usize = 32;
const DIAGNOSTIC_LIMIT: usize = 10;
const ROOT_CRATE_NAME: &str = "lattice";
const CORE_CRATE_NAME: &str = "lattice_core";
const MODULE_FILE_COUNT_LINT: &str = "module_file_count";
const EXCESSIVE_FILE_LENGTH_LINT: &str = "excessive_file_length";
const NORMALIZED_MODULE_FILE_COUNT_MESSAGE: &str = "module directory exceeds configured source-file limit";
const NORMALIZED_FILE_LENGTH_MESSAGE: &str = "source file exceeds configured line limit";
const MOVED_PATHS: &[(&str, &str)] = &[
    ("src/content_identity/mod.rs", "crates/lattice-core/src/content_identity/mod.rs"),
    ("src/control_flow.rs", "crates/lattice-core/src/control/flow.rs"),
    ("src/engine/value.rs", "crates/lattice-core/src/value.rs"),
    ("src/expressions.rs", "crates/lattice-core/src/expressions.rs"),
];

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Baseline {
    schema_version: u64,
    finding_schema_version: u64,
    metadata: Value,
    baseline_hash: String,
    findings: Vec<BaselineFinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize)]
struct Occurrence {
    #[serde(rename = "normalized_snippet_hash")]
    _normalized_snippet_hash: Option<String>,
    lint: String,
    group: String,
    severity: String,
    file: String,
    #[serde(rename = "line")]
    _line: u64,
    #[serde(rename = "column")]
    _column: u64,
    message: String,
    suggestion: Option<String>,
    applicability: Option<String>,
    principle: String,
    effect_family: String,
    evidence_tier: String,
    crate_name: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BaselineKey {
    identity_status: String,
    lint: String,
    group: String,
    severity: String,
    file: String,
    message: String,
    crate_name: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct OccurrenceKey {
    lint: String,
    group: String,
    severity: String,
    file: String,
    message: String,
    suggestion: Option<String>,
    applicability: Option<String>,
    principle: String,
    effect_family: String,
    evidence_tier: String,
    crate_name: String,
}

#[derive(Debug, Serialize)]
struct MigrationReport {
    schema: &'static str,
    old_root_baseline_blake3: String,
    new_root_baseline_blake3: String,
    new_core_baseline_blake3: String,
    old_root_results_blake3: String,
    new_root_results_blake3: String,
    new_core_results_blake3: String,
    old_root_records: usize,
    old_root_identities: usize,
    new_root_identities: usize,
    new_core_identities: usize,
    exact_root_rekeys: usize,
    removed_root_identities: usize,
    retained_root_occurrences: usize,
    old_moved_occurrences: usize,
    moved_occurrences: usize,
    reviewed_warning_reductions: usize,
    verdict: &'static str,
}

fn normalized_message(lint: &str, message: &str) -> String {
    match lint {
        MODULE_FILE_COUNT_LINT => NORMALIZED_MODULE_FILE_COUNT_MESSAGE.to_owned(),
        EXCESSIVE_FILE_LENGTH_LINT => NORMALIZED_FILE_LENGTH_MESSAGE.to_owned(),
        _ => message.to_owned(),
    }
}

fn baseline_key(finding: &BaselineFinding) -> BaselineKey {
    BaselineKey {
        identity_status: finding.identity_status.clone(),
        lint: finding.lint.clone(),
        group: finding.group.clone(),
        severity: finding.severity.clone(),
        file: finding.file.clone(),
        message: normalized_message(&finding.lint, &finding.message),
        crate_name: finding.crate_name.clone(),
    }
}

fn moved_destination(source: &str) -> Option<&'static str> {
    MOVED_PATHS
        .iter()
        .find_map(|(old, new)| (*old == source).then_some(*new))
}

fn occurrence_key(occurrence: &Occurrence, normalize_moved_path: bool) -> Result<OccurrenceKey, String> {
    let (file, crate_name) = if normalize_moved_path {
        let destination = moved_destination(&occurrence.file)
            .ok_or_else(|| format!("old occurrence is outside the moved source set: {}", occurrence.file))?;
        (destination.to_owned(), CORE_CRATE_NAME.to_owned())
    } else {
        (occurrence.file.clone(), occurrence.crate_name.clone())
    };

    Ok(OccurrenceKey {
        lint: occurrence.lint.clone(),
        group: occurrence.group.clone(),
        severity: occurrence.severity.clone(),
        file,
        message: normalized_message(&occurrence.lint, &occurrence.message),
        suggestion: occurrence.suggestion.clone(),
        applicability: occurrence.applicability.clone(),
        principle: occurrence.principle.clone(),
        effect_family: occurrence.effect_family.clone(),
        evidence_tier: occurrence.evidence_tier.clone(),
        crate_name,
    })
}

fn multiset<T: Ord>(items: impl IntoIterator<Item = T>) -> BTreeMap<T, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        let count = counts.entry(item).or_insert(0);
        *count += 1;
    }
    counts
}

fn unique_identity_count(baseline: &Baseline) -> usize {
    baseline
        .findings
        .iter()
        .map(|finding| finding.stable_id.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

fn validate_baseline_transition(
    old: &Baseline,
    new_root: &Baseline,
    new_core: &Baseline,
    enforce_production_counts: bool,
) -> Result<(usize, usize), String> {
    let old_unique_count = unique_identity_count(old);
    let new_root_unique_count = unique_identity_count(new_root);
    let new_core_unique_count = unique_identity_count(new_core);
    if enforce_production_counts {
        if old.findings.len() != EXPECTED_OLD_ROOT_RECORDS {
            return Err(format!("old root record count changed: {}", old.findings.len()));
        }
        if old_unique_count != EXPECTED_OLD_ROOT_IDENTITIES {
            return Err(format!("old root unique identity count changed: {old_unique_count}"));
        }
        if new_root_unique_count != EXPECTED_NEW_ROOT_IDENTITIES {
            return Err(format!("new root identity count changed: {new_root_unique_count}"));
        }
        if new_core_unique_count != EXPECTED_CORE_IDENTITIES {
            return Err(format!("new core identity count changed: {new_core_unique_count}"));
        }
    }

    let old_by_key = multiset(old.findings.iter().map(baseline_key));
    let new_by_key = multiset(new_root.findings.iter().map(baseline_key));
    let mut exact_rekeys = 0usize;

    for (key, new_count) in &new_by_key {
        if !old_by_key.contains_key(key) {
            return Err(format!("new root baseline contains unreviewed semantic finding: {key:?}"));
        }
        exact_rekeys += new_count;
    }

    let removed = old_unique_count.saturating_sub(new_root_unique_count);
    if enforce_production_counts && removed != EXPECTED_REMOVED_ROOT_IDENTITIES {
        return Err(format!("removed root identity count changed: {removed}"));
    }

    for new_finding in &new_root.findings {
        let matching_old = old.findings.iter().find(|old_finding| baseline_key(old_finding) == baseline_key(new_finding));
        let old_finding = matching_old.ok_or_else(|| "missing exact old baseline match".to_owned())?;
        if old_finding.stable_id == new_finding.stable_id || old_finding.semantic_id == new_finding.semantic_id {
            return Err(format!("expected config-bound identity rekey for {}", new_finding.stable_id));
        }
    }

    Ok((exact_rekeys, removed))
}

fn validate_root_occurrences(
    old_occurrences: &[Occurrence],
    root_occurrences: &[Occurrence],
) -> Result<usize, String> {
    let old_retained = old_occurrences
        .iter()
        .filter(|occurrence| moved_destination(&occurrence.file).is_none())
        .map(|occurrence| occurrence_key(occurrence, false))
        .collect::<Result<Vec<_>, _>>()?;
    let root = root_occurrences
        .iter()
        .map(|occurrence| {
            if occurrence.crate_name != ROOT_CRATE_NAME {
                return Err(format!("root result has unexpected crate: {}", occurrence.crate_name));
            }
            occurrence_key(occurrence, false)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let old_counts = multiset(old_retained);
    let root_counts = multiset(root);
    if old_counts != root_counts {
        let added = root_counts
            .iter()
            .filter(|(key, count)| old_counts.get(*key).copied().unwrap_or(0) < **count)
            .take(DIAGNOSTIC_LIMIT)
            .collect::<Vec<_>>();
        let removed = old_counts
            .iter()
            .filter(|(key, count)| root_counts.get(*key).copied().unwrap_or(0) < **count)
            .take(DIAGNOSTIC_LIMIT)
            .collect::<Vec<_>>();
        return Err(format!("retained root warning semantics or multiplicity changed; added={added:?}; removed={removed:?}"));
    }
    Ok(root_occurrences.len())
}

fn validate_moved_occurrences(
    old_occurrences: &[Occurrence],
    core_occurrences: &[Occurrence],
    enforce_production_counts: bool,
) -> Result<usize, String> {
    let old_moved = old_occurrences
        .iter()
        .filter(|occurrence| moved_destination(&occurrence.file).is_some())
        .map(|occurrence| occurrence_key(occurrence, true))
        .collect::<Result<Vec<_>, _>>()?;
    let core = core_occurrences
        .iter()
        .map(|occurrence| {
            if occurrence.crate_name != CORE_CRATE_NAME {
                return Err(format!("core result has unexpected crate: {}", occurrence.crate_name));
            }
            occurrence_key(occurrence, false)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if enforce_production_counts && old_moved.len() != EXPECTED_OLD_MOVED_OCCURRENCES {
        return Err(format!("old moved occurrence count changed: {}", old_moved.len()));
    }
    let reviewed_reduction = old_moved
        .iter()
        .filter(|key| {
            key.lint == "underscore_in_module_filename"
                && key.file == "crates/lattice-core/src/control/flow.rs"
        })
        .count();
    if reviewed_reduction != 1 {
        return Err(format!("reviewed module-path warning reduction changed: {reviewed_reduction}"));
    }
    let retained_old = old_moved
        .into_iter()
        .filter(|key| {
            !(key.lint == "underscore_in_module_filename"
                && key.file == "crates/lattice-core/src/control/flow.rs")
        })
        .collect::<Vec<_>>();
    if enforce_production_counts && core.len() != EXPECTED_CORE_OCCURRENCES {
        return Err(format!("core occurrence count changed: {}", core.len()));
    }
    if multiset(retained_old) != multiset(core) {
        return Err("moved warning occurrence semantics or multiplicity changed".to_owned());
    }
    Ok(core_occurrences.len())
}

fn parse_baseline(path: &Path) -> Result<(Baseline, Vec<u8>), String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let baseline = serde_json::from_slice(&bytes).map_err(|error| format!("parse {}: {error}", path.display()))?;
    Ok((baseline, bytes))
}

fn parse_jsonl(path: &Path) -> Result<Vec<Occurrence>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            serde_json::from_str(line)
                .map_err(|error| format!("parse {} line {}: {error}", path.display(), index + 1))
        })
        .collect()
}

fn blake3_id(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

fn file_blake3(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {} for BLAKE3: {error}", path.display()))?;
    Ok(blake3_id(&bytes))
}

fn execute(args: &[String]) -> Result<(), String> {
    if args.len() != EXECUTE_ARG_COUNT {
        return Err(format!("usage: {} {EXECUTE_ARG} OLD_ROOT NEW_ROOT OLD_RESULTS NEW_ROOT_RESULTS NEW_CORE_RESULTS NEW_CORE OUT_ROOT OUT_CORE REPORT", args[0]));
    }

    let (old, old_bytes) = parse_baseline(Path::new(&args[2]))?;
    let (new_root, new_root_bytes) = parse_baseline(Path::new(&args[3]))?;
    let old_occurrences = parse_jsonl(Path::new(&args[4]))?;
    let root_occurrences = parse_jsonl(Path::new(&args[5]))?;
    let core_occurrences = parse_jsonl(Path::new(&args[6]))?;
    let (new_core, new_core_bytes) = parse_baseline(Path::new(&args[7]))?;

    let retained_root_occurrences = validate_root_occurrences(&old_occurrences, &root_occurrences)?;
    let (exact_root_rekeys, removed_root_identities) =
        validate_baseline_transition(&old, &new_root, &new_core, true)?;
    let moved_occurrences = validate_moved_occurrences(&old_occurrences, &core_occurrences, true)?;

    fs::write(&args[8], &new_root_bytes).map_err(|error| format!("write {}: {error}", args[8]))?;
    fs::write(&args[9], &new_core_bytes).map_err(|error| format!("write {}: {error}", args[9]))?;

    let report = MigrationReport {
        schema: "lattice-octet-baseline-migration/v1",
        old_root_baseline_blake3: blake3_id(&old_bytes),
        new_root_baseline_blake3: blake3_id(&new_root_bytes),
        new_core_baseline_blake3: blake3_id(&new_core_bytes),
        old_root_results_blake3: file_blake3(Path::new(&args[4]))?,
        new_root_results_blake3: file_blake3(Path::new(&args[5]))?,
        new_core_results_blake3: file_blake3(Path::new(&args[6]))?,
        old_root_records: old.findings.len(),
        old_root_identities: unique_identity_count(&old),
        new_root_identities: unique_identity_count(&new_root),
        new_core_identities: unique_identity_count(&new_core),
        exact_root_rekeys,
        removed_root_identities,
        retained_root_occurrences,
        old_moved_occurrences: EXPECTED_OLD_MOVED_OCCURRENCES,
        moved_occurrences,
        reviewed_warning_reductions: EXPECTED_OLD_MOVED_OCCURRENCES - EXPECTED_CORE_OCCURRENCES,
        verdict: "pass",
    };
    let report_bytes = serde_json::to_vec_pretty(&report).map_err(|error| format!("serialize report: {error}"))?;
    fs::write(&args[10], report_bytes).map_err(|error| format!("write {}: {error}", args[10]))?;
    Ok(())
}

fn fixture_finding(file: &str, stable_id: &str) -> BaselineFinding {
    BaselineFinding {
        stable_id: stable_id.to_owned(),
        semantic_id: stable_id.to_owned(),
        identity_status: "computed".to_owned(),
        lint: "example_lint".to_owned(),
        group: "tigerstyle::example".to_owned(),
        severity: "warning".to_owned(),
        file: file.to_owned(),
        line: 1,
        column: 1,
        message: "example".to_owned(),
        crate_name: ROOT_CRATE_NAME.to_owned(),
    }
}

fn fixture_baseline(findings: Vec<BaselineFinding>) -> Baseline {
    Baseline {
        schema_version: 1,
        finding_schema_version: 2,
        metadata: Value::Null,
        baseline_hash: "b3:fixture".to_owned(),
        findings,
    }
}

fn fixture_occurrence(file: &str, crate_name: &str) -> Occurrence {
    Occurrence {
        _normalized_snippet_hash: Some("b3:snippet".to_owned()),
        lint: "example_lint".to_owned(),
        group: "tigerstyle::example".to_owned(),
        severity: "warning".to_owned(),
        file: file.to_owned(),
        _line: 1,
        _column: 1,
        message: "example".to_owned(),
        suggestion: None,
        applicability: None,
        principle: "Example".to_owned(),
        effect_family: "unknown".to_owned(),
        evidence_tier: "static_lint".to_owned(),
        crate_name: crate_name.to_owned(),
    }
}

fn self_test() -> Result<(), String> {
    let unchanged_old = fixture_finding("src/lib.rs", "b3:old-unchanged");
    let moved_old = fixture_finding("src/control_flow.rs", "b3:old-moved");
    let mut unchanged_new = unchanged_old.clone();
    unchanged_new.stable_id = "b3:new-unchanged".to_owned();
    unchanged_new.semantic_id = "b3:new-unchanged".to_owned();
    let mut core_new = moved_old.clone();
    core_new.file = "crates/lattice-core/src/control/flow.rs".to_owned();
    core_new.crate_name = CORE_CRATE_NAME.to_owned();
    core_new.stable_id = "b3:new-core".to_owned();
    core_new.semantic_id = "b3:new-core".to_owned();

    let old = fixture_baseline(vec![unchanged_old, moved_old]);
    let root = fixture_baseline(vec![unchanged_new.clone()]);
    let core = fixture_baseline(vec![core_new]);
    validate_baseline_transition(&old, &root, &core, false)?;

    let old_occurrence = fixture_occurrence("src/expressions.rs", ROOT_CRATE_NAME);
    let core_occurrence = fixture_occurrence("crates/lattice-core/src/expressions.rs", CORE_CRATE_NAME);
    let mut old_reduced = fixture_occurrence("src/control_flow.rs", ROOT_CRATE_NAME);
    old_reduced.lint = "underscore_in_module_filename".to_owned();
    validate_root_occurrences(&[old_occurrence.clone(), old_reduced.clone()], &[])?;
    validate_moved_occurrences(
        &[old_occurrence.clone(), old_reduced.clone()],
        &[core_occurrence],
        false,
    )?;

    let mut unreviewed = unchanged_new;
    unreviewed.file = "src/new.rs".to_owned();
    let invalid_root = fixture_baseline(vec![unreviewed]);
    if validate_baseline_transition(&old, &invalid_root, &core, false).is_ok() {
        return Err("negative baseline fixture did not reject unreviewed finding".to_owned());
    }
    if validate_moved_occurrences(&[old_occurrence, old_reduced], &[], false).is_ok() {
        return Err("negative occurrence fixture did not reject missing occurrence".to_owned());
    }
    Ok(())
}

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    let result = match args.get(1).map(String::as_str) {
        Some(SELF_TEST_ARG) if args.len() == SELF_TEST_ARG_COUNT => self_test(),
        Some(EXECUTE_ARG) => execute(&args),
        _ => Err(format!("usage: {} {SELF_TEST_ARG} | {EXECUTE_ARG} ...", args[0])),
    };

    match result {
        Ok(()) => ExitCode::from(SUCCESS_EXIT_CODE),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(FAILURE_EXIT_CODE)
        }
    }
}
