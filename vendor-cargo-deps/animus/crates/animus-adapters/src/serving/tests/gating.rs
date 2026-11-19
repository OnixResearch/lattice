//! Authoritative serving-owner admission and current recheck before dispatch.

fn query_for(request_id: &str) -> crate::serving::AdmissionQuery {
    let snapshot = super::fixtures::snapshot_with(|_| {});
    let screening = super::fixtures::screen(
        &super::fixtures::profile(),
        &snapshot,
        &[super::fixtures::candidate()],
    );
    crate::serving::admission_query(
        &screening.eligible()[0],
        &super::fixtures::caller(),
        request_id,
    )
    .expect("query")
}

fn grant(
    edit: impl FnOnce(&mut crate::serving::GrantDraft<'static>),
) -> crate::serving::OwnerReply {
    let mut draft = super::fixtures::grant_draft();
    edit(&mut draft);
    crate::serving::OwnerReply::granted(crate::serving::Grant::new(&draft).expect("grant"))
}

fn verdict(
    profile: &crate::serving::Profile,
    owner: &crate::serving::OwnerReply,
    current: crate::serving::CurrentReading<'_>,
    existing: crate::mesh::DispatchAdmission,
) -> crate::serving::GateVerdict {
    let query = query_for(super::fixtures::REQUEST_ONE);
    crate::serving::gate_dispatch(&crate::serving::GateInput {
        profile,
        query: &query,
        owner,
        current,
        existing,
    })
}

fn verdict_with_current(
    edit: impl FnOnce(&mut crate::serving::CurrentState<'static>),
) -> crate::serving::GateVerdict {
    let mut state = super::fixtures::current();
    edit(&mut state);
    verdict(
        &super::fixtures::profile(),
        &grant(|_| {}),
        crate::serving::CurrentReading::Observed(state),
        super::fixtures::existing(),
    )
}

fn verdict_with_grant(
    profile: &crate::serving::Profile,
    edit: impl FnOnce(&mut crate::serving::GrantDraft<'static>),
) -> crate::serving::GateVerdict {
    verdict(
        profile,
        &grant(edit),
        crate::serving::CurrentReading::Observed(super::fixtures::current()),
        super::fixtures::existing(),
    )
}

fn admitted(verdict: crate::serving::GateVerdict) -> crate::serving::DispatchPermit {
    match verdict {
        crate::serving::GateVerdict::Admitted(permit) => permit,
        crate::serving::GateVerdict::Rejected(_) | crate::serving::GateVerdict::Blocked(_) => {
            unreachable!("fixture dispatch must be admitted: {verdict:?}")
        }
    }
}

/// Deterministic serving owner with a fixed number of request slots.
struct SlotOwner {
    free_slots: u32,
    issued: u32,
}

impl SlotOwner {
    fn admit(&mut self, query: &crate::serving::AdmissionQuery) -> crate::serving::OwnerReply {
        let Some(remaining) = self.free_slots.checked_sub(1) else {
            return crate::serving::OwnerReply::Rejected(crate::serving::OwnerRejection::Saturated);
        };
        self.free_slots = remaining;
        self.issued = self.issued.saturating_add(1);
        let grant_ref = alloc::format!("grant-{}", self.issued);
        crate::serving::OwnerReply::granted(
            crate::serving::Grant::new(&crate::serving::GrantDraft {
                grant_ref: grant_ref.as_str(),
                request_id: query.request_id(),
                ..super::fixtures::grant_draft()
            })
            .expect("grant"),
        )
    }
}

#[test]
fn granted_current_request_is_admitted() {
    let permit = admitted(verdict_with_grant(&super::fixtures::profile(), |_| {}));
    assert_eq!(permit.grant_ref(), super::fixtures::GRANT_ONE);
    assert_eq!(permit.admission_epoch(), super::fixtures::EPOCH_ONE);
    assert!(!permit.is_replica_changed());
}

#[test]
fn two_callers_observing_the_last_slot_cannot_both_dispatch() {
    let snapshot = super::fixtures::snapshot_with(|_| {});
    let profile = super::fixtures::profile();
    let first = super::fixtures::screen(&profile, &snapshot, &[super::fixtures::candidate()]);
    let second = super::fixtures::screen(&profile, &snapshot, &[super::fixtures::candidate()]);
    assert_eq!(
        first.eligible()[0].capacity(),
        second.eligible()[0].capacity()
    );
    let mut owner = SlotOwner {
        free_slots: 1,
        issued: 0,
    };
    let mut verdicts = alloc::vec::Vec::with_capacity(2);
    for (screening, request_id) in [
        (&first, super::fixtures::REQUEST_ONE),
        (&second, super::fixtures::REQUEST_TWO),
    ] {
        let query = crate::serving::admission_query(
            &screening.eligible()[0],
            &super::fixtures::caller(),
            request_id,
        )
        .expect("query");
        let reply = owner.admit(&query);
        verdicts.push(crate::serving::gate_dispatch(&crate::serving::GateInput {
            profile: &profile,
            query: &query,
            owner: &reply,
            current: crate::serving::CurrentReading::Observed(super::fixtures::current()),
            existing: super::fixtures::existing(),
        }));
    }
    assert!(matches!(
        verdicts[0],
        crate::serving::GateVerdict::Admitted(_)
    ));
    assert_eq!(
        verdicts[1],
        crate::serving::GateVerdict::Rejected(crate::serving::GateRejection::OwnerRejected(
            crate::serving::OwnerRejection::Saturated
        ))
    );
}

#[test]
fn unknown_capacity_still_needs_the_owner_grant() {
    let decides = super::fixtures::profile_with(
        crate::serving::CapacityRule::OwnerAdmissionDecides,
        crate::serving::ReplicaRule::SameModelAllowed,
    );
    let rejected =
        crate::serving::OwnerReply::Rejected(crate::serving::OwnerRejection::QuotaExhausted);
    assert_eq!(
        verdict(
            &decides,
            &rejected,
            crate::serving::CurrentReading::Observed(super::fixtures::current()),
            super::fixtures::existing(),
        ),
        crate::serving::GateVerdict::Rejected(crate::serving::GateRejection::OwnerRejected(
            crate::serving::OwnerRejection::QuotaExhausted
        ))
    );
    assert_eq!(
        crate::serving::OwnerRejection::QuotaExhausted.as_str(),
        "quota-exhausted"
    );
}

#[test]
fn stale_or_revoked_grants_block_dispatch() {
    assert_eq!(
        verdict_with_current(|state| state.now_tick = Some(super::fixtures::GRANT_VALID_UNTIL_TICK)),
        crate::serving::GateVerdict::Rejected(crate::serving::GateRejection::GrantExpired)
    );
    assert_eq!(
        verdict_with_current(|state| state.admission_epoch = super::fixtures::EPOCH_TWO),
        crate::serving::GateVerdict::Rejected(crate::serving::GateRejection::GrantRevoked)
    );
}

#[test]
fn foreign_or_replaced_grants_block_dispatch() {
    let profile = super::fixtures::profile();
    let cases: [(GrantEdit, crate::serving::GateRejection); 3] = [
        (
            |draft| draft.request_id = super::fixtures::REQUEST_TWO,
            crate::serving::GateRejection::GrantForOtherRequest,
        ),
        (
            |draft| draft.service_id = super::fixtures::SERVICE_B,
            crate::serving::GateRejection::GrantTargetMismatch,
        ),
        (
            |draft| draft.deployment_generation = super::fixtures::GENERATION_TWO,
            crate::serving::GateRejection::GenerationReplaced,
        ),
    ];
    for (edit, expected) in cases {
        assert_eq!(
            verdict_with_grant(&profile, edit),
            crate::serving::GateVerdict::Rejected(expected)
        );
    }
    assert_eq!(
        profile.replica_rule(),
        crate::serving::ReplicaRule::SameModelAllowed
    );
}

type GrantEdit = fn(&mut crate::serving::GrantDraft<'static>);

#[test]
fn unavailable_admission_or_recheck_blocks_dispatch() {
    let profile = super::fixtures::profile();
    let observed = crate::serving::CurrentReading::Observed(super::fixtures::current());
    let cases = [
        (
            crate::serving::OwnerReply::Unavailable,
            observed,
            crate::serving::GateBlock::AdmissionUnavailable,
        ),
        (
            crate::serving::OwnerReply::PortIssue,
            observed,
            crate::serving::GateBlock::AdmissionPortIssue,
        ),
        (
            grant(|_| {}),
            crate::serving::CurrentReading::Unavailable,
            crate::serving::GateBlock::RecheckUnavailable,
        ),
        (
            grant(|_| {}),
            crate::serving::CurrentReading::PortIssue,
            crate::serving::GateBlock::RecheckPortIssue,
        ),
    ];
    for (owner, current, expected) in cases {
        assert_eq!(
            verdict(&profile, &owner, current, super::fixtures::existing()),
            crate::serving::GateVerdict::Blocked(expected)
        );
    }
    assert_eq!(
        verdict_with_current(|state| state.now_tick = None),
        crate::serving::GateVerdict::Blocked(crate::serving::GateBlock::ClockUnknown)
    );
}

#[test]
fn existing_privacy_capability_and_budget_decisions_still_gate() {
    let mut over_budget = super::fixtures::existing();
    over_budget.budget_admitted = false;
    let mut private = super::fixtures::existing();
    private.privacy_admitted = false;
    let mut capability = super::fixtures::existing();
    capability.exact_model = crate::mesh::ExactModelDecision::CapabilityDenied;
    let cases = [
        (over_budget, crate::mesh::DispatchDecision::PolicyDenied),
        (private, crate::mesh::DispatchDecision::PolicyDenied),
        (capability, crate::mesh::DispatchDecision::ExactModelDenied),
    ];
    let observed = crate::serving::CurrentReading::Observed(super::fixtures::current());
    for (facts, expected) in cases {
        let refused = verdict(&super::fixtures::profile(), &grant(|_| {}), observed, facts);
        assert_eq!(
            refused,
            crate::serving::GateVerdict::Rejected(crate::serving::GateRejection::ExistingPolicy(
                expected
            ))
        );
    }
    assert_eq!(
        crate::mesh::admit_dispatch(&super::fixtures::existing()),
        crate::mesh::DispatchDecision::Admitted
    );
}

#[test]
fn replacement_between_selection_and_dispatch_is_rejected() {
    let cases: [(CurrentEdit, crate::serving::GateRejection); 6] = [
        (
            |state| state.deployment_generation = super::fixtures::GENERATION_TWO,
            crate::serving::GateRejection::GenerationReplaced,
        ),
        (
            |state| state.deployment_id = "deploy-b",
            crate::serving::GateRejection::GenerationReplaced,
        ),
        (
            |state| state.endpoint_identity = "endpoint:b",
            crate::serving::GateRejection::EndpointReplaced,
        ),
        (
            |state| state.model_id = super::fixtures::MODEL_B,
            crate::serving::GateRejection::ModelSubstituted,
        ),
        (
            |state| state.configuration_ref = super::fixtures::CONFIG_B,
            crate::serving::GateRejection::ConfigurationChanged,
        ),
        (
            |state| state.readiness = crate::serving::Readiness::Draining,
            crate::serving::GateRejection::NotReadyAtDispatch(crate::serving::Readiness::Draining),
        ),
    ];
    for (edit, expected) in cases {
        assert_eq!(
            verdict_with_current(edit),
            crate::serving::GateVerdict::Rejected(expected)
        );
    }
    assert_eq!(
        super::fixtures::current().endpoint_identity,
        super::fixtures::ENDPOINT_A
    );
}

type CurrentEdit = fn(&mut crate::serving::CurrentState<'static>);

#[test]
fn unauthorized_model_substitution_in_the_grant_is_rejected() {
    let profile = super::fixtures::profile();
    assert_eq!(
        verdict_with_grant(&profile, |draft| draft.model_id = super::fixtures::MODEL_B),
        crate::serving::GateVerdict::Rejected(crate::serving::GateRejection::ModelSubstituted)
    );
    assert_eq!(
        verdict_with_grant(&profile, |draft| draft.configuration_ref =
            super::fixtures::CONFIG_B),
        crate::serving::GateVerdict::Rejected(crate::serving::GateRejection::ConfigurationChanged)
    );
}

#[test]
fn same_model_replica_change_follows_the_profile() {
    let moved = |draft: &mut crate::serving::GrantDraft<'static>| {
        draft.replica = Some(super::fixtures::REPLICA_TWO);
    };
    let permit = admitted(verdict_with_grant(&super::fixtures::profile(), moved));
    assert!(permit.is_replica_changed());
    assert_eq!(
        permit.replica(),
        &crate::serving::ReplicaFact::new(Some(super::fixtures::REPLICA_TWO)).expect("replica")
    );
    let pinned = super::fixtures::profile_with(
        crate::serving::CapacityRule::KnownRequired,
        crate::serving::ReplicaRule::Pinned,
    );
    assert_eq!(
        verdict_with_grant(&pinned, moved),
        crate::serving::GateVerdict::Rejected(crate::serving::GateRejection::ReplicaChangeDenied)
    );
    let opaque = admitted(verdict_with_grant(&pinned, |draft| draft.replica = None));
    assert_eq!(opaque.replica(), &crate::serving::ReplicaFact::Unknown);
    assert!(!opaque.is_replica_changed());
}
