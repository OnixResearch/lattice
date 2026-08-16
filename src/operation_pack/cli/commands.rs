fn lock_command(args: &LockArgs) -> Result<bool, super::Error> {
    let candidate = super::shell::load(&args.source.pack)?;
    let facts = admission_facts(&candidate, &args.trust)?;
    let lock = super::create_lock(&candidate, &facts)?;
    if args.confirm_create {
        super::shell::installation::write_lock_create_only(&args.destination, &lock)?;
    }
    print_json(&serde_json::json!({
        "lock": lock,
        "created": args.confirm_create,
        "required_flag": (!args.confirm_create).then_some("--confirm-create")
    }))?;
    Ok(true)
}

fn install_command(args: &InstallArgs) -> Result<bool, super::Error> {
    let admitted = admitted_pack(&args.source.pack, &args.lock, &args.trust)?;
    if !args.confirm_create {
        print_json(&serde_json::json!({
            "pack_identity": admitted.candidate.pack_identity,
            "lock_identity": admitted.lock.identity,
            "installed": false,
            "required_flag": "--confirm-create"
        }))?;
        return Ok(true);
    }
    let receipt =
        super::shell::installation::install_pack_create_only(&admitted, &args.source.pack, &args.destination)?;
    print_json(&receipt)?;
    Ok(true)
}

fn run_command(app_config: &crate::config::AppConfig, args: &RunArgs) -> Result<bool, super::Error> {
    let admitted = admitted_pack(&args.source.pack, &args.lock, &args.trust)?;
    if !args.confirm_read_only {
        print_json(&serde_json::json!({
            "pack_identity": admitted.candidate.pack_identity,
            "lock_identity": admitted.lock.identity,
            "instance": args.instance,
            "method": args.method,
            "executed": false,
            "required_flag": "--confirm-read-only"
        }))?;
        return Ok(true);
    }
    let report = super::shell::execution::run(super::shell::execution::Request {
        app_config,
        pack_root: &args.source.pack,
        admitted: &admitted,
        instances_path: &args.instances,
        instance_id: &args.instance,
        method: &args.method,
        arguments_json: &args.arguments_json,
        sequence: args.sequence,
        prohibited_fragments: &[],
        cancellation: None,
    })?;
    let is_completed = report.status == super::shell::execution::Status::Completed;
    print_json(&report)?;
    Ok(is_completed)
}

fn skill_show_command(args: &SkillShowArgs) -> Result<bool, super::Error> {
    let admitted = admitted_pack(&args.source.pack, &args.lock, &args.trust)?;
    let disclosed = super::shell::disclose_skill(&args.source.pack, &admitted, &args.name, args.include_references)?;
    print_json(&disclosed)?;
    Ok(true)
}

fn skill_install_command(args: &SkillInstallArgs) -> Result<bool, super::Error> {
    let admitted = admitted_pack(&args.show.source.pack, &args.show.lock, &args.show.trust)?;
    if !args.confirm_create {
        print_json(&serde_json::json!({
            "pack_identity": admitted.candidate.pack_identity,
            "skill": args.show.name,
            "installed": false,
            "required_flag": "--confirm-create"
        }))?;
        return Ok(true);
    }
    let receipt = super::shell::installation::install_skill_create_only(
        &admitted,
        &args.show.source.pack,
        &args.show.name,
        &args.host,
        &args.destination_root,
    )?;
    print_json(&receipt)?;
    Ok(true)
}

fn skill_remove_command(args: &SkillRemoveArgs) -> Result<bool, super::Error> {
    if !args.confirm_remove {
        print_json(&serde_json::json!({
            "skill": args.name,
            "removed": false,
            "required_flag": "--confirm-remove"
        }))?;
        return Ok(true);
    }
    super::shell::installation::remove_skill(&args.destination_root, &args.name, &args.skill_identity)?;
    print_json(&serde_json::json!({"skill_identity": args.skill_identity, "removed": true}))?;
    Ok(true)
}

fn registry_search_command(args: &RegistrySearchArgs) -> Result<bool, super::Error> {
    let catalog = super::shell::load_registry(&args.catalog)?;
    let entries = super::registry::search(&catalog, &args.query);
    let freshness = super::registry::freshness(Some(&catalog), args.observed_epoch_seconds, args.maximum_age_seconds);
    print_json(&serde_json::json!({"entries": entries, "freshness": freshness, "mutated": false}))?;
    Ok(true)
}

fn registry_info_command(args: &RegistryInfoArgs) -> Result<bool, super::Error> {
    let catalog = super::shell::load_registry(&args.catalog)?;
    let entry = super::registry::exact(&catalog, &args.pack_identity)?;
    print_json(&serde_json::json!({"entry": entry, "content_resolved": false, "mutated": false}))?;
    Ok(true)
}

fn registry_restore_command(args: &RegistryRestoreArgs) -> Result<bool, super::Error> {
    let catalog = super::shell::load_registry(&args.catalog)?;
    let plan = super::registry::restore_plan(&catalog, &args.pack_identity, &args.destination)?;
    print_json(&plan)?;
    Ok(true)
}

fn registry_promote_command(args: &RegistryPromoteArgs) -> Result<bool, super::Error> {
    let catalog = super::shell::load_registry(&args.catalog)?;
    let promoted = super::registry::promote(&catalog, &args.pack_identity, args.target.into())?;
    if args.confirm_create {
        super::shell::installation::write_registry_create_only(&args.destination, &promoted)?;
    }
    print_json(&serde_json::json!({
        "catalog": promoted,
        "created": args.confirm_create,
        "pack_bytes_changed": false,
        "required_flag": (!args.confirm_create).then_some("--confirm-create")
    }))?;
    Ok(true)
}

fn doctor_command(app_config: &crate::config::AppConfig, args: &DoctorArgs) -> Result<bool, super::Error> {
    let candidate = super::shell::load(&args.source.pack);
    let lock = args.lock.as_deref().map(super::shell::load_lock).transpose()?;
    let facts = candidate.as_ref().ok().map(|candidate| admission_facts(candidate, &args.trust)).transpose()?;
    let mut report =
        super::doctor::evaluate(candidate.as_ref(), lock.as_ref(), facts.as_ref(), super::doctor::ShellFacts {
            cache_present: args.cache_present,
            registry_configured: args.registry_configured,
            registry_available: args.registry_available,
        });
    if args.run_probe {
        let candidate = candidate?;
        let lock = lock.ok_or(super::Error::Invalid("doctor probe lock"))?;
        let facts = facts.ok_or(super::Error::Invalid("doctor probe admission facts"))?;
        let admitted = super::admit(&candidate, &lock, &facts)?;
        let process = app_config.effective_agent_runtime().local_process;
        let outcome = super::shell::probe::run(&args.source.pack, &admitted, &process, None)?;
        let has_succeeded = outcome.status == super::shell::probe::Status::Succeeded;
        report.probe_outcome = Some(outcome);
        if !has_succeeded {
            report.healthy = false;
            report.findings.push(super::doctor::Finding {
                id: "probe_execution".to_string(),
                severity: super::doctor::Severity::Fatal,
                message: "the declared no-effect probe did not complete".to_string(),
                remediation: Some("inspect the exact runtime artifact and its bounded output".to_string()),
            });
        }
    }
    let is_healthy = report.healthy;
    print_json(&report)?;
    Ok(is_healthy)
}

fn admitted_pack(
    pack: &std::path::Path,
    lock_path: &std::path::Path,
    trust: &TrustArgs,
) -> Result<super::model::AdmittedPack, super::Error> {
    let candidate = super::shell::load(pack)?;
    let lock = super::shell::load_lock(lock_path)?;
    let facts = admission_facts(&candidate, trust)?;
    super::admit(&candidate, &lock, &facts)
}

fn admission_facts(
    candidate: &super::model::Candidate,
    trust: &TrustArgs,
) -> Result<super::model::AdmissionFacts, super::Error> {
    if let Some(path) = &trust.admission_facts {
        let metadata = std::fs::symlink_metadata(path).map_err(|_| super::Error::Io("admission facts observation"))?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() > super::model::MAXIMUM_MANIFEST_BYTES
        {
            return Err(super::Error::Invalid("admission facts file"));
        }
        let value = crate::config::load_nickel_as_json(path).map_err(super::Error::Nickel)?;
        let facts = serde_json::from_value(value).map_err(|_| super::Error::Invalid("admission facts schema"))?;
        return Ok(facts);
    }
    if !trust.confirm_trust_local {
        return Err(super::Error::Invalid("explicit local trust confirmation"));
    }
    Ok(super::model::AdmissionFacts {
        trusted_pack_identity: candidate.pack_identity.clone(),
        platform: super::shell::platform_label(),
        dependency_identities: std::collections::BTreeMap::new(),
        allowed_licenses: trust.allow_license.clone(),
        allowed_capabilities: candidate.manifest.admission.allowed_capabilities.clone(),
        evidence: std::collections::BTreeMap::new(),
    })
}

fn print_json(value: &impl serde::Serialize) -> Result<(), super::Error> {
    let output = serde_json::to_string_pretty(value).map_err(|_| super::Error::Encoding)?;
    println!("{output}");
    Ok(())
}

impl From<ChannelArg> for super::model::ReleaseChannel {
    fn from(value: ChannelArg) -> Self {
        match value {
            ChannelArg::Beta => Self::Beta,
            ChannelArg::Rc => Self::Rc,
            ChannelArg::Stable => Self::Stable,
        }
    }
}
