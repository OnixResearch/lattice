//! Host model gateway of a VM-bound attempt (C3/C4). The guest's forwarded
//! broker reaches it over vsock; it holds the only model credential and asks
//! Kiln's effect authority before and after every upstream call. See
//! `docs/vm-execution-profile.md`.

pub mod admission;
pub mod authority;
pub mod core;
pub mod shell;
#[cfg(all(test, unix))]
pub(crate) mod tests;

#[derive(clap::Args, Debug)]
pub struct CommandArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Serve one VM attempt binding's forwarded model requests.
    Serve(Serve),
}

#[derive(clap::Args, Debug, Clone)]
pub struct Serve {
    /// Evaluated `lattice.vm-execution-profile.v1` JSON.
    #[arg(long)]
    pub profile: std::path::PathBuf,
    /// `lattice.vm-attempt-binding.v1` JSON.
    #[arg(long)]
    pub binding: std::path::PathBuf,
    /// `lattice.model-gateway-upstream.v1` JSON: endpoint, exact model,
    /// host-only env credential reference and request ceiling.
    #[arg(long)]
    pub upstream: std::path::PathBuf,
    /// Unix socket Cloud Hypervisor connects for guest port `model_vsock_port`
    /// (`<vsock socket>_<port>`); created with mode 0600.
    #[arg(long)]
    pub listen_uds: std::path::PathBuf,
    /// PID of the Cloud Hypervisor process; every other peer is dropped.
    #[arg(long)]
    pub vmm_pid: u32,
    /// Kiln's `lattice.vm-effect-authority.v1` Unix socket.
    #[arg(long)]
    pub authority_socket: std::path::PathBuf,
}

pub const SERVE_SCHEMA: &str = "lattice.model-gateway-serve.v1";

pub fn run_command(arguments: &CommandArgs) -> Result<(), String> {
    match &arguments.command {
        Command::Serve(serve) => run_serve(serve),
    }
}

/// The standalone command: admit, resolve the credential, listen, announce
/// readiness, serve until stopped.
#[cfg(unix)]
fn run_serve(arguments: &Serve) -> Result<(), String> {
    let admitted = shell::admit(&arguments.profile, &arguments.binding, &arguments.upstream)?;
    if shell::now_ms()? >= admitted.binding.deadline_unix_ms {
        return Err("model gateway refused: the host clock reached the binding deadline".into());
    }
    let credential = shell::credential(&admitted.upstream)?;
    let bound = admission::Bound {
        binding: admitted.binding_identity.clone(),
        deadline_unix_ms: admitted.binding.deadline_unix_ms,
        model: admitted.upstream.model.clone(),
        max_requests: admitted.upstream.max_requests,
    };
    let listener = shell::listen(&arguments.listen_uds)?;
    let stopped = std::sync::atomic::AtomicBool::new(false);
    let port = authority::UnixSocket {
        socket: arguments.authority_socket.clone(),
    };
    let mut server = shell::Server {
        session: admission::Session::new(bound.clone()),
        gate: authority::KilnAuthorityGate::new(port, bound),
        endpoint: admitted.upstream.endpoint.clone(),
        credential,
        vmm_pid: arguments.vmm_pid,
        stopped: &stopped,
        clock: &shell::now_ms,
        dropped_connections: 0,
        undelivered_replies: 0,
    };
    shell::announce(&serde_json::json!({
        "schema": SERVE_SCHEMA,
        "state": "listening",
        "profile": admitted.profile_identity,
        "binding": admitted.binding_identity,
    }))?;
    debug_assert!(!stopped.load(std::sync::atomic::Ordering::Acquire));
    debug_assert_eq!(server.session.forwarded_requests, 0);
    server.serve(&listener)
}

#[cfg(not(unix))]
fn run_serve(_arguments: &Serve) -> Result<(), String> {
    Err("model gateway requires Unix sockets".into())
}
