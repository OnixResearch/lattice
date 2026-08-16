#![allow(
    unknown_lints,
    assertion_density,
    reason = "publication and removal failures return typed errors instead of terminating the process"
)]
#![allow(
    ambiguous_params,
    reason = "public install and remove boundaries retain explicit names for host and exact identity inputs"
)]
#![allow(
    path_segment_repetition,
    reason = "pack terminology stays explicit at create-only publication boundaries"
)]
#![allow(
    compound_condition,
    reason = "file-name admission is one closed lexical safety predicate"
)]

use std::io::Write as _;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallReceipt {
    pub schema: String,
    pub pack_identity: String,
    pub lock_identity: String,
    pub installed_content_count: usize,
    pub source_content_omitted: bool,
}

pub fn write_lock_create_only(
    destination: &std::path::Path,
    lock: &super::super::model::Lock,
) -> Result<(), super::super::Error> {
    let bytes = serde_json::to_vec_pretty(lock).map_err(|_| super::super::Error::Encoding)?;
    write_file_create_only(destination, &bytes)
}

pub fn install_pack_create_only(
    admitted: &super::super::model::AdmittedPack,
    source_root: &std::path::Path,
    destination: &std::path::Path,
) -> Result<InstallReceipt, super::super::Error> {
    if destination.exists() {
        return Err(super::super::Error::Conflict("pack installation destination exists"));
    }
    let parent = destination.parent().ok_or(super::super::Error::Invalid("installation parent"))?;
    validate_directory(parent)?;
    let name = safe_file_name(destination)?;
    let staging_name = format!(".{name}.staging");
    let staging = parent.join(&staging_name);
    if staging.exists() {
        return Err(super::super::Error::Conflict("pack installation staging exists"));
    }
    std::fs::create_dir(&staging).map_err(|_| super::super::Error::Io("pack staging create"))?;
    let result = copy_pack(admitted, source_root, &staging);
    if let Err(error) = result {
        remove_staging(&staging)?;
        return Err(error);
    }
    let staged = match super::load(&staging) {
        Ok(staged) => staged,
        Err(error) => {
            remove_staging(&staging)?;
            return Err(error);
        }
    };
    if let Err(error) = super::super::admission::validate_lock(&staged, &admitted.lock) {
        remove_staging(&staging)?;
        return Err(error);
    }
    std::fs::rename(&staging, destination).map_err(|_| super::super::Error::Io("pack installation publish"))?;
    sync_directory(parent)?;
    Ok(InstallReceipt {
        schema: super::super::model::INSTALL_RECEIPT_SCHEMA.to_string(),
        pack_identity: admitted.candidate.pack_identity.clone(),
        lock_identity: admitted.lock.identity.clone(),
        installed_content_count: admitted.candidate.manifest.contents.len(),
        source_content_omitted: true,
    })
}

pub fn install_skill_create_only(
    admitted: &super::super::model::AdmittedPack,
    source_root: &std::path::Path,
    skill_name: &str,
    host: &str,
    destination_root: &std::path::Path,
) -> Result<super::super::skill::InstallReceipt, super::super::Error> {
    validate_directory(destination_root)?;
    let skill = admitted
        .candidate
        .manifest
        .skills
        .iter()
        .find(|skill| skill.name == skill_name)
        .ok_or(super::super::Error::Invalid("skill name"))?;
    if !skill.hosts.iter().any(|allowed| allowed == host) || skill.trust != "trusted" {
        return Err(super::super::Error::Invalid("skill host or trust"));
    }
    let body = super::read_content(source_root, &admitted.candidate, &skill.body_content_id)?;
    super::super::skill::validate_body(admitted, skill_name, &body)?;
    let destination = destination_root.join(skill_name);
    if destination.exists() {
        return Err(super::super::Error::Conflict("skill installation exists"));
    }
    let staging = destination_root.join(format!(".{skill_name}.staging"));
    if staging.exists() {
        return Err(super::super::Error::Conflict("skill staging exists"));
    }
    std::fs::create_dir(&staging).map_err(|_| super::super::Error::Io("skill staging create"))?;
    let receipt = super::super::skill::InstallReceipt {
        schema: super::super::model::SKILL_INSTALL_RECEIPT_SCHEMA.to_string(),
        pack_identity: admitted.candidate.pack_identity.clone(),
        skill_identity: super::super::identity::skill_identity(skill)?,
        body_identity: super::super::identity::content_identity(&body),
        host: host.to_string(),
        grants_authority: false,
    };
    let write_result = (|| {
        write_file_create_only(&staging.join("SKILL.md"), &body)?;
        let receipt_bytes = serde_json::to_vec_pretty(&receipt).map_err(|_| super::super::Error::Encoding)?;
        write_file_create_only(&staging.join("install.json"), &receipt_bytes)?;
        if !skill.reference_content_ids.is_empty() {
            std::fs::create_dir(staging.join("references"))
                .map_err(|_| super::super::Error::Io("skill references create"))?;
        }
        for reference_id in &skill.reference_content_ids {
            let reference = super::read_content(source_root, &admitted.candidate, reference_id)?;
            write_file_create_only(&staging.join("references").join(format!("{reference_id}.md")), &reference)?;
        }
        Ok(())
    })();
    if let Err(error) = write_result {
        remove_staging(&staging)?;
        return Err(error);
    }
    std::fs::rename(&staging, &destination).map_err(|_| super::super::Error::Io("skill installation publish"))?;
    sync_directory(destination_root)?;
    Ok(receipt)
}

pub fn remove_skill(
    destination_root: &std::path::Path,
    skill_name: &str,
    expected_skill_identity: &str,
) -> Result<(), super::super::Error> {
    validate_directory(destination_root)?;
    super::super::validation::validate_identity(expected_skill_identity)?;
    let destination = destination_root.join(skill_name);
    let destination_metadata = std::fs::symlink_metadata(&destination)
        .map_err(|_| super::super::Error::Io("skill destination observation"))?;
    if destination_metadata.file_type().is_symlink() || !destination_metadata.is_dir() {
        return Err(super::super::Error::Invalid("skill destination file type"));
    }
    let receipt_bytes =
        std::fs::read(destination.join("install.json")).map_err(|_| super::super::Error::Io("skill receipt read"))?;
    let receipt: super::super::skill::InstallReceipt =
        serde_json::from_slice(&receipt_bytes).map_err(|_| super::super::Error::Invalid("skill receipt"))?;
    let body = std::fs::read(destination.join("SKILL.md")).map_err(|_| super::super::Error::Io("skill body read"))?;
    if receipt.skill_identity != expected_skill_identity
        || receipt.body_identity != super::super::identity::content_identity(&body)
    {
        return Err(super::super::Error::Conflict("installed skill identity differs"));
    }
    std::fs::remove_dir_all(destination).map_err(|_| super::super::Error::Io("skill remove"))?;
    sync_directory(destination_root)
}

pub fn write_registry_create_only(
    destination: &std::path::Path,
    catalog: &super::super::registry::Catalog,
) -> Result<(), super::super::Error> {
    let validated = super::super::registry::validate_catalog(catalog.clone())?;
    let value = serde_json::to_value(validated).map_err(|_| super::super::Error::Encoding)?;
    let rendered =
        crate::orchestration::format::render_nickel_value(&value, 0).map_err(|_| super::super::Error::Encoding)?;
    write_file_create_only(destination, rendered.as_bytes())
}

fn copy_pack(
    admitted: &super::super::model::AdmittedPack,
    source_root: &std::path::Path,
    staging: &std::path::Path,
) -> Result<(), super::super::Error> {
    let manifest =
        std::fs::read(source_root.join("manifest.ncl")).map_err(|_| super::super::Error::Io("manifest copy read"))?;
    write_file_create_only(&staging.join("manifest.ncl"), &manifest)?;
    for content in &admitted.candidate.manifest.contents {
        let bytes = super::read_content(source_root, &admitted.candidate, &content.id)?;
        let destination = staging.join(&content.path);
        let parent = destination.parent().ok_or(super::super::Error::Invalid("content destination parent"))?;
        std::fs::create_dir_all(parent).map_err(|_| super::super::Error::Io("content directory create"))?;
        write_file_create_only(&destination, &bytes)?;
        let source_permissions = std::fs::metadata(source_root.join(&content.path))
            .map_err(|_| super::super::Error::Io("content permissions read"))?
            .permissions();
        std::fs::set_permissions(&destination, source_permissions)
            .map_err(|_| super::super::Error::Io("content permissions write"))?;
    }
    sync_directory(staging)
}

fn write_file_create_only(path: &std::path::Path, bytes: &[u8]) -> Result<(), super::super::Error> {
    let parent = path.parent().ok_or(super::super::Error::Invalid("file parent"))?;
    validate_directory(parent)?;
    let name = safe_file_name(path)?;
    let directory = cap_std::fs::Dir::open_ambient_dir(parent, cap_std::ambient_authority())
        .map_err(|_| super::super::Error::Io("file parent authority"))?;
    let mut options = cap_std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = directory
        .open_with(name, &options)
        .map_err(|_| super::super::Error::Conflict("create-only file exists"))?;
    file.write_all(bytes).map_err(|_| super::super::Error::Io("file write"))?;
    file.sync_all().map_err(|_| super::super::Error::Io("file sync"))?;
    directory
        .open(".")
        .and_then(|opened| opened.sync_all())
        .map_err(|_| super::super::Error::Io("file parent sync"))
}

fn validate_directory(path: &std::path::Path) -> Result<(), super::super::Error> {
    if !path.is_absolute() {
        return Err(super::super::Error::Invalid("directory must be absolute"));
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|_| super::super::Error::Io("directory observation"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(super::super::Error::Invalid("directory file type"));
    }
    Ok(())
}

fn safe_file_name(path: &std::path::Path) -> Result<&str, super::super::Error> {
    let name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or(super::super::Error::Invalid("file name"))?;
    if name.is_empty() || !name.is_ascii() || matches!(name, "." | "..") || name.contains('/') || name.contains('\\') {
        return Err(super::super::Error::Invalid("file name"));
    }
    Ok(name)
}

fn remove_staging(path: &std::path::Path) -> Result<(), super::super::Error> {
    std::fs::remove_dir_all(path).map_err(|_| super::super::Error::Io("staging cleanup"))
}

fn sync_directory(path: &std::path::Path) -> Result<(), super::super::Error> {
    std::fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| super::super::Error::Io("directory sync"))
}
