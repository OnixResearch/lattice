//! Attempt-local model broker. Credentials and outbound networking stay on the host.
//! In a VM guest the broker forwards every admitted request over vsock to the
//! host model gateway (`forwarded`) and holds no credential at all.
// r[impl lattice.candidate_worker.model_broker]
// r[impl lattice.candidate_worker.model_broker.scenario.enforcement]
// r[impl lattice.candidate_worker.model_broker.scenario.denial]

pub(crate) mod admission;
mod brokered;
#[cfg(unix)]
mod forwarded;
#[cfg(unix)]
mod host;
#[cfg(all(test, unix))]
mod isolation_tests;
pub(crate) mod serving;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(unix)]
mod wire;

pub const SCHEMA: &str = "lattice.candidate-model-broker.v1";
pub const PORT: u16 = 41001;
pub const SOCKET: &str = "/run/lattice-model.sock";
pub const MODEL_CONFIG: &str = "/tmp/omp-home/.omp/agent/models.yml";
pub const FORWARDED_SCHEMA: &str = "lattice.model-broker-forwarded-upstream.v1";

/// Immutable runtime artifact. A byte identity is not source/build provenance.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Executable {
    pub path: std::path::PathBuf,
    pub blake3: String,
}

/// The guest broker's host gateway (C4): every admitted request is forwarded
/// over AF_VSOCK to `VMADDR_CID_HOST` at `vsock_port`, framed with the
/// binding identity. A forwarding profile carries no credential.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forwarded {
    pub schema: String,
    pub vsock_port: crate::vm_profile::VsockPort,
    /// `b3:` identity of the request's `lattice.vm-attempt-binding.v1`.
    pub binding: String,
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
    /// Present only in a VM guest; absent, the profile serializes and behaves
    /// exactly as before.
    #[serde(default = "local", skip_serializing_if = "Option::is_none")]
    pub forwarded: Option<Forwarded>,
}

/// A broker that calls its endpoint directly: no forwarded gateway.
const fn local() -> Option<Forwarded> {
    None
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
    /// `request_index` is the broker's index of this request: the run's
    /// first index plus the requests it admitted before.
    fn admit(
        &mut self,
        checks: crate::execution_binding::serving::gate::Checks,
        request_index: u32,
    ) -> Option<GateTicket>;
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

/// Permission for one upstream call; served and recorded routes carry what
/// their settlement needs.
#[derive(Debug)]
pub struct GateTicket(Permit);

#[derive(Debug)]
enum Permit {
    Unserved,
    Served(Box<crate::execution_binding::serving::gate::Ticket>),
    /// The synced intent of a recorded brokered request.
    Recorded(Box<lattice_core::execution_binding::serving::brokered::Intent>),
}

impl GateTicket {
    /// A ticket of a gate without a serving-owner producer ticket.
    #[must_use]
    pub(crate) const fn unserved() -> Self {
        Self(Permit::Unserved)
    }
}

/// The broker's own checks alone: no serving port is called and no request
/// evidence is written. Attempts use [`RecordedRoute`] instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DirectRoute;

impl ModelRequestGate for DirectRoute {
    fn admit(&mut self, checks: crate::execution_binding::serving::gate::Checks, _: u32) -> Option<GateTicket> {
        is_admitted(checks).then_some(GateTicket::unserved())
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

/// Whether the broker's own model, budget and deadline checks all passed.
const fn is_admitted(checks: crate::execution_binding::serving::gate::Checks) -> bool {
    checks.is_exact_model && checks.is_within_budget && checks.is_current
}

/// Durable, write-once brokered request evidence of one attempt.
pub trait RequestEvidence: Send {
    /// Create and sync the dispatch intent before upstream contact. An intent
    /// that already exists for its request, even with identical bytes, MUST
    /// be refused: each request index admits at most one upstream contact.
    fn create_intent(
        &mut self,
        intent: &lattice_core::execution_binding::serving::brokered::Intent,
    ) -> Result<(), String>;
    /// Persist the terminal record after the upstream call ended; an
    /// identical replay is accepted, a different one refused.
    fn write_record(
        &mut self,
        record: &lattice_core::execution_binding::serving::brokered::Record,
    ) -> Result<(), String>;
}

/// A route without a serving declaration whose every relayed request leaves a
/// synced intent before upstream contact and a terminal record after it. The
/// broker's own checks decide; no serving port is called. Each request index
/// is admitted at most once and only above every index admitted before.
// r[impl lattice.serving_binding.brokered_evidence]
pub struct RecordedRoute<'a> {
    context: lattice_core::execution_binding::serving::brokered::Context,
    evidence: &'a mut dyn RequestEvidence,
    /// The highest request index this route admitted.
    highest: Option<u32>,
}

impl<'a> RecordedRoute<'a> {
    #[must_use]
    pub fn new(
        context: lattice_core::execution_binding::serving::brokered::Context,
        evidence: &'a mut dyn RequestEvidence,
    ) -> Self {
        Self {
            context,
            evidence,
            highest: None,
        }
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
    /// The C4 request index of this incarnation's first forwarded request.
    /// Every broker run of one attempt starts where the previous runs' index
    /// range ends, so a host gateway never sees an index twice; `0` for a
    /// single-shot attempt and for a direct route.
    pub first_request_index: u32,
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
