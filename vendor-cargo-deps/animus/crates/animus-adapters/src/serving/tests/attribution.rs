//! Metadata-only receipts and usage provenance.

const REPORTED: crate::serving::Usage = crate::serving::Usage {
    input_tokens: crate::serving::Counter::Known(1_812),
    output_tokens: crate::serving::Counter::Known(240),
    origin: crate::serving::UsageOrigin::ProviderReported,
};

struct Dispatched {
    screening: crate::serving::Screening,
    permit: crate::serving::DispatchPermit,
    admission: crate::serving::AdmissionRecord,
}

fn dispatched(replica: Option<&'static str>) -> Dispatched {
    let snapshot = super::fixtures::snapshot_with(|_| {});
    let screening = super::fixtures::screen(
        &super::fixtures::profile(),
        &snapshot,
        &[super::fixtures::candidate()],
    );
    let query = crate::serving::admission_query(
        &screening.eligible()[0],
        &super::fixtures::caller(),
        super::fixtures::REQUEST_ONE,
    )
    .expect("query");
    let owner = crate::serving::OwnerReply::granted(
        crate::serving::Grant::new(&crate::serving::GrantDraft {
            replica,
            ..super::fixtures::grant_draft()
        })
        .expect("grant"),
    );
    let verdict = crate::serving::gate_dispatch(&crate::serving::GateInput {
        profile: &super::fixtures::profile(),
        query: &query,
        owner: &owner,
        current: crate::serving::CurrentReading::Observed(super::fixtures::current()),
        existing: super::fixtures::existing(),
    });
    let crate::serving::GateVerdict::Admitted(permit) = verdict else {
        unreachable!("fixture dispatch is admitted: {verdict:?}");
    };
    assert_eq!(permit.grant_ref(), super::fixtures::GRANT_ONE);
    assert_eq!(screening.eligible().len(), 1);
    Dispatched {
        screening,
        permit,
        admission: crate::serving::AdmissionRecord::from_reply(&owner),
    }
}

fn completed(
    state: &Dispatched,
    caller: &crate::serving::CallerBinding,
    evidence: crate::serving::EvidenceLabel,
) -> Result<crate::serving::InvocationReceipt, crate::serving::ContractIssue> {
    let query = crate::serving::admission_query(
        &state.screening.eligible()[0],
        caller,
        super::fixtures::REQUEST_ONE,
    )
    .expect("query");
    let facts = crate::serving::ResponseFacts {
        usage: REPORTED,
        policy_generation: None,
    };
    let response =
        crate::serving::response_provenance(&query, Some(super::fixtures::MODEL_A), &facts);
    assert_eq!(response.rejection(), None);
    assert_eq!(query.request_id(), super::fixtures::REQUEST_ONE);
    crate::serving::project_receipt(&crate::serving::ReceiptInput {
        evidence,
        request_id: super::fixtures::REQUEST_ONE,
        caller,
        screening: &state.screening,
        selected: state.screening.eligible().first(),
        admission: &state.admission,
        permit: Some(&state.permit),
        outcome: crate::serving::DispatchOutcome::Completed,
        response: Some(response),
        usage: REPORTED,
        provider_receipt_identity: Some("mesh-receipt-1"),
    })
}

#[test]
fn completed_receipt_keys_usage_to_the_named_route() {
    let state = dispatched(Some(super::fixtures::REPLICA_ONE));
    let fixture = crate::serving::EvidenceLabel::Fixture;
    let receipt = completed(&state, &super::fixtures::caller(), fixture).expect("receipt");
    let route = receipt.selected().expect("selected route");
    assert_eq!(route.route().route_name(), "primary");
    assert_eq!(route.route().route_ref(), Some(super::fixtures::ROUTE_REF));
    assert_eq!(route.deployment().deployment_generation(), 1);
    assert_eq!(route.model().model_id(), super::fixtures::MODEL_A);
    assert_eq!(receipt.usage(), REPORTED);
    assert_eq!(receipt.caller().attempt_id(), "attempt-1");
    assert_eq!(receipt.provider_receipt_identity(), Some("mesh-receipt-1"));
    assert_eq!(receipt.screened_counts(), (1, 0));
    assert!(receipt.identity().starts_with("blake3:"));
    let again = completed(&state, &super::fixtures::caller(), fixture).expect("again");
    assert_eq!(receipt.identity(), again.identity());
    let live = crate::serving::EvidenceLabel::LiveEndpoint;
    let labelled = completed(&state, &super::fixtures::caller(), live).expect("live label");
    assert_ne!(receipt.identity(), labelled.identity());
}

#[test]
fn replica_change_keeps_agent_identity_and_records_new_evidence() {
    let fixture = crate::serving::EvidenceLabel::Fixture;
    let original = completed(
        &dispatched(Some(super::fixtures::REPLICA_ONE)),
        &super::fixtures::caller(),
        fixture,
    )
    .expect("receipt");
    let moved = completed(
        &dispatched(Some(super::fixtures::REPLICA_TWO)),
        &super::fixtures::caller(),
        fixture,
    )
    .expect("receipt");
    assert_eq!(original.caller(), moved.caller());
    assert!(!original.is_replica_changed());
    assert!(moved.is_replica_changed());
    assert_ne!(original.identity(), moved.identity());
}

#[test]
fn sessions_sharing_a_deployment_have_separate_usage() {
    let state = dispatched(Some(super::fixtures::REPLICA_ONE));
    let other_session = crate::serving::CallerBinding::new(crate::serving::CallerDraft {
        agent: "agent-2",
        session: "session-2",
        attempt: "attempt-2",
    })
    .expect("caller");
    let fixture = crate::serving::EvidenceLabel::Fixture;
    let first = completed(&state, &super::fixtures::caller(), fixture).expect("first");
    let second = completed(&state, &other_session, fixture).expect("second");
    let first_route = first.selected().expect("route");
    let second_route = second.selected().expect("route");
    assert_eq!(first_route.deployment(), second_route.deployment());
    assert_eq!(
        first_route.resource_consumer_ref(),
        Some(super::fixtures::SERVING_CONSUMER)
    );
    assert_ne!(first.identity(), second.identity());
    assert_ne!(first.caller().session_id(), second.caller().session_id());
}

fn base_input<'a>(
    state: &'a Dispatched,
    caller: &'a crate::serving::CallerBinding,
) -> crate::serving::ReceiptInput<'a> {
    crate::serving::ReceiptInput {
        evidence: crate::serving::EvidenceLabel::Fixture,
        request_id: super::fixtures::REQUEST_ONE,
        caller,
        screening: &state.screening,
        selected: state.screening.eligible().first(),
        admission: &crate::serving::AdmissionRecord::NotRequested,
        permit: None,
        outcome: crate::serving::DispatchOutcome::NotDispatched(
            crate::serving::NotDispatched::NoEligibleRoute,
        ),
        response: None,
        usage: crate::serving::Usage::UNKNOWN,
        provider_receipt_identity: None,
    }
}

#[test]
fn refusal_before_dispatch_is_distinct_from_uncertain_effects() {
    let state = dispatched(None);
    let caller = super::fixtures::caller();
    let blocked = crate::serving::project_receipt(&crate::serving::ReceiptInput {
        admission: &crate::serving::AdmissionRecord::Unavailable,
        outcome: crate::serving::DispatchOutcome::NotDispatched(
            crate::serving::NotDispatched::Blocked(crate::serving::GateBlock::AdmissionUnavailable),
        ),
        ..base_input(&state, &caller)
    })
    .expect("blocked receipt");
    assert_eq!(blocked.usage().origin, crate::serving::UsageOrigin::Unknown);
    let uncertain = crate::serving::project_receipt(&crate::serving::ReceiptInput {
        admission: &state.admission,
        permit: Some(&state.permit),
        outcome: crate::serving::DispatchOutcome::Uncertain(
            crate::serving::UncertainCause::TimedOut,
        ),
        ..base_input(&state, &caller)
    })
    .expect("uncertain receipt");
    assert!(!blocked.outcome().is_dispatched());
    assert!(uncertain.outcome().is_dispatched());
    assert_eq!(
        uncertain.usage().input_tokens,
        crate::serving::Counter::Unknown
    );
}

#[test]
fn contradictory_inputs_are_rejected() {
    let state = dispatched(None);
    let caller = super::fixtures::caller();
    let base = base_input(&state, &caller);
    assert!(crate::serving::project_receipt(&base).is_ok());
    let cases = [
        crate::serving::ReceiptInput {
            usage: REPORTED,
            ..base
        },
        crate::serving::ReceiptInput {
            provider_receipt_identity: Some("mesh-receipt-1"),
            ..base
        },
        crate::serving::ReceiptInput {
            permit: Some(&state.permit),
            ..base
        },
        crate::serving::ReceiptInput {
            outcome: crate::serving::DispatchOutcome::Completed,
            ..base
        },
        crate::serving::ReceiptInput {
            outcome: crate::serving::DispatchOutcome::Uncertain(
                crate::serving::UncertainCause::Cancelled,
            ),
            permit: Some(&state.permit),
            ..base
        },
        crate::serving::ReceiptInput {
            outcome: crate::serving::DispatchOutcome::Completed,
            permit: Some(&state.permit),
            admission: &state.admission,
            response: Some(crate::serving::ResponseProvenance {
                model: crate::mesh::ResponseModelDecision::Substituted,
                policy_generation: crate::serving::PolicyGenerationProvenance::NotRequired,
            }),
            ..base
        },
    ];
    for input in cases {
        assert_eq!(
            crate::serving::project_receipt(&input),
            Err(crate::serving::ContractIssue::InconsistentReceipt)
        );
    }
    assert_eq!(
        crate::serving::project_receipt(&crate::serving::ReceiptInput {
            request_id: "request with spaces",
            ..base
        }),
        Err(crate::serving::ContractIssue::UnsafeToken {
            field: "request_id"
        })
    );
}
