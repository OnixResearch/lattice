//! Workflow-plus-agent revisions (Lattice #25, "Pin compiled workflow and
//! agent revisions to Cairn campaigns") at fixture level: one immutable
//! bundle binds a compiled workflow-intent plan and an agent profile
//! revision, and the existing lifecycle manages it.

use lattice_core::agent_revision::Refusal;
use lattice_core::agent_revision::facts::Bundle;
use lattice_core::agent_revision::ops::prepare;
use lattice_core::agent_revision::ops::selection as select;
use lattice_core::agent_revision::ops::sessions;
use lattice_core::agent_revision::ports as verdict;
use lattice_core::agent_revision::state::Lifecycle;
use lattice_core::agent_revision::state::Preparation;
use lattice_core::agent_revision::state::SelectionKind;
use lattice_core::agent_revision::workflow;
use lattice_core::agent_revision::workflow::adaptation;
use lattice_core::agent_revision::workflow::publication;

const FAMILY: &str = "rust-implementation";
const NAME: &str = "verify-change";

fn plan_digest(value: &str) -> String {
    format!("b3:{}", blake3::hash(value.as_bytes()).to_hex())
}

fn reference(value: &str) -> String {
    format!("blake3:{}", blake3::hash(value.as_bytes()).to_hex())
}

fn obligation(id: &str) -> workflow::Obligation {
    workflow::Obligation {
        id: id.into(),
        obligation_identity: plan_digest(&format!("obligation:{id}")),
        guidance_identity: plan_digest(&format!("guidance:{id}")),
        remaining_condition_identity: plan_digest(&format!("remaining:{id}")),
    }
}

/// A compiled plan's identities; `variant` stands for a recompiled plan.
fn plan(variant: &str) -> workflow::Plan {
    workflow::Plan {
        schema: "lattice.workflow-intent-plan.v1".into(),
        compiler_revision: "lattice.workflow-intent-compiler.v1".into(),
        compiler_artifact_identity: plan_digest("compiler-artifact"),
        identity: plan_digest(&format!("plan:{variant}")),
        graph_identity: reference(&format!("graph:{variant}")),
        runtime_profile_identity: plan_digest("runtime-profile"),
        cohort: workflow::Cohort {
            producer_identity: plan_digest("producer"),
            selection_identity: plan_digest("selection"),
            obligations_identity: plan_digest("obligations"),
            profile_identity: plan_digest("profile"),
            context_identity: plan_digest("context"),
            source_identity: plan_digest("source"),
            policy_identity: plan_digest("policy"),
            dependency_identity: plan_digest("dependencies"),
        },
        obligations: vec![obligation("A"), obligation("B")],
    }
}

fn worker(revision: char) -> Bundle {
    Bundle {
        schema: lattice_core::agent_revision::facts::BUNDLE_SCHEMA.into(),
        definition: format!("blake3:{}", "d".repeat(64)),
        definition_name: "rust-implementer".into(),
        revision: revision.to_string().repeat(64),
        adapter: "omp".into(),
        protocol_revision: "e325ffc73ab86e03bdd20a91b77d8e2b3964c906".into(),
        executable_identity: "e".repeat(64),
        compute: lattice_core::execution_binding::request::ComputeRequirement::HostLocal {},
        task_families: Some(vec![FAMILY.into()]),
        tools: Some(vec!["edit".into(), "read".into()]),
        routes: vec![lattice_core::agent_revision::facts::HostedRoute {
            name: "primary".into(),
            route_identity: reference("route"),
            provider_model: "model".into(),
            weight_identity: lattice_core::agent_revision::facts::Unexposed::Unknown,
            model_version: lattice_core::agent_revision::facts::Unexposed::Unknown,
        }],
        workflow: None,
    }
}

fn compose(plan: workflow::Plan, worker: &Bundle) -> Bundle {
    workflow::compose(NAME, plan, worker).expect("composes")
}

/// R1: plan v1 with worker 1. R2: plan v2 (a changed plan). R3: plan v1
/// with worker 3 (a changed agent profile).
fn fixtures() -> [Bundle; 3] {
    [
        compose(plan("v1"), &worker('1')),
        compose(plan("v2"), &worker('1')),
        compose(plan("v1"), &worker('3')),
    ]
}

fn ok<T>(result: Result<lattice_core::agent_revision::ops::Applied<T>, Refusal>) -> (Lifecycle, T) {
    let applied = result.expect("transition applies");
    (applied.lifecycle, applied.outcome)
}

fn attachment(bundle: &Bundle, qualification_ref: &str) -> verdict::Attachment {
    verdict::Attachment {
        qualification_ref: qualification_ref.into(),
        subject: verdict::Subject {
            definition_identity: bundle.definition.clone(),
            profile_revision: bundle.revision.clone(),
            task_family: FAMILY.into(),
        },
        evidence_class: verdict::EvidenceClass::TaskFamilyEvaluation,
        protocol_identity: reference("protocol"),
        verifier_identity: reference("verifier"),
        observed_at_ms: 1_795_200_000_000,
        provenance: verdict::Provenance::Fixture,
    }
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

fn started(lifecycle: &Lifecycle, bundle: &Bundle) -> (Lifecycle, prepare::Ticket) {
    let (desired, _) = ok(prepare::desire(lifecycle, bundle));
    match ok(prepare::begin(&desired, &bundle.revision)) {
        (next, prepare::Begin::Started { ticket }) => (next, ticket),
        (_, other) => panic!("expected a started preparation, got {other:?}"),
    }
}

fn prepared(lifecycle: &Lifecycle, bundle: &Bundle) -> Lifecycle {
    let (pending, ticket) = started(lifecycle, bundle);
    let observation = verdict::Prepared::Prepared {
        artifacts: vec![reference(&format!("artifact:{}", bundle.revision))],
    };
    ok(prepare::record(&pending, &ticket, &observation)).0
}

fn promoted(lifecycle: &Lifecycle, request: &str, bundle: &Bundle) -> Lifecycle {
    let qualification_ref = reference(&format!("qualification:{}", bundle.revision));
    let promotion = command(
        SelectionKind::Promotion,
        request,
        &bundle.revision,
        &qualification_ref,
        lifecycle.rollout_generation,
    );
    let answer = verdict::Verdict::Qualified(attachment(bundle, &qualification_ref));
    ok(select::select(lifecycle, &promotion, &answer)).0
}

fn admitted(lifecycle: &Lifecycle, session: &str, revision: &str) -> Lifecycle {
    ok(sessions::admit(lifecycle, sessions::Entry { session, revision })).0
}

/// A on R1; R2 failed preparation; B admitted on R1.
fn r2_failed() -> Lifecycle {
    let [r1, r2, _] = fixtures();
    let start = Lifecycle::new(&r1.definition);
    let with_a = admitted(&promoted(&prepared(&start, &r1), "promote-r1", &r1), "session-a", &r1.revision);
    let (pending, ticket) = started(&with_a, &r2);
    let failure = verdict::Prepared::Failed { artifacts: Vec::new() };
    let failed = ok(prepare::record(&pending, &ticket, &failure)).0;
    admitted(&failed, "session-b", &r1.revision)
}

#[test]
// r[verify lattice.agent_revisions.workflow]
// r[verify lattice.agent_revisions.workflow.scenario.identity]
fn the_bundle_binds_the_plan_and_the_worker_revision() {
    let [r1, r2, r3] = fixtures();
    assert_eq!(r1.schema, workflow::BUNDLE_SCHEMA);
    assert_eq!(r1, compose(plan("v1"), &worker('1')), "composition is deterministic");
    assert!(lattice_core::agent_revision::is_bare_digest(&r1.revision));
    let revisions = [&r1.revision, &r2.revision, &r3.revision];
    assert_eq!(revisions.iter().collect::<std::collections::BTreeSet<_>>().len(), 3);
    assert!(r1.definition == r2.definition && r1.definition == r3.definition, "one definition across revisions");
    assert_ne!(r1.definition, worker('1').definition, "never a worker definition");
    let binding = workflow::binding(&r3).expect("valid");
    assert_eq!(binding.worker.revision, worker('3').revision);
    assert_eq!(
        binding.worker.bundle_identity,
        lattice_core::agent_revision::facts::identity(&worker('3')).expect("worker identity")
    );
    assert_eq!(workflow::worker_bundle(&r3, binding), worker('3'));
    let plain = serde_json::to_value(worker('1')).expect("encodes");
    assert!(plain.get("workflow").is_none(), "a plain bundle keeps its bytes");
    let decoded: Bundle = serde_json::from_value(serde_json::to_value(&r1).expect("encodes")).expect("decodes");
    assert_eq!(decoded, r1);
}

#[test]
// r[verify lattice.agent_revisions.workflow]
fn a_bundle_that_disagrees_with_its_binding_is_refused() {
    let [r1, ..] = fixtures();
    let mut moved_plan = r1.clone();
    moved_plan.workflow.as_mut().expect("binding").plan.identity = plan_digest("plan:other");
    let mut moved_worker = r1.clone();
    moved_worker.executable_identity = "9".repeat(64);
    let mut renamed = r1.clone();
    renamed.definition_name = "other".into();
    let mut plain_schema = r1.clone();
    plain_schema.schema = lattice_core::agent_revision::facts::BUNDLE_SCHEMA.into();
    let mut stripped = r1.clone();
    stripped.workflow = None;
    let mut unsorted = r1.clone();
    unsorted.workflow.as_mut().expect("binding").plan.obligations.reverse();
    let mut raw_plan = r1.clone();
    raw_plan.workflow.as_mut().expect("binding").plan.identity = "plan".into();
    for tampered in [
        moved_plan,
        moved_worker,
        renamed,
        plain_schema,
        stripped,
        unsorted,
        raw_plan,
    ] {
        assert_eq!(lattice_core::agent_revision::facts::validate(&tampered).unwrap_err(), Refusal::Malformed);
        assert!(prepare::desire(&Lifecycle::new(&r1.definition), &tampered).is_err());
    }
    assert_eq!(workflow::compose(NAME, plan("v1"), &r1).unwrap_err(), Refusal::Malformed, "no nesting");
    let mut empty = plan("v1");
    empty.obligations.clear();
    assert_eq!(workflow::compose(NAME, empty, &worker('1')).unwrap_err(), Refusal::Malformed);
    assert_eq!(workflow::compose("", plan("v1"), &worker('1')).unwrap_err(), Refusal::Malformed);
    assert_eq!(
        prepare::desire(&Lifecycle::new(&r1.definition), &worker('1')).unwrap_err(),
        Refusal::DefinitionMismatch,
        "a plain worker bundle never enters a workflow-agent lifecycle"
    );
}

#[test]
// r[verify lattice.agent_revisions.workflow.scenario.lifecycle]
fn a_failed_r2_leaves_r1_usable_and_promoted_r3_serves_new_sessions() {
    let [r1, r2, r3] = fixtures();
    let lifecycle = r2_failed();
    assert_eq!(lifecycle.last_usable.as_deref(), Some(r1.revision.as_str()));
    assert_eq!(lifecycle.selected.as_ref().map(|selected| &selected.revision), Some(&r1.revision));
    assert!(matches!(lifecycle.candidates[&r2.revision].preparation, Preparation::Failed { .. }));
    assert_eq!(lifecycle.sessions["session-b"].revision, r1.revision);
    let with_r3 = promoted(&prepared(&lifecycle, &r3), "promote-r3", &r3);
    let with_c = admitted(&with_r3, "session-c", &r3.revision);
    assert_eq!(with_c.sessions["session-a"].revision, r1.revision, "A keeps its exact graph and worker");
    assert_eq!(with_c.sessions["session-c"].revision, r3.revision);
    assert_eq!(
        sessions::admit(&with_c, sessions::Entry {
            session: "session-d",
            revision: &r1.revision,
        })
        .unwrap_err(),
        Refusal::RevisionNotSelected
    );
    let rollback_ref = reference(&format!("qualification:{}", r1.revision));
    let rollback = command(SelectionKind::Rollback, "rollback-r1", &r1.revision, &rollback_ref, 2);
    let answer = verdict::Verdict::Qualified(attachment(&r1, &rollback_ref));
    let (rolled, _) = ok(select::select(&with_c, &rollback, &answer));
    assert_eq!(rolled.selected.as_ref().map(|selected| &selected.revision), Some(&r1.revision));
    assert_eq!(rolled.sessions["session-c"].revision, r3.revision, "rollback never rewrites a pin");
}

#[test]
// r[verify lattice.agent_revisions.workflow.scenario.lifecycle]
fn a_late_r2_result_cannot_override_r3() {
    let [r1, r2, r3] = fixtures();
    let start = promoted(&prepared(&Lifecycle::new(&r1.definition), &r1), "promote-r1", &r1);
    let (abandoned, late_ticket) = started(&start, &r2);
    let with_r3 = promoted(&prepared(&abandoned, &r3), "promote-r3", &r3);
    let late = verdict::Prepared::Prepared {
        artifacts: vec![reference("late")],
    };
    let (after, _) = ok(prepare::record(&with_r3, &late_ticket, &late));
    assert!(matches!(after.candidates[&r2.revision].preparation, Preparation::Superseded { .. }));
    assert_eq!(after.selected, with_r3.selected);
    assert_eq!(after.rollout_generation, with_r3.rollout_generation);
    assert_eq!(after.last_usable.as_deref(), Some(r3.revision.as_str()));
    let qualification_ref = reference("late-r2");
    let promote = command(SelectionKind::Promotion, "promote-r2", &r2.revision, &qualification_ref, 2);
    let answer = verdict::Verdict::Qualified(attachment(&r2, &qualification_ref));
    assert_eq!(select::select(&after, &promote, &answer).unwrap_err(), Refusal::PreparationSuperseded);
}

#[test]
// r[verify lattice.agent_revisions.workflow.scenario.evaluation]
fn a_positive_prior_evaluation_does_not_approve_another_intent() {
    let [r1, r2, _] = fixtures();
    let lifecycle = prepared(&promoted(&prepared(&Lifecycle::new(&r1.definition), &r1), "promote-r1", &r1), &r2);
    let r1_ref = reference(&format!("qualification:{}", r1.revision));
    let reuse = command(SelectionKind::Promotion, "promote-r2", &r2.revision, &r1_ref, 1);
    let r1_answer = verdict::Verdict::Qualified(attachment(&r1, &r1_ref));
    assert_eq!(select::select(&lifecycle, &reuse, &r1_answer).unwrap_err(), Refusal::QualificationMismatch);
    let fresh_ref = reference("fresh");
    let stale_subject = command(SelectionKind::Promotion, "promote-r2", &r2.revision, &fresh_ref, 1);
    let mut for_r1 = attachment(&r1, &fresh_ref);
    let answer = verdict::Verdict::Qualified(for_r1.clone());
    assert_eq!(select::select(&lifecycle, &stale_subject, &answer).unwrap_err(), Refusal::NotQualified {
        reason: verdict::Unqualified::SubjectMismatch,
    });
    for_r1.subject.definition_identity = worker('1').definition;
    for_r1.subject.profile_revision = worker('1').revision;
    let worker_only = verdict::Verdict::Qualified(for_r1);
    assert_eq!(
        select::select(&lifecycle, &stale_subject, &worker_only).unwrap_err(),
        Refusal::NotQualified {
            reason: verdict::Unqualified::SubjectMismatch,
        },
        "the worker's own evaluation does not approve a workflow intent"
    );
    let own = verdict::Verdict::Qualified(attachment(&r2, &fresh_ref));
    assert!(select::select(&lifecycle, &stale_subject, &own).is_ok());
}

/// One proposed adaptation of R1 per kind the issue names.
fn adaptations(r1: &Bundle) -> Vec<(&'static str, Bundle, adaptation::Change)> {
    let mut changed_plan = plan("v1");
    changed_plan.identity = plan_digest("plan:adapted");
    changed_plan.obligations.push(obligation("C"));
    let mut prompt = plan("prompt");
    prompt.obligations[0].guidance_identity = plan_digest("guidance:A:2");
    let mut requirement = plan("requirement");
    requirement.obligations[1].remaining_condition_identity = plan_digest("remaining:B:2");
    let mut policy = plan("policy");
    policy.cohort.policy_identity = plan_digest("policy:2");
    let mut tools = worker('1');
    tools.tools = Some(vec!["edit".into(), "read".into(), "write".into()]);
    tools.revision = "4".repeat(64);
    assert_eq!(workflow::binding(r1).expect("valid").plan, plan("v1"));
    vec![
        ("plan", compose(changed_plan, &worker('1')), adaptation::Change::Requirements),
        ("prompt", compose(prompt, &worker('1')), adaptation::Change::Prompt),
        ("requirement", compose(requirement, &worker('1')), adaptation::Change::Requirements),
        ("policy", compose(policy, &worker('1')), adaptation::Change::Policy),
        ("tool", compose(plan("v1"), &tools), adaptation::Change::Tools),
    ]
}

#[test]
// r[verify lattice.agent_revisions.workflow.adaptation]
// r[verify lattice.agent_revisions.workflow.adaptation.scenario.pinned]
fn an_adaptation_gets_a_new_identity_and_never_mutates_an_active_pin() {
    let [r1, ..] = fixtures();
    let active = admitted(
        &promoted(&prepared(&Lifecycle::new(&r1.definition), &r1), "promote-r1", &r1),
        "session-a",
        &r1.revision,
    );
    for (kind, proposed, expected) in adaptations(&r1) {
        let proposal = adaptation::propose(&r1, &proposed).expect("proposes");
        let adaptation::Proposal::Adaptation {
            schema,
            active_revision,
            proposed_revision,
            changes,
            applicability,
            ..
        } = proposal
        else {
            panic!("{kind}: expected an adaptation");
        };
        assert_eq!(schema, adaptation::PROPOSAL_SCHEMA);
        assert_eq!((active_revision, proposed_revision.clone()), (r1.revision.clone(), proposed.revision.clone()));
        assert_ne!(proposed_revision, r1.revision, "{kind}: a new identity");
        assert!(changes.contains(&expected), "{kind}: {changes:?}");
        assert_eq!(applicability.decision, adaptation::Decision::Unknown);
        assert_eq!(applicability.owners, ["cairn#14", "kiln#83"]);
        let (desired, _) = ok(prepare::desire(&active, &proposed));
        assert_eq!(desired.selected, active.selected, "{kind}: the selection is unchanged");
        assert_eq!(desired.sessions, active.sessions, "{kind}: every pin is unchanged");
        assert_eq!(sessions::effect_permitted(&desired, "session-a").expect("A acts").revision, r1.revision);
    }
}

#[test]
// r[verify lattice.agent_revisions.workflow.adaptation]
fn a_proposal_is_refused_or_unchanged_outside_one_definition() {
    let [r1, r2, r3] = fixtures();
    assert_eq!(adaptation::propose(&r1, &r1.clone()).expect("unchanged"), adaptation::Proposal::Unchanged {
        revision: r1.revision.clone(),
    });
    let other = workflow::compose("other-pairing", plan("v1"), &worker('1')).expect("composes");
    assert_eq!(adaptation::propose(&r1, &other).unwrap_err(), Refusal::DefinitionMismatch);
    assert_eq!(adaptation::propose(&r1, &worker('1')).unwrap_err(), Refusal::Malformed);
    let adaptation::Proposal::Adaptation { changes, .. } = adaptation::propose(&r1, &r2).expect("proposes") else {
        panic!("expected an adaptation");
    };
    assert_eq!(changes, [adaptation::Change::Plan, adaptation::Change::Graph]);
    let adaptation::Proposal::Adaptation { changes, .. } = adaptation::propose(&r1, &r3).expect("proposes") else {
        panic!("expected an adaptation");
    };
    assert_eq!(changes, [adaptation::Change::WorkerRevision]);
    let decided = serde_json::json!({ "decision": "applicable", "owners": ["kiln#83"] });
    assert!(serde_json::from_value::<adaptation::Applicability>(decided).is_err(), "Lattice records no decision");
}

#[test]
// r[verify lattice.agent_revisions.workflow.projection]
fn the_kiln_projection_keeps_workflow_and_worker_revisions_apart() {
    let [r1, _, r3] = fixtures();
    let lifecycle = admitted(&promoted(&prepared(&r2_failed(), &r3), "promote-r3", &r3), "session-c", &r3.revision);
    let slots = publication::slots(&lifecycle).expect("projects").expect("workflow slots");
    assert_eq!(slots.schema, publication::SLOTS_SCHEMA);
    assert_eq!(slots.name, NAME);
    let selected = slots.selected_for_new_sessions.as_ref().expect("selected");
    assert_eq!(selected.bundle.revision, r3.revision);
    assert_eq!(selected.bundle.worker.revision, worker('3').revision);
    assert_eq!(selected.bundle.workflow.plan_identity, plan("v1").identity);
    let pinned = slots
        .pinned
        .iter()
        .map(|pinned| (pinned.session.as_str(), pinned.bundle.worker.revision.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(pinned, [
        ("session-a", worker('1').revision.as_str()),
        ("session-b", worker('1').revision.as_str()),
        ("session-c", worker('3').revision.as_str()),
    ]);
    assert_eq!(slots.last_usable.as_ref().map(|entry| &entry.revision), Some(&r3.revision));
    let encoded = serde_json::to_string(&slots).expect("encodes");
    for restricted in [
        &plan("v1").cohort.context_identity,
        &plan("v1").obligations[0].guidance_identity,
    ] {
        assert!(!encoded.contains(restricted.as_str()), "projection leaks {restricted}");
    }
    let decoded: publication::Slots = serde_json::from_str(&encoded).expect("decodes");
    assert_eq!(decoded, slots);
    let (revoked, _) = ok(sessions::revoke(&lifecycle, &sessions::Target::Revision(r1.revision.clone())));
    let blocked = publication::slots(&revoked).expect("projects").expect("slots");
    assert!(
        blocked
            .pinned
            .iter()
            .filter(|pinned| pinned.bundle.revision == r1.revision)
            .all(|pinned| pinned.is_blocked)
    );
    assert!(
        blocked
            .pinned
            .iter()
            .filter(|pinned| pinned.bundle.revision == r3.revision)
            .all(|pinned| !pinned.is_blocked)
    );
}

#[test]
// r[verify lattice.agent_revisions.workflow.projection]
fn the_projection_is_absent_for_plain_lifecycles_and_refuses_inconsistency() {
    let plain = Lifecycle::new(&worker('1').definition);
    let (plain, _) = ok(prepare::desire(&plain, &worker('1')));
    assert_eq!(publication::slots(&plain).expect("projects"), None);
    let [r1, ..] = fixtures();
    let (mut corrupted, _) = ok(prepare::desire(&Lifecycle::new(&r1.definition), &r1));
    corrupted
        .candidates
        .get_mut(&r1.revision)
        .expect("candidate")
        .bundle
        .workflow
        .as_mut()
        .expect("binding")
        .name = "renamed".into();
    assert_eq!(publication::slots(&corrupted).unwrap_err(), Refusal::Malformed);
    let (mut missing, _) = ok(prepare::desire(&Lifecycle::new(&r1.definition), &r1));
    missing.last_usable = Some("9".repeat(64));
    assert_eq!(publication::slots(&missing).unwrap_err(), Refusal::UnknownRevision);
}

#[test]
// r[verify lattice.agent_revisions.workflow.attempt]
fn an_attempt_runs_only_the_bound_worker_revision() {
    let [r1, _, r3] = fixtures();
    assert_eq!(workflow::check_attempt(&r1, &worker('1').revision), Ok(()));
    assert_eq!(workflow::check_attempt(&r3, &worker('1').revision).unwrap_err(), Refusal::RevisionNotSelected);
    assert_eq!(workflow::check_attempt(&r1, "not-a-revision").unwrap_err(), Refusal::Malformed);
    assert_eq!(workflow::check_attempt(&worker('1'), &worker('1').revision).unwrap_err(), Refusal::Malformed);
}

#[test]
// r[verify lattice.agent_revisions.workflow.scenario.identity]
fn fixture_identities_are_stable_and_recorded() {
    let [r1, r2, r3] = fixtures();
    let entry = |bundle: &Bundle| {
        serde_json::json!({
            "revision": bundle.revision,
            "bundle_identity": lattice_core::agent_revision::facts::identity(bundle).expect("identity"),
            "plan_identity": workflow::binding(bundle).expect("valid").plan.identity,
            "worker_revision": workflow::binding(bundle).expect("valid").worker.revision,
        })
    };
    let identities = serde_json::json!({
        "schema": workflow::BUNDLE_SCHEMA,
        "definition": r1.definition,
        "R1": entry(&r1),
        "R2": entry(&r2),
        "R3": entry(&r3),
        "live_qualification": false,
    });
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/agent-revisions/workflow-agent-identities.json");
    let recorded: serde_json::Value =
        serde_json::from_slice(&std::fs::read(golden).expect("golden identities read")).expect("golden decodes");
    assert_eq!(recorded, identities, "computed: {}", serde_json::to_string_pretty(&identities).expect("encode"));
}
