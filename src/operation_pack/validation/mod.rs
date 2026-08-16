#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "validation formulas use ordered collections and the local primitive vocabulary throughout"
)]
#![allow(
    assertion_density,
    reason = "untrusted manifest values return typed errors instead of terminating the process"
)]
#![allow(
    compound_condition,
    reason = "each compound predicate expresses one closed schema or authority rejection rule"
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
mod primitives;

use primitives::content_map;
use primitives::sort_unique;
use primitives::validate_id;
use primitives::validate_platform;
use primitives::validate_safe_text;
use primitives::validate_scoped_id;

const RUNTIME_KIND: &str = "bounded_process";
const TRUSTED: &str = "trusted";
const UNTRUSTED: &str = "untrusted";
const SKILL_VERSION: &str = "1.0.0";
const SKILL_CATEGORY: &str = "operation-pack";
const ANALYZER_NON_CLAIM: &str = "analyzer output does not strengthen the original operation observation";
const MINIMUM_BOUND: usize = 1;
const MINIMUM_TIMEOUT_SECONDS: u64 = 1;

pub fn validate_manifest(mut manifest: super::model::Manifest) -> Result<super::model::Manifest, super::Error> {
    validate_top_level(&manifest)?;
    normalize(&mut manifest)?;
    validate_contents(&manifest)?;
    validate_runtimes(&manifest)?;
    validate_operations(&manifest)?;
    validate_analyzers(&manifest)?;
    validate_skills(&manifest)?;
    validate_dependencies(&manifest)?;
    validate_profile(&manifest)?;
    validate_release(&manifest)?;
    validate_probe(&manifest)?;
    Ok(manifest)
}

fn validate_top_level(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    if manifest.schema != super::model::MANIFEST_SCHEMA {
        return Err(super::Error::Invalid("manifest schema"));
    }
    validate_scoped_id(&manifest.pack_id)?;
    validate_safe_text(&manifest.description)?;
    if manifest.contents.is_empty() || manifest.contents.len() > super::model::MAXIMUM_CONTENT_ITEMS {
        return Err(super::Error::Invalid("content count"));
    }
    if manifest.operations.is_empty() || manifest.operations.len() > super::model::MAXIMUM_OPERATIONS {
        return Err(super::Error::Invalid("operation count"));
    }
    if manifest.runtimes.is_empty() || manifest.runtimes.len() > super::model::MAXIMUM_RUNTIMES {
        return Err(super::Error::Invalid("runtime count"));
    }
    if manifest.analyzers.len() > super::model::MAXIMUM_ANALYZERS
        || manifest.skills.len() > super::model::MAXIMUM_SKILLS
        || manifest.dependencies.len() > super::model::MAXIMUM_DEPENDENCIES
        || manifest.platforms.is_empty()
        || manifest.platforms.len() > super::model::MAXIMUM_PLATFORMS
        || manifest.licenses.is_empty()
        || manifest.licenses.len() > super::model::MAXIMUM_LICENSES
        || manifest.required_non_claims.len() > super::model::MAXIMUM_NON_CLAIMS
    {
        return Err(super::Error::Invalid("manifest collection bound"));
    }
    for required in &manifest.required_non_claims {
        validate_safe_text(required)?;
    }
    for required in super::model::REQUIRED_NON_CLAIMS {
        if !manifest.required_non_claims.iter().any(|value| value == required) {
            return Err(super::Error::Invalid("required pack non-claim"));
        }
    }
    Ok(())
}

fn normalize(manifest: &mut super::model::Manifest) -> Result<(), super::Error> {
    manifest.contents.sort_by(|left, right| left.id.cmp(&right.id));
    manifest.operations.sort_by(|left, right| left.type_id.cmp(&right.type_id));
    manifest.runtimes.sort_by(|left, right| left.id.cmp(&right.id));
    manifest.analyzers.sort_by(|left, right| left.id.cmp(&right.id));
    manifest.skills.sort_by(|left, right| left.name.cmp(&right.name));
    manifest.dependencies.sort_by(|left, right| left.pack_id.cmp(&right.pack_id));
    sort_unique(&mut manifest.platforms)?;
    sort_unique(&mut manifest.licenses)?;
    sort_unique(&mut manifest.required_non_claims)?;
    sort_unique(&mut manifest.admission.required_evidence)?;
    sort_unique(&mut manifest.admission.allowed_licenses)?;
    sort_unique(&mut manifest.admission.allowed_capabilities)?;
    for operation in &mut manifest.operations {
        sort_unique(&mut operation.methods)?;
    }
    for runtime in &mut manifest.runtimes {
        sort_unique(&mut runtime.launch_labels)?;
        sort_unique(&mut runtime.platforms)?;
        sort_unique(&mut runtime.capabilities)?;
    }
    for analyzer in &mut manifest.analyzers {
        sort_unique(&mut analyzer.data_view)?;
        sort_unique(&mut analyzer.required_non_claims)?;
    }
    for skill in &mut manifest.skills {
        sort_unique(&mut skill.reference_content_ids)?;
        sort_unique(&mut skill.hosts)?;
        sort_unique(&mut skill.positive_routes)?;
        sort_unique(&mut skill.negative_routes)?;
    }
    Ok(())
}

fn validate_contents(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    for content in &manifest.contents {
        validate_id(&content.id)?;
        validate_safe_relative_path(&content.path)?;
        validate_identity(&content.content_identity)?;
        validate_safe_text(&content.media_class)?;
        if content.byte_length == 0 || content.byte_length > super::model::MAXIMUM_PACK_BYTES {
            return Err(super::Error::Invalid("content byte bound"));
        }
        total = total.checked_add(content.byte_length).ok_or(super::Error::Invalid("pack byte overflow"))?;
        if total > super::model::MAXIMUM_PACK_BYTES || !ids.insert(&content.id) || !paths.insert(&content.path) {
            return Err(super::Error::Invalid("duplicate or oversized content"));
        }
    }
    Ok(())
}

fn validate_runtimes(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    let content = content_map(manifest);
    let mut ids = BTreeSet::new();
    for runtime in &manifest.runtimes {
        validate_id(&runtime.id)?;
        if !ids.insert(&runtime.id) || runtime.kind != RUNTIME_KIND || !runtime.clear_environment {
            return Err(super::Error::Invalid("runtime class"));
        }
        let artifact = content.get(runtime.content_id.as_str()).ok_or(super::Error::Invalid("runtime content"))?;
        if artifact.role != super::model::ContentRole::RuntimeArtifact {
            return Err(super::Error::Invalid("runtime content role"));
        }
        if runtime.launch_labels.is_empty() || runtime.platforms.is_empty() || runtime.capabilities.is_empty() {
            return Err(super::Error::Invalid("runtime declaration"));
        }
        if runtime.timeout_seconds < MINIMUM_TIMEOUT_SECONDS
            || runtime.input_bytes < MINIMUM_BOUND
            || runtime.output_bytes < MINIMUM_BOUND
            || runtime.input_bytes > crate::operation::model::MAXIMUM_OPERATION_BYTES
            || runtime.output_bytes > crate::operation::model::MAXIMUM_OPERATION_BYTES
        {
            return Err(super::Error::Invalid("runtime bounds"));
        }
        for launch_label in &runtime.launch_labels {
            validate_id(launch_label)?;
        }
        for platform in &runtime.platforms {
            validate_platform(platform)?;
            if !manifest.platforms.contains(platform) {
                return Err(super::Error::Invalid("runtime platform widening"));
            }
        }
    }
    Ok(())
}

fn validate_operations(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    let content = content_map(manifest);
    let runtimes = manifest.runtimes.iter().map(|runtime| (runtime.id.as_str(), runtime)).collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    for operation in &manifest.operations {
        validate_scoped_id(&operation.type_id)?;
        validate_identity(&operation.type_revision)?;
        if !ids.insert(&operation.type_id) || operation.methods.is_empty() {
            return Err(super::Error::Invalid("operation binding"));
        }
        let catalog = content
            .get(operation.catalog_content_id.as_str())
            .ok_or(super::Error::Invalid("operation catalog content"))?;
        let runtime = runtimes.get(operation.runtime_id.as_str()).ok_or(super::Error::Invalid("operation runtime"))?;
        validate_id(&operation.launch_label)?;
        if catalog.role != super::model::ContentRole::OperationCatalog
            || !runtime.launch_labels.contains(&operation.launch_label)
        {
            return Err(super::Error::Invalid("operation content or runtime"));
        }
        for method in &operation.methods {
            validate_id(method)?;
        }
    }
    Ok(())
}

include!("rules.rs");
