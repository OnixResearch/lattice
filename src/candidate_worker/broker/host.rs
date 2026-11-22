use std::os::unix::fs::DirBuilderExt;
use std::os::unix::fs::PermissionsExt;

struct StopOnDrop<'a>(&'a std::sync::atomic::AtomicBool);

impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::Release);
    }
}

struct Directory(std::path::PathBuf);

impl Directory {
    fn create(workspace: &std::path::Path) -> Result<Self, String> {
        let parent = workspace.parent().ok_or("workspace has no broker staging parent")?;
        let path = parent.join(format!(".lattice-model-{:032x}", rand::random::<u128>()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .map_err(|_| "private model broker directory is unavailable")?;
        Ok(Self(path))
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        for name in ["model.sock", "models.yml"] {
            let _ = std::fs::remove_file(self.0.join(name));
        }
        let _ = std::fs::remove_dir(&self.0);
    }
}

fn mount(
    directory: &Directory,
    request: &super::super::Request,
    profile: &super::Profile,
    executables: (std::path::PathBuf, std::path::PathBuf),
) -> Result<super::Mount, String> {
    let binding_identity = super::admission::binding_identity(request)?;
    let bytes = serde_json::to_vec(&(profile, &binding_identity)).map_err(|_| "broker profile cannot be encoded")?;
    Ok(super::Mount {
        socket: directory.0.join("model.sock"),
        config: directory.0.join("models.yml"),
        relay: executables.0,
        shell: executables.1,
        profile_identity: blake3::hash(&bytes).to_hex().to_string(),
        binding_identity,
    })
}

fn listener(mount: &super::Mount) -> Result<std::os::unix::net::UnixListener, String> {
    let listener =
        std::os::unix::net::UnixListener::bind(&mount.socket).map_err(|_| "private model socket is unavailable")?;
    std::fs::set_permissions(&mount.socket, std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "model socket permissions are unavailable")?;
    listener.set_nonblocking(true).map_err(|_| "bounded model listener is unavailable")?;
    Ok(listener)
}

fn write_model_config(mount: &super::Mount, profile: &super::Profile) -> Result<(), String> {
    let config = serde_json::json!({"providers":{"lattice":{
        "api":"openai-completions", "apiKey":"lattice-broker-local",
        "baseUrl":format!("http://127.0.0.1:{}/v1", super::PORT),
        "models":[{"id":profile.model,"name":"Admitted attempt model","api":"openai-completions"}]
    }}});
    std::fs::write(&mount.config, config.to_string()).map_err(|_| "private model configuration is unavailable")?;
    std::fs::set_permissions(&mount.config, std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "model configuration permissions are unavailable".into())
}

pub(super) fn with_route<T>(
    mut request: super::super::Request,
    profile: &super::Profile,
    services: super::Services<'_>,
    cancellation: &std::sync::atomic::AtomicBool,
    run: impl FnOnce(super::super::Request) -> T,
) -> Result<(T, bool), String> {
    // A VM-bound broker reaches its host gateway before anything else, so a
    // guest without one never starts a worker or falls back to a host route.
    let forwarding = match &profile.forwarded {
        Some(forwarded) => {
            super::admission::validate(profile)?;
            let target = super::forwarded::Target::of(forwarded);
            super::forwarded::probe(&target)?;
            Some(target)
        }
        None => None,
    };
    let executables = super::admission::admit(&mut request, profile)?;
    debug_assert!(!request.boundary.network);
    debug_assert!(request.require_isolation);
    let directory = Directory::create(&request.boundary.root)?;
    let mount = mount(&directory, &request, profile, executables)?;
    let listener = listener(&mount)?;
    write_model_config(&mount, profile)?;
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let deadline = crate::operation::shell::SystemClock
        .observe_monotonic_instant()
        .checked_add(std::time::Duration::from_secs(request.timeout_secs))
        .ok_or("model broker deadline is invalid")?;
    let handler = super::wire::Handler {
        profile,
        credentials: services.credentials,
        gate: services.gate,
        cancellation,
        stopped: &stopped,
        deadline,
        requests: 0,
        failure: None,
        forwarding,
    };
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("lattice-model-broker".into())
            .spawn_scoped(scope, || serve(listener, handler))
            .map_err(|_| "model broker worker is unavailable")?;
        let _stop_on_exit = StopOnDrop(&stopped);
        request.broker = Some(mount);
        let value = run(request);
        stopped.store(true, std::sync::atomic::Ordering::Release);
        let is_stopped_cleanly = matches!(worker.join(), Ok(Ok(())));
        Ok((value, is_stopped_cleanly))
    })
}

fn serve(listener: std::os::unix::net::UnixListener, mut handler: super::wire::Handler<'_>) -> Result<(), String> {
    while handler.is_current() {
        match listener.accept() {
            Ok((stream, _)) => handler.exchange(stream)?,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(_) => return Err("model broker listener failed after start".into()),
        }
    }
    Ok(())
}
