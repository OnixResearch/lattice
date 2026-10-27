//! Pure composition-activation graph validation, preflight, and safe inspection.
//!
//! The module consumes explicit graph metadata and host-supplied readiness facts.
//! It performs no discovery, startup, provider, process, filesystem, network,
//! credential, persistence, terminal, or daemon effect.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::too_many_parameters,
        tigerstyle::usize_in_public_api,
        reason = "published activation compatibility contracts retain their typed stable signatures and collection limits"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::compound_condition,
        tigerstyle::function_length,
        reason = "activation preflight returns bounded issue lists for invalid graphs instead of panicking assertions"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::borrowed_argument_types,
        reason = "issue collectors require mutable Vec ownership and capacity while accumulating bounded diagnostics"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        reason = "the no_std activation component uses explicit collection and hash imports to keep dependencies visible"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::bool_naming,
        tigerstyle::numeric_units,
        reason = "published activation vocabulary uses protocol field names fixed by its versioned contract"
    )
)]

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use blake3::Hasher;

const GRAPH_DOMAIN: &str = "animus/composition-activation-graph/v1";
const SNAPSHOT_DOMAIN: &str = "animus/composition-activation-snapshot/v1";
const SNAPSHOT_COMPONENTS_DOMAIN: &str = "animus/composition-activation-components/v1";
const SNAPSHOT_ISSUES_DOMAIN: &str = "animus/composition-activation-issues/v1";
const GRAPH_SCHEMA: &str = "animus.composition-activation-graph.v1";
const SNAPSHOT_SCHEMA: &str = "animus.composition-activation-snapshot.v1";
const HASH_SEPARATOR: u8 = 0;
const MINIMUM_GENERATION: u64 = 1;
const MAX_COMPONENT_COUNT: usize = 512;
const MAX_PORT_COUNT: usize = 512;
const MAX_EVENT_COUNT: usize = 1_024;
const MAX_SELECTION_COUNT: usize = 512;
const MAX_OBSERVATION_COUNT: usize = 512;
const MAX_NON_CLAIM_COUNT: usize = 16;
const MAX_LABEL_BYTES: usize = 256;
const MAX_ISSUE_COUNT: usize = 1_024;
const REQUIRED_NON_CLAIMS: [&str; 6] = [
    "not-implementation-correctness",
    "not-runtime-liveness",
    "not-effect-success",
    "not-authority-grant",
    "not-production-readiness",
    "not-release-eligibility",
];
const FORBIDDEN_EVIDENCE_MARKERS: [&str; 11] = [
    "/home/",
    "api_key",
    "credential",
    "endpoint=",
    "private path",
    "prompt=",
    "provider payload",
    "secret",
    "sk-",
    "stdout:",
    "stderr:",
];

/// Required or optional activation posture.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationPosture {
    /// Absence or unavailability blocks activation.
    Required,
    /// Absence or unavailability can produce explicit degradation.
    Optional,
}

impl ActivationPosture {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
        }
    }
}

/// Provider multiplicity for one application-owned port.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PortMultiplicity {
    /// Exactly one provider must be selected.
    ExactlyOne,
    /// Zero or one provider can be selected.
    ZeroOrOne,
    /// One or more providers must be selected.
    OneOrMore,
}

impl PortMultiplicity {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ExactlyOne => "exactly_one",
            Self::ZeroOrOne => "zero_or_one",
            Self::OneOrMore => "one_or_more",
        }
    }

    const fn admits(self, count: usize) -> bool {
        match self {
            Self::ExactlyOne => count == 1,
            Self::ZeroOrOne => count <= 1,
            Self::OneOrMore => count >= 1,
        }
    }
}

/// Closed authority classes used by activation metadata.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationAuthorityClass {
    /// No host authority is required.
    None,
    /// A caller injects an implementation.
    InjectedOnly,
    /// The selected shell owns the effect capability.
    ShellOwned,
}

impl ActivationAuthorityClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::InjectedOnly => "injected_only",
            Self::ShellOwned => "shell_owned",
        }
    }

    const fn admits(self, required: Self) -> bool {
        match (self, required) {
            (_, Self::None)
            | (Self::InjectedOnly, Self::InjectedOnly)
            | (Self::ShellOwned, Self::InjectedOnly | Self::ShellOwned) => true,
            (Self::None, Self::InjectedOnly | Self::ShellOwned)
            | (Self::InjectedOnly, Self::ShellOwned) => false,
        }
    }
}

/// Bounded event delivery class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationDeliveryClass {
    /// One producer order is preserved.
    Ordered,
    /// A complete snapshot precedes incremental events.
    SnapshotFirst,
    /// Handlers run in a declared serial order.
    Serial,
    /// Independent handlers can run without an order claim.
    Parallel,
    /// Each handler can narrow one candidate in order.
    Waterfall,
}

impl ActivationDeliveryClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ordered => "ordered",
            Self::SnapshotFirst => "snapshot_first",
            Self::Serial => "serial",
            Self::Parallel => "parallel",
            Self::Waterfall => "waterfall",
        }
    }
}

/// Event durability class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationDurabilityClass {
    /// The existing durable owner records the event.
    Durable,
    /// The event exists only for the current live generation.
    LiveOnly,
    /// Only bounded metadata can persist.
    MetadataOnly,
}

impl ActivationDurabilityClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Durable => "durable",
            Self::LiveOnly => "live_only",
            Self::MetadataOnly => "metadata_only",
        }
    }
}

/// One component in the composition graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationComponent {
    /// Stable component identity.
    pub component_id: String,
    /// Required feature label or `always`.
    pub feature_gate: String,
    /// Maximum authority needed by this component.
    pub authority: ActivationAuthorityClass,
    /// Activation posture in the selected profile.
    pub posture: ActivationPosture,
}

/// One application-owned capability seam.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationPort {
    /// Stable port identity.
    pub port_id: String,
    /// Component that owns the application contract.
    pub owner_component_id: String,
    /// Declared provider component identities.
    pub provider_component_ids: Vec<String>,
    /// Declared direct consumer component identities.
    pub consumer_component_ids: Vec<String>,
    /// Safe scope class.
    pub scope_class: String,
    /// Provider multiplicity.
    pub multiplicity: PortMultiplicity,
    /// Component that owns teardown for this seam.
    pub lifecycle_owner_component_id: String,
    /// Activation posture for this seam.
    pub posture: ActivationPosture,
    /// Feature label that admits the seam.
    pub feature_gate: String,
    /// Authority needed by a selected provider.
    pub authority: ActivationAuthorityClass,
    /// Focused verification rail label.
    pub focused_check: String,
}

/// One declared event route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationEventRoute {
    /// Stable event identity.
    pub event_id: String,
    /// Declared producer component.
    pub producer_component_id: String,
    /// Declared consumer components.
    pub consumer_component_ids: Vec<String>,
    /// Delivery class.
    pub delivery: ActivationDeliveryClass,
    /// Durability class.
    pub durability: ActivationDurabilityClass,
    /// Component that owns route teardown.
    pub lifecycle_owner_component_id: String,
    /// Route activation posture.
    pub posture: ActivationPosture,
    /// True only when every edge in a supported event cycle is bounded.
    pub bounded_cycle: bool,
}

/// Draft graph from one checked catalog projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationGraphDraft {
    /// Components available to selected composition roots.
    pub components: Vec<ActivationComponent>,
    /// Application-owned port seams.
    pub ports: Vec<ActivationPort>,
    /// Event ownership routes.
    pub events: Vec<ActivationEventRoute>,
    /// Mandatory claim boundaries.
    pub non_claims: Vec<String>,
}

/// Revalidated composition graph with deterministic identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedActivationGraph {
    components: Vec<ActivationComponent>,
    ports: Vec<ActivationPort>,
    events: Vec<ActivationEventRoute>,
    non_claims: Vec<String>,
    identity: String,
}

impl ValidatedActivationGraph {
    /// Validates references, multiplicity, cycles, evidence labels, and bounds.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the graph is malformed or incomplete.
    pub fn validate(draft: ActivationGraphDraft) -> Result<Self, ActivationGraphError> {
        validate_graph_bounds(&draft)?;
        validate_required_non_claims(&draft.non_claims)?;
        let component_index = validate_components(&draft.components)?;
        validate_ports(&draft.ports, &component_index)?;
        validate_events(&draft.events, &component_index)?;
        validate_event_cycles(&draft.events)?;
        let identity = graph_identity(&draft);
        Ok(Self {
            components: draft.components,
            ports: draft.ports,
            events: draft.events,
            non_claims: draft.non_claims,
            identity,
        })
    }

    /// Returns the canonical BLAKE3 graph identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the validated components.
    #[must_use]
    pub fn components(&self) -> &[ActivationComponent] {
        self.components.as_slice()
    }

    /// Returns the validated port seams.
    #[must_use]
    pub fn ports(&self) -> &[ActivationPort] {
        self.ports.as_slice()
    }

    /// Returns the validated event routes.
    #[must_use]
    pub fn events(&self) -> &[ActivationEventRoute] {
        self.events.as_slice()
    }
}

/// One selected composition and its authority ceiling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedComposition {
    /// Current activation generation.
    pub generation: u64,
    /// Selected component identities.
    pub component_ids: Vec<String>,
    /// Explicit enabled feature labels.
    pub enabled_features: Vec<String>,
    /// Maximum authority admitted for this composition.
    pub authority_ceiling: ActivationAuthorityClass,
}

struct PreflightIndexes<'graph, 'selection, 'observation> {
    selected: BTreeSet<&'selection str>,
    enabled_features: BTreeSet<&'selection str>,
    observations: BTreeMap<&'observation str, &'observation ActivationReadinessObservation>,
    components: BTreeMap<&'graph str, &'graph ActivationComponent>,
}

/// Current readiness state supplied by the host.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationReadinessStatus {
    /// The component reports current readiness.
    Ready,
    /// The component reports current unavailability.
    Unavailable,
}

/// Safe readiness class for one graph component.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationComponentStatusKind {
    /// The composition did not select this component.
    NotSelected,
    /// The selected component has no readiness observation.
    MissingReadiness,
    /// The selected component observation belongs to another generation.
    StaleReadiness,
    /// The selected component is current and ready.
    Ready,
    /// The selected component is current and unavailable.
    Unavailable,
}

impl ActivationComponentStatusKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NotSelected => "not_selected",
            Self::MissingReadiness => "missing_readiness",
            Self::StaleReadiness => "stale_readiness",
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Metadata-only status for one graph component.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActivationComponentStatus {
    /// Stable component identity.
    pub component_id: String,
    /// Required feature label or `always`.
    pub feature_gate: String,
    /// Maximum authority needed by this component.
    pub authority: ActivationAuthorityClass,
    /// Required or optional activation posture.
    pub posture: ActivationPosture,
    /// Safe current selection and readiness class.
    pub status: ActivationComponentStatusKind,
}

/// Host-supplied component readiness observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationReadinessObservation {
    /// Component identity.
    pub component_id: String,
    /// Activation generation observed by the host.
    pub generation: u64,
    /// Safe observation identity.
    pub observation_id: String,
    /// Current readiness status.
    pub status: ActivationReadinessStatus,
}

/// Safe activation issue class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationIssueKind {
    /// A required graph component was not selected.
    MissingRequiredComponent,
    /// The selected component is unknown to the graph.
    UnknownSelectedComponent,
    /// The selected feature is absent.
    FeatureMismatch,
    /// The composition authority ceiling is too narrow.
    AuthorityMismatch,
    /// A required readiness observation is absent.
    MissingReadiness,
    /// An optional readiness observation is absent.
    OptionalReadinessMissing,
    /// A readiness observation belongs to another generation.
    StaleReadiness,
    /// A required component is unavailable.
    RequiredUnavailable,
    /// An optional component is unavailable.
    OptionalUnavailable,
    /// A required port has no selected provider.
    MissingProvider,
    /// An optional port has no selected provider.
    OptionalProviderUnavailable,
    /// Selected providers violate declared multiplicity.
    InvalidMultiplicity,
    /// A selected seam or route lacks its lifecycle owner.
    MissingLifecycleOwner,
    /// A required producer has no selected event consumer.
    EventRouteGap,
    /// An optional event route has no selected consumer.
    OptionalEventRouteGap,
}

impl ActivationIssueKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::MissingRequiredComponent => "missing_required_component",
            Self::UnknownSelectedComponent => "unknown_selected_component",
            Self::FeatureMismatch => "feature_mismatch",
            Self::AuthorityMismatch => "authority_mismatch",
            Self::MissingReadiness => "missing_readiness",
            Self::OptionalReadinessMissing => "optional_readiness_missing",
            Self::StaleReadiness => "stale_readiness",
            Self::RequiredUnavailable => "required_unavailable",
            Self::OptionalUnavailable => "optional_unavailable",
            Self::MissingProvider => "missing_provider",
            Self::OptionalProviderUnavailable => "optional_provider_unavailable",
            Self::InvalidMultiplicity => "invalid_multiplicity",
            Self::MissingLifecycleOwner => "missing_lifecycle_owner",
            Self::EventRouteGap => "event_route_gap",
            Self::OptionalEventRouteGap => "optional_event_route_gap",
        }
    }

    const fn is_blocking(self) -> bool {
        !matches!(
            self,
            Self::OptionalReadinessMissing
                | Self::OptionalUnavailable
                | Self::OptionalProviderUnavailable
                | Self::OptionalEventRouteGap
        )
    }
}

/// Metadata-only activation issue.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ActivationIssue {
    /// Safe issue class.
    pub kind: ActivationIssueKind,
    /// Safe component, port, or event identity.
    pub subject: String,
}

/// Activation decision status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationDecisionStatus {
    /// All required components and routes are current.
    Ready,
    /// Required closure is ready and optional capability is unavailable.
    DegradedOptional,
    /// One or more required checks failed.
    Blocked,
}

impl ActivationDecisionStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::DegradedOptional => "degraded_optional",
            Self::Blocked => "blocked",
        }
    }
}

/// Pure activation preflight decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationDecision {
    /// Canonical graph identity.
    pub graph_identity: String,
    /// Selected activation generation.
    pub generation: u64,
    /// Decision status.
    pub status: ActivationDecisionStatus,
    /// Safe deterministic component statuses.
    pub components: Vec<ActivationComponentStatus>,
    /// Safe deterministic issues.
    pub issues: Vec<ActivationIssue>,
    /// Number of selected components.
    pub selected_component_count: u32,
    /// Number of current ready observations.
    pub ready_component_count: u32,
}

impl ActivationDecision {
    /// Returns true only for `ready` and `degraded_optional` decisions.
    #[must_use]
    pub const fn effects_admitted(&self) -> bool {
        matches!(
            self.status,
            ActivationDecisionStatus::Ready | ActivationDecisionStatus::DegradedOptional
        )
    }
}

/// Runs deterministic activation preflight over explicit values.
///
/// # Errors
///
/// Returns a typed error for malformed selection or observation sets.
pub fn preflight_composition_activation(
    graph: &ValidatedActivationGraph,
    selection: &SelectedComposition,
    observations: &[ActivationReadinessObservation],
) -> Result<ActivationDecision, ActivationGraphError> {
    validate_preflight_bounds(selection, observations)?;
    let indexes = build_preflight_indexes(graph, selection, observations)?;
    let mut issues = Vec::new();
    let mut component_statuses =
        evaluate_component_statuses(graph, selection, &indexes, &mut issues)?;
    validate_selected_ports(graph, selection, &indexes, &mut issues)?;
    validate_selected_events(graph, &indexes.selected, &mut issues)?;
    component_statuses.sort();
    issues.sort();
    let status = decision_status(issues.as_slice());
    let selected_component_count = count_u32(indexes.selected.len())?;
    let ready_component_count = count_u32(
        observations
            .iter()
            .filter(|observation| {
                indexes.selected.contains(observation.component_id.as_str())
                    && observation.generation == selection.generation
                    && observation.status == ActivationReadinessStatus::Ready
            })
            .count(),
    )?;
    Ok(ActivationDecision {
        graph_identity: graph.identity.clone(),
        generation: selection.generation,
        status,
        components: component_statuses,
        issues,
        selected_component_count,
        ready_component_count,
    })
}

/// Bounded read-only activation snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationSnapshot {
    /// Snapshot schema.
    pub schema: &'static str,
    /// Canonical snapshot identity.
    pub snapshot_identity: String,
    /// Canonical graph identity.
    pub graph_identity: String,
    /// Activation generation.
    pub generation: u64,
    /// Exact decision status.
    pub status: ActivationDecisionStatus,
    /// Safe component identities, classes, and readiness statuses.
    pub components: Vec<ActivationComponentStatus>,
    /// Safe issue classes and subjects.
    pub issues: Vec<ActivationIssue>,
    /// Number of selected components.
    pub selected_component_count: u32,
    /// Number of current ready components.
    pub ready_component_count: u32,
    /// Mandatory claim boundaries.
    pub non_claims: Vec<String>,
}

/// Projects a metadata-only read snapshot from one decision.
///
/// # Errors
///
/// Returns a typed error when the decision is not bound to this graph.
pub fn project_activation_snapshot(
    graph: &ValidatedActivationGraph,
    decision: &ActivationDecision,
) -> Result<ActivationSnapshot, ActivationGraphError> {
    validate_decision_for_snapshot(graph, decision)?;
    let component_identity = component_statuses_identity(decision.components.as_slice());
    let issue_identity = issues_identity(decision.issues.as_slice());
    let generation_text = decision.generation.to_string();
    let selected_text = decision.selected_component_count.to_string();
    let ready_text = decision.ready_component_count.to_string();
    let snapshot_identity = hash_parts(
        SNAPSHOT_DOMAIN,
        &[
            SNAPSHOT_SCHEMA,
            graph.identity.as_str(),
            generation_text.as_str(),
            decision.status.as_str(),
            component_identity.as_str(),
            issue_identity.as_str(),
            selected_text.as_str(),
            ready_text.as_str(),
        ],
    );
    Ok(ActivationSnapshot {
        schema: SNAPSHOT_SCHEMA,
        snapshot_identity,
        graph_identity: graph.identity.clone(),
        generation: decision.generation,
        status: decision.status,
        components: decision.components.clone(),
        issues: decision.issues.clone(),
        selected_component_count: decision.selected_component_count,
        ready_component_count: decision.ready_component_count,
        non_claims: graph.non_claims.clone(),
    })
}

/// Rejects private material and unsupported claims in rendered evidence.
///
/// # Errors
///
/// Returns [`ActivationGraphError::UnsafeEvidence`] for unsafe text.
pub fn validate_activation_evidence_text(value: &str) -> Result<(), ActivationGraphError> {
    let normalized = value.to_ascii_lowercase();
    if FORBIDDEN_EVIDENCE_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        || normalized.contains("proves readiness")
        || normalized.contains("guarantees delivery")
    {
        return Err(ActivationGraphError::UnsafeEvidence);
    }
    Ok(())
}

/// Graph or preflight construction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationGraphError {
    /// A collection exceeded its declared bound.
    BoundExceeded,
    /// A stable label was empty, oversized, unsafe, or duplicated.
    InvalidLabel,
    /// A graph reference targets an unknown component.
    UnknownComponentReference,
    /// A required port has no provider or consumer declaration.
    OrphanRequiredPort,
    /// Provider declarations violate the port multiplicity.
    InvalidPortMultiplicity,
    /// A provider authority class cannot satisfy its port contract.
    PortAuthorityMismatch,
    /// A required event has no consumer declaration.
    MissingEventConsumer,
    /// An event cycle contains an unbounded edge.
    UnsupportedEventCycle,
    /// Mandatory non-claims are absent or reordered.
    InvalidNonClaims,
    /// A readiness observation is duplicated or malformed.
    InvalidObservation,
    /// A decision is malformed or bound to another graph.
    InvalidDecision,
    /// A public count does not fit its bounded representation.
    CountOverflow,
    /// Rendered evidence contains private material or an overclaim.
    UnsafeEvidence,
}

fn validate_graph_bounds(draft: &ActivationGraphDraft) -> Result<(), ActivationGraphError> {
    if draft.components.is_empty()
        || draft.components.len() > MAX_COMPONENT_COUNT
        || draft.ports.len() > MAX_PORT_COUNT
        || draft.events.len() > MAX_EVENT_COUNT
        || draft.non_claims.len() > MAX_NON_CLAIM_COUNT
    {
        return Err(ActivationGraphError::BoundExceeded);
    }
    Ok(())
}

fn validate_preflight_bounds(
    selection: &SelectedComposition,
    observations: &[ActivationReadinessObservation],
) -> Result<(), ActivationGraphError> {
    if selection.generation < MINIMUM_GENERATION
        || selection.component_ids.is_empty()
        || selection.component_ids.len() > MAX_SELECTION_COUNT
        || observations.len() > MAX_OBSERVATION_COUNT
    {
        return Err(ActivationGraphError::BoundExceeded);
    }
    Ok(())
}

fn validate_required_non_claims(values: &[String]) -> Result<(), ActivationGraphError> {
    if values.len() != REQUIRED_NON_CLAIMS.len()
        || values
            .iter()
            .map(String::as_str)
            .ne(REQUIRED_NON_CLAIMS.iter().copied())
    {
        return Err(ActivationGraphError::InvalidNonClaims);
    }
    Ok(())
}

fn validate_components(
    components: &[ActivationComponent],
) -> Result<BTreeMap<&str, &ActivationComponent>, ActivationGraphError> {
    let mut index = BTreeMap::new();
    for component in components {
        validate_label(component.component_id.as_str())?;
        validate_label(component.feature_gate.as_str())?;
        if index
            .insert(component.component_id.as_str(), component)
            .is_some()
        {
            return Err(ActivationGraphError::InvalidLabel);
        }
    }
    Ok(index)
}

fn validate_ports(
    ports: &[ActivationPort],
    components: &BTreeMap<&str, &ActivationComponent>,
) -> Result<(), ActivationGraphError> {
    let mut port_ids = BTreeSet::new();
    for port in ports {
        validate_label(port.port_id.as_str())?;
        validate_label(port.scope_class.as_str())?;
        validate_label(port.feature_gate.as_str())?;
        validate_label(port.focused_check.as_str())?;
        if !port_ids.insert(port.port_id.as_str()) {
            return Err(ActivationGraphError::InvalidLabel);
        }
        validate_component_ref(port.owner_component_id.as_str(), components)?;
        validate_component_ref(port.lifecycle_owner_component_id.as_str(), components)?;
        if port.consumer_component_ids.is_empty()
            || (port.posture == ActivationPosture::Required
                && port.provider_component_ids.is_empty())
        {
            return Err(ActivationGraphError::OrphanRequiredPort);
        }
        if !port.multiplicity.admits(port.provider_component_ids.len()) {
            return Err(ActivationGraphError::InvalidPortMultiplicity);
        }
        unique_labels(port.provider_component_ids.as_slice())?;
        unique_labels(port.consumer_component_ids.as_slice())?;
        for provider_id in &port.provider_component_ids {
            let provider = components
                .get(provider_id.as_str())
                .ok_or(ActivationGraphError::UnknownComponentReference)?;
            if !provider.authority.admits(port.authority) {
                return Err(ActivationGraphError::PortAuthorityMismatch);
            }
        }
        for consumer_id in &port.consumer_component_ids {
            validate_component_ref(consumer_id.as_str(), components)?;
        }
    }
    Ok(())
}

fn validate_events(
    events: &[ActivationEventRoute],
    components: &BTreeMap<&str, &ActivationComponent>,
) -> Result<(), ActivationGraphError> {
    let mut event_ids = BTreeSet::new();
    for event in events {
        validate_label(event.event_id.as_str())?;
        if !event_ids.insert(event.event_id.as_str()) {
            return Err(ActivationGraphError::InvalidLabel);
        }
        validate_component_ref(event.producer_component_id.as_str(), components)?;
        validate_component_ref(event.lifecycle_owner_component_id.as_str(), components)?;
        if event.posture == ActivationPosture::Required && event.consumer_component_ids.is_empty() {
            return Err(ActivationGraphError::MissingEventConsumer);
        }
        unique_labels(event.consumer_component_ids.as_slice())?;
        for consumer_id in &event.consumer_component_ids {
            validate_component_ref(consumer_id.as_str(), components)?;
        }
    }
    Ok(())
}

fn validate_component_ref(
    component_id: &str,
    components: &BTreeMap<&str, &ActivationComponent>,
) -> Result<(), ActivationGraphError> {
    if components.contains_key(component_id) {
        Ok(())
    } else {
        Err(ActivationGraphError::UnknownComponentReference)
    }
}

fn validate_event_cycles(events: &[ActivationEventRoute]) -> Result<(), ActivationGraphError> {
    let mut edges = BTreeMap::<&str, Vec<&str>>::new();
    for event in events {
        for consumer in &event.consumer_component_ids {
            edges
                .entry(event.producer_component_id.as_str())
                .or_default()
                .push(consumer.as_str());
        }
    }
    for event in events.iter().filter(|event| !event.bounded_cycle) {
        for consumer in &event.consumer_component_ids {
            let mut visited = BTreeSet::new();
            if path_exists(
                consumer.as_str(),
                event.producer_component_id.as_str(),
                &edges,
                &mut visited,
            ) {
                return Err(ActivationGraphError::UnsupportedEventCycle);
            }
        }
    }
    Ok(())
}

fn path_exists<'a>(
    current: &'a str,
    target: &str,
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    visited: &mut BTreeSet<&'a str>,
) -> bool {
    if current == target {
        return true;
    }
    if !visited.insert(current) {
        return false;
    }
    edges.get(current).is_some_and(|next_values| {
        next_values
            .iter()
            .any(|next| path_exists(next, target, edges, visited))
    })
}

fn index_observations(
    observations: &[ActivationReadinessObservation],
) -> Result<BTreeMap<&str, &ActivationReadinessObservation>, ActivationGraphError> {
    let mut index = BTreeMap::new();
    let mut observation_ids = BTreeSet::new();
    for observation in observations {
        validate_label(observation.component_id.as_str())?;
        validate_label(observation.observation_id.as_str())?;
        if observation.generation < MINIMUM_GENERATION
            || !observation_ids.insert(observation.observation_id.as_str())
            || index
                .insert(observation.component_id.as_str(), observation)
                .is_some()
        {
            return Err(ActivationGraphError::InvalidObservation);
        }
    }
    Ok(index)
}

fn build_preflight_indexes<'graph, 'selection, 'observation>(
    graph: &'graph ValidatedActivationGraph,
    selection: &'selection SelectedComposition,
    observations: &'observation [ActivationReadinessObservation],
) -> Result<PreflightIndexes<'graph, 'selection, 'observation>, ActivationGraphError> {
    let indexes = PreflightIndexes {
        selected: unique_labels(selection.component_ids.as_slice())?,
        enabled_features: unique_labels(selection.enabled_features.as_slice())?,
        observations: index_observations(observations)?,
        components: graph
            .components
            .iter()
            .map(|component| (component.component_id.as_str(), component))
            .collect(),
    };
    if observations.iter().any(|observation| {
        !indexes
            .components
            .contains_key(observation.component_id.as_str())
    }) {
        return Err(ActivationGraphError::InvalidObservation);
    }
    Ok(indexes)
}

fn evaluate_component_statuses(
    graph: &ValidatedActivationGraph,
    selection: &SelectedComposition,
    indexes: &PreflightIndexes<'_, '_, '_>,
    issues: &mut Vec<ActivationIssue>,
) -> Result<Vec<ActivationComponentStatus>, ActivationGraphError> {
    for component_id in &selection.component_ids {
        if !indexes.components.contains_key(component_id.as_str()) {
            push_issue(
                issues,
                ActivationIssueKind::UnknownSelectedComponent,
                component_id,
            )?;
        }
    }
    let mut statuses = Vec::with_capacity(graph.components.len());
    for component in &graph.components {
        let component_id = component.component_id.as_str();
        if !indexes.selected.contains(component_id) {
            if component.posture == ActivationPosture::Required {
                push_issue(
                    issues,
                    ActivationIssueKind::MissingRequiredComponent,
                    component_id,
                )?;
            }
            statuses.push(component_status(
                component,
                ActivationComponentStatusKind::NotSelected,
            ));
            continue;
        }
        if component.feature_gate != "always"
            && !indexes
                .enabled_features
                .contains(component.feature_gate.as_str())
        {
            push_issue(issues, ActivationIssueKind::FeatureMismatch, component_id)?;
        }
        if !selection.authority_ceiling.admits(component.authority) {
            push_issue(issues, ActivationIssueKind::AuthorityMismatch, component_id)?;
        }
        let readiness = component_readiness(component, selection, indexes, issues)?;
        statuses.push(component_status(component, readiness));
    }
    Ok(statuses)
}

fn component_readiness(
    component: &ActivationComponent,
    selection: &SelectedComposition,
    indexes: &PreflightIndexes<'_, '_, '_>,
    issues: &mut Vec<ActivationIssue>,
) -> Result<ActivationComponentStatusKind, ActivationGraphError> {
    let component_id = component.component_id.as_str();
    let readiness = match indexes.observations.get(component_id) {
        None => {
            let kind = if component.posture == ActivationPosture::Required {
                ActivationIssueKind::MissingReadiness
            } else {
                ActivationIssueKind::OptionalReadinessMissing
            };
            push_issue(issues, kind, component_id)?;
            ActivationComponentStatusKind::MissingReadiness
        }
        Some(observation) if observation.generation != selection.generation => {
            push_issue(issues, ActivationIssueKind::StaleReadiness, component_id)?;
            ActivationComponentStatusKind::StaleReadiness
        }
        Some(observation) if observation.status == ActivationReadinessStatus::Unavailable => {
            let kind = if component.posture == ActivationPosture::Required {
                ActivationIssueKind::RequiredUnavailable
            } else {
                ActivationIssueKind::OptionalUnavailable
            };
            push_issue(issues, kind, component_id)?;
            ActivationComponentStatusKind::Unavailable
        }
        Some(_observation) => ActivationComponentStatusKind::Ready,
    };
    Ok(readiness)
}

fn validate_selected_ports(
    graph: &ValidatedActivationGraph,
    selection: &SelectedComposition,
    indexes: &PreflightIndexes<'_, '_, '_>,
    issues: &mut Vec<ActivationIssue>,
) -> Result<(), ActivationGraphError> {
    for port in &graph.ports {
        let selected_consumer_count = port
            .consumer_component_ids
            .iter()
            .filter(|consumer| indexes.selected.contains(consumer.as_str()))
            .count();
        if selected_consumer_count == 0 {
            continue;
        }
        if !indexes
            .selected
            .contains(port.lifecycle_owner_component_id.as_str())
        {
            push_issue(
                issues,
                ActivationIssueKind::MissingLifecycleOwner,
                port.port_id.as_str(),
            )?;
        }
        if port.feature_gate != "always"
            && !indexes
                .enabled_features
                .contains(port.feature_gate.as_str())
        {
            let kind = if port.posture == ActivationPosture::Required {
                ActivationIssueKind::FeatureMismatch
            } else {
                ActivationIssueKind::OptionalProviderUnavailable
            };
            push_issue(issues, kind, port.port_id.as_str())?;
            continue;
        }
        let selected_provider_count = port
            .provider_component_ids
            .iter()
            .filter(|provider| indexes.selected.contains(provider.as_str()))
            .count();
        if selected_provider_count == 0 {
            let kind = if port.posture == ActivationPosture::Required {
                ActivationIssueKind::MissingProvider
            } else {
                ActivationIssueKind::OptionalProviderUnavailable
            };
            push_issue(issues, kind, port.port_id.as_str())?;
        } else if !port.multiplicity.admits(selected_provider_count) {
            push_issue(
                issues,
                ActivationIssueKind::InvalidMultiplicity,
                port.port_id.as_str(),
            )?;
        }
        if selected_provider_count > 0 && !selection.authority_ceiling.admits(port.authority) {
            push_issue(
                issues,
                ActivationIssueKind::AuthorityMismatch,
                port.port_id.as_str(),
            )?;
        }
    }
    Ok(())
}

fn validate_selected_events(
    graph: &ValidatedActivationGraph,
    selected: &BTreeSet<&str>,
    issues: &mut Vec<ActivationIssue>,
) -> Result<(), ActivationGraphError> {
    for event in &graph.events {
        if !selected.contains(event.producer_component_id.as_str()) {
            continue;
        }
        if !selected.contains(event.lifecycle_owner_component_id.as_str()) {
            push_issue(
                issues,
                ActivationIssueKind::MissingLifecycleOwner,
                event.event_id.as_str(),
            )?;
        }
        let selected_consumer = event
            .consumer_component_ids
            .iter()
            .any(|consumer| selected.contains(consumer.as_str()));
        if !selected_consumer {
            let kind = if event.posture == ActivationPosture::Required {
                ActivationIssueKind::EventRouteGap
            } else {
                ActivationIssueKind::OptionalEventRouteGap
            };
            push_issue(issues, kind, event.event_id.as_str())?;
        }
    }
    Ok(())
}

fn component_status(
    component: &ActivationComponent,
    status: ActivationComponentStatusKind,
) -> ActivationComponentStatus {
    ActivationComponentStatus {
        component_id: component.component_id.clone(),
        feature_gate: component.feature_gate.clone(),
        authority: component.authority,
        posture: component.posture,
        status,
    }
}

fn validate_decision_for_snapshot(
    graph: &ValidatedActivationGraph,
    decision: &ActivationDecision,
) -> Result<(), ActivationGraphError> {
    if decision.graph_identity != graph.identity
        || decision.generation < MINIMUM_GENERATION
        || decision.components.len() != graph.components.len()
        || decision.components.len() > MAX_COMPONENT_COUNT
        || decision.issues.len() > MAX_ISSUE_COUNT
        || decision.selected_component_count > count_u32(MAX_SELECTION_COUNT)?
        || decision.ready_component_count > decision.selected_component_count
        || decision.status != decision_status(decision.issues.as_slice())
        || decision.components.windows(2).any(|pair| pair[0] > pair[1])
        || decision.issues.windows(2).any(|pair| pair[0] > pair[1])
    {
        return Err(ActivationGraphError::InvalidDecision);
    }
    let graph_components = graph
        .components
        .iter()
        .map(|component| (component.component_id.as_str(), component))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for status in &decision.components {
        validate_label(status.component_id.as_str())?;
        validate_label(status.feature_gate.as_str())?;
        let Some(component) = graph_components.get(status.component_id.as_str()) else {
            return Err(ActivationGraphError::InvalidDecision);
        };
        if !seen.insert(status.component_id.as_str())
            || status.feature_gate != component.feature_gate
            || status.authority != component.authority
            || status.posture != component.posture
        {
            return Err(ActivationGraphError::InvalidDecision);
        }
    }
    for issue in &decision.issues {
        validate_label(issue.subject.as_str())?;
    }
    let ready_component_count = decision
        .components
        .iter()
        .filter(|component| component.status == ActivationComponentStatusKind::Ready)
        .count();
    let known_selected_count = decision
        .components
        .iter()
        .filter(|component| component.status != ActivationComponentStatusKind::NotSelected)
        .count();
    let unknown_selected_count = decision
        .issues
        .iter()
        .filter(|issue| issue.kind == ActivationIssueKind::UnknownSelectedComponent)
        .count();
    let selected_component_count = known_selected_count
        .checked_add(unknown_selected_count)
        .ok_or(ActivationGraphError::CountOverflow)?;
    if count_u32(ready_component_count)? != decision.ready_component_count
        || count_u32(selected_component_count)? != decision.selected_component_count
    {
        return Err(ActivationGraphError::InvalidDecision);
    }
    for component in &decision.components {
        let expected_issue = match (component.status, component.posture) {
            (ActivationComponentStatusKind::NotSelected, ActivationPosture::Required) => {
                Some(ActivationIssueKind::MissingRequiredComponent)
            }
            (ActivationComponentStatusKind::MissingReadiness, ActivationPosture::Required) => {
                Some(ActivationIssueKind::MissingReadiness)
            }
            (ActivationComponentStatusKind::MissingReadiness, ActivationPosture::Optional) => {
                Some(ActivationIssueKind::OptionalReadinessMissing)
            }
            (ActivationComponentStatusKind::StaleReadiness, _) => {
                Some(ActivationIssueKind::StaleReadiness)
            }
            (ActivationComponentStatusKind::Unavailable, ActivationPosture::Required) => {
                Some(ActivationIssueKind::RequiredUnavailable)
            }
            (ActivationComponentStatusKind::Unavailable, ActivationPosture::Optional) => {
                Some(ActivationIssueKind::OptionalUnavailable)
            }
            (ActivationComponentStatusKind::NotSelected, ActivationPosture::Optional)
            | (ActivationComponentStatusKind::Ready, _) => None,
        };
        if expected_issue.is_some_and(|kind| {
            !decision
                .issues
                .iter()
                .any(|issue| issue.kind == kind && issue.subject == component.component_id)
        }) {
            return Err(ActivationGraphError::InvalidDecision);
        }
    }
    Ok(())
}

fn push_issue(
    issues: &mut Vec<ActivationIssue>,
    kind: ActivationIssueKind,
    subject: &str,
) -> Result<(), ActivationGraphError> {
    if issues.len() >= MAX_ISSUE_COUNT {
        return Err(ActivationGraphError::BoundExceeded);
    }
    issues.push(ActivationIssue {
        kind,
        subject: subject.to_string(),
    });
    Ok(())
}

fn decision_status(issues: &[ActivationIssue]) -> ActivationDecisionStatus {
    if issues.iter().any(|issue| issue.kind.is_blocking()) {
        ActivationDecisionStatus::Blocked
    } else if issues.is_empty() {
        ActivationDecisionStatus::Ready
    } else {
        ActivationDecisionStatus::DegradedOptional
    }
}

fn unique_labels(values: &[String]) -> Result<BTreeSet<&str>, ActivationGraphError> {
    let mut unique = BTreeSet::new();
    for value in values {
        validate_label(value.as_str())?;
        if !unique.insert(value.as_str()) {
            return Err(ActivationGraphError::InvalidLabel);
        }
    }
    Ok(unique)
}

fn validate_label(value: &str) -> Result<(), ActivationGraphError> {
    let is_valid = !value.trim().is_empty()
        && value.len() <= MAX_LABEL_BYTES
        && !value.contains('|')
        && !value.contains('\0')
        && !value.contains('\n')
        && !value.contains('\r');
    if is_valid {
        validate_activation_evidence_text(value)
            .map_err(|_error| ActivationGraphError::InvalidLabel)
    } else {
        Err(ActivationGraphError::InvalidLabel)
    }
}

fn count_u32(value: usize) -> Result<u32, ActivationGraphError> {
    u32::try_from(value).map_err(|_error| ActivationGraphError::CountOverflow)
}

fn graph_identity(draft: &ActivationGraphDraft) -> String {
    let mut component_parts = draft.components.clone();
    component_parts.sort_by(|left, right| left.component_id.cmp(&right.component_id));
    let mut port_parts = draft.ports.clone();
    port_parts.sort_by(|left, right| left.port_id.cmp(&right.port_id));
    let mut event_parts = draft.events.clone();
    event_parts.sort_by(|left, right| left.event_id.cmp(&right.event_id));
    let mut hasher = Hasher::new();
    hash_value(&mut hasher, GRAPH_DOMAIN);
    hash_value(&mut hasher, GRAPH_SCHEMA);
    for component in component_parts {
        hash_value(&mut hasher, component.component_id.as_str());
        hash_value(&mut hasher, component.feature_gate.as_str());
        hash_value(&mut hasher, component.authority.as_str());
        hash_value(&mut hasher, component.posture.as_str());
    }
    for port in port_parts {
        hash_value(&mut hasher, port.port_id.as_str());
        hash_value(&mut hasher, port.owner_component_id.as_str());
        hash_sorted_values(&mut hasher, port.provider_component_ids.as_slice());
        hash_sorted_values(&mut hasher, port.consumer_component_ids.as_slice());
        hash_value(&mut hasher, port.scope_class.as_str());
        hash_value(&mut hasher, port.multiplicity.as_str());
        hash_value(&mut hasher, port.lifecycle_owner_component_id.as_str());
        hash_value(&mut hasher, port.posture.as_str());
        hash_value(&mut hasher, port.feature_gate.as_str());
        hash_value(&mut hasher, port.authority.as_str());
        hash_value(&mut hasher, port.focused_check.as_str());
    }
    for event in event_parts {
        hash_value(&mut hasher, event.event_id.as_str());
        hash_value(&mut hasher, event.producer_component_id.as_str());
        hash_sorted_values(&mut hasher, event.consumer_component_ids.as_slice());
        hash_value(&mut hasher, event.delivery.as_str());
        hash_value(&mut hasher, event.durability.as_str());
        hash_value(&mut hasher, event.lifecycle_owner_component_id.as_str());
        hash_value(&mut hasher, event.posture.as_str());
        hash_value(
            &mut hasher,
            if event.bounded_cycle {
                "bounded_cycle"
            } else {
                "acyclic_required"
            },
        );
    }
    hash_sorted_values(&mut hasher, draft.non_claims.as_slice());
    hasher.finalize().to_hex().to_string()
}

fn hash_sorted_values(hasher: &mut Hasher, values: &[String]) {
    let mut sorted = values.to_vec();
    sorted.sort();
    for value in sorted {
        hash_value(hasher, value.as_str());
    }
}

fn component_statuses_identity(values: &[ActivationComponentStatus]) -> String {
    let mut hasher = Hasher::new();
    hash_value(&mut hasher, SNAPSHOT_COMPONENTS_DOMAIN);
    for component in values {
        hash_value(&mut hasher, component.component_id.as_str());
        hash_value(&mut hasher, component.feature_gate.as_str());
        hash_value(&mut hasher, component.authority.as_str());
        hash_value(&mut hasher, component.posture.as_str());
        hash_value(&mut hasher, component.status.as_str());
    }
    hasher.finalize().to_hex().to_string()
}

fn issues_identity(values: &[ActivationIssue]) -> String {
    let mut hasher = Hasher::new();
    hash_value(&mut hasher, SNAPSHOT_ISSUES_DOMAIN);
    for issue in values {
        hash_value(&mut hasher, issue.kind.as_str());
        hash_value(&mut hasher, issue.subject.as_str());
    }
    hasher.finalize().to_hex().to_string()
}

fn hash_parts(domain: &str, values: &[&str]) -> String {
    let mut hasher = Hasher::new();
    hash_value(&mut hasher, domain);
    for value in values {
        hash_value(&mut hasher, value);
    }
    hasher.finalize().to_hex().to_string()
}

fn hash_value(hasher: &mut Hasher, value: &str) {
    hasher.update(&[HASH_SEPARATOR]);
    hasher.update(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    const GENERATION: u64 = 7;
    const STALE_GENERATION: u64 = 6;
    const EXPECTED_SELECTED_COMPONENT_COUNT: u32 = 3;
    const EXPECTED_READY_COMPONENT_COUNT: u32 = 3;
    const EXPECTED_GRAPH_COMPONENT_COUNT: usize = 4;

    #[derive(Debug, Eq, PartialEq)]
    struct HostControlInspectionFixture {
        schema: &'static str,
        graph_identity: String,
        generation: u64,
        status: ActivationDecisionStatus,
        component_count: usize,
        issue_count: usize,
    }

    impl From<&ActivationSnapshot> for HostControlInspectionFixture {
        fn from(snapshot: &ActivationSnapshot) -> Self {
            Self {
                schema: snapshot.schema,
                graph_identity: snapshot.graph_identity.clone(),
                generation: snapshot.generation,
                status: snapshot.status,
                component_count: snapshot.components.len(),
                issue_count: snapshot.issues.len(),
            }
        }
    }

    fn required_non_claims() -> Vec<String> {
        REQUIRED_NON_CLAIMS
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    }

    fn component(
        component_id: &str,
        feature_gate: &str,
        authority: ActivationAuthorityClass,
        posture: ActivationPosture,
    ) -> ActivationComponent {
        ActivationComponent {
            component_id: component_id.to_string(),
            feature_gate: feature_gate.to_string(),
            authority,
            posture,
        }
    }

    fn graph_draft() -> ActivationGraphDraft {
        ActivationGraphDraft {
            components: vec![
                component(
                    "runtime.core",
                    "always",
                    ActivationAuthorityClass::None,
                    ActivationPosture::Required,
                ),
                component(
                    "host.provider",
                    "live-provider",
                    ActivationAuthorityClass::ShellOwned,
                    ActivationPosture::Required,
                ),
                component(
                    "product.shell",
                    "live-product",
                    ActivationAuthorityClass::ShellOwned,
                    ActivationPosture::Required,
                ),
                component(
                    "host.optional-observer",
                    "optional-observer",
                    ActivationAuthorityClass::InjectedOnly,
                    ActivationPosture::Optional,
                ),
            ],
            ports: vec![ActivationPort {
                port_id: "port.model".to_string(),
                owner_component_id: "runtime.core".to_string(),
                provider_component_ids: vec!["host.provider".to_string()],
                consumer_component_ids: vec!["product.shell".to_string()],
                scope_class: "application".to_string(),
                multiplicity: PortMultiplicity::ExactlyOne,
                lifecycle_owner_component_id: "product.shell".to_string(),
                posture: ActivationPosture::Required,
                feature_gate: "live-provider".to_string(),
                authority: ActivationAuthorityClass::ShellOwned,
                focused_check: "check-composition-activation-graph".to_string(),
            }],
            events: vec![ActivationEventRoute {
                event_id: "event.model-result".to_string(),
                producer_component_id: "runtime.core".to_string(),
                consumer_component_ids: vec!["product.shell".to_string()],
                delivery: ActivationDeliveryClass::Ordered,
                durability: ActivationDurabilityClass::MetadataOnly,
                lifecycle_owner_component_id: "product.shell".to_string(),
                posture: ActivationPosture::Required,
                bounded_cycle: false,
            }],
            non_claims: required_non_claims(),
        }
    }

    fn graph() -> ValidatedActivationGraph {
        ValidatedActivationGraph::validate(graph_draft()).expect("graph")
    }

    fn selection() -> SelectedComposition {
        SelectedComposition {
            generation: GENERATION,
            component_ids: vec![
                "runtime.core".to_string(),
                "host.provider".to_string(),
                "product.shell".to_string(),
            ],
            enabled_features: vec!["live-provider".to_string(), "live-product".to_string()],
            authority_ceiling: ActivationAuthorityClass::ShellOwned,
        }
    }

    fn observation(component_id: &str) -> ActivationReadinessObservation {
        ActivationReadinessObservation {
            component_id: component_id.to_string(),
            generation: GENERATION,
            observation_id: alloc::format!("observation-{component_id}"),
            status: ActivationReadinessStatus::Ready,
        }
    }

    fn observations() -> Vec<ActivationReadinessObservation> {
        vec![
            observation("runtime.core"),
            observation("host.provider"),
            observation("product.shell"),
        ]
    }

    #[test]
    fn complete_required_closure_is_ready_and_deterministic() {
        // r[verify agent_harness_runtime.composition_activation_graph.preflight]
        let graph = graph();
        let first = preflight_composition_activation(&graph, &selection(), &observations())
            .expect("first preflight");
        let second = preflight_composition_activation(&graph, &selection(), &observations())
            .expect("second preflight");
        let snapshot = project_activation_snapshot(&graph, &first).expect("safe snapshot");
        let host_control = HostControlInspectionFixture::from(&snapshot);

        assert_eq!(first, second);
        assert_eq!(first.status, ActivationDecisionStatus::Ready);
        assert!(first.effects_admitted());
        assert_eq!(
            first.selected_component_count,
            EXPECTED_SELECTED_COMPONENT_COUNT
        );
        assert_eq!(first.ready_component_count, EXPECTED_READY_COMPONENT_COUNT);
        assert_eq!(snapshot.status, ActivationDecisionStatus::Ready);
        assert_eq!(snapshot.graph_identity, graph.identity());
        assert_eq!(snapshot.components.len(), EXPECTED_GRAPH_COMPONENT_COUNT);
        assert_eq!(host_control.schema, SNAPSHOT_SCHEMA);
        assert_eq!(host_control.graph_identity, graph.identity());
        assert_eq!(host_control.generation, GENERATION);
        assert_eq!(host_control.status, ActivationDecisionStatus::Ready);
        assert_eq!(host_control.component_count, EXPECTED_GRAPH_COMPONENT_COUNT);
        assert_eq!(host_control.issue_count, 0);

        let mut bounded_cycle = graph_draft();
        bounded_cycle.events[0].bounded_cycle = true;
        bounded_cycle.events.push(ActivationEventRoute {
            event_id: "event.bounded-reply".to_string(),
            producer_component_id: "product.shell".to_string(),
            consumer_component_ids: vec!["runtime.core".to_string()],
            delivery: ActivationDeliveryClass::Ordered,
            durability: ActivationDurabilityClass::LiveOnly,
            lifecycle_owner_component_id: "product.shell".to_string(),
            posture: ActivationPosture::Required,
            bounded_cycle: true,
        });
        assert!(ValidatedActivationGraph::validate(bounded_cycle).is_ok());
    }

    #[test]
    fn optional_unavailability_is_visible_without_substitution() {
        // r[verify agent_harness_runtime.composition_activation_graph.preflight]
        let graph = graph();
        let mut selection = selection();
        selection
            .component_ids
            .push("host.optional-observer".to_string());
        selection
            .enabled_features
            .push("optional-observer".to_string());
        let mut observations = observations();
        let mut optional = observation("host.optional-observer");
        optional.status = ActivationReadinessStatus::Unavailable;
        observations.push(optional);

        let decision = preflight_composition_activation(&graph, &selection, &observations)
            .expect("degraded preflight");

        assert_eq!(decision.status, ActivationDecisionStatus::DegradedOptional);
        assert!(decision.effects_admitted());
        assert_eq!(
            decision.issues,
            vec![ActivationIssue {
                kind: ActivationIssueKind::OptionalUnavailable,
                subject: "host.optional-observer".to_string(),
            }]
        );

        observations.pop();
        let missing = preflight_composition_activation(&graph, &selection, &observations)
            .expect("missing optional observation");
        assert_eq!(missing.status, ActivationDecisionStatus::DegradedOptional);
        assert_eq!(
            missing.issues,
            vec![ActivationIssue {
                kind: ActivationIssueKind::OptionalReadinessMissing,
                subject: "host.optional-observer".to_string(),
            }]
        );
    }

    #[test]
    fn missing_provider_stale_readiness_and_authority_fail_closed() {
        // r[verify agent_harness_runtime.composition_activation_graph.fail_loud]
        let graph = graph();
        let mut blocked_selection = selection();
        blocked_selection
            .component_ids
            .retain(|component| component != "host.provider");
        blocked_selection.authority_ceiling = ActivationAuthorityClass::InjectedOnly;
        let mut blocked_observations = observations();
        blocked_observations[0].generation = STALE_GENERATION;

        let decision =
            preflight_composition_activation(&graph, &blocked_selection, &blocked_observations)
                .expect("blocked preflight");

        assert_eq!(decision.status, ActivationDecisionStatus::Blocked);
        assert!(!decision.effects_admitted());
        assert!(
            decision
                .issues
                .iter()
                .any(|issue| issue.kind == ActivationIssueKind::MissingProvider)
        );
        assert!(
            decision
                .issues
                .iter()
                .any(|issue| issue.kind == ActivationIssueKind::StaleReadiness)
        );
        assert!(
            decision
                .issues
                .iter()
                .any(|issue| issue.kind == ActivationIssueKind::AuthorityMismatch)
        );

        let mut lifecycle_selection = selection();
        lifecycle_selection
            .component_ids
            .retain(|component| component != "product.shell");
        let lifecycle = preflight_composition_activation(
            &graph,
            &lifecycle_selection,
            observations().as_slice(),
        )
        .expect("lifecycle gap");
        assert_eq!(lifecycle.status, ActivationDecisionStatus::Blocked);
        assert!(
            lifecycle
                .issues
                .iter()
                .any(|issue| { issue.kind == ActivationIssueKind::MissingLifecycleOwner })
        );
        assert!(
            lifecycle
                .issues
                .iter()
                .any(|issue| issue.kind == ActivationIssueKind::EventRouteGap)
        );
    }

    #[test]
    fn graph_validation_rejects_unknown_refs_multiplicity_and_unbounded_cycles() {
        // r[verify agent_harness_runtime.composition_activation_graph.catalog]
        let mut unknown = graph_draft();
        unknown.ports[0].provider_component_ids = vec!["unknown.provider".to_string()];
        assert_eq!(
            ValidatedActivationGraph::validate(unknown),
            Err(ActivationGraphError::UnknownComponentReference)
        );

        let mut multiplicity = graph_draft();
        multiplicity.ports[0]
            .provider_component_ids
            .push("runtime.core".to_string());
        assert_eq!(
            ValidatedActivationGraph::validate(multiplicity),
            Err(ActivationGraphError::InvalidPortMultiplicity)
        );

        let mut cycle = graph_draft();
        cycle.events.push(ActivationEventRoute {
            event_id: "event.shell-result".to_string(),
            producer_component_id: "product.shell".to_string(),
            consumer_component_ids: vec!["runtime.core".to_string()],
            delivery: ActivationDeliveryClass::Ordered,
            durability: ActivationDurabilityClass::LiveOnly,
            lifecycle_owner_component_id: "product.shell".to_string(),
            posture: ActivationPosture::Required,
            bounded_cycle: false,
        });
        assert_eq!(
            ValidatedActivationGraph::validate(cycle),
            Err(ActivationGraphError::UnsupportedEventCycle)
        );

        let mut duplicate_observation = observations();
        duplicate_observation[1].observation_id = duplicate_observation[0].observation_id.clone();
        assert_eq!(
            preflight_composition_activation(&graph(), &selection(), &duplicate_observation),
            Err(ActivationGraphError::InvalidObservation)
        );
    }

    #[test]
    fn evidence_rejects_private_material_and_overclaims() {
        // r[verify agent_harness_runtime.composition_activation_graph.inspection]
        assert_eq!(
            validate_activation_evidence_text("credential=secret"),
            Err(ActivationGraphError::UnsafeEvidence)
        );
        assert_eq!(
            validate_activation_evidence_text("this proves readiness"),
            Err(ActivationGraphError::UnsafeEvidence)
        );
        assert!(validate_activation_evidence_text("status=blocked issue=stale_readiness").is_ok());

        let graph = graph();
        let mut decision = preflight_composition_activation(&graph, &selection(), &observations())
            .expect("decision");
        decision.graph_identity = "other-graph".to_string();
        assert_eq!(
            project_activation_snapshot(&graph, &decision),
            Err(ActivationGraphError::InvalidDecision)
        );

        let mut inconsistent =
            preflight_composition_activation(&graph, &selection(), &observations())
                .expect("decision");
        inconsistent.components[0].status = ActivationComponentStatusKind::MissingReadiness;
        inconsistent.ready_component_count = inconsistent.ready_component_count.saturating_sub(1);
        assert_eq!(
            project_activation_snapshot(&graph, &inconsistent),
            Err(ActivationGraphError::InvalidDecision)
        );

        let mut nul_label = graph_draft();
        nul_label.components[0].component_id = "runtime\0core".to_string();
        assert_eq!(
            ValidatedActivationGraph::validate(nul_label),
            Err(ActivationGraphError::InvalidLabel)
        );
    }
}
