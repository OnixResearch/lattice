//! Attempt-local model broker. Credentials and outbound networking stay on the host.
// r[impl lattice.candidate_worker.model_broker]
// r[impl lattice.candidate_worker.model_broker.scenario.enforcement]
// r[impl lattice.candidate_worker.model_broker.scenario.denial]

pub(crate) mod admission;
#[cfg(unix)]
mod host;
#[cfg(all(test, unix))]
mod isolation_tests;
mod serving;
#[cfg(test)]
mod tests;
#[cfg(unix)]
mod wire;

pub const SCHEMA: &str = "lattice.candidate-model-broker.v1";
pub const PORT: u16 = 41001;
pub const SOCKET: &str = "/run/lattice-model.sock";
pub const MODEL_CONFIG: &str = "/tmp/omp-home/.omp/agent/models.yml";

/// Immutable runtime artifact. A byte identity is not source/build provenance.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Executable {
    pub path: std::path::PathBuf,
    pub blake3: String,
}

/// Installed by the trusted dispatcher, never obtained from model output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema: String,
    pub endpoint: String,
    pub model: String,
    pub credential: Option<crate::credentials::SecretRef>,
    pub relay: Executable,
    pub shell: Executable,
    pub sandbox_blake3: String,
    pub expires_at_ms: u64,
    pub max_requests: u32,
}

/// Only a running host broker can construct this mount grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    socket: std::path::PathBuf,
    config: std::path::PathBuf,
    relay: std::path::PathBuf,
    shell: std::path::PathBuf,
    pub(super) profile_identity: String,
    binding_identity: String,
}

impl Mount {
    pub(super) fn admit(&self, request: &super::Request) -> Result<(), String> {
        if admission::binding_identity(request)? != self.binding_identity {
            return Err("model broker mount belongs to a different attempt or enforcement request".into());
        }
        Ok(())
    }

    pub(super) fn arguments(&self) -> Vec<String> {
        vec![
            "--dir".into(),
            "/run".into(),
            "--ro-bind".into(),
            self.socket.to_string_lossy().into_owned(),
            SOCKET.into(),
            "--dir".into(),
            "/tmp/omp-home/.omp".into(),
            "--dir".into(),
            "/tmp/omp-home/.omp/agent".into(),
            "--ro-bind".into(),
            self.config.to_string_lossy().into_owned(),
            MODEL_CONFIG.into(),
        ]
    }

    pub(super) fn launch(&self, program: String, args: &[String]) -> Vec<String> {
        let mut launch = vec![
            self.shell.to_string_lossy().into_owned(), "-c".into(),
            // Positional parameters preserve every caller argument literally.
            // PID-namespace teardown owns the relay and all worker descendants.
            "\"$1\" \"$2\" \"$3\" & relay=$!; count=0; until \"$1\" -u OPEN:/dev/null TCP4:127.0.0.1:41001,connect-timeout=0.1 2>/dev/null; do count=$((count+1)); if [ \"$count\" -ge 100 ] || ! kill -0 \"$relay\" 2>/dev/null; then kill \"$relay\" 2>/dev/null; wait \"$relay\" 2>/dev/null; exit 125; fi; done; shift 3; \"$@\"; status=$?; kill \"$relay\" 2>/dev/null; wait \"$relay\" 2>/dev/null; exit \"$status\"".into(),
            "lattice-model-relay".into(),
            self.relay.to_string_lossy().into_owned(),
            format!("TCP4-LISTEN:{PORT},bind=127.0.0.1,reuseaddr,fork"),
            format!("UNIX-CONNECT:{SOCKET}"),
            program,
        ];
        launch.extend_from_slice(args);
        launch
    }
}

/// Per-request decision before upstream contact and settlement after it.
/// Serving-owner admission plugs in here; the broker's own model, budget,
/// deadline and credential checks stay in force for every gate.
// r[impl lattice.serving_binding.port_boundary.scenario.unserved]
pub trait ModelRequestGate: Send {
    /// `None` refuses the request; nothing reaches the upstream endpoint.
    fn admit(&mut self, checks: crate::execution_binding::serving::gate::Checks) -> Option<GateTicket>;
    /// Record how the admitted call ended. `Ok(false)` withholds the response
    /// from the worker; `Err` means evidence could not be retained.
    fn settle(
        &mut self,
        ticket: GateTicket,
        upstream: crate::execution_binding::serving::gate::Upstream,
    ) -> Result<bool, String>;
    /// Record that the broker's request budget refused a request before the
    /// gate; `Err` means that refusal could not be retained.
    fn exhausted(&mut self) -> Result<(), String>;
}

/// Permission for one upstream call; served routes carry their producer ticket.
#[derive(Debug)]
pub struct GateTicket(Option<Box<crate::execution_binding::serving::gate::Ticket>>);

/// A route without a serving declaration: the broker's own checks decide,
/// no serving port is called and no request record is written.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DirectRoute;

impl ModelRequestGate for DirectRoute {
    fn admit(&mut self, checks: crate::execution_binding::serving::gate::Checks) -> Option<GateTicket> {
        let is_admitted = checks.is_exact_model && checks.is_within_budget && checks.is_current;
        is_admitted.then_some(GateTicket(None))
    }

    fn settle(
        &mut self,
        _ticket: GateTicket,
        _upstream: crate::execution_binding::serving::gate::Upstream,
    ) -> Result<bool, String> {
        Ok(true)
    }

    fn exhausted(&mut self) -> Result<(), String> {
        Ok(())
    }
}

/// A route served through the serving-owner gate of one attempt.
pub struct OwnerGate<'a> {
    pub session: crate::execution_binding::serving::gate::Session<'a>,
    pub route_name: String,
}

/// Host-side services of one broker incarnation.
pub struct Services<'a> {
    pub credentials: &'a dyn crate::credentials::CredentialProvider,
    pub gate: &'a mut dyn ModelRequestGate,
}

pub(super) fn with_route<T>(
    request: super::Request,
    profile: &Profile,
    services: Services<'_>,
    cancellation: &std::sync::atomic::AtomicBool,
    run: impl FnOnce(super::Request) -> T,
) -> Result<(T, bool), String> {
    #[cfg(unix)]
    {
        host::with_route(request, profile, services, cancellation, run)
    }
    #[cfg(not(unix))]
    {
        let _ = (request, profile, services, cancellation, run);
        Err("candidate model broker requires Unix sockets".into())
    }
}
