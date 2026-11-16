pub(super) fn admit(request: &super::super::Request, profile: &super::Profile) -> Result<(), &'static str> {
    if profile.revision != super::OMP_REVISION || !super::super::result::is_blake3(&profile.executable_blake3) {
        return Err("OMP revision and executable byte pin must be admitted");
    }
    if !request.require_isolation || request.sandbox_program.is_none() || !request.args.is_empty() {
        return Err("OMP requires an explicit workspace provider and profile-owned argv");
    }
    admit_input(profile)?;
    if !std::path::Path::new(&request.program).is_absolute() {
        return Err("OMP executable must be an absolute reviewed path");
    }
    Ok(())
}

pub(super) fn arguments(request: &super::super::Request, profile: &super::Profile) -> Vec<String> {
    let mut args = Vec::with_capacity(24);
    args.extend([
        "launch".into(),
        "--cwd".into(),
        "/workspace".into(),
        "--model".into(),
        profile.model.clone(),
    ]);
    args.extend(PRINT_OPTIONS.iter().map(|option| (*option).to_string()));
    args.extend([
        format!("{}s", request.timeout_secs),
        "--".into(),
        profile.prompt.clone(),
    ]);
    args
}

// Reviewed print-mode flags are fixed; only model, deadline and prompt vary.
const PRINT_OPTIONS: [&str; 16] = [
    "--no-session",
    "--no-title",
    "-p",
    "--mode",
    "json",
    "--no-extensions",
    "--no-skills",
    "--no-rules",
    "--no-context-files",
    "--no-lsp",
    "--no-pty",
    "--tools",
    "read,edit,write",
    "--approval-mode",
    "write",
    "--max-time",
];

fn admit_input(profile: &super::Profile) -> Result<(), &'static str> {
    let is_model_invalid = profile.model.trim().is_empty() || profile.model.contains('\0');
    let is_prompt_invalid = profile.prompt.trim().is_empty() || profile.prompt.contains('\0');
    if is_model_invalid || is_prompt_invalid || profile.prompt.trim_start().starts_with('/') {
        return Err("OMP model and one non-command prompt must be explicit");
    }
    if profile
        .model
        .split_once('/')
        .is_none_or(|(provider, model)| provider.is_empty() || model.is_empty())
    {
        return Err("OMP requires an exact provider/model selector");
    }
    Ok(())
}
