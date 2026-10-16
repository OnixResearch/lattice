use std::process::Command;

#[path = "identity_audit/core.rs"]
mod core;
#[path = "identity_audit/shell.rs"]
mod shell;

use core::BinarySnapshot;
use core::TextSnapshot;
use core::accepted_legacy_id_manifest_hash;
use core::archive_manifest_hash;
use core::stale_branding_findings;

const CANONICAL_PACKAGE_NAME: &str = "lattice";
const LEGACY_PRODUCT_NAME: &str = concat!("n8n", "clone");
const EXPECTED_ACCEPTED_ID_MANIFEST_BLAKE3: &str = "9a7274e65812d089ca587151e94c765fec703c4cd06d6a280fa67c5bc848c95e";
const EXPECTED_RENAME_ARCHIVE_MANIFEST_BLAKE3: &str =
    "cb0b15ff320803f9e98ae49f01691c4b478c0bf21d2c223b90d5d9e1f72f481d";

// r[verify lattice.project_identity.canonical.scenario.build-and-run]
#[test]
fn cargo_and_binary_expose_lattice_identity() {
    assert_eq!(env!("CARGO_PKG_NAME"), CANONICAL_PACKAGE_NAME);

    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--help")
        .output()
        .expect("Lattice binary should run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help output should be UTF-8");
    assert!(stdout.contains(CANONICAL_PACKAGE_NAME));
    assert!(!stdout.contains(LEGACY_PRODUCT_NAME));
}

// r[verify lattice.project_identity.canonical.scenario.stale-branding]
#[test]
fn maintained_identity_audit_rejects_stale_public_branding() {
    let root = shell::repository_root();
    let public_files = shell::read_public_text(&root).expect("public identity surfaces should be readable");
    let stale = stale_branding_findings(&public_files);

    assert!(stale.is_empty(), "stale public branding: {stale:#?}");
}

// r[verify lattice.project_identity.stable-identities.scenario.cairn-history]
#[test]
fn maintained_identity_audit_pins_cairn_ids_and_rename_archive() {
    let root = shell::repository_root();
    let accepted_specs = shell::read_accepted_specs(&root).expect("accepted specs should be readable");
    let rename_archive = shell::read_rename_archive(&root).expect("rename archive should be readable");
    let accepted_id_hash = accepted_legacy_id_manifest_hash(&accepted_specs);
    let rename_archive_hash = archive_manifest_hash(&rename_archive);

    assert_eq!(
        accepted_id_hash, EXPECTED_ACCEPTED_ID_MANIFEST_BLAKE3,
        "durable accepted requirement identity manifest changed"
    );
    assert_eq!(
        rename_archive_hash, EXPECTED_RENAME_ARCHIVE_MANIFEST_BLAKE3,
        "original rename archive manifest changed"
    );
}

#[test]
fn identity_manifests_reject_requirement_and_archive_mutation() {
    let legacy = ["n8n", "clone"].concat();
    let original_specs = vec![TextSnapshot::new(
        ".cairn/specs/example/spec.md",
        format!("r[{legacy}.example]"),
    )];
    let changed_specs = vec![TextSnapshot::new(
        ".cairn/specs/example/spec.md",
        format!("r[{legacy}.changed]"),
    )];
    let original_archive = vec![BinarySnapshot::new(
        ".cairn/archive/rename/evidence.md",
        b"stable".to_vec(),
    )];
    let changed_archive = vec![BinarySnapshot::new(
        ".cairn/archive/rename/evidence.md",
        b"changed".to_vec(),
    )];

    assert_ne!(accepted_legacy_id_manifest_hash(&original_specs), accepted_legacy_id_manifest_hash(&changed_specs));
    assert_ne!(archive_manifest_hash(&original_archive), archive_manifest_hash(&changed_archive));
}
