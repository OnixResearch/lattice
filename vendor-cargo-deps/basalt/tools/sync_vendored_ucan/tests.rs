use crate::model::{
    EXECUTABLE_FILE_MODE, Mode, REGULAR_FILE_MODE, REQUIRED_SNAPSHOT_PATHS, SnapshotEntry,
    parse_mode, render_manifest_material, validate_required_entries, validate_source_revision,
};

const TEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TEST_BYTE_LEN: u64 = 1;
const TEST_REVISION: &str = "1111111111111111111111111111111111111111";

fn entry(path: &str) -> SnapshotEntry {
    SnapshotEntry {
        mode: REGULAR_FILE_MODE,
        digest: TEST_DIGEST.to_owned(),
        path: path.to_owned(),
        byte_len: TEST_BYTE_LEN,
    }
}

#[test]
fn required_license_and_source_inventory_is_admitted() {
    let entries = REQUIRED_SNAPSHOT_PATHS
        .iter()
        .map(|path| entry(path))
        .collect::<Vec<_>>();

    assert!(validate_required_entries(&entries).is_ok());
}

#[test]
fn missing_holder_observation_sources_are_rejected() {
    for missing in [
        "src/token/holder_invocation/recheck.rs",
        "crates/ucan-core/src/holder_invocation/recheck.rs",
        "docs/holder-authorization-recheck.md",
    ] {
        let entries = REQUIRED_SNAPSHOT_PATHS
            .iter()
            .filter(|path| **path != missing)
            .map(|path| entry(path))
            .collect::<Vec<_>>();
        let error = validate_required_entries(&entries).expect_err("missing observation source must fail");
        assert!(error.contains(missing));
    }
}

#[test]
fn missing_license_is_rejected() {
    let entries = REQUIRED_SNAPSHOT_PATHS
        .iter()
        .filter(|path| !path.ends_with("AGPL-3.0-or-later.txt"))
        .map(|path| entry(path))
        .collect::<Vec<_>>();

    let error =
        validate_required_entries(&entries).expect_err("missing AGPL license text must fail");
    assert!(error.contains("AGPL-3.0-or-later.txt"));
}

#[test]
// r[verify ucan_token_flow.vendored_source.exact_revision]
fn exact_source_revision_modes_are_admitted() {
    let arguments = vec![
        String::from("sync-vendored-ucan.rs"),
        String::from("--import-revision"),
        String::from("../ucan"),
        TEST_REVISION.to_owned(),
    ];
    let mode = parse_mode(&arguments).expect("an exact source revision must be admitted");
    assert!(matches!(mode, Mode::Import(_, revision) if revision == TEST_REVISION));
    assert!(validate_source_revision(TEST_REVISION).is_ok());
}

#[test]
// r[verify ucan_token_flow.vendored_source.exact_revision]
fn moving_or_malformed_source_revisions_are_rejected() {
    assert!(validate_source_revision("main").is_err());
    assert!(validate_source_revision("ABCDEF1111111111111111111111111111111111").is_err());
    let arguments = vec![
        String::from("sync-vendored-ucan.rs"),
        String::from("--check-source-revision"),
        String::from("../ucan"),
        String::from("main"),
    ];
    assert!(parse_mode(&arguments).is_err());
}

#[test]
fn manifest_material_is_deterministic_and_mode_sensitive() {
    let regular = entry("src/lib.rs");
    let mut executable = regular.clone();
    executable.mode = EXECUTABLE_FILE_MODE;

    let first = render_manifest_material(&[regular.clone(), executable.clone()]);
    let second = render_manifest_material(&[regular, executable]);

    assert_eq!(first, second);
    assert!(first.contains(REGULAR_FILE_MODE));
    assert!(first.contains(EXECUTABLE_FILE_MODE));
}
