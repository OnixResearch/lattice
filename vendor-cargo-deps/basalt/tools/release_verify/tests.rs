use super::model::{
    NICKEL_EVALUATOR_IDENTITY, PRIVATE_PUBLISH_SETTING, REQUIRED_LICENSE, REQUIRED_PACKAGE_FILES,
    REQUIRED_PRIVATE_SOURCE_FILES,
};
use super::validators::{
    validate_nickel_identity, validate_package_contents, validate_package_metadata,
    validate_private_source_inventory,
};

const OMITTED_SYNC_MODULE: &str = "tools/sync_vendored_ucan/process.rs";
const OMITTED_PRIVATE_MANIFEST: &str = "vendor/ucan/Cargo.toml";
const PRIVATE_METADATA_SUFFIX: &str = "\"publish\":[]";
const PUBLIC_METADATA_SUFFIX: &str = "\"publish\":[\"crates-io\"]";
const WRONG_NICKEL_IDENTITY: &str = "nickel-lang-cli nickel 1.16.0 (rev stale)";

fn metadata(suffix: &str) -> String {
    format!("{{\"license\":\"{REQUIRED_LICENSE}\",{suffix}}}")
}

#[test]
fn exact_nickel_identity_is_admitted() {
    assert!(validate_nickel_identity(NICKEL_EVALUATOR_IDENTITY).is_ok());
}

#[test]
fn stale_nickel_identity_is_rejected() {
    assert!(validate_nickel_identity(WRONG_NICKEL_IDENTITY).is_err());
}

#[test]
fn private_package_inventory_accepts_the_complete_reviewed_boundary() {
    let package_list = REQUIRED_PACKAGE_FILES.join("\n");
    let manifest =
        format!("[package]\n{PRIVATE_PUBLISH_SETTING}\nlicense = \"{REQUIRED_LICENSE}\"\n");

    assert!(validate_package_contents(&package_list).is_ok());
    assert!(validate_private_source_inventory(REQUIRED_PRIVATE_SOURCE_FILES).is_ok());
    assert!(validate_package_metadata(&manifest, &metadata(PRIVATE_METADATA_SUFFIX)).is_ok());
}

#[test]
fn private_package_inventory_rejects_an_omitted_sync_module() {
    let package_list = REQUIRED_PACKAGE_FILES
        .iter()
        .filter(|path| **path != OMITTED_SYNC_MODULE)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    let error = validate_package_contents(&package_list)
        .expect_err("missing bootstrap module must fail package validation");

    assert!(error.contains(OMITTED_SYNC_MODULE));
}

#[test]
fn private_source_inventory_rejects_an_omitted_vendor_manifest() {
    let inventory = REQUIRED_PRIVATE_SOURCE_FILES
        .iter()
        .filter(|path| **path != OMITTED_PRIVATE_MANIFEST)
        .copied()
        .collect::<Vec<_>>();
    let error = validate_private_source_inventory(&inventory)
        .expect_err("missing vendored manifest must fail private source validation");

    assert!(error.contains(OMITTED_PRIVATE_MANIFEST));
}

#[test]
fn package_metadata_rejects_a_manifest_without_private_publish_setting() {
    let manifest = format!("[package]\nlicense = \"{REQUIRED_LICENSE}\"\n");

    assert!(validate_package_metadata(&manifest, &metadata(PRIVATE_METADATA_SUFFIX)).is_err());
}

#[test]
fn package_metadata_rejects_registry_publish_metadata() {
    let manifest =
        format!("[package]\n{PRIVATE_PUBLISH_SETTING}\nlicense = \"{REQUIRED_LICENSE}\"\n");

    assert!(validate_package_metadata(&manifest, &metadata(PUBLIC_METADATA_SUFFIX)).is_err());
}
