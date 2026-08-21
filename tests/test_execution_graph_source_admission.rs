use std::collections::BTreeSet;

use serde_json::Value;

const ADMISSION_SOURCE: &str = include_str!("fixtures/radicle/choregraph-source-admission-v1.ncl");
const CARGO_SOURCE: &str = include_str!("../Cargo.toml");
const CARGO_LOCK_SOURCE: &str = include_str!("../Cargo.lock");
const FLAKE_SOURCE: &str = include_str!("../flake.nix");
const FLAKE_LOCK_SOURCE: &str = include_str!("../flake.lock");

const RID: &str = "rad:zL2ncTUeASVYwcoGkEXv9JKgGbAF";
const REVISION: &str = "0b2000edaf86d5a7a8601246afbc0d526f4a2a07";
const READINESS_PROOF_REVISION: &str = "b3e08e19750f53bdbcae970cdf58a47a791ed20b";
const ARCHIVE_BLAKE3: &str = "8c35370153e249cc2fe9454746a82950d830e1451b3cd77cece96321acf1ea1e";
const LICENSE: &str = "AGPL-3.0-or-later";
const HTTPS_URL: &str = "https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git";
const NAR_HASH: &str = "sha256-0aF9Hhhp/9yrm6tO977JSss0FcqmqPNYKZocqF4PbBA=";
const CARGO_PIN: &str = "seaglass-execution-graph = { package = \"choregraph-seaglass\", git = \"https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git\", rev = \"0b2000edaf86d5a7a8601246afbc0d526f4a2a07\" }";
const LOCK_SOURCE: &str = "git+https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git?rev=0b2000edaf86d5a7a8601246afbc0d526f4a2a07#0b2000edaf86d5a7a8601246afbc0d526f4a2a07";
const FLAKE_PIN: &str = "url = \"git+https://seed.radicle.garden/zL2ncTUeASVYwcoGkEXv9JKgGbAF.git?rev=0b2000edaf86d5a7a8601246afbc0d526f4a2a07\";";
const CARGO_LOCK_PACKAGE_COUNT: usize = 5;
const HISTORICAL_PUBLICATION_REVISION: &str = "fc47c5f4ecbb7b4341af690fa42199f25d57f54c";

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

fn admission_matches(source: &str) -> bool {
    let required_lines = [
        "schema = \"lattice.choregraph-source-admission.v1\"".to_owned(),
        "status = 'accepted".to_owned(),
        format!("rid = \"{RID}\""),
        format!("revision = \"{REVISION}\""),
        format!("readiness_proof_revision = \"{READINESS_PROOF_REVISION}\""),
        format!("source_url = \"{HTTPS_URL}\""),
        format!("source_archive_blake3 = \"{ARCHIVE_BLAKE3}\""),
        format!("license = \"{LICENSE}\""),
        format!("cargo_revision = \"{REVISION}\""),
        format!("nix_revision = \"{REVISION}\""),
        format!("publication_receipt_revision = \"{HISTORICAL_PUBLICATION_REVISION}\""),
        "publication_receipt_matches_revision = false".to_owned(),
        "execution_graph_baseline = true".to_owned(),
        "readiness_completeness = true".to_owned(),
        "forbidden_fallbacks = [\"floating\", \"github\", \"sibling\", \"vendor\", \"runtime\"]".to_owned(),
    ];
    required_lines
        .iter()
        .all(|required| source.lines().any(|line| line.trim().trim_end_matches(',') == required))
}

fn flake_lock_matches(source: &str) -> bool {
    let Ok(lock) = serde_json::from_str::<Value>(source) else {
        return false;
    };
    let Some(node) = lock.pointer("/nodes/choregraph") else {
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
    if !admission_matches(inputs.receipt) {
        issues.insert(ISSUE_RECEIPT);
    }
    if !inputs.cargo.lines().any(|line| line.trim() == CARGO_PIN)
        || inputs.cargo.contains("seaglass-execution-graph = { path =")
    {
        issues.insert(ISSUE_CARGO);
    }
    if inputs.cargo_lock.matches(LOCK_SOURCE).count() != CARGO_LOCK_PACKAGE_COUNT {
        issues.insert(ISSUE_CARGO_LOCK);
    }
    if !inputs.flake.contains(FLAKE_PIN) || !inputs.flake.contains("src = choregraph;") {
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
    let issues = validate(current_inputs());
    assert!(issues.is_empty(), "unexpected source issues: {issues:?}");
}

// r[verify lattice.execution_graph_external_adoption.source.scenario.rejected]
#[test]
fn malformed_and_incomplete_receipts_are_rejected() {
    let base = current_inputs();
    let wrong_rid = replace_once(base.receipt, &format!("rid = \"{RID}\""), "rid = \"rad:zUnknownChoregraph\"");
    let wrong_revision = replace_once(
        base.receipt,
        &format!("revision = \"{REVISION}\""),
        "revision = \"0000000000000000000000000000000000000000\"",
    );
    let wrong_hash = replace_once(
        base.receipt,
        &format!("source_archive_blake3 = \"{ARCHIVE_BLAKE3}\""),
        "source_archive_blake3 = \"invalid-archive-hash\"",
    );
    let wrong_license =
        replace_once(base.receipt, &format!("license = \"{LICENSE}\""), "license = \"LicenseRef-Unknown\"");
    let wrong_endpoint = replace_once(
        base.receipt,
        &format!("source_url = \"{HTTPS_URL}\""),
        "source_url = \"https://github.com/onixresearch/choregraph\"",
    );
    let floating = replace_once(base.receipt, &format!("revision = \"{REVISION}\""), "revision = \"main\"");
    let fixtures = [
        "{",
        &wrong_rid,
        &wrong_revision,
        &wrong_hash,
        &wrong_license,
        &wrong_endpoint,
        &floating,
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
    let path_manifest = replace_once(base.cargo, CARGO_PIN, "seaglass-execution-graph = { path = \"../choregraph\" }");
    let floating_pin = CARGO_PIN.replace(REVISION, "main");
    let floating_manifest = replace_once(base.cargo, CARGO_PIN, &floating_pin);
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
