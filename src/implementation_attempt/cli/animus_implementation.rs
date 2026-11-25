//! Opt-in, host-injected Animus implementation execution and closed child lineage.
//!
//! A model response cannot be a lineage source. The injected owner must use its
//! pinned native child-lifecycle exporter, and must run inside the admitted
//! workspace, model route, isolation and deadline. The standalone CLI has no
//! such adapter and refuses this profile before creating an execution intent.

#[cfg(test)]
mod tests;

use std::sync::atomic::AtomicBool;

pub const LINEAGE_SCHEMA: &str = "animus.implementation-child-lineage.v1";
pub const ADAPTER_SCHEMA: &str = "lattice.animus-implementation-host.v1";
pub const DEFINITION: &str = "animus-implementation-worker";
const MAX_EDGES: usize = 32;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub adapter_revision: String,
    pub adapter_identity: String,
    pub base_identity: String,
    pub prompt: String,
    pub model_broker: crate::candidate_worker::broker::Profile,
}

impl Profile {
    pub fn validate(&self, now_ms: u64, expires_at_ms: u64) -> Result<(), String> {
        if !is_identity(&self.adapter_revision)
            || !is_identity(&self.adapter_identity)
            || !is_identity(&self.base_identity)
            || self.prompt.trim().is_empty()
            || self.model_broker.expires_at_ms <= now_ms
            || self.model_broker.expires_at_ms > expires_at_ms
        {
            return Err("Animus implementation profile is unsupported".into());
        }
        crate::candidate_worker::broker::admission::validate(&self.model_broker)
    }
}

/// Separate profile domain; OMP's v3 framing and all historical bytes stay intact.
pub(super) fn profile_revision(request: &super::Request, profile: &Profile) -> Result<String, String> {
    let capability = request.worker_capability.as_ref().map(|declaration| declaration.canonical())
        .transpose().map_err(|error| format!("unsupported Animus capability: {error:?}"))?;
    let compute = serde_json::to_value(&request.worker_compute).map_err(|error| error.to_string())?;
    let broker = &profile.model_broker;
    let framed = serde_json::json!({
        "schema": "lattice.animus-implementation-profile-revision.v1",
        "request_schema": super::REQUEST_SCHEMA,
        "adapter_schema": ADAPTER_SCHEMA,
        "adapter_revision": profile.adapter_revision,
        "adapter_identity": profile.adapter_identity,
        "model": broker.model,
        "broker_schema": broker.schema,
        "broker_endpoint": broker.endpoint,
        "broker_credential": broker.credential,
        "broker_relay": broker.relay,
        "broker_shell": broker.shell,
        "broker_sandbox": broker.sandbox_blake3,
        "broker_max_requests": broker.max_requests,
        "read_paths": request.read_paths,
        "write_paths": request.write_paths,
        "output_limit_bytes": request.output_limit_bytes,
        "allow_unchanged": request.allow_unchanged,
        "compute": compute,
        "capability": capability,
    });
    let bytes = serde_json::to_vec(&framed).map_err(|error| error.to_string())?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub parent_id: String,
    pub child_id: String,
    pub state: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lineage {
    pub schema: String,
    pub work_identity: String,
    pub attempt_identity: String,
    pub fence: u64,
    pub request_identity: String,
    pub worker_incarnation: String,
    pub profile_revision: String,
    pub root_id: String,
    pub coverage: String,
    pub edges: Vec<Edge>,
}

impl Lineage {
    pub fn validate(&self, expected: &HostRequest<'_>) -> Result<(), String> {
        if self.schema != LINEAGE_SCHEMA
            || self.work_identity != expected.work_identity
            || self.attempt_identity != expected.attempt_identity
            || self.fence != expected.fence
            || self.request_identity != expected.request_identity
            || !is_identity(&self.work_identity)
            || !is_identity(&self.attempt_identity)
            || !is_identity(&self.request_identity)
            || !is_identity(&self.worker_incarnation)
            || !is_identity(&self.profile_revision)
            || self.worker_incarnation != expected.worker_incarnation
            || self.profile_revision != expected.profile_revision
            || !is_identity(&self.root_id)
            || !matches!(self.coverage.as_str(), "complete" | "partial")
            || self.edges.len() > MAX_EDGES
        {
            return Err("Animus lineage is missing, crossed or unsupported".into());
        }
        let mut children = std::collections::BTreeSet::new();
        for edge in &self.edges {
            if !is_identity(&edge.parent_id)
                || !is_identity(&edge.child_id)
                || edge.parent_id != self.root_id
                || !children.insert(edge.child_id.as_str())
                || edge.child_id == self.root_id
                || !valid_closed_state(&edge.state)
            {
                return Err("Animus lineage contains a malformed or crossed child lifecycle".into());
            }
        }
        Ok(())
    }
}

fn valid_closed_state(state: &str) -> bool {
    matches!(state, "completed" | "partial" | "failed" | "timed_out" | "cancelled" | "bound_exceeded")
}

pub(super) fn is_identity(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Bound request supplied to a host, never parsed from model output.
pub struct HostRequest<'a> {
    pub work_identity: &'a str,
    pub attempt_identity: &'a str,
    pub fence: u64,
    pub request_identity: &'a str,
    pub worker_incarnation: &'a str,
    pub profile_revision: &'a str,
    pub profile: &'a Profile,
}

/// Native host result; candidate bytes are still derived by Lattice's
/// workspace capture, not accepted from this lineage receipt.
pub struct HostObservation {
    pub outcome: crate::candidate_worker::Outcome,
    pub lineage: Option<Lineage>,
}

/// A genuine, version-pinned host adapter owns child events and execution.
/// The interface is deliberately distinct from a command/path capability.
pub trait Host: Send {
    fn adapter_identity(&self) -> &str;
    fn run(
        &mut self,
        request: &HostRequest<'_>,
        worker: &crate::candidate_worker::Request,
        cancellation: &AtomicBool,
        services: crate::candidate_worker::broker::Services<'_>,
    ) -> Result<HostObservation, String>;
}

const RETAINED_FILE: &str = "animus-child-lineage.json";

pub(super) fn incarnation(request_identity: &str) -> String {
    // The retained v3 worker incarnation is the sealed native request digest.
    request_identity.to_owned()
}

pub(super) fn retained(
    request: &super::Request,
    directory: &std::path::Path,
    request_identity: &str,
) -> Option<serde_json::Value> {
    let profile = request.animus_implementation.as_ref()?;
    let path = directory.join(RETAINED_FILE);
    let bytes = super::artifacts::read_bounded(&path, 32_768).ok()?;
    let lineage: Lineage = serde_json::from_slice(&bytes).ok()?;
    let worker_incarnation = incarnation(request_identity);
    let expected = HostRequest {
        work_identity: &request.envelope.work_id,
        attempt_identity: &request.envelope.attempt_id,
        fence: request.envelope.fence,
        request_identity,
        worker_incarnation: &worker_incarnation,
        profile_revision: &request.envelope.worker_profile,
        profile,
    };
    lineage.validate(&expected).ok()?;
    serde_json::to_value(lineage).ok()
}

struct Step {
    request: super::Request,
    worker: crate::candidate_worker::Request,
    store: crate::store::Store,
    receipt: super::binding::usage::WorkerReceipt,
    serving: Box<dyn crate::execution_binding::serving::OwnerPort>,
    draw: Option<super::binding::draw::SingleShot>,
    profile: Profile,
    host: Box<dyn Host>,
    directory: std::path::PathBuf,
}

impl Step {
    fn observe(mut self, cancellation: &AtomicBool) -> Result<(), String> {
        let credentials = super::session::launch::credentials(&self.profile.model_broker)?;
        let before = crate::candidate_capture::workspace(&self.worker.boundary.root)
            .map_err(|error| error.to_string())?;
        let before_modes = crate::candidate_capture::snapshot_modes(&self.worker.boundary.root, &before)
            .map_err(|error| error.to_string())?;
        let base_identity = crate::candidate::snapshot_identity(&before).map_err(|error| error.to_string())?;
        if base_identity != self.profile.base_identity {
            return Err("Animus workspace differs from admitted base; nothing dispatched".into());
        }
        let started_epoch_ms = super::epoch_ms(&crate::operation::shell::SystemClock).ok();
        let mut files = super::binding::requests::RequestFiles::new(&self.directory)?;
        let admitted = self.receipt.admitted();
        let mut served;
        let mut recorded;
        let gate: &mut dyn crate::candidate_worker::broker::ModelRequestGate = if admitted.serving.is_some() {
            let session = crate::execution_binding::serving::gate::Session::new(
                admitted, self.receipt.request_identity(), self.serving.as_mut(), &mut files,
            )?;
            served = crate::candidate_worker::broker::OwnerGate {
                session,
                route_name: super::binding::PRIMARY_ROUTE.to_owned(),
            };
            &mut served
        } else {
            recorded = super::binding::requests::recorded(admitted, &mut files);
            &mut recorded
        };
        let services = crate::candidate_worker::broker::Services {
            credentials: &credentials,
            gate,
            first_request_index: 0,
        };
        let worker_incarnation = incarnation(self.receipt.request_identity());
        let native = HostRequest {
            work_identity: &self.request.envelope.work_id,
            attempt_identity: &self.request.envelope.attempt_id,
            fence: self.request.envelope.fence,
            request_identity: self.receipt.request_identity(),
            worker_incarnation: &worker_incarnation,
            profile_revision: &self.request.envelope.worker_profile,
            profile: &self.profile,
        };
        // The host must use the injected gate and admitted boundary. The
        // returned candidate is processed by the same publication/recovery
        // path as OMP; the lineage is independent observational metadata.
        let observation = self.host.run(&native, &self.worker, cancellation, services)?;
        let outcome = verify_candidate(&self.request, &self.worker.boundary.root, &before, &before_modes,
            observation.outcome);
        if let Some(lineage) = &observation.lineage {
            if lineage.validate(&native).is_ok() {
                let bytes = serde_json::to_vec(lineage).map_err(|error| error.to_string())?;
                super::artifacts::write_once(&self.directory.join(RETAINED_FILE), &bytes)?;
            }
        }
        let window = super::binding::usage::WorkerWindow {
            started_epoch_ms,
            finished_epoch_ms: super::epoch_ms(&crate::operation::shell::SystemClock).ok(),
        };
        let coverage = self.receipt.persist(&self.store, window, &self.directory)?;
        if let Some(draw) = &self.draw {
            draw.retain(&self.directory, &coverage)?;
        }
        if let crate::candidate_worker::Outcome::Unchanged { observation, .. } = &outcome {
            super::artifacts::write_once(&self.directory.join("unchanged-observation.json"),
                &serde_json::to_vec(observation).map_err(|error| error.to_string())?)?;
        }
        let outcome = crate::candidate_worker::publish_outcome(&self.worker, &self.store, outcome, None);
        crate::candidate_worker::AttemptResult::from_outcome(&outcome)
            .validate().map_err(|error| error.to_string())
    }
}

fn verify_candidate(
    request: &super::Request,
    root: &std::path::Path,
    before: &std::collections::BTreeMap<String, Vec<u8>>,
    before_modes: &std::collections::BTreeMap<String, u32>,
    outcome: crate::candidate_worker::Outcome,
) -> crate::candidate_worker::Outcome {
    let (receipt, changed_files, unchanged) = match &outcome {
        crate::candidate_worker::Outcome::Produced { receipt, changed_files, .. } =>
            (Some(receipt.as_ref()), Some(changed_files), None),
        crate::candidate_worker::Outcome::Unchanged { observation, .. } =>
            (None, None, Some(observation.as_ref())),
        _ => return outcome,
    };
    let valid = crate::candidate_capture::workspace(root).ok().is_some_and(|after| {
        let modes_match = crate::candidate_capture::snapshot_modes(root, &after).ok().is_some_and(|after_modes| {
            before_modes.iter().all(|(path, mode)| after_modes.get(path).is_none_or(|observed| observed == mode))
        });
        let fence = request.envelope.fence.to_string();
        let bindings = crate::candidate::Bindings {
            work_id: &request.envelope.work_id,
            attempt_id: &request.envelope.attempt_id,
            fence: &fence,
            workspace_id: &request.envelope.workspace_id,
        };
        modes_match && match (receipt, changed_files, unchanged) {
            (Some(receipt), Some(changed_files), None) => {
                crate::candidate::verify_receipt_from_snapshots(receipt, bindings, before, &after).is_ok()
                    && changed_files.len() == receipt.changed_paths.iter().filter(|path| after.contains_key(*path)).count()
                    && receipt.changed_paths.iter().all(|path| {
                        after.get(path).map_or(!changed_files.contains_key(path), |bytes| changed_files.get(path) == Some(bytes))
                    })
            }
            (None, None, Some(observation)) => request.allow_unchanged
                && crate::candidate::observe_unchanged_from_snapshots(bindings, before, &after)
                    .is_ok_and(|expected| &expected == observation),
            _ => false,
        }
    });
    if valid { return outcome; }
    match outcome {
        crate::candidate_worker::Outcome::Produced { stdout, stderr, .. }
        | crate::candidate_worker::Outcome::Unchanged { stdout, stderr, .. } =>
            crate::candidate_worker::Outcome::Unknown {
                blocker: "Animus host candidate differs from independently captured workspace".into(),
                stdout, stderr,
            },
        other => other,
    }
}

pub(super) async fn execute(
    request: &super::Request,
    worker: crate::candidate_worker::Request,
    store: crate::store::Store,
    receipt: super::binding::usage::WorkerReceipt,
    serving: Box<dyn crate::execution_binding::serving::OwnerPort>,
    draw: Option<super::binding::draw::SingleShot>,
    profile: Profile,
    host: Box<dyn Host>,
    directory: &std::path::Path,
) -> Result<(), String> {
    let step = Step { request: request.clone(), worker, store, receipt, serving, draw, profile, host,
        directory: directory.to_path_buf() };
    let cancellation = std::sync::Arc::new(AtomicBool::new(false));
    let identity = step.receipt.request_identity().to_owned();
    let signal_task = super::cancellation::watch(cancellation.clone(), directory.join("cancel.request"), identity);
    let observed = tokio::task::spawn_blocking(move || step.observe(&cancellation)).await;
    let signal_result = super::cancellation::finish(signal_task).await;
    observed.map_err(|error| error.to_string())??;
    signal_result
}
