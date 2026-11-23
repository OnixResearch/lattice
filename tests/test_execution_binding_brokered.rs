//! Brokered model-request evidence without a serving declaration (#31): the
//! attempt broker's recorded route, its unattributed coverage, and the exact
//! request count complete evidence proves, beside served coverage. Every
//! request and response here is a synthetic fixture; no endpoint runs.

// r[verify lattice.serving_binding.brokered_evidence]
// r[verify lattice.serving_binding.brokered_evidence.scenario.unattributed]
// r[verify lattice.serving_binding.brokered_evidence.scenario.charged]
// r[verify lattice.serving_binding.brokered_evidence.scenario.conflict]

use lattice::candidate_worker::broker::ModelRequestGate;
use lattice::candidate_worker::broker::RecordedRoute;
use lattice::candidate_worker::broker::RequestEvidence;
use lattice::execution_binding::core::admission::Decision as Admission;
use lattice::execution_binding::core::admission::ProviderReading;
use lattice::execution_binding::core::admission::admit;
use lattice::execution_binding::core::admission::bound::Admitted;
use lattice::execution_binding::core::projection::accounting::WorkerCoverage;
use lattice::execution_binding::core::projection::accounting::is_valid_provenance;
use lattice::execution_binding::core::projection::accounting::provenance;
use lattice::execution_binding::core::request::*;
use lattice::execution_binding::core::serving::brokered;
use lattice::execution_binding::core::serving::record::Mismatch;
use lattice::execution_binding::core::serving::usage::Conflict;
use lattice::execution_binding::core::serving::usage::EntryClass;
use lattice::execution_binding::core::serving::usage::ModelRequestCoverage;
use lattice::execution_binding::serving::gate::Checks;
use lattice::execution_binding::serving::gate::ResponseFacts;
use lattice::execution_binding::serving::gate::Upstream;

const ROUTE_PRIMARY: &str = "blake3:1111111111111111111111111111111111111111111111111111111111111111";
const OK: Checks = Checks {
    is_exact_model: true,
    is_within_budget: true,
    is_current: true,
};

/// In-memory brokered evidence; `refuse_*` simulates a write that cannot be retained.
#[derive(Default)]
struct Evidence {
    intents: Vec<brokered::Intent>,
    records: Vec<brokered::Record>,
    refuse_intents: bool,
    refuse_records: bool,
}

impl RequestEvidence for Evidence {
    fn write_intent(&mut self, intent: &brokered::Intent) -> Result<(), String> {
        if self.refuse_intents {
            return Err("intent not retained".into());
        }
        self.intents.push(intent.clone());
        Ok(())
    }

    fn write_record(&mut self, record: &brokered::Record) -> Result<(), String> {
        if self.refuse_records {
            return Err("record not retained".into());
        }
        self.records.push(record.clone());
        Ok(())
    }
}

fn unserved(routes: RouteDeclaration) -> Admitted {
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
        serving: None,
    };
    let requirement = WorkerRequirement {
        compute: ComputeRequirement::HostLocal {},
        model,
    };
    let attempt = AttemptContext {
        work_id: "work-31",
        attempt_id: "attempt-b",
        fence: 7,
        envelope_identity: "envelope-31",
        profile_revision: "profile-31",
    };
    match admit(&declaration, &requirement, attempt, ProviderReading::NotRead) {
        Admission::Admitted(admitted) => *admitted,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn primary() -> Admitted {
    unserved(RouteDeclaration::Routes {
        routes: vec![NamedRoute {
            name: "primary".into(),
            route_identity: ROUTE_PRIMARY.into(),
        }],
    })
}

fn responded(tokens: Option<(u64, u64)>) -> Upstream {
    Upstream::Responded(ResponseFacts {
        model: Some("model-a".into()),
        input_tokens: tokens.map(|(input, _)| input),
        output_tokens: tokens.map(|(_, output)| output),
        provider_receipt: Some("chatcmpl-1".into()),
    })
}

/// Relay one request at `request_index` and settle it with `upstream`, or
/// leave its intent without a record when `upstream` is `None`.
fn relay(gate: &mut RecordedRoute<'_>, request_index: u32, upstream: Option<Upstream>) {
    let ticket = gate.admit(OK, request_index).expect("admitted");
    if let Some(upstream) = upstream {
        assert!(gate.settle(ticket, upstream).expect("record retained"), "the response reaches the worker");
    }
}

fn summary(coverage: ModelRequestCoverage) -> brokered::Summary {
    match coverage {
        ModelRequestCoverage::BrokeredUnattributed(summary) => *summary,
        other => panic!("expected brokered coverage, got {other:?}"),
    }
}

#[test]
fn a_complete_brokered_run_counts_every_relayed_request_without_attribution() {
    let admitted = primary();
    let mut evidence = Evidence::default();
    let mut gate = RecordedRoute {
        context: brokered::Context::new(&admitted, "primary"),
        evidence: &mut evidence,
    };
    let ticket = gate.admit(OK, 0).expect("admitted");
    let refused = Checks {
        is_within_budget: false,
        ..OK
    };
    assert!(gate.admit(refused, 1).is_none(), "the broker's own checks still refuse");
    assert!(gate.settle(ticket, responded(Some((120, 40)))).expect("retained"));
    relay(&mut gate, 1, Some(responded(None)));
    relay(&mut gate, 2, Some(responded(Some((5, 1)))));
    drop(gate);
    assert_eq!((evidence.intents.len(), evidence.records.len()), (3, 3), "a refusal writes nothing");
    let indices: Vec<u64> = evidence.intents.iter().map(|intent| intent.request_index).collect();
    assert_eq!(indices, [0, 1, 2]);
    assert_eq!(evidence.intents[0].request_id, brokered::request_id(&admitted.binding_identity, 0));
    let coverage = brokered::summarize(&admitted, &evidence.intents, &evidence.records).expect("summary");
    assert_eq!(coverage.observed_requests(None), Some(3), "complete evidence is an exact count");
    assert_eq!(coverage.observed_requests(Some(&(0..2))), Some(2), "a step counts only its index range");
    assert_eq!(coverage.observed_requests(Some(&(3..6))), Some(0));
    let attempt = provenance(&admitted, "incarnation-b", WorkerCoverage::HostMeasured, coverage.clone());
    assert!(is_valid_provenance(&attempt), "brokered coverage is valid attempt provenance");
    let text = serde_json::to_string(&coverage).expect("coverage json");
    assert!(text.starts_with("{\"brokered_unattributed\":"), "{text}");
    for attribution in [
        "service_id",
        "deployment",
        "grant_ref",
        "replica",
        "resource_consumer_ref",
    ] {
        assert!(!text.contains(attribution), "brokered evidence attributes no service: {attribution}");
    }
    let summary = summary(coverage);
    assert!(summary.is_count_complete);
    assert_eq!((summary.dispatched_requests, summary.unknown_usage_requests), (3, 1));
    assert_eq!((summary.known_input_tokens, summary.known_output_tokens), (125, 41));
    assert!(summary.requests.iter().all(|entry| entry.class == EntryClass::Completed));
    // Identical replays count once.
    let mut replayed = evidence.records.clone();
    replayed.extend(evidence.records.iter().cloned());
    let again = brokered::summarize(&admitted, &evidence.intents, &replayed).expect("replay");
    assert_eq!(again.observed_requests(None), Some(3));
}

#[test]
fn an_unconfirmed_or_uncertain_request_is_never_an_exact_count() {
    let admitted = primary();
    let mut evidence = Evidence::default();
    let mut gate = RecordedRoute {
        context: brokered::Context::new(&admitted, "primary"),
        evidence: &mut evidence,
    };
    relay(&mut gate, 0, Some(responded(Some((1, 1)))));
    // The controller is lost after the intent: no terminal record.
    relay(&mut gate, 1, None);
    drop(gate);
    let coverage = brokered::summarize(&admitted, &evidence.intents, &evidence.records).expect("summary");
    assert_eq!(coverage.observed_requests(None), None, "an intent without a record is not counted exactly");
    assert_eq!(coverage.observed_requests(Some(&(0..1))), Some(1), "another step's range stays exact");
    assert_eq!(coverage.observed_requests(Some(&(1..2))), None);
    let unconfirmed = summary(coverage);
    assert!(!unconfirmed.is_count_complete);
    assert_eq!((unconfirmed.dispatched_requests, unconfirmed.unconfirmed_requests), (2, 1));
    assert_eq!(unconfirmed.requests[1].class, EntryClass::DispatchUnconfirmed);

    for upstream in [Upstream::TimedOut, Upstream::TransportFailed, Upstream::Cancelled] {
        let mut evidence = Evidence::default();
        let mut gate = RecordedRoute {
            context: brokered::Context::new(&admitted, "primary"),
            evidence: &mut evidence,
        };
        relay(&mut gate, 0, Some(upstream));
        drop(gate);
        assert!(evidence.records[0].outcome.label.starts_with("uncertain-"));
        let coverage = brokered::summarize(&admitted, &evidence.intents, &evidence.records).expect("summary");
        assert_eq!(coverage.observed_requests(None), None, "an uncertain request is not counted exactly");
        assert_eq!(summary(coverage).uncertain_requests, 1);
    }
    let unobserved = brokered::summarize(&admitted, &[], &[]).expect("empty evidence");
    assert_eq!(unobserved.observed_requests(None), Some(0), "a retained empty ledger proves no request");
    assert_eq!(ModelRequestCoverage::Unknown.observed_requests(None), None, "no ledger proves nothing");
    assert_eq!(ModelRequestCoverage::NoModel.observed_requests(None), None);
}

#[test]
fn brokered_evidence_conflicts_and_unretained_writes_fail_closed() {
    let admitted = primary();
    let mut evidence = Evidence::default();
    let mut gate = RecordedRoute {
        context: brokered::Context::new(&admitted, "primary"),
        evidence: &mut evidence,
    };
    relay(&mut gate, 0, Some(responded(Some((1, 1)))));
    drop(gate);
    let (intents, records) = (evidence.intents, evidence.records);
    let mut changed = records[0].clone();
    changed.outcome.label = "uncertain-timed-out".into();
    changed.outcome.class = lattice::execution_binding::core::serving::record::OutcomeClass::Uncertain;
    changed.usage = lattice::execution_binding::core::serving::record::Usage::UNKNOWN;
    let conflicting = [records[0].clone(), changed];
    assert_eq!(brokered::summarize(&admitted, &intents, &conflicting), Err(Conflict::ConflictingRecord));
    assert_eq!(brokered::summarize(&admitted, &[], &records), Err(Conflict::DispatchWithoutIntent));
    let mut moved = records[0].clone();
    moved.request_index = 9;
    assert_eq!(
        brokered::summarize(&admitted, &intents, &[moved]),
        Err(Conflict::Mismatch {
            mismatch: Mismatch::Crossed { field: "request_id" }
        })
    );
    let mut crossed = records[0].clone();
    crossed.fence = 8;
    assert_eq!(
        brokered::summarize(&admitted, &intents, &[crossed]),
        Err(Conflict::Mismatch {
            mismatch: Mismatch::Crossed { field: "fence" }
        })
    );
    let no_model = unserved(RouteDeclaration::NoModel {});
    assert_eq!(brokered::summarize(&no_model, &[], &[]), Ok(ModelRequestCoverage::NoModel));
    assert_eq!(
        brokered::summarize(&no_model, &intents, &[]),
        Err(Conflict::Mismatch {
            mismatch: Mismatch::Crossed {
                field: "binding_identity"
            }
        })
    );

    // An intent that cannot be retained refuses the request before upstream;
    // a record that cannot be retained fails the broker.
    let mut refusing = Evidence {
        refuse_intents: true,
        ..Evidence::default()
    };
    let mut gate = RecordedRoute {
        context: brokered::Context::new(&admitted, "primary"),
        evidence: &mut refusing,
    };
    assert!(gate.admit(OK, 0).is_none(), "no durable intent, no upstream contact");
    let mut unrecorded = Evidence {
        refuse_records: true,
        ..Evidence::default()
    };
    let mut gate = RecordedRoute {
        context: brokered::Context::new(&admitted, "primary"),
        evidence: &mut unrecorded,
    };
    let ticket = gate.admit(OK, 0).expect("admitted");
    assert!(gate.settle(ticket, responded(None)).is_err(), "an unretained record fails the broker");
    drop(gate);
    assert_eq!((unrecorded.intents.len(), unrecorded.records.len()), (1, 0));
}
