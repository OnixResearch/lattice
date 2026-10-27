//! Operator smoke for the real bounded provider. Never prints resolved material.
//! Run with `--features animus-secretspec --example secretspec_delivery`.

#[cfg(feature = "animus-secretspec")]
fn main() -> std::process::ExitCode {
    match delivery() {
        Ok(()) => {
            println!("credential_secretspec_delivered; material omitted");
            std::process::ExitCode::SUCCESS
        }
        Err(code) => {
            eprintln!("{code}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(feature = "animus-secretspec"))]
fn main() -> std::process::ExitCode {
    eprintln!("requires --features animus-secretspec");
    std::process::ExitCode::FAILURE
}

#[cfg(feature = "animus-secretspec")]
fn delivery() -> Result<(), &'static str> {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    use animus_std_secretspec::ResolverDeployment;
    use animus_std_secretspec::SopsCredentialName;
    use lattice::credentials::CredentialDestination;
    use lattice::credentials::CredentialProvider;
    use lattice::credentials::CredentialResolutionContext;
    use lattice::credentials::CredentialSubject;
    use lattice::credentials::SecretRef;
    use lattice::credentials::config::ProviderScheme;
    use lattice::credentials::config::SecretSpecProviderConfig;
    use lattice::credentials::secretspec::AnimusSecretSpecProvider;
    use lattice::credentials::secretspec::BoundedSecretSpecHost;
    use lattice::credentials::secretspec::HostInputSource;

    // Files are explicit operator inputs, not ambient SecretSpec discovery.
    // config.json policy.json manifest worker cwd source [SOPS guard-directory]
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if !(6..=7).contains(&args.len()) {
        return Err(
            "usage: secretspec_delivery CONFIG_JSON POLICY_JSON MANIFEST WORKER CWD SOURCE [SOPS_GUARD_DIRECTORY]",
        );
    }
    let config_bytes = std::fs::read(&args[0]).map_err(|_| "credential_secretspec_configuration_unavailable")?;
    let config: SecretSpecProviderConfig =
        serde_json::from_slice(&config_bytes).map_err(|_| "credential_secretspec_invalid_configuration")?;
    config.validate().map_err(|error| error.code())?;
    let policy_bytes = std::fs::read(&args[1]).map_err(|_| "credential_secretspec_policy_unavailable")?;
    let mapping = config.declarations.first().ok_or("credential_secretspec_missing_declaration")?;
    let source = match mapping.scheme {
        ProviderScheme::Environment => HostInputSource::Environment {
            variable: args[5].to_str().ok_or("credential_secretspec_invalid_configuration")?.into(),
        },
        ProviderScheme::SystemdCredential => HostInputSource::SystemdCredential {
            directory: PathBuf::from(&args[5]),
        },
        ProviderScheme::Sops => HostInputSource::Sops {
            guard_directory: PathBuf::from(args.get(6).ok_or("credential_secretspec_guard_required")?),
            executable: PathBuf::from(&args[5]),
            // This smoke explicitly selects age; the production host supports the
            // other named provider-credential inputs without inheriting the env.
            credential_variables: vec![(SopsCredentialName::AgeKey, "LATTICE_SMOKE_SOPS_AGE_KEY".into())],
        },
    };
    let host = BoundedSecretSpecHost::new(
        policy_bytes,
        ResolverDeployment {
            manifest_path: PathBuf::from(&args[2]),
            worker_executable: PathBuf::from(&args[3]),
            working_directory: PathBuf::from(&args[4]),
        },
        source,
    )
    .map_err(|error| error.code())?;
    let reference = SecretRef::new(&config.id, &mapping.key, mapping.field.clone());
    let context = CredentialResolutionContext {
        subject: CredentialSubject::Node {
            workflow_id: "operator-smoke".into(),
            node_id: "delivery".into(),
        },
        binding: "API_TOKEN".into(),
        reference: reference.clone(),
        destination: CredentialDestination::AgentEnvironment,
        reason: config.reason.clone(),
    };
    let provider = AnimusSecretSpecProvider::new(config, Arc::new(host), Arc::new(AtomicBool::new(false)))
        .map_err(|error| error.code())?;
    // An ephemeral consumer comparison proves delivery without printing or
    // hashing the value. The expected value is an explicit smoke-only input.
    let expected = zeroize::Zeroizing::new(
        std::env::var("LATTICE_SMOKE_EXPECTED_VALUE").map_err(|_| "credential_secretspec_expected_input_required")?,
    );
    let value = provider
        .resolve_in_context(&reference, &context, None)
        .map_err(|_| "credential_secretspec_resolution_failed")?;
    if value.expose_for_execution() != expected.as_str() {
        return Err("credential_secretspec_delivery_mismatch");
    }
    Ok(())
}
