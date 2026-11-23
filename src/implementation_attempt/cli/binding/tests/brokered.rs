//! A single-shot attempt without a serving declaration: its broker's request
//! files feed the persisted attempt usage and the retained lineage draw.
//! Synthetic requests only; no worker, broker process or endpoint runs.

// r[verify lattice.serving_binding.brokered_evidence]
// r[verify lattice.serving_binding.brokered_evidence.scenario.unattributed]
// r[verify lattice.serving_binding.brokered_evidence.scenario.conflict]
// r[verify lattice.implementation_attempt.observed_charge]
// r[verify lattice.implementation_attempt.observed_charge.scenario.single_shot]
// r[verify lattice.implementation_attempt.observed_charge.scenario.incomplete]
// r[verify lattice.implementation_attempt.observed_charge.scenario.replay]

use crate::candidate_worker::broker::ModelRequestGate as _;
use crate::candidate_worker::broker::RequestEvidence as _;

type Admitted = lattice_core::execution_binding::admission::bound::Admitted;
type Coverage = lattice_core::execution_binding::serving::usage::ModelRequestCoverage;

/// The broker ceiling of these attempts, above what they dispatch.
const CEILING: u32 = 4;
const OK: crate::execution_binding::serving::gate::Checks = crate::execution_binding::serving::gate::Checks {
    is_exact_model: true,
    is_within_budget: true,
    is_current: true,
};
const WINDOW: super::super::usage::WorkerWindow = super::super::usage::WorkerWindow {
    started_epoch_ms: Some(10),
    finished_epoch_ms: Some(52),
};

struct Attempt {
    _root: tempfile::TempDir,
    request: super::super::super::Request,
    admitted: Admitted,
    directory: std::path::PathBuf,
    store: crate::store::Store,
}

/// A host-local attempt with a lineage and no serving declaration.
fn attempt() -> Attempt {
    let root = tempfile::tempdir().expect("scratch");
    let mut request = super::super::super::tests::host_local();
    request.omp.model_broker.max_requests = CEILING;
    request.lineage = Some(crate::implementation_attempt::lineage::tests::binding(4, 4));
    assert!(request.execution_binding.serving.is_none());
    let admitted = super::super::admit(&request, &mut crate::execution_binding::UnavailableComputePort)
        .expect("binding")
        .0;
    let directory = root.path().canonicalize().expect("canonical scratch").join("attempt");
    super::super::super::artifacts::private_directory(&directory).expect("attempt directory");
    let store = crate::store::Store::new(root.path().join("usage.redb")).expect("store");
    Attempt {
        _root: root,
        request,
        admitted,
        directory,
        store,
    }
}

fn responded() -> crate::execution_binding::serving::gate::Upstream {
    crate::execution_binding::serving::gate::Upstream::Responded(
        crate::execution_binding::serving::gate::ResponseFacts {
            model: Some("model-a".into()),
            input_tokens: Some(10),
            output_tokens: Some(2),
            provider_receipt: None,
        },
    )
}

/// Relay one request per entry through the attempt's recorded route; `None`
/// leaves the request's intent without a terminal record.
fn relay(attempt: &Attempt, outcomes: Vec<Option<crate::execution_binding::serving::gate::Upstream>>) {
    let mut files = super::super::requests::RequestFiles::new(&attempt.directory).expect("request files");
    let mut gate = super::super::requests::recorded(&attempt.admitted, &mut files);
    for (index, upstream) in (0_u32..).zip(outcomes) {
        let ticket = gate.admit(OK, index).expect("admitted");
        if let Some(upstream) = upstream {
            gate.settle(ticket, upstream).expect("record retained");
        }
    }
}

fn persist(attempt: &Attempt) -> Result<Coverage, String> {
    super::super::usage::WorkerReceipt::new(attempt.admitted.clone(), "request-identity".into()).persist(
        &attempt.store,
        WINDOW,
        &attempt.directory,
    )
}

fn usage(draw: &serde_json::Value) -> &serde_json::Value {
    &draw["usage"][0]
}

#[test]
fn a_complete_brokered_run_draws_exactly_its_requests_as_observed_and_replays_them() {
    let attempt = attempt();
    relay(&attempt, vec![Some(responded()), Some(responded()), Some(responded())]);
    let coverage = persist(&attempt).expect("persisted before the result");
    let Coverage::BrokeredUnattributed(summary) = &coverage else {
        panic!("brokered coverage, got {coverage:?}");
    };
    assert_eq!((summary.dispatched_requests, summary.is_count_complete), (3, true));
    let frame = super::super::draw::SingleShot::of(&attempt.request).expect("lineage");
    frame.retain(&attempt.directory, &coverage).expect("retained before the result");
    let draw = frame.retained(&attempt.directory).expect("retained draw");
    assert_eq!(usage(&draw)["model_requests"], 3);
    assert_eq!(usage(&draw)["provenance"], "host_measured");
    assert_eq!(usage(&draw)["charged"], serde_json::json!({"model_requests": 3, "tool_invocations": 0}));
    // Replays are idempotent.
    assert_eq!(persist(&attempt).expect("identical replay"), coverage);
    frame.retain(&attempt.directory, &coverage).expect("identical draw replay");
    assert_eq!(frame.retained(&attempt.directory).expect("replayed draw"), draw);
    // Evidence lost after the draw is retained never lowers or changes it,
    // and a different draw cannot replace it.
    let requests = attempt.directory.join("model-requests");
    let record = std::fs::read_dir(&requests)
        .expect("evidence")
        .map(|entry| entry.expect("entry").path())
        .find(|path| path.to_string_lossy().ends_with(".record.json"))
        .expect("a record");
    std::fs::remove_file(record).expect("lose one record");
    let degraded = super::super::requests::coverage(&attempt.admitted, &attempt.directory).expect("coverage");
    assert_eq!(degraded.observed_requests(None), None);
    assert!(frame.retain(&attempt.directory, &degraded).is_err(), "a retained draw is write-once");
    std::fs::remove_dir_all(&requests).expect("lose all evidence");
    assert_eq!(frame.retained(&attempt.directory).expect("replayed draw"), draw, "never reduced");
}

#[test]
fn one_intent_without_a_record_draws_the_reservation_with_unknown_provenance() {
    let attempt = attempt();
    relay(&attempt, vec![Some(responded()), None]);
    let coverage = persist(&attempt).expect("persisted");
    let Coverage::BrokeredUnattributed(summary) = &coverage else {
        panic!("brokered coverage, got {coverage:?}");
    };
    assert_eq!((summary.dispatched_requests, summary.unconfirmed_requests), (2, 1));
    let frame = super::super::draw::SingleShot::of(&attempt.request).expect("lineage");
    frame.retain(&attempt.directory, &coverage).expect("retained");
    let draw = frame.retained(&attempt.directory).expect("retained draw");
    assert_eq!(usage(&draw)["model_requests"], serde_json::Value::Null, "unknown, never zero");
    assert_eq!(usage(&draw)["provenance"], "unknown");
    assert_eq!(usage(&draw)["charged"]["model_requests"], CEILING, "the reservation is charged");
}

/// Evidence lost after a request but before the draw is retained is
/// `unknown` and draws the reservation; only the pre-created empty directory
/// draws an exact zero.
#[test]
fn evidence_lost_before_the_draw_charges_the_reservation_and_an_empty_ledger_charges_zero() {
    let lost = attempt();
    relay(&lost, vec![Some(responded())]);
    std::fs::remove_dir_all(lost.directory.join("model-requests")).expect("lose the evidence");
    let coverage = persist(&lost).expect("persisted");
    assert_eq!(coverage, Coverage::Unknown, "lost evidence is never zero");
    let frame = super::super::draw::SingleShot::of(&lost.request).expect("lineage");
    frame.retain(&lost.directory, &coverage).expect("retained");
    let draw = frame.retained(&lost.directory).expect("retained draw");
    assert_eq!(usage(&draw)["provenance"], "unknown");
    assert_eq!(usage(&draw)["charged"]["model_requests"], CEILING, "the reservation is charged");

    let empty = attempt();
    super::super::requests::RequestFiles::new(&empty.directory).expect("pre-created before the broker");
    let coverage = persist(&empty).expect("persisted");
    assert_eq!(coverage.observed_requests(None), Some(0));
    frame.retain(&empty.directory, &coverage).expect("retained");
    let draw = frame.retained(&empty.directory).expect("retained draw");
    assert_eq!(usage(&draw)["provenance"], "host_measured");
    assert_eq!(usage(&draw)["charged"]["model_requests"], 0, "no request reached upstream");
}

#[test]
fn without_retained_evidence_or_a_retained_draw_the_reservation_is_charged() {
    let attempt = attempt();
    // No broker created the evidence directory: an attempt from before #31.
    let coverage = persist(&attempt).expect("persisted");
    assert_eq!(coverage, Coverage::Unknown, "missing evidence is unobserved, not zero");
    let frame = super::super::draw::SingleShot::of(&attempt.request).expect("lineage");
    relay(&attempt, vec![Some(responded())]);
    // Complete evidence without a retained draw is never derived into a lower draw.
    let draw = frame.retained(&attempt.directory).expect("ceiling");
    assert_eq!(
        (usage(&draw)["provenance"].as_str(), usage(&draw)["charged"]["model_requests"].as_u64()),
        (Some("unknown"), Some(u64::from(CEILING)))
    );
    // A retained draw crossed to another lineage is refused.
    let mut other = attempt.request.clone();
    other.lineage = Some(crate::implementation_attempt::lineage::tests::binding(5, 4));
    let coverage = super::super::requests::coverage(&attempt.admitted, &attempt.directory).expect("coverage");
    frame.retain(&attempt.directory, &coverage).expect("retained");
    let crossed = super::super::draw::SingleShot::of(&other).expect("other lineage");
    assert!(crossed.retained(&attempt.directory).is_err(), "a crossed draw is refused");
}

#[test]
// r[verify lattice.serving_binding.brokered_evidence.scenario.conflict]
fn a_record_renamed_away_from_its_request_id_is_not_evidence() {
    let attempt = attempt();
    relay(&attempt, vec![Some(responded())]);
    let requests = attempt.directory.join("model-requests");
    let record = std::fs::read_dir(&requests)
        .expect("evidence")
        .map(|entry| entry.expect("entry").path())
        .find(|path| path.to_string_lossy().ends_with(".record.json"))
        .expect("a record");
    std::fs::rename(&record, requests.join("stray.record.json")).expect("rename the record");
    let refused = super::super::requests::coverage(&attempt.admitted, &attempt.directory);
    let message = refused.expect_err("a file that does not name its request is not evidence");
    assert!(message.contains("does not name its request"), "{message}");
}

#[test]
// r[verify lattice.implementation_attempt.observed_charge.scenario.replay]
fn a_partial_loss_keeps_the_retained_draw_and_degrades_the_raw_projection() {
    let attempt = attempt();
    relay(&attempt, vec![Some(responded())]);
    let coverage = persist(&attempt).expect("persisted before the result");
    let frame = super::super::draw::SingleShot::of(&attempt.request).expect("lineage");
    frame.retain(&attempt.directory, &coverage).expect("retained before the result");
    let draw = frame.retained(&attempt.directory).expect("retained draw");
    let requests = attempt.directory.join("model-requests");
    let intent = std::fs::read_dir(&requests)
        .expect("evidence")
        .map(|entry| entry.expect("entry").path())
        .find(|path| path.to_string_lossy().ends_with(".intent.json"))
        .expect("an intent");
    std::fs::remove_file(intent).expect("lose one intent");
    let mut fields = serde_json::Map::new();
    fields.insert("execution_binding".to_owned(), serde_json::Value::from("binding-identity"));
    super::super::usage::attach(&mut fields, &attempt.request, &attempt.admitted, &attempt.directory)
        .expect("a partial loss does not discard the draw");
    assert_eq!(fields["budget_lineage"], draw, "the retained draw is replayed unchanged");
    assert_eq!(fields["model_request_evidence"]["coverage"], "unknown", "the raw projection is not a charge");
}

#[test]
fn brokered_request_files_are_write_once_and_conflicts_fail_closed() {
    let attempt = attempt();
    relay(&attempt, vec![Some(responded())]);
    let context = lattice_core::execution_binding::serving::brokered::Context::new(&attempt.admitted, "primary");
    let intent = context.intent(0);
    let record = |label: &str| {
        lattice_core::execution_binding::serving::brokered::Record::settled(
            &intent,
            lattice_core::execution_binding::serving::record::Outcome {
                class: lattice_core::execution_binding::serving::record::OutcomeClass::Uncertain,
                label: label.into(),
                detail: None,
            },
            lattice_core::execution_binding::serving::record::Usage::UNKNOWN,
            None,
        )
    };
    let mut files = super::super::requests::RequestFiles::new(&attempt.directory).expect("request files");
    assert!(files.create_intent(&intent).is_err(), "an identical intent never admits a second contact");
    assert!(files.write_record(&record("uncertain-timed-out")).is_err(), "a different record is refused");
    // A restarted broker over the retained files refuses the used index with
    // nothing written, and still admits a fresh one.
    {
        let mut gate = super::super::requests::recorded(&attempt.admitted, &mut files);
        assert!(gate.admit(OK, 0).is_none(), "a reused index is refused before upstream");
        let ticket = gate.admit(OK, 1).expect("a fresh index");
        gate.settle(ticket, responded()).expect("record retained");
    }
    let replayed = super::super::requests::coverage(&attempt.admitted, &attempt.directory).expect("re-read");
    assert_eq!(replayed.observed_requests(None), Some(2), "each index counts once, distinct indices separately");
    // A record whose intent was never written is refused, and with it the
    // attempt usage: nothing is persisted or drawn from conflicting evidence.
    let orphan = lattice_core::execution_binding::serving::brokered::Record::settled(
        &context.intent(7),
        record("uncertain-cancelled").outcome,
        lattice_core::execution_binding::serving::record::Usage::UNKNOWN,
        None,
    );
    files.write_record(&orphan).expect("a record without its intent");
    let conflict = persist(&attempt).expect_err("conflicting evidence fails closed");
    assert!(conflict.contains("DispatchWithoutIntent"), "{conflict}");
    assert!(
        attempt
            .store
            .load_command_worker_usage(&attempt.admitted.attempt.attempt_id)
            .expect("load")
            .is_none()
    );
    std::fs::write(attempt.directory.join("model-requests/stray.json"), b"{}").expect("stray file");
    assert!(super::super::requests::coverage(&attempt.admitted, &attempt.directory).is_err());
}
