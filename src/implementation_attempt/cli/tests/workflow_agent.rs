//! Attempts that name a workflow-agent revision (Lattice #25) over the real
//! `agent-revision` and `implementation-attempt` CLI shells.
//!
//! The workflow-agent bundle is composed in process from a fixture plan and
//! the attempt's own worker bundle, desired, prepared through the fixture
//! port and promoted with an Animus #18 fixture attachment for the pair.

// r[verify lattice.agent_revisions.workflow.attempt]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.subject]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.refused]

use super::revisions::Fixture;

type Request = super::super::Request;
type Refusal = lattice_core::agent_revision::Refusal;
type Reason = super::super::refusal::Reason;

fn digest(label: &str) -> String {
    format!("b3:{}", blake3::hash(label.as_bytes()).to_hex())
}

/// A one-obligation fixture plan; another `guidance` is another plan.
fn compiled(guidance: &str) -> crate::workflow::intent::CompiledPlan {
    use crate::workflow::intent as wi;
    let cohort = wi::SourceCohort {
        producer_identity: digest("producer"),
        selection_identity: digest("selection"),
        obligations_identity: digest("obligations"),
        profile_identity: digest("profile"),
        context_identity: digest("context"),
        source_identity: digest("source"),
        policy_identity: digest("policy"),
        dependency_identity: digest("dependencies"),
    };
    let obligation = wi::Obligation {
        id: "A".into(),
        obligation_identity: digest("obligation:A"),
        guidance_identity: digest(&format!("{guidance}:A")),
        remaining_condition_identity: digest("remaining:A"),
        precondition_identity: digest("precondition:A"),
        operation_class: "cairn_validate".into(),
        action_identity: digest("action:A"),
        prerequisites: Vec::new(),
        required_observations: vec![wi::ObservationContract {
            id: "evidence-A".into(),
            schema: "lattice.workflow-intent-fixture-result.v1".into(),
            predicate_identity: digest("predicate:A"),
        }],
        required_capabilities: std::collections::BTreeSet::from(["native-cairn".into()]),
    };
    wi::compile(wi::CompileRequest {
        intent: wi::Definition {
            schema: wi::INTENT_SCHEMA.into(),
            cohort: cohort.clone(),
            coverage: wi::ContextCoverage::Complete,
            unsupported_fields: Vec::new(),
            obligations: vec![obligation],
            alternatives: Vec::new(),
            selected_alternative: None,
        },
        facts: wi::CompilerFacts {
            expected_cohort: cohort,
            compiler_artifact_identity: digest("compiler"),
            runtime_profile_identity: digest("runtime-profile"),
            available_capabilities: std::collections::BTreeSet::from(["native-cairn".into()]),
            available_implementations: std::collections::BTreeSet::new(),
        },
    })
    .expect("the fixture plan compiles")
}

/// Compose the workflow-agent bundle pairing the fixture plan with
/// `worker`'s own bundle, then desire and prepare it; returns its definition
/// and revision.
fn desired(fixture: &Fixture, worker: &Request) -> (String, String) {
    desired_as(fixture, worker, "verify-change", "guidance")
}

/// [`desired`] under pairing `name` with plan `guidance`.
fn desired_as(fixture: &Fixture, worker: &Request, name: &str, guidance: &str) -> (String, String) {
    let resolved = crate::agent_revision::bundle::resolve(worker).expect("resolves");
    let composed = crate::agent_revision::composite::compose(name, &compiled(guidance), resolved).expect("composes");
    let store = crate::store::Store::new(&fixture.store).expect("store");
    crate::agent_revision::transact(&store, &composed.definition, crate::agent_revision::Seed::Create, |lifecycle| {
        lattice_core::agent_revision::ops::prepare::desire(lifecycle, &composed.bundle)
    })
    .expect("commits")
    .expect("desired");
    drop(store);
    fixture
        .revision(&[
            "prepare",
            "--definition",
            &composed.definition,
            "--revision",
            &composed.revision,
        ])
        .expect("prepare through the fixture port");
    (composed.definition, composed.revision)
}

/// `worker` naming the workflow-agent `definition` and `revision`.
fn named(mut worker: Request, definition: &str, revision: &str) -> Request {
    worker.workflow_agent = Some(crate::implementation_attempt::WorkflowAgent {
        definition: definition.into(),
        revision: revision.into(),
    });
    worker
}

fn stored(fixture: &Fixture, definition: &str) -> Option<lattice_core::agent_revision::state::Lifecycle> {
    let store = crate::store::Store::new(&fixture.store).expect("store");
    store.load_lifecycle(definition).expect("load").map(|(_, lifecycle)| lifecycle)
}

fn admit(fixture: &Fixture, request: &Request, session: &str) -> crate::agent_revision::seam::Decision {
    let store = crate::store::Store::new(&fixture.store).expect("store");
    crate::agent_revision::seam::admit(&store, request, session).expect("seam")
}

fn effect(fixture: &Fixture, request: &Request, session: &str) -> crate::agent_revision::seam::Effect {
    let store = crate::store::Store::new(&fixture.store).expect("store");
    crate::agent_revision::seam::effect(&store, request, session, crate::agent_revision::seam::Admission::Managed)
        .expect("seam")
}

/// Refused under the per-work lock after the fence record and before any
/// binding, intent or session pin: the refusal closed the attempt-fence.
fn assert_refused_before_any_intent(fixture: &Fixture, request: &Request, definition: &str) {
    super::revisions::assert_not_started(&fixture.directory(request));
    let fence = fixture
        .state
        .join("works")
        .join(blake3::hash(request.envelope.work_id.as_bytes()).to_hex().as_str());
    let recorded = std::fs::read(fence.join(format!("{:020}", request.envelope.fence))).expect("fence record");
    assert_eq!(recorded, request.envelope.attempt_id.as_bytes(), "the fence precedes admission");
    assert!(
        stored(fixture, definition).is_none_or(|lifecycle| lifecycle.sessions.is_empty()),
        "nothing is pinned"
    );
}

/// Refused as an error before the state root is touched.
fn assert_untouched(fixture: &Fixture, request: &Request, definition: &str) {
    assert!(!fixture.directory(request).exists(), "no attempt directory");
    assert!(
        stored(fixture, definition).is_none_or(|lifecycle| lifecycle.sessions.is_empty()),
        "nothing is pinned"
    );
}

#[test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.absent]
fn an_absent_workflow_agent_keeps_request_bytes_identity_and_revision() {
    let plain = super::host_local();
    let bytes = serde_json::to_vec(&plain).expect("bytes");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert!(value.get("workflow_agent").is_none(), "an absent field stays absent");
    let decoded: Request = serde_json::from_slice(&bytes).expect("decodes");
    assert_eq!(serde_json::to_vec(&decoded).expect("re-encode"), bytes, "byte-identical round trip");
    let resolved = crate::agent_revision::bundle::resolve(&plain).expect("resolves");
    assert_eq!(
        crate::agent_revision::seam::subject(&plain).expect("subject"),
        crate::agent_revision::seam::Subject {
            definition: resolved.definition.identity,
            revision: resolved.revision,
        }
    );

    let definition = format!("blake3:{}", "1".repeat(64));
    let pair = named(plain.clone(), &definition, &"2".repeat(64));
    pair.validate(1).expect("a well-formed pair validates");
    let revision = plain.worker_profile_identity().expect("revision");
    assert_eq!(pair.worker_profile_identity().expect("revision"), revision, "the pair is per-attempt");
    assert_eq!(pair.envelope.identity(), plain.envelope.identity(), "the envelope is unchanged");
    let pair_bytes = serde_json::to_vec(&pair).expect("bytes");
    assert_ne!(blake3::hash(&pair_bytes), blake3::hash(&bytes), "the exact request bytes bind the pair");
    let encoded: serde_json::Value = serde_json::from_slice(&pair_bytes).expect("json");
    assert_eq!(encoded["workflow_agent"], serde_json::json!({"definition": definition, "revision": "2".repeat(64)}));
    let decoded: Request = serde_json::from_slice(&pair_bytes).expect("decodes");
    assert_eq!(decoded.workflow_agent, pair.workflow_agent);
    assert_eq!(
        crate::agent_revision::seam::subject(&pair).expect("subject"),
        crate::agent_revision::seam::Subject {
            definition: definition.clone(),
            revision: "2".repeat(64),
        }
    );

    let mut extra = encoded;
    extra["workflow_agent"]["worker"] = serde_json::Value::from("fabricated");
    assert!(serde_json::from_value::<Request>(extra).is_err(), "unknown pair fields are refused");
    for (definition, revision) in [
        ("1".repeat(64), "2".repeat(64)),
        (format!("blake3:{}", "A".repeat(64)), "2".repeat(64)),
        (definition.clone(), "2".repeat(63)),
        (definition.clone(), format!("blake3:{}", "2".repeat(64))),
    ] {
        let malformed = named(plain.clone(), &definition, &revision);
        let error = malformed.validate(1).expect_err("malformed pair");
        assert!(error.contains("workflow-agent"), "{error}");
    }
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.subject]
async fn a_matching_bundle_admits_the_attempt_under_the_pair_not_the_worker() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-workflow", 7);
    let (worker_path, _) = fixture.write("worker.json", &worker);
    // The worker definition is managed with nothing selected, so the worker
    // alone admits no session.
    let (worker_definition, _) = fixture.prepared(&worker, &worker_path);
    let refused = fixture.attempt("run", &worker_path, true).await;
    assert_eq!(refused, Err(Reason::RevisionNotAdmitted.message().to_owned()));

    let (definition, revision) = desired(&fixture, &worker);
    assert_ne!(definition, worker_definition);
    let qualification_ref = fixture.promoted(&definition, &revision);
    // The refused worker attempt recorded fence 7 before its admission.
    let request = named(fixture.request("attempt-workflow-pair", 8), &definition, &revision);
    let (request_path, identity) = fixture.write("pair.json", &request);
    fixture
        .attempt("run", &request_path, true)
        .await
        .expect("the pair admits; the worker reaches a terminal result");

    let directory = fixture.directory(&request);
    let super::super::session::intent::Record::Owned(recorded) =
        super::revisions::recorded(&directory, &request, &identity)
    else {
        panic!("the run owns its intent");
    };
    assert_eq!(recorded.admission, crate::agent_revision::seam::Admission::Managed);
    let pin = recorded.pin.expect("a managed intent records its pin");
    assert_eq!(
        (pin.revision.as_str(), pin.qualification_ref.as_str()),
        (revision.as_str(), qualification_ref.as_str())
    );
    assert!(directory.join("raw-events.jsonl").exists(), "the worker step ran after the permit");
    assert!(fixture.lifecycle(&definition).sessions.is_empty(), "the pair's session was closed");
    let worker_lifecycle = fixture.lifecycle(&worker_definition);
    assert!(
        worker_lifecycle.sessions.is_empty() && worker_lifecycle.selected.is_none(),
        "the worker is untouched"
    );

    let inventory = fixture.inventory(&request_path, true).expect("inventory");
    let pinned = &inventory["instance"]["pinned"];
    assert_eq!(pinned["value"]["status"], "released", "the pin is read from the pair's lifecycle");
    assert_eq!(pinned["value"]["revision"], revision.as_str());
    assert_eq!(pinned["value"]["pin_identity"], pin.pin_identity.as_str());
    assert_eq!(pinned["value"]["is_blocked"], false);
    assert_eq!(
        inventory["profile"]["definition"]["identity"],
        worker_definition.as_str(),
        "the profile is the worker"
    );
    fixture
        .attempt("recover", &request_path, true)
        .await
        .expect("recovery closes the pair's session again");
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.subject]
async fn a_pinned_session_keys_on_the_pair() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-pinned", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let request = named(worker.clone(), &definition, &revision);
    let (request_path, _) = fixture.write("pair.json", &request);
    let session = super::super::session::revision::identifier(&request);
    fixture.lost_after_pin(&request_path).expect("run pins the pair under the attempt's fence");
    let pinned = fixture.lifecycle(&definition).sessions.get(&session).cloned().expect("pinned in the pair");
    assert_eq!(pinned.revision, revision);
    let worker_definition = crate::agent_revision::bundle::resolve(&worker).expect("resolves").definition.identity;
    assert!(stored(&fixture, &worker_definition).is_none(), "the worker definition has no lifecycle");
    let crate::agent_revision::seam::Effect::Permitted { permit } = effect(&fixture, &request, &session) else {
        panic!("the pinned pair permits its effect");
    };
    assert_eq!(permit.revision, revision);
    assert!(
        matches!(admit(&fixture, &request, &session), crate::agent_revision::seam::Decision::Admitted {
            admitted: lattice_core::agent_revision::ops::sessions::Admitted::AlreadyPinned { .. },
        }),
        "a repeated admission finds the same pin"
    );
    // The worker alone names no lifecycle for this session.
    assert_eq!(admit(&fixture, &worker, &session), crate::agent_revision::seam::Decision::Unmanaged);

    fixture
        .revision(&["revoke", "--definition", &definition, "--session", &session])
        .expect("revoke the session");
    assert_eq!(effect(&fixture, &request, &session), crate::agent_revision::seam::Effect::Blocked {
        refusal: Refusal::SessionRevoked,
    });
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.subject]
async fn revoking_the_pair_refuses_new_attempts_and_the_retained_export() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-revoked", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let first = named(worker, &definition, &revision);
    let (first_path, _) = fixture.write("first.json", &first);
    fixture.attempt("run", &first_path, true).await.expect("the pair admits");
    fixture
        .revision(&["revoke", "--definition", &definition, "--revision", &revision])
        .expect("revoke the pair");

    let second = named(fixture.request("attempt-after-revocation", 8), &definition, &revision);
    let (second_path, _) = fixture.write("second.json", &second);
    let refused = fixture.attempt("run", &second_path, true).await;
    assert_eq!(refused, Err(Reason::RevisionNotAdmitted.message().to_owned()));
    assert_refused_before_any_intent(&fixture, &second, &definition);
    assert_eq!(admit(&fixture, &second, "session-probe"), crate::agent_revision::seam::Decision::Refused {
        refusal: Refusal::RevisionRevoked,
    });

    let output_parent = fixture.root.join("exports");
    std::fs::create_dir(&output_parent).expect("output parent");
    let blocked = super::super::export::run(
        &first_path,
        &fixture.state,
        &output_parent,
        super::super::session::revision::Store(Some(&fixture.store)),
    );
    assert_eq!(blocked, Err(Reason::ExportRevoked.message().to_owned()));
    assert!(std::fs::read_dir(&output_parent).expect("exports").next().is_none(), "nothing is exported");
}

/// A provider-allocation worker request, and a fixture port with the two
/// observations its admission and its pre-spawn revalidation read.
fn provider(fixture: &Fixture) -> (Request, crate::execution_binding::FixtureComputePort) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive/agent-a-1-active.json");
    let observation =
        lattice_core::execution_binding::validation::decode_observation(&std::fs::read(path).expect("fixture"))
            .expect("valid fixture");
    let mut worker = super::request(
        &observation.allocation.attempt_id,
        lattice_core::execution_binding::request::ComputeRequirement::ProviderExecution {
            direct_accelerator: None,
        },
        lattice_core::execution_binding::request::ComputeDeclaration::ProviderAllocation {
            allocation_id: observation.allocation.id.clone(),
            host_id: observation.allocation.host_id.clone(),
            consumer_id: observation.allocation.consumer_id.clone(),
            owner_generation: observation.allocation.owner_generation,
        },
    );
    worker.workspace_root = fixture.root.join("workspace");
    let mut later = observation.clone();
    later.port.observation_sequence = 2;
    later.offer = None;
    let reply = |observed| Ok(crate::execution_binding::AllocationReply::Observed(Box::new(observed)));
    let replies = crate::execution_binding::FixtureComputePort::new(vec![reply(observation), reply(later)]);
    (super::with_capability(worker, super::revisions::capability()), replies)
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.subject]
async fn revoking_the_pairs_session_after_admission_blocks_the_attempt_before_spawn() {
    let fixture = Fixture::new();
    let (worker, replies) = provider(&fixture);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let request = named(worker, &definition, &revision);
    let (request_path, _) = fixture.write("pair.json", &request);
    let mut port = super::revisions::RevokingPort {
        fixture: &fixture,
        definition: definition.clone(),
        session: super::super::session::revision::identifier(&request),
        replies,
        calls: 0,
    };
    let blocked = super::super::execution::run(
        &request_path,
        &fixture.state,
        Some(super::super::Ports {
            compute: &mut port,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
            repair: None,
        }),
        super::super::session::revision::Store(Some(&fixture.store)),
    )
    .await;
    assert_eq!(blocked, Err(Reason::RevisionBlocked.message().to_owned()));
    assert_eq!(port.calls, 2, "admitted, then revalidated right before spawn");
    let directory = fixture.directory(&request);
    let block = super::super::session::revision::retained_block(&directory).expect("readable").expect("recorded");
    assert_eq!(block["refusal"]["code"], "session_revoked");
    for name in ["raw-events.jsonl", "omp-observation.json"] {
        assert!(!directory.join(name).exists(), "{name}: the worker never spawned");
    }
    assert!(fixture.lifecycle(&definition).sessions.is_empty(), "the blocked session is closed");
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.refused]
async fn a_bundle_for_another_worker_profile_is_refused_before_any_intent() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-mismatch", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let mut other = worker.clone();
    other.omp.executable_blake3 = "9".repeat(64);
    other.envelope.worker_profile = other.worker_profile_identity().expect("revision");
    other.admitted_envelope = other.envelope.identity();
    assert_ne!(other.envelope.worker_profile, worker.envelope.worker_profile, "another worker profile revision");
    let request = named(other, &definition, &revision);
    let (request_path, _) = fixture.write("mismatch.json", &request);
    let refused = fixture.attempt("run", &request_path, true).await;
    assert_eq!(refused, Err(Reason::RevisionNotAdmitted.message().to_owned()));
    assert_refused_before_any_intent(&fixture, &request, &definition);
    let session = super::super::session::revision::identifier(&request);
    assert_eq!(admit(&fixture, &request, &session), crate::agent_revision::seam::Decision::Refused {
        refusal: Refusal::RevisionNotSelected,
    });
    assert!(fixture.lifecycle(&definition).sessions.is_empty());

    // The worker's own plain bundle is not a workflow-agent revision.
    let plain = Fixture::new();
    let worker = plain.request("attempt-plain-pair", 7);
    let (worker_path, _) = plain.write("worker.json", &worker);
    let (worker_definition, worker_revision) = plain.prepared(&worker, &worker_path);
    plain.promoted(&worker_definition, &worker_revision);
    let request = named(worker, &worker_definition, &worker_revision);
    let (request_path, _) = plain.write("plain.json", &request);
    let refused = plain.attempt("run", &request_path, true).await;
    assert_eq!(refused, Err(Reason::RevisionNotAdmitted.message().to_owned()));
    assert_refused_before_any_intent(&plain, &request, &worker_definition);
    assert_eq!(admit(&plain, &request, "session-probe"), crate::agent_revision::seam::Decision::Refused {
        refusal: Refusal::Malformed,
    });
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.refused]
async fn an_unknown_definition_or_revision_or_a_missing_store_is_refused_before_any_intent() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-unknown", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let unknown_definition = format!("blake3:{}", blake3::hash(b"unknown definition").to_hex());
    let unknown_revision = "f".repeat(64);
    // Each refusal records its fence first, so every attempt takes a new one.
    let pairs = [
        (8, unknown_definition.as_str(), revision.as_str()),
        (9, definition.as_str(), unknown_revision.as_str()),
    ];
    for (fence, pair_definition, pair_revision) in pairs {
        let attempt = format!("attempt-unknown-{fence}");
        let request = named(fixture.request(&attempt, fence), pair_definition, pair_revision);
        let index = fence;
        let (request_path, _) = fixture.write(&format!("unknown-{index}.json"), &request);
        let refused = fixture.attempt("run", &request_path, true).await;
        assert_eq!(refused, Err(Reason::RevisionNotAdmitted.message().to_owned()), "{index}");
        assert_refused_before_any_intent(&fixture, &request, pair_definition);
        assert_eq!(admit(&fixture, &request, "session-probe"), crate::agent_revision::seam::Decision::Refused {
            refusal: Refusal::UnknownRevision,
        });
        // The refusal reported `dispatched: false`, so even a valid request
        // for this attempt-fence can never create an intent.
        let valid = named(fixture.request(&attempt, fence), &definition, &revision);
        let (valid_path, _) = fixture.write(&format!("valid-{index}.json"), &valid);
        let closed = fixture.attempt("run", &valid_path, true).await;
        assert_eq!(closed, Err(Reason::AttemptClosed.message().to_owned()), "{index}");
        assert_refused_before_any_intent(&fixture, &valid, &definition);
    }
    assert!(stored(&fixture, &unknown_definition).is_none(), "a refused pair creates no lifecycle");
    assert!(fixture.lifecycle(&definition).sessions.is_empty());

    let malformed = named(worker.clone(), "definition", &revision);
    assert_eq!(admit(&fixture, &malformed, "session-probe"), crate::agent_revision::seam::Decision::Refused {
        refusal: Refusal::Malformed,
    });
    let (malformed_path, _) = fixture.write("malformed.json", &malformed);
    let error = fixture.attempt("run", &malformed_path, true).await.expect_err("malformed pair");
    assert!(error.contains("workflow-agent"), "{error}");
    assert_untouched(&fixture, &malformed, &definition);

    let storeless = named(fixture.request("attempt-storeless", 7), &definition, &revision);
    let (storeless_path, _) = fixture.write("storeless.json", &storeless);
    let error = fixture.attempt("run", &storeless_path, false).await.expect_err("a pair needs its store");
    assert!(error.contains("--revision-store"), "{error}");
    assert_untouched(&fixture, &storeless, &definition);
}

/// Answers the provider allocation and, on revalidation (after the first
/// request's intent and pin, before its spawn), resumes the paused second
/// request on another thread and records its outcome and the live pins.
struct ResumingPort<'a> {
    fixture: &'a Fixture,
    paused: Option<super::super::execution::admission::Paused>,
    definitions: [String; 2],
    replies: crate::execution_binding::FixtureComputePort,
    calls: u32,
    resumed: Option<Result<(), String>>,
    pins: [usize; 2],
}

impl crate::execution_binding::ComputeAllocationPort for ResumingPort<'_> {
    fn observe_allocation(
        &mut self,
        query: &crate::execution_binding::AllocationQuery,
    ) -> Result<crate::execution_binding::AllocationReply, crate::execution_binding::ComputePortIssue> {
        self.calls += 1;
        if self.calls == 2
            && let Some(paused) = self.paused.take()
        {
            let store = self.fixture.store.as_path();
            let resumed = std::thread::scope(|scope| {
                scope
                    .spawn(move || {
                        let runtime =
                            tokio::runtime::Builder::new_current_thread().enable_all().build().expect("runtime");
                        let ports = super::super::Ports {
                            compute: &mut crate::execution_binding::UnavailableComputePort,
                            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
                            repair: None,
                        };
                        runtime.block_on(paused.resume(ports, super::super::session::revision::Store(Some(store))))
                    })
                    .join()
                    .expect("the paused request returns")
            });
            self.resumed = Some(resumed);
            self.pins = self.definitions.clone().map(|definition| self.fixture.lifecycle(&definition).sessions.len());
        }
        self.replies.observe_allocation(query)
    }
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.single_flight]
// r[verify lattice.implementation_attempt.consumer.scenario.closure]
async fn a_request_paused_while_another_dispatched_its_attempt_is_substituted_and_never_pins() {
    let fixture = Fixture::new();
    let (worker, replies) = provider(&fixture);
    let (first_definition, first_revision) = desired_as(&fixture, &worker, "verify-change", "guidance");
    fixture.promoted(&first_definition, &first_revision);
    let (second_definition, second_revision) = desired_as(&fixture, &worker, "review-change", "guidance");
    fixture.promoted(&second_definition, &second_revision);
    assert_ne!(first_definition, second_definition, "two selected pairs");
    let first = named(worker.clone(), &first_definition, &first_revision);
    let second = named(worker, &second_definition, &second_revision);
    assert_eq!(first.envelope, second.envelope, "one envelope, one attempt-fence, one session");
    let (first_path, first_identity) = fixture.write("first.json", &first);
    let (second_path, _) = fixture.write("second.json", &second);
    // R2 parsed its request and found no intent, then paused.
    let paused = super::super::execution::admission::Paused::load(&second_path, &fixture.state).expect("R2 loads");
    let untouched = fixture.lifecycle(&second_definition);
    let mut port = ResumingPort {
        fixture: &fixture,
        paused: Some(paused),
        definitions: [first_definition.clone(), second_definition.clone()],
        replies,
        calls: 0,
        resumed: None,
        pins: [0, 0],
    };
    super::super::execution::run(
        &first_path,
        &fixture.state,
        Some(super::super::Ports {
            compute: &mut port,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
            repair: None,
        }),
        super::super::session::revision::Store(Some(&fixture.store)),
    )
    .await
    .expect("R1 reaches a terminal result");
    assert_eq!(port.calls, 2, "admitted, then revalidated right before spawn");
    // R2 resumed after R1's intent: a possibly executed attempt, never admitted.
    assert_eq!(port.resumed, Some(Err(Reason::SubstitutedRequest.message().to_owned())));
    let record = super::super::refusal::record(&second.envelope, Reason::SubstitutedRequest);
    assert_eq!(record["dispatched"], true);
    assert_eq!(port.pins, [1, 0], "exactly R1's pin was live when R2 ended");
    assert_eq!(fixture.lifecycle(&second_definition), untouched, "R2 never pinned or closed anything");
    let directory = fixture.directory(&first);
    let super::super::session::intent::Record::Owned(recorded) =
        super::revisions::recorded(&directory, &first, &first_identity)
    else {
        panic!("R1 owns the attempt's intent");
    };
    assert_eq!(recorded.pin.expect("R1's pin").revision, first_revision);
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.subject]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.single_flight]
async fn a_request_naming_another_revision_never_acts_on_the_pinned_session() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-two-revisions", 7);
    let (definition, first) = desired_as(&fixture, &worker, "verify-change", "guidance");
    fixture.promoted(&definition, &first);
    let (same_definition, second) = desired_as(&fixture, &worker, "verify-change", "guidance-2");
    assert_eq!((same_definition.as_str(), first == second), (definition.as_str(), false), "two revisions");
    let pinned = named(worker.clone(), &definition, &first);
    let other = named(worker, &definition, &second);
    let (pinned_path, _) = fixture.write("pinned.json", &pinned);
    let session = super::super::session::revision::identifier(&pinned);
    assert_eq!(session, super::super::session::revision::identifier(&other), "one envelope");
    fixture.lost_after_pin(&pinned_path).expect("run pins the selected revision");
    let crate::agent_revision::seam::Effect::Permitted { permit } = effect(&fixture, &pinned, &session) else {
        panic!("the pinned revision permits its effect");
    };
    let conflict = Refusal::SessionConflict;
    assert_eq!(effect(&fixture, &other, &session), crate::agent_revision::seam::Effect::Blocked {
        refusal: conflict.clone(),
    });
    let managed = crate::agent_revision::seam::Admission::Managed;
    let store = crate::store::Store::new(&fixture.store).expect("store");
    let retained = crate::agent_revision::seam::retained(&store, &other, &session, managed, permit.clone());
    assert_eq!(retained.expect("seam"), crate::agent_revision::seam::Effect::Blocked {
        refusal: conflict.clone(),
    });
    let closed = crate::agent_revision::seam::close(&store, &other, &session, managed, &[]).expect("seam");
    assert_eq!(closed, crate::agent_revision::seam::Closure::Refused { refusal: conflict });
    drop(store);
    let live = fixture.lifecycle(&definition).sessions.get(&session).cloned().expect("still pinned");
    assert_eq!(live.revision, first);
    assert_eq!(live.status, lattice_core::agent_revision::state::SessionStatus::Live);
    assert!(matches!(effect(&fixture, &pinned, &session), crate::agent_revision::seam::Effect::Permitted { .. }));
    // The pinned request's own session still closes.
    let store = crate::store::Store::new(&fixture.store).expect("store");
    let closed = crate::agent_revision::seam::close(&store, &pinned, &session, managed, &[]).expect("seam");
    assert!(matches!(closed, crate::agent_revision::seam::Closure::Closed { .. }), "{closed:?}");
    drop(store);
    assert!(fixture.lifecycle(&definition).sessions.is_empty());
}

/// `worker` naming `definition` and `revision` for its own attempt id and
/// fence of the same work.
fn attempt(fixture: &Fixture, attempt_id: &str, fence: u64, definition: &str, revision: &str) -> Request {
    named(fixture.request(attempt_id, fence), definition, revision)
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.refused]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.operator]
async fn admit_session_refuses_the_attempt_session_before_any_state_change() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-admit-template", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let request = attempt(&fixture, "attempt-admit", 7, &definition, &revision);
    let (request_path, _) = fixture.write("request.json", &request);
    let session = super::super::session::revision::identifier(&request);
    assert_eq!(session, request.envelope.identity(), "the attempt session is the envelope identity");

    let refused = fixture.admit_session(&request_path, &session);
    let attempt_session = crate::agent_revision::operator::AdmitRefusal::AttemptSession;
    assert_eq!(refused, Err(attempt_session.to_string()));
    assert!(
        attempt_session.to_string().contains("implementation-attempt run"),
        "it names the command that admits it"
    );
    // No state root: no lock file, fence record, attempt directory or marker.
    assert!(!fixture.state.exists(), "nothing was written");
    assert!(fixture.lifecycle(&definition).sessions.is_empty(), "nothing was pinned");

    // Any other session is pinned after run's request validation.
    fixture.admit_session(&request_path, "operator-session").expect("a non-attempt session pins");
    let pinned = fixture.lifecycle(&definition).sessions.get("operator-session").cloned().expect("pinned");
    assert_eq!(pinned.revision, revision);
    let mut expired = request.clone();
    expired.expires_at_ms = 1;
    let (expired_path, _) = fixture.write("expired.json", &expired);
    assert!(fixture.admit_session(&expired_path, "operator-expired").is_err(), "run's validation refuses it");
    assert!(!fixture.lifecycle(&definition).sessions.contains_key("operator-expired"));
    let mut superseded = request.clone();
    superseded.envelope.worker_profile = "e".repeat(64);
    superseded.admitted_envelope = superseded.envelope.identity();
    let (superseded_path, _) = fixture.write("superseded.json", &superseded);
    let refused = fixture.admit_session(&superseded_path, "operator-superseded");
    assert!(refused.is_err(), "a superseded profile revision is refused before any pin");
    assert!(!fixture.lifecycle(&definition).sessions.contains_key("operator-superseded"));
    assert!(!fixture.state.exists());
}

#[tokio::test]
// r[verify lattice.agent_revisions.workflow.attempt.scenario.operator]
async fn the_operator_check_and_admission_read_one_parse() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-admit-parse", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let request = attempt(&fixture, "attempt-admit-parse", 7, &definition, &revision);
    let session = request.envelope.identity();
    let operator = crate::agent_revision::operator::AdmitRefusal::AttemptSession;
    assert_eq!(crate::agent_revision::operator::check(&request, &session), Err(operator));
    assert_eq!(crate::agent_revision::operator::check(&request, "operator-session"), Ok(()));
    // The admission takes the parsed request, never a path: no request file
    // exists, so nothing can be read a second time.
    let refused = crate::agent_revision::operator::admit(&request, &fixture.store, &session);
    assert_eq!(refused, Err(operator.to_string()));
    let admitted = crate::agent_revision::operator::admit(&request, &fixture.store, "operator-session");
    assert!(matches!(admitted, Ok(crate::agent_revision::seam::Decision::Admitted { .. })), "{admitted:?}");
    assert!(fixture.lifecycle(&definition).sessions.contains_key("operator-session"));
    assert!(!fixture.lifecycle(&definition).sessions.contains_key(&session));
}

#[tokio::test]
// r[verify lattice.implementation_attempt.consumer.scenario.reservation]
async fn a_pin_left_without_its_intent_is_closed_before_anything_else() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-orphan", 7);
    let (first_definition, first_revision) = desired_as(&fixture, &worker, "verify-change", "guidance");
    fixture.promoted(&first_definition, &first_revision);
    let (second_definition, second_revision) = desired_as(&fixture, &worker, "review-change", "guidance");
    fixture.promoted(&second_definition, &second_revision);
    let record = super::super::session::revision::reservation::FILE;

    // R1 pins its pair and stops before its intent, as a crash would leave it.
    let first = named(worker.clone(), &first_definition, &first_revision);
    let (first_path, _) = fixture.write("first.json", &first);
    fixture.lost_after_pin(&first_path).expect("R1 pins");
    let session = super::super::session::revision::identifier(&first);
    let directory = fixture.directory(&first);
    assert!(fixture.lifecycle(&first_definition).sessions.contains_key(&session));
    assert!(directory.join(record).is_file(), "the write-ahead record names R1's pin");
    // R2 names another pair under the same envelope: R1's pin is closed
    // before R2 is admitted, and R2 runs.
    let second = named(worker, &second_definition, &second_revision);
    let (second_path, second_identity) = fixture.write("second.json", &second);
    fixture.attempt("run", &second_path, true).await.expect("R2 runs once R1's pin is closed");
    assert!(fixture.lifecycle(&first_definition).sessions.is_empty(), "R1's orphaned pin was closed");
    let super::super::session::intent::Record::Owned(recorded) =
        super::revisions::recorded(&directory, &second, &second_identity)
    else {
        panic!("R2 owns the attempt's intent");
    };
    assert_eq!(recorded.pin.expect("R2's pin").revision, second_revision);
    assert!(!directory.join(record).exists(), "the intent superseded the record");

    // Recovery of an orphaned pin closes it and the attempt.
    let orphan = attempt(&fixture, "attempt-orphan-recovered", 8, &first_definition, &first_revision);
    let (orphan_path, _) = fixture.write("orphan.json", &orphan);
    fixture.lost_after_pin(&orphan_path).expect("pinned, then lost before its intent");
    let error = fixture.attempt("recover", &orphan_path, false).await.expect_err("the record needs its store");
    assert!(error.contains("--revision-store"), "{error}");
    assert!(!fixture.lifecycle(&first_definition).sessions.is_empty(), "nothing was closed without the store");
    fixture
        .attempt("recover", &orphan_path, true)
        .await
        .expect("recovery closes the pin and the attempt");
    assert!(fixture.lifecycle(&first_definition).sessions.is_empty(), "the orphaned pin was closed");
    let orphan_directory = fixture.directory(&orphan);
    super::revisions::assert_not_started(&orphan_directory);
    assert!(!orphan_directory.join(record).exists());

    // A record that cannot be attributed to the live pin leaves it alone.
    let foreign = attempt(&fixture, "attempt-orphan-foreign", 9, &first_definition, &first_revision);
    let (foreign_path, _) = fixture.write("foreign.json", &foreign);
    fixture.lost_after_pin(&foreign_path).expect("pinned");
    let foreign_directory = fixture.directory(&foreign);
    let bytes = std::fs::read(foreign_directory.join(record)).expect("record");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    value["revision"] = serde_json::Value::from("f".repeat(64));
    std::fs::remove_file(foreign_directory.join(record)).expect("replace the record");
    std::fs::write(foreign_directory.join(record), serde_json::to_vec(&value).expect("bytes")).expect("record");
    fixture.attempt("recover", &foreign_path, true).await.expect("recovery closes the attempt");
    let session = super::super::session::revision::identifier(&foreign);
    let live = fixture.lifecycle(&first_definition).sessions.get(&session).cloned().expect("still pinned");
    assert_eq!(live.revision, first_revision, "a pin of another revision is never touched");
}

/// `request` as work `work_id` would present it.
fn of_work(mut request: Request, work_id: &str) -> Request {
    request.envelope.work_id = work_id.into();
    request.admitted_envelope = request.envelope.identity();
    request
}

#[tokio::test]
// r[verify lattice.implementation_attempt.consumer.scenario.reservation]
async fn another_works_pending_admission_is_left_untouched_and_crossed() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-pending-crossed", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let record = super::super::session::revision::reservation::FILE;

    // Work A is pinned and lost before its intent.
    let first = named(worker, &definition, &revision);
    let (first_path, _) = fixture.write("work-a.json", &first);
    fixture.lost_after_pin(&first_path).expect("A pins");
    let directory = fixture.directory(&first);
    let first_session = super::super::session::revision::identifier(&first);
    let recorded = std::fs::read(directory.join(record)).expect("A's write-ahead record");

    // Work B reuses A's attempt id and fence.
    let second = of_work(first.clone(), "work-b");
    assert_eq!(fixture.directory(&second), directory, "one attempt directory");
    let (second_path, _) = fixture.write("work-b.json", &second);
    let crossed = Err(Reason::CrossedAttempt.message().to_owned());
    assert_eq!(fixture.attempt("recover", &second_path, true).await, crossed, "B's recovery");
    assert_eq!(fixture.attempt("run", &second_path, true).await, crossed, "B's run");
    assert_eq!(std::fs::read(directory.join(record)).expect("record"), recorded, "A's record is untouched");
    assert!(!directory.join(super::super::session::intent::FILE).exists(), "no intent or marker");
    let live = fixture.lifecycle(&definition).sessions.get(&first_session).cloned().expect("A's pin");
    assert_eq!(live.status, lattice_core::agent_revision::state::SessionStatus::Live);

    // A's own recovery still closes its pin and its attempt.
    fixture.attempt("recover", &first_path, true).await.expect("A's recovery");
    assert!(fixture.lifecycle(&definition).sessions.is_empty(), "A's pin was closed");
    assert!(!directory.join(record).exists());
    super::revisions::assert_not_started(&directory);
}

#[tokio::test]
// r[verify lattice.implementation_attempt.consumer.scenario.reservation]
async fn work_bs_intent_beside_work_as_pending_admission_leaves_both_untouched() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-pending-interleaved", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let record = super::super::session::revision::reservation::FILE;
    let intent_file = super::super::session::intent::FILE;

    // B passed its pending-record check, A pinned and wrote its record, then
    // B wrote its execution intent (unmanaged here) and may be running.
    let first = named(worker, &definition, &revision);
    let (first_path, _) = fixture.write("work-a.json", &first);
    let second = of_work(first.clone(), "work-b");
    let (second_path, second_identity) = fixture.write("work-b.json", &second);
    fixture.lost_after_pin(&first_path).expect("A pins");
    let directory = fixture.directory(&first);
    std::fs::write(directory.join(intent_file), &second_identity).expect("B's intent");
    let recorded = std::fs::read(directory.join(record)).expect("A's record");
    let session = super::super::session::revision::identifier(&first);

    let crossed = Err(Reason::CrossedAttempt.message().to_owned());
    assert_eq!(fixture.attempt("run", &second_path, true).await, crossed, "a duplicate run of B");
    assert_eq!(fixture.attempt("recover", &second_path, true).await, crossed, "B's recovery");
    assert!(super::super::session::intent::is_execution_retained(&directory).expect("read"), "dispatched");
    assert_eq!(std::fs::read(directory.join(intent_file)).expect("intent"), second_identity.as_bytes(), "no marker");
    assert_eq!(std::fs::read(directory.join(record)).expect("record"), recorded, "A's record is untouched");
    let live = fixture.lifecycle(&definition).sessions.get(&session).cloned().expect("A's pin");
    assert_eq!(live.status, lattice_core::agent_revision::state::SessionStatus::Live);
}

#[tokio::test]
// r[verify lattice.implementation_attempt.consumer.scenario.reservation]
// r[verify lattice.implementation_attempt.consumer.scenario.closure]
async fn a_superseded_closure_closes_a_pin_left_without_its_intent_first() {
    let fixture = Fixture::new();
    let worker = fixture.request("attempt-pending-superseded", 7);
    let (definition, revision) = desired(&fixture, &worker);
    fixture.promoted(&definition, &revision);
    let record = super::super::session::revision::reservation::FILE;
    let first = named(worker, &definition, &revision);
    let (first_path, _) = fixture.write("first.json", &first);
    fixture.lost_after_pin(&first_path).expect("pinned, then lost before its intent");
    let directory = fixture.directory(&first);
    assert!(directory.join(record).is_file() && !fixture.lifecycle(&definition).sessions.is_empty());

    // The same attempt under a superseded profile framing closes the attempt.
    let mut superseded = first.clone();
    superseded.envelope.worker_profile = "e".repeat(64);
    superseded.admitted_envelope = superseded.envelope.identity();
    let (superseded_path, _) = fixture.write("superseded.json", &superseded);
    let refused = fixture.attempt("run", &superseded_path, true).await;
    assert_eq!(refused, Err(Reason::ProfileRevisionSuperseded.message().to_owned()));
    assert!(fixture.lifecycle(&definition).sessions.is_empty(), "the pin was closed before the marker");
    assert!(!directory.join(record).exists(), "the record was removed after its pin");
    let marker = std::fs::read(directory.join(super::super::session::intent::FILE)).expect("closure marker");
    assert!(super::super::recovery::is_closed_marker(&marker));

    // Recovery afterwards finds nothing to reconcile, even without the store.
    let error = fixture.attempt("recover", &first_path, false).await;
    assert_eq!(error, Err(Reason::AttemptClosed.message().to_owned()));
    assert!(!directory.join(record).exists());
    assert!(fixture.lifecycle(&definition).sessions.is_empty());
}
