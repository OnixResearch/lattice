//! `inspect` over retained synthetic state; no worker runs and no port is called.

// r[verify lattice.execution_binding.instances.scenario.projection]
// r[verify lattice.execution_binding.instances.scenario.unknown]
// r[verify lattice.execution_binding.profile_facts.scenario.projection]

struct Retained {
    _root: tempfile::TempDir,
    request_path: std::path::PathBuf,
    state: std::path::PathBuf,
    directory: std::path::PathBuf,
    identity: String,
    binding_identity: String,
}

fn retained(has_intent: bool) -> Retained {
    let root = tempfile::tempdir().expect("scratch");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive/agent-a-1-active.json");
    let observation =
        lattice_core::execution_binding::validation::decode_observation(&std::fs::read(fixture).expect("fixture"))
            .expect("valid fixture");
    let request = super::super::tests::request(
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
    let bytes = serde_json::to_vec(&request).expect("request bytes");
    let request_path = root.path().join("request.json");
    std::fs::write(&request_path, &bytes).expect("request");
    let state = root.path().join("state");
    super::super::artifacts::private_directory(&state).expect("state");
    let directory = super::super::artifacts::directory(&state, &request);
    super::super::artifacts::private_directory(&directory).expect("attempt directory");
    let mut port = crate::execution_binding::FixtureComputePort::new(vec![Ok(
        crate::execution_binding::AllocationReply::Observed(Box::new(observation)),
    )]);
    let bound = super::super::binding::admit_and_persist(&request, &directory, &mut port).expect("binding");
    let identity = blake3::hash(&bytes).to_hex().to_string();
    if has_intent {
        super::super::artifacts::write_once(&directory.join("intent.json"), identity.as_bytes()).expect("intent");
    }
    Retained {
        _root: root,
        request_path,
        state,
        directory,
        identity,
        binding_identity: bound.binding_identity,
    }
}

fn publish(retained: &Retained, disposition: crate::candidate::CandidateDisposition) {
    let store = crate::store::Store::new(retained.directory.join("candidate.redb")).expect("store");
    let result = crate::candidate_worker::AttemptResult {
        schema: crate::candidate_worker::CANDIDATE_ATTEMPT_RESULT_SCHEMA.into(),
        disposition,
        receipt: None,
        published_artifacts: Vec::new(),
        stdout_blake3: blake3::hash(b"").to_hex().to_string(),
        stderr_blake3: blake3::hash(b"").to_hex().to_string(),
        blocker: Some("synthetic terminal".into()),
        compatibility: None,
        process_failure: None,
        observed_artifacts: std::collections::BTreeMap::new(),
    };
    store.save_candidate_attempt_result("attempt-agent-a-1", &result).expect("terminal result");
}

/// Inventory as the standalone CLI projects it: no serving-owner port injected.
fn project(request_path: &std::path::Path, state: &std::path::Path) -> Result<serde_json::Value, String> {
    super::project(request_path, state, &mut crate::execution_binding::serving::UnavailableOwner)
}

#[test]
fn inventory_separates_profile_and_terminal_instance_without_material() {
    let retained = retained(true);
    publish(&retained, crate::candidate::CandidateDisposition::Failed);
    let output = project(&retained.request_path, &retained.state).expect("inventory");
    let profile = &output["profile"];
    let instance = &output["instance"];
    assert_eq!(profile["schema"], "lattice.worker-profile-facts.v1");
    assert_eq!(profile["compute"]["support"], "declared");
    assert_eq!(profile["cpu_millis"]["support"], "unknown");
    assert_eq!(profile["model_routes"]["value"]["routes"][0]["name"], "primary");
    assert_eq!(instance["schema"], "lattice.worker-instance-observation.v1");
    assert_eq!(instance["lifecycle"]["state"], "terminal_observed");
    assert_eq!(instance["lifecycle"]["disposition"], "failed");
    assert_eq!(instance["binding_identity"], retained.binding_identity.as_str());
    assert_eq!(instance["incarnation"], retained.identity.as_str());
    assert_eq!(instance["fence"], 7);
    assert_eq!(instance["model_readiness"]["state"], "no_serving_port");
    assert_eq!(profile["serving_readiness"]["state"], "no_serving_port");
    assert_eq!(output["model_requests"], "unknown", "an unserved binding observes no per-request usage");
    let text = output.to_string();
    for forbidden in [
        "/usr/bin",
        "/nix/store",
        "/workspace",
        "model.invalid",
        "make the candidate",
        "MODEL_TOKEN",
        "synthetic-backend-agent-a-1",
        "2222222222222222",
        "\"healthy\"",
        "\"idle\"",
        "\"running\"",
    ] {
        assert!(!text.contains(forbidden), "{forbidden} leaked into inventory");
    }
}

#[test]
fn retained_lifecycle_and_cancellation_stay_conservative() {
    let admitted_only = retained(false);
    let output = project(&admitted_only.request_path, &admitted_only.state).expect("inventory");
    assert_eq!(output["instance"]["lifecycle"]["state"], "admitted_not_dispatched");
    let dispatched = retained(true);
    let output = project(&dispatched.request_path, &dispatched.state).expect("inventory");
    assert_eq!(output["instance"]["lifecycle"]["state"], "dispatch_recorded_outcome_unknown");
    assert_eq!(output["instance"]["cancellation"], "not_requested");
    super::super::cancel(&dispatched.request_path, &dispatched.state).expect("cancel requested");
    let output = project(&dispatched.request_path, &dispatched.state).expect("inventory");
    assert_eq!(output["instance"]["cancellation"], "requested_unobserved");
    publish(&dispatched, crate::candidate::CandidateDisposition::Cancelled);
    let output = project(&dispatched.request_path, &dispatched.state).expect("inventory");
    assert_eq!(output["instance"]["cancellation"], "observed_terminated");
}

#[test]
fn missing_or_unreadable_state_never_becomes_a_guessed_instance() {
    let retained = retained(true);
    let empty_state = retained.state.with_file_name("empty-state");
    let output = project(&retained.request_path, &empty_state).expect("no retained execution");
    assert!(output["instance"].is_null());
    assert!(output["profile"].is_object());
    std::fs::remove_file(retained.directory.join(super::super::binding::ADMITTED_FILE)).expect("remove binding");
    assert!(project(&retained.request_path, &retained.state).is_err(), "intent without binding");
    std::fs::write(retained.directory.join(super::super::binding::ADMITTED_FILE), b"{").expect("corrupt binding");
    assert!(project(&retained.request_path, &retained.state).is_err(), "unreadable binding");
}
