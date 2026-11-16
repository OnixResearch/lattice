use super::*;

mod endpoint;
mod setup;

#[test]
#[ignore = "requires reviewed pinned OMP, bubblewrap, socat and shell; uses only a local HTTP 400 endpoint"]
fn omp_worker_retains_a_real_http_400_terminal_failure() {
    let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let endpoint = endpoint::Endpoint::start(None);
    let fixture = setup::Fixture::new(&endpoint.url());
    let observed = fixture.run(&cancellation);
    let request = endpoint.finish();
    assert_eq!(request["model"], fixture.profile.model_broker.model);
    assert!(matches!(&observed.outcome, Outcome::Failed { .. }), "{observed:?}");
    let observation = observed.observation.as_ref().expect("real terminal JSONL for HTTP 400");
    assert_eq!(observation.terminal, omp::Terminal::Failed);
    assert!(std::str::from_utf8(observed.raw_events()).expect("JSONL UTF-8").contains(endpoint::ERROR_MARKER));
    assert_observation_identity(&observed);
    assert_nonproduced(&observed, crate::candidate::CandidateDisposition::Failed);
    fixture.assert_workspace_retained();
}

#[test]
#[ignore = "requires reviewed pinned OMP, bubblewrap, socat and shell; cancels only after a local request arrives"]
fn omp_worker_cancels_after_reaching_a_held_local_endpoint() {
    let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let endpoint = endpoint::Endpoint::start(Some(std::sync::Arc::clone(&cancellation)));
    let fixture = setup::Fixture::new(&endpoint.url());
    let observed = fixture.run(&cancellation);
    let request = endpoint.finish();
    assert_eq!(request["model"], fixture.profile.model_broker.model);
    assert!(cancellation.load(std::sync::atomic::Ordering::Acquire));
    assert!(matches!(&observed.outcome, Outcome::Cancelled { .. }), "{observed:?}");
    // Hard process cancellation may truncate JSONL before OMP emits agent_end.
    // Only a normalized real terminal can establish an OMP aborted observation.
    if let Ok(observation) = &observed.observation {
        assert_eq!(observation.terminal, omp::Terminal::Aborted);
        assert_observation_identity(&observed);
    }
    assert_nonproduced(&observed, crate::candidate::CandidateDisposition::Cancelled);
    // Retention is inspected independently; cancellation is not workspace-release
    // or coordinator/acceptance evidence, even if the process has been reaped.
    fixture.assert_workspace_retained();
}

fn assert_observation_identity(observed: &omp::RunResult) {
    let observation = observed.observation.as_ref().expect("normalized real OMP observation");
    let first = std::str::from_utf8(observed.raw_events()).expect("JSONL UTF-8").lines().next().expect("session");
    let session: serde_json::Value = serde_json::from_str(first).expect("real session header");
    assert_eq!(session["id"], observation.session_id);
    assert_eq!(session["cwd"], "/workspace");
    assert_eq!(observation.schema, omp::OMP_RESULT_SCHEMA);
    assert_eq!(observation.raw_events_blake3, crate::pure_inet_profile::blake3_identity(observed.raw_events()));
    assert!(!observation.usage_complete);
}

fn assert_nonproduced(observed: &omp::RunResult, disposition: crate::candidate::CandidateDisposition) {
    let result = AttemptResult::from_outcome(&observed.outcome);
    result.validate().expect("valid consumer outcome");
    assert_eq!(result.disposition, disposition);
    assert_eq!(result.stdout_blake3, crate::pure_inet_profile::blake3_identity(observed.raw_events()));
    assert!(result.receipt.is_none());
    assert!(result.published_artifacts.is_empty());
    assert_eq!(observed.revision, omp::OMP_REVISION);
    assert!(observed.compatibility.is_some());
}
