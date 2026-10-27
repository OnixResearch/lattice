//! Optional `no_std` daemon client, supervisor, and worker contracts.
//!
//! This crate contains pure role, routing, attachment, recovery, containment,
//! and metadata-only receipt decisions. Hosts own processes, sockets,
//! transcripts, persistence, clocks, and all other effects. Process separation
//! is lifecycle containment only; it is not a security sandbox.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
#![allow(clippy::too_many_arguments, clippy::uninlined_format_args)]
#![forbid(unsafe_code)]

extern crate alloc;

#[path = "handoff.rs"]
mod transaction;
/// Stable public module name for transactional handoff DTOs and ports.
pub mod handoff {
    pub use crate::transaction::*;
}
pub use transaction::*;

use alloc::{format, string::String, vec, vec::Vec};

/// The daemon contract schema.
pub const SCHEMA: &str = "animus.daemon-workers.v1";
/// The reviewed Prime Agent cohort.
pub const REVIEWED_COHORT: &str = "prime-agent-daemon-workers";
/// The reviewed cohort license.
pub const REVIEWED_LICENSE: &str = "MIT";
/// The exact reviewed Prime Agent revision.
pub const REVIEWED_REVISION: &str = "c5991bc853d27754aed345c13ff4d2e05c40f1f4";
/// The policy identity domain.
pub const POLICY_DOMAIN: &str = "animus.daemon-workers-policy.v1";
/// The envelope identity domain.
pub const ENVELOPE_DOMAIN: &str = "animus.daemon-workers-envelope.v1";
/// The worker-tree identity domain.
pub const TREE_DOMAIN: &str = "animus.daemon-workers-tree.v1";
/// The receipt identity domain.
pub const RECEIPT_DOMAIN: &str = "animus.daemon-workers-receipt.v1";
/// The BLAKE3 hexadecimal identity length.
pub const BLAKE3_HEX_CHARS: usize = 64;
/// The supported envelope schema version.
pub const ENVELOPE_VERSION: u16 = 1;
/// The minimum accepted sequence number.
pub const MIN_SEQUENCE: u64 = 1;
/// The minimum accepted attachment limit.
pub const MIN_ATTACHMENT_LIMIT: u16 = 1;
/// The minimum required receipt non-claims.
pub const MIN_NON_CLAIMS: usize = 4;
/// The required lifecycle-only containment non-claim.
pub const NOT_SECURITY_SANDBOX: &str = "not-security-sandbox";
/// The required transcript redaction marker.
pub const TRANSCRIPT_REDACTED: &str = "transcript-content-redacted";

const HASH_SEPARATOR: u8 = 0;
const MAX_LABEL_BYTES: usize = 256;
const MAX_RECEIPT_COUNT: u32 = 1_000;

/// A daemon process role.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Role {
    /// Renders observations, accepts input, and owns local UI preferences.
    Client,
    /// Discovers workers and owns routing, attachments, and health views.
    Supervisor,
    /// Owns exactly one root session tree and its execution descendants.
    Worker,
}

impl Role {
    /// Returns the stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Supervisor => "supervisor",
            Self::Worker => "worker",
        }
    }

    /// Returns the exact authority set for this role.
    #[must_use]
    pub const fn authorities(self) -> &'static [Authority] {
        match self {
            Self::Client => &[
                Authority::Rendering,
                Authority::Input,
                Authority::UiPreferences,
            ],
            Self::Supervisor => &[
                Authority::Discovery,
                Authority::Routing,
                Authority::Attachments,
                Authority::WorkerHealth,
                Authority::CatalogScan,
            ],
            Self::Worker => &[
                Authority::SessionTree,
                Authority::Queue,
                Authority::Schedules,
                Authority::KernelHandles,
                Authority::Descendants,
            ],
        }
    }

    /// Returns whether the role owns the authority.
    #[must_use]
    pub fn allows(self, authority: Authority) -> bool {
        let authorities = self.authorities();
        let mut index = 0;
        while index < authorities.len() {
            if authorities[index] == authority {
                return true;
            }
            index += 1;
        }
        false
    }
}

/// A role authority that can be checked without a host.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Authority {
    /// Client rendering authority.
    Rendering,
    /// Client input authority.
    Input,
    /// Client-local preference authority.
    UiPreferences,
    /// Supervisor worker discovery authority.
    Discovery,
    /// Supervisor envelope routing authority.
    Routing,
    /// Supervisor attachment authority.
    Attachments,
    /// Supervisor worker-health classification authority.
    WorkerHealth,
    /// Supervisor catalog-scan authority.
    CatalogScan,
    /// Worker root-session-tree authority.
    SessionTree,
    /// Worker prompt-queue authority.
    Queue,
    /// Worker schedule authority.
    Schedules,
    /// Worker kernel-handle authority.
    KernelHandles,
    /// Worker descendant authority.
    Descendants,
    /// An intentionally unsupported host effect.
    ProcessOrSocket,
}

impl Authority {
    /// Returns the stable authority label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rendering => "rendering",
            Self::Input => "input",
            Self::UiPreferences => "ui_preferences",
            Self::Discovery => "discovery",
            Self::Routing => "routing",
            Self::Attachments => "attachments",
            Self::WorkerHealth => "worker_health",
            Self::CatalogScan => "catalog_scan",
            Self::SessionTree => "session_tree",
            Self::Queue => "queue",
            Self::Schedules => "schedules",
            Self::KernelHandles => "kernel_handles",
            Self::Descendants => "descendants",
            Self::ProcessOrSocket => "process_or_socket",
        }
    }
}

/// A command action accepted by one role.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CommandAction {
    /// Render a bounded observation.
    Render,
    /// Submit client input.
    Input,
    /// Discover workers.
    Discover,
    /// Route an envelope.
    Route,
    /// Attach a client subscription.
    Attach,
    /// Detach a client subscription.
    Detach,
    /// Classify worker health.
    Health,
    /// Execute work in the owned worker tree.
    Execute,
}

impl CommandAction {
    const fn required_role(self) -> Role {
        match self {
            Self::Render | Self::Input => Role::Client,
            Self::Discover | Self::Route | Self::Attach | Self::Detach | Self::Health => {
                Role::Supervisor
            }
            Self::Execute => Role::Worker,
        }
    }

    /// Returns the stable command label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Render => "render",
            Self::Input => "input",
            Self::Discover => "discover",
            Self::Route => "route",
            Self::Attach => "attach",
            Self::Detach => "detach",
            Self::Health => "health",
            Self::Execute => "execute",
        }
    }
}

/// A versioned command envelope. Its body is a typed action, not a prompt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandEnvelope {
    /// Envelope schema version.
    pub version: u16,
    /// Stable caller-provided envelope label.
    pub envelope_id: String,
    /// Role that sent the command.
    pub source: Role,
    /// Role that receives the command.
    pub target: Role,
    /// Typed command action.
    pub action: CommandAction,
    /// Optional target worker identity.
    pub worker_id: Option<String>,
}

impl CommandEnvelope {
    /// Creates an envelope after checking role ownership and bounds.
    pub fn new(
        envelope_id: &str,
        source: Role,
        target: Role,
        action: CommandAction,
        worker_id: Option<&str>,
    ) -> Result<Self, Error> {
        validate_label(envelope_id)?;
        if source != action.required_role() {
            return Err(Error::OutOfRoleAuthority {
                role: source,
                authority: action.as_str().into(),
            });
        }
        if action == CommandAction::Execute && target != Role::Worker {
            return Err(Error::OutOfRoleAuthority {
                role: target,
                authority: Authority::SessionTree.as_str().into(),
            });
        }
        let worker_id = worker_id.map(String::from);
        if let Some(value) = &worker_id {
            validate_label(value)?;
        }
        Ok(Self {
            version: ENVELOPE_VERSION,
            envelope_id: envelope_id.into(),
            source,
            target,
            action,
            worker_id,
        })
    }

    /// Returns the domain-separated envelope identity.
    #[must_use]
    pub fn identity(&self) -> String {
        digest(
            ENVELOPE_DOMAIN,
            &format!(
                "{}|{}|{}|{}|{}|{}",
                self.version,
                self.envelope_id,
                self.source.as_str(),
                self.target.as_str(),
                self.action.as_str(),
                self.worker_id.as_deref().unwrap_or("none")
            ),
        )
    }
}

/// A versioned event envelope emitted by a role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventEnvelope {
    /// Envelope schema version.
    pub version: u16,
    /// Stable event label.
    pub event_id: String,
    /// Role that emitted the event.
    pub source: Role,
    /// Stable event class.
    pub class: EventClass,
}

/// A bounded event class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EventClass {
    /// Attachment changed.
    Attachment,
    /// Recovery was admitted.
    Recovery,
    /// Worker health changed.
    Health,
    /// Worker lifecycle containment changed.
    Containment,
}

impl EventClass {
    /// Returns the stable event label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Attachment => "attachment",
            Self::Recovery => "recovery",
            Self::Health => "health",
            Self::Containment => "containment",
        }
    }
}

/// A policy draft exported from Nickel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDraft {
    /// Reviewed cohort label.
    pub cohort: String,
    /// Reviewed source revision.
    pub source_revision: String,
    /// Maximum simultaneous client subscriptions per worker.
    pub max_attachments: u16,
    /// Maximum metadata count in one receipt.
    pub max_receipt_count: u32,
    /// Required receipt field names.
    pub receipt_fields: Vec<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
}

/// An admitted Nickel policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    /// Reviewed cohort label.
    pub cohort: String,
    /// Reviewed source revision.
    pub source_revision: String,
    /// Maximum simultaneous client subscriptions per worker.
    pub max_attachments: u16,
    /// Maximum metadata count in one receipt.
    pub max_receipt_count: u32,
    /// Required receipt field names.
    pub receipt_fields: Vec<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
    /// Domain-separated policy identity.
    pub identity: String,
}

impl Policy {
    /// Admits a generated policy and derives its BLAKE3 identity.
    pub fn new(draft: PolicyDraft) -> Result<Self, Error> {
        validate_label(&draft.cohort)?;
        validate_label(&draft.source_revision)?;
        if draft.cohort != REVIEWED_COHORT
            || draft.source_revision != REVIEWED_REVISION
            || draft.max_attachments < MIN_ATTACHMENT_LIMIT
            || draft.max_receipt_count == 0
            || draft.max_receipt_count > MAX_RECEIPT_COUNT
            || draft.receipt_fields.is_empty()
            || draft.non_claims.len() < MIN_NON_CLAIMS
            || !draft
                .non_claims
                .iter()
                .any(|value| value == NOT_SECURITY_SANDBOX)
        {
            return Err(Error::PolicyRejected);
        }
        for field in draft.receipt_fields.iter().chain(draft.non_claims.iter()) {
            validate_label(field)?;
        }
        let identity_material = format!(
            "{}|{}|{}|{}|{}|{}",
            draft.cohort,
            draft.source_revision,
            draft.max_attachments,
            draft.max_receipt_count,
            join(&draft.receipt_fields),
            join(&draft.non_claims),
        );
        Ok(Self {
            cohort: draft.cohort,
            source_revision: draft.source_revision,
            max_attachments: draft.max_attachments,
            max_receipt_count: draft.max_receipt_count,
            receipt_fields: draft.receipt_fields,
            non_claims: draft.non_claims,
            identity: digest(POLICY_DOMAIN, &identity_material),
        })
    }
}

/// Builds the reviewed policy used by fixtures and examples.
#[must_use]
pub fn reviewed_policy() -> Policy {
    Policy::new(PolicyDraft {
        cohort: REVIEWED_COHORT.into(),
        source_revision: REVIEWED_REVISION.into(),
        max_attachments: 4,
        max_receipt_count: 32,
        receipt_fields: vec![
            "policy_identity".into(),
            "role".into(),
            "envelope_identity".into(),
            "attachment_class".into(),
            "recovery_class".into(),
            "health_class".into(),
            "bounded_counts".into(),
            "non_claims".into(),
        ],
        non_claims: vec![
            NOT_SECURITY_SANDBOX.into(),
            "not-crash-freedom".into(),
            "not-arbitrary-host-durability".into(),
            "not-multi-machine-supervision".into(),
        ],
    })
    .expect("reviewed daemon policy is valid")
}

/// The state of a worker process from the supervisor's supplied observation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkerHealth {
    /// Worker is running and accepting routed work.
    Healthy,
    /// Worker is running but needs host recovery attention.
    Degraded,
    /// Worker is not available for routing.
    Unavailable,
}

impl WorkerHealth {
    /// Returns the stable health label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}

/// The one root session tree owned by one worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerTree {
    /// Owning worker identity.
    pub worker_id: String,
    /// Exactly one root session identity.
    pub root_session_id: String,
    /// Host-owned queue identity.
    pub queue_id: String,
    /// Number of admitted schedules.
    pub schedule_count: u32,
    /// Number of host-owned kernel handles.
    pub kernel_handle_count: u32,
    /// Number of descendants below the root.
    pub descendant_count: u32,
    /// Domain-separated tree identity.
    pub identity: String,
}

impl WorkerTree {
    /// Creates a single-tree ownership record without opening any handle.
    pub fn new(
        worker_id: &str,
        root_session_id: &str,
        queue_id: &str,
        schedule_count: u32,
        kernel_handle_count: u32,
        descendant_count: u32,
    ) -> Result<Self, Error> {
        validate_label(worker_id)?;
        validate_label(root_session_id)?;
        validate_label(queue_id)?;
        let identity = digest(
            TREE_DOMAIN,
            &format!(
                "{}|{}|{}|{}|{}|{}",
                worker_id,
                root_session_id,
                queue_id,
                schedule_count,
                kernel_handle_count,
                descendant_count
            ),
        );
        Ok(Self {
            worker_id: worker_id.into(),
            root_session_id: root_session_id.into(),
            queue_id: queue_id.into(),
            schedule_count,
            kernel_handle_count,
            descendant_count,
            identity,
        })
    }

    /// Checks that access stays within this worker's tree.
    pub fn admit_tree_access(&self, requester_worker_id: &str) -> Result<(), Error> {
        validate_label(requester_worker_id)?;
        if requester_worker_id != self.worker_id {
            return Err(Error::CrossWorkerTreeAccess);
        }
        Ok(())
    }

    /// Recomputes the tree identity to detect forged ownership records.
    #[must_use]
    pub fn identity_matches(&self) -> bool {
        digest(
            TREE_DOMAIN,
            &format!(
                "{}|{}|{}|{}|{}|{}",
                self.worker_id,
                self.root_session_id,
                self.queue_id,
                self.schedule_count,
                self.kernel_handle_count,
                self.descendant_count
            ),
        ) == self.identity
    }
}

/// A worker observation supplied to the supervisor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerRecord {
    /// Worker identity.
    pub worker_id: String,
    /// Root session identity.
    pub root_session_id: String,
    /// Worker health.
    pub health: WorkerHealth,
    /// Current attachment count.
    pub attachment_count: u16,
}

/// Canonical supervisor registry state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupervisorState {
    /// Sorted worker observations.
    pub workers: Vec<WorkerRecord>,
}

impl SupervisorState {
    /// Creates a deterministic registry and rejects duplicate workers.
    pub fn new(mut workers: Vec<WorkerRecord>) -> Result<Self, Error> {
        for worker in &workers {
            validate_label(&worker.worker_id)?;
            validate_label(&worker.root_session_id)?;
        }
        workers.sort_by(|left, right| left.worker_id.cmp(&right.worker_id));
        if workers
            .windows(2)
            .any(|pair| pair[0].worker_id == pair[1].worker_id)
        {
            return Err(Error::DuplicateWorker);
        }
        Ok(Self { workers })
    }
}

/// A pure supervisor operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupervisorRequest {
    /// List known workers.
    Discover,
    /// Route a command envelope.
    Route(CommandEnvelope),
    /// Admit one client subscription.
    Attach {
        /// Client identity.
        client_id: String,
        /// Worker identity.
        worker_id: String,
    },
    /// End one client subscription.
    Detach {
        /// Client identity.
        client_id: String,
        /// Worker identity.
        worker_id: String,
    },
    /// Return one worker's supplied health.
    Health {
        /// Worker identity.
        worker_id: String,
    },
    /// Count catalog entries without discovering host state.
    CatalogScan,
}

/// A pure supervisor result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupervisorDecision {
    /// Sorted worker identities.
    Discovered(Vec<String>),
    /// Target worker selected for a routed command.
    Routed {
        /// Worker identity.
        worker_id: String,
    },
    /// Attachment accepted.
    Attached {
        /// Worker identity.
        worker_id: String,
    },
    /// Detachment accepted without changing worker execution.
    Detached {
        /// Worker identity.
        worker_id: String,
    },
    /// Supplied health classification.
    Health(WorkerHealth),
    /// Number of catalog entries observed.
    CatalogScanned {
        /// Number of workers in the supplied catalog.
        worker_count: usize,
    },
}

/// Runs a supervisor decision over supplied state only.
pub fn decide_supervisor(
    state: &SupervisorState,
    request: &SupervisorRequest,
    policy: &Policy,
) -> Result<SupervisorDecision, Error> {
    match request {
        SupervisorRequest::Discover => Ok(SupervisorDecision::Discovered(
            state
                .workers
                .iter()
                .map(|worker| worker.worker_id.clone())
                .collect(),
        )),
        SupervisorRequest::CatalogScan => Ok(SupervisorDecision::CatalogScanned {
            worker_count: state.workers.len(),
        }),
        SupervisorRequest::Health { worker_id } => {
            find_worker(state, worker_id).map(|worker| SupervisorDecision::Health(worker.health))
        }
        SupervisorRequest::Route(envelope) => {
            validate_envelope(envelope)?;
            if envelope.source == Role::Supervisor && envelope.action == CommandAction::Execute {
                return Err(Error::SupervisorExecutionDenied);
            }
            let worker_id = envelope.worker_id.as_deref().ok_or(Error::UnknownWorker)?;
            let worker = find_worker(state, worker_id)?;
            if worker.health == WorkerHealth::Unavailable {
                return Err(Error::WorkerUnavailable);
            }
            Ok(SupervisorDecision::Routed {
                worker_id: worker.worker_id.clone(),
            })
        }
        SupervisorRequest::Attach {
            client_id,
            worker_id,
        } => {
            validate_label(client_id)?;
            let worker = find_worker(state, worker_id)?;
            if worker.attachment_count >= policy.max_attachments {
                return Err(Error::AttachmentLimitExceeded);
            }
            if worker.health == WorkerHealth::Unavailable {
                return Err(Error::WorkerUnavailable);
            }
            Ok(SupervisorDecision::Attached {
                worker_id: worker.worker_id.clone(),
            })
        }
        SupervisorRequest::Detach {
            client_id,
            worker_id,
        } => {
            validate_label(client_id)?;
            let worker = find_worker(state, worker_id)?;
            Ok(SupervisorDecision::Detached {
                worker_id: worker.worker_id.clone(),
            })
        }
    }
}

/// A live stream or typed recovery source for reattach.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReattachSource {
    /// Reconnect to a live worker stream.
    LiveStream {
        /// Last sequence observed by the client.
        last_sequence: u64,
    },
    /// Reconnect from a host-owned recovery snapshot.
    RecoverySnapshot {
        /// Snapshot identity, not snapshot content.
        snapshot_identity: String,
        /// Typed snapshot freshness.
        freshness: RecoveryFreshness,
    },
}

/// Freshness of a recovery snapshot.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RecoveryFreshness {
    /// Snapshot is current for the worker generation.
    Current,
    /// Snapshot is stale and needs repair before use.
    Stale,
    /// Freshness was not established.
    Unknown,
}

impl RecoveryFreshness {
    /// Returns the stable freshness label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Stale => "stale",
            Self::Unknown => "unknown",
        }
    }
}

/// A pure reattach decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReattachPlan {
    /// Selected recovery class.
    pub source: ReattachSource,
    /// The worker remains the execution owner.
    pub execution_continues: bool,
}

/// Admits a live or current recovery reattach.
pub fn admit_reattach(source: ReattachSource) -> Result<ReattachPlan, Error> {
    match &source {
        ReattachSource::LiveStream { last_sequence } if *last_sequence < MIN_SEQUENCE => {
            Err(Error::InvalidSequence)
        }
        ReattachSource::LiveStream { .. } => Ok(ReattachPlan {
            source,
            execution_continues: true,
        }),
        ReattachSource::RecoverySnapshot {
            snapshot_identity,
            freshness,
        } => {
            validate_hash(snapshot_identity)?;
            if *freshness != RecoveryFreshness::Current {
                return Err(Error::StaleRecoverySnapshot);
            }
            Ok(ReattachPlan {
                source,
                execution_continues: true,
            })
        }
    }
}

/// A pure execution observation that contains no attachment state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionDecision {
    /// The worker may continue independently of clients.
    pub execution_continues: bool,
    /// The worker tree identity bound to the decision.
    pub tree_identity: String,
    /// Pending work count supplied by the host.
    pub pending_work: u32,
}

/// Plans worker execution without reading attachment state.
pub fn plan_worker_execution(
    tree: &WorkerTree,
    pending_work: u32,
) -> Result<ExecutionDecision, Error> {
    if !tree.identity_matches() {
        return Err(Error::ForgedWorkerTree);
    }
    Ok(ExecutionDecision {
        execution_continues: true,
        tree_identity: tree.identity.clone(),
        pending_work,
    })
}

/// Lifecycle-only facts provided by process separation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContainmentFact {
    /// Worker and kernel have distinct lifecycle domains.
    IndependentCrashDomain,
    /// Host can tear down one lifecycle domain.
    ExplicitTeardown,
    /// Host may restart a failed lifecycle domain.
    RestartableWorker,
}

impl ContainmentFact {
    /// Returns the stable containment label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IndependentCrashDomain => "independent-crash-domain",
            Self::ExplicitTeardown => "explicit-teardown",
            Self::RestartableWorker => "restartable-worker",
        }
    }
}

/// A containment observation with no security claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainmentReport {
    /// Lifecycle facts observed by the host.
    pub facts: Vec<ContainmentFact>,
    /// Must remain the required non-claim.
    pub non_claim: String,
    /// Optional untrusted claim material, rejected if present.
    pub claim: Option<String>,
}

/// Validates lifecycle containment and rejects sandbox or security claims.
pub fn validate_containment(report: &ContainmentReport) -> Result<(), Error> {
    if report.facts.is_empty() || report.non_claim != NOT_SECURITY_SANDBOX {
        return Err(Error::ContainmentClaimRejected);
    }
    if report.claim.is_some() {
        return Err(Error::ContainmentClaimRejected);
    }
    Ok(())
}

/// The attachment class bound into a receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AttachmentClass {
    /// Client subscription is live.
    Live,
    /// Client is detached.
    Detached,
}

impl AttachmentClass {
    /// Returns the stable attachment label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Detached => "detached",
        }
    }
}

/// The recovery class bound into a receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RecoveryClass {
    /// No recovery was required.
    None,
    /// A live stream was selected.
    LiveStream,
    /// A current snapshot was selected.
    CurrentSnapshot,
}

impl RecoveryClass {
    /// Returns the stable recovery label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LiveStream => "live_stream",
            Self::CurrentSnapshot => "current_snapshot",
        }
    }
}

/// A metadata-only receipt draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptDraft {
    /// Role that produced the event.
    pub role: Role,
    /// Envelope identity.
    pub envelope_identity: String,
    /// Attachment class.
    pub attachment_class: AttachmentClass,
    /// Recovery class.
    pub recovery_class: RecoveryClass,
    /// Worker health class.
    pub health_class: WorkerHealth,
    /// Bounded count metadata.
    pub bounded_count: u32,
    /// Required containment non-claim.
    pub non_claim: String,
    /// Untrusted content material, rejected without echoing.
    pub content: Option<String>,
    /// Untrusted overclaim material, rejected without echoing.
    pub overclaim: Option<String>,
}

/// A validated metadata-only daemon receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DaemonReceipt {
    /// Policy identity.
    pub policy_identity: String,
    /// Receipt role.
    pub role: Role,
    /// Envelope identity.
    pub envelope_identity: String,
    /// Attachment class.
    pub attachment_class: AttachmentClass,
    /// Recovery class.
    pub recovery_class: RecoveryClass,
    /// Health class.
    pub health_class: WorkerHealth,
    /// Bounded count metadata.
    pub bounded_count: u32,
    /// Required non-claim.
    pub non_claim: String,
    /// Domain-separated receipt identity.
    pub identity: String,
}

/// Projects a receipt without copying transcript or prompt content.
pub fn project_receipt(policy: &Policy, draft: &ReceiptDraft) -> Result<DaemonReceipt, Error> {
    if draft.non_claim != NOT_SECURITY_SANDBOX
        || draft.content.is_some()
        || draft.overclaim.is_some()
    {
        return Err(Error::ReceiptContainsContent);
    }
    if draft.bounded_count > policy.max_receipt_count {
        return Err(Error::ReceiptCountExceeded);
    }
    validate_hash(&draft.envelope_identity)?;
    let material = format!(
        "{}|{}|{}|{}|{}|{}|{}",
        policy.identity,
        draft.role.as_str(),
        draft.envelope_identity,
        draft.attachment_class.as_str(),
        draft.recovery_class.as_str(),
        draft.health_class.as_str(),
        draft.bounded_count
    );
    Ok(DaemonReceipt {
        policy_identity: policy.identity.clone(),
        role: draft.role,
        envelope_identity: draft.envelope_identity.clone(),
        attachment_class: draft.attachment_class,
        recovery_class: draft.recovery_class,
        health_class: draft.health_class,
        bounded_count: draft.bounded_count,
        non_claim: draft.non_claim.clone(),
        identity: digest(RECEIPT_DOMAIN, &material),
    })
}

/// Typed failures for daemon decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A label is empty or too large.
    InvalidLabel,
    /// A role attempted an authority owned by another role.
    OutOfRoleAuthority {
        /// Role that attempted the action.
        role: Role,
        /// Authority that was denied.
        authority: String,
    },
    /// The generated policy failed closed.
    PolicyRejected,
    /// The envelope version or role action is invalid.
    InvalidEnvelope,
    /// The target worker is absent.
    UnknownWorker,
    /// The target worker is unavailable.
    WorkerUnavailable,
    /// The supervisor attempted to execute session work.
    SupervisorExecutionDenied,
    /// The attachment bound was reached.
    AttachmentLimitExceeded,
    /// Two worker records have the same identity.
    DuplicateWorker,
    /// A worker attempted access to another worker's tree.
    CrossWorkerTreeAccess,
    /// A worker tree identity does not match its fields.
    ForgedWorkerTree,
    /// A sequence is below the admitted minimum.
    InvalidSequence,
    /// A recovery snapshot is stale or unknown.
    StaleRecoverySnapshot,
    /// A hash is not a lowercase BLAKE3 identity.
    InvalidHash,
    /// Containment included a forbidden security claim.
    ContainmentClaimRejected,
    /// A receipt draft contained private content or overclaim material.
    ReceiptContainsContent,
    /// A receipt count exceeded policy.
    ReceiptCountExceeded,
}

fn validate_envelope(envelope: &CommandEnvelope) -> Result<(), Error> {
    if envelope.version != ENVELOPE_VERSION
        || envelope.envelope_id.is_empty()
        || envelope.source != envelope.action.required_role()
    {
        return Err(Error::InvalidEnvelope);
    }
    Ok(())
}

fn find_worker<'a>(state: &'a SupervisorState, worker_id: &str) -> Result<&'a WorkerRecord, Error> {
    state
        .workers
        .iter()
        .find(|worker| worker.worker_id == worker_id)
        .ok_or(Error::UnknownWorker)
}

fn validate_label(value: &str) -> Result<(), Error> {
    if value.is_empty() || value.len() > MAX_LABEL_BYTES || value.contains('\n') {
        return Err(Error::InvalidLabel);
    }
    Ok(())
}

fn validate_hash(value: &str) -> Result<(), Error> {
    if value.len() != BLAKE3_HEX_CHARS
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(Error::InvalidHash);
    }
    Ok(())
}

fn digest(domain: &str, material: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[HASH_SEPARATOR]);
    hasher.update(material.as_bytes());
    format!("{}", hasher.finalize().to_hex())
}

fn join(values: &[String]) -> String {
    let mut result = String::new();
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            result.push('|');
        }
        result.push_str(value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORKER: &str = "worker-a";
    const SESSION: &str = "session-a";
    const QUEUE: &str = "queue-a";
    const ENVELOPE: &str = "envelope-a";
    const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn tree() -> WorkerTree {
        WorkerTree::new(WORKER, SESSION, QUEUE, 2, 1, 3).expect("tree")
    }

    #[test]
    fn role_authority_and_worker_tree_are_exclusive() {
        assert!(Role::Client.allows(Authority::Rendering));
        assert!(!Role::Client.allows(Authority::SessionTree));
        assert!(Role::Supervisor.allows(Authority::Routing));
        assert!(Role::Worker.allows(Authority::KernelHandles));
        assert_eq!(tree().admit_tree_access(WORKER), Ok(()));
    }

    #[test]
    fn envelopes_and_supervisor_routing_are_deterministic() {
        let envelope = CommandEnvelope::new(
            ENVELOPE,
            Role::Supervisor,
            Role::Worker,
            CommandAction::Route,
            Some(WORKER),
        )
        .expect("envelope");
        let state = SupervisorState::new(vec![WorkerRecord {
            worker_id: WORKER.into(),
            root_session_id: SESSION.into(),
            health: WorkerHealth::Healthy,
            attachment_count: 0,
        }])
        .expect("state");
        let decision = decide_supervisor(
            &state,
            &SupervisorRequest::Route(envelope),
            &reviewed_policy(),
        )
        .expect("route");
        assert_eq!(
            decision,
            SupervisorDecision::Routed {
                worker_id: WORKER.into()
            }
        );
    }

    #[test]
    fn detach_does_not_change_worker_execution_and_live_reattach_continues() {
        let execution = plan_worker_execution(&tree(), 2).expect("execution");
        assert!(execution.execution_continues);
        let reattach =
            admit_reattach(ReattachSource::LiveStream { last_sequence: 1 }).expect("live reattach");
        assert!(reattach.execution_continues);
        let snapshot = admit_reattach(ReattachSource::RecoverySnapshot {
            snapshot_identity: HASH.into(),
            freshness: RecoveryFreshness::Current,
        })
        .expect("snapshot reattach");
        assert!(snapshot.execution_continues);
    }

    #[test]
    fn containment_and_receipt_projection_are_metadata_only() {
        validate_containment(&ContainmentReport {
            facts: vec![
                ContainmentFact::IndependentCrashDomain,
                ContainmentFact::ExplicitTeardown,
            ],
            non_claim: NOT_SECURITY_SANDBOX.into(),
            claim: None,
        })
        .expect("containment");
        let receipt = project_receipt(
            &reviewed_policy(),
            &ReceiptDraft {
                role: Role::Worker,
                envelope_identity: HASH.into(),
                attachment_class: AttachmentClass::Detached,
                recovery_class: RecoveryClass::CurrentSnapshot,
                health_class: WorkerHealth::Healthy,
                bounded_count: 1,
                non_claim: NOT_SECURITY_SANDBOX.into(),
                content: None,
                overclaim: None,
            },
        )
        .expect("receipt");
        assert_eq!(receipt.identity.len(), BLAKE3_HEX_CHARS);
        assert_eq!(receipt.non_claim, NOT_SECURITY_SANDBOX);
    }

    #[test]
    fn negative_out_of_role_cross_worker_stale_and_claim_inputs_fail_closed() {
        assert!(matches!(
            CommandEnvelope::new(
                ENVELOPE,
                Role::Client,
                Role::Worker,
                CommandAction::Execute,
                None
            ),
            Err(Error::OutOfRoleAuthority { .. })
        ));
        assert_eq!(
            tree().admit_tree_access("worker-b"),
            Err(Error::CrossWorkerTreeAccess)
        );
        assert_eq!(
            admit_reattach(ReattachSource::RecoverySnapshot {
                snapshot_identity: HASH.into(),
                freshness: RecoveryFreshness::Stale,
            }),
            Err(Error::StaleRecoverySnapshot)
        );
        assert_eq!(
            validate_containment(&ContainmentReport {
                facts: vec![ContainmentFact::RestartableWorker],
                non_claim: NOT_SECURITY_SANDBOX.into(),
                claim: Some("security sandbox".into()),
            }),
            Err(Error::ContainmentClaimRejected)
        );
        assert_eq!(
            project_receipt(
                &reviewed_policy(),
                &ReceiptDraft {
                    role: Role::Worker,
                    envelope_identity: HASH.into(),
                    attachment_class: AttachmentClass::Live,
                    recovery_class: RecoveryClass::None,
                    health_class: WorkerHealth::Healthy,
                    bounded_count: 1,
                    non_claim: NOT_SECURITY_SANDBOX.into(),
                    content: Some("prompt body".into()),
                    overclaim: None,
                },
            ),
            Err(Error::ReceiptContainsContent)
        );
    }
}
