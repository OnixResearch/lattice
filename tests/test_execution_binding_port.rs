//! The injected compute port is observation-only.

// r[verify lattice.execution_binding.port_boundary]
// r[verify lattice.execution_binding.port_boundary.scenario.read_only]
// r[verify lattice.execution_binding.port_boundary.scenario.unavailable]

use lattice::execution_binding::ComputeAllocationPort;

fn query() -> lattice::execution_binding::AllocationQuery {
    lattice::execution_binding::AllocationQuery {
        allocation_id: "agent-a-1".into(),
        host_id: "host-a".into(),
        consumer_id: "agent-1".into(),
        attempt_id: "attempt-agent-a-1".into(),
    }
}

fn observation() -> lattice::execution_binding::core::model::ComputeAllocationObservation {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive/agent-a-1-active.json");
    lattice::execution_binding::core::validation::decode_observation(&std::fs::read(path).expect("fixture"))
        .expect("valid synthetic fixture")
}

#[test]
fn standalone_port_is_always_unavailable_and_blocks() {
    let mut port = lattice::execution_binding::UnavailableComputePort;
    let reply = port.observe_allocation(&query());
    assert_eq!(
        reply,
        Ok(lattice::execution_binding::AllocationReply::Unavailable {
            reason: lattice::execution_binding::PortUnavailableReason::ProviderNotConfigured,
        })
    );
    assert_eq!(
        lattice::execution_binding::reading(&reply),
        lattice::execution_binding::core::admission::ProviderReading::Unavailable
    );
}

#[test]
fn fixture_port_replays_scripted_replies_and_records_every_read() {
    let observed = observation();
    let mut port = lattice::execution_binding::FixtureComputePort::new(vec![
        Ok(lattice::execution_binding::AllocationReply::Observed(Box::new(observed.clone()))),
        Ok(lattice::execution_binding::AllocationReply::NotFound),
        Err(lattice::execution_binding::ComputePortIssue::new(
            lattice::execution_binding::PortIssueKind::Timeout,
            "provider timed out",
        )),
    ]);
    let first = port.observe_allocation(&query());
    assert_eq!(
        lattice::execution_binding::reading(&first),
        lattice::execution_binding::core::admission::ProviderReading::Observed(&observed)
    );
    let second = port.observe_allocation(&query());
    assert_eq!(
        lattice::execution_binding::reading(&second),
        lattice::execution_binding::core::admission::ProviderReading::NotFound
    );
    let third = port.observe_allocation(&query());
    assert_eq!(
        lattice::execution_binding::reading(&third),
        lattice::execution_binding::core::admission::ProviderReading::PortIssue
    );
    let exhausted = port.observe_allocation(&query()).expect_err("no scripted reply");
    assert_eq!(exhausted.kind, lattice::execution_binding::PortIssueKind::MalformedReply);
    assert_eq!(port.calls(), vec![query(); 4].as_slice());
}

#[test]
fn port_issue_text_is_bounded_on_a_character_boundary() {
    let long = "é".repeat(300);
    let issue =
        lattice::execution_binding::ComputePortIssue::new(lattice::execution_binding::PortIssueKind::Transport, &long);
    assert!(issue.detail().len() <= 256);
    assert!(issue.detail().chars().all(|character| character == 'é'));
    assert_eq!(issue.detail().len(), 256);
}

/// A port implements exactly one read method; there is nothing to reserve,
/// activate, release, drain, retire or probe.
struct CountingPort {
    reads: u32,
}

impl lattice::execution_binding::ComputeAllocationPort for CountingPort {
    fn observe_allocation(
        &mut self,
        _query: &lattice::execution_binding::AllocationQuery,
    ) -> Result<lattice::execution_binding::AllocationReply, lattice::execution_binding::ComputePortIssue> {
        self.reads += 1;
        Ok(lattice::execution_binding::AllocationReply::NotFound)
    }
}

#[test]
fn the_port_trait_has_only_a_read_method() {
    let mut port = CountingPort { reads: 0 };
    let port_object: &mut dyn lattice::execution_binding::ComputeAllocationPort = &mut port;
    assert!(port_object.observe_allocation(&query()).is_ok());
    assert_eq!(port.reads, 1);
}
