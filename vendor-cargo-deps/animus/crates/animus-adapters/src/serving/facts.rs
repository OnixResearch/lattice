//! Bounded host-supplied availability snapshots.

/// Serving readiness as observed by the snapshot source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Readiness {
    /// The service is listed but no deployment is present.
    Absent,
    /// Readiness was not established.
    Unknown,
    /// The model is listed but still loading.
    Loading,
    /// The deployment reports it can accept requests.
    Ready,
    /// The deployment is draining and must not receive new requests.
    Draining,
    /// The deployment reports it cannot accept requests.
    Unavailable,
    /// The source itself marked this observation stale.
    Stale,
}

impl Readiness {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Unknown => "unknown",
            Self::Loading => "loading",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Unavailable => "unavailable",
            Self::Stale => "stale",
        }
    }
}

/// Where requests to the service are executed, for privacy screening.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Locality {
    /// Executes on the caller's host.
    Local,
    /// Executes on a privately administered remote host.
    PrivateRemote,
    /// Executes at a hosted API provider.
    HostedRemote,
}

impl Locality {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::PrivateRemote => "private-remote",
            Self::HostedRemote => "hosted-remote",
        }
    }
}

/// Whether the serving owner offers request-scoped admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionOffer {
    /// The serving owner grants or rejects each request before dispatch.
    OwnerAdmission,
    /// No request admission is offered.
    NotOffered,
}

impl AdmissionOffer {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OwnerAdmission => "owner-admission",
            Self::NotOffered => "not-offered",
        }
    }
}

/// Capability support per request feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilitySet {
    /// Text generation.
    pub text: crate::mesh::Capability,
    /// Tool calls.
    pub tools: crate::mesh::Capability,
    /// Streaming.
    pub streaming: crate::mesh::Capability,
    /// Structured output.
    pub structured_output: crate::mesh::Capability,
    /// Developer role messages.
    pub developer_role: crate::mesh::Capability,
}

impl CapabilitySet {
    /// Capabilities in fixed order: text, tools, streaming, structured output, developer role.
    #[must_use]
    pub const fn ordered(self) -> [crate::mesh::Capability; 5] {
        [
            self.text,
            self.tools,
            self.streaming,
            self.structured_output,
            self.developer_role,
        ]
    }
}

/// Stable capability label.
#[must_use]
pub const fn capability_label(capability: crate::mesh::Capability) -> &'static str {
    match capability {
        crate::mesh::Capability::Supported => "supported",
        crate::mesh::Capability::Unsupported => "unsupported",
        crate::mesh::Capability::Unknown => "unknown",
    }
}

/// Supported request envelope in tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Envelope {
    max_context_tokens: u32,
    max_output_tokens: u32,
}

impl Envelope {
    /// Validates an envelope.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractIssue::InvalidEnvelope`] for zero or inverted bounds.
    pub const fn new(
        max_context_tokens: u32,
        max_output_tokens: u32,
    ) -> Result<Self, super::ContractIssue> {
        if max_output_tokens == 0 || max_context_tokens < max_output_tokens {
            return Err(super::ContractIssue::InvalidEnvelope);
        }
        Ok(Self {
            max_context_tokens,
            max_output_tokens,
        })
    }

    /// Maximum input plus output tokens.
    #[must_use]
    pub const fn max_context_tokens(self) -> u32 {
        self.max_context_tokens
    }

    /// Maximum output tokens.
    #[must_use]
    pub const fn max_output_tokens(self) -> u32 {
        self.max_output_tokens
    }

    /// Whether an `(input, output)` token request fits.
    #[must_use]
    pub const fn admits(self, tokens: (u32, u32)) -> bool {
        let total_tokens = tokens.0.saturating_add(tokens.1);
        tokens.1 <= self.max_output_tokens && total_tokens <= self.max_context_tokens
    }
}

/// A counter that may be unknown; unknown is never zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Counter {
    /// The counter was not established.
    Unknown,
    /// The counter's value.
    Known(u64),
}

/// What a capacity observation measures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityKind {
    /// Remaining concurrent request slots.
    RequestSlots,
    /// Requests waiting in the serving queue; load, not capacity.
    QueueDepth,
    /// Remaining KV-cache token capacity.
    KvCacheTokens,
    /// Remaining hosted-provider request quota.
    RequestQuota,
    /// Remaining hosted-provider token quota.
    TokenQuota,
    /// Free accelerator or host memory; never request capacity.
    FreeMemory,
}

impl CapacityKind {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestSlots => "request-slots",
            Self::QueueDepth => "queue-depth",
            Self::KvCacheTokens => "kv-cache-tokens",
            Self::RequestQuota => "request-quota",
            Self::TokenQuota => "token-quota",
            Self::FreeMemory => "free-memory",
        }
    }

    /// The only unit that measures this kind.
    #[must_use]
    pub const fn unit(self) -> CapacityUnit {
        match self {
            Self::RequestSlots | Self::QueueDepth | Self::RequestQuota => CapacityUnit::Requests,
            Self::KvCacheTokens | Self::TokenQuota => CapacityUnit::Tokens,
            Self::FreeMemory => CapacityUnit::Bytes,
        }
    }

    /// Whether this kind can evidence remaining request capacity.
    #[must_use]
    pub const fn gates_requests(self) -> bool {
        match self {
            Self::RequestSlots | Self::KvCacheTokens | Self::RequestQuota | Self::TokenQuota => {
                true
            }
            Self::QueueDepth | Self::FreeMemory => false,
        }
    }
}

/// Unit of a capacity observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityUnit {
    /// Requests.
    Requests,
    /// Tokens.
    Tokens,
    /// Bytes.
    Bytes,
}

impl CapacityUnit {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requests => "requests",
            Self::Tokens => "tokens",
            Self::Bytes => "bytes",
        }
    }
}

/// Scope that a capacity observation covers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityScope {
    /// One replica.
    Replica,
    /// One deployment generation.
    Deployment,
    /// The logical service across its deployments.
    Service,
    /// A hosted-provider account or key quota.
    Account,
}

impl CapacityScope {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Replica => "replica",
            Self::Deployment => "deployment",
            Self::Service => "service",
            Self::Account => "account",
        }
    }
}

/// Who produced a capacity observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityOrigin {
    /// The serving owner.
    ServingOwner,
    /// A hosted provider's API, for example rate-limit headers.
    ProviderApi,
    /// A host probe; recorded but never sufficient capacity evidence.
    HostProbe,
}

impl CapacityOrigin {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ServingOwner => "serving-owner",
            Self::ProviderApi => "provider-api",
            Self::HostProbe => "host-probe",
        }
    }
}

/// Borrowed capacity observation fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapacityDraft {
    /// Measured quantity.
    pub kind: CapacityKind,
    /// Explicit unit; must match the kind.
    pub unit: CapacityUnit,
    /// Explicit scope.
    pub scope: CapacityScope,
    /// Explicit source.
    pub origin: CapacityOrigin,
    /// Counter value or unknown.
    pub value: Counter,
    /// Source logical tick of the observation.
    pub observed_tick: u64,
    /// Exclusive logical tick at which the observation expires.
    pub valid_until_tick: u64,
}

/// Validated capacity observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapacityObservation(CapacityDraft);

impl CapacityObservation {
    fn new(draft: CapacityDraft) -> Result<Self, super::ContractIssue> {
        if draft.kind.unit() != draft.unit {
            return Err(super::ContractIssue::UnitMismatch);
        }
        super::basis::window(
            "capacity.valid_until_tick",
            (draft.observed_tick, draft.valid_until_tick),
        )?;
        Ok(Self(draft))
    }

    /// The observation fields.
    #[must_use]
    pub const fn fields(&self) -> &CapacityDraft {
        &self.0
    }

    /// Whether this is trusted, unexpired request-gating evidence at `now_tick`.
    pub(super) fn is_fresh_gating(&self, now_tick: u64) -> bool {
        self.is_trusted_gating() && self.0.valid_until_tick > now_tick
    }

    /// Whether this request-gating observation came from the owner or provider.
    pub(super) fn is_trusted_gating(&self) -> bool {
        self.0.kind.gates_requests() && self.0.origin != CapacityOrigin::HostProbe
    }
}

/// Borrowed model identity claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelDraft<'a> {
    /// Exact model identifier.
    pub model_id: &'a str,
    /// Exact configuration identity.
    pub configuration_ref: &'a str,
    /// Whether the source claimed or the host observed this identity.
    pub attestation: super::Attestation,
}

/// Borrowed service observation fields.
#[derive(Clone, Copy, Debug)]
pub struct ServiceDraft<'a> {
    /// Logical service identifier.
    pub service_id: &'a str,
    /// Safe endpoint identity; not a URL.
    pub endpoint_identity: &'a str,
    /// Serving deployment identifier.
    pub deployment_id: &'a str,
    /// Deployment generation (> 0).
    pub deployment_generation: u64,
    /// Execution locality.
    pub locality: Locality,
    /// Model identity; `None` means unknown.
    pub model: Option<ModelDraft<'a>>,
    /// Readiness.
    pub readiness: Readiness,
    /// Capability support.
    pub capabilities: CapabilitySet,
    /// Supported request envelope; `None` means unknown.
    pub envelope: Option<Envelope>,
    /// Capacity observations with explicit units, scope, source, and freshness.
    pub capacity: &'a [CapacityDraft],
    /// Request-admission offer.
    pub admission: AdmissionOffer,
    /// Replica placement if the provider exposes it.
    pub replica: Option<&'a str>,
    /// Opaque compute consumer reference, for example a serving allocation id.
    pub resource_consumer_ref: Option<&'a str>,
    /// Rollout facts when the source qualifies a rollout profile.
    pub rollout: Option<super::RolloutDraft<'a>>,
}

/// Validated service observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceObservation {
    pub(super) deployment: super::DeploymentBinding,
    pub(super) locality: Locality,
    pub(super) model: Option<(super::ModelBinding, super::Attestation)>,
    pub(super) readiness: Readiness,
    pub(super) capabilities: CapabilitySet,
    pub(super) envelope: Option<Envelope>,
    pub(super) capacity: alloc::vec::Vec<CapacityObservation>,
    pub(super) admission: AdmissionOffer,
    pub(super) replica: super::ReplicaFact,
    pub(super) resource_consumer_ref: Option<super::Token>,
    pub(super) rollout: Option<super::RolloutFacts>,
}

impl ServiceObservation {
    fn new(draft: &ServiceDraft<'_>) -> Result<Self, super::ContractIssue> {
        let model = match draft.model {
            None => None,
            Some(model) => Some((
                super::ModelBinding::new(model.model_id, model.configuration_ref)?,
                model.attestation,
            )),
        };
        let resource_consumer_ref = match draft.resource_consumer_ref {
            None => None,
            Some(value) => Some(super::Token::new("resource_consumer_ref", value)?),
        };
        let rollout = match draft.rollout {
            None => None,
            Some(rollout) => Some(super::RolloutFacts::new(&rollout)?),
        };
        Ok(Self {
            deployment: deployment_binding(draft)?,
            locality: draft.locality,
            model,
            readiness: draft.readiness,
            capabilities: draft.capabilities,
            envelope: draft.envelope,
            capacity: collect_capacity(draft.capacity)?,
            admission: draft.admission,
            replica: super::ReplicaFact::new(draft.replica)?,
            resource_consumer_ref,
            rollout,
        })
    }

    /// Deployment binding.
    #[must_use]
    pub const fn deployment(&self) -> &super::DeploymentBinding {
        &self.deployment
    }

    /// Execution locality.
    #[must_use]
    pub const fn locality(&self) -> Locality {
        self.locality
    }

    /// Readiness.
    #[must_use]
    pub const fn readiness(&self) -> Readiness {
        self.readiness
    }

    /// Model identity and how it was established; `None` is unknown.
    #[must_use]
    pub fn model(&self) -> Option<(&super::ModelBinding, super::Attestation)> {
        self.model
            .as_ref()
            .map(|(model, attestation)| (model, *attestation))
    }

    /// Capacity observations.
    #[must_use]
    pub fn capacity(&self) -> &[CapacityObservation] {
        self.capacity.as_slice()
    }

    /// Opaque compute consumer reference.
    #[must_use]
    pub fn resource_consumer_ref(&self) -> Option<&str> {
        self.resource_consumer_ref
            .as_ref()
            .map(super::Token::as_str)
    }
}

fn deployment_binding(
    draft: &ServiceDraft<'_>,
) -> Result<super::DeploymentBinding, super::ContractIssue> {
    Ok(super::DeploymentBinding {
        service_id: super::Token::new("service_id", draft.service_id)?,
        endpoint_identity: super::Token::new("endpoint_identity", draft.endpoint_identity)?,
        deployment_id: super::Token::new("deployment_id", draft.deployment_id)?,
        deployment_generation: super::basis::positive(
            "deployment_generation",
            draft.deployment_generation,
        )?,
    })
}

fn collect_capacity(
    drafts: &[CapacityDraft],
) -> Result<alloc::vec::Vec<CapacityObservation>, super::ContractIssue> {
    let count =
        super::basis::bounded_len("capacity", drafts.len(), super::MAX_CAPACITY_OBSERVATIONS)?;
    let mut observations = alloc::vec::Vec::with_capacity(drafts.len());
    for draft in drafts {
        let is_repeated = observations.iter().any(|seen: &CapacityObservation| {
            seen.0.kind == draft.kind && seen.0.scope == draft.scope
        });
        if is_repeated {
            return Err(super::ContractIssue::Duplicate { field: "capacity" });
        }
        observations.push(CapacityObservation::new(*draft)?);
    }
    debug_assert_eq!(u32::try_from(observations.len()).ok(), Some(count));
    Ok(observations)
}

/// Borrowed snapshot fields.
#[derive(Clone, Copy, Debug)]
pub struct SnapshotDraft<'a> {
    /// Snapshot source identity.
    pub source_id: &'a str,
    /// Source generation (> 0); a restarted source increments it.
    pub source_generation: u64,
    /// Observation sequence (> 0), strictly increasing per source generation.
    pub sequence: u64,
    /// Source logical tick of the observation.
    pub observed_tick: u64,
    /// Exclusive logical tick at which the snapshot expires.
    pub valid_until_tick: u64,
    /// Service observations (1 to [`super::MAX_SERVICES`]).
    pub services: &'a [ServiceDraft<'a>],
}

/// Validated availability snapshot with a BLAKE3 identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AvailabilitySnapshot {
    pub(super) source_id: super::Token,
    pub(super) source_generation: u64,
    pub(super) sequence: u64,
    pub(super) observed_tick: u64,
    pub(super) valid_until_tick: u64,
    pub(super) services: alloc::vec::Vec<ServiceObservation>,
    pub(super) identity: super::ContentRef,
}

impl AvailabilitySnapshot {
    /// Validates a snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractIssue`] for unsafe identities, zero generations
    /// or sequences, inverted windows, empty, oversized, or duplicate service
    /// lists, unit mismatches, invalid envelopes, and invalid rollout facts.
    pub fn new(draft: &SnapshotDraft<'_>) -> Result<Self, super::ContractIssue> {
        let source_id = super::Token::new("source_id", draft.source_id)?;
        let source_generation =
            super::basis::positive("source_generation", draft.source_generation)?;
        let sequence = super::basis::positive("sequence", draft.sequence)?;
        super::basis::window(
            "valid_until_tick",
            (draft.observed_tick, draft.valid_until_tick),
        )?;
        let services = collect_services(draft.services)?;
        let identity = draft_identity(draft, &services);
        Ok(Self {
            source_id,
            source_generation,
            sequence,
            observed_tick: draft.observed_tick,
            valid_until_tick: draft.valid_until_tick,
            services,
            identity,
        })
    }

    /// BLAKE3 identity over every validated field.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Source identity.
    #[must_use]
    pub fn source_id(&self) -> &str {
        self.source_id.as_str()
    }

    /// Source generation.
    #[must_use]
    pub const fn source_generation(&self) -> u64 {
        self.source_generation
    }

    /// Observation sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Source `(observed_tick, valid_until_tick)` window.
    #[must_use]
    pub const fn validity_ticks(&self) -> (u64, u64) {
        (self.observed_tick, self.valid_until_tick)
    }

    /// Service observations.
    #[must_use]
    pub fn services(&self) -> &[ServiceObservation] {
        self.services.as_slice()
    }

    pub(super) fn service(&self, service_id: &super::Token) -> Option<&ServiceObservation> {
        self.services
            .iter()
            .find(|service| &service.deployment.service_id == service_id)
    }
}

fn collect_services(
    drafts: &[ServiceDraft<'_>],
) -> Result<alloc::vec::Vec<ServiceObservation>, super::ContractIssue> {
    if drafts.is_empty() {
        return Err(super::ContractIssue::Empty { field: "services" });
    }
    let count = super::basis::bounded_len("services", drafts.len(), super::MAX_SERVICES)?;
    let mut services = alloc::vec::Vec::with_capacity(drafts.len());
    for draft in drafts {
        let observation = ServiceObservation::new(draft)?;
        let is_repeated = services.iter().any(|seen: &ServiceObservation| {
            seen.deployment.service_id == observation.deployment.service_id
        });
        if is_repeated {
            return Err(super::ContractIssue::Duplicate {
                field: "service_id",
            });
        }
        services.push(observation);
    }
    debug_assert_eq!(u32::try_from(services.len()).ok(), Some(count));
    Ok(services)
}

fn draft_identity(draft: &SnapshotDraft<'_>, services: &[ServiceObservation]) -> super::ContentRef {
    let mut framer = super::basis::Framer::new(super::SNAPSHOT_DOMAIN);
    framer.text(super::SNAPSHOT_SCHEMA);
    framer.text(draft.source_id);
    framer.number(draft.source_generation);
    framer.number(draft.sequence);
    framer.number(draft.observed_tick);
    framer.number(draft.valid_until_tick);
    for service in services {
        frame_deployment(&mut framer, service);
        frame_supply(&mut framer, service);
    }
    framer.finish()
}

fn frame_deployment(framer: &mut super::basis::Framer, service: &ServiceObservation) {
    let deployment = &service.deployment;
    debug_assert!(deployment.deployment_generation > 0);
    debug_assert!(!deployment.service_id.as_str().is_empty());
    framer.text(deployment.service_id.as_str());
    framer.text(deployment.endpoint_identity.as_str());
    framer.text(deployment.deployment_id.as_str());
    framer.number(deployment.deployment_generation);
    framer.text(service.locality.as_str());
    let model = service.model.as_ref();
    framer.optional(model.map(|(binding, _)| binding.model_id()));
    framer.optional(model.map(|(binding, _)| binding.configuration_ref()));
    framer.optional(model.map(|(_, attestation)| attestation.as_str()));
    framer.text(service.readiness.as_str());
    framer.text(service.admission.as_str());
    framer.text(service.replica.label());
    framer.optional(
        service
            .resource_consumer_ref
            .as_ref()
            .map(super::Token::as_str),
    );
}

fn frame_supply(framer: &mut super::basis::Framer, service: &ServiceObservation) {
    for capability in service.capabilities.ordered() {
        framer.text(capability_label(capability));
    }
    match service.envelope {
        None => framer.text("envelope-unknown"),
        Some(envelope) => {
            framer.number(u64::from(envelope.max_context_tokens));
            framer.number(u64::from(envelope.max_output_tokens));
        }
    }
    for observation in &service.capacity {
        frame_capacity(framer, observation.fields());
    }
    framer.optional(service.rollout.as_ref().map(super::RolloutFacts::identity));
}

fn frame_capacity(framer: &mut super::basis::Framer, capacity: &CapacityDraft) {
    framer.text(capacity.kind.as_str());
    framer.text(capacity.unit.as_str());
    framer.text(capacity.scope.as_str());
    framer.text(capacity.origin.as_str());
    match capacity.value {
        Counter::Unknown => framer.text("unknown"),
        Counter::Known(value) => framer.number(value),
    }
    framer.number(capacity.observed_tick);
    framer.number(capacity.valid_until_tick);
}
