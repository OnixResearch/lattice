//! Producer drafts built from the declaration and the broker's facts.

const REQUEST_DOMAIN: &str = "lattice-model-request-v1 2026-11-19";
const BACKEND_LABEL: &str = "backend:lattice-model-broker";
const ROLE_LABEL: &str = "primary";

type Admitted = lattice_core::execution_binding::admission::bound::Admitted;

/// The declared route as a producer route candidate, carrying Lattice's
/// `lattice-model-route-v1` identity as the caller-owned route reference.
pub(super) fn candidate(
    admitted: &Admitted,
    route: &lattice_core::execution_binding::serving::Route,
) -> Result<animus_adapters::serving::RouteCandidate, String> {
    let route_ref = match &admitted.model_routes {
        lattice_core::execution_binding::request::RouteDeclaration::Routes { routes } => {
            routes.iter().find(|named| named.name == route.name).map(|named| named.route_identity.as_str())
        }
        lattice_core::execution_binding::request::RouteDeclaration::NoModel {} => None,
    }
    .ok_or("served route is not a declared model route")?;
    route_candidate(route, Some(route_ref))
}

/// The declared route as a producer route candidate with an optional route reference.
pub(super) fn route_candidate(
    route: &lattice_core::execution_binding::serving::Route,
    route_ref: Option<&str>,
) -> Result<animus_adapters::serving::RouteCandidate, String> {
    let label = format!("route:{}", route.name);
    let requirements = route.requirements;
    debug_assert!(requirements.output_tokens > 0, "validated requirements have output tokens");
    debug_assert!(lattice_core::execution_binding::serving::is_producer_token(&route.service_id));
    animus_adapters::serving::RouteCandidate::new(&animus_adapters::serving::RouteCandidateDraft {
        route_name: &route.name,
        route_ref,
        label: &label,
        backend: BACKEND_LABEL,
        role: ROLE_LABEL,
        service_id: &route.service_id,
        model_id: &route.model_id,
        configuration_ref: &route.configuration_ref,
        requirements: animus_adapters::mesh::Requirements {
            tools: requirements.needs_tools,
            streaming: requirements.needs_streaming,
            structured_output: requirements.needs_structured_output,
            developer_role: requirements.needs_developer_role,
            input_tokens: requirements.input_tokens,
            output_tokens: requirements.output_tokens,
        },
        rollout: None,
    })
    .map_err(|issue| format!("served route is not a producer route candidate: {issue:?}"))
}

/// The declared serving profile as the producer profile.
pub(super) fn profile(
    profile: &lattice_core::execution_binding::serving::Profile,
) -> Result<animus_adapters::serving::Profile, String> {
    animus_adapters::serving::Profile::new(animus_adapters::serving::ProfileDraft {
        profile_id: &profile.profile_id,
        identity: match profile.identity_rule {
            lattice_core::execution_binding::serving::IdentityRule::ClaimedAccepted => {
                animus_adapters::serving::IdentityRule::ClaimedAccepted
            }
            lattice_core::execution_binding::serving::IdentityRule::ObservedRequired => {
                animus_adapters::serving::IdentityRule::ObservedRequired
            }
        },
        capacity: match profile.capacity_rule {
            lattice_core::execution_binding::serving::CapacityRule::KnownRequired => {
                animus_adapters::serving::CapacityRule::KnownRequired
            }
            lattice_core::execution_binding::serving::CapacityRule::OwnerAdmissionDecides => {
                animus_adapters::serving::CapacityRule::OwnerAdmissionDecides
            }
        },
        replica: match profile.replica_rule {
            lattice_core::execution_binding::serving::ReplicaRule::SameModelAllowed => {
                animus_adapters::serving::ReplicaRule::SameModelAllowed
            }
            lattice_core::execution_binding::serving::ReplicaRule::Pinned => {
                animus_adapters::serving::ReplicaRule::Pinned
            }
        },
        max_snapshot_age_ticks: profile.max_snapshot_age_ticks,
    })
    .map_err(|issue| format!("serving profile is not a producer profile: {issue:?}"))
}

/// The declared privacy band as the producer's.
pub(super) const fn privacy(
    privacy: lattice_core::execution_binding::serving::Privacy,
) -> animus_adapters::router::PrivacyBand {
    match privacy {
        lattice_core::execution_binding::serving::Privacy::LocalOnly => animus_adapters::router::PrivacyBand::LocalOnly,
        lattice_core::execution_binding::serving::Privacy::RemoteAllowed => {
            animus_adapters::router::PrivacyBand::RemoteAllowed
        }
    }
}

/// Deterministic request identity: binding and route, then the per-route ordinal.
pub(super) fn request_id((binding_identity, route_name): (&str, &str), ordinal: u64) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(REQUEST_DOMAIN);
    hasher.update(binding_identity.as_bytes());
    hasher.update(&[0]);
    hasher.update(route_name.as_bytes());
    hasher.update(&[0]);
    hasher.update(&ordinal.to_le_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

pub(super) fn reading(current: &super::Current) -> animus_adapters::serving::CurrentReading<'_> {
    match current {
        super::Current::Observed(facts) => {
            animus_adapters::serving::CurrentReading::Observed(animus_adapters::serving::CurrentState {
                service_id: &facts.service_id,
                endpoint_identity: &facts.endpoint_identity,
                deployment_id: &facts.deployment_id,
                deployment_generation: facts.deployment_generation,
                model_id: &facts.model_id,
                configuration_ref: &facts.configuration_ref,
                readiness: facts.readiness,
                admission_epoch: facts.admission_epoch,
                now_tick: facts.now_tick,
                policy_generation: facts.policy_generation,
            })
        }
        super::Current::Unavailable => animus_adapters::serving::CurrentReading::Unavailable,
        super::Current::PortIssue => animus_adapters::serving::CurrentReading::PortIssue,
    }
}

/// The broker's checks as the producer's existing Mesh dispatch admission.
/// Profile, privacy, authority and remote admission were settled when the
/// binding was admitted; privacy is screened again per request.
pub(super) const fn existing(checks: super::gate::Checks) -> animus_adapters::mesh::DispatchAdmission {
    animus_adapters::mesh::DispatchAdmission {
        exact_model: if checks.is_exact_model {
            animus_adapters::mesh::ExactModelDecision::Admitted
        } else {
            animus_adapters::mesh::ExactModelDecision::MissingModel
        },
        profile_admitted: true,
        privacy_admitted: true,
        budget_admitted: checks.is_within_budget,
        authority_admitted: true,
        remote_admitted: true,
        deadline_available: checks.is_current,
        cancellation_available: true,
        transport_available: true,
    }
}

pub(super) type Settled = (
    animus_adapters::serving::DispatchOutcome,
    Option<animus_adapters::serving::ResponseProvenance>,
    animus_adapters::serving::Usage,
    Option<String>,
);

pub(super) fn responded(
    query: &animus_adapters::serving::AdmissionQuery,
    facts: super::gate::ResponseFacts,
) -> Settled {
    let counter = |value: Option<u64>| {
        value.map_or(animus_adapters::serving::Counter::Unknown, animus_adapters::serving::Counter::Known)
    };
    let has_report = facts.input_tokens.is_some() || facts.output_tokens.is_some();
    let usage = animus_adapters::serving::Usage {
        input_tokens: counter(facts.input_tokens),
        output_tokens: counter(facts.output_tokens),
        origin: if has_report {
            animus_adapters::serving::UsageOrigin::ProviderReported
        } else {
            animus_adapters::serving::UsageOrigin::Unknown
        },
    };
    let response_facts = animus_adapters::serving::ResponseFacts {
        usage,
        policy_generation: None,
    };
    let provenance = animus_adapters::serving::response_provenance(query, facts.model.as_deref(), &response_facts);
    let outcome = provenance.rejection().map_or(
        animus_adapters::serving::DispatchOutcome::Completed,
        animus_adapters::serving::DispatchOutcome::ResponseRejected,
    );
    let provider = facts
        .provider_receipt
        .filter(|value| lattice_core::execution_binding::serving::is_producer_token(value));
    debug_assert!(has_report || usage.origin == animus_adapters::serving::UsageOrigin::Unknown);
    (outcome, Some(provenance), usage, provider)
}

pub(super) const fn uncertain(cause: animus_adapters::serving::UncertainCause) -> Settled {
    (
        animus_adapters::serving::DispatchOutcome::Uncertain(cause),
        None,
        animus_adapters::serving::Usage::UNKNOWN,
        None,
    )
}

/// The deployment, model and replica a producer receipt selected.
pub(super) fn served(
    receipt: &animus_adapters::serving::InvocationReceipt,
    selected: &animus_adapters::serving::EligibleRoute,
) -> lattice_core::execution_binding::serving::record::Served {
    let deployment = selected.deployment();
    debug_assert!(deployment.deployment_generation() > 0, "producer generations are positive");
    debug_assert_eq!(receipt.selected().map(|route| route.route().route_name()), Some(selected.route().route_name()));
    let replica = match receipt.replica() {
        animus_adapters::serving::ReplicaFact::Observed(token) => Some(token.as_str().to_owned()),
        animus_adapters::serving::ReplicaFact::Unknown => None,
    };
    lattice_core::execution_binding::serving::record::Served {
        endpoint_identity: deployment.endpoint_identity().to_owned(),
        deployment_id: deployment.deployment_id().to_owned(),
        deployment_generation: deployment.deployment_generation(),
        model_id: selected.model().model_id().to_owned(),
        configuration_ref: selected.model().configuration_ref().to_owned(),
        attestation: match selected.attestation() {
            animus_adapters::serving::Attestation::Claimed => {
                lattice_core::execution_binding::serving::record::Attestation::Claimed
            }
            animus_adapters::serving::Attestation::Observed => {
                lattice_core::execution_binding::serving::record::Attestation::Observed
            }
        },
        capacity: match selected.capacity() {
            animus_adapters::serving::CapacityStanding::Known { kind, value } => {
                lattice_core::execution_binding::serving::record::Capacity::Known {
                    kind: kind.as_str().to_owned(),
                    value,
                }
            }
            animus_adapters::serving::CapacityStanding::UnknownOwnerDecides => {
                lattice_core::execution_binding::serving::record::Capacity::OwnerDecides {}
            }
        },
        replica,
        is_replica_changed: receipt.is_replica_changed(),
        resource_consumer_ref: selected.resource_consumer_ref().map(str::to_owned),
    }
}
