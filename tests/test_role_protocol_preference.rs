//! Declared preference factors on projected role-protocol choices (Lattice #16).
//!
//! The annotated cohort comes from the published Choregraph
//! `choregraph.role-choreography.profile.preference-annotations.v1` profile. Factors
//! are metadata: these fixtures show that they are recorded only under the explicit
//! profile and never change exact-label admission, cursors, ownership or authority.

mod common;

use lattice::engine::protocol::ActionRequest;
use lattice::engine::protocol::AnnotatedProjection;
use lattice::engine::protocol::Cohort;
use lattice::engine::protocol::Error;
use lattice::engine::protocol::FactorDisposition;
use lattice::engine::protocol::GameMapping;
use lattice::engine::protocol::Issue;
use lattice::engine::protocol::LabelId;
use lattice::engine::protocol::PreferenceProfile;
use lattice::engine::protocol::SessionRecord;
use lattice::engine::protocol::SessionStatus;
use lattice::engine::protocol::admit_annotated_projection;
use lattice::engine::protocol::admit_observation;
use lattice::engine::protocol::create_session;
use lattice::engine::protocol::inspect;
use lattice::engine::protocol::mark_dispatched_unknown;
use lattice::engine::protocol::plan_action;
use lattice::engine::protocol::plan_session;
use lattice::engine::protocol::prepare_attempt;
use lattice::engine::protocol::record_authorization;
use lattice::engine::protocol::settle_attempt;
use lattice::engine::protocol::shell::ProtocolAuthority;
use lattice::engine::protocol::shell::SessionRepository;

const FACTOR_COST: u32 = 1;
const FACTOR_LATENCY: u32 = 2;
const WORLD_MARKET: u32 = 1;
const UNDECLARED_LABEL: u32 = 999;
const SPAN_END: u32 = 8;
const SOURCE_ENVELOPE: &[u8] = b"lattice-role-protocol-annotated-source";
const FACTOR_NAMES: [&str; 3] = ["cost", "latency", "market"];

const fn span() -> choregraph_choreography::model::participants::SourceSpan {
    choregraph_choreography::model::participants::SourceSpan {
        start: 0,
        end: SPAN_END,
    }
}

fn factor(id: u32, name: &str) -> choregraph_choreography::preference::model::FactorDeclaration {
    choregraph_choreography::preference::model::FactorDeclaration {
        id: choregraph_choreography::preference::model::FactorId(id),
        role: common::protocol::ROLE_A,
        name: name.to_owned(),
        span: span(),
    }
}

/// A projected `InternalChoice`/`Offer` fixture with declared factors on the choice at `path`.
fn annotated_source(
    protocol: choregraph_choreography::model::tree::Node,
    path: Vec<choregraph_choreography::preference::model::PathStep>,
    names: [&str; 3],
) -> choregraph_choreography::preference::model::AnnotatedSource {
    choregraph_choreography::preference::model::AnnotatedSource {
        schema: choregraph_choreography::preference::model::ANNOTATED_SCHEMA.to_owned(),
        base: common::protocol::native_source(protocol),
        annotations: choregraph_choreography::preference::model::AnnotationSet {
            factors: vec![factor(FACTOR_COST, names[0]), factor(FACTOR_LATENCY, names[1])],
            worlds: vec![choregraph_choreography::preference::model::WorldDeclaration {
                id: choregraph_choreography::preference::model::WorldId(WORLD_MARKET),
                name: names[2].to_owned(),
                span: span(),
            }],
            choices: vec![choregraph_choreography::preference::model::ChoiceAnnotation {
                path,
                decider: common::protocol::ROLE_A,
                factors: vec![
                    choregraph_choreography::preference::model::FactorId(FACTOR_COST),
                    choregraph_choreography::preference::model::FactorId(FACTOR_LATENCY),
                ],
                world: Some(choregraph_choreography::preference::model::WorldId(WORLD_MARKET)),
                span: span(),
            }],
        },
        non_claims: choregraph_choreography::preference::model::REQUIRED_NON_CLAIMS.map(str::to_owned).to_vec(),
    }
}

fn compile(
    source: &choregraph_choreography::preference::model::AnnotatedSource,
    selection: choregraph_choreography::preference::ProfileSelection,
) -> choregraph_choreography::preference::Report {
    let evaluated = serde_json::to_vec(source).unwrap_or_else(|error| panic!("fixture encode failed: {error}"));
    choregraph_choreography::preference::compile_evaluated(&evaluated, SOURCE_ENVELOPE, selection)
}

fn bundle(
    report: &choregraph_choreography::preference::Report,
) -> &choregraph_choreography::preference::linkage::AnnotatedBundle {
    report.bundle().unwrap_or_else(|| panic!("annotated fixture was refused: {:?}", report.refusal()))
}

fn annotated_report(names: [&str; 3]) -> choregraph_choreography::preference::Report {
    compile(
        &annotated_source(common::protocol::choice(), Vec::new(), names),
        choregraph_choreography::preference::ProfileSelection::PreferenceAnnotations,
    )
}

fn exact(bundle: &choregraph_choreography::preference::linkage::AnnotatedBundle) -> AnnotatedProjection<'_> {
    AnnotatedProjection {
        bundle,
        receipt: bundle.receipt_body(),
        receipt_bytes: &bundle.receipt().packed_bytes,
        local_set: bundle.local_set_body(),
    }
}

fn intake(
    bundle: &choregraph_choreography::preference::linkage::AnnotatedBundle,
    profile: PreferenceProfile,
) -> lattice::engine::protocol::AnnotatedIntake {
    admit_annotated_projection(exact(bundle), profile)
        .unwrap_or_else(|error| panic!("annotated intake failed: {error:?}"))
}

fn session(cohort: Cohort) -> SessionRecord {
    let placements = common::protocol::native_placements(&cohort);
    let plan = plan_session(cohort, placements, common::protocol::identity("preference-session-nonce"))
        .unwrap_or_else(|error| panic!("session plan failed: {error:?}"));
    create_session(plan).unwrap_or_else(|error| panic!("session creation failed: {error:?}"))
}

fn request(role: choregraph_choreography::model::participants::RoleId, label: u32) -> ActionRequest {
    ActionRequest {
        source: common::protocol::lattice_role(role),
        label: LabelId(label),
        payload_reference: common::protocol::identity("preference-payload"),
    }
}

fn record(cohort: &Cohort) -> &lattice::engine::protocol::PreferenceRecord {
    cohort.preference.as_ref().unwrap_or_else(|| panic!("annotated cohort has no recorded factors"))
}

fn json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| panic!("encode failed: {error}"))
}

/// Commits one exact action through the pure core; returns the settled session and its envelopes.
fn commit(
    current: &SessionRecord,
    request: ActionRequest,
) -> (SessionRecord, Vec<lattice::engine::protocol::MessageEnvelope>) {
    let action = plan_action(current, request).unwrap_or_else(|error| panic!("action plan failed: {error:?}"));
    let prepared = prepare_attempt(current, action).unwrap_or_else(|error| panic!("prepare failed: {error:?}"));
    let attempt = prepared.attempts.last().unwrap_or_else(|| panic!("no prepared attempt")).clone();
    let authorized =
        record_authorization(&prepared, &attempt.identity, vec![common::protocol::identity("authority-attempt")])
            .unwrap_or_else(|error| panic!("authorization record failed: {error:?}"));
    let mut observed = mark_dispatched_unknown(&authorized, &attempt.identity)
        .unwrap_or_else(|error| panic!("dispatch state failed: {error:?}"));
    for envelope in &attempt.messages {
        observed = admit_observation(&observed, envelope, true, common::protocol::identity("receiver-accepted"), None)
            .unwrap_or_else(|error| panic!("observation failed: {error:?}"));
    }
    let settled =
        settle_attempt(&observed, &attempt.identity).unwrap_or_else(|error| panic!("settle failed: {error:?}"));
    (settled, attempt.messages)
}

/// Exact-label decision facts that must not depend on the factors profile.
#[derive(serde::Serialize)]
struct DecisionView<'a> {
    status: SessionStatus,
    cursors: &'a [lattice::engine::protocol::RoleCursor],
    owners: &'a [lattice::engine::protocol::OwnedValue],
    attempts: Vec<AttemptView<'a>>,
}

#[derive(serde::Serialize)]
struct AttemptView<'a> {
    state: lattice::engine::protocol::AttemptState,
    kind: lattice::engine::protocol::ActionKind,
    source: lattice::engine::protocol::RoleId,
    label: LabelId,
    payload_tag: lattice::engine::protocol::PayloadTag,
    value: Option<lattice::engine::protocol::ValueId>,
    messages: Vec<EnvelopeView<'a>>,
}

#[derive(serde::Serialize)]
struct EnvelopeView<'a> {
    schema: &'a str,
    protocol: &'a lattice::engine::protocol::Identity,
    complete_local_set: &'a lattice::engine::protocol::Identity,
    source_role: lattice::engine::protocol::RoleId,
    destination_role: lattice::engine::protocol::RoleId,
    source_local_artifact: &'a lattice::engine::protocol::Identity,
    destination_local_artifact: &'a lattice::engine::protocol::Identity,
    label: LabelId,
    payload_tag: lattice::engine::protocol::PayloadTag,
    value: Option<lattice::engine::protocol::ValueId>,
    schema_set: &'a lattice::engine::protocol::Identity,
    adapter_cohort: &'a lattice::engine::protocol::Identity,
}

fn decisions(session: &SessionRecord) -> String {
    json(&DecisionView {
        status: session.status,
        cursors: &session.cursors,
        owners: &session.owners,
        attempts: session
            .attempts
            .iter()
            .map(|attempt| AttemptView {
                state: attempt.state,
                kind: attempt.kind,
                source: attempt.request.source,
                label: attempt.request.label,
                payload_tag: attempt.payload_tag,
                value: attempt.value,
                messages: attempt
                    .messages
                    .iter()
                    .map(|envelope| EnvelopeView {
                        schema: &envelope.schema,
                        protocol: &envelope.protocol,
                        complete_local_set: &envelope.complete_local_set,
                        source_role: envelope.source_role,
                        destination_role: envelope.destination_role,
                        source_local_artifact: &envelope.source_local_artifact,
                        destination_local_artifact: &envelope.destination_local_artifact,
                        label: envelope.label,
                        payload_tag: envelope.payload_tag,
                        value: envelope.value,
                        schema_set: &envelope.schema_set,
                        adapter_cohort: &envelope.adapter_cohort,
                    })
                    .collect(),
            })
            .collect(),
    })
}

fn contains_factor_names(text: &str) -> bool {
    FACTOR_NAMES.iter().any(|name| text.contains(&format!("\"{name}\"")))
}

/// Records every envelope it is asked to authorize and denies all of them, like the standalone CLI.
#[derive(Default)]
struct RecordingDeniedAuthority {
    requests: std::sync::Mutex<Vec<String>>,
}

impl ProtocolAuthority for RecordingDeniedAuthority {
    fn authorize(
        &self,
        _session: &SessionRecord,
        envelope: &lattice::engine::protocol::MessageEnvelope,
    ) -> Result<lattice::engine::protocol::Identity, Issue> {
        self.requests.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(json(envelope));
        Err(Issue::AuthorityDenied)
    }
}

fn denied_dispatch(current: &SessionRecord, label: u32) -> (Result<(), Error>, SessionRecord, Vec<String>) {
    let directory = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let store = lattice::store::Store::new(directory.path().join("protocol.redb"))
        .unwrap_or_else(|error| panic!("store failed: {error}"));
    let authority = RecordingDeniedAuthority::default();
    let adapters = lattice::engine::protocol::shell::Adapters::new(
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let runtime = lattice::engine::protocol::shell::Runtime::new(&store, &authority, &adapters);
    let created = runtime.create(current.plan.clone()).unwrap_or_else(|error| panic!("create failed: {error:?}"));
    let outcome = runtime.dispatch(&created, request(common::protocol::ROLE_A, label)).map(|_| ());
    let persisted = <lattice::store::Store as SessionRepository>::load(&store, &created.plan.session)
        .unwrap_or_else(|error| panic!("load failed: {error:?}"))
        .unwrap_or_else(|| panic!("session was not persisted"));
    let requests = authority.requests.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    (outcome, persisted, requests)
}

// r[verify lattice.role_protocol.preference_factors.recorded]
#[test]
fn selected_profile_records_declared_factors_for_the_decider_choice() {
    let report = annotated_report(FACTOR_NAMES);
    let admitted = intake(bundle(&report), PreferenceProfile::RecordDeclaredFactors);
    assert_eq!(admitted.report.disposition, FactorDisposition::FactorsRecorded);
    assert_eq!(admitted.report.game_mapping, GameMapping::NotFormalized);
    assert_eq!(report.game_mapping(), choregraph_choreography::preference::linkage::GameMapping::NotFormalized);
    assert_eq!(admitted.report.schema, lattice::engine::protocol::PREFERENCE_REPORT_SCHEMA);
    assert!(admitted.report.non_claims.iter().any(|claim| claim == "no-preference-authority"));
    assert!(admitted.report.non_claims.iter().any(|claim| claim == "no-label-boolean-conversion"));

    let recorded = record(&admitted.cohort);
    assert_eq!(recorded.producer_profile, choregraph_choreography::preference::ANNOTATED_PROFILE);
    assert_eq!(recorded.base_receipt, admitted.cohort.projection_receipt);
    assert_eq!(recorded.complete_local_set, admitted.cohort.complete_local_set);
    assert_eq!(
        recorded.annotated_receipt.as_str(),
        choregraph_choreography::linkage::digest::prefixed_identity(&bundle(&report).receipt().identity)
    );
    assert_eq!(recorded.roles.len(), 2);
    assert_eq!(recorded.roles[0].role, common::protocol::lattice_role(common::protocol::ROLE_A));
    assert_eq!(recorded.roles[0].local_artifact, admitted.cohort.roles[0].local_artifact);
    assert!(recorded.roles[1].decisions.is_empty());
    let decision = &recorded.roles[0].decisions[0];
    assert!(decision.local_path.is_empty());
    assert_eq!(decision.labels, vec![
        LabelId(common::protocol::LABEL_A.0),
        LabelId(common::protocol::LABEL_B.0)
    ]);
    assert_eq!(decision.factors.iter().map(|factor| factor.name.as_str()).collect::<Vec<_>>(), ["cost", "latency"]);
    assert!(decision.factors.iter().all(|factor| factor.role == recorded.roles[0].role));
    assert_eq!(decision.world.as_ref().map(|world| world.name.as_str()), Some("market"));

    let created = session(admitted.cohort);
    let inspection = inspect(&created, &[]).unwrap_or_else(|error| panic!("inspect failed: {error:?}"));
    let exposed = inspection.preference.as_ref().unwrap_or_else(|| panic!("inspection omits recorded factors"));
    assert_eq!(exposed.disposition, FactorDisposition::FactorsRecorded);
    assert_eq!(exposed.game_mapping, GameMapping::NotFormalized);
    assert_eq!(exposed.roles, recorded_roles(&created));
    let rendered = json(&inspection);
    assert!(rendered.contains("\"factors-recorded\""));
    assert!(rendered.contains("\"not-formalized\""));
    assert!(contains_factor_names(&rendered));
}

fn recorded_roles(session: &SessionRecord) -> Vec<lattice::engine::protocol::RoleChoices> {
    record(&session.plan.cohort).roles.clone()
}

// r[verify lattice.role_protocol.preference_factors.recorded]
#[test]
fn recorded_factors_survive_persistence_and_operator_inspection() {
    let report = annotated_report(FACTOR_NAMES);
    let created = session(intake(bundle(&report), PreferenceProfile::RecordDeclaredFactors).cohort);
    let directory = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let store = lattice::store::Store::new(directory.path().join("protocol.redb"))
        .unwrap_or_else(|error| panic!("store failed: {error}"));
    <lattice::store::Store as SessionRepository>::create(&store, &created)
        .unwrap_or_else(|error| panic!("persist failed: {error:?}"));
    let loaded = <lattice::store::Store as SessionRepository>::load(&store, &created.plan.session)
        .unwrap_or_else(|error| panic!("load failed: {error:?}"))
        .unwrap_or_else(|| panic!("session missing"));
    assert_eq!(loaded, created);

    let inspected = lattice::protocol_cli::run(&store, &lattice::protocol_cli::CommandArgs {
        command: lattice::protocol_cli::CliCommand::Inspect {
            session: created.plan.session.as_str().to_owned(),
        },
    });
    assert!(inspected.success, "{}", inspected.output);
    assert!(inspected.output.contains("factors-recorded"));
    assert!(inspected.output.contains("not-formalized"));
    assert!(contains_factor_names(&inspected.output));
}

// r[verify lattice.role_protocol.preference_factors.default_off]
#[test]
fn default_profile_ignores_factors_and_matches_unannotated_admission() {
    assert_eq!(PreferenceProfile::default(), PreferenceProfile::ExactLabelBaseline);
    let report = annotated_report(FACTOR_NAMES);
    let annotated = bundle(&report);
    let ignored = intake(annotated, PreferenceProfile::default());
    assert_eq!(ignored.report.disposition, FactorDisposition::FactorsIgnoredProfileOff);
    assert_eq!(ignored.report.game_mapping, GameMapping::NotFormalized);
    assert!(ignored.cohort.preference.is_none());

    let unannotated = lattice::engine::protocol::admit_projection(lattice::engine::protocol::FrozenProjection {
        projected: annotated.projected(),
        bundle: annotated.base(),
    })
    .unwrap_or_else(|error| panic!("v1 intake failed: {error:?}"));
    assert_eq!(ignored.cohort, unannotated);
    assert_eq!(json(&ignored.cohort), json(&unannotated));
    assert!(!json(&ignored.cohort).contains("preference"));

    let ignored_session = session(ignored.cohort);
    let unannotated_session = session(unannotated);
    assert_eq!(json(&ignored_session), json(&unannotated_session));
    let inspection = inspect(&ignored_session, &[]).unwrap_or_else(|error| panic!("inspect failed: {error:?}"));
    assert!(inspection.preference.is_none());
    let rendered = json(&inspection);
    assert!(!rendered.contains("preference"));
    assert!(!contains_factor_names(&rendered));
    assert_eq!(
        rendered,
        json(&inspect(&unannotated_session, &[]).unwrap_or_else(|error| panic!("inspect failed: {error:?}")))
    );
}

// r[verify lattice.role_protocol.preference_factors.default_off]
#[test]
fn annotated_input_without_opt_in_records_nothing() {
    let source = annotated_source(common::protocol::choice(), Vec::new(), FACTOR_NAMES);
    let refused = compile(&source, choregraph_choreography::preference::ProfileSelection::default());
    assert_eq!(refused.status(), choregraph_choreography::preference::ProjectionStatus::Refused);
    assert!(refused.bundle().is_none());

    let unannotated = common::protocol::cohort(common::protocol::native_source(common::protocol::choice()));
    let placements = common::protocol::native_placements(&unannotated);
    let mut input = serde_json::to_value(lattice::protocol_cli::SessionCreateInput {
        choreography: common::protocol::native_source(common::protocol::choice()),
        placements,
        nonce: common::protocol::identity("cli-annotated"),
    })
    .unwrap_or_else(|error| panic!("encode failed: {error}"));
    input["choreography"] = serde_json::to_value(&source).unwrap_or_else(|error| panic!("encode failed: {error}"));
    assert!(serde_json::from_value::<lattice::protocol_cli::SessionCreateInput>(input).is_err());

    let mut forged = serde_json::to_value(&unannotated).unwrap_or_else(|error| panic!("encode failed: {error}"));
    forged["annotations"] = serde_json::to_value(&source.annotations).unwrap_or_else(|error| panic!("{error}"));
    assert!(serde_json::from_value::<Cohort>(forged).is_err());
}

// r[verify lattice.role_protocol.preference_factors.baseline_comparison]
#[test]
fn exact_label_decisions_ownership_and_authority_match_the_baseline() {
    let report = annotated_report(FACTOR_NAMES);
    let annotated = bundle(&report);
    let baseline = session(intake(annotated, PreferenceProfile::ExactLabelBaseline).cohort);
    let selected = session(intake(annotated, PreferenceProfile::RecordDeclaredFactors).cohort);
    assert!(selected.plan.cohort.preference.is_some());
    assert_eq!(baseline.cursors, selected.cursors);
    assert_eq!(baseline.owners, selected.owners);

    for (role, label) in [
        (common::protocol::ROLE_A, UNDECLARED_LABEL),
        (common::protocol::ROLE_A, FACTOR_COST),
        (common::protocol::ROLE_A, WORLD_MARKET),
        (common::protocol::ROLE_B, common::protocol::LABEL_A.0),
    ] {
        let expected = plan_action(&baseline, request(role, label)).map(|action| action.kind);
        assert!(expected.is_err());
        assert_eq!(plan_action(&selected, request(role, label)).map(|action| action.kind), expected);
    }
    for label in [common::protocol::LABEL_A.0, common::protocol::LABEL_B.0] {
        let baseline_action = plan_action(&baseline, request(common::protocol::ROLE_A, label));
        assert!(baseline_action.is_ok(), "declared label {label} must stay plannable");
        let selected_action = plan_action(&selected, request(common::protocol::ROLE_A, label));
        assert_eq!(
            baseline_action.map(|action| (action.kind, action.messages.len())),
            selected_action.map(|action| (action.kind, action.messages.len()))
        );
    }

    let (baseline_choice, baseline_envelopes) =
        commit(&baseline, request(common::protocol::ROLE_A, common::protocol::LABEL_B.0));
    let (selected_choice, selected_envelopes) =
        commit(&selected, request(common::protocol::ROLE_A, common::protocol::LABEL_B.0));
    assert_eq!(decisions(&baseline_choice), decisions(&selected_choice));
    assert!(!contains_factor_names(&json(&selected_envelopes)));
    assert_eq!(baseline_envelopes.len(), selected_envelopes.len());

    let (baseline_done, _) = commit(&baseline_choice, request(common::protocol::ROLE_A, common::protocol::LABEL_B.0));
    let (selected_done, _) = commit(&selected_choice, request(common::protocol::ROLE_A, common::protocol::LABEL_B.0));
    assert_eq!(decisions(&baseline_done), decisions(&selected_done));
    assert_eq!(selected_done.status, SessionStatus::Complete);
    assert_eq!(selected_done.plan.cohort.preference, selected.plan.cohort.preference);

    let (baseline_outcome, baseline_persisted, baseline_requests) =
        denied_dispatch(&baseline, common::protocol::LABEL_A.0);
    let (selected_outcome, selected_persisted, selected_requests) =
        denied_dispatch(&selected, common::protocol::LABEL_A.0);
    assert_eq!(baseline_outcome, Err(Error::Issue(Issue::AuthorityDenied)));
    assert_eq!(selected_outcome, baseline_outcome);
    assert_eq!(decisions(&baseline_persisted), decisions(&selected_persisted));
    assert_eq!(baseline_requests.len(), selected_requests.len());
    assert!(!selected_requests.iter().any(|envelope| contains_factor_names(envelope)));
}

// r[verify lattice.role_protocol.preference_factors.baseline_comparison]
#[test]
fn owned_transfer_before_the_choice_moves_ownership_identically_under_both_profiles() {
    let protocol = common::protocol::transfer(
        common::protocol::ROLE_A,
        common::protocol::ROLE_B,
        common::protocol::LABEL_A,
        common::protocol::choice(),
    );
    let report = compile(
        &annotated_source(protocol, vec![choregraph_choreography::preference::model::PathStep::Next], FACTOR_NAMES),
        choregraph_choreography::preference::ProfileSelection::PreferenceAnnotations,
    );
    let annotated = bundle(&report);
    let baseline = session(intake(annotated, PreferenceProfile::ExactLabelBaseline).cohort);
    let selected = session(intake(annotated, PreferenceProfile::RecordDeclaredFactors).cohort);
    let decision = &record(&selected.plan.cohort).roles[0].decisions[0];
    assert_eq!(decision.local_path, vec![lattice::engine::protocol::ChoiceStep::Next]);
    assert_eq!(decision.global_path, vec![lattice::engine::protocol::ChoiceStep::Next]);

    let (baseline_moved, _) = commit(&baseline, request(common::protocol::ROLE_A, common::protocol::LABEL_A.0));
    let (selected_moved, _) = commit(&selected, request(common::protocol::ROLE_A, common::protocol::LABEL_A.0));
    assert_eq!(decisions(&baseline_moved), decisions(&selected_moved));
    assert_eq!(selected_moved.owners[0].owner, common::protocol::lattice_role(common::protocol::ROLE_B));
    assert_ne!(selected_moved.owners, selected.owners);
    assert_eq!(
        plan_action(&selected_moved, request(common::protocol::ROLE_A, common::protocol::LABEL_A.0))
            .map(|action| action.kind),
        Ok(lattice::engine::protocol::ActionKind::Choice)
    );

    let mut baseline_current = baseline_moved;
    let mut selected_current = selected_moved;
    for _ in 0..2 {
        (baseline_current, _) =
            commit(&baseline_current, request(common::protocol::ROLE_A, common::protocol::LABEL_A.0));
        (selected_current, _) =
            commit(&selected_current, request(common::protocol::ROLE_A, common::protocol::LABEL_A.0));
        assert_eq!(decisions(&baseline_current), decisions(&selected_current));
    }
    assert_eq!(selected_current.status, SessionStatus::Complete);
    assert_eq!(selected_current.owners, baseline_current.owners);
}

// r[verify lattice.role_protocol.preference_factors.identity]
// r[verify lattice.role_protocol.preference_factors.identity.crossed]
#[test]
fn crossed_receipt_or_local_set_is_rejected() {
    let first = annotated_report(FACTOR_NAMES);
    let second = annotated_report(["risk", "latency", "market"]);
    let (first, second) = (bundle(&first), bundle(&second));
    assert_ne!(first.receipt().identity, second.receipt().identity);
    assert_eq!(first.base().local_set().identity, second.base().local_set().identity);

    for profile in [
        PreferenceProfile::ExactLabelBaseline,
        PreferenceProfile::RecordDeclaredFactors,
    ] {
        let crossed_receipt = AnnotatedProjection {
            receipt: second.receipt_body(),
            receipt_bytes: &second.receipt().packed_bytes,
            ..exact(first)
        };
        assert_eq!(admit_annotated_projection(crossed_receipt, profile), Err(Error::Issue(Issue::IdentityMismatch)));
        let crossed_bytes = AnnotatedProjection {
            receipt_bytes: &second.receipt().packed_bytes,
            ..exact(first)
        };
        assert_eq!(admit_annotated_projection(crossed_bytes, profile), Err(Error::Issue(Issue::IdentityMismatch)));
        let crossed_local_set = AnnotatedProjection {
            local_set: second.local_set_body(),
            ..exact(first)
        };
        assert_eq!(admit_annotated_projection(crossed_local_set, profile), Err(Error::Issue(Issue::IdentityMismatch)));
    }

    let mut moved = first.local_set_body().clone();
    let decision = moved.locals[0].decisions.remove(0);
    moved.locals[1].decisions.push(decision);
    let mut narrowed = first.local_set_body().clone();
    narrowed.locals[0].decisions[0].labels.truncate(1);
    let mut reassigned = first.local_set_body().clone();
    reassigned.locals[0].decisions[0].factors[0].role = common::protocol::ROLE_B;
    let mut reordered = first.local_set_body().clone();
    reordered.locals.reverse();
    for tampered in [moved, narrowed, reassigned, reordered] {
        let input = AnnotatedProjection {
            local_set: &tampered,
            ..exact(first)
        };
        assert_eq!(
            admit_annotated_projection(input, PreferenceProfile::RecordDeclaredFactors),
            Err(Error::Issue(Issue::IdentityMismatch))
        );
    }

    let mut relinked = first.receipt_body().clone();
    relinked.base_receipt = second.receipt_body().strict_dto.clone();
    let input = AnnotatedProjection {
        receipt: &relinked,
        ..exact(first)
    };
    assert_eq!(
        admit_annotated_projection(input, PreferenceProfile::RecordDeclaredFactors),
        Err(Error::Issue(Issue::IdentityMismatch))
    );
}

fn forged(edit: impl FnOnce(&mut lattice::engine::protocol::PreferenceRecord)) -> Result<(), Error> {
    let report = annotated_report(FACTOR_NAMES);
    let mut cohort = intake(bundle(&report), PreferenceProfile::RecordDeclaredFactors).cohort;
    edit(cohort.preference.as_mut().unwrap_or_else(|| panic!("annotated cohort has no record")));
    let placements = common::protocol::native_placements(&cohort);
    plan_session(cohort, placements, common::protocol::identity("forged-nonce")).map(|_| ())
}

// r[verify lattice.role_protocol.preference_factors.boundary]
// r[verify lattice.role_protocol.preference_factors.boundary.forged]
#[test]
fn factors_cannot_select_a_label_or_grant_authority() {
    assert_eq!(forged(|_| ()), Ok(()));
    assert_eq!(
        forged(|record| record.roles[0].decisions[0].labels.truncate(1)),
        Err(Error::Issue(Issue::IdentityMismatch))
    );
    assert_eq!(
        forged(|record| record.roles[0].decisions[0].labels.reverse()),
        Err(Error::Issue(Issue::IdentityMismatch))
    );
    assert_eq!(
        forged(|record| {
            record.roles[0].decisions[0].factors[0].role = common::protocol::lattice_role(common::protocol::ROLE_B);
        }),
        Err(Error::Issue(Issue::IdentityMismatch))
    );
    assert_eq!(
        forged(|record| {
            let decision = record.roles[0].decisions.remove(0);
            record.roles[1].decisions.push(decision);
        }),
        Err(Error::Issue(Issue::IdentityMismatch))
    );
    assert_eq!(
        forged(
            |record| record.roles[0].decisions[0].local_path.push(lattice::engine::protocol::ChoiceStep::Branch {
                label: LabelId(common::protocol::LABEL_A.0),
            })
        ),
        Err(Error::Issue(Issue::IdentityMismatch))
    );
    assert_eq!(
        forged(|record| record.roles[0].decisions[0].factors.clear()),
        Err(Error::Issue(Issue::IdentityMismatch))
    );
    assert_eq!(
        forged(|record| record.non_claims.retain(|claim| claim != "no-preference-authority")),
        Err(Error::Issue(Issue::UnsupportedCohort))
    );
    assert_eq!(
        forged(|record| record.producer_profile = choregraph_choreography::preference::BASELINE_PROFILE.to_owned()),
        Err(Error::Issue(Issue::UnsupportedCohort))
    );
    assert_eq!(
        forged(|record| record.base_receipt = common::protocol::identity("stale-receipt")),
        Err(Error::Issue(Issue::IdentityMismatch))
    );

    let report = annotated_report(FACTOR_NAMES);
    let cohort = intake(bundle(&report), PreferenceProfile::RecordDeclaredFactors).cohort;
    let encoded = serde_json::to_value(&cohort).unwrap_or_else(|error| panic!("encode failed: {error}"));
    for (pointer, field) in [
        ("/preference/roles/0/decisions/0", "selected_label"),
        ("/preference/roles/0/decisions/0/factors/0", "weight"),
        ("/preference", "authority"),
        ("/preference", "utility"),
    ] {
        let mut candidate = encoded.clone();
        let target = candidate.pointer_mut(pointer).unwrap_or_else(|| panic!("fixture lacks {pointer}"));
        target[field] = serde_json::json!(common::protocol::LABEL_A.0);
        assert!(serde_json::from_value::<Cohort>(candidate).is_err(), "{pointer}/{field} was admitted");
    }
    assert_eq!(serde_json::from_value::<Cohort>(encoded).ok(), Some(cohort));
}
