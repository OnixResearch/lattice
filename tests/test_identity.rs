use std::process::Command;

const CANONICAL_PACKAGE_NAME: &str = "lattice";
const LEGACY_PRODUCT_NAME: &str = "n8nclone";

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
