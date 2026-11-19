//! Shared deterministic fixtures. Every value is synthetic.

pub(super) const CONFIG_A: &str =
    "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(super) const CONFIG_B: &str =
    "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const ROUTE_REF: &str =
    "blake3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
pub(super) const SERVICE_A: &str = "svc-a";
pub(super) const SERVICE_B: &str = "svc-b";
pub(super) const MODEL_A: &str = "model-a";
pub(super) const MODEL_B: &str = "model-b";
pub(super) const DEPLOYMENT_A: &str = "deploy-a";
pub(super) const ENDPOINT_A: &str = "endpoint:a";
pub(super) const REPLICA_ONE: &str = "replica-1";
pub(super) const REPLICA_TWO: &str = "replica-2";
pub(super) const SERVING_CONSUMER: &str = "serving-b";
pub(super) const LABEL_PRIMARY: &str = "model:primary";
pub(super) const LABEL_SECONDARY: &str = "model:secondary";
pub(super) const BACKEND_REMOTE: &str = "backend:remote";
pub(super) const REQUEST_ONE: &str = "request-1";
pub(super) const REQUEST_TWO: &str = "request-2";
pub(super) const GRANT_ONE: &str = "grant-1";
pub(super) const GENERATION_ONE: u64 = 1;
pub(super) const GENERATION_TWO: u64 = 2;
pub(super) const EPOCH_ONE: u64 = 1;
pub(super) const EPOCH_TWO: u64 = 2;
pub(super) const SEQUENCE_ONE: u64 = 1;
pub(super) const SEQUENCE_TWO: u64 = 2;
pub(super) const SEQUENCE_THREE: u64 = 3;
pub(super) const OBSERVED_TICK: u64 = 100;
pub(super) const BEFORE_OBSERVED_TICK: u64 = 99;
pub(super) const NOW_TICK: u64 = 105;
pub(super) const TOO_OLD_TICK: u64 = 131;
pub(super) const VALID_UNTIL_TICK: u64 = 160;
pub(super) const GRANT_VALID_UNTIL_TICK: u64 = 130;
pub(super) const MAX_AGE_TICKS: u64 = 30;
pub(super) const CONTEXT_TOKENS: u32 = 8_192;
pub(super) const OUTPUT_TOKENS: u32 = 1_024;
pub(super) const INPUT_TOKENS: u32 = 2_000;
pub(super) const REQUEST_OUTPUT_TOKENS: u32 = 500;
pub(super) const ONE_SLOT: u64 = 1;

pub(super) const ALL_SUPPORTED: crate::serving::CapabilitySet = crate::serving::CapabilitySet {
    text: crate::mesh::Capability::Supported,
    tools: crate::mesh::Capability::Supported,
    streaming: crate::mesh::Capability::Supported,
    structured_output: crate::mesh::Capability::Supported,
    developer_role: crate::mesh::Capability::Supported,
};

pub(super) const ONE_SLOT_CAPACITY: [crate::serving::CapacityDraft; 1] =
    [crate::serving::CapacityDraft {
        kind: crate::serving::CapacityKind::RequestSlots,
        unit: crate::serving::CapacityUnit::Requests,
        scope: crate::serving::CapacityScope::Deployment,
        origin: crate::serving::CapacityOrigin::ServingOwner,
        value: crate::serving::Counter::Known(ONE_SLOT),
        observed_tick: OBSERVED_TICK,
        valid_until_tick: VALID_UNTIL_TICK,
    }];

pub(super) const REQUIREMENTS: crate::mesh::Requirements = crate::mesh::Requirements {
    tools: true,
    streaming: false,
    structured_output: false,
    developer_role: false,
    input_tokens: INPUT_TOKENS,
    output_tokens: REQUEST_OUTPUT_TOKENS,
};

pub(super) fn envelope() -> crate::serving::Envelope {
    crate::serving::Envelope::new(CONTEXT_TOKENS, OUTPUT_TOKENS).expect("fixture envelope is valid")
}

pub(super) fn service() -> crate::serving::ServiceDraft<'static> {
    crate::serving::ServiceDraft {
        service_id: SERVICE_A,
        endpoint_identity: ENDPOINT_A,
        deployment_id: DEPLOYMENT_A,
        deployment_generation: GENERATION_ONE,
        locality: crate::serving::Locality::PrivateRemote,
        model: Some(crate::serving::ModelDraft {
            model_id: MODEL_A,
            configuration_ref: CONFIG_A,
            attestation: crate::serving::Attestation::Observed,
        }),
        readiness: crate::serving::Readiness::Ready,
        capabilities: ALL_SUPPORTED,
        envelope: Some(envelope()),
        capacity: &ONE_SLOT_CAPACITY,
        admission: crate::serving::AdmissionOffer::OwnerAdmission,
        replica: Some(REPLICA_ONE),
        resource_consumer_ref: Some(SERVING_CONSUMER),
        rollout: None,
    }
}

pub(super) const fn snapshot_draft<'a>(
    services: &'a [crate::serving::ServiceDraft<'a>],
) -> crate::serving::SnapshotDraft<'a> {
    crate::serving::SnapshotDraft {
        source_id: "host-observer",
        source_generation: GENERATION_ONE,
        sequence: SEQUENCE_ONE,
        observed_tick: OBSERVED_TICK,
        valid_until_tick: VALID_UNTIL_TICK,
        services,
    }
}

pub(super) fn snapshot_of(
    services: &[crate::serving::ServiceDraft<'_>],
) -> crate::serving::AvailabilitySnapshot {
    crate::serving::AvailabilitySnapshot::new(&snapshot_draft(services))
        .expect("fixture snapshot is valid")
}

pub(super) fn snapshot_with(
    edit: impl FnOnce(&mut crate::serving::ServiceDraft<'static>),
) -> crate::serving::AvailabilitySnapshot {
    let mut draft = service();
    edit(&mut draft);
    snapshot_of(&[draft])
}

pub(super) fn profile_with(
    capacity: crate::serving::CapacityRule,
    replica: crate::serving::ReplicaRule,
) -> crate::serving::Profile {
    crate::serving::Profile::new(crate::serving::ProfileDraft {
        profile_id: "capacity-aware-v1",
        identity: crate::serving::IdentityRule::ObservedRequired,
        capacity,
        replica,
        max_snapshot_age_ticks: MAX_AGE_TICKS,
    })
    .expect("fixture profile is valid")
}

pub(super) fn profile() -> crate::serving::Profile {
    profile_with(
        crate::serving::CapacityRule::KnownRequired,
        crate::serving::ReplicaRule::SameModelAllowed,
    )
}

pub(super) const fn candidate_draft() -> crate::serving::RouteCandidateDraft<'static> {
    crate::serving::RouteCandidateDraft {
        route_name: "primary",
        route_ref: Some(ROUTE_REF),
        label: LABEL_PRIMARY,
        backend: BACKEND_REMOTE,
        role: "primary",
        service_id: SERVICE_A,
        model_id: MODEL_A,
        configuration_ref: CONFIG_A,
        requirements: REQUIREMENTS,
        rollout: None,
    }
}

pub(super) fn candidate() -> crate::serving::RouteCandidate {
    crate::serving::RouteCandidate::new(&candidate_draft()).expect("fixture candidate is valid")
}

pub(super) fn candidate_with(
    edit: impl FnOnce(&mut crate::serving::RouteCandidateDraft<'static>),
) -> crate::serving::RouteCandidate {
    let mut draft = candidate_draft();
    edit(&mut draft);
    crate::serving::RouteCandidate::new(&draft).expect("fixture candidate is valid")
}

pub(super) const fn fresh() -> crate::serving::Freshness {
    crate::serving::Freshness {
        now_tick: Some(NOW_TICK),
        newest_accepted: None,
    }
}

pub(super) fn screen_at(
    profile: &crate::serving::Profile,
    snapshot: &crate::serving::AvailabilitySnapshot,
    view: (crate::serving::Freshness, crate::router::PrivacyBand),
    candidates: &[crate::serving::RouteCandidate],
) -> crate::serving::Screening {
    crate::serving::screen_routes(&crate::serving::ScreenRequest {
        profile,
        snapshot,
        freshness: view.0,
        privacy: view.1,
        candidates,
    })
    .expect("fixture candidates are valid")
}

pub(super) fn screen(
    profile: &crate::serving::Profile,
    snapshot: &crate::serving::AvailabilitySnapshot,
    candidates: &[crate::serving::RouteCandidate],
) -> crate::serving::Screening {
    screen_at(
        profile,
        snapshot,
        (fresh(), crate::router::PrivacyBand::RemoteAllowed),
        candidates,
    )
}

pub(super) fn caller() -> crate::serving::CallerBinding {
    crate::serving::CallerBinding::new(crate::serving::CallerDraft {
        agent: "agent-1",
        session: "session-1",
        attempt: "attempt-1",
    })
    .expect("fixture caller is valid")
}

pub(super) const fn existing() -> crate::mesh::DispatchAdmission {
    crate::mesh::DispatchAdmission {
        exact_model: crate::mesh::ExactModelDecision::Admitted,
        profile_admitted: true,
        privacy_admitted: true,
        budget_admitted: true,
        authority_admitted: true,
        remote_admitted: true,
        deadline_available: true,
        cancellation_available: true,
        transport_available: true,
    }
}

pub(super) const fn grant_draft() -> crate::serving::GrantDraft<'static> {
    crate::serving::GrantDraft {
        grant_ref: GRANT_ONE,
        request_id: REQUEST_ONE,
        service_id: SERVICE_A,
        deployment_id: DEPLOYMENT_A,
        deployment_generation: GENERATION_ONE,
        model_id: MODEL_A,
        configuration_ref: CONFIG_A,
        admission_epoch: EPOCH_ONE,
        issued_tick: NOW_TICK,
        valid_until_tick: GRANT_VALID_UNTIL_TICK,
        replica: Some(REPLICA_ONE),
        policy_generation: None,
    }
}

pub(super) const fn current() -> crate::serving::CurrentState<'static> {
    crate::serving::CurrentState {
        service_id: SERVICE_A,
        endpoint_identity: ENDPOINT_A,
        deployment_id: DEPLOYMENT_A,
        deployment_generation: GENERATION_ONE,
        model_id: MODEL_A,
        configuration_ref: CONFIG_A,
        readiness: crate::serving::Readiness::Ready,
        admission_epoch: EPOCH_ONE,
        now_tick: Some(NOW_TICK),
        policy_generation: None,
    }
}

/// The only exclusion reason of a single-candidate screening.
pub(super) fn only_reason(
    snapshot: &crate::serving::AvailabilitySnapshot,
) -> crate::serving::ExclusionReason {
    let screening = screen(&profile(), snapshot, &[candidate()]);
    assert!(screening.eligible().is_empty());
    assert_eq!(screening.exclusions().len(), 1);
    screening.exclusions()[0].reason()
}
