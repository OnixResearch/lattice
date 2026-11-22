//! Pure core for workflow-template bundles carried by operation packs as runbook catalogs.
//!
//! The core never reads files, spawns processes or contacts a repository. The shell supplies
//! measured pack declarations and workflow-template loader observations.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "untrusted pack declarations return typed errors instead of terminating the process"
)]

pub mod decision;
pub mod source;

/// Pack-relative directory that holds every carried bundle as `<root>/<bundle-id>/...`.
pub const BUNDLE_ROOT: &str = "workflow-templates";
/// The bundle manifest that the workflow-template loader requires in each bundle directory.
pub const BUNDLE_MANIFEST: &str = "manifest.ncl";
pub const MAXIMUM_WORKFLOW_BUNDLES: usize = crate::workflow_template::MAX_CATALOG_BUNDLES;
const MAXIMUM_BUNDLE_ID_BYTES: usize = 64;
const PATH_SEPARATOR: char = '/';

/// One workflow-template bundle carried by a pack, pinned to its loader identity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowBundleDeclaration {
    pub id: String,
    pub bundle_identity: String,
}

/// Validate bundle declarations against the declared content placement.
///
/// Every `workflow_bundle` content item must live under `workflow-templates/<declared id>/`,
/// every declared bundle must carry its `manifest.ncl`, and no other role may use that root.
pub fn validate_declarations(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    if manifest.workflow_bundles.len() > MAXIMUM_WORKFLOW_BUNDLES {
        return Err(super::Error::Invalid("workflow bundle count"));
    }
    let mut ids = std::collections::BTreeSet::new();
    for declaration in &manifest.workflow_bundles {
        validate_bundle_id(&declaration.id)?;
        super::validation::validate_identity(&declaration.bundle_identity)?;
        if !ids.insert(declaration.id.as_str()) {
            return Err(super::Error::Invalid("duplicate workflow bundle id"));
        }
    }
    let mut manifests = std::collections::BTreeSet::new();
    for content in &manifest.contents {
        let is_bundle_role = content.role == super::model::ContentRole::WorkflowBundle;
        match (is_bundle_role, is_under_bundle_root(&content.path), bundle_member(&content.path)) {
            (true, true, Some((id, relative))) if ids.contains(id) => {
                if relative == BUNDLE_MANIFEST {
                    manifests.insert(id);
                }
            }
            (false, false, _) => {}
            _ => return Err(super::Error::Invalid("workflow bundle content placement")),
        }
    }
    if manifests.len() != ids.len() {
        return Err(super::Error::Invalid("workflow bundle manifest content"));
    }
    debug_assert!(manifests.len() <= MAXIMUM_WORKFLOW_BUNDLES);
    Ok(())
}

/// Split a pack path `workflow-templates/<id>/<relative>` into its bundle id and member path.
pub fn bundle_member(path: &str) -> Option<(&str, &str)> {
    let rest = path.strip_prefix(BUNDLE_ROOT)?.strip_prefix(PATH_SEPARATOR)?;
    let (id, relative) = rest.split_once(PATH_SEPARATOR)?;
    if id.is_empty() || relative.is_empty() {
        return None;
    }
    Some((id, relative))
}

/// The pack content items that make up one declared bundle, keyed by bundle-relative path.
pub fn bundle_contents<'manifest>(
    manifest: &'manifest super::model::Manifest,
    bundle_id: &str,
) -> std::collections::BTreeMap<&'manifest str, &'manifest super::model::ContentItem> {
    manifest
        .contents
        .iter()
        .filter(|content| content.role == super::model::ContentRole::WorkflowBundle)
        .filter_map(|content| {
            let (id, relative) = bundle_member(&content.path)?;
            (id == bundle_id).then_some((relative, content))
        })
        .collect()
}

/// Bundle ids are the loader's identifier subset that is also one safe path component.
pub fn validate_bundle_id(value: &str) -> Result<(), super::Error> {
    let has_valid_length = !value.is_empty() && value.len() <= MAXIMUM_BUNDLE_ID_BYTES;
    let has_lowercase_prefix = value.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase());
    let has_valid_bytes = value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !(has_valid_length && has_lowercase_prefix && has_valid_bytes) {
        return Err(super::Error::Invalid("workflow bundle id"));
    }
    Ok(())
}

fn is_under_bundle_root(path: &str) -> bool {
    path == BUNDLE_ROOT || path.strip_prefix(BUNDLE_ROOT).is_some_and(|rest| rest.starts_with(PATH_SEPARATOR))
}
