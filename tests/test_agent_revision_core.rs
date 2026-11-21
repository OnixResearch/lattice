//! Pure agent revision lifecycle (Lattice #25): issue criteria at fixture
//! level, with a negative case for every refusal.

use lattice_core::agent_revision::Refusal;
use lattice_core::agent_revision::ops::prepare;
use lattice_core::agent_revision::ops::retention as retain;
use lattice_core::agent_revision::ops::selection as select;
use lattice_core::agent_revision::ops::sessions;
use lattice_core::agent_revision::ports as verdict;
use lattice_core::agent_revision::state::Lifecycle;
use lattice_core::agent_revision::state::Preparation;
use lattice_core::agent_revision::state::SelectionKind;

const FAMILY: &str = "rust-implementation";

fn definition() -> String {
    format!("blake3:{}", "d".repeat(64))
}

fn revision(digit: char) -> String {
    digit.to_string().repeat(64)
}

fn reference(digit: char) -> String {
    format!("blake3:{}", blake3::hash(digit.to_string().as_bytes()).to_hex())
}

fn bundle(revision: &str) -> lattice_core::agent_revision::facts::Bundle {
    lattice_core::agent_revision::facts::Bundle {
        schema: lattice_core::agent_revision::facts::BUNDLE_SCHEMA.into(),
        definition: definition(),
        definition_name: "omp-attempt-worker".into(),
        revision: revision.into(),
        adapter: "omp".into(),
        protocol_revision: "e325ffc73ab86e03bdd20a91b77d8e2b3964c906".into(),
        executable_identity: "e".repeat(64),
        compute: lattice_core::execution_binding::request::ComputeRequirement::HostLocal {},
        task_families: Some(vec![FAMILY.into()]),
        tools: None,
        routes: vec![lattice_core::agent_revision::facts::HostedRoute {
            name: "primary".into(),
            route_identity: reference('7'),
            provider_model: "model".into(),
            weight_identity: lattice_core::agent_revision::facts::Unexposed::Unknown,
            model_version: lattice_core::agent_revision::facts::Unexposed::Unknown,
        }],
        components: None,
    }
}

fn attachment(qualification_ref: &str) -> verdict::Attachment {
    verdict::Attachment {
        qualification_ref: qualification_ref.into(),
        evidence_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        protocol_identity: reference('a'),
        verifier_identity: reference('b'),
        observed_at_ms: 1_795_200_000_000,
    }
}

fn qualified(qualification_ref: &str) -> verdict::Verdict {
    verdict::Verdict::Qualified(attachment(qualification_ref))
}

fn command(
    kind: SelectionKind,
    request: &str,
    revision: &str,
    qualification_ref: &str,
    expected: u64,
) -> select::Command {
    select::Command {
        kind,
        request_id: request.into(),
        revision: revision.into(),
        qualification_ref: qualification_ref.into(),
        policy: lattice_core::agent_revision::state::Policy {
            task_family: FAMILY.into(),
            required_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        },
        expected_rollout_generation: expected,
    }
}

fn ok<T>(result: Result<lattice_core::agent_revision::ops::Applied<T>, Refusal>) -> (Lifecycle, T) {
    let applied = result.expect("transition applies");
    (applied.lifecycle, applied.outcome)
}

fn ticket(lifecycle: &Lifecycle, revision: &str) -> (Lifecycle, prepare::Ticket) {
    match ok(prepare::begin(lifecycle, revision)) {
        (next, prepare::Begin::Started { ticket }) => (next, ticket),
        (_, other) => panic!("expected a started preparation, got {other:?}"),
    }
}

fn prepared(lifecycle: &Lifecycle, revision: &str, artifact: char) -> Lifecycle {
    let (desired, _) = ok(prepare::desire(lifecycle, &bundle(revision)));
    let (pending, ticket) = ticket(&desired, revision);
    let observation = verdict::Prepared::Prepared {
        artifacts: vec![reference(artifact)],
    };
    ok(prepare::record(&pending, &ticket, &observation)).0
}

fn promoted(lifecycle: &Lifecycle, request: &str, revision: &str, qualification: char) -> Lifecycle {
    let qualification_ref = reference(qualification);
    let promotion =
        command(SelectionKind::Promotion, request, revision, &qualification_ref, lifecycle.rollout_generation);
    ok(select::select(lifecycle, &promotion, &qualified(&qualification_ref))).0
}

fn entry<'a>(session: &'a str, revision: &'a str) -> sessions::Entry<'a> {
    sessions::Entry { session, revision }
}

fn admitted(lifecycle: &Lifecycle, session: &str, revision: &str) -> Lifecycle {
    ok(sessions::admit(lifecycle, entry(session, revision))).0
}

/// A pinned on R1; R2 failed preparation.
fn r2_failed() -> Lifecycle {
    let start = Lifecycle::new(&definition());
    let r1 = promoted(&prepared(&start, &revision('1'), 'a'), "promote-r1", &revision('1'), 'q');
    let with_a = admitted(&r1, "session-a", &revision('1'));
    let (desired, _) = ok(prepare::desire(&with_a, &bundle(&revision('2'))));
    let (pending, ticket) = ticket(&desired, &revision('2'));
    let failure = verdict::Prepared::Failed {
        artifacts: vec![reference('c')],
    };
    ok(prepare::record(&pending, &ticket, &failure)).0
}

#[test]
// r[verify lattice.agent_revisions.preparation.scenario.failed]
// r[verify lattice.agent_revisions.sessions.scenario.pinned]
fn failed_r2_keeps_a_on_r1_and_admits_b_on_r1() {
    let lifecycle = r2_failed();
    assert_eq!(lifecycle.last_usable.as_deref(), Some(revision('1').as_str()));
    assert_eq!(lifecycle.selected.as_ref().map(|selected| selected.revision.clone()), Some(revision('1')));
    assert!(matches!(lifecycle.candidates[&revision('2')].preparation, Preparation::Failed { generation: 2 }));
    assert_eq!(sessions::effect_permitted(&lifecycle, "session-a").expect("A may act").revision, revision('1'));
    let with_b = admitted(&lifecycle, "session-b", &revision('1'));
    assert_eq!(with_b.sessions["session-b"].revision, revision('1'));
    assert_eq!(
        sessions::admit(&lifecycle, entry("session-b", &revision('2'))).unwrap_err(),
        Refusal::RevisionNotSelected
    );
    let promote_r2 = command(SelectionKind::Promotion, "promote-r2", &revision('2'), &reference('r'), 1);
    assert_eq!(
        select::select(&lifecycle, &promote_r2, &qualified(&reference('r'))).unwrap_err(),
        Refusal::PreparationFailed
    );
    assert_eq!(prepare::begin(&lifecycle, &revision('2')).unwrap_err(), Refusal::PreparationFailed);
}

#[test]
// r[verify lattice.agent_revisions.sessions.scenario.pinned]
// r[verify lattice.agent_revisions.promotion]
// r[verify lattice.agent_revisions.facts.scenario.coexist]
fn promoted_r3_serves_new_c_while_a_stays_on_r1() {
    let r3 = promoted(&prepared(&r2_failed(), &revision('3'), 'b'), "promote-r3", &revision('3'), 's');
    assert_eq!(r3.rollout_generation, 2);
    let with_c = admitted(&r3, "session-c", &revision('3'));
    assert_eq!(with_c.sessions["session-c"].revision, revision('3'));
    assert_eq!(with_c.sessions["session-a"].revision, revision('1'));
    assert_eq!(sessions::effect_permitted(&with_c, "session-a").expect("A keeps R1").revision, revision('1'));
    assert_eq!(
        sessions::admit(&with_c, entry("session-d", &revision('1'))).unwrap_err(),
        Refusal::RevisionNotSelected
    );
    assert_eq!(sessions::admit(&with_c, entry("session-a", &revision('3'))).unwrap_err(), Refusal::SessionConflict);
    let (_, again) = ok(sessions::admit(&with_c, entry("session-c", &revision('3'))));
    assert!(matches!(again, sessions::Admitted::AlreadyPinned { .. }));
}

#[test]
// r[verify lattice.agent_revisions.preparation.scenario.late]
fn late_r2_result_cannot_replace_r3_or_its_generation() {
    let start = prepared(&Lifecycle::new(&definition()), &revision('1'), 'a');
    let (desired, _) = ok(prepare::desire(&start, &bundle(&revision('2'))));
    let (abandoned, late_ticket) = ticket(&desired, &revision('2'));
    let r3 = promoted(&prepared(&abandoned, &revision('3'), 'b'), "promote-r3", &revision('3'), 's');
    let late = verdict::Prepared::Prepared {
        artifacts: vec![reference('c')],
    };
    let (after, recorded) = ok(prepare::record(&r3, &late_ticket, &late));
    assert!(matches!(recorded, prepare::Recorded::Applied {
        preparation: Preparation::Superseded { generation: 2 },
        ..
    }));
    assert_eq!(after.desired, r3.desired);
    assert_eq!(after.selected, r3.selected);
    assert_eq!(after.last_usable.as_deref(), Some(revision('3').as_str()));
    assert_eq!(after.rollout_generation, r3.rollout_generation);
    let promote_r2 = command(SelectionKind::Promotion, "promote-r2", &revision('2'), &reference('r'), 1);
    assert_eq!(
        select::select(&after, &promote_r2, &qualified(&reference('r'))).unwrap_err(),
        Refusal::PreparationSuperseded
    );
    assert_eq!(prepare::begin(&after, &revision('2')).unwrap_err(), Refusal::NotDesired);
    let differing = verdict::Prepared::Failed { artifacts: Vec::new() };
    assert_eq!(prepare::record(&after, &late_ticket, &differing).unwrap_err(), Refusal::ResultConflict);
    let (_, repeated) = ok(prepare::record(&after, &late_ticket, &late));
    assert!(matches!(repeated, prepare::Recorded::Unchanged { .. }));
    let stale = prepare::Ticket {
        generation: 1,
        ..late_ticket
    };
    assert_eq!(prepare::record(&after, &stale, &late).unwrap_err(), Refusal::StaleTicket);
    assert!(retain::plan(&after).iter().any(|sweep| sweep.revision == revision('2')));
}

#[test]
// r[verify lattice.agent_revisions.promotion.scenario.idempotent]
// r[verify lattice.agent_revisions.promotion.scenario.concurrent]
fn duplicate_and_concurrent_promotion_select_once() {
    let both = prepared(&prepared(&Lifecycle::new(&definition()), &revision('1'), 'a'), &revision('4'), 'b');
    let first = command(SelectionKind::Promotion, "promote-1", &revision('1'), &reference('q'), 0);
    let rival = command(SelectionKind::Promotion, "promote-4", &revision('4'), &reference('t'), 0);
    let (selected, outcome) = ok(select::select(&both, &first, &qualified(&reference('q'))));
    assert!(matches!(outcome, select::Selected::Applied { .. }));
    assert_eq!(
        select::select(&selected, &rival, &qualified(&reference('t'))).unwrap_err(),
        Refusal::GenerationConflict
    );
    let (unchanged, duplicate) = ok(select::select(&selected, &first, &verdict::Verdict::Unavailable {}));
    assert!(matches!(duplicate, select::Selected::AlreadyApplied { is_current: true, .. }));
    assert_eq!(unchanged, selected);
    assert_eq!(selected.rollout_generation, 1);
    let reused = command(SelectionKind::Promotion, "promote-1", &revision('4'), &reference('t'), 1);
    assert_eq!(
        select::select(&selected, &reused, &qualified(&reference('t'))).unwrap_err(),
        Refusal::RequestConflict
    );
    let (_, again) = ok(prepare::begin(&selected, &revision('4')));
    assert_eq!(again, prepare::Begin::AlreadyPrepared { generation: 2 });
    let (pending, started) = ticket(&ok(prepare::desire(&selected, &bundle(&revision('5')))).0, &revision('5'));
    let (_, reissued) = ok(prepare::begin(&pending, &revision('5')));
    assert_eq!(reissued, prepare::Begin::Reissued { ticket: started });
}

#[test]
// r[verify lattice.agent_revisions.revocation.scenario.effects]
fn revocation_blocks_old_and_new_sessions_without_rewriting_pins() {
    let r3 = promoted(&prepared(&r2_failed(), &revision('3'), 'b'), "promote-r3", &revision('3'), 's');
    let with_c = admitted(&r3, "session-c", &revision('3'));
    let (old_revoked, _) = ok(sessions::revoke(&with_c, &sessions::Target::Qualification(reference('q'))));
    assert_eq!(sessions::effect_permitted(&old_revoked, "session-a").unwrap_err(), Refusal::QualificationRevoked);
    assert!(sessions::effect_permitted(&old_revoked, "session-c").is_ok());
    let (new_revoked, _) = ok(sessions::revoke(&old_revoked, &sessions::Target::Revision(revision('3'))));
    assert_eq!(sessions::effect_permitted(&new_revoked, "session-c").unwrap_err(), Refusal::RevisionRevoked);
    assert_eq!(
        sessions::admit(&new_revoked, entry("session-e", &revision('3'))).unwrap_err(),
        Refusal::RevisionRevoked
    );
    assert_eq!(new_revoked.last_usable, None);
    assert_eq!(new_revoked.sessions["session-c"].revision, revision('3'));
    let (session_revoked, _) = ok(sessions::revoke(&with_c, &sessions::Target::Session("session-c".into())));
    assert_eq!(sessions::effect_permitted(&session_revoked, "session-c").unwrap_err(), Refusal::SessionRevoked);
    assert_eq!(
        sessions::admit(&session_revoked, entry("session-c", &revision('3'))).unwrap_err(),
        Refusal::SessionRevoked
    );
    let (_, repeated) = ok(sessions::revoke(&session_revoked, &sessions::Target::Session("session-c".into())));
    assert_eq!(repeated, sessions::Revocation::AlreadyRevoked);
    let rollback = command(SelectionKind::Rollback, "rollback-r1", &revision('1'), &reference('q'), 2);
    assert_eq!(
        select::select(&old_revoked, &rollback, &qualified(&reference('q'))).unwrap_err(),
        Refusal::QualificationRevoked
    );
    assert_eq!(prepare::desire(&new_revoked, &bundle(&revision('3'))).unwrap_err(), Refusal::RevisionRevoked);
    assert_eq!(sessions::effect_permitted(&with_c, "session-z").unwrap_err(), Refusal::UnknownSession);
}

#[test]
// r[verify lattice.agent_revisions.promotion]
// r[verify lattice.agent_revisions.promotion.scenario.mismatch]
fn qualification_must_match_subject_class_and_reference() {
    let r1 =
        promoted(&prepared(&Lifecycle::new(&definition()), &revision('1'), 'a'), "promote-r1", &revision('1'), 'q');
    let changed = prepared(&r1, &revision('6'), 'b');
    let reuse = command(SelectionKind::Promotion, "promote-6", &revision('6'), &reference('q'), 1);
    assert_eq!(
        select::select(&changed, &reuse, &qualified(&reference('q'))).unwrap_err(),
        Refusal::QualificationMismatch
    );
    let fresh = command(SelectionKind::Promotion, "promote-6", &revision('6'), &reference('u'), 1);
    let stale = verdict::Verdict::NotQualified {
        reason: verdict::Unqualified::Stale,
    };
    let expected = Refusal::NotQualified {
        reason: verdict::Unqualified::Stale,
    };
    assert_eq!(select::select(&changed, &fresh, &stale).unwrap_err(), expected);
    assert_eq!(
        select::select(&changed, &fresh, &qualified(&reference('v'))).unwrap_err(),
        Refusal::QualificationMismatch
    );
    let mut weaker = attachment(&reference('u'));
    weaker.evidence_class = verdict::EvidenceClass::DeclaredSupport;
    let weaker = verdict::Verdict::Qualified(weaker);
    assert_eq!(select::select(&changed, &fresh, &weaker).unwrap_err(), Refusal::InsufficientClass);
    let mut malformed = attachment(&reference('u'));
    malformed.verifier_identity = "verifier".into();
    let malformed_refusal = Refusal::NotQualified {
        reason: verdict::Unqualified::Malformed,
    };
    let malformed = verdict::Verdict::Qualified(malformed);
    assert_eq!(select::select(&changed, &fresh, &malformed).unwrap_err(), malformed_refusal);
    let unavailable = select::select(&changed, &fresh, &verdict::Verdict::Unavailable {}).unwrap_err();
    assert_eq!(unavailable, Refusal::QualificationUnavailable {
        owner: "animus#18".into()
    });
    let query = select::query(&changed, &fresh, 7).expect("query");
    assert_eq!(query.profile_revision, revision('6'));
    assert_eq!(query.definition_identity, definition());
    let selected_again = command(SelectionKind::Promotion, "promote-1b", &revision('1'), &reference('q'), 1);
    assert_eq!(
        select::select(&r1, &selected_again, &qualified(&reference('q'))).unwrap_err(),
        Refusal::AlreadySelected
    );
}

#[test]
// r[verify lattice.agent_revisions.rollback]
// r[verify lattice.agent_revisions.rollback.scenario.history]
fn rollback_selects_an_earlier_revision_without_rewriting_sessions() {
    let r3 = promoted(&prepared(&r2_failed(), &revision('3'), 'b'), "promote-r3", &revision('3'), 's');
    let with_c = admitted(&r3, "session-c", &revision('3'));
    let rollback = command(SelectionKind::Rollback, "rollback-r1", &revision('1'), &reference('q'), 2);
    let (rolled, outcome) = ok(select::select(&with_c, &rollback, &qualified(&reference('q'))));
    assert!(matches!(outcome, select::Selected::Applied { .. }));
    let selected = rolled.selected.clone().expect("selection");
    assert_eq!(
        (selected.revision.as_str(), selected.kind, selected.rollout_generation),
        (revision('1').as_str(), SelectionKind::Rollback, 3)
    );
    assert_eq!(rolled.sessions, with_c.sessions);
    let with_f = admitted(&rolled, "session-f", &revision('1'));
    assert_eq!(with_f.sessions["session-c"].revision, revision('3'));
    let never = command(SelectionKind::Rollback, "rollback-r2", &revision('2'), &reference('r'), 3);
    assert_eq!(
        select::select(&rolled, &never, &qualified(&reference('r'))).unwrap_err(),
        Refusal::PreparationFailed
    );
    let unselected = prepared(&rolled, &revision('7'), 'e');
    let never = command(SelectionKind::Rollback, "rollback-r7", &revision('7'), &reference('w'), 3);
    assert_eq!(
        select::select(&unselected, &never, &qualified(&reference('w'))).unwrap_err(),
        Refusal::NotPreviouslySelected
    );
    let other_ref = command(SelectionKind::Rollback, "rollback-r3", &revision('3'), &reference('x'), 3);
    assert_eq!(
        select::select(&rolled, &other_ref, &qualified(&reference('x'))).unwrap_err(),
        Refusal::QualificationMismatch
    );
    let current = command(SelectionKind::Rollback, "rollback-r1b", &revision('1'), &reference('q'), 3);
    assert_eq!(
        select::select(&rolled, &current, &qualified(&reference('q'))).unwrap_err(),
        Refusal::AlreadySelected
    );
    let original = command(SelectionKind::Promotion, "promote-r3", &revision('3'), &reference('s'), 1);
    let (_, receipt) = ok(select::select(&rolled, &original, &verdict::Verdict::Unavailable {}));
    assert!(matches!(receipt, select::Selected::AlreadyApplied { is_current: false, .. }));
    let resent = command(SelectionKind::Promotion, "promote-r3", &revision('3'), &reference('s'), 3);
    assert_eq!(select::select(&rolled, &resent, &qualified(&reference('s'))).unwrap_err(), Refusal::RequestConflict);
}

#[test]
// r[verify lattice.agent_revisions.retention]
// r[verify lattice.agent_revisions.retention.scenario.uncertain]
// r[verify lattice.agent_revisions.retention.scenario.holds]
fn retention_and_uncertain_cleanup_never_release_occupied_resources() {
    let lifecycle = r2_failed();
    assert_eq!(retain::retained_by(&lifecycle, &revision('1')), vec![
        retain::Reason::Selected,
        retain::Reason::LastUsable,
        retain::Reason::LiveSession
    ]);
    let plan = retain::plan(&lifecycle);
    assert_eq!(plan.iter().map(|sweep| sweep.revision.clone()).collect::<Vec<_>>(), vec![revision('2')]);
    assert_eq!(
        retain::record(&lifecycle, &revision('1'), verdict::Cleanup::Released).unwrap_err(),
        Refusal::Retained
    );
    let (uncertain, swept) = ok(retain::record(&lifecycle, &revision('2'), verdict::Cleanup::Unknown));
    assert!(matches!(swept, retain::Swept::Uncertain { .. }));
    assert_eq!(uncertain.candidates[&revision('2')].artifacts, vec![reference('c')]);
    assert_eq!(retain::plan(&uncertain).len(), 1);
    let (released, _) = ok(retain::record(&uncertain, &revision('2'), verdict::Cleanup::Released));
    assert!(released.candidates[&revision('2')].artifacts.is_empty());
    let r3 = promoted(&prepared(&lifecycle, &revision('3'), 'b'), "promote-r3", &revision('3'), 's');
    let (closed, _) = ok(sessions::close(&r3, "session-a", &[reference('k')]));
    assert_eq!(sessions::effect_permitted(&closed, "session-a").unwrap_err(), Refusal::SessionClosed);
    assert_eq!(retain::retained_by(&closed, &revision('1')), vec![retain::Reason::Checkpoint]);
    let checkpoint = lattice_core::agent_revision::state::Hold {
        kind: lattice_core::agent_revision::state::HoldKind::Checkpoint,
        reference: reference('k'),
    };
    let (released_hold, _) = ok(retain::release(&closed, &revision('1'), &checkpoint));
    assert!(retain::retained_by(&released_hold, &revision('1')).is_empty());
    let evidence = lattice_core::agent_revision::state::Hold {
        kind: lattice_core::agent_revision::state::HoldKind::RecoveryEvidence,
        reference: reference('m'),
    };
    let (held, _) = ok(retain::hold(&released_hold, &revision('1'), &evidence));
    assert_eq!(retain::retained_by(&held, &revision('1')), vec![retain::Reason::RecoveryEvidence]);
    let (desired, _) = ok(prepare::desire(&held, &bundle(&revision('8'))));
    let (pending, unknown_ticket) = ticket(&desired, &revision('8'));
    let (unknown, _) = ok(prepare::record(&pending, &unknown_ticket, &verdict::Prepared::Unknown {}));
    assert!(retain::retained_by(&unknown, &revision('8')).contains(&retain::Reason::PreparationUncertain));
    assert_eq!(
        select::select(
            &unknown,
            &command(SelectionKind::Promotion, "p8", &revision('8'), &reference('y'), 2),
            &qualified(&reference('y'))
        )
        .unwrap_err(),
        Refusal::PreparationUncertain
    );
}

#[test]
// r[verify lattice.agent_revisions.core.scenario.refusals]
fn remaining_refusals_are_typed_and_leave_the_lifecycle_unchanged() {
    let empty = Lifecycle::new(&definition());
    assert_eq!(prepare::begin(&empty, &revision('1')).unwrap_err(), Refusal::NotDesired);
    let mut foreign = bundle(&revision('1'));
    foreign.definition = reference('f');
    assert_eq!(prepare::desire(&empty, &foreign).unwrap_err(), Refusal::DefinitionMismatch);
    let mut malformed = bundle(&revision('1'));
    malformed.revision = format!("blake3:{}", revision('1'));
    assert_eq!(prepare::desire(&empty, &malformed).unwrap_err(), Refusal::Malformed);
    let (desired, _) = ok(prepare::desire(&empty, &bundle(&revision('1'))));
    let (_, unchanged) = ok(prepare::desire(&desired, &bundle(&revision('1'))));
    assert!(matches!(unchanged, prepare::Desire::Unchanged { generation: 1, .. }));
    let promote = command(SelectionKind::Promotion, "p1", &revision('1'), &reference('q'), 0);
    assert_eq!(select::select(&desired, &promote, &qualified(&reference('q'))).unwrap_err(), Refusal::NotPrepared);
    let (pending, _) = ticket(&desired, &revision('1'));
    assert_eq!(
        select::select(&pending, &promote, &qualified(&reference('q'))).unwrap_err(),
        Refusal::PreparationInFlight
    );
    let (moved, _) = ok(prepare::desire(&pending, &bundle(&revision('2'))));
    assert_eq!(prepare::begin(&moved, &revision('1')).unwrap_err(), Refusal::NotDesired);
    let unknown = command(SelectionKind::Promotion, "p9", &revision('9'), &reference('q'), 0);
    assert_eq!(
        select::select(&desired, &unknown, &qualified(&reference('q'))).unwrap_err(),
        Refusal::UnknownRevision
    );
    let bad = command(SelectionKind::Promotion, "", &revision('1'), &reference('q'), 0);
    assert_eq!(select::select(&desired, &bad, &qualified(&reference('q'))).unwrap_err(), Refusal::Malformed);
    assert_eq!(sessions::admit(&desired, entry("session-a", &revision('1'))).unwrap_err(), Refusal::NoSelection);
    assert_eq!(sessions::close(&desired, "session-a", &[]).unwrap_err(), Refusal::UnknownSession);
    let mut exhausted = empty;
    exhausted.change_generation = u64::MAX;
    assert_eq!(prepare::desire(&exhausted, &bundle(&revision('1'))).unwrap_err(), Refusal::GenerationExhausted);
}

#[test]
// r[verify lattice.agent_revisions.bundle.scenario.hosted]
fn hosted_routes_admit_only_unknown_weight_and_version() {
    let encoded = serde_json::to_value(bundle(&revision('1'))).expect("bundle encodes");
    assert_eq!(encoded["routes"][0]["weight_identity"], "unknown");
    assert_eq!(encoded["routes"][0]["model_version"], "unknown");
    let mut claimed = encoded.clone();
    claimed["routes"][0]["weight_identity"] = serde_json::json!(reference('9'));
    assert!(serde_json::from_value::<lattice_core::agent_revision::facts::Bundle>(claimed).is_err());
    let identity = lattice_core::agent_revision::facts::identity(&bundle(&revision('1'))).expect("identity");
    assert!(lattice_core::agent_revision::is_reference(&identity));
}

#[test]
// r[verify lattice.agent_revisions.framing.scenario.undeclared]
// r[verify lattice.agent_revisions.framing.scenario.declared]
fn components_frame_only_when_declared_and_every_change_alters_the_digest() {
    use lattice_core::agent_revision::framing;
    assert_eq!(framing::digest(None), Ok(None));
    let base = framing::Components {
        schema: framing::COMPONENTS_SCHEMA.into(),
        memory: Some(framing::Memory::SingleShotNoSession {}),
        checkpoint: Some(framing::Checkpoint::Unsupported {}),
        tools: Some(framing::Tools {
            protocol: "omp-print-json".into(),
            names: vec!["write".into(), "read".into(), "edit".into()],
        }),
        model_requirements: Some(vec![framing::Requirement {
            route: "primary".into(),
            requirement_ref: reference('1'),
        }]),
        compatibility: Some(framing::Compatibility {
            producer: "lattice.implementation-attempt-run.v3".into(),
            consumer: "lattice.candidate-worker-compatibility.v1".into(),
        }),
    };
    let digest = framing::digest(Some(&base)).expect("valid").expect("declared");
    let mut reordered = base.clone();
    if let Some(tools) = reordered.tools.as_mut() {
        tools.names.reverse();
    }
    assert_eq!(framing::digest(Some(&reordered)).expect("valid"), Some(digest.clone()));
    let mut memory = base.clone();
    memory.memory = Some(framing::Memory::SessionScoped {
        schema: "omp-session.v1".into(),
    });
    let mut tools = base.clone();
    tools.tools = Some(framing::Tools {
        protocol: "omp-print-json".into(),
        names: vec!["read".into()],
    });
    let mut requirement = base.clone();
    requirement.model_requirements = Some(vec![framing::Requirement {
        route: "primary".into(),
        requirement_ref: reference('2'),
    }]);
    let mut empty = base.clone();
    empty.memory = None;
    for changed in [memory, tools, requirement, empty] {
        assert_ne!(framing::digest(Some(&changed)).expect("valid"), Some(digest.clone()));
    }
    let mut schema = base.clone();
    schema.schema = "lattice.agent-revision-components.v2".into();
    assert_eq!(framing::digest(Some(&schema)), Err(framing::Defect::Schema));
    let mut duplicate = base.clone();
    duplicate.tools = Some(framing::Tools {
        protocol: "omp".into(),
        names: vec!["read".into(), "read".into()],
    });
    assert_eq!(framing::digest(Some(&duplicate)), Err(framing::Defect::Duplicate { field: "tools" }));
    let mut name = base.clone();
    name.tools = Some(framing::Tools {
        protocol: "OMP".into(),
        names: vec!["read".into()],
    });
    assert_eq!(framing::digest(Some(&name)), Err(framing::Defect::Name { field: "tools" }));
    let mut count = base.clone();
    count.tools = Some(framing::Tools {
        protocol: "omp".into(),
        names: Vec::new(),
    });
    assert_eq!(framing::digest(Some(&count)), Err(framing::Defect::Count { field: "tools" }));
    let mut unreferenced = base;
    unreferenced.model_requirements = Some(vec![framing::Requirement {
        route: "primary".into(),
        requirement_ref: "1".repeat(64),
    }]);
    assert_eq!(
        framing::digest(Some(&unreferenced)),
        Err(framing::Defect::Reference {
            field: "model_requirements"
        })
    );
}

#[test]
// r[verify lattice.agent_revisions.projection.scenario.slots]
fn slots_project_each_fact_and_blocked_pins() {
    let r3 = promoted(&prepared(&r2_failed(), &revision('3'), 'b'), "promote-r3", &revision('3'), 's');
    let with_c = admitted(&r3, "session-c", &revision('3'));
    let (revoked, _) = ok(sessions::revoke(&with_c, &sessions::Target::Qualification(reference('q'))));
    let slots = lattice_core::agent_revision::projection::slots(&revoked);
    assert_eq!(slots.schema, lattice_core::agent_revision::projection::SLOTS_SCHEMA);
    assert_eq!(
        slots.desired.as_ref().map(|desired| (desired.revision.clone(), desired.generation)),
        Some((revision('3'), 3))
    );
    assert_eq!(slots.prepared, vec![revision('1'), revision('3')]);
    assert_eq!(
        slots
            .qualified
            .iter()
            .map(|admitted| (admitted.revision.clone(), admitted.is_revoked))
            .collect::<Vec<_>>(),
        vec![(revision('1'), true), (revision('3'), false)]
    );
    assert_eq!(
        slots
            .selected_for_new_sessions
            .as_ref()
            .map(|selected| (selected.revision.clone(), selected.rollout_generation)),
        Some((revision('3'), 2))
    );
    assert_eq!(slots.last_usable, Some(revision('3')));
    let pinned = slots
        .pinned
        .iter()
        .map(|pin| (pin.session.as_str(), pin.revision.clone(), pin.is_blocked))
        .collect::<Vec<_>>();
    assert_eq!(pinned, vec![("session-a", revision('1'), true), ("session-c", revision('3'), false)]);
    let encoded = serde_json::to_string(&slots).expect("slots encode");
    assert!(!encoded.contains("artifacts"));
    assert!(!encoded.contains(&reference('a')));
}

/// The fixture scenario's exact identities, in a deterministic document.
fn fixture_identities() -> serde_json::Value {
    let start = Lifecycle::new(&definition());
    let (desired_r1, _) = ok(prepare::desire(&start, &bundle(&revision('1'))));
    let (pending_r1, ticket_r1) = ticket(&desired_r1, &revision('1'));
    let r1_prepared = ok(prepare::record(&pending_r1, &ticket_r1, &verdict::Prepared::Prepared {
        artifacts: vec![reference('a')],
    }))
    .0;
    let promote_r1 = command(SelectionKind::Promotion, "promote-r1", &revision('1'), &reference('q'), 0);
    let (r1, promotion_r1) = ok(select::select(&r1_prepared, &promote_r1, &qualified(&reference('q'))));
    let (with_a, pin_a) = ok(sessions::admit(&r1, entry("session-a", &revision('1'))));
    let (desired_r2, _) = ok(prepare::desire(&with_a, &bundle(&revision('2'))));
    let (pending_r2, ticket_r2) = ticket(&desired_r2, &revision('2'));
    let failed = ok(prepare::record(&pending_r2, &ticket_r2, &verdict::Prepared::Failed {
        artifacts: vec![reference('c')],
    }))
    .0;
    let (with_b, pin_b) = ok(sessions::admit(&failed, entry("session-b", &revision('1'))));
    let (desired_r3, _) = ok(prepare::desire(&with_b, &bundle(&revision('3'))));
    let (pending_r3, ticket_r3) = ticket(&desired_r3, &revision('3'));
    let r3_prepared = ok(prepare::record(&pending_r3, &ticket_r3, &verdict::Prepared::Prepared {
        artifacts: vec![reference('b')],
    }))
    .0;
    let promote_r3 = command(SelectionKind::Promotion, "promote-r3", &revision('3'), &reference('s'), 1);
    let (r3, promotion_r3) = ok(select::select(&r3_prepared, &promote_r3, &qualified(&reference('s'))));
    let (_, pin_c) = ok(sessions::admit(&r3, entry("session-c", &revision('3'))));
    let promotion = |outcome: &select::Selected| match outcome {
        select::Selected::Applied { promotion_identity, .. }
        | select::Selected::AlreadyApplied { promotion_identity, .. } => promotion_identity.clone(),
    };
    let pin = |outcome: &sessions::Admitted| match outcome {
        sessions::Admitted::Pinned { pin_identity, .. } | sessions::Admitted::AlreadyPinned { pin_identity, .. } => {
            pin_identity.clone()
        }
    };
    let bundle_identity =
        |digit| lattice_core::agent_revision::facts::identity(&bundle(&revision(digit))).expect("bundle identity");
    let policy = lattice_core::agent_revision::identity::policy(&promote_r1.policy).expect("policy identity");
    serde_json::json!({
        "definition": definition(),
        "revisions": { "R1": revision('1'), "R2": revision('2'), "R3": revision('3') },
        "bundles": { "R1": bundle_identity('1'), "R2": bundle_identity('2'), "R3": bundle_identity('3') },
        "policy": { "task_family": FAMILY, "required_class": "task_family_evaluation", "identity": policy },
        "preparations": {
            "R1": { "generation": ticket_r1.generation, "identity": ticket_r1.preparation_identity, "observation": "prepared" },
            "R2": { "generation": ticket_r2.generation, "identity": ticket_r2.preparation_identity, "observation": "failed" },
            "R3": { "generation": ticket_r3.generation, "identity": ticket_r3.preparation_identity, "observation": "prepared" },
        },
        "qualifications": { "R1": reference('q'), "R3": reference('s') },
        "promotions": { "R1": promotion(&promotion_r1), "R3": promotion(&promotion_r3) },
        "sessions": {
            "A": { "revision": revision('1'), "pin": pin(&pin_a) },
            "B": { "revision": revision('1'), "pin": pin(&pin_b) },
            "C": { "revision": revision('3'), "pin": pin(&pin_c) },
        },
    })
}

#[test]
// r[verify lattice.agent_revisions.identities.scenario.recorded]
fn fixture_identities_are_stable_and_recorded() {
    let identities = fixture_identities();
    assert_eq!(identities, fixture_identities());
    let pins = ["A", "B", "C"].map(|session| identities["sessions"][session]["pin"].as_str().expect("pin").to_owned());
    assert!(pins.iter().all(|pin| lattice_core::agent_revision::is_reference(pin)));
    assert_eq!(pins.iter().collect::<std::collections::BTreeSet<_>>().len(), pins.len());
    assert_ne!(identities["promotions"]["R1"], identities["promotions"]["R3"]);
    if let Ok(path) = std::env::var("LATTICE_AGENT_REVISION_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&identities).expect("encode")).expect("capture written");
    }
    let evidence = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".cairn/changes/manage-agent-revisions/evidence/fixture-qualification.json");
    let recorded: serde_json::Value =
        serde_json::from_slice(&std::fs::read(evidence).expect("evidence read")).expect("evidence decodes");
    assert_eq!(recorded["live_qualification"], false);
    assert_eq!(recorded["fixture"], identities);
}

#[test]
// r[verify lattice.agent_revisions.preparation.scenario.reissue]
fn an_abandoned_older_attempt_is_reissued_and_settled() {
    let empty = Lifecycle::new(&definition());
    let (desired, _) = ok(prepare::desire(&empty, &bundle(&revision('1'))));
    let (pending, first) = ticket(&desired, &revision('1'));
    let (unknown, _) = ok(prepare::record(&pending, &first, &verdict::Prepared::Unknown {}));
    let (moved, _) = ok(prepare::desire(&unknown, &bundle(&revision('2'))));
    let (back, _) = ok(prepare::desire(&moved, &bundle(&revision('1'))));
    assert_eq!(back.desired.as_ref().map(|desired| desired.generation), Some(3));
    assert_eq!(prepare::begin(&back, &revision('1')).unwrap_err(), Refusal::PreparationInFlight);
    let reissued = prepare::reissue(&back, &revision('1')).expect("the open attempt is reissued");
    assert_eq!(reissued, first);
    let late = verdict::Prepared::Prepared {
        artifacts: vec![reference('a')],
    };
    let (settled, recorded) = ok(prepare::record(&back, &reissued, &late));
    assert!(matches!(recorded, prepare::Recorded::Applied {
        preparation: Preparation::Superseded { generation: 1 },
        last_usable: None,
    }));
    assert_eq!(prepare::reissue(&settled, &revision('1')).unwrap_err(), Refusal::StaleTicket);
    assert_eq!(prepare::begin(&settled, &revision('1')).unwrap_err(), Refusal::PreparationSuperseded);
    let (again, _) = ok(prepare::desire(&settled, &bundle(&revision('1'))));
    let (pending_again, current) = ticket(&again, &revision('1'));
    assert_eq!(current.generation, 4);
    let (prepared_again, _) = ok(prepare::record(&pending_again, &current, &late));
    assert!(matches!(prepared_again.candidates[&revision('1')].preparation, Preparation::Prepared {
        generation: 4
    }));
    assert_eq!(prepared_again.last_usable.as_deref(), Some(revision('1').as_str()));
    let (abandoned, failed_ticket) =
        ticket(&ok(prepare::desire(&prepared_again, &bundle(&revision('2')))).0, &revision('2'));
    let (elsewhere, _) = ok(prepare::desire(&abandoned, &bundle(&revision('3'))));
    assert_eq!(prepare::begin(&elsewhere, &revision('2')).unwrap_err(), Refusal::NotDesired);
    let reissued = prepare::reissue(&elsewhere, &revision('2')).expect("the pending attempt is reissued");
    assert_eq!(reissued, failed_ticket);
    let failure = verdict::Prepared::Failed { artifacts: Vec::new() };
    let (failed, _) = ok(prepare::record(&elsewhere, &reissued, &failure));
    assert!(matches!(failed.candidates[&revision('2')].preparation, Preparation::Failed { .. }));
    assert_eq!(prepare::reissue(&empty, &revision('1')).unwrap_err(), Refusal::UnknownRevision);
}

#[test]
// r[verify lattice.agent_revisions.preparation.scenario.revoked]
fn a_revoked_desired_revision_never_starts_preparation() {
    let (desired, _) = ok(prepare::desire(&Lifecycle::new(&definition()), &bundle(&revision('2'))));
    let (revoked, _) = ok(sessions::revoke(&desired, &sessions::Target::Revision(revision('2'))));
    assert_eq!(prepare::begin(&revoked, &revision('2')).unwrap_err(), Refusal::RevisionRevoked);
    let (pending, started) = ticket(&desired, &revision('2'));
    let (revoked_pending, _) = ok(sessions::revoke(&pending, &sessions::Target::Revision(revision('2'))));
    let (_, reissued) = ok(prepare::begin(&revoked_pending, &revision('2')));
    assert_eq!(reissued, prepare::Begin::Reissued {
        ticket: started.clone()
    });
    let observation = verdict::Prepared::Prepared {
        artifacts: vec![reference('c')],
    };
    let (recorded, _) = ok(prepare::record(&revoked_pending, &started, &observation));
    assert_eq!(recorded.last_usable, None);
    assert_eq!(recorded.candidates[&revision('2')].artifacts, vec![reference('c')]);
}

#[test]
// r[verify lattice.agent_revisions.preparation.scenario.redelivered]
fn a_redelivered_prepared_result_is_recorded_after_its_generation_moves_on() {
    let start = prepared(&Lifecycle::new(&definition()), &revision('1'), 'a');
    let (desired, _) = ok(prepare::desire(&start, &bundle(&revision('2'))));
    let (pending, r2_ticket) = ticket(&desired, &revision('2'));
    let observation = verdict::Prepared::Prepared {
        artifacts: vec![reference('c')],
    };
    let (recorded, _) = ok(prepare::record(&pending, &r2_ticket, &observation));
    let (moved, _) = ok(prepare::desire(&recorded, &bundle(&revision('3'))));
    let (unchanged, again) = ok(prepare::record(&moved, &r2_ticket, &observation));
    assert_eq!(again, prepare::Recorded::Unchanged {
        preparation: Preparation::Prepared { generation: 2 }
    });
    assert_eq!(unchanged, moved);
    let failure = verdict::Prepared::Failed { artifacts: Vec::new() };
    assert_eq!(prepare::record(&moved, &r2_ticket, &failure).unwrap_err(), Refusal::ResultConflict);
    let unreported = verdict::Prepared::Prepared {
        artifacts: vec![reference('d')],
    };
    assert_eq!(prepare::record(&moved, &r2_ticket, &unreported).unwrap_err(), Refusal::ResultConflict);
}

#[test]
// r[verify lattice.agent_revisions.sessions.scenario.readmitted]
fn readmission_rechecks_closure_and_revocation() {
    let r3 = promoted(&prepared(&r2_failed(), &revision('3'), 'b'), "promote-r3", &revision('3'), 's');
    let with_c = admitted(&r3, "session-c", &revision('3'));
    let (closed, _) = ok(sessions::close(&with_c, "session-a", &[reference('k')]));
    assert_eq!(sessions::admit(&closed, entry("session-a", &revision('1'))).unwrap_err(), Refusal::SessionClosed);
    let (revision_revoked, _) = ok(sessions::revoke(&with_c, &sessions::Target::Revision(revision('3'))));
    assert_eq!(
        sessions::admit(&revision_revoked, entry("session-c", &revision('3'))).unwrap_err(),
        Refusal::RevisionRevoked
    );
    let (qualification_revoked, _) = ok(sessions::revoke(&with_c, &sessions::Target::Qualification(reference('s'))));
    assert_eq!(
        sessions::admit(&qualification_revoked, entry("session-c", &revision('3'))).unwrap_err(),
        Refusal::QualificationRevoked
    );
    let (unchanged, again) = ok(sessions::admit(&with_c, entry("session-c", &revision('3'))));
    assert!(matches!(again, sessions::Admitted::AlreadyPinned { .. }));
    assert_eq!(unchanged, with_c);
}

#[test]
// r[verify lattice.agent_revisions.revocation.scenario.capacity]
fn referenced_targets_are_revoked_even_when_revocations_are_full() {
    let r3 = promoted(&prepared(&r2_failed(), &revision('3'), 'b'), "promote-r3", &revision('3'), 's');
    let mut full = admitted(&r3, "session-c", &revision('3'));
    for index in 0..lattice_core::agent_revision::MAX_REVOCATIONS {
        let digits = format!("{index:064x}");
        full.revoked.qualifications.insert(format!("blake3:{digits}"));
        full.revoked.revisions.insert(digits);
        full.revoked.sessions.insert(format!("unreferenced-{index}"));
    }
    for unreferenced in [
        sessions::Target::Revision(revision('e')),
        sessions::Target::Qualification(reference('z')),
        sessions::Target::Session("never-admitted".into()),
    ] {
        assert_eq!(sessions::revoke(&full, &unreferenced).unwrap_err(), Refusal::CapacityExhausted);
    }
    let (session, _) = ok(sessions::revoke(&full, &sessions::Target::Session("session-c".into())));
    assert_eq!(sessions::effect_permitted(&session, "session-c").unwrap_err(), Refusal::SessionRevoked);
    let (revision_revoked, _) = ok(sessions::revoke(&full, &sessions::Target::Revision(revision('3'))));
    assert_eq!(sessions::effect_permitted(&revision_revoked, "session-c").unwrap_err(), Refusal::RevisionRevoked);
    assert_eq!(
        sessions::admit(&revision_revoked, entry("session-d", &revision('3'))).unwrap_err(),
        Refusal::RevisionRevoked
    );
    let (pinned, _) = ok(sessions::revoke(&full, &sessions::Target::Qualification(reference('q'))));
    assert_eq!(sessions::effect_permitted(&pinned, "session-a").unwrap_err(), Refusal::QualificationRevoked);
    let (failed, _) = ok(sessions::revoke(&full, &sessions::Target::Revision(revision('2'))));
    assert!(failed.revoked.revisions.contains(&revision('2')));
}

#[test]
// r[verify lattice.agent_revisions.core.scenario.capacity]
fn sessions_count_live_pins_and_closed_sessions_are_forgotten() {
    let r1 =
        promoted(&prepared(&Lifecycle::new(&definition()), &revision('1'), 'a'), "promote-r1", &revision('1'), 'q');
    let lifetime = lattice_core::agent_revision::MAX_SESSIONS + 64;
    let mut cycled = r1.clone();
    for index in 0..lifetime {
        let session = format!("cycled-{index}");
        cycled = admitted(&cycled, &session, &revision('1'));
        cycled = ok(sessions::close(&cycled, &session, &[])).0;
    }
    assert!(cycled.sessions.is_empty());
    assert_eq!(sessions::effect_permitted(&cycled, "cycled-0").unwrap_err(), Refusal::UnknownSession);
    assert_eq!(sessions::close(&cycled, "cycled-0", &[]).unwrap_err(), Refusal::UnknownSession);
    let mut full = r1;
    for index in 0..lattice_core::agent_revision::MAX_SESSIONS {
        full = admitted(&full, &format!("live-{index}"), &revision('1'));
    }
    assert_eq!(sessions::admit(&full, entry("one-more", &revision('1'))).unwrap_err(), Refusal::CapacityExhausted);
    let (checkpointed, _) = ok(sessions::close(&full, "live-0", &[reference('k')]));
    assert_eq!(checkpointed.sessions["live-0"].checkpoints, vec![reference('k')]);
    assert_eq!(sessions::effect_permitted(&checkpointed, "live-0").unwrap_err(), Refusal::SessionClosed);
    let (_, repeated) = ok(sessions::close(&checkpointed, "live-0", &[reference('k')]));
    assert!(matches!(repeated, sessions::Closed::AlreadyClosed { .. }));
    let with_room = admitted(&checkpointed, "one-more", &revision('1'));
    assert_eq!(
        sessions::admit(&with_room, entry("two-more", &revision('1'))).unwrap_err(),
        Refusal::CapacityExhausted
    );
    let checkpoint = lattice_core::agent_revision::state::Hold {
        kind: lattice_core::agent_revision::state::HoldKind::Checkpoint,
        reference: reference('k'),
    };
    let (released, _) = ok(retain::release(&with_room, &revision('1'), &checkpoint));
    assert!(!released.sessions.contains_key("live-0"));
}

#[test]
// r[verify lattice.agent_revisions.core.scenario.capacity]
// r[verify lattice.agent_revisions.promotion.scenario.idempotent]
fn receipts_slide_so_selection_never_exhausts_and_forgotten_requests_stay_refused() {
    let both = prepared(&prepared(&Lifecycle::new(&definition()), &revision('1'), 'a'), &revision('2'), 'b');
    let mut lifecycle = promoted(&both, "p-0", &revision('1'), 'q');
    lifecycle = promoted(&lifecycle, "p-1", &revision('2'), 'r');
    let lifetime = lattice_core::agent_revision::MAX_RECEIPTS + 64;
    for index in 0..lifetime {
        let (target, qualification) = if index % 2 == 0 { ('1', 'q') } else { ('2', 'r') };
        let rollback = command(
            SelectionKind::Rollback,
            &format!("rollback-{index}"),
            &revision(target),
            &reference(qualification),
            lifecycle.rollout_generation,
        );
        lifecycle = ok(select::select(&lifecycle, &rollback, &qualified(&reference(qualification)))).0;
    }
    assert_eq!(lifecycle.rollout_generation, u64::from(lifetime) + 2);
    assert_eq!(lifecycle.selected.as_ref().map(|selected| selected.revision.clone()), Some(revision('2')));
    assert_eq!(lifecycle.receipts.len(), usize::try_from(lattice_core::agent_revision::MAX_RECEIPTS).expect("fits"));
    assert!(!lifecycle.receipts.contains_key("p-0"));
    let forgotten = command(SelectionKind::Promotion, "p-0", &revision('1'), &reference('q'), 0);
    assert_eq!(
        select::select(&lifecycle, &forgotten, &qualified(&reference('q'))).unwrap_err(),
        Refusal::GenerationConflict
    );
    let latest = format!("rollback-{}", lifetime - 1);
    let repeated =
        command(SelectionKind::Rollback, &latest, &revision('2'), &reference('r'), lifecycle.rollout_generation - 1);
    let (_, receipt) = ok(select::select(&lifecycle, &repeated, &verdict::Verdict::Unavailable {}));
    assert!(matches!(receipt, select::Selected::AlreadyApplied { is_current: true, .. }));
}

#[test]
// r[verify lattice.agent_revisions.core.scenario.capacity]
fn unreferenced_revisions_are_forgotten_so_desire_never_exhausts() {
    let r1 =
        promoted(&prepared(&Lifecycle::new(&definition()), &revision('1'), 'a'), "promote-r1", &revision('1'), 'q');
    let with_a = admitted(&r1, "session-a", &revision('1'));
    let (desired_nine, _) = ok(prepare::desire(&with_a, &bundle(&revision('9'))));
    let (mut lifecycle, _) = ok(sessions::revoke(&desired_nine, &sessions::Target::Revision(revision('9'))));
    let lifetime = lattice_core::agent_revision::MAX_CANDIDATES * 3;
    for index in 0..lifetime {
        let digits = format!("{:064x}", u64::from(index) + 0x100);
        lifecycle = ok(prepare::desire(&lifecycle, &bundle(&digits))).0;
        assert!(
            lifecycle.candidates.len() <= usize::try_from(lattice_core::agent_revision::MAX_CANDIDATES).expect("fits")
        );
    }
    assert_eq!(lifecycle.change_generation, u64::from(lifetime) + 2);
    assert!(lifecycle.candidates.contains_key(&revision('1')));
    assert!(lifecycle.previously_selected.contains(&revision('1')));
    assert!(!lifecycle.candidates.contains_key(&revision('9')));
    assert_eq!(prepare::desire(&lifecycle, &bundle(&revision('9'))).unwrap_err(), Refusal::RevisionRevoked);
    assert!(sessions::effect_permitted(&lifecycle, "session-a").is_ok());
    let mut pending = Lifecycle::new(&definition());
    for index in 0..lattice_core::agent_revision::MAX_CANDIDATES {
        let digits = format!("{index:064x}");
        let (desired, _) = ok(prepare::desire(&pending, &bundle(&digits)));
        pending = ticket(&desired, &digits).0;
    }
    assert_eq!(prepare::desire(&pending, &bundle(&revision('f'))).unwrap_err(), Refusal::CapacityExhausted);
}
