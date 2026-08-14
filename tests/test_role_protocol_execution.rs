mod common;

use lattice::engine::protocol::ActionRequest;
use lattice::engine::protocol::AttemptState;
use lattice::engine::protocol::Error;
use lattice::engine::protocol::Issue;
use lattice::engine::protocol::RoleProgram;
use lattice::engine::protocol::SessionStatus;
use lattice::engine::protocol::abandon_unknown;
use lattice::engine::protocol::admit_observation;
use lattice::engine::protocol::create_session;
use lattice::engine::protocol::identity;
use lattice::engine::protocol::inspect;
use lattice::engine::protocol::mark_dispatched_unknown;
use lattice::engine::protocol::plan_action;
use lattice::engine::protocol::plan_session;
use lattice::engine::protocol::prepare_attempt;
use lattice::engine::protocol::record_authorization;
use lattice::engine::protocol::replay;
use lattice::engine::protocol::retry_unknown;
use lattice::engine::protocol::safe_diagnostic_text;
use lattice::engine::protocol::settle_attempt;

fn session_for(
    source: choregraph_choreography::RoleChoreographySource,
    external: bool,
) -> lattice::engine::protocol::SessionRecord {
    let cohort = common::protocol::cohort(source);
    let placements = if external {
        common::protocol::external_placements(&cohort)
    } else {
        common::protocol::native_placements(&cohort)
    };
    let plan = match plan_session(cohort, placements, common::protocol::identity("session-nonce")) {
        Ok(plan) => plan,
        Err(error) => panic!("session plan failed: {error:?}"),
    };
    match create_session(plan) {
        Ok(session) => session,
        Err(error) => panic!("session creation failed: {error:?}"),
    }
}

fn prepared_transfer() -> lattice::engine::protocol::SessionRecord {
    let session = session_for(
        common::protocol::native_source(common::protocol::transfer(
            common::protocol::ROLE_A,
            common::protocol::ROLE_B,
            common::protocol::LABEL_A,
            common::protocol::end(),
        )),
        false,
    );
    let action = match plan_action(&session, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: lattice::engine::protocol::LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("payload"),
    }) {
        Ok(action) => action,
        Err(error) => panic!("action plan failed: {error:?}"),
    };
    match prepare_attempt(&session, action) {
        Ok(prepared) => prepared,
        Err(error) => panic!("attempt prepare failed: {error:?}"),
    }
}

fn authorized(prepared: &lattice::engine::protocol::SessionRecord) -> lattice::engine::protocol::SessionRecord {
    let attempt = match prepared.attempts.last() {
        Some(attempt) => attempt,
        None => panic!("prepared fixture has no attempt"),
    };
    match record_authorization(prepared, &attempt.identity, vec![common::protocol::identity("authority-attempt")]) {
        Ok(session) => session,
        Err(error) => panic!("authorization record failed: {error:?}"),
    }
}

fn unknown(prepared: &lattice::engine::protocol::SessionRecord) -> lattice::engine::protocol::SessionRecord {
    let authorized = authorized(prepared);
    let attempt = match authorized.attempts.last() {
        Some(attempt) => attempt.identity.clone(),
        None => panic!("authorized fixture has no attempt"),
    };
    match mark_dispatched_unknown(&authorized, &attempt) {
        Ok(session) => session,
        Err(error) => panic!("dispatch state failed: {error:?}"),
    }
}

// r[verify lattice.role_protocol.intake.valid]
// r[verify lattice.role_protocol.placement.valid]
// r[verify lattice.role_protocol.session_identity.changed]
#[test]
fn frozen_cohort_and_complete_placement_create_exact_distinct_sessions() {
    let cohort = common::protocol::cohort(common::protocol::native_source(common::protocol::end()));
    let first = plan_session(
        cohort.clone(),
        common::protocol::native_placements(&cohort),
        common::protocol::identity("nonce-a"),
    );
    let second = plan_session(
        cohort.clone(),
        common::protocol::native_placements(&cohort),
        common::protocol::identity("nonce-b"),
    );
    assert!(first.is_ok());
    assert!(second.is_ok());
    let first = match first {
        Ok(plan) => plan,
        Err(error) => panic!("first plan failed: {error:?}"),
    };
    let second = match second {
        Ok(plan) => plan,
        Err(error) => panic!("second plan failed: {error:?}"),
    };
    assert_ne!(first.identity, second.identity);
    assert_ne!(first.session, second.session);
    assert_eq!(first.roles.len(), cohort.roles.len());
}

#[test]
fn stale_projection_receipt_is_rejected_by_exact_intake() {
    let (projected, bundle) =
        common::protocol::projection_parts(common::protocol::native_source(common::protocol::end()));
    let changed = common::protocol::projection_parts(common::protocol::native_source(common::protocol::communication(
        common::protocol::ROLE_A,
        common::protocol::ROLE_B,
        common::protocol::LABEL_A,
        common::protocol::PAYLOAD_A,
        common::protocol::end(),
    )));
    let result = lattice::engine::protocol::admit_projection(lattice::engine::protocol::FrozenProjection {
        projected: &projected,
        bundle: &changed.1,
    });
    assert_eq!(result, Err(Error::Issue(Issue::IdentityMismatch)));
    assert_ne!(bundle.receipt().identity, changed.1.receipt().identity);
}

// r[verify lattice.role_protocol.intake.partial]
// r[verify lattice.role_protocol.placement.fallback]
#[test]
fn incomplete_or_mixed_placement_fails_before_session_creation() {
    let cohort = common::protocol::cohort(common::protocol::native_source(common::protocol::end()));
    let mut missing = common::protocol::native_placements(&cohort);
    missing.pop();
    assert_eq!(
        plan_session(cohort.clone(), missing, common::protocol::identity("missing")),
        Err(Error::Issue(Issue::InvalidPlacement))
    );

    let mut mixed = common::protocol::native_placements(&cohort);
    mixed[0].kind = lattice::engine::protocol::PlacementKind::External {
        contract: common::protocol::identity("wrong-contract"),
        input_schema: common::protocol::identity("wrong-input"),
        output_schema: common::protocol::identity("wrong-output"),
        destination_policy: common::protocol::identity("wrong-policy"),
        credential_reference: common::protocol::identity("wrong-credential"),
    };
    assert_eq!(
        plan_session(cohort, mixed, common::protocol::identity("mixed")),
        Err(Error::Issue(Issue::InvalidPlacement))
    );
}

#[test]
fn forged_session_plan_identity_is_rejected_before_state_creation() {
    let cohort = common::protocol::cohort(common::protocol::native_source(common::protocol::end()));
    let mut plan = plan_session(
        cohort.clone(),
        common::protocol::native_placements(&cohort),
        common::protocol::identity("forged-plan"),
    )
    .unwrap_or_else(|error| panic!("plan failed: {error:?}"));
    plan.identity = common::protocol::identity("wrong-plan-identity");
    assert_eq!(create_session(plan), Err(Error::Issue(Issue::IdentityMismatch)));
}

// r[verify lattice.role_protocol.transitions.transfer]
// r[verify lattice.role_protocol.ownership.stale]
#[test]
fn exact_owned_transfer_commits_both_cursors_and_excludes_former_owner() {
    let prepared = prepared_transfer();
    let mut current = unknown(&prepared);
    let attempt = match current.attempts.last() {
        Some(attempt) => attempt.clone(),
        None => panic!("unknown fixture has no attempt"),
    };
    let envelope = attempt.messages[0].clone();
    current =
        match admit_observation(&current, &envelope, true, common::protocol::identity("receiver-observation"), None) {
            Ok(session) => session,
            Err(error) => panic!("observation failed: {error:?}"),
        };
    let committed = match settle_attempt(&current, &attempt.identity) {
        Ok(session) => session,
        Err(error) => panic!("settle failed: {error:?}"),
    };
    assert_eq!(committed.status, SessionStatus::Complete);
    assert!(committed.cursors.iter().all(|cursor| cursor.protocol == RoleProgram::End));
    assert_eq!(committed.owners[0].owner, common::protocol::lattice_role(common::protocol::ROLE_B));
    assert_eq!(committed.attempts[0].state, AttemptState::Committed);
}

// r[verify lattice.role_protocol.envelopes.cross_session]
// r[verify lattice.role_protocol.fixtures.coverage]
#[test]
fn stale_wrong_step_and_duplicate_envelopes_do_not_advance() {
    let prepared = prepared_transfer();
    let current = unknown(&prepared);
    let attempt = current.attempts[0].clone();
    let mut stale = attempt.messages[0].clone();
    stale.session = common::protocol::identity("another-session");
    assert_eq!(
        admit_observation(&current, &stale, true, common::protocol::identity("stale-observation"), None,),
        Err(Error::Issue(Issue::IdentityMismatch))
    );
    assert_eq!(current.cursors, prepared.cursors);

    let first = match admit_observation(
        &current,
        &attempt.messages[0],
        true,
        common::protocol::identity("accepted-observation"),
        None,
    ) {
        Ok(session) => session,
        Err(error) => panic!("first observation failed: {error:?}"),
    };
    assert_eq!(
        admit_observation(
            &first,
            &attempt.messages[0],
            true,
            common::protocol::identity("duplicate-observation"),
            None,
        ),
        Err(Error::Issue(Issue::DuplicateMessage))
    );
    assert_eq!(first.cursors, current.cursors);
}

#[test]
fn forged_envelope_fields_fail_without_state_advancement() {
    let prepared = prepared_transfer();
    let current = unknown(&prepared);
    let original = current.attempts[0].messages[0].clone();
    let mut forged = vec![
        {
            let mut envelope = original.clone();
            envelope.source_role = common::protocol::lattice_role(common::protocol::ROLE_B);
            envelope
        },
        {
            let mut envelope = original.clone();
            envelope.destination_role = common::protocol::lattice_role(common::protocol::ROLE_A);
            envelope
        },
        {
            let mut envelope = original.clone();
            envelope.source_local_artifact = common::protocol::identity("wrong-source-local");
            envelope
        },
        {
            let mut envelope = original.clone();
            envelope.destination_local_artifact = common::protocol::identity("wrong-destination-local");
            envelope
        },
        {
            let mut envelope = original.clone();
            envelope.payload_tag = lattice::engine::protocol::PayloadTag(common::protocol::PAYLOAD_B.0);
            envelope
        },
        {
            let mut envelope = original.clone();
            envelope.value = None;
            envelope
        },
        {
            let mut envelope = original.clone();
            envelope.adapter_cohort = common::protocol::identity("wrong-adapter-cohort");
            envelope
        },
    ];
    for envelope in forged.drain(..) {
        assert!(matches!(
            admit_observation(&current, &envelope, true, common::protocol::identity("forged-observation"), None,),
            Err(Error::Issue(_))
        ));
        assert_eq!(current.cursors, prepared.cursors);
        assert_eq!(current.owners, prepared.owners);
    }
}

// r[verify lattice.role_protocol.transitions.stale_label]
#[test]
fn exact_choice_label_advances_and_stale_label_is_rejected() {
    let session = session_for(common::protocol::native_source(common::protocol::choice()), false);
    let stale = plan_action(&session, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: lattice::engine::protocol::LabelId(999),
        payload_reference: common::protocol::identity("choice-payload"),
    });
    assert_eq!(stale, Err(Error::Issue(Issue::WrongLabel)));

    let action = plan_action(&session, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: lattice::engine::protocol::LabelId(common::protocol::LABEL_B.0),
        payload_reference: common::protocol::identity("choice-payload"),
    });
    let action = match action {
        Ok(action) => action,
        Err(error) => panic!("choice action failed: {error:?}"),
    };
    assert_eq!(action.messages.len(), 1);
    let prepared = match prepare_attempt(&session, action) {
        Ok(prepared) => prepared,
        Err(error) => panic!("choice prepare failed: {error:?}"),
    };
    let unknown = unknown(&prepared);
    let attempt = unknown.attempts[0].clone();
    let observed = match admit_observation(
        &unknown,
        &attempt.messages[0],
        true,
        common::protocol::identity("choice-observation"),
        None,
    ) {
        Ok(observed) => observed,
        Err(error) => panic!("choice observation failed: {error:?}"),
    };
    let settled = match settle_attempt(&observed, &attempt.identity) {
        Ok(settled) => settled,
        Err(error) => panic!("choice settle failed: {error:?}"),
    };
    assert!(matches!(
        settled.cursors[0].protocol,
        RoleProgram::Send { label, .. } if label == lattice::engine::protocol::LabelId(common::protocol::LABEL_B.0)
    ));
    assert!(matches!(
        settled.cursors[1].protocol,
        RoleProgram::Receive { label, .. } if label == lattice::engine::protocol::LabelId(common::protocol::LABEL_B.0)
    ));
}

#[test]
fn rejected_receiver_observation_fails_without_cursor_or_owner_advance() {
    let prepared = prepared_transfer();
    let unknown = unknown(&prepared);
    let attempt = unknown.attempts[0].clone();
    let observed = match admit_observation(
        &unknown,
        &attempt.messages[0],
        false,
        common::protocol::identity("rejected-observation"),
        Some(Issue::WrongPayload),
    ) {
        Ok(observed) => observed,
        Err(error) => panic!("rejected observation failed: {error:?}"),
    };
    let failed = match settle_attempt(&observed, &attempt.identity) {
        Ok(failed) => failed,
        Err(error) => panic!("failed settle failed: {error:?}"),
    };
    assert_eq!(failed.attempts[0].state, AttemptState::Failed);
    assert_eq!(failed.cursors, prepared.cursors);
    assert_eq!(failed.owners, prepared.owners);
}

// r[verify lattice.role_protocol.uncertainty.crash]
#[test]
fn crash_after_dispatch_remains_unknown_and_needs_explicit_recovery() {
    let current = unknown(&prepared_transfer());
    let attempt = current.attempts[0].identity.clone();
    assert_eq!(current.attempts[0].state, AttemptState::DispatchedUnknown);
    assert_eq!(retry_unknown(&current, &attempt, false), Err(Error::Issue(Issue::AcknowledgementRequired)));
    assert_eq!(abandon_unknown(&current, &attempt, false), Err(Error::Issue(Issue::AcknowledgementRequired)));
    let recovery = match retry_unknown(&current, &attempt, true) {
        Ok(recovery) => recovery,
        Err(error) => panic!("retry recovery failed: {error:?}"),
    };
    assert_eq!(recovery.recovered.attempts[0].state, AttemptState::AbandonedUnknown);
    assert_eq!(recovery.prepared.attempts.len(), 2);
    assert_ne!(recovery.prepared.attempts[1].identity, current.attempts[0].identity);
    assert_ne!(recovery.prepared.attempts[1].action, current.attempts[0].action);
    assert_eq!(recovery.prepared.attempts[1].sequence, 1);
}

#[test]
fn each_persisted_crash_prefix_replays_without_inventing_progress() {
    let initial = session_for(
        common::protocol::native_source(common::protocol::transfer(
            common::protocol::ROLE_A,
            common::protocol::ROLE_B,
            common::protocol::LABEL_A,
            common::protocol::end(),
        )),
        false,
    );
    assert_eq!(replay(initial.plan.clone(), &initial.events), Ok(initial.clone()));
    let prepared = prepared_transfer();
    assert_eq!(replay(prepared.plan.clone(), &prepared.events), Ok(prepared.clone()));
    let authorized = authorized(&prepared);
    assert_eq!(replay(authorized.plan.clone(), &authorized.events), Ok(authorized.clone()));
    let unknown = unknown(&prepared);
    assert_eq!(replay(unknown.plan.clone(), &unknown.events), Ok(unknown.clone()));
    let attempt = unknown.attempts[0].clone();
    let observed = match admit_observation(
        &unknown,
        &attempt.messages[0],
        true,
        common::protocol::identity("crash-window-observation"),
        None,
    ) {
        Ok(observed) => observed,
        Err(error) => panic!("observation failed: {error:?}"),
    };
    assert_eq!(replay(observed.plan.clone(), &observed.events), Ok(observed));
}

#[test]
fn chained_return_transfer_moves_owned_value_once_per_commit() {
    let session = session_for(
        common::protocol::native_source(common::protocol::transfer(
            common::protocol::ROLE_A,
            common::protocol::ROLE_B,
            common::protocol::LABEL_A,
            common::protocol::transfer(
                common::protocol::ROLE_B,
                common::protocol::ROLE_A,
                common::protocol::LABEL_B,
                common::protocol::end(),
            ),
        )),
        false,
    );
    let first_action = match plan_action(&session, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: lattice::engine::protocol::LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("first-transfer"),
    }) {
        Ok(action) => action,
        Err(error) => panic!("first action failed: {error:?}"),
    };
    let first_prepared =
        prepare_attempt(&session, first_action).unwrap_or_else(|error| panic!("first prepare failed: {error:?}"));
    let first_unknown = unknown(&first_prepared);
    let first_attempt = first_unknown.attempts[0].clone();
    let first_observed = admit_observation(
        &first_unknown,
        &first_attempt.messages[0],
        true,
        common::protocol::identity("first-receipt"),
        None,
    )
    .unwrap_or_else(|error| panic!("first receipt failed: {error:?}"));
    let first_committed = settle_attempt(&first_observed, &first_attempt.identity)
        .unwrap_or_else(|error| panic!("first settle failed: {error:?}"));
    assert_eq!(first_committed.owners[0].owner, common::protocol::lattice_role(common::protocol::ROLE_B));
    assert_eq!(
        plan_action(&first_committed, ActionRequest {
            source: common::protocol::lattice_role(common::protocol::ROLE_A),
            label: lattice::engine::protocol::LabelId(common::protocol::LABEL_B.0),
            payload_reference: common::protocol::identity("former-owner"),
        },),
        Err(Error::Issue(Issue::UnexpectedAction))
    );
    let return_action = plan_action(&first_committed, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_B),
        label: lattice::engine::protocol::LabelId(common::protocol::LABEL_B.0),
        payload_reference: common::protocol::identity("return-transfer"),
    });
    assert!(return_action.is_ok());
}

// r[verify lattice.role_protocol.replay.send]
// r[verify lattice.role_protocol.state.deterministic]
#[test]
fn replay_is_deterministic_and_emits_no_adapter_request() {
    let current = unknown(&prepared_transfer());
    let replayed = replay(current.plan.clone(), &current.events);
    assert_eq!(replayed, Ok(current.clone()));
    assert_eq!(replay(current.plan.clone(), &current.events), replayed);
}

// r[verify lattice.role_protocol.external_roles.denied]
#[test]
fn external_role_requires_exact_contract_placement() {
    let cohort = common::protocol::cohort(common::protocol::external_source(common::protocol::transfer(
        common::protocol::ROLE_A,
        common::protocol::ROLE_B,
        common::protocol::LABEL_A,
        common::protocol::end(),
    )));
    let exact = plan_session(
        cohort.clone(),
        common::protocol::external_placements(&cohort),
        common::protocol::identity("external-nonce"),
    );
    assert!(exact.is_ok());
    let mut wrong = common::protocol::external_placements(&cohort);
    if let lattice::engine::protocol::PlacementKind::External { contract, .. } = &mut wrong[1].kind {
        *contract = common::protocol::identity("wrong-contract");
    }
    assert_eq!(
        plan_session(cohort, wrong, common::protocol::identity("external-wrong")),
        Err(Error::Issue(Issue::InvalidPlacement))
    );
}

// r[verify lattice.role_protocol.diagnostics.redacted]
// r[verify lattice.role_protocol.evidence.replay]
// r[verify lattice.role_protocol.boundary.overclaim]
#[test]
fn inspection_is_bounded_and_omits_payloads_credentials_paths_and_overclaims() {
    let current = unknown(&prepared_transfer());
    let report = match inspect(&current, &[Issue::AdapterFailed]) {
        Ok(report) => report,
        Err(error) => panic!("inspection failed: {error:?}"),
    };
    assert!(identity(&report).is_ok());
    let rendered = match serde_json::to_string(&report) {
        Ok(rendered) => rendered,
        Err(error) => panic!("report rendering failed: {error}"),
    };
    assert!(!rendered.contains("receiver-observation"));
    assert!(!rendered.contains("credential"));
    assert!(!rendered.contains("/home/"));
    assert!(!rendered.contains("exactly-once-delivery-proven"));
    assert!(!safe_diagnostic_text("https://secret.invalid token=secret"));
    assert!(safe_diagnostic_text("adapter rejected before transport start"));
}
