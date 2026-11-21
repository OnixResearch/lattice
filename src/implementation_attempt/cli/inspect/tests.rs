//! `inspect` over retained synthetic state; no worker runs and no port is called.

// r[verify lattice.execution_binding.instances.scenario.projection]
// r[verify lattice.execution_binding.instances.scenario.unknown]
// r[verify lattice.execution_binding.profile_facts.scenario.projection]
// r[verify lattice.worker_inventory.schema]
// r[verify lattice.worker_inventory.schema.scenario.cutover]
// r[verify lattice.worker_inventory.capability.scenario.redacted]
// r[verify lattice.worker_inventory.capability.scenario.malformed]
// r[verify lattice.worker_inventory.definition.scenario.stable]
// r[verify lattice.worker_inventory.wait_reasons.scenario.disclosure]
// r[verify lattice.worker_inventory.captures]
// r[verify lattice.worker_inventory.captures.scenario.fixture]

struct Retained {
    _root: tempfile::TempDir,
    request_path: std::path::PathBuf,
    state: std::path::PathBuf,
    directory: std::path::PathBuf,
    identity: String,
    binding_identity: String,
}

fn retained(has_intent: bool) -> Retained {
    retained_from("agent-a-1-active.json", has_intent, std::convert::identity)
}

/// Retained state admitted against one positive provider observation, with
/// the request adjusted as a dispatcher would export it.
fn retained_from(
    fixture_name: &str,
    has_intent: bool,
    adjust: impl FnOnce(super::super::Request) -> super::super::Request,
) -> Retained {
    let root = tempfile::tempdir().expect("scratch");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/execution-binding/observations/positive")
        .join(fixture_name);
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
    let request = adjust(request);
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
    assert_eq!(output["schema"], "lattice.worker-inventory.v2");
    assert_eq!(profile["schema"], "lattice.worker-profile-facts.v2");
    assert_eq!(profile["definition"]["name"], "omp-attempt-worker");
    assert_eq!(profile["definition"]["basis"], "owner_default");
    assert_eq!(profile["task_families"], serde_json::json!({"support": "unknown", "value": null}));
    assert_eq!(profile["tools"], serde_json::json!({"support": "unknown", "value": null}));
    assert_eq!(profile["revisions"]["selected_for_new_sessions"]["owner"], "lattice#25");
    assert_eq!(profile["wait_reasons"][0]["code"], "capability_undeclared");
    assert_eq!(profile["compute"]["support"], "declared");
    assert_eq!(profile["cpu_millis"]["support"], "unknown");
    assert_eq!(profile["model_routes"]["value"]["routes"][0]["name"], "primary");
    assert_eq!(instance["schema"], "lattice.worker-instance-observation.v2");
    assert_eq!(instance["definition_identity"], profile["definition"]["identity"]);
    assert_eq!(instance["liveness"]["state"], "exited");
    assert_eq!(instance["liveness"]["runtime"], "single_shot");
    assert_eq!(instance["wait_reasons"], serde_json::json!([]), "a terminal instance waits for nothing");
    assert_eq!(instance["attempt_id"], "attempt-agent-a-1");
    assert_eq!(instance["work_id"], "work-24");
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
        "\"live\"",
        "worker-inventory.v1",
        "profile-facts.v1",
        "instance-observation.v1",
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
    super::super::cancellation::request(&dispatched.request_path, &dispatched.state).expect("cancel requested");
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

fn declared(tools: &[&str]) -> impl FnOnce(super::super::Request) -> super::super::Request {
    let capability = super::super::tests::capability(tools);
    move |request| super::super::tests::with_capability(request, capability)
}

#[test]
fn declared_capability_reaches_inventory_under_a_definition_stable_across_revisions() {
    let first = retained_from("agent-a-1-active.json", true, declared(&["read", "edit"]));
    let output = project(&first.request_path, &first.state).expect("inventory");
    let profile = &output["profile"];
    assert_eq!(profile["definition"]["name"], "rust-implementer");
    assert_eq!(profile["definition"]["basis"], "declared");
    assert_eq!(
        profile["task_families"],
        serde_json::json!({
            "support": "declared",
            "value": ["cairn-task", "rust-implementation"],
        })
    );
    assert_eq!(profile["tools"], serde_json::json!({"support": "declared", "value": ["edit", "read"]}));
    assert!(!output.to_string().contains("capability_undeclared"));
    let instance = &output["instance"];
    assert_eq!(instance["definition_identity"], profile["definition"]["identity"]);
    assert_eq!(instance["liveness"]["state"], "unknown");
    assert_eq!(instance["liveness"]["unknown_cause"], "unobserved");
    let codes = instance["wait_reasons"].as_array().expect("reasons").iter().map(|reason| reason["code"].clone());
    assert_eq!(codes.collect::<Vec<_>>(), [
        "serving_port_absent",
        "dispatch_outcome_unknown",
        "liveness_unobserved",
        "host_reachability_unknown",
        "allocation_liveness_unknown",
    ]);
    let second = retained_from("agent-a-2-active.json", true, declared(&["read", "edit", "bash"]));
    let other = project(&second.request_path, &second.state).expect("inventory");
    assert_ne!(other["profile"]["revision"], profile["revision"], "new tools are a new revision");
    assert_eq!(other["profile"]["definition"], profile["definition"], "of the same definition");
    let again = project(&first.request_path, &first.state).expect("inventory again");
    assert_eq!(again["profile"], output["profile"], "profile facts are deterministic");
    assert_eq!(again["instance"]["wait_reasons"], instance["wait_reasons"], "reasons are deterministic");
}

#[test]
fn malformed_capability_is_refused_without_naming_it() {
    let retained = retained(true);
    let mut request: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&retained.request_path).expect("request")).expect("request json");
    request["worker_capability"] = serde_json::json!({
        "schema": lattice_core::execution_binding::capability::CAPABILITY_SCHEMA,
        "definition": "rust-implementer",
        "tools": {"kind": "declared", "names": ["/usr/bin/bash"]},
    });
    std::fs::write(&retained.request_path, serde_json::to_vec(&request).expect("bytes")).expect("rewrite");
    let error = project(&retained.request_path, &retained.state).expect_err("malformed declaration");
    assert!(error.contains("worker capability declaration is unsupported"), "{error}");
    assert!(!error.contains("/usr/bin/bash"), "{error}");
}

#[test]
fn a_dispatch_past_its_admitted_window_is_stale_never_live() {
    let expired = retained_from("agent-a-1-active.json", true, |mut request| {
        request.expires_at_ms = 1;
        request
    });
    let output = project(&expired.request_path, &expired.state).expect("inventory");
    let instance = &output["instance"];
    assert_eq!(instance["lifecycle"]["state"], "dispatch_recorded_outcome_unknown");
    assert_eq!(instance["liveness"]["state"], "unknown");
    assert_eq!(instance["liveness"]["unknown_cause"], "stale");
    assert!(instance["wait_reasons"].to_string().contains("liveness_stale"));
    let pending = retained_from("agent-a-1-active.json", false, |mut request| {
        request.expires_at_ms = 1;
        request
    });
    let output = project(&pending.request_path, &pending.state).expect("inventory");
    assert_eq!(output["instance"]["liveness"]["state"], "not_started");
    assert!(output["instance"]["wait_reasons"].to_string().contains("admission_expired"));
}

/// Writes the v2 captures Kiln uses as fixtures, only when
/// `LATTICE_INVENTORY_CAPTURE_DIR` names an existing directory.
#[test]
fn capture_worker_inventory_v2() {
    let Some(dir) = std::env::var_os("LATTICE_INVENTORY_CAPTURE_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let write = |name: &str, value: &serde_json::Value| {
        std::fs::write(dir.join(name), serde_json::to_vec_pretty(value).expect("json")).expect("write capture");
    };
    let terminal = retained(true);
    publish(&terminal, crate::candidate::CandidateDisposition::Failed);
    write(
        "agent-a-1-terminal-failed.json",
        &project(&terminal.request_path, &terminal.state).expect("inventory"),
    );
    let admitted = retained(false);
    write(
        "agent-a-1-admitted-not-dispatched.json",
        &project(&admitted.request_path, &admitted.state).expect("inventory"),
    );
    let dispatched = retained(true);
    write(
        "agent-a-1-dispatched-outcome-unknown.json",
        &project(&dispatched.request_path, &dispatched.state).expect("inventory"),
    );
    let empty = dispatched.state.with_file_name("empty-state");
    write("profile-without-instance.json", &project(&dispatched.request_path, &empty).expect("inventory"));
    let second = retained_from("agent-a-2-active.json", true, std::convert::identity);
    write(
        "agent-a-2-dispatched-outcome-unknown.json",
        &project(&second.request_path, &second.state).expect("inventory"),
    );
    let capability = retained_from("agent-a-1-active.json", true, declared(&["read", "edit"]));
    write(
        "agent-a-1-declared-capability-dispatched.json",
        &project(&capability.request_path, &capability.state).expect("inventory"),
    );
}
