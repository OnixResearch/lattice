use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const CORE_MANIFEST_PATH: &str = "crates/lattice-core/Cargo.toml";
const INVALID_HOST_MANIFEST_PATH: &str = "tests/fixtures/lattice-core-boundary/invalid-host/Cargo.toml";
const MALFORMED_MANIFEST_PATH: &str = "tests/fixtures/lattice-core-boundary/malformed/manifest.toml";
const ALLOWED_DEPENDENCIES: [&str; 5] = [
    "blake3",
    "content-identity-core",
    "rat-nodegraph",
    "serde",
    "serde_json",
];

fn dependency_names(manifest: &str) -> Result<BTreeSet<String>, String> {
    let document = toml::from_str::<toml::Table>(manifest).map_err(|error| error.to_string())?;
    let dependencies = document
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "manifest has no dependencies table".to_string())?;
    Ok(dependencies.keys().cloned().collect())
}

fn forbidden_dependencies(manifest: &str) -> Result<Vec<String>, String> {
    let allowed = ALLOWED_DEPENDENCIES.into_iter().collect::<BTreeSet<_>>();
    let forbidden = dependency_names(manifest)?
        .into_iter()
        .filter(|dependency| !allowed.contains(dependency.as_str()))
        .collect();
    Ok(forbidden)
}

fn read_manifest(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

// r[verify lattice.architecture.core_crate.dependencies]
#[test]
fn current_core_manifest_uses_only_admitted_dependencies() {
    let manifest = read_manifest(Path::new(CORE_MANIFEST_PATH));
    assert_eq!(forbidden_dependencies(&manifest), Ok(Vec::new()));
}

#[test]
fn host_dependency_is_rejected_by_name() {
    let manifest = read_manifest(Path::new(INVALID_HOST_MANIFEST_PATH));
    assert_eq!(forbidden_dependencies(&manifest), Ok(vec!["tokio".to_string()]));
}

#[test]
fn malformed_manifest_fails_closed() {
    let manifest = read_manifest(Path::new(MALFORMED_MANIFEST_PATH));
    assert!(forbidden_dependencies(&manifest).is_err());
}
