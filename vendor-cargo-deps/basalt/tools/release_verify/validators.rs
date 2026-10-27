use super::model::{
    NICKEL_EVALUATOR_IDENTITY, PRIVATE_PUBLISH_SETTING, REQUIRED_LICENSE,
    REQUIRED_PACKAGE_FILES, REQUIRED_PRIVATE_SOURCE_FILES,
};
use std::fs;
use std::path::Path;

pub(super) fn validate_nickel_identity(output: &str) -> Result<(), String> {
    if output.trim() == NICKEL_EVALUATOR_IDENTITY {
        Ok(())
    } else {
        Err(format!(
            "Nickel evaluator identity mismatch: expected {NICKEL_EVALUATOR_IDENTITY}, got {}",
            output.trim()
        ))
    }
}

pub(super) fn validate_package_contents(package_list: &str) -> Result<(), String> {
    let missing = REQUIRED_PACKAGE_FILES
        .iter()
        .filter(|path| !package_list.lines().any(|line| line.trim() == **path))
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "package contents missing required paths: {}",
            missing.join(", ")
        ))
    }
}

pub(super) fn validate_private_source(root: &Path) -> Result<(), String> {
    let present = REQUIRED_PRIVATE_SOURCE_FILES
        .iter()
        .filter(|path| root.join(path).is_file())
        .copied()
        .collect::<Vec<_>>();
    validate_private_source_inventory(&present)
}

pub(super) fn validate_private_source_inventory(inventory: &[&str]) -> Result<(), String> {
    let missing = REQUIRED_PRIVATE_SOURCE_FILES
        .iter()
        .filter(|path| !inventory.contains(path))
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "private source boundary missing required paths: {}",
            missing.join(", ")
        ))
    }
}

pub(super) fn validate_license_metadata(
    manifest_path: &Path,
    metadata_json: &str,
) -> Result<(), String> {
    let manifest = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "could not read manifest for license metadata check {}: {error}",
            manifest_path.display()
        )
    })?;
    validate_package_metadata(&manifest, metadata_json)
}

pub(super) fn validate_package_metadata(manifest: &str, metadata_json: &str) -> Result<(), String> {
    let manifest_license = format!("license = \"{REQUIRED_LICENSE}\"");
    let metadata_license = format!("\"license\":\"{REQUIRED_LICENSE}\"");
    if !manifest.lines().any(|line| line.trim() == manifest_license) {
        return Err(format!(
            "manifest license metadata must remain {REQUIRED_LICENSE}"
        ));
    }
    if !metadata_json.contains(&metadata_license) {
        return Err(format!(
            "cargo metadata did not report package license {REQUIRED_LICENSE}"
        ));
    }
    if !manifest
        .lines()
        .any(|line| line.trim() == PRIVATE_PUBLISH_SETTING)
    {
        return Err("manifest must keep Basalt private with publish = false".to_owned());
    }
    if !metadata_json.contains("\"publish\":[]") {
        return Err("cargo metadata did not report Basalt as private".to_owned());
    }
    Ok(())
}
