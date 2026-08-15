#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "readiness observation uses its local path and report vocabulary"
)]
#![allow(
    assertion_density,
    reason = "readiness uses explicit fallible observations and negative filesystem tests"
)]

use std::path::Path;

use serde::Serialize;

pub const CAIRN_PILOT_TYPE_ID: &str = "cairn.repository";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub catalog_identity: String,
    pub type_count: usize,
    pub instance_count: usize,
    pub pilot_instance_count: usize,
    pub pilot_type_revision: String,
    pub executor_identity: String,
    pub credential_reference_count: usize,
}

pub fn inspect(catalog_path: &Path, instances_path: &Path, executable_path: &Path) -> Result<Report, super::Error> {
    let catalog = super::format::load_catalog(catalog_path)?;
    let instances = super::format::load_instances(instances_path, &catalog)?;
    let operation_type = catalog
        .types
        .get(CAIRN_PILOT_TYPE_ID)
        .ok_or_else(|| super::Error::Invalid(String::from("Cairn pilot type is unavailable")))?;
    let executor_identity = super::identity::executor_file(executable_path)?;
    if operation_type.definition.runtime.executor_identity != executor_identity {
        return Err(super::Error::Invalid(String::from("Cairn pilot executor identity is stale")));
    }
    let pilot_instances = instances
        .values()
        .filter(|instance| instance.definition.type_id == CAIRN_PILOT_TYPE_ID)
        .collect::<Vec<_>>();
    if pilot_instances.is_empty() {
        return Err(super::Error::Invalid(String::from("Cairn pilot instance is unavailable")));
    }
    let mut credential_reference_count = 0usize;
    for instance in &pilot_instances {
        validate_repository_root(instance)?;
        credential_reference_count = credential_reference_count
            .checked_add(instance.definition.credentials.len())
            .ok_or_else(|| super::Error::Invalid(String::from("credential reference count overflowed")))?;
    }
    Ok(Report {
        catalog_identity: catalog.identity.clone(),
        type_count: catalog.types.len(),
        instance_count: instances.len(),
        pilot_instance_count: pilot_instances.len(),
        pilot_type_revision: operation_type.definition.revision.clone(),
        executor_identity,
        credential_reference_count,
    })
}

fn validate_repository_root(instance: &super::model::ValidatedInstance) -> Result<(), super::Error> {
    let root = instance
        .definition
        .target
        .get(super::runtime::REPOSITORY_ROOT_TARGET)
        .and_then(serde_json::Value::as_str)
        .map(Path::new)
        .ok_or_else(|| super::Error::Invalid(String::from("Cairn pilot repository root is unavailable")))?;
    if !root.is_absolute() {
        return Err(super::Error::Invalid(String::from("Cairn pilot repository root is not absolute")));
    }
    let metadata = std::fs::symlink_metadata(root).map_err(|_| super::Error::FileAccess)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(super::Error::Invalid(String::from("Cairn pilot repository root is unsafe")));
    }
    Ok(())
}
