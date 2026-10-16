#[path = "../scripts/octet_baseline_migration.rs"]
mod migration;

use serde_json::Value;
use serde_json::json;

const CONFIG_HASH: &str = "b3:warning-config";

fn finding(id: &str, lint: &str, message: &str) -> Value {
    json!({
        "stable_id": id, "semantic_id": id, "identity_status": "computed",
        "lint": lint, "group": "tigerstyle::structure", "severity": "warning",
        "file": "src/engine/executor.rs", "line": 10, "column": 1,
        "message": message, "crate_name": "lattice",
    })
}

fn metadata() -> Value {
    json!({
        "tool_name": "cargo-octet", "tool_version": "0.1.0", "rustc_version": "reviewed-rustc",
        "toolchain": "reviewed-nightly", "profile_name": "workspace-metadata",
        "profile_hash": "b3:profile", "config_hash": CONFIG_HASH,
    })
}

struct Fixture {
    directory: tempfile::TempDir,
    args: Vec<String>,
}

impl Fixture {
    fn new(old: Vec<Value>, current: Vec<Value>) -> Self {
        Self::with_reference(&old, &old, &current)
    }

    fn with_reference(old: &[Value], reference: &[Value], current: &[Value]) -> Self {
        let directory = tempfile::tempdir().expect("migration directory");
        let paths = [
            "old.json",
            "reference.jsonl",
            "reference-status.json",
            "results.jsonl",
            "status.json",
            "output.json",
        ]
        .map(|name| directory.path().join(name));
        let baseline = json!({
            "schema_version": 1, "finding_schema_version": 2, "metadata": metadata(),
            "baseline_hash": "b3:old", "findings": old,
        });
        std::fs::write(&paths[0], baseline.to_string()).expect("old baseline");
        for (results, status, observations) in [(&paths[1], &paths[2], reference), (&paths[3], &paths[4], current)] {
            let rows = observations
                .iter()
                .map(|finding| {
                    let mut observation = finding.clone();
                    observation["config_hash"] = json!(CONFIG_HASH);
                    observation.to_string()
                })
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(results, rows).expect("complete observations");
            let value = json!({
                "metadata": metadata(), "total_findings": observations.len(),
                "cargo_process_exit": {"classification": "success", "code": 0},
            });
            std::fs::write(status, value.to_string()).expect("complete runner status");
        }
        let args = std::iter::once("octet-baseline-migrate".to_string())
            .chain(paths.iter().map(|path| path.to_string_lossy().into_owned()))
            .collect();
        Self { directory, args }
    }

    fn run(&self, execute: bool) -> Result<(), String> {
        let mut args = self.args.clone();
        if execute {
            args.push("--execute".to_string());
        }
        migration::run(&args)
    }

    fn output_path(&self) -> std::path::PathBuf {
        self.directory.path().join("output.json")
    }

    fn output(&self) -> Value {
        serde_json::from_slice(&std::fs::read(self.output_path()).expect("written baseline")).expect("valid baseline")
    }

    fn assert_refused(&self) {
        std::fs::write(self.output_path(), b"preserve existing baseline\n").expect("existing output");
        assert!(self.run(true).is_err());
        assert_eq!(std::fs::read(self.output_path()).expect("output stays present"), b"preserve existing baseline\n");
    }
}

// r[verify lattice.repository_validation.octet_baseline_migration]
#[test]
fn identity_only_migration_preserves_every_warning_and_dry_run_does_not_write() {
    let old = vec![
        finding("b3:old-length", "excessive_file_length", "source file is 4000 lines (limit: 500)"),
        finding(
            "b3:old-name",
            "path_segment_repetition",
            "item name `Executor` repeats `executor` from ancestor path segment `executor`",
        ),
    ];
    let mut current = old.clone();
    current[0]["stable_id"] = json!("b3:new-length");
    current[0]["semantic_id"] = json!("b3:new-length");
    current[0]["message"] = json!("source file is 4015 lines (limit: 500)");
    current[1]["stable_id"] = json!("b3:new-name");
    current[1]["semantic_id"] = json!("b3:new-name");
    current[1]["line"] = json!(42);
    let fixture = Fixture::new(old, current.clone());
    fixture.run(false).expect("identity-only dry run");
    assert!(!fixture.output_path().exists());
    fixture.run(true).expect("identity-only execute");
    let output = fixture.output();
    assert_eq!(output["findings"], json!(current));
    assert_eq!(output["metadata"], metadata());
    assert_eq!(output["schema_version"], 1);
    assert_eq!(output["finding_schema_version"], 2);
    let payload = json!({"schema_version": 1, "stable_ids": ["b3:new-length", "b3:new-name"]});
    assert_eq!(output["baseline_hash"], format!("b3:{}", blake3::hash(payload.to_string().as_bytes()).to_hex()));
}

// r[verify lattice.repository_validation.octet_baseline_migration.scenario.wording]
#[test]
fn reviewed_wording_changes_preserve_parameter_type_count_and_quantity_name() {
    for (lint, before, after) in [
        (
            "ambiguous_params",
            "2 consecutive `&str` parameters are easy to swap by accident",
            "2 `&str` parameters are easy to swap by accident",
        ),
        (
            "numeric_units",
            "`timeout` looks like a quantity but has no unit suffix",
            "`timeout` looks like a raw numeric quantity but has no unit suffix",
        ),
        (
            "assertion_density",
            "function `run` has 0 assertion(s) in 73 lines (need 2+)",
            "function `run` has 0 assertion(s) in 76 lines (need 2+)",
        ),
        (
            "function_length",
            "function `run` is 73 lines (limit: 50)",
            "function `run` is 76 lines (limit: 50)",
        ),
    ] {
        let current = finding("b3:current", lint, after);
        let fixture = Fixture::new(vec![finding("b3:old", lint, before)], vec![current.clone()]);
        fixture.run(true).expect("reviewed wording transition");
        assert_eq!(fixture.output()["findings"], json!([current]));
    }
}

// r[verify lattice.repository_validation.octet_baseline_migration.scenario.multiplicity]
#[test]
fn semantic_occurrence_growth_and_loss_refuse_without_touching_baseline() {
    let old = finding("b3:old", "ambiguous_params", "2 `&str` parameters are easy to swap by accident");
    let one = finding("b3:new-one", "ambiguous_params", "2 `&str` parameters are easy to swap by accident");
    let two = finding("b3:new-two", "ambiguous_params", "2 `&str` parameters are easy to swap by accident");
    Fixture::new(vec![old.clone()], vec![one.clone(), two.clone()]).assert_refused();
    Fixture::new(vec![old, two], vec![one]).assert_refused();
}

// r[verify lattice.repository_validation.octet_baseline_migration.scenario.fail_closed]
#[test]
fn changed_warning_facts_and_unreviewed_wording_refuse_without_writing() {
    for (lint, before, after) in [
        (
            "ambiguous_params",
            "2 `&str` parameters are easy to swap by accident",
            "3 `&str` parameters are easy to swap by accident",
        ),
        (
            "ambiguous_params",
            "2 `&str` parameters are easy to swap by accident",
            "2 `u64` parameters are easy to swap by accident",
        ),
        (
            "numeric_units",
            "`timeout` looks like a quantity but has no unit suffix",
            "`bytes` looks like a raw numeric quantity but has no unit suffix",
        ),
        (
            "numeric_units",
            "`timeout` looks like a quantity but has no unit suffix",
            "`timeout` has the wrong unit",
        ),
        (
            "excessive_file_length",
            "source file is 4000 lines (limit: 500)",
            "source file is 4001 lines (limit: 600)",
        ),
        ("excessive_file_length", "source file is 4000 lines (limit: 500)", "unreviewed file warning"),
        (
            "assertion_density",
            "function `run` has 0 assertion(s) in 73 lines (need 2+)",
            "function `run` has 1 assertion(s) in 76 lines (need 2+)",
        ),
        (
            "function_length",
            "function `run` is 73 lines (limit: 50)",
            "function `other` is 76 lines (limit: 50)",
        ),
    ] {
        Fixture::new(vec![finding("b3:old", lint, before)], vec![finding("b3:new", lint, after)]).assert_refused();
    }
    let old = finding("b3:old", "demo", "same warning");
    for (field, changed) in [
        ("file", "src/other.rs"),
        ("severity", "error"),
        ("lint", "other"),
        ("group", "other"),
        ("identity_status", "fallback"),
        ("crate_name", "other"),
    ] {
        let mut current = old.clone();
        current[field] = json!(changed);
        Fixture::new(vec![old.clone()], vec![current]).assert_refused();
    }
}

#[test]
fn repeated_runner_identities_require_a_complete_matching_reference() {
    let old = finding("b3:old", "demo", "same warning");
    let one = finding("b3:current", "demo", "same warning");
    let mut repeated = one.clone();
    repeated["line"] = json!(20);
    let fixture =
        Fixture::with_reference(std::slice::from_ref(&old), &[old.clone(), old.clone()], &[one.clone(), repeated]);
    fixture.run(true).expect("repeated stable identity");
    assert_eq!(fixture.output()["findings"], json!([one.clone()]));
    let mut conflict = one.clone();
    conflict["message"] = json!("different warning under same identity");
    Fixture::new(vec![old], vec![one, conflict]).assert_refused();
}

#[test]
fn canonical_identity_collisions_cannot_hide_changed_raw_warning_facts() {
    let first = finding("b3:old", "demo", "first warning");
    let second = finding("b3:old", "demo", "second warning at the same compiler span");
    let mut current = vec![first.clone(), second.clone()];
    for row in &mut current {
        row["stable_id"] = json!("b3:current");
        row["semantic_id"] = json!("b3:current");
    }
    let reference = [first.clone(), second];
    let fixture = Fixture::with_reference(std::slice::from_ref(&first), &reference, &current);
    fixture.run(true).expect("complete colliding facts preserve canonical baseline");
    assert_eq!(fixture.output()["findings"], json!([current[0].clone()]));
    let mut changed = current.clone();
    changed[1]["message"] = json!("unreviewed hidden warning");
    Fixture::with_reference(std::slice::from_ref(&first), &reference, &changed).assert_refused();
    current.push(current[1].clone());
    Fixture::with_reference(std::slice::from_ref(&first), &reference, &current).assert_refused();
}

#[test]
fn incomplete_observation_and_changed_tool_profile_or_config_fail_closed() {
    let finding = finding("b3:unchanged", "demo", "same warning");
    for field in [
        "tool_name",
        "tool_version",
        "rustc_version",
        "toolchain",
        "profile_name",
        "profile_hash",
        "config_hash",
    ] {
        for status_index in [3, 5] {
            let fixture = Fixture::new(vec![finding.clone()], vec![finding.clone()]);
            let status_path = &fixture.args[status_index];
            let mut status: Value =
                serde_json::from_slice(&std::fs::read(status_path).expect("status")).expect("status JSON");
            status["metadata"][field] = json!("unreviewed");
            std::fs::write(status_path, status.to_string()).expect("changed status");
            fixture.assert_refused();
        }
    }
    for (field, value) in [
        ("cargo_process_exit", json!({"classification":"failure", "code":1})),
        ("total_findings", json!(2)),
    ] {
        for status_index in [3, 5] {
            let fixture = Fixture::new(vec![finding.clone()], vec![finding.clone()]);
            let status_path = &fixture.args[status_index];
            let mut status: Value =
                serde_json::from_slice(&std::fs::read(status_path).expect("status")).expect("status JSON");
            status[field] = value.clone();
            std::fs::write(status_path, status.to_string()).expect("incomplete status");
            fixture.assert_refused();
        }
    }
}

// r[verify lattice.repository_validation.octet_baseline_migration.scenario.fail_closed]
#[test]
fn raw_observation_config_must_match_each_reference_and_candidate_status() {
    let findings = vec![
        finding("b3:first", "demo", "first warning"),
        finding("b3:second", "demo", "second warning"),
    ];
    let valid = Fixture::new(findings.clone(), findings.clone());
    valid.run(true).expect("matching raw and status config cohort");
    assert_eq!(valid.output()["findings"], json!(findings));

    for results_index in [2, 4] {
        for config_hash in [Some(json!("b3:foreign-config")), None] {
            let fixture = Fixture::new(findings.clone(), findings.clone());
            let results_path = &fixture.args[results_index];
            let mut rows = std::fs::read_to_string(results_path)
                .expect("raw observations")
                .lines()
                .map(|line| serde_json::from_str::<Value>(line).expect("raw observation JSON"))
                .collect::<Vec<_>>();
            match config_hash {
                Some(hash) => rows[1]["config_hash"] = hash,
                None => {
                    rows[1].as_object_mut().expect("observation object").remove("config_hash");
                }
            }
            rows[1]["stable_id"] = json!("b3:foreign-config-identity");
            rows[1]["semantic_id"] = json!("b3:foreign-config-identity");
            let changed = rows.iter().map(Value::to_string).collect::<Vec<_>>().join("\n");
            std::fs::write(results_path, changed).expect("foreign-cohort observations with unchanged status");
            fixture.assert_refused();
        }
    }
}
