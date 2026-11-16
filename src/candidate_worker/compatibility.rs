pub(super) fn executable_identity(path: &std::path::Path) -> Option<String> {
    if !std::fs::metadata(path).ok()?.is_file() {
        return None;
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(std::fs::File::open(path).ok()?).ok()?;
    Some(hasher.finalize().to_hex().to_string())
}

pub(super) fn observe(request: &super::Request) -> Option<super::Compatibility> {
    Some(super::Compatibility {
        schema: super::CANDIDATE_WORKER_COMPATIBILITY_SCHEMA.to_string(),
        worker_program_identity: program_identity(request)?,
        environment_identity: environment_identity(request),
        workspace_provider_identity: provider_identity(request)?,
        result_schema: super::CANDIDATE_ATTEMPT_RESULT_SCHEMA.to_string(),
    })
}

fn program_identity(request: &super::Request) -> Option<String> {
    let program = std::path::Path::new(&request.program);
    let program = if program.is_absolute() {
        program.to_path_buf()
    } else if program.components().count() > 1 {
        request.boundary.root.join(program)
    } else {
        which::which(&request.program).ok()?
    };
    executable_identity(&std::fs::canonicalize(program).ok()?)
}

fn provider_identity(request: &super::Request) -> Option<String> {
    if request.require_isolation {
        let (sandbox, args) = super::sandbox::command(request).ok()?;
        let executable = executable_identity(std::path::Path::new(&sandbox))?;
        let broker = request.broker.as_ref().map(|mount| mount.profile_identity.as_str());
        Some(crate::pure_inet_profile::blake3_identity(&serde_json::to_vec(&(executable, args, broker)).ok()?))
    } else {
        Some(crate::pure_inet_profile::blake3_identity(b"unsandboxed"))
    }
}

fn environment_identity(request: &super::Request) -> String {
    crate::pure_inet_profile::blake3_identity(if request.require_isolation {
        b"clear;PATH=/usr/bin:/bin;HOME=/tmp/omp-home;private-tmp.v1"
    } else {
        super::CANDIDATE_WORKER_ENVIRONMENT.as_bytes()
    })
}
