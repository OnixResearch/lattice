//! Eligibility before preference.

const fn slots(
    value: crate::serving::Counter,
    valid_until_tick: u64,
) -> crate::serving::CapacityDraft {
    crate::serving::CapacityDraft {
        kind: crate::serving::CapacityKind::RequestSlots,
        unit: crate::serving::CapacityUnit::Requests,
        scope: crate::serving::CapacityScope::Deployment,
        origin: crate::serving::CapacityOrigin::ServingOwner,
        value,
        observed_tick: super::fixtures::OBSERVED_TICK,
        valid_until_tick,
    }
}

const fn account_quota(
    kind: crate::serving::CapacityKind,
    value: crate::serving::Counter,
) -> crate::serving::CapacityDraft {
    crate::serving::CapacityDraft {
        kind,
        unit: kind.unit(),
        scope: crate::serving::CapacityScope::Account,
        origin: crate::serving::CapacityOrigin::ProviderApi,
        value,
        observed_tick: super::fixtures::OBSERVED_TICK,
        valid_until_tick: super::fixtures::VALID_UNTIL_TICK,
    }
}

const EXHAUSTED: [crate::serving::CapacityDraft; 1] = [slots(
    crate::serving::Counter::Known(0),
    super::fixtures::VALID_UNTIL_TICK,
)];
const UNKNOWN_SLOTS: [crate::serving::CapacityDraft; 1] = [slots(
    crate::serving::Counter::Unknown,
    super::fixtures::VALID_UNTIL_TICK,
)];
const EXPIRED_SLOTS: [crate::serving::CapacityDraft; 1] = [slots(
    crate::serving::Counter::Known(4),
    super::fixtures::NOW_TICK,
)];
const FREE_MEMORY_ONLY: [crate::serving::CapacityDraft; 2] = [
    crate::serving::CapacityDraft {
        kind: crate::serving::CapacityKind::FreeMemory,
        unit: crate::serving::CapacityUnit::Bytes,
        scope: crate::serving::CapacityScope::Replica,
        origin: crate::serving::CapacityOrigin::HostProbe,
        value: crate::serving::Counter::Known(68_719_476_736),
        observed_tick: super::fixtures::OBSERVED_TICK,
        valid_until_tick: super::fixtures::VALID_UNTIL_TICK,
    },
    crate::serving::CapacityDraft {
        kind: crate::serving::CapacityKind::RequestSlots,
        unit: crate::serving::CapacityUnit::Requests,
        scope: crate::serving::CapacityScope::Replica,
        origin: crate::serving::CapacityOrigin::HostProbe,
        value: crate::serving::Counter::Known(8),
        observed_tick: super::fixtures::OBSERVED_TICK,
        valid_until_tick: super::fixtures::VALID_UNTIL_TICK,
    },
];
const HOSTED_QUOTA: [crate::serving::CapacityDraft; 2] = [
    account_quota(
        crate::serving::CapacityKind::TokenQuota,
        crate::serving::Counter::Known(100_000),
    ),
    account_quota(
        crate::serving::CapacityKind::RequestQuota,
        crate::serving::Counter::Unknown,
    ),
];
const SMALL_TOKEN_QUOTA: [crate::serving::CapacityDraft; 1] = [account_quota(
    crate::serving::CapacityKind::TokenQuota,
    crate::serving::Counter::Known(1_000),
)];

fn owner_decides() -> crate::serving::Profile {
    super::fixtures::profile_with(
        crate::serving::CapacityRule::OwnerAdmissionDecides,
        crate::serving::ReplicaRule::SameModelAllowed,
    )
}

#[test]
fn fresh_ready_route_is_eligible_with_capacity_evidence() {
    let snapshot = super::fixtures::snapshot_with(|_| {});
    let screening = super::fixtures::screen(
        &super::fixtures::profile(),
        &snapshot,
        &[super::fixtures::candidate()],
    );
    assert_eq!(screening.status(), crate::serving::SnapshotStatus::Current);
    let route = &screening.eligible()[0];
    assert_eq!(route.route().route_name(), "primary");
    assert_eq!(route.deployment().deployment_generation(), 1);
    assert_eq!(route.attestation(), crate::serving::Attestation::Observed);
    assert_eq!(
        route.capacity(),
        crate::serving::CapacityStanding::Known {
            kind: crate::serving::CapacityKind::RequestSlots,
            value: 1
        }
    );
    assert!(!route.catalog_entry().is_local());
    assert_eq!(screening.eligible_catalog().len(), 1);
}

#[test]
fn model_listed_before_readiness_is_not_eligible() {
    for readiness in [
        crate::serving::Readiness::Loading,
        crate::serving::Readiness::Absent,
        crate::serving::Readiness::Unknown,
        crate::serving::Readiness::Draining,
        crate::serving::Readiness::Unavailable,
        crate::serving::Readiness::Stale,
    ] {
        let snapshot = super::fixtures::snapshot_with(|draft| draft.readiness = readiness);
        assert_eq!(
            super::fixtures::only_reason(&snapshot),
            crate::serving::ExclusionReason::NotReady(readiness)
        );
    }
    assert_eq!(crate::serving::Readiness::Loading.as_str(), "loading");
}

#[test]
fn load_and_drain_transitions_follow_the_newest_snapshot() {
    let mut loading = super::fixtures::service();
    loading.readiness = crate::serving::Readiness::Loading;
    let mut draining = super::fixtures::service();
    draining.readiness = crate::serving::Readiness::Draining;
    let sequence = |services: &[crate::serving::ServiceDraft<'_>], sequence: u64| {
        crate::serving::AvailabilitySnapshot::new(&crate::serving::SnapshotDraft {
            sequence,
            ..super::fixtures::snapshot_draft(services)
        })
        .expect("snapshot")
    };
    let profile = super::fixtures::profile();
    let candidates = [super::fixtures::candidate()];
    let loaded = sequence(&[loading], super::fixtures::SEQUENCE_ONE);
    let ready = sequence(&[super::fixtures::service()], super::fixtures::SEQUENCE_TWO);
    let drained = sequence(&[draining], super::fixtures::SEQUENCE_THREE);
    assert!(
        super::fixtures::screen(&profile, &loaded, &candidates)
            .eligible()
            .is_empty()
    );
    assert_eq!(
        super::fixtures::screen(&profile, &ready, &candidates)
            .eligible()
            .len(),
        1
    );
    assert!(
        super::fixtures::screen(&profile, &drained, &candidates)
            .eligible()
            .is_empty()
    );
}

#[test]
fn unknown_capacity_depends_on_the_profile_rule() {
    let snapshot = super::fixtures::snapshot_with(|draft| draft.capacity = &UNKNOWN_SLOTS);
    assert_eq!(
        super::fixtures::only_reason(&snapshot),
        crate::serving::ExclusionReason::CapacityUnknown
    );
    let empty = super::fixtures::snapshot_with(|draft| draft.capacity = &[]);
    assert_eq!(
        super::fixtures::only_reason(&empty),
        crate::serving::ExclusionReason::CapacityUnknown
    );
    let screening =
        super::fixtures::screen(&owner_decides(), &snapshot, &[super::fixtures::candidate()]);
    assert_eq!(
        screening.eligible()[0].capacity(),
        crate::serving::CapacityStanding::UnknownOwnerDecides
    );
}

#[test]
fn exhausted_and_stale_capacity_are_excluded() {
    let exhausted = super::fixtures::snapshot_with(|draft| draft.capacity = &EXHAUSTED);
    assert_eq!(
        super::fixtures::only_reason(&exhausted),
        crate::serving::ExclusionReason::CapacityExhausted
    );
    let decided = super::fixtures::screen(
        &owner_decides(),
        &exhausted,
        &[super::fixtures::candidate()],
    );
    assert!(decided.eligible().is_empty());
    let stale = super::fixtures::snapshot_with(|draft| draft.capacity = &EXPIRED_SLOTS);
    assert_eq!(
        super::fixtures::only_reason(&stale),
        crate::serving::ExclusionReason::CapacityStale
    );
}

#[test]
fn free_memory_and_host_probes_are_not_request_capacity() {
    let snapshot = super::fixtures::snapshot_with(|draft| draft.capacity = &FREE_MEMORY_ONLY);
    assert_eq!(
        super::fixtures::only_reason(&snapshot),
        crate::serving::ExclusionReason::CapacityUnknown
    );
    assert!(!crate::serving::CapacityKind::FreeMemory.gates_requests());
}

#[test]
fn hosted_quota_is_usable_without_gpu_facts() {
    let snapshot = super::fixtures::snapshot_with(|draft| {
        draft.locality = crate::serving::Locality::HostedRemote;
        draft.capacity = &HOSTED_QUOTA;
        draft.replica = None;
        draft.resource_consumer_ref = None;
    });
    let screening = super::fixtures::screen(
        &super::fixtures::profile(),
        &snapshot,
        &[super::fixtures::candidate()],
    );
    let route = &screening.eligible()[0];
    assert_eq!(
        route.capacity(),
        crate::serving::CapacityStanding::Known {
            kind: crate::serving::CapacityKind::TokenQuota,
            value: 100_000
        }
    );
    assert_eq!(route.replica(), &crate::serving::ReplicaFact::Unknown);
    assert_eq!(route.resource_consumer_ref(), None);
    let small = super::fixtures::snapshot_with(|draft| {
        draft.locality = crate::serving::Locality::HostedRemote;
        draft.capacity = &SMALL_TOKEN_QUOTA;
    });
    assert_eq!(
        super::fixtures::only_reason(&small),
        crate::serving::ExclusionReason::CapacityExhausted
    );
}

fn status_for(freshness: crate::serving::Freshness) -> crate::serving::Screening {
    let snapshot = super::fixtures::snapshot_with(|_| {});
    super::fixtures::screen_at(
        &super::fixtures::profile(),
        &snapshot,
        (freshness, crate::router::PrivacyBand::RemoteAllowed),
        &[super::fixtures::candidate()],
    )
}

#[test]
fn snapshot_freshness_is_host_supplied_and_fails_closed() {
    let at = |now_tick, newest_accepted| crate::serving::Freshness {
        now_tick,
        newest_accepted,
    };
    let now = Some(super::fixtures::NOW_TICK);
    let cases = [
        (
            at(None, None),
            crate::serving::SnapshotStatus::FreshnessUnknown,
        ),
        (
            at(Some(super::fixtures::BEFORE_OBSERVED_TICK), None),
            crate::serving::SnapshotStatus::FreshnessUnknown,
        ),
        (
            at(Some(super::fixtures::VALID_UNTIL_TICK), None),
            crate::serving::SnapshotStatus::Expired,
        ),
        (
            at(Some(super::fixtures::TOO_OLD_TICK), None),
            crate::serving::SnapshotStatus::TooOld,
        ),
        (
            at(
                now,
                Some((
                    super::fixtures::GENERATION_ONE,
                    super::fixtures::SEQUENCE_TWO,
                )),
            ),
            crate::serving::SnapshotStatus::OutOfOrder,
        ),
        (
            at(
                now,
                Some((
                    super::fixtures::GENERATION_TWO,
                    super::fixtures::SEQUENCE_ONE,
                )),
            ),
            crate::serving::SnapshotStatus::OutOfOrder,
        ),
    ];
    for (freshness, expected) in cases {
        let screening = status_for(freshness);
        assert_eq!(screening.status(), expected);
        assert_eq!(
            screening.exclusions()[0].reason(),
            crate::serving::ExclusionReason::Snapshot(expected)
        );
    }
    let same = at(
        now,
        Some((
            super::fixtures::GENERATION_ONE,
            super::fixtures::SEQUENCE_ONE,
        )),
    );
    assert_eq!(
        status_for(same).status(),
        crate::serving::SnapshotStatus::Current
    );
}

/// Snapshot whose service serves `(model_id, configuration_ref)`.
fn with_model(model: (&'static str, &'static str)) -> crate::serving::AvailabilitySnapshot {
    super::fixtures::snapshot_with(|draft| {
        draft.model = Some(crate::serving::ModelDraft {
            model_id: model.0,
            configuration_ref: model.1,
            attestation: crate::serving::Attestation::Observed,
        });
    })
}

#[test]
fn wrong_model_or_configuration_is_excluded() {
    assert_eq!(
        super::fixtures::only_reason(&with_model((
            super::fixtures::MODEL_B,
            super::fixtures::CONFIG_A
        ))),
        crate::serving::ExclusionReason::ModelMismatch
    );
    assert_eq!(
        super::fixtures::only_reason(&with_model((
            super::fixtures::MODEL_A,
            super::fixtures::CONFIG_B
        ))),
        crate::serving::ExclusionReason::ConfigurationMismatch
    );
    let unknown = super::fixtures::snapshot_with(|draft| draft.model = None);
    assert_eq!(
        super::fixtures::only_reason(&unknown),
        crate::serving::ExclusionReason::ModelUnknown
    );
}

#[test]
fn claimed_identity_needs_an_accepting_profile() {
    let claimed = super::fixtures::snapshot_with(|draft| {
        draft.model = Some(crate::serving::ModelDraft {
            model_id: super::fixtures::MODEL_A,
            configuration_ref: super::fixtures::CONFIG_A,
            attestation: crate::serving::Attestation::Claimed,
        });
    });
    assert_eq!(
        super::fixtures::only_reason(&claimed),
        crate::serving::ExclusionReason::ModelOnlyClaimed
    );
    let accepting = crate::serving::Profile::new(crate::serving::ProfileDraft {
        profile_id: "claims-accepted",
        identity: crate::serving::IdentityRule::ClaimedAccepted,
        capacity: crate::serving::CapacityRule::KnownRequired,
        replica: crate::serving::ReplicaRule::SameModelAllowed,
        max_snapshot_age_ticks: super::fixtures::MAX_AGE_TICKS,
    })
    .expect("profile");
    let screening = super::fixtures::screen(&accepting, &claimed, &[super::fixtures::candidate()]);
    assert_eq!(
        screening.eligible()[0].attestation(),
        crate::serving::Attestation::Claimed
    );
}

#[test]
fn local_only_requests_need_a_local_service() {
    let local_only = (
        super::fixtures::fresh(),
        crate::router::PrivacyBand::LocalOnly,
    );
    let profile = super::fixtures::profile();
    let candidates = [super::fixtures::candidate()];
    let remote = super::fixtures::snapshot_with(|_| {});
    let denied = super::fixtures::screen_at(&profile, &remote, local_only, &candidates);
    assert_eq!(
        denied.exclusions()[0].reason(),
        crate::serving::ExclusionReason::PrivacyDenied
    );
    let local =
        super::fixtures::snapshot_with(|draft| draft.locality = crate::serving::Locality::Local);
    let admitted = super::fixtures::screen_at(&profile, &local, local_only, &candidates);
    assert!(admitted.eligible()[0].catalog_entry().is_local());
}

#[test]
fn capability_envelope_and_admission_offers_are_preserved() {
    let cases: [(ServiceEdit, crate::serving::ExclusionReason); 6] = [
        (
            |draft| draft.capabilities.tools = crate::mesh::Capability::Unsupported,
            crate::serving::ExclusionReason::CapabilityUnsupported,
        ),
        (
            |draft| draft.capabilities.tools = crate::mesh::Capability::Unknown,
            crate::serving::ExclusionReason::CapabilityUnknown,
        ),
        (
            |draft| draft.envelope = None,
            crate::serving::ExclusionReason::EnvelopeUnknown,
        ),
        (
            |draft| draft.envelope = crate::serving::Envelope::new(2_048, 256).ok(),
            crate::serving::ExclusionReason::EnvelopeExceeded,
        ),
        (
            |draft| draft.admission = crate::serving::AdmissionOffer::NotOffered,
            crate::serving::ExclusionReason::AdmissionNotOffered,
        ),
        (
            |draft| draft.service_id = super::fixtures::SERVICE_B,
            crate::serving::ExclusionReason::ServiceAbsent,
        ),
    ];
    for (edit, expected) in cases {
        let snapshot = super::fixtures::snapshot_with(edit);
        assert_eq!(super::fixtures::only_reason(&snapshot), expected);
    }
    assert_eq!(
        crate::serving::ExclusionReason::ServiceAbsent.as_str(),
        "service-absent"
    );
}

type ServiceEdit = fn(&mut crate::serving::ServiceDraft<'static>);

fn two_route_world() -> (
    crate::serving::AvailabilitySnapshot,
    [crate::serving::RouteCandidate; 2],
) {
    let mut draining = super::fixtures::service();
    draining.service_id = super::fixtures::SERVICE_B;
    draining.readiness = crate::serving::Readiness::Draining;
    let snapshot = super::fixtures::snapshot_of(&[super::fixtures::service(), draining]);
    let secondary = super::fixtures::candidate_with(|draft| {
        draft.route_name = "secondary";
        draft.label = super::fixtures::LABEL_SECONDARY;
        draft.service_id = super::fixtures::SERVICE_B;
    });
    (snapshot, [secondary, super::fixtures::candidate()])
}

#[test]
fn learned_preference_cannot_admit_an_ineligible_route() {
    let (snapshot, candidates) = two_route_world();
    let screening = super::fixtures::screen(&super::fixtures::profile(), &snapshot, &candidates);
    assert_eq!(screening.exclusions()[0].route_name(), "secondary");
    let chosen = crate::serving::rank_eligible(
        &screening,
        &[
            super::fixtures::LABEL_SECONDARY,
            super::fixtures::LABEL_PRIMARY,
        ],
    )
    .expect("eligible");
    assert_eq!(
        chosen.catalog_entry().label(),
        super::fixtures::LABEL_PRIMARY
    );
    assert!(
        crate::serving::rank_eligible(
            &screening,
            &[super::fixtures::LABEL_SECONDARY, "model:unlisted"]
        )
        .is_none()
    );
}

fn single_recipe() -> crate::router::Recipe {
    crate::router::Recipe::new(crate::router::RecipeDraft {
        id: "recipe:single",
        family: "single",
        roles: &["primary"],
        output_contract: "text",
        max_candidates: 1,
        max_parallelism: 1,
        min_success_quorum: 1,
        timeout_class: "standard",
        max_synthesis_attempts: 0,
        max_repair_attempts: Some(0),
        fallback_policy: "deny",
    })
    .expect("recipe")
}

fn simple_signals() -> crate::router::Signals {
    crate::router::Signals::project(crate::router::SignalDraft {
        task_shape: "simple",
        risk_band: "low",
        contract_pressure: "low",
        context_pressure: "low",
        latency_band: "low",
        cost_band: "low",
        privacy_band: "remote_allowed",
        fallback_eligibility: "ineligible",
    })
    .expect("signals")
}

#[test]
fn existing_recipe_selection_sees_only_eligible_routes() {
    let (snapshot, candidates) = two_route_world();
    let screening = super::fixtures::screen(&super::fixtures::profile(), &snapshot, &candidates);
    let catalog = screening.eligible_catalog();
    let budget = crate::router::Budget::new(1, &[super::fixtures::BACKEND_REMOTE]).expect("budget");
    let plan =
        crate::router::select_recipe(&simple_signals(), &[single_recipe()], &catalog, &budget)
            .expect("plan");
    assert_eq!(plan.selected_models(), [super::fixtures::LABEL_PRIMARY]);
    let denied = crate::router::Budget::new(1, &["backend:other"]).expect("budget");
    assert_eq!(
        crate::router::select_recipe(&simple_signals(), &[single_recipe()], &catalog, &denied),
        Err(crate::router::SelectionDenial::NoEligibleModel)
    );
}

#[test]
fn one_agent_binds_several_named_model_services() {
    let mut second = super::fixtures::service();
    second.service_id = super::fixtures::SERVICE_B;
    second.model = Some(crate::serving::ModelDraft {
        model_id: super::fixtures::MODEL_B,
        configuration_ref: super::fixtures::CONFIG_B,
        attestation: crate::serving::Attestation::Observed,
    });
    let snapshot = super::fixtures::snapshot_of(&[super::fixtures::service(), second]);
    let reviewer = super::fixtures::candidate_with(|draft| {
        draft.route_name = "reviewer";
        draft.route_ref = None;
        draft.label = super::fixtures::LABEL_SECONDARY;
        draft.role = "judge";
        draft.service_id = super::fixtures::SERVICE_B;
        draft.model_id = super::fixtures::MODEL_B;
        draft.configuration_ref = super::fixtures::CONFIG_B;
    });
    let screening = super::fixtures::screen(
        &super::fixtures::profile(),
        &snapshot,
        &[super::fixtures::candidate(), reviewer],
    );
    assert_eq!(screening.eligible().len(), 2);
    assert_eq!(screening.eligible()[1].route().route_ref(), None);
    assert_eq!(
        screening.eligible()[0].route().route_ref(),
        Some(super::fixtures::ROUTE_REF)
    );
}

fn candidate_issue(candidates: &[crate::serving::RouteCandidate]) -> crate::serving::ContractIssue {
    let snapshot = super::fixtures::snapshot_with(|_| {});
    crate::serving::screen_routes(&crate::serving::ScreenRequest {
        profile: &super::fixtures::profile(),
        snapshot: &snapshot,
        freshness: super::fixtures::fresh(),
        privacy: crate::router::PrivacyBand::RemoteAllowed,
        candidates,
    })
    .expect_err("invalid candidate list")
}

#[test]
fn candidate_lists_are_bounded_and_unique() {
    assert_eq!(
        candidate_issue(&[]),
        crate::serving::ContractIssue::Empty {
            field: "candidates"
        }
    );
    assert_eq!(
        candidate_issue(&[super::fixtures::candidate(), super::fixtures::candidate()]),
        crate::serving::ContractIssue::Duplicate {
            field: "route_name"
        }
    );
    let same_label = super::fixtures::candidate_with(|draft| draft.route_name = "other");
    assert_eq!(
        candidate_issue(&[super::fixtures::candidate(), same_label]),
        crate::serving::ContractIssue::Duplicate {
            field: "router_model_label"
        }
    );
    let many: alloc::vec::Vec<_> = (0..=crate::serving::MAX_ROUTE_CANDIDATES)
        .map(|_| super::fixtures::candidate())
        .collect();
    assert_eq!(
        candidate_issue(&many),
        crate::serving::ContractIssue::TooMany {
            field: "candidates"
        }
    );
}
