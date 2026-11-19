//! Serving-owner request admission and per-request usage (#24 serving half,
//! #13 usage bindings) over Animus's pure serving gate. Every port, snapshot,
//! grant and response here is a synthetic fixture; no live serving owner,
//! endpoint, provider or device is exercised.

// r[verify lattice.serving_binding.declaration]
// r[verify lattice.serving_binding.declaration.scenario.admitted]
// r[verify lattice.serving_binding.declaration.scenario.unsupported]
// r[verify lattice.serving_binding.port_boundary]
// r[verify lattice.serving_binding.port_boundary.scenario.unavailable]
// r[verify lattice.serving_binding.port_boundary.scenario.unserved]
// r[verify lattice.serving_binding.request_admission]
// r[verify lattice.serving_binding.request_admission.scenario.not_reservation]
// r[verify lattice.serving_binding.request_admission.scenario.refused]
// r[verify lattice.serving_binding.request_observation]
// r[verify lattice.serving_binding.request_observation.scenario.uncertain]
// r[verify lattice.serving_binding.request_observation.scenario.cancellation]
// r[verify lattice.serving_binding.usage]
// r[verify lattice.serving_binding.usage.scenario.dedup]
// r[verify lattice.serving_binding.usage.scenario.partial]
// r[verify lattice.serving_binding.usage.scenario.settled]
// r[verify lattice.serving_binding.independence]
// r[verify lattice.serving_binding.independence.scenario.gpu]
// r[verify lattice.serving_binding.non_claims.scenario.fixture]
// r[verify lattice.serving_binding.non_claims.scenario.provenance]
// r[verify lattice.serving_binding.overrun]
// r[verify lattice.serving_binding.overrun.scenario.stop]
// r[verify lattice.serving_binding.overrun.scenario.inexact]
// r[verify lattice.serving_binding.overrun.scenario.budget_exhausted]

use animus_adapters::serving as producer;
use lattice::execution_binding::core::admission::Decision as Admission;
use lattice::execution_binding::core::admission::ProviderReading;
use lattice::execution_binding::core::admission::Rejection;
use lattice::execution_binding::core::admission::admit;
use lattice::execution_binding::core::admission::bound::Admitted;
use lattice::execution_binding::core::projection::accounting::ProvenanceCompute;
use lattice::execution_binding::core::projection::accounting::WorkerCoverage;
use lattice::execution_binding::core::projection::accounting::provenance;
use lattice::execution_binding::core::request::*;
use lattice::execution_binding::core::serving::record::Counter;
use lattice::execution_binding::core::serving::record::Evidence;
use lattice::execution_binding::core::serving::record::Observation as RequestRecord;
use lattice::execution_binding::core::serving::record::OutcomeClass;
use lattice::execution_binding::core::serving::usage::Conflict;
use lattice::execution_binding::core::serving::usage::EntryClass;
use lattice::execution_binding::core::serving::usage::ModelRequestCoverage;
use lattice::execution_binding::core::serving::usage::Summary;
use lattice::execution_binding::core::serving::usage::summarize;
use lattice::execution_binding::core::serving::{self as declared};
use lattice::execution_binding::serving::Call;
use lattice::execution_binding::serving::Current;
use lattice::execution_binding::serving::CurrentFacts;
use lattice::execution_binding::serving::FixtureOwner;
use lattice::execution_binding::serving::MemorySink;
use lattice::execution_binding::serving::OwnerScript;
use lattice::execution_binding::serving::SnapshotReply;
use lattice::execution_binding::serving::UnavailableOwner;
use lattice::execution_binding::serving::gate::Checks;
use lattice::execution_binding::serving::gate::Decision;
use lattice::execution_binding::serving::gate::ResponseFacts;
use lattice::execution_binding::serving::gate::Session;
use lattice::execution_binding::serving::gate::Upstream;

const CONFIG: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ROUTE_PRIMARY: &str = "blake3:1111111111111111111111111111111111111111111111111111111111111111";
const ROUTE_SECONDARY: &str = "blake3:2222222222222222222222222222222222222222222222222222222222222222";
const MODEL: &str = "model-a";
const SHARED: &str = "svc-shared";
const OTHER: &str = "svc-other";
const HOSTED: &str = "svc-hosted";
const CONSUMER: &str = "serving-b";
const OBSERVED_TICK: u64 = 100;
const NOW_TICK: u64 = 105;
const VALID_UNTIL_TICK: u64 = 160;
const GRANT_UNTIL_TICK: u64 = 130;
const OK: Checks = Checks {
    is_exact_model: true,
    is_within_budget: true,
    is_current: true,
};

const ALL_SUPPORTED: producer::CapabilitySet = producer::CapabilitySet {
    text: animus_adapters::mesh::Capability::Supported,
    tools: animus_adapters::mesh::Capability::Supported,
    streaming: animus_adapters::mesh::Capability::Supported,
    structured_output: animus_adapters::mesh::Capability::Supported,
    developer_role: animus_adapters::mesh::Capability::Supported,
};

const ONE_SLOT: [producer::CapacityDraft; 1] = [producer::CapacityDraft {
    kind: producer::CapacityKind::RequestSlots,
    unit: producer::CapacityUnit::Requests,
    scope: producer::CapacityScope::Deployment,
    origin: producer::CapacityOrigin::ServingOwner,
    value: producer::Counter::Known(1),
    observed_tick: OBSERVED_TICK,
    valid_until_tick: VALID_UNTIL_TICK,
}];

/// Static endpoint and deployment identities of the fixture services.
fn names(service: &str) -> (&'static str, &'static str) {
    match service {
        SHARED => ("endpoint:svc-shared", "deploy-svc-shared"),
        OTHER => ("endpoint:svc-other", "deploy-svc-other"),
        HOSTED => ("endpoint:svc-hosted", "deploy-svc-hosted"),
        other => panic!("unknown fixture service {other}"),
    }
}

fn endpoint(service: &str) -> String {
    names(service).0.to_owned()
}

fn deployment(service: &str) -> String {
    names(service).1.to_owned()
}

fn served(name: &str, service: &str) -> declared::Route {
    declared::Route {
        name: name.into(),
        service_id: service.into(),
        endpoint_identity: endpoint(service),
        model_id: MODEL.into(),
        configuration_ref: CONFIG.into(),
        requirements: declared::Requirements {
            needs_tools: true,
            needs_streaming: false,
            needs_structured_output: false,
            needs_developer_role: false,
            input_tokens: 2_000,
            output_tokens: 500,
        },
    }
}

fn declaration(routes: &[(&str, &str)]) -> declared::Declaration {
    declared::Declaration {
        schema: declared::DECLARATION_SCHEMA.into(),
        profile: declared::Profile {
            profile_id: "capacity-aware-observed-v1".into(),
            identity_rule: declared::IdentityRule::ObservedRequired,
            capacity_rule: declared::CapacityRule::KnownRequired,
            replica_rule: declared::ReplicaRule::SameModelAllowed,
            max_snapshot_age_ticks: 30,
        },
        privacy: declared::Privacy::RemoteAllowed,
        attribution: declared::Attribution::NotApportioned {},
        routes: routes.iter().map(|(name, service)| served(name, service)).collect(),
        stop_on_overrun: false,
    }
}

fn named(names: &[&str]) -> RouteDeclaration {
    RouteDeclaration::Routes {
        routes: names
            .iter()
            .map(|name| NamedRoute {
                name: (*name).into(),
                route_identity: if *name == "primary" {
                    ROUTE_PRIMARY
                } else {
                    ROUTE_SECONDARY
                }
                .into(),
            })
            .collect(),
    }
}

fn decide(attempt_id: &str, routes: RouteDeclaration, serving: Option<declared::Declaration>) -> Admission {
    let model = match &routes {
        RouteDeclaration::NoModel {} => ModelRequirement::NoModel {},
        RouteDeclaration::Routes { routes } => ModelRequirement::Routes {
            names: routes.iter().map(|route| route.name.clone()).collect(),
        },
    };
    let declaration = Declaration {
        schema: DECLARATION_SCHEMA.into(),
        compute: ComputeDeclaration::HostLocal {},
        model_routes: routes,
        serving,
    };
    let requirement = WorkerRequirement {
        compute: ComputeRequirement::HostLocal {},
        model,
    };
    let attempt = AttemptContext {
        work_id: "work-24",
        attempt_id,
        fence: 7,
        envelope_identity: "envelope-24",
        profile_revision: "profile-24",
    };
    admit(&declaration, &requirement, attempt, ProviderReading::NotRead)
}

fn bound(attempt_id: &str, routes: &[(&str, &str)]) -> Admitted {
    let names = routes.iter().map(|(name, _)| *name).collect::<Vec<_>>();
    match decide(attempt_id, named(&names), Some(declaration(routes))) {
        Admission::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn service(service_id: &'static str) -> producer::ServiceDraft<'static> {
    producer::ServiceDraft {
        service_id,
        endpoint_identity: names(service_id).0,
        deployment_id: names(service_id).1,
        deployment_generation: 1,
        locality: if service_id == HOSTED {
            producer::Locality::HostedRemote
        } else {
            producer::Locality::PrivateRemote
        },
        model: Some(producer::ModelDraft {
            model_id: MODEL,
            configuration_ref: CONFIG,
            attestation: producer::Attestation::Observed,
        }),
        readiness: producer::Readiness::Ready,
        capabilities: ALL_SUPPORTED,
        envelope: Some(producer::Envelope::new(8_192, 1_024).expect("envelope")),
        capacity: &ONE_SLOT,
        admission: producer::AdmissionOffer::OwnerAdmission,
        replica: Some("replica-1"),
        resource_consumer_ref: (service_id != HOSTED).then_some(CONSUMER),
        rollout: None,
    }
}

fn snapshot_at(services: &[producer::ServiceDraft<'_>], sequence: u64, now_tick: Option<u64>) -> SnapshotReply {
    let snapshot = producer::AvailabilitySnapshot::new(&producer::SnapshotDraft {
        source_id: "host-observer",
        source_generation: 1,
        sequence,
        observed_tick: OBSERVED_TICK,
        valid_until_tick: VALID_UNTIL_TICK,
        services,
    })
    .expect("fixture snapshot");
    SnapshotReply::Observed {
        snapshot: Box::new(snapshot),
        now_tick,
    }
}

fn snapshot(services: &[producer::ServiceDraft<'_>]) -> SnapshotReply {
    snapshot_at(services, 1, Some(NOW_TICK))
}

fn grant() -> OwnerScript {
    OwnerScript::GrantQueried {
        grant_ref: "grant-1".into(),
        admission_epoch: 1,
        issued_tick: OBSERVED_TICK,
        valid_until_tick: GRANT_UNTIL_TICK,
    }
}

fn facts(service_id: &str) -> CurrentFacts {
    CurrentFacts {
        service_id: service_id.into(),
        endpoint_identity: endpoint(service_id),
        deployment_id: deployment(service_id),
        deployment_generation: 1,
        model_id: MODEL.into(),
        configuration_ref: CONFIG.into(),
        readiness: producer::Readiness::Ready,
        admission_epoch: 1,
        now_tick: Some(NOW_TICK),
        policy_generation: None,
    }
}

fn current(service_id: &str) -> Current {
    Current::Observed(Box::new(facts(service_id)))
}

fn responded(input: u64, output: u64) -> Upstream {
    Upstream::Responded(ResponseFacts {
        model: Some(MODEL.into()),
        input_tokens: Some(input),
        output_tokens: Some(output),
        provider_receipt: Some("chatcmpl-1".into()),
    })
}

/// Admit one request on `route` and settle it with `upstream` when dispatched.
fn request(session: &mut Session<'_>, route: &str, checks: Checks, upstream: Upstream) -> RequestRecord {
    match session.admit(route, checks).expect("gate decision") {
        Decision::Dispatch(ticket) => session.settle(*ticket, upstream).expect("settled"),
        Decision::Refused(record) => *record,
    }
}

fn served_summary(coverage: ModelRequestCoverage) -> Summary {
    match coverage {
        ModelRequestCoverage::Served(summary) => *summary,
        other => panic!("expected served coverage, got {other:?}"),
    }
}

#[test]
fn cpu_worker_calls_a_separately_hosted_service_with_both_bindings_recorded() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut port = FixtureOwner::new(vec![snapshot(&[service(SHARED)])], vec![grant()], vec![current(SHARED)]);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("session");
    let record = request(&mut session, "primary", OK, responded(120, 40));
    drop(session);
    assert_eq!(record.outcome.class, OutcomeClass::Completed);
    assert_eq!(record.evidence, Evidence::Fixture);
    assert_eq!((record.route_identity.as_str(), record.service_id.as_str()), (ROUTE_PRIMARY, SHARED));
    let selected = record.served.as_ref().expect("selected deployment");
    assert_eq!((selected.deployment_id.as_str(), selected.deployment_generation), ("deploy-svc-shared", 1));
    assert_eq!(selected.resource_consumer_ref.as_deref(), Some(CONSUMER));
    assert!(record.receipt_identity.as_deref().is_some_and(|identity| identity.starts_with("blake3:")));
    assert_eq!(sink.intents.len(), 1, "the intent precedes upstream contact");
    let coverage = summarize(&admitted, &sink.intents, &sink.records).expect("summary");
    let attempt = provenance(&admitted, "incarnation-a", WorkerCoverage::HostMeasured, coverage.clone());
    assert!(lattice::execution_binding::core::projection::accounting::is_valid_provenance(&attempt));
    assert_eq!(attempt.compute, ProvenanceCompute::HostLocal {}, "no GPU lease is fabricated");
    let summary = served_summary(coverage);
    assert!(summary.is_complete);
    let usage = &summary.services[0];
    assert_eq!((usage.known_input_tokens, usage.known_output_tokens), (120, 40));
    assert_eq!((usage.dispatched_requests, usage.unknown_usage_requests), (1, 0));
}

#[test]
fn two_attempts_share_one_service_and_a_capacity_snapshot_is_never_a_reservation() {
    let first = bound("attempt-a", &[("primary", SHARED)]);
    let second = bound("attempt-b", &[("primary", SHARED)]);
    let mut first_port = FixtureOwner::new(vec![snapshot(&[service(SHARED)])], vec![grant()], vec![current(SHARED)]);
    let mut second_port = FixtureOwner::new(
        vec![snapshot(&[service(SHARED)])],
        vec![OwnerScript::Reply(producer::OwnerReply::Rejected(
            producer::OwnerRejection::Saturated,
        ))],
        vec![],
    );
    let (mut first_sink, mut second_sink) = (MemorySink::default(), MemorySink::default());
    let mut a = Session::new(&first, "incarnation-a", &mut first_port, &mut first_sink).expect("a");
    let mut b = Session::new(&second, "incarnation-b", &mut second_port, &mut second_sink).expect("b");
    let dispatched = request(&mut a, "primary", OK, responded(10, 5));
    let refused = request(&mut b, "primary", OK, responded(10, 5));
    drop((a, b));
    assert_eq!(dispatched.outcome.class, OutcomeClass::Completed);
    assert_eq!((refused.outcome.class, refused.outcome.label.as_str()), (OutcomeClass::NotDispatched, "saturated"));
    assert!(second_sink.intents.is_empty(), "the second caller never reached upstream");
    assert!(!second_port.calls().iter().any(|call| matches!(call, Call::Recheck { .. })));
    for (admitted, sink) in [(&first, &first_sink), (&second, &second_sink)] {
        let summary = served_summary(summarize(admitted, &sink.intents, &sink.records).expect("summary"));
        let usage = &summary.services[0];
        assert_eq!(usage.attribution, declared::Attribution::NotApportioned {});
        let text = serde_json::to_string(&summary).expect("summary json");
        assert!(!text.contains("claims"), "no shared deployment claim is charged per worker");
    }
    let mut unavailable = FixtureOwner::new(vec![snapshot(&[service(SHARED)])], vec![], vec![]);
    let mut sink = MemorySink::default();
    let third = bound("attempt-c", &[("primary", SHARED)]);
    let mut c = Session::new(&third, "incarnation-c", &mut unavailable, &mut sink).expect("c");
    let blocked = request(&mut c, "primary", OK, responded(1, 1));
    assert_eq!(blocked.outcome.label, "admission-unavailable", "capacity evidence alone never dispatches");
    drop(c);
    assert!(sink.intents.is_empty());
}

#[test]
fn one_worker_calling_two_services_keeps_per_service_provenance() {
    let admitted = bound("attempt-multi", &[("primary", SHARED), ("secondary", OTHER)]);
    let services = [service(SHARED), service(OTHER)];
    let mut port = FixtureOwner::new(
        vec![snapshot(&services), snapshot_at(&services, 2, Some(NOW_TICK))],
        vec![grant(), grant()],
        vec![current(SHARED), current(OTHER)],
    );
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-m", &mut port, &mut sink).expect("session");
    request(&mut session, "primary", OK, responded(100, 10));
    request(&mut session, "secondary", OK, Upstream::TimedOut);
    drop(session);
    let summary = served_summary(summarize(&admitted, &sink.intents, &sink.records).expect("summary"));
    assert!(!summary.is_complete);
    let [primary, secondary] = summary.services.as_slice() else {
        panic!("two services expected");
    };
    assert_eq!((primary.service_id.as_str(), primary.known_input_tokens), (SHARED, 100));
    assert_eq!((secondary.service_id.as_str(), secondary.unknown_usage_requests), (OTHER, 1));
    assert_eq!(
        (primary.route_identity.as_str(), secondary.route_identity.as_str()),
        (ROUTE_PRIMARY, ROUTE_SECONDARY)
    );
    assert_eq!(secondary.known_input_tokens, 0, "unknown usage is counted, never summed as zero tokens");
}

#[test]
fn hosted_api_binds_without_gpu_and_no_model_has_no_inference_usage() {
    let hosted = bound("attempt-hosted", &[("primary", HOSTED)]);
    let mut port = FixtureOwner::new(vec![snapshot(&[service(HOSTED)])], vec![grant()], vec![current(HOSTED)]);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&hosted, "incarnation-h", &mut port, &mut sink).expect("session");
    let record = request(&mut session, "primary", OK, responded(7, 3));
    drop(session);
    assert_eq!(record.served.as_ref().and_then(|served| served.resource_consumer_ref.clone()), None);
    let coverage = summarize(&hosted, &sink.intents, &sink.records).expect("hosted summary");
    let attempt = provenance(&hosted, "incarnation-h", WorkerCoverage::Unknown, coverage);
    assert_eq!(attempt.compute, ProvenanceCompute::HostLocal {});
    let Admission::Admitted(no_model) = decide("attempt-none", RouteDeclaration::NoModel {}, None) else {
        panic!("no-model admits");
    };
    assert_eq!(summarize(&no_model, &[], &[]), Ok(ModelRequestCoverage::NoModel));
    let refused = decide("attempt-none", RouteDeclaration::NoModel {}, Some(declaration(&[("primary", SHARED)])));
    assert_eq!(
        refused,
        Admission::Rejected(Rejection::UnsupportedServing {
            defect: declared::Defect::NoModel
        })
    );
}

type Edit = Box<dyn Fn(&mut declared::Declaration)>;

#[test]
fn unsupported_declarations_are_refused_before_any_binding() {
    let cases: Vec<(&str, Edit, declared::Defect)> = vec![
        ("schema", Box::new(|d| d.schema = "lattice.serving-binding.v0".into()), declared::Defect::Schema),
        ("undeclared", Box::new(|d| d.routes[0].name = "other".into()), declared::Defect::UndeclaredRoute),
        ("repeated", Box::new(|d| d.routes.push(d.routes[0].clone())), declared::Defect::Routes),
        ("token", Box::new(|d| d.routes[0].service_id = "svc/a".into()), declared::Defect::Token {
            field: "service_id",
        }),
        (
            "config",
            Box::new(|d| d.routes[0].configuration_ref = "blake3:AA".into()),
            declared::Defect::Reference {
                field: "configuration_ref",
            },
        ),
        ("output", Box::new(|d| d.routes[0].requirements.output_tokens = 0), declared::Defect::OutputTokens),
        ("age", Box::new(|d| d.profile.max_snapshot_age_ticks = 3_601), declared::Defect::SnapshotAge),
        (
            "policy",
            Box::new(|d| {
                d.attribution = declared::Attribution::DeclaredEstimate {
                    policy_schema: "share-v1".into(),
                    policy_ref: "not-a-ref".into(),
                }
            }),
            declared::Defect::Reference { field: "policy_ref" },
        ),
    ];
    for (name, edit, defect) in cases {
        let mut serving = declaration(&[("primary", SHARED)]);
        edit(&mut serving);
        let decision = decide("attempt-a", named(&["primary"]), Some(serving));
        assert_eq!(decision, Admission::Rejected(Rejection::UnsupportedServing { defect }), "{name}");
    }
}

#[test]
fn serving_declaration_is_framed_only_when_present() {
    let without = match decide("attempt-a", named(&["primary"]), None) {
        Admission::Admitted(admitted) => *admitted,
        other => panic!("{other:?}"),
    };
    let with = bound("attempt-a", &[("primary", SHARED)]);
    assert_ne!(without.binding_identity, with.binding_identity);
    let mut bare = with.clone();
    bare.serving = None;
    assert_eq!(
        lattice::execution_binding::core::admission::bound::identity(&bare).expect("identity"),
        without.binding_identity,
        "bindings without a serving declaration keep their identity"
    );
    let json = serde_json::to_string(&without).expect("json");
    assert!(!json.contains("serving"), "absent declarations serialize as before");
}

#[test]
fn stale_changed_revoked_and_exhausted_facts_never_dispatch() {
    let shared = || vec![service(SHARED)];
    let draining = || {
        let mut draft = service(SHARED);
        draft.readiness = producer::Readiness::Draining;
        vec![draft]
    };
    let with = |edit: fn(&mut CurrentFacts)| {
        let mut facts = facts(SHARED);
        edit(&mut facts);
        Current::Observed(Box::new(facts))
    };
    let other_request = producer::OwnerReply::granted(
        producer::Grant::new(&producer::GrantDraft {
            grant_ref: "grant-x",
            request_id: "request-of-another-attempt",
            service_id: SHARED,
            deployment_id: "deploy-svc-shared",
            deployment_generation: 1,
            model_id: MODEL,
            configuration_ref: CONFIG,
            admission_epoch: 1,
            issued_tick: OBSERVED_TICK,
            valid_until_tick: GRANT_UNTIL_TICK,
            replica: Some("replica-1"),
            policy_generation: None,
        })
        .expect("grant"),
    );
    type Case = (&'static str, SnapshotReply, Option<OwnerScript>, Option<Current>, Checks);
    let cases: Vec<Case> = vec![
        ("expired", snapshot_at(&shared(), 1, Some(VALID_UNTIL_TICK)), None, None, OK),
        ("too-old", snapshot_at(&shared(), 1, Some(OBSERVED_TICK + 31)), None, None, OK),
        ("freshness-unknown", snapshot_at(&shared(), 1, None), None, None, OK),
        ("not-ready:draining", snapshot(&draining()), None, None, OK),
        (
            "model-substituted",
            snapshot(&shared()),
            Some(grant()),
            Some(with(|f| f.model_id = "model-z".into())),
            OK,
        ),
        (
            "configuration-changed",
            snapshot(&shared()),
            Some(grant()),
            Some(with(|f| f.configuration_ref = format!("blake3:{}", "b".repeat(64)))),
            OK,
        ),
        (
            "endpoint-replaced",
            snapshot(&shared()),
            Some(grant()),
            Some(with(|f| f.endpoint_identity = "endpoint:z".into())),
            OK,
        ),
        (
            "generation-replaced",
            snapshot(&shared()),
            Some(grant()),
            Some(with(|f| f.deployment_generation = 2)),
            OK,
        ),
        (
            "not-ready-at-dispatch",
            snapshot(&shared()),
            Some(grant()),
            Some(with(|f| f.readiness = producer::Readiness::Draining)),
            OK,
        ),
        ("grant-revoked", snapshot(&shared()), Some(grant()), Some(with(|f| f.admission_epoch = 2)), OK),
        (
            "grant-expired",
            snapshot(&shared()),
            Some(grant()),
            Some(with(|f| f.now_tick = Some(GRANT_UNTIL_TICK))),
            OK,
        ),
        ("grant-for-other-request", snapshot(&shared()), Some(OwnerScript::Reply(other_request)), None, OK),
        (
            "quota-exhausted",
            snapshot(&shared()),
            Some(OwnerScript::Reply(producer::OwnerReply::Rejected(producer::OwnerRejection::QuotaExhausted))),
            None,
            OK,
        ),
        ("existing-policy-denied", snapshot(&shared()), Some(grant()), Some(current(SHARED)), Checks {
            is_within_budget: false,
            ..OK
        }),
        ("snapshot-unavailable", SnapshotReply::Unavailable, None, None, OK),
    ];
    for (label, reply, owner, recheck, checks) in cases {
        let admitted = bound("attempt-a", &[("primary", SHARED)]);
        let mut port = FixtureOwner::new(vec![reply], owner.into_iter().collect(), recheck.into_iter().collect());
        let mut sink = MemorySink::default();
        let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
        let record = request(&mut session, "primary", checks, responded(1, 1));
        drop(session);
        let observed = record.outcome.detail.as_deref().unwrap_or(record.outcome.label.as_str());
        assert_eq!(observed, label, "{record:?}");
        assert_eq!(record.outcome.class, OutcomeClass::NotDispatched, "{label}");
        assert!(sink.intents.is_empty(), "{label}: no upstream contact");
        assert_eq!(record.usage.input_tokens, Counter::Unknown, "{label}");
    }
}

#[test]
fn out_of_order_snapshot_after_a_newer_accepted_one_is_excluded() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let services = [service(SHARED)];
    let mut port = FixtureOwner::new(
        vec![
            snapshot_at(&services, 5, Some(NOW_TICK)),
            snapshot_at(&services, 4, Some(NOW_TICK)),
        ],
        vec![grant()],
        vec![current(SHARED)],
    );
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    assert_eq!(request(&mut session, "primary", OK, responded(1, 1)).outcome.class, OutcomeClass::Completed);
    let stale = request(&mut session, "primary", OK, responded(1, 1));
    assert_eq!(stale.outcome.detail.as_deref(), Some("out-of-order"));
}

#[test]
fn timeout_and_lost_acknowledgement_never_retry_or_reroute() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let services = [service(SHARED), service(OTHER)];
    let mut port = FixtureOwner::new(
        vec![snapshot(&services), snapshot_at(&services, 2, Some(NOW_TICK))],
        vec![grant(), grant()],
        vec![current(SHARED), current(SHARED)],
    );
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    let timed_out = request(&mut session, "primary", OK, Upstream::TimedOut);
    let lost = request(&mut session, "primary", OK, Upstream::TransportFailed);
    drop(session);
    assert_eq!(timed_out.outcome.label, "uncertain-timed-out");
    assert_eq!(lost.outcome.label, "uncertain-transport-failed");
    assert_ne!(timed_out.request_id, lost.request_id, "the next request has a new identity");
    assert_eq!((timed_out.ordinal, lost.ordinal), (1, 2));
    for record in [&timed_out, &lost] {
        assert_eq!(record.service_id, SHARED, "the declared route is never rerouted");
        assert_eq!(record.usage, lattice::execution_binding::core::serving::record::Usage::UNKNOWN);
    }
    let admits = port.calls().iter().filter(|call| matches!(call, Call::Admit { .. })).count();
    assert_eq!(admits, 2, "each request was admitted exactly once; nothing was retried");
    for call in port.calls() {
        if let Call::Snapshot(query) = call {
            assert_eq!(query.service_id, SHARED);
        }
    }
}

#[test]
fn cancellation_is_recorded_uncertain_and_requested_not_observed() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut port = FixtureOwner::new(vec![snapshot(&[service(SHARED)])], vec![grant()], vec![current(SHARED)]);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    let record = request(&mut session, "primary", OK, Upstream::Cancelled);
    drop(session);
    assert_eq!(
        (record.outcome.class, record.outcome.label.as_str()),
        (OutcomeClass::Uncertain, "uncertain-cancelled")
    );
    let summary = served_summary(summarize(&admitted, &sink.intents, &sink.records).expect("summary"));
    assert!(!summary.is_complete, "a cancelled call may have consumed usage");
    assert_eq!(summary.services[0].unknown_usage_requests, 1);
}

#[test]
fn duplicate_receipts_count_once_and_conflicts_are_refused() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut port = FixtureOwner::new(vec![snapshot(&[service(SHARED)])], vec![grant()], vec![current(SHARED)]);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    request(&mut session, "primary", OK, responded(9, 9));
    drop(session);
    let once = summarize(&admitted, &sink.intents, &sink.records).expect("once");
    let records = [sink.records.clone(), sink.records.clone()].concat();
    let intents = [sink.intents.clone(), sink.intents.clone()].concat();
    assert_eq!(summarize(&admitted, &intents, &records), Ok(once), "identical duplicates count once");
    let mut conflicting = sink.records[0].clone();
    conflicting.usage.input_tokens = Counter::Known(1);
    let conflicted = [sink.records.clone(), vec![conflicting]].concat();
    assert_eq!(summarize(&admitted, &sink.intents, &conflicted), Err(Conflict::ConflictingRecord));
    assert_eq!(summarize(&admitted, &[], &sink.records), Err(Conflict::DispatchWithoutIntent));
    let crossed = bound("attempt-b", &[("primary", SHARED)]);
    assert!(matches!(summarize(&crossed, &sink.intents, &sink.records), Err(Conflict::Mismatch { .. })));
    let mut tampered = sink.records.clone();
    tampered[0].service_id = OTHER.into();
    assert!(matches!(summarize(&admitted, &sink.intents, &tampered), Err(Conflict::Mismatch { .. })));
}

#[test]
fn partial_telemetry_after_interruption_keeps_known_usage_and_unknown_remainder() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let services = [service(SHARED)];
    let mut port = FixtureOwner::new(
        vec![snapshot(&services), snapshot_at(&services, 2, Some(NOW_TICK))],
        vec![grant(), grant()],
        vec![current(SHARED), current(SHARED)],
    );
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    request(&mut session, "primary", OK, responded(30, 12));
    let Decision::Dispatch(_interrupted) = session.admit("primary", OK).expect("second admitted") else {
        panic!("second request dispatches");
    };
    drop(session);
    let summary = served_summary(summarize(&admitted, &sink.intents, &sink.records).expect("summary"));
    let usage = &summary.services[0];
    assert!(!summary.is_complete);
    assert_eq!((usage.dispatched_requests, usage.unknown_usage_requests), (2, 1));
    assert_eq!((usage.known_input_tokens, usage.known_output_tokens), (30, 12));
    assert_eq!(usage.requests[1].class, EntryClass::DispatchUnconfirmed);
    assert_eq!(usage.requests[1].usage.input_tokens, Counter::Unknown);
}

#[test]
fn settled_attempt_usage_cannot_be_reset_by_reconnect_or_recovery() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut port = FixtureOwner::new(vec![snapshot(&[service(SHARED)])], vec![grant()], vec![current(SHARED)]);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    request(&mut session, "primary", OK, responded(11, 4));
    drop(session);
    let usage = |coverage: ModelRequestCoverage| lattice::orchestration::CommandWorkerObservation {
        schema: lattice::orchestration::USAGE_OBSERVATION_SCHEMA.into(),
        scope: lattice::orchestration::Scope::Attempt,
        workflow_revision: admitted.attempt.envelope_identity.clone(),
        instance_id: admitted.attempt.work_id.clone(),
        dispatch_id: "incarnation-a".into(),
        run_id: 0,
        node_id: 0,
        invocation_id: "incarnation-a".into(),
        worker_profile: admitted.profile_revision.clone(),
        attempt_id: admitted.attempt.attempt_id.clone(),
        agent_invocations: None,
        model_requests: None,
        subagent_calls: None,
        tokens: None,
        cost_microunits: None,
        duration_millis: None,
        provenance: lattice::orchestration::UsageProvenance::Unknown,
        complete: false,
        accounting: None,
        execution_binding: Some(provenance(&admitted, "incarnation-a", WorkerCoverage::Unknown, coverage)),
    };
    let settled = usage(summarize(&admitted, &sink.intents, &sink.records).expect("summary"));
    settled.validate().expect("served attempt receipt");
    let root = tempfile::tempdir().expect("store root");
    let store = lattice::store::Store::new(root.path().join("usage.redb")).expect("store");
    store.save_command_worker_usage(&settled).expect("settled once");
    store.save_command_worker_usage(&settled).expect("identical replay is idempotent");
    let reset = usage(summarize(&admitted, &[], &[]).expect("empty summary"));
    assert!(store.save_command_worker_usage(&reset).is_err(), "a reconnect cannot reset settled usage");
    assert_eq!(store.load_command_worker_usage("attempt-a").expect("load"), Some(settled));
}

#[test]
fn unavailable_owner_blocks_served_routes_and_unserved_bindings_stay_unknown() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut port = UnavailableOwner;
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    let record = request(&mut session, "primary", OK, responded(1, 1));
    drop(session);
    assert_eq!(record.outcome.label, "snapshot-unavailable");
    assert!(sink.intents.is_empty());
    let unserved = match decide("attempt-u", named(&["primary"]), None) {
        Admission::Admitted(admitted) => *admitted,
        other => panic!("{other:?}"),
    };
    assert_eq!(summarize(&unserved, &[], &[]), Ok(ModelRequestCoverage::Unknown));
    assert_eq!(summarize(&unserved, &[], &sink.records), Err(Conflict::Unserved));
    let mut port = UnavailableOwner;
    assert!(Session::new(&unserved, "incarnation-u", &mut port, &mut sink).is_err());
    let mut direct = lattice::candidate_worker::broker::DirectRoute;
    assert!(lattice::candidate_worker::broker::ModelRequestGate::admit(&mut direct, OK).is_some());
    let exhausted = Checks {
        is_within_budget: false,
        ..OK
    };
    assert!(lattice::candidate_worker::broker::ModelRequestGate::admit(&mut direct, exhausted).is_none());
}

#[test]
fn rejected_response_model_is_withheld_from_the_worker() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut port = FixtureOwner::new(vec![snapshot(&[service(SHARED)])], vec![grant()], vec![current(SHARED)]);
    let mut sink = MemorySink::default();
    let session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    let mut gate = lattice::candidate_worker::broker::OwnerGate {
        session,
        route_name: "primary".into(),
    };
    let ticket = lattice::candidate_worker::broker::ModelRequestGate::admit(&mut gate, OK).expect("admitted");
    let substituted = Upstream::Responded(ResponseFacts {
        model: Some("model-z".into()),
        input_tokens: Some(5),
        output_tokens: Some(5),
        provider_receipt: None,
    });
    let is_delivered =
        lattice::candidate_worker::broker::ModelRequestGate::settle(&mut gate, ticket, substituted).expect("retained");
    assert!(!is_delivered, "a substituted model response never reaches the worker");
    drop(gate);
    assert_eq!(sink.records[0].outcome.class, OutcomeClass::ResponseRejected);
    assert_eq!(sink.records[0].outcome.label, "response-model-substituted");
}

#[test]
fn gpu_bound_worker_is_never_admitted_from_a_cpu_offer_because_a_service_is_ready() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive/agent-a-1-active.json");
    let cpu_only =
        lattice::execution_binding::core::validation::decode_observation(&std::fs::read(path).expect("fixture"))
            .expect("valid synthetic fixture");
    let declaration = Declaration {
        schema: DECLARATION_SCHEMA.into(),
        compute: ComputeDeclaration::ProviderAllocation {
            allocation_id: cpu_only.allocation.id.clone(),
            host_id: cpu_only.allocation.host_id.clone(),
            consumer_id: cpu_only.allocation.consumer_id.clone(),
            owner_generation: cpu_only.allocation.owner_generation,
        },
        model_routes: named(&["primary"]),
        serving: Some(declaration(&[("primary", SHARED)])),
    };
    let requirement = WorkerRequirement {
        compute: ComputeRequirement::ProviderExecution {
            direct_accelerator: Some(DirectAccelerator {
                min_chips: 1,
                accepted_granularities: vec![lattice::execution_binding::core::model::Granularity::Alias],
                backend: None,
                runtime: None,
                architecture: None,
                minimum_enforcement: lattice::execution_binding::core::model::Enforcement::LogicalFenceOnly,
            }),
        },
        model: ModelRequirement::Routes {
            names: vec!["primary".into()],
        },
    };
    let attempt = AttemptContext {
        work_id: "work-24",
        attempt_id: &cpu_only.allocation.attempt_id,
        fence: 7,
        envelope_identity: "envelope-24",
        profile_revision: "profile-24",
    };
    let decision = admit(&declaration, &requirement, attempt, ProviderReading::Observed(&cpu_only));
    assert_eq!(decision, Admission::Rejected(Rejection::InsufficientDevice));
}

/// An embedder-supplied port: the default provenance, backed by fixture replies.
struct Embedded(FixtureOwner);

impl lattice::execution_binding::serving::OwnerPort for Embedded {
    fn observe_snapshot(&mut self, query: &lattice::execution_binding::serving::SnapshotQuery) -> SnapshotReply {
        self.0.observe_snapshot(query)
    }

    fn admit_request(&mut self, query: &producer::AdmissionQuery) -> producer::OwnerReply {
        self.0.admit_request(query)
    }

    fn recheck(&mut self, query: &producer::AdmissionQuery) -> Current {
        self.0.recheck(query)
    }
}

#[test]
fn live_evidence_requires_an_embedder_port_and_an_admitted_upstream_call() {
    use lattice::execution_binding::serving::OwnerPort as _;
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut unavailable = UnavailableOwner;
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut unavailable, &mut sink).expect("s");
    let blocked = request(&mut session, "primary", OK, responded(1, 1));
    drop(session);
    assert_eq!(blocked.evidence, Evidence::NoUpstreamContact, "a blocked request never claims live evidence");
    let services = [service(SHARED)];
    let mut embedded = Embedded(FixtureOwner::new(
        vec![snapshot(&services), snapshot_at(&services, 2, Some(NOW_TICK))],
        vec![
            grant(),
            OwnerScript::Reply(producer::OwnerReply::Rejected(producer::OwnerRejection::Saturated)),
        ],
        vec![current(SHARED)],
    ));
    assert_eq!(embedded.provenance(), lattice::execution_binding::serving::PortProvenance::Embedder);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut embedded, &mut sink).expect("s");
    let live = request(&mut session, "primary", OK, responded(2, 2));
    let refused = request(&mut session, "primary", OK, responded(2, 2));
    drop(session);
    assert_eq!(live.evidence, Evidence::LiveEndpoint);
    assert_eq!(refused.evidence, Evidence::NoUpstreamContact);
    let mut fixture = FixtureOwner::new(vec![snapshot(&services)], vec![grant()], vec![current(SHARED)]);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut fixture, &mut sink).expect("s");
    assert_eq!(request(&mut session, "primary", OK, responded(1, 1)).evidence, Evidence::Fixture);
}

/// A binding whose declaration sets `stop_on_overrun`.
fn stopping(attempt_id: &str) -> Admitted {
    let mut serving = declaration(&[("primary", SHARED)]);
    serving.stop_on_overrun = true;
    match decide(attempt_id, named(&["primary"]), Some(serving)) {
        Admission::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

/// A port that admits `count` requests on `svc-shared` with in-order snapshots.
fn admitting(count: u64) -> FixtureOwner {
    let services = [service(SHARED)];
    FixtureOwner::new(
        (1..=count).map(|sequence| snapshot_at(&services, sequence, Some(NOW_TICK))).collect(),
        (0..count).map(|_| grant()).collect(),
        (0..count).map(|_| current(SHARED)).collect(),
    )
}

#[test]
fn stop_on_overrun_is_framed_into_the_identity_only_when_set() {
    let unset = bound("attempt-a", &[("primary", SHARED)]);
    let set = stopping("attempt-a");
    assert_ne!(unset.binding_identity, set.binding_identity, "the policy is part of the admitted binding");
    let canonical = declared::canonical(unset.serving.as_ref().expect("declaration")).expect("canonical");
    assert!(!canonical.contains("stop_on_overrun"), "an unset policy keeps the declaration's bytes");
    let decoded: declared::Declaration = serde_json::from_str(&canonical).expect("decodes without the field");
    assert!(!decoded.stop_on_overrun);
    let framed = declared::canonical(set.serving.as_ref().expect("declaration")).expect("canonical");
    assert!(framed.contains("\"stop_on_overrun\":true"));
}

#[test]
fn observed_overrun_stays_visible_beside_the_reservation() {
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut port = admitting(2);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    let over = request(&mut session, "primary", OK, responded(2_400, 300));
    assert!(session.overrun().is_overrun(), "2 700 known tokens exceed the 2 500-token envelope");
    let next = request(&mut session, "primary", OK, responded(10, 5));
    drop(session);
    assert_eq!(over.outcome.class, OutcomeClass::Completed);
    assert_eq!(
        next.outcome.class,
        OutcomeClass::Completed,
        "without the policy an overrun is reported, not enforced"
    );
    let overrun = served_summary(summarize(&admitted, &sink.intents, &sink.records).expect("summary")).overrun;
    assert_eq!((overrun.reserved_tokens, overrun.known_tokens), (5_000, 2_715), "reserved is never usage");
    assert_eq!((overrun.overrun_requests, overrun.overrun_tokens), (1, 200), "an under-used request never offsets");
    assert!(!overrun.is_exact_ceiling, "an overrun reservation is not a ceiling");
    assert_eq!((overrun.stop_on_overrun, overrun.refused_requests), (false, 0));
}

#[test]
fn stop_on_overrun_refuses_the_next_request_with_zero_gate_or_port_calls() {
    let admitted = stopping("attempt-a");
    let mut port = admitting(2);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    assert_eq!(request(&mut session, "primary", OK, responded(3_000, 0)).outcome.class, OutcomeClass::Completed);
    let stopped = request(&mut session, "primary", OK, responded(1, 1));
    let again = request(&mut session, "primary", OK, responded(1, 1));
    drop(session);
    assert_eq!(
        (stopped.outcome.class, stopped.outcome.label.as_str()),
        (OutcomeClass::NotDispatched, "overrun-stopped")
    );
    assert_eq!(again, stopped, "a stopped attempt reuses its one refusal record");
    assert_eq!(port.calls().len(), 3, "only the first request reached snapshot, admission and recheck");
    assert_eq!((sink.intents.len(), sink.records.len()), (1, 2));
    let overrun = served_summary(summarize(&admitted, &sink.intents, &sink.records).expect("summary")).overrun;
    assert!(overrun.stop_on_overrun);
    assert_eq!((overrun.overrun_requests, overrun.overrun_tokens, overrun.refused_requests), (1, 500, 1));
    let within = stopping("attempt-b");
    let mut port = admitting(2);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&within, "incarnation-b", &mut port, &mut sink).expect("s");
    request(&mut session, "primary", OK, responded(2_000, 500));
    let next = request(&mut session, "primary", OK, responded(1, 1));
    drop(session);
    assert_eq!(next.outcome.class, OutcomeClass::Completed, "usage within the envelope never stops");
    let overrun = served_summary(summarize(&within, &sink.intents, &sink.records).expect("summary")).overrun;
    assert!(overrun.is_exact_ceiling, "every request reported usage within its envelope");
}

#[test]
fn unknown_usage_is_never_headroom_and_never_an_exact_ceiling() {
    let admitted = stopping("attempt-a");
    let mut port = admitting(2);
    let mut sink = MemorySink::default();
    let mut session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    request(&mut session, "primary", OK, Upstream::TimedOut);
    assert!(!session.overrun().must_stop(), "unknown usage is not a fabricated overrun");
    request(&mut session, "primary", OK, responded(2_600, 0));
    let stopped = request(&mut session, "primary", OK, responded(1, 1));
    drop(session);
    assert_eq!(stopped.outcome.label, "overrun-stopped", "the timed-out request's envelope is not headroom");
    let summary = served_summary(summarize(&admitted, &sink.intents, &sink.records).expect("summary"));
    let overrun = summary.overrun;
    assert_eq!((overrun.reserved_tokens, overrun.known_tokens), (5_000, 2_600));
    assert_eq!((overrun.unknown_usage_requests, overrun.overrun_tokens), (1, 100));
    assert!(!overrun.is_exact_ceiling && !summary.is_complete);
    let mut port = admitting(1);
    let mut sink = MemorySink::default();
    let unknown_only = bound("attempt-b", &[("primary", SHARED)]);
    let mut session = Session::new(&unknown_only, "incarnation-b", &mut port, &mut sink).expect("s");
    request(&mut session, "primary", OK, Upstream::Cancelled);
    drop(session);
    let overrun = served_summary(summarize(&unknown_only, &sink.intents, &sink.records).expect("summary")).overrun;
    assert_eq!((overrun.overrun_requests, overrun.unknown_usage_requests), (0, 1));
    assert!(!overrun.is_exact_ceiling, "an unknown remainder is never advertised as an exact ceiling");
}

#[test]
fn budget_exhausted_refusal_is_recorded_once_and_replays_idempotently() {
    use lattice::candidate_worker::broker::ModelRequestGate as _;
    use lattice::execution_binding::serving::Sink as _;
    let admitted = bound("attempt-a", &[("primary", SHARED)]);
    let mut port = FixtureOwner::default();
    let mut sink = MemorySink::default();
    let session = Session::new(&admitted, "incarnation-a", &mut port, &mut sink).expect("s");
    let mut gate = lattice::candidate_worker::broker::OwnerGate {
        session,
        route_name: "primary".into(),
    };
    gate.exhausted().expect("recorded");
    gate.exhausted().expect("refused again without a second record");
    drop(gate);
    assert!(port.calls().is_empty(), "the broker's own budget refuses before any serving port call");
    assert!(sink.intents.is_empty());
    let [record] = sink.records.as_slice() else {
        panic!("exactly one record, got {:?}", sink.records);
    };
    let record = record.clone();
    assert_eq!(
        (record.outcome.class, record.outcome.label.as_str()),
        (OutcomeClass::NotDispatched, "budget-exhausted")
    );
    sink.write_record(&record).expect("identical replay is idempotent");
    assert_eq!(sink.records.len(), 1);
    let once = summarize(&admitted, &[], &sink.records).expect("summary");
    let doubled = [sink.records.clone(), sink.records.clone()].concat();
    assert_eq!(summarize(&admitted, &[], &doubled), Ok(once.clone()), "a replayed refusal counts once");
    let summary = served_summary(once);
    assert_eq!((summary.services[0].not_dispatched_requests, summary.overrun.refused_requests), (1, 1));
    assert_eq!(summary.services[0].requests[0].class, EntryClass::NotDispatched);
}
