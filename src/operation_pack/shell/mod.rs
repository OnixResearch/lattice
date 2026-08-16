#![allow(
    unknown_lints,
    assertion_density,
    reason = "filesystem observations return typed admission errors instead of terminating the process"
)]

use std::io::Read as _;

const MANIFEST_FILE: &str = "manifest.ncl";
const READ_BUFFER_BYTES: usize = 65_536;
const TREE_ENTRY_MULTIPLIER: usize = 2;
const MAXIMUM_TREE_ENTRIES: usize = super::model::MAXIMUM_CONTENT_ITEMS.saturating_mul(TREE_ENTRY_MULTIPLIER);
const IMPORT_TOKEN: &[u8] = b"import";
#[cfg(unix)]
const EXECUTABLE_MODE_MASK: u32 = 0o111;

pub mod execution;
pub mod installation;
pub mod probe;

pub fn validate_manifest_contract(path: &std::path::Path) -> Result<(), super::Error> {
    let value = crate::config::load_nickel_as_json(path).map_err(super::Error::Nickel)?;
    let manifest = serde_json::from_value::<super::model::Manifest>(value)
        .map_err(|_| super::Error::Invalid("strict manifest schema"))?;
    super::validation::validate_manifest(manifest)?;
    Ok(())
}

pub fn load(root: &std::path::Path) -> Result<super::model::Candidate, super::Error> {
    validate_root(root)?;
    let manifest_path = root.join(MANIFEST_FILE);
    let source = read_bounded_regular(&manifest_path, super::model::MAXIMUM_MANIFEST_BYTES)?;
    let source_text = std::str::from_utf8(&source).map_err(|_| super::Error::Invalid("manifest UTF-8"))?;
    if contains_import_token(source_text) {
        return Err(super::Error::Invalid("manifest import"));
    }
    let value = crate::nickel::evaluate_source(source_text).map_err(|error| super::Error::Nickel(error.to_string()))?;
    let decoded = serde_json::from_value::<super::model::Manifest>(value)
        .map_err(|_| super::Error::Invalid("strict manifest schema"))?;
    let manifest = super::validation::validate_manifest(decoded)?;
    let declared_paths = manifest
        .contents
        .iter()
        .map(|content| content.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let observed_paths = collect_files(root)?;
    if observed_paths.len() != declared_paths.len().saturating_add(1)
        || !observed_paths.contains(MANIFEST_FILE)
        || observed_paths
            .iter()
            .filter(|path| path.as_str() != MANIFEST_FILE)
            .any(|path| !declared_paths.contains(path.as_str()))
    {
        return Err(super::Error::Invalid("undeclared pack content"));
    }
    validate_runtime_files(root, &manifest)?;
    let mut measured_contents = Vec::with_capacity(manifest.contents.len());
    for declared in &manifest.contents {
        let bytes = read_bounded_regular(&root.join(&declared.path), declared.byte_length)?;
        let observed_length_bytes =
            u64::try_from(bytes.len()).map_err(|_| super::Error::Invalid("content byte length"))?;
        let observed_identity = super::identity::content_identity(&bytes);
        if declared.role == super::model::ContentRole::OperationCatalog {
            let catalog_source =
                std::str::from_utf8(&bytes).map_err(|_| super::Error::Invalid("operation catalog UTF-8"))?;
            if contains_import_token(catalog_source) {
                return Err(super::Error::Invalid("operation catalog import"));
            }
        }
        if observed_length_bytes != declared.byte_length || observed_identity != declared.content_identity {
            return Err(super::Error::Invalid("content identity"));
        }
        measured_contents.push(super::model::MeasuredContent {
            id: declared.id.clone(),
            path: declared.path.clone(),
            content_identity: observed_identity,
            byte_length: observed_length_bytes,
        });
    }
    let manifest_identity = super::identity::manifest_identity(&manifest)?;
    let pack_identity = super::identity::pack_identity(&manifest)?;
    let source_locator = super::identity::content_identity(root.as_os_str().as_encoded_bytes());
    let candidate = super::model::Candidate {
        manifest,
        manifest_identity,
        pack_identity,
        measured_contents,
        source_locator,
    };
    super::admission::validate_candidate(&candidate)?;
    Ok(candidate)
}

pub fn read_content(
    root: &std::path::Path,
    candidate: &super::model::Candidate,
    content_id: &str,
) -> Result<Vec<u8>, super::Error> {
    let content = candidate
        .manifest
        .contents
        .iter()
        .find(|content| content.id == content_id)
        .ok_or(super::Error::Invalid("content id"))?;
    let bytes = read_bounded_regular(&root.join(&content.path), content.byte_length)?;
    if super::identity::content_identity(&bytes) != content.content_identity {
        return Err(super::Error::Invalid("content changed"));
    }
    Ok(bytes)
}

pub fn content_path(
    root: &std::path::Path,
    candidate: &super::model::Candidate,
    content_id: &str,
) -> Result<std::path::PathBuf, super::Error> {
    let content = candidate
        .manifest
        .contents
        .iter()
        .find(|content| content.id == content_id)
        .ok_or(super::Error::Invalid("content id"))?;
    read_content(root, candidate, content_id)?;
    Ok(root.join(&content.path))
}

pub fn disclose_skill(
    root: &std::path::Path,
    admitted: &super::model::AdmittedPack,
    name: &str,
    include_references: bool,
) -> Result<super::skill::DisclosedBody, super::Error> {
    let skill = admitted
        .candidate
        .manifest
        .skills
        .iter()
        .find(|skill| skill.name == name)
        .ok_or(super::Error::Invalid("skill name"))?;
    let body_bytes = read_content(root, &admitted.candidate, &skill.body_content_id)?;
    super::skill::validate_body(admitted, name, &body_bytes)?;
    let body = String::from_utf8(body_bytes).map_err(|_| super::Error::Invalid("skill body UTF-8"))?;
    let mut references = Vec::new();
    if include_references {
        references.reserve(skill.reference_content_ids.len());
        for content_id in &skill.reference_content_ids {
            let bytes = read_content(root, &admitted.candidate, content_id)?;
            let reference = admitted
                .candidate
                .manifest
                .contents
                .iter()
                .find(|content| content.id == *content_id)
                .ok_or(super::Error::Invalid("skill reference"))?;
            references.push(super::skill::DisclosedReference {
                content_id: content_id.clone(),
                content_identity: reference.content_identity.clone(),
                body: String::from_utf8(bytes).map_err(|_| super::Error::Invalid("skill reference UTF-8"))?,
            });
        }
    }
    Ok(super::skill::DisclosedBody {
        name: skill.name.clone(),
        skill_identity: super::identity::skill_identity(skill)?,
        body_identity: super::identity::content_identity(body.as_bytes()),
        body,
        references,
        grants_authority: false,
    })
}

pub fn load_lock(path: &std::path::Path) -> Result<super::model::Lock, super::Error> {
    let bytes = read_bounded_regular(path, super::model::MAXIMUM_MANIFEST_BYTES)?;
    serde_json::from_slice(&bytes).map_err(|_| super::Error::Invalid("lock schema"))
}

pub fn load_registry(path: &std::path::Path) -> Result<super::registry::Catalog, super::Error> {
    let bytes = read_bounded_regular(path, super::model::MAXIMUM_MANIFEST_BYTES)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| super::Error::Invalid("registry UTF-8"))?;
    if contains_import_token(text) {
        return Err(super::Error::Invalid("registry import"));
    }
    let value = crate::nickel::evaluate_source(text).map_err(|error| super::Error::Nickel(error.to_string()))?;
    let catalog = serde_json::from_value(value).map_err(|_| super::Error::Invalid("registry schema"))?;
    super::registry::validate_catalog(catalog)
}

pub fn platform_label() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

include!("support.rs");
