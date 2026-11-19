//! Eligibility screening of named model routes before any preference.

const SINGLE_REQUEST: u64 = 1;

/// Host-supplied freshness basis. The core reads no clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Freshness {
    /// Current logical tick in the snapshot source's time domain; `None` is unknown.
    pub now_tick: Option<u64>,
    /// Newest `(source_generation, sequence)` the host already accepted from this source.
    pub newest_accepted: Option<(u64, u64)>,
}

/// Snapshot-level freshness result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotStatus {
    /// The snapshot is in order, unexpired, and within the profile's age bound.
    Current,
    /// The host supplied no current tick, or a tick before the observation.
    FreshnessUnknown,
    /// The snapshot is past its validity window.
    Expired,
    /// The snapshot is older than the profile's age bound.
    TooOld,
    /// The snapshot is older than one the host already accepted.
    OutOfOrder,
}

impl SnapshotStatus {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::FreshnessUnknown => "freshness-unknown",
            Self::Expired => "expired",
            Self::TooOld => "too-old",
            Self::OutOfOrder => "out-of-order",
        }
    }
}

/// Why a route is not eligible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExclusionReason {
    /// The whole snapshot is not current.
    Snapshot(SnapshotStatus),
    /// The snapshot does not list the route's service.
    ServiceAbsent,
    /// The service is not ready.
    NotReady(super::Readiness),
    /// The service's model identity is unknown.
    ModelUnknown,
    /// The service serves another model.
    ModelMismatch,
    /// The service serves the model under another configuration.
    ConfigurationMismatch,
    /// The profile requires an observed identity; the source only claimed it.
    ModelOnlyClaimed,
    /// A local-only request cannot use a remote service.
    PrivacyDenied,
    /// A required capability is unsupported.
    CapabilityUnsupported,
    /// A required capability is unknown.
    CapabilityUnknown,
    /// The service's request envelope is unknown.
    EnvelopeUnknown,
    /// The request exceeds the service's envelope.
    EnvelopeExceeded,
    /// The rollout requirement has no rollout facts.
    RolloutUnavailable,
    /// A rollout fact does not match the requirement.
    RolloutMismatch(super::RolloutField),
    /// The serving owner offers no request admission.
    AdmissionNotOffered,
    /// No fresh known request-gating counter exists.
    CapacityUnknown,
    /// Every known request-gating counter expired.
    CapacityStale,
    /// A fresh known counter cannot fit the request.
    CapacityExhausted,
}

impl ExclusionReason {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Snapshot(status) => status.as_str(),
            Self::ServiceAbsent => "service-absent",
            Self::NotReady(_) => "not-ready",
            Self::ModelUnknown => "model-unknown",
            Self::ModelMismatch => "model-mismatch",
            Self::ConfigurationMismatch => "configuration-mismatch",
            Self::ModelOnlyClaimed => "model-only-claimed",
            Self::PrivacyDenied => "privacy-denied",
            Self::CapabilityUnsupported => "capability-unsupported",
            Self::CapabilityUnknown => "capability-unknown",
            Self::EnvelopeUnknown => "envelope-unknown",
            Self::EnvelopeExceeded => "envelope-exceeded",
            Self::RolloutUnavailable => "rollout-unavailable",
            Self::RolloutMismatch(_) => "rollout-mismatch",
            Self::AdmissionNotOffered => "admission-not-offered",
            Self::CapacityUnknown => "capacity-unknown",
            Self::CapacityStale => "capacity-stale",
            Self::CapacityExhausted => "capacity-exhausted",
        }
    }
}

/// One excluded route and its reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Exclusion {
    route_name: super::Token,
    reason: ExclusionReason,
}

impl Exclusion {
    /// Named route.
    #[must_use]
    pub fn route_name(&self) -> &str {
        self.route_name.as_str()
    }

    /// Reason.
    #[must_use]
    pub const fn reason(&self) -> ExclusionReason {
        self.reason
    }
}

/// Capacity evidence behind an eligible route. It is never a reservation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityStanding {
    /// A fresh known counter fits the request.
    Known {
        /// Counter kind.
        kind: super::CapacityKind,
        /// Counter value.
        value: u64,
    },
    /// Capacity is unknown or expired; the owner's admission decides.
    UnknownOwnerDecides,
}

/// Borrowed named route candidate.
#[derive(Clone, Copy, Debug)]
pub struct RouteCandidateDraft<'a> {
    /// Named route, for example `primary`.
    pub route_name: &'a str,
    /// Caller-owned route identity, for example a `lattice-model-route-v1` reference.
    pub route_ref: Option<&'a str>,
    /// Distinct router catalog label for this route.
    pub label: &'a str,
    /// Router backend label.
    pub backend: &'a str,
    /// Router role label.
    pub role: &'a str,
    /// Logical service the route binds.
    pub service_id: &'a str,
    /// Exact model the route admits.
    pub model_id: &'a str,
    /// Exact configuration the route admits.
    pub configuration_ref: &'a str,
    /// Required capabilities and token bounds.
    pub requirements: crate::mesh::Requirements,
    /// Optional rollout requirement.
    pub rollout: Option<super::RolloutRequirementDraft<'a>>,
}

/// Validated named route candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteCandidate {
    route: super::RouteBinding,
    entry: crate::router::CatalogEntry,
    service_id: super::Token,
    model: super::ModelBinding,
    requirements: crate::mesh::Requirements,
    rollout: Option<super::RolloutRequirement>,
}

impl RouteCandidate {
    /// Validates a route candidate.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractIssue`] for unsafe identities, noncanonical
    /// references, invalid router labels, or an invalid rollout requirement.
    pub fn new(draft: &RouteCandidateDraft<'_>) -> Result<Self, super::ContractIssue> {
        let route_ref = match draft.route_ref {
            None => None,
            Some(value) => Some(super::ContentRef::new("route_ref", value)?),
        };
        let rollout = match draft.rollout {
            None => None,
            Some(rollout) => Some(super::RolloutRequirement::new(&rollout)?),
        };
        Ok(Self {
            route: super::RouteBinding {
                route_name: super::Token::new("route_name", draft.route_name)?,
                route_ref,
            },
            entry: catalog_entry((draft.label, draft.backend, draft.role), false)?,
            service_id: super::Token::new("service_id", draft.service_id)?,
            model: super::ModelBinding::new(draft.model_id, draft.configuration_ref)?,
            requirements: draft.requirements,
            rollout,
        })
    }

    /// Route binding.
    #[must_use]
    pub const fn route(&self) -> &super::RouteBinding {
        &self.route
    }
}

/// Builds a router catalog entry from `(label, backend, role)`.
fn catalog_entry(
    labels: (&str, &str, &str),
    is_local: bool,
) -> Result<crate::router::CatalogEntry, super::ContractIssue> {
    crate::router::CatalogEntry::new(crate::router::CatalogEntryDraft {
        label: labels.0,
        backend: labels.1,
        role: labels.2,
        local: is_local,
    })
    .map_err(|_| super::ContractIssue::UnsafeToken {
        field: "router_catalog_entry",
    })
}

/// Borrowed screening request.
#[derive(Clone, Copy, Debug)]
pub struct ScreenRequest<'a> {
    /// Explicitly enabled capacity-aware profile.
    pub profile: &'a super::Profile,
    /// Validated snapshot.
    pub snapshot: &'a super::AvailabilitySnapshot,
    /// Host-supplied freshness basis.
    pub freshness: Freshness,
    /// Request privacy band.
    pub privacy: crate::router::PrivacyBand,
    /// Named route candidates.
    pub candidates: &'a [RouteCandidate],
}

/// One eligible route with everything dispatch must later revalidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EligibleRoute {
    pub(super) route: super::RouteBinding,
    pub(super) entry: crate::router::CatalogEntry,
    pub(super) deployment: super::DeploymentBinding,
    pub(super) model: super::ModelBinding,
    pub(super) attestation: super::Attestation,
    pub(super) replica: super::ReplicaFact,
    pub(super) standing: CapacityStanding,
    pub(super) requirements: crate::mesh::Requirements,
    pub(super) rollout: Option<super::RolloutRequirement>,
    pub(super) resource_consumer_ref: Option<super::Token>,
}

impl EligibleRoute {
    /// Route binding.
    #[must_use]
    pub const fn route(&self) -> &super::RouteBinding {
        &self.route
    }

    /// Router catalog entry whose locality reflects the observed service.
    #[must_use]
    pub const fn catalog_entry(&self) -> &crate::router::CatalogEntry {
        &self.entry
    }

    /// Deployment selected at screening time.
    #[must_use]
    pub const fn deployment(&self) -> &super::DeploymentBinding {
        &self.deployment
    }

    /// Exact model binding.
    #[must_use]
    pub const fn model(&self) -> &super::ModelBinding {
        &self.model
    }

    /// How the model identity was established.
    #[must_use]
    pub const fn attestation(&self) -> super::Attestation {
        self.attestation
    }

    /// Capacity evidence; never a reservation.
    #[must_use]
    pub const fn capacity(&self) -> CapacityStanding {
        self.standing
    }

    /// Replica placement observed at screening time.
    #[must_use]
    pub const fn replica(&self) -> &super::ReplicaFact {
        &self.replica
    }

    /// Required capabilities and token bounds.
    #[must_use]
    pub const fn requirements(&self) -> &crate::mesh::Requirements {
        &self.requirements
    }

    /// Opaque compute consumer reference of the serving deployment.
    #[must_use]
    pub fn resource_consumer_ref(&self) -> Option<&str> {
        self.resource_consumer_ref
            .as_ref()
            .map(super::Token::as_str)
    }
}

/// Screening result: the eligible set and every exclusion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Screening {
    pub(super) profile_identity: super::ContentRef,
    pub(super) snapshot_identity: super::ContentRef,
    pub(super) snapshot_position: (u64, u64),
    pub(super) status: SnapshotStatus,
    pub(super) eligible: alloc::vec::Vec<EligibleRoute>,
    pub(super) exclusions: alloc::vec::Vec<Exclusion>,
}

impl Screening {
    /// Snapshot freshness.
    #[must_use]
    pub const fn status(&self) -> SnapshotStatus {
        self.status
    }

    /// Profile identity.
    #[must_use]
    pub fn profile_identity(&self) -> &str {
        self.profile_identity.as_str()
    }

    /// Snapshot identity.
    #[must_use]
    pub fn snapshot_identity(&self) -> &str {
        self.snapshot_identity.as_str()
    }

    /// Eligible routes in candidate order.
    #[must_use]
    pub fn eligible(&self) -> &[EligibleRoute] {
        self.eligible.as_slice()
    }

    /// Excluded routes in candidate order.
    #[must_use]
    pub fn exclusions(&self) -> &[Exclusion] {
        self.exclusions.as_slice()
    }

    /// Router catalog of eligible routes only, for existing recipe selection and rotation.
    #[must_use]
    pub fn eligible_catalog(&self) -> alloc::vec::Vec<crate::router::CatalogEntry> {
        self.eligible
            .iter()
            .map(|route| route.entry.clone())
            .collect()
    }

    /// Eligible route for a selected catalog label.
    #[must_use]
    pub fn route_for_label(&self, label: &str) -> Option<&EligibleRoute> {
        self.eligible
            .iter()
            .find(|route| route.entry.label() == label)
    }
}

/// Screens named route candidates against one snapshot under one profile.
///
/// # Errors
///
/// Returns [`super::ContractIssue`] for an empty, oversized, or duplicate candidate list.
pub fn screen_routes(request: &ScreenRequest<'_>) -> Result<Screening, super::ContractIssue> {
    validate_candidates(request.candidates)?;
    let current = snapshot_status(request);
    let mut eligible = alloc::vec::Vec::with_capacity(request.candidates.len());
    let mut exclusions = alloc::vec::Vec::with_capacity(request.candidates.len());
    for candidate in request.candidates {
        match screen_candidate(request, current, candidate) {
            Ok(route) => eligible.push(route),
            Err(reason) => exclusions.push(Exclusion {
                route_name: candidate.route.route_name.clone(),
                reason,
            }),
        }
    }
    debug_assert!(eligible.len() <= request.candidates.len());
    debug_assert!(exclusions.len() <= request.candidates.len());
    Ok(Screening {
        profile_identity: super::ContentRef::new("profile_identity", request.profile.identity())?,
        snapshot_identity: super::ContentRef::new(
            "snapshot_identity",
            request.snapshot.identity(),
        )?,
        snapshot_position: (
            request.snapshot.source_generation,
            request.snapshot.sequence,
        ),
        status: current.map_or_else(|status| status, |_| SnapshotStatus::Current),
        eligible,
        exclusions,
    })
}

/// Applies a preference order (for example a learned ranking) to the eligible
/// set only. Labels that were excluded, unknown, or unlisted are skipped; a
/// preference never makes an ineligible route admissible.
#[must_use]
pub fn rank_eligible<'a>(
    screening: &'a Screening,
    preference: &[&str],
) -> Option<&'a EligibleRoute> {
    preference
        .iter()
        .find_map(|label| screening.route_for_label(label))
}

fn validate_candidates(candidates: &[RouteCandidate]) -> Result<(), super::ContractIssue> {
    if candidates.is_empty() {
        return Err(super::ContractIssue::Empty {
            field: "candidates",
        });
    }
    let count =
        super::basis::bounded_len("candidates", candidates.len(), super::MAX_ROUTE_CANDIDATES)?;
    debug_assert!(count > 0);
    debug_assert!(count <= super::MAX_ROUTE_CANDIDATES);
    for (index, candidate) in candidates.iter().enumerate() {
        let earlier = candidates.get(..index).unwrap_or_default();
        let is_repeated_name = earlier
            .iter()
            .any(|seen| seen.route.route_name == candidate.route.route_name);
        if is_repeated_name {
            return Err(super::ContractIssue::Duplicate {
                field: "route_name",
            });
        }
        let is_repeated_label = earlier
            .iter()
            .any(|seen| seen.entry.label() == candidate.entry.label());
        if is_repeated_label {
            return Err(super::ContractIssue::Duplicate {
                field: "router_model_label",
            });
        }
    }
    Ok(())
}

/// Returns the current tick when the snapshot is current.
fn snapshot_status(request: &ScreenRequest<'_>) -> Result<u64, SnapshotStatus> {
    let snapshot = request.snapshot;
    let position = (snapshot.source_generation, snapshot.sequence);
    let is_older = request
        .freshness
        .newest_accepted
        .is_some_and(|newest| position < newest);
    if is_older {
        return Err(SnapshotStatus::OutOfOrder);
    }
    let now_tick = request
        .freshness
        .now_tick
        .ok_or(SnapshotStatus::FreshnessUnknown)?;
    let age_ticks = now_tick
        .checked_sub(snapshot.observed_tick)
        .ok_or(SnapshotStatus::FreshnessUnknown)?;
    debug_assert!(age_ticks <= now_tick);
    if now_tick >= snapshot.valid_until_tick {
        return Err(SnapshotStatus::Expired);
    }
    if age_ticks > request.profile.max_snapshot_age_ticks() {
        return Err(SnapshotStatus::TooOld);
    }
    debug_assert!(snapshot.observed_tick <= now_tick);
    Ok(now_tick)
}

fn screen_candidate(
    request: &ScreenRequest<'_>,
    current: Result<u64, SnapshotStatus>,
    candidate: &RouteCandidate,
) -> Result<EligibleRoute, ExclusionReason> {
    let now_tick = current.map_err(ExclusionReason::Snapshot)?;
    let service = request
        .snapshot
        .service(&candidate.service_id)
        .ok_or(ExclusionReason::ServiceAbsent)?;
    if service.readiness != super::Readiness::Ready {
        return Err(ExclusionReason::NotReady(service.readiness));
    }
    let attestation = check_model(request.profile, service, &candidate.model)?;
    check_privacy(request.privacy, service.locality)?;
    check_capabilities(service, &candidate.requirements)?;
    check_rollout(service, candidate)?;
    if service.admission != super::AdmissionOffer::OwnerAdmission {
        return Err(ExclusionReason::AdmissionNotOffered);
    }
    let standing = capacity_standing(
        request.profile.capacity_rule(),
        service,
        now_tick,
        &candidate.requirements,
    )?;
    debug_assert_eq!(service.deployment.service_id, candidate.service_id);
    Ok(eligible_route(candidate, service, attestation, standing))
}

fn eligible_route(
    candidate: &RouteCandidate,
    service: &super::ServiceObservation,
    attestation: super::Attestation,
    standing: CapacityStanding,
) -> EligibleRoute {
    let is_local = service.locality == super::Locality::Local;
    let entry = catalog_entry(
        (
            candidate.entry.label(),
            candidate.entry.backend(),
            candidate.entry.role().as_str(),
        ),
        is_local,
    )
    .unwrap_or_else(|_| candidate.entry.clone());
    debug_assert_eq!(entry.label(), candidate.entry.label());
    EligibleRoute {
        route: candidate.route.clone(),
        entry,
        deployment: service.deployment.clone(),
        model: candidate.model.clone(),
        attestation,
        replica: service.replica.clone(),
        standing,
        requirements: candidate.requirements,
        rollout: candidate.rollout.clone(),
        resource_consumer_ref: service.resource_consumer_ref.clone(),
    }
}

fn check_model(
    profile: &super::Profile,
    service: &super::ServiceObservation,
    expected: &super::ModelBinding,
) -> Result<super::Attestation, ExclusionReason> {
    let (model, attestation) = service
        .model
        .as_ref()
        .ok_or(ExclusionReason::ModelUnknown)?;
    if model.model_id() != expected.model_id() {
        return Err(ExclusionReason::ModelMismatch);
    }
    if model.configuration_ref() != expected.configuration_ref() {
        return Err(ExclusionReason::ConfigurationMismatch);
    }
    let is_observed_required = profile.identity_rule() == super::IdentityRule::ObservedRequired;
    if is_observed_required && *attestation != super::Attestation::Observed {
        return Err(ExclusionReason::ModelOnlyClaimed);
    }
    Ok(*attestation)
}

fn check_privacy(
    privacy: crate::router::PrivacyBand,
    locality: super::Locality,
) -> Result<(), ExclusionReason> {
    match (privacy, locality) {
        (
            crate::router::PrivacyBand::LocalOnly,
            super::Locality::PrivateRemote | super::Locality::HostedRemote,
        ) => Err(ExclusionReason::PrivacyDenied),
        (crate::router::PrivacyBand::LocalOnly, super::Locality::Local)
        | (crate::router::PrivacyBand::RemoteAllowed, _) => Ok(()),
    }
}

fn check_capabilities(
    service: &super::ServiceObservation,
    requirements: &crate::mesh::Requirements,
) -> Result<(), ExclusionReason> {
    let required = [
        true,
        requirements.tools,
        requirements.streaming,
        requirements.structured_output,
        requirements.developer_role,
    ];
    debug_assert_eq!(required.len(), service.capabilities.ordered().len());
    for (is_needed, support) in required.iter().zip(service.capabilities.ordered()) {
        match (is_needed, support) {
            (true, crate::mesh::Capability::Unsupported) => {
                return Err(ExclusionReason::CapabilityUnsupported);
            }
            (true, crate::mesh::Capability::Unknown) => {
                return Err(ExclusionReason::CapabilityUnknown);
            }
            (true, crate::mesh::Capability::Supported) | (false, _) => {}
        }
    }
    let envelope = service.envelope.ok_or(ExclusionReason::EnvelopeUnknown)?;
    if envelope.admits((requirements.input_tokens, requirements.output_tokens)) {
        debug_assert!(requirements.output_tokens <= envelope.max_output_tokens());
        return Ok(());
    }
    Err(ExclusionReason::EnvelopeExceeded)
}

fn check_rollout(
    service: &super::ServiceObservation,
    candidate: &RouteCandidate,
) -> Result<(), ExclusionReason> {
    let Some(requirement) = candidate.rollout.as_ref() else {
        return Ok(());
    };
    let tokens = (
        candidate.requirements.input_tokens,
        candidate.requirements.output_tokens,
    );
    debug_assert!(requirement.policy_generation() > 0);
    debug_assert!(tokens.1 <= tokens.0.saturating_add(tokens.1));
    requirement
        .check(
            service.rollout.as_ref(),
            (candidate.model.configuration_ref(), tokens),
        )
        .map_err(|field| {
            field.map_or(
                ExclusionReason::RolloutUnavailable,
                ExclusionReason::RolloutMismatch,
            )
        })
}

enum CapacityEvidence {
    Fits(super::CapacityKind, u64),
    Exhausted,
    Stale,
    Unknown,
}

/// The host's current tick and the request's total tokens.
#[derive(Clone, Copy)]
struct Demand {
    now_tick: u64,
    request_tokens: u64,
}

fn capacity_standing(
    rule: super::CapacityRule,
    service: &super::ServiceObservation,
    now_tick: u64,
    requirements: &crate::mesh::Requirements,
) -> Result<CapacityStanding, ExclusionReason> {
    let demand = Demand {
        now_tick,
        request_tokens: u64::from(
            requirements
                .input_tokens
                .saturating_add(requirements.output_tokens),
        ),
    };
    debug_assert!(demand.request_tokens >= u64::from(requirements.output_tokens));
    debug_assert!(is_within_capacity_bound(service));
    match (rule, capacity_evidence(service, demand)) {
        (_, CapacityEvidence::Fits(kind, value)) => Ok(CapacityStanding::Known { kind, value }),
        (_, CapacityEvidence::Exhausted) => Err(ExclusionReason::CapacityExhausted),
        (super::CapacityRule::KnownRequired, CapacityEvidence::Stale) => {
            Err(ExclusionReason::CapacityStale)
        }
        (super::CapacityRule::KnownRequired, CapacityEvidence::Unknown) => {
            Err(ExclusionReason::CapacityUnknown)
        }
        (
            super::CapacityRule::OwnerAdmissionDecides,
            CapacityEvidence::Stale | CapacityEvidence::Unknown,
        ) => Ok(CapacityStanding::UnknownOwnerDecides),
    }
}

/// Any fresh trusted counter that cannot fit the request is exhaustion; otherwise
/// the first fresh trusted counter that fits is the evidence.
fn capacity_evidence(service: &super::ServiceObservation, demand: Demand) -> CapacityEvidence {
    debug_assert!(is_within_capacity_bound(service));
    let mut fits = None;
    for observation in &service.capacity {
        let super::Counter::Known(value) = observation.fields().value else {
            continue;
        };
        if !observation.is_fresh_gating(demand.now_tick) {
            continue;
        }
        let kind = observation.fields().kind;
        if value < needed_units(kind, demand.request_tokens) {
            return CapacityEvidence::Exhausted;
        }
        fits = fits.or(Some((kind, value)));
    }
    let has_known_trusted = service.capacity.iter().any(|observation| {
        observation.is_trusted_gating()
            && matches!(observation.fields().value, super::Counter::Known(_))
    });
    debug_assert!(fits.is_none() || has_known_trusted);
    match (fits, has_known_trusted) {
        (Some((kind, value)), _) => CapacityEvidence::Fits(kind, value),
        (None, true) => CapacityEvidence::Stale,
        (None, false) => CapacityEvidence::Unknown,
    }
}

fn is_within_capacity_bound(service: &super::ServiceObservation) -> bool {
    u32::try_from(service.capacity.len())
        .is_ok_and(|count| count <= super::MAX_CAPACITY_OBSERVATIONS)
}

const fn needed_units(kind: super::CapacityKind, request_tokens: u64) -> u64 {
    match kind {
        super::CapacityKind::KvCacheTokens | super::CapacityKind::TokenQuota => request_tokens,
        super::CapacityKind::RequestSlots
        | super::CapacityKind::RequestQuota
        | super::CapacityKind::QueueDepth
        | super::CapacityKind::FreeMemory => SINGLE_REQUEST,
    }
}
