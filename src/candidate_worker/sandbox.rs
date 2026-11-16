const SYSTEM_BASES: [&str; 5] = ["/nix/store", "/usr", "/bin", "/lib", "/lib64"];
const ENVIRONMENT: [&str; 16] = [
    "--unshare-user",
    "--disable-userns",
    "--unshare-pid",
    "--unshare-ipc",
    "--unshare-uts",
    "--unshare-cgroup",
    "--die-with-parent",
    "--cap-drop",
    "ALL",
    "--clearenv",
    "--setenv",
    "PATH",
    "/usr/bin:/bin",
    "--setenv",
    "HOME",
    "/tmp/omp-home",
];
const PRIVATE_ROOT: [&str; 12] = [
    "--proc",
    "/proc",
    "--dev",
    "/dev",
    "--tmpfs",
    "/tmp",
    "--dir",
    "/tmp/omp-home",
    "--tmpfs",
    "/workspace",
    "--chdir",
    "/workspace",
];

pub(super) fn command(request: &super::Request) -> Result<(String, Vec<String>), String> {
    let sandbox = provider(request)?;
    let root =
        std::fs::canonicalize(&request.boundary.root).map_err(|_| "workspace root is unavailable".to_string())?;
    let program = program_path(&request.program, &root, &request.boundary)?;
    let args = environment_arguments(request.boundary.network);
    let args = read_mounts(request, &root, args)?;
    let mut args = write_mounts(request, &root, args)?;
    args.extend(["--remount-ro", "/workspace", "--remount-ro", "/", "--"].map(str::to_string));
    args.push(program);
    args.extend(request.args.iter().cloned());
    Ok((path_text(&sandbox)?, args))
}

fn provider(request: &super::Request) -> Result<std::path::PathBuf, String> {
    let Some(sandbox) = request.sandbox_program.as_ref() else {
        return Err("no reviewed workspace isolation provider is configured".to_string());
    };
    let sandbox =
        std::fs::canonicalize(sandbox).map_err(|_| "configured sandbox provider is unavailable".to_string())?;
    if !sandbox.is_file() {
        return Err("configured sandbox provider is not a file".to_string());
    }
    Ok(sandbox)
}

fn read_mounts(request: &super::Request, root: &std::path::Path, mut args: Vec<String>) -> Result<Vec<String>, String> {
    for read in &request.boundary.read_paths {
        if within_path(read, &request.boundary.write_paths) {
            continue;
        }
        let source = root.join(read);
        if source.exists() {
            args.extend([
                "--ro-bind".to_string(),
                path_text(&source)?,
                format!("/workspace/{read}"),
            ]);
        }
    }
    Ok(args)
}

fn write_mounts(
    request: &super::Request,
    root: &std::path::Path,
    mut args: Vec<String>,
) -> Result<Vec<String>, String> {
    for write in &request.boundary.write_paths {
        let source = root.join(write);
        if !source.exists() {
            return Err("isolated workspace write paths must already exist".to_string());
        }
        args.extend(["--bind".to_string(), path_text(&source)?, format!("/workspace/{write}")]);
    }
    Ok(args)
}

fn program_path(
    program: &str,
    root: &std::path::Path,
    boundary: &crate::candidate_capture::WorkspaceBoundary,
) -> Result<String, String> {
    let canonical = resolve_program(program, root)?;
    if let Ok(relative) = canonical.strip_prefix(root) {
        return workspace_program(relative, boundary);
    }
    if SYSTEM_BASES.iter().any(|base| canonical.starts_with(base)) {
        return path_text(&canonical);
    }
    Err("worker program is outside the sandbox".to_string())
}

fn resolve_program(program: &str, root: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let requested = std::path::Path::new(program);
    let path = if requested.is_absolute() {
        requested.to_path_buf()
    } else if requested.components().count() > 1 {
        root.join(requested)
    } else {
        which::which(program).map_err(|_| "worker program is unavailable".to_string())?
    };
    let canonical = std::fs::canonicalize(path).map_err(|_| "worker program is unavailable".to_string())?;
    if !canonical.is_file() {
        return Err("worker program is unavailable".to_string());
    }
    Ok(canonical)
}

fn workspace_program(
    relative: &std::path::Path,
    boundary: &crate::candidate_capture::WorkspaceBoundary,
) -> Result<String, String> {
    let relative = relative.to_str().ok_or_else(|| "worker program path is not UTF-8".to_string())?;
    if !within_path(relative, &boundary.read_paths) && !within_path(relative, &boundary.write_paths) {
        return Err("worker program is outside the admitted workspace paths".to_string());
    }
    Ok(format!("/workspace/{relative}"))
}

fn within_path(path: &str, admitted: &[String]) -> bool {
    admitted.iter().any(|prefix| {
        path == prefix || path.strip_prefix(prefix.as_str()).is_some_and(|suffix| suffix.starts_with('/'))
    })
}

fn path_text(path: &std::path::Path) -> Result<String, String> {
    path.to_str().map(str::to_string).ok_or_else(|| "workspace path is not UTF-8".to_string())
}

fn environment_arguments(has_network: bool) -> Vec<String> {
    let mut args: Vec<String> = ENVIRONMENT.iter().map(|value| (*value).to_string()).collect();
    if !has_network {
        args.push("--unshare-net".to_string());
    }
    let mut args = system_mounts(args);
    args.extend(PRIVATE_ROOT.iter().map(|value| (*value).to_string()));
    args
}

fn system_mounts(mut args: Vec<String>) -> Vec<String> {
    for base in SYSTEM_BASES {
        if std::path::Path::new(base).exists() {
            args.extend(["--ro-bind".to_string(), base.to_string(), base.to_string()]);
        }
    }
    if std::path::Path::new("/run/current-system/sw/bin").exists() {
        args.extend(["--ro-bind", "/run/current-system/sw/bin", "/usr/bin"].map(str::to_string));
    }
    args
}
