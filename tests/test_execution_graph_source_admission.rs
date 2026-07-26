use std::collections::BTreeSet;

use serde_json::Value;

const ADMISSION_SOURCE: &str = include_str!("fixtures/radicle/execution-graph-source-admission-v1.json");
const CARGO_SOURCE: &str = include_str!("../Cargo.toml");
const CARGO_LOCK_SOURCE: &str = include_str!("../Cargo.lock");
const FLAKE_SOURCE: &str = include_str!("../flake.nix");
const FLAKE_LOCK_SOURCE: &str = include_str!("../flake.lock");

const SCHEMA_VERSION: u64 = 1;
const SEED_COUNT: u64 = 2;
const NOT_FOUND_STATUS: u64 = 404;
const RID: &str = "rad:z2oYsb9jGTyp68BKYhzpivY1eK58a";
const REVISION: &str = "03736f1ec46c377ff86b451260ad68aa70ff3b0b";
const ARCHIVE_BLAKE3: &str = "4b5aa3756369236fc82fbbf501d35993cfa208f142694cdd30ca370d6241192c";
const LICENSE: &str = "AGPL-3.0-or-later";
const HTTPS_URL: &str = "https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git";
const NAR_HASH: &str = "sha256-SzdJGsS631Hk3CbPLThz+2YPgvdoB1cXb3BPusJTX8I=";
const CARGO_PIN: &str = "seaglass-execution-graph = { package = \"execution-graph-core\", git = \"https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git\", rev = \"03736f1ec46c377ff86b451260ad68aa70ff3b0b\" }";
const LOCK_SOURCE: &str = "git+https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git?rev=03736f1ec46c377ff86b451260ad68aa70ff3b0b#03736f1ec46c377ff86b451260ad68aa70ff3b0b";
const FLAKE_PIN: &str = "url = \"git+https://git.onix.computer/z2oYsb9jGTyp68BKYhzpivY1eK58a.git?rev=03736f1ec46c377ff86b451260ad68aa70ff3b0b\";";

const ISSUE_RECEIPT: &str = "source.receipt";
const ISSUE_CARGO: &str = "source.cargo";
const ISSUE_CARGO_LOCK: &str = "source.cargo_lock";
const ISSUE_NIX: &str = "source.nix";
const ISSUE_NIX_LOCK: &str = "source.nix_lock";
const ISSUE_DUPLICATE: &str = "source.duplicate_package";

#[derive(Clone, Copy)]
struct Inputs<'a> {
    receipt: &'a str,
    cargo: &'a str,
    cargo_lock: &'a str,
    flake: &'a str,
    flake_lock: &'a str,
    vendor_present: bool,
}

fn current_inputs() -> Inputs<'static> {
    Inputs {
        receipt: ADMISSION_SOURCE,
        cargo: CARGO_SOURCE,
        cargo_lock: CARGO_LOCK_SOURCE,
        flake: FLAKE_SOURCE,
        flake_lock: FLAKE_LOCK_SOURCE,
        vendor_present: false,
    }
}

fn receipt_matches(source: &str) -> bool {
    let Ok(receipt) = serde_json::from_str::<Value>(source) else {
        return false;
    };
    receipt.pointer("/schema_version").and_then(Value::as_u64) == Some(SCHEMA_VERSION)
        && receipt.pointer("/receipt_type").and_then(Value::as_str)
            == Some("lattice.radicle-execution-graph-source-admission.v1")
        && receipt.pointer("/status").and_then(Value::as_str) == Some("accepted")
        && receipt.pointer("/repository/rid").and_then(Value::as_str) == Some(RID)
        && receipt.pointer("/repository/reviewed_commit").and_then(Value::as_str) == Some(REVISION)
        && receipt.pointer("/repository/source_archive_blake3").and_then(Value::as_str) == Some(ARCHIVE_BLAKE3)
        && receipt.pointer("/repository/license").and_then(Value::as_str) == Some(LICENSE)
        && receipt.pointer("/sources/cargo_git").and_then(Value::as_str) == Some(HTTPS_URL)
        && receipt.pointer("/sources/cargo_locked_source").and_then(Value::as_str) == Some(LOCK_SOURCE)
        && receipt.pointer("/sources/nix_nar_hash").and_then(Value::as_str) == Some(NAR_HASH)
        && receipt.pointer("/availability/seed_count").and_then(Value::as_u64) == Some(SEED_COUNT)
        && receipt.pointer("/availability/receive_pack_status").and_then(Value::as_u64) == Some(NOT_FOUND_STATUS)
        && receipt.pointer("/availability/unknown_rid_status").and_then(Value::as_u64) == Some(NOT_FOUND_STATUS)
        && receipt.pointer("/availability/github_fallback_configured").and_then(Value::as_bool) == Some(false)
}

fn flake_lock_matches(source: &str) -> bool {
    let Ok(lock) = serde_json::from_str::<Value>(source) else {
        return false;
    };
    let Some(node) = lock.pointer("/nodes/execution-graph") else {
        return false;
    };
    node.pointer("/locked/rev").and_then(Value::as_str) == Some(REVISION)
        && node.pointer("/locked/url").and_then(Value::as_str) == Some(HTTPS_URL)
        && node.pointer("/locked/narHash").and_then(Value::as_str) == Some(NAR_HASH)
        && node.pointer("/original/rev").and_then(Value::as_str) == Some(REVISION)
        && node.pointer("/original/url").and_then(Value::as_str) == Some(HTTPS_URL)
}

fn validate(inputs: Inputs<'_>) -> BTreeSet<&'static str> {
    let mut issues = BTreeSet::new();
    if !receipt_matches(inputs.receipt) {
        issues.insert(ISSUE_RECEIPT);
    }
    if !inputs.cargo.lines().any(|line| line.trim() == CARGO_PIN)
        || inputs.cargo.contains("seaglass-execution-graph = { path =")
    {
        issues.insert(ISSUE_CARGO);
    }
    if !inputs.cargo_lock.contains(LOCK_SOURCE) {
        issues.insert(ISSUE_CARGO_LOCK);
    }
    if !inputs.flake.contains(FLAKE_PIN) || !inputs.flake.contains("src = execution-graph;") {
        issues.insert(ISSUE_NIX);
    }
    if !flake_lock_matches(inputs.flake_lock) {
        issues.insert(ISSUE_NIX_LOCK);
    }
    if inputs.vendor_present {
        issues.insert(ISSUE_DUPLICATE);
    }
    issues
}

fn replace_once(source: &str, old: &str, new: &str) -> String {
    assert!(source.contains(old), "negative fixture must replace an existing value");
    source.replacen(old, new, 1)
}

// r[verify lattice.execution_graph_external_adoption.source.scenario.accepted]
#[test]
fn exact_source_is_accepted() {
    assert!(validate(current_inputs()).is_empty());
}

// r[verify lattice.execution_graph_external_adoption.source.scenario.rejected]
#[test]
fn malformed_and_incomplete_receipts_are_rejected() {
    let base = current_inputs();
    let wrong_rid = replace_once(base.receipt, RID, "rad:zUnknownExecutionGraph");
    let wrong_revision = replace_once(base.receipt, REVISION, "0000000000000000000000000000000000000000");
    let wrong_hash = replace_once(base.receipt, ARCHIVE_BLAKE3, "invalid-archive-hash");
    let wrong_license = replace_once(base.receipt, LICENSE, "LicenseRef-Unknown");
    let wrong_endpoint = replace_once(base.receipt, HTTPS_URL, "https://github.com/OnixResearch/execution-graph");
    let fallback =
        replace_once(base.receipt, "\"github_fallback_configured\": false", "\"github_fallback_configured\": true");
    let fixtures = [
        "{",
        &wrong_rid,
        &wrong_revision,
        &wrong_hash,
        &wrong_license,
        &wrong_endpoint,
        &fallback,
    ];

    for fixture in fixtures {
        assert_eq!(
            validate(Inputs {
                receipt: fixture,
                ..base
            }),
            BTreeSet::from([ISSUE_RECEIPT])
        );
    }
}

// r[verify lattice.execution_graph_external_adoption.compatibility.scenario.differ]
// r[verify lattice.execution_graph_external_adoption.cutover.scenario.accepted]
#[test]
fn source_drift_and_duplicate_packages_are_rejected() {
    let base = current_inputs();
    let path_manifest =
        replace_once(base.cargo, CARGO_PIN, "seaglass-execution-graph = { path = \"../execution-graph\" }");
    let floating_manifest = replace_once(base.cargo, REVISION, "main");
    let wrong_cargo_lock = replace_once(base.cargo_lock, REVISION, "0000000000000000000000000000000000000000");
    let wrong_flake = replace_once(base.flake, REVISION, "0000000000000000000000000000000000000000");
    let wrong_flake_lock = replace_once(base.flake_lock, NAR_HASH, "sha256-invalid");

    assert!(
        validate(Inputs {
            cargo: &path_manifest,
            ..base
        })
        .contains(ISSUE_CARGO)
    );
    assert!(
        validate(Inputs {
            cargo: &floating_manifest,
            ..base
        })
        .contains(ISSUE_CARGO)
    );
    assert!(
        validate(Inputs {
            cargo_lock: &wrong_cargo_lock,
            ..base
        })
        .contains(ISSUE_CARGO_LOCK)
    );
    assert!(
        validate(Inputs {
            flake: &wrong_flake,
            ..base
        })
        .contains(ISSUE_NIX)
    );
    assert!(
        validate(Inputs {
            flake_lock: &wrong_flake_lock,
            ..base
        })
        .contains(ISSUE_NIX_LOCK)
    );
    assert_eq!(
        validate(Inputs {
            vendor_present: true,
            ..base
        }),
        BTreeSet::from([ISSUE_DUPLICATE])
    );
}
