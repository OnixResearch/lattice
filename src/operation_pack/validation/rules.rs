fn validate_analyzers(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    let runtimes = manifest.runtimes.iter().map(|runtime| (&runtime.id, runtime)).collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    for analyzer in &manifest.analyzers {
        validate_id(&analyzer.id)?;
        validate_identity(&analyzer.output_schema_identity)?;
        validate_identity(&analyzer.retention_identity)?;
        if !ids.insert(&analyzer.id) || analyzer.data_view.is_empty() {
            return Err(super::Error::Invalid("analyzer declaration"));
        }
        let runtime = runtimes.get(&analyzer.runtime_id).ok_or(super::Error::Invalid("analyzer runtime"))?;
        if runtime.kind == super::wasm::RUNTIME_KIND {
            return Err(super::Error::Invalid("component analyzer unsupported"));
        }
        if !runtime.launch_labels.contains(&analyzer.launch_label) {
            return Err(super::Error::Invalid("analyzer launch label"));
        }
        if analyzer.input_bytes < MINIMUM_BOUND
            || analyzer.input_bytes > runtime.input_bytes
            || analyzer.timeout_seconds < MINIMUM_TIMEOUT_SECONDS
            || analyzer.timeout_seconds > runtime.timeout_seconds
            || analyzer.output_bytes < MINIMUM_BOUND
            || analyzer.output_bytes > runtime.output_bytes
            || analyzer.maximum_markdown_bytes == 0
            || analyzer.maximum_markdown_bytes > analyzer.output_bytes
            || analyzer.maximum_markdown_bytes > super::model::MAXIMUM_ANALYZER_MARKDOWN_BYTES
            || !analyzer.required_non_claims.iter().any(|value| value == ANALYZER_NON_CLAIM)
        {
            return Err(super::Error::Invalid("analyzer policy"));
        }
        for non_claim in &analyzer.required_non_claims {
            validate_safe_text(non_claim)?;
        }
    }
    Ok(())
}

fn validate_skills(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    let content = content_map(manifest);
    let mut names = BTreeSet::new();
    for skill in &manifest.skills {
        validate_id(&skill.name)?;
        validate_safe_text(&skill.description)?;
        if !names.insert(&skill.name) || skill.hosts.is_empty() || skill.positive_routes.is_empty() {
            return Err(super::Error::Invalid("skill declaration"));
        }
        let body = content.get(skill.body_content_id.as_str()).ok_or(super::Error::Invalid("skill body"))?;
        if body.role != super::model::ContentRole::SkillBody {
            return Err(super::Error::Invalid("skill body role"));
        }
        for reference in &skill.reference_content_ids {
            let item = content.get(reference.as_str()).ok_or(super::Error::Invalid("skill reference"))?;
            if item.role != super::model::ContentRole::SkillReference {
                return Err(super::Error::Invalid("skill reference role"));
            }
        }
        if skill.trust != TRUSTED && skill.trust != UNTRUSTED {
            return Err(super::Error::Invalid("skill trust"));
        }
        for host in &skill.hosts {
            validate_id(host)?;
        }
        for route in skill.positive_routes.iter().chain(&skill.negative_routes) {
            validate_safe_text(route)?;
        }
        let tags = ["lattice", "operation-pack"];
        animus_adapters::SkillManifest::validate(&animus_adapters::SkillManifestDraft {
            name: Some(&skill.name),
            description: Some(&skill.description),
            version: Some(SKILL_VERSION),
            category: Some(SKILL_CATEGORY),
            tags: &tags,
            related_skills: &[],
            provenance: Some(&manifest.pack_id),
            trust_status: Some(&skill.trust),
            content_identity: Some(
                body.content_identity.strip_prefix("b3:").ok_or(super::Error::Invalid("skill content identity"))?,
            ),
        })
        .map_err(|_| super::Error::Invalid("Animus skill manifest"))?;
    }
    Ok(())
}

fn validate_dependencies(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    let mut ids = BTreeSet::new();
    for dependency in &manifest.dependencies {
        validate_scoped_id(&dependency.pack_id)?;
        validate_identity(&dependency.pack_identity)?;
        if dependency.pack_id == manifest.pack_id || !ids.insert(&dependency.pack_id) {
            return Err(super::Error::Invalid("dependency cycle or duplicate"));
        }
    }
    Ok(())
}

fn validate_profile(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    validate_identity(&manifest.admission.identity)?;
    if manifest.admission.allowed_licenses.is_empty() {
        return Err(super::Error::Invalid("admission profile"));
    }
    for license in manifest.licenses.iter().chain(&manifest.admission.allowed_licenses) {
        validate_safe_text(license)?;
    }
    if manifest.licenses.iter().any(|license| !manifest.admission.allowed_licenses.contains(license))
        || manifest
            .runtimes
            .iter()
            .flat_map(|runtime| &runtime.capabilities)
            .any(|capability| !manifest.admission.allowed_capabilities.contains(capability))
    {
        return Err(super::Error::Invalid("admission profile contradiction"));
    }
    for platform in &manifest.platforms {
        validate_platform(platform)?;
    }
    Ok(())
}

fn validate_release(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    validate_safe_text(&manifest.release.display_version)?;
    if manifest.release.superseded_by.is_some() && !manifest.release.deprecated {
        return Err(super::Error::Invalid("release supersession contradiction"));
    }
    if let Some(identity) = &manifest.release.superseded_by {
        validate_identity(identity)?;
        if identity == &super::identity::pack_identity(manifest)? {
            return Err(super::Error::Invalid("release self supersession"));
        }
    }
    if manifest.release.publication_epoch_seconds.is_some_and(|epoch| epoch < 0) {
        return Err(super::Error::Invalid("release publication epoch"));
    }
    Ok(())
}

fn validate_probe(manifest: &super::model::Manifest) -> Result<(), super::Error> {
    let Some(probe) = &manifest.probe else {
        return Ok(());
    };
    if probe.effectful || probe.uses_production_credentials {
        return Err(super::Error::Invalid("doctor probe authority"));
    }
    let runtime = manifest
        .runtimes
        .iter()
        .find(|runtime| runtime.id == probe.runtime_id)
        .ok_or(super::Error::Invalid("doctor probe runtime"))?;
    if runtime.kind == super::wasm::RUNTIME_KIND {
        return Err(super::Error::Invalid("component probe unsupported"));
    }
    if !runtime.launch_labels.contains(&probe.launch_label) {
        return Err(super::Error::Invalid("doctor probe launch label"));
    }
    Ok(())
}

pub use primitives::validate_identity;
pub use primitives::validate_safe_relative_path;
