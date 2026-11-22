pub(super) const MAX_REQUESTS: u32 = 256;

pub(crate) fn now_ms() -> Result<u64, String> {
    let observed = crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
    u64::try_from(observed).map_err(|_| "model broker clock is unavailable".into())
}

pub(crate) fn validate(profile: &super::Profile) -> Result<(), String> {
    if profile.schema != super::SCHEMA || profile.max_requests == 0 || profile.max_requests > MAX_REQUESTS {
        return Err("model broker schema or request bound is unsupported".into());
    }
    validate_model(&profile.model)?;
    validate_endpoint(&profile.endpoint)?;
    if let Some(reference) = &profile.credential {
        crate::credentials::validate_secret_ref(reference).map_err(|_| "model credential reference is invalid")?;
    }
    if !super::super::result::is_blake3(&profile.sandbox_blake3) {
        return Err("sandbox byte identity is invalid".into());
    }
    Ok(())
}

fn validate_model(model: &str) -> Result<(), String> {
    if model.is_empty() || model.len() > 256 {
        return Err("model broker requires one bounded exact model".into());
    }
    if model.trim() != model || model.chars().any(char::is_control) {
        return Err("model broker requires one canonical exact model".into());
    }
    Ok(())
}

fn validate_endpoint(endpoint: &str) -> Result<(), String> {
    if endpoint.len() > crate::http::MAX_URL_BYTES {
        return Err("model endpoint exceeds its byte bound".into());
    }
    let url = url::Url::parse(endpoint).map_err(|_| "model endpoint is invalid")?;
    let is_loopback =
        url.host_str().is_some_and(|host| host.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback()));
    let is_transport_admitted = url.scheme() == "https" || url.scheme() == "http" && is_loopback;
    let has_url_authority = !url.username().is_empty() || url.password().is_some();
    let has_dynamic_target = url.query().is_some() || url.fragment().is_some();
    if !is_transport_admitted || has_url_authority || has_dynamic_target {
        return Err("model endpoint transport or URL authority is not admitted".into());
    }
    if !url.path().ends_with("/chat/completions") || url.host().is_none() || url.as_str() != endpoint {
        return Err("model endpoint is not canonical chat-completion HTTPS or literal loopback HTTP".into());
    }
    debug_assert!(url.username().is_empty());
    debug_assert!(url.query().is_none());
    Ok(())
}

/// The canonical Nix store path of `value` when its bytes are its pin.
pub(crate) fn executable(value: &super::Executable) -> Result<std::path::PathBuf, String> {
    if !super::super::result::is_blake3(&value.blake3) || !value.path.is_absolute() {
        return Err("runtime executable byte identity is invalid".into());
    }
    let path = std::fs::canonicalize(&value.path).map_err(|_| "runtime executable is unavailable")?;
    if !path.starts_with("/nix/store") || path.to_str().is_none() {
        return Err("runtime executable must be an immutable UTF-8 Nix store artifact".into());
    }
    let identity =
        super::super::compatibility::executable_identity(&path).ok_or("runtime executable bytes are unavailable")?;
    if identity != value.blake3 {
        return Err("runtime executable differs from its admitted bytes".into());
    }
    Ok(path)
}

fn validate_boundary(request: &super::super::Request, profile: &super::Profile) -> Result<(), String> {
    if profile.expires_at_ms <= now_ms()? || request.broker.is_some() {
        return Err("model broker authority expired or its incarnation is already selected".into());
    }
    if request.boundary.network || !request.require_isolation || request.boundary.credentials {
        return Err("model broker requires private network and credential boundaries".into());
    }
    Ok(())
}

pub(super) fn admit(
    request: &mut super::super::Request,
    profile: &super::Profile,
) -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    validate(profile)?;
    validate_boundary(request, profile)?;
    let sandbox = super::Executable {
        path: request.sandbox_program.as_ref().ok_or("sandbox is unavailable")?.into(),
        blake3: profile.sandbox_blake3.clone(),
    };
    request.sandbox_program =
        Some(executable(&sandbox)?.into_os_string().into_string().map_err(|_| "sandbox path is not UTF-8")?);
    let program = std::fs::canonicalize(&request.program).map_err(|_| "worker executable is unavailable")?;
    if !program.starts_with("/nix/store") {
        return Err("OMP executable must be an immutable Nix store artifact".into());
    }
    request.program = program.into_os_string().into_string().map_err(|_| "worker path is not UTF-8")?;
    Ok((executable(&profile.relay)?, executable(&profile.shell)?))
}

pub(super) fn binding_identity(request: &super::super::Request) -> Result<String, String> {
    let boundary = &request.boundary;
    let attempt = (&request.work_id, &request.attempt_id, &request.fence, &request.workspace_id);
    let runtime = (&request.program, &request.args, request.require_isolation, &request.sandbox_program);
    let paths = (&boundary.root, &boundary.read_paths, &boundary.write_paths, boundary.network, boundary.credentials);
    let bytes = serde_json::to_vec(&(attempt, runtime, paths, request.timeout_secs, request.output_limit_bytes))
        .map_err(|_| "model broker enforcement binding cannot be encoded")?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}
