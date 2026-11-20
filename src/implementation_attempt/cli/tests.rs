// r[verify lattice.execution_binding.profile_facts.scenario.unsupported]
// r[verify lattice.execution_binding.persistence.scenario.changed_binding]
// r[verify lattice.worker_inventory.capability]
// r[verify lattice.worker_inventory.capability.scenario.undeclared]
// r[verify lattice.worker_inventory.capability.scenario.declared]
// r[verify lattice.worker_inventory.capability.scenario.malformed]

/// A valid v3 request with one brokered `primary` route.
pub(super) fn request(
    attempt_id: &str,
    compute: lattice_core::execution_binding::request::ComputeRequirement,
    declared: lattice_core::execution_binding::request::ComputeDeclaration,
) -> super::Request {
    let mut envelope = crate::implementation_attempt::ImplementationAttemptEnvelope {
        schema: crate::implementation_attempt::IMPLEMENTATION_ATTEMPT_ENVELOPE_SCHEMA.into(),
        work_id: "work-24".into(),
        work_version: 1,
        source_kind: crate::implementation_attempt::SourceKind::CairnTask,
        source_id: "task-24".into(),
        repository: "OnixResearch/lattice".into(),
        base: "a".repeat(40),
        readiness_snapshot: "b".repeat(64),
        task_spec_digest: "c".repeat(64),
        verification_profile: "f".repeat(64),
        attempt_id: attempt_id.into(),
        fence: 7,
        workspace_id: "workspace-24".into(),
        worker_profile: String::new(),
        budget: crate::implementation_attempt::AttemptBudget {
            max_turns: 1,
            max_seconds: 60,
            max_cost_micros: 1,
        },
        policy_digest: "d".repeat(64),
    };
    let mut request = super::Request {
        schema: super::REQUEST_SCHEMA.into(),
        envelope: envelope.clone(),
        admitted_envelope: String::new(),
        expires_at_ms: u64::MAX,
        workspace_root: "/workspace".into(),
        read_paths: vec!["src".into()],
        write_paths: vec!["src".into()],
        program: "/usr/bin/worker".into(),
        sandbox_program: "/usr/bin/bwrap".into(),
        omp: crate::candidate_worker::omp::Profile {
            revision: crate::candidate_worker::omp::OMP_REVISION.into(),
            executable_blake3: "e".repeat(64),
            base_identity: "0".repeat(64),
            model: "lattice/model".into(),
            prompt: "make the candidate".into(),
            model_broker: crate::candidate_worker::broker::Profile {
                schema: crate::candidate_worker::broker::SCHEMA.into(),
                endpoint: "https://model.invalid/v1/chat/completions".into(),
                model: "model".into(),
                credential: Some(crate::credentials::SecretRef::new("env", "MODEL_TOKEN", None)),
                relay: crate::candidate_worker::broker::Executable {
                    path: "/nix/store/relay".into(),
                    blake3: "a".repeat(64),
                },
                shell: crate::candidate_worker::broker::Executable {
                    path: "/nix/store/shell".into(),
                    blake3: "b".repeat(64),
                },
                sandbox_blake3: "c".repeat(64),
                expires_at_ms: u64::MAX,
                max_requests: 1,
            },
        },
        output_limit_bytes: 4096,
        allow_unchanged: false,
        worker_compute: compute,
        execution_binding: lattice_core::execution_binding::request::Declaration {
            schema: lattice_core::execution_binding::request::DECLARATION_SCHEMA.into(),
            serving: None,
            compute: declared,
            model_routes: lattice_core::execution_binding::request::RouteDeclaration::NoModel {},
        },
        worker_capability: None,
    };
    request.execution_binding.model_routes = lattice_core::execution_binding::request::RouteDeclaration::Routes {
        routes: vec![super::binding::primary_route(&request).expect("primary route identity")],
    };
    envelope.worker_profile = request.worker_profile_identity().expect("profile identity");
    request.envelope = envelope;
    request.admitted_envelope = request.envelope.identity();
    request
}

pub(super) fn host_local() -> super::Request {
    request(
        "attempt-host-local",
        lattice_core::execution_binding::request::ComputeRequirement::HostLocal {},
        lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {},
    )
}

#[test]
fn legacy_v2_requests_are_refused_and_name_v3_readmission() {
    let root = tempfile::tempdir().expect("scratch");
    let path = root.path().join("request.json");
    let mut legacy = serde_json::to_value(host_local()).expect("request value");
    legacy["schema"] = serde_json::Value::from(super::LEGACY_REQUEST_SCHEMA);
    std::fs::write(&path, serde_json::to_vec(&legacy).expect("legacy bytes")).expect("write legacy");
    let error = super::artifacts::load_request(&path).expect_err("v2 refused before effects");
    assert!(error.contains("readmit"), "{error}");
    assert!(error.contains(super::REQUEST_SCHEMA), "{error}");
    let mut unknown = serde_json::to_value(host_local()).expect("request value");
    unknown["worker_compute"] = serde_json::json!({"kind": "gpu_local"});
    std::fs::write(&path, serde_json::to_vec(&unknown).expect("bytes")).expect("write unknown variant");
    assert!(super::artifacts::load_request(&path).is_err());
    let mut extra = serde_json::to_value(host_local()).expect("request value");
    extra["execution_binding"]["lease"] = serde_json::Value::from("fabricated");
    std::fs::write(&path, serde_json::to_vec(&extra).expect("bytes")).expect("write unknown field");
    assert!(super::artifacts::load_request(&path).is_err());
}

#[test]
fn unsupported_model_and_compute_declarations_fail_validation() {
    let request = host_local();
    request.validate(1).expect("host-local hosted-API request validates");
    let mut no_model = host_local();
    no_model.execution_binding.model_routes = lattice_core::execution_binding::request::RouteDeclaration::NoModel {};
    assert!(no_model.validate(1).expect_err("no-model OMP").contains("no-model"));
    let mut extra = host_local();
    if let lattice_core::execution_binding::request::RouteDeclaration::Routes { routes } =
        &mut extra.execution_binding.model_routes
    {
        let mut second = routes[0].clone();
        second.name = "secondary".into();
        routes.push(second);
    }
    assert!(extra.validate(1).expect_err("extra route").contains("extra routes"));
    let mut crossed = host_local();
    if let lattice_core::execution_binding::request::RouteDeclaration::Routes { routes } =
        &mut crossed.execution_binding.model_routes
    {
        routes[0].route_identity = format!("blake3:{}", "9".repeat(64));
    }
    assert!(crossed.validate(1).expect_err("substituted route").contains("differs"));
    let accelerator =
        request_with_accelerator(lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {});
    assert!(accelerator.validate(1).expect_err("accelerator on host").contains("placement"));
}

fn request_with_accelerator(declared: lattice_core::execution_binding::request::ComputeDeclaration) -> super::Request {
    request(
        "attempt-accelerator",
        lattice_core::execution_binding::request::ComputeRequirement::ProviderExecution {
            direct_accelerator: Some(lattice_core::execution_binding::request::DirectAccelerator {
                min_chips: 1,
                accepted_granularities: vec![lattice_core::execution_binding::model::Granularity::Chip],
                backend: None,
                runtime: None,
                architecture: None,
                minimum_enforcement: lattice_core::execution_binding::model::Enforcement::BackendFenced,
            }),
        },
        declared,
    )
}

#[test]
fn compute_requirement_is_part_of_the_profile_revision() {
    let mut changed = host_local();
    changed.worker_compute = lattice_core::execution_binding::request::ComputeRequirement::ProviderExecution {
        direct_accelerator: None,
    };
    assert_ne!(changed.worker_profile_identity().expect("identity"), changed.envelope.worker_profile);
    assert!(changed.validate(1).expect_err("stale profile revision").contains("stale, crossed"));
}

/// `request` re-admitted with `capability`, as a dispatcher would export it.
pub(super) fn with_capability(
    mut request: super::Request,
    capability: lattice_core::execution_binding::capability::Declaration,
) -> super::Request {
    request.worker_capability = Some(capability);
    request.envelope.worker_profile = request.worker_profile_identity().expect("profile identity");
    request.admitted_envelope = request.envelope.identity();
    request
}

pub(super) fn capability(tools: &[&str]) -> lattice_core::execution_binding::capability::Declaration {
    lattice_core::execution_binding::capability::Declaration {
        schema: lattice_core::execution_binding::capability::CAPABILITY_SCHEMA.into(),
        definition: "rust-implementer".into(),
        task_families: Some(lattice_core::execution_binding::capability::NameSet::Declared {
            names: vec!["rust-implementation".into(), "cairn-task".into()],
        }),
        tools: Some(lattice_core::execution_binding::capability::NameSet::Declared {
            names: tools.iter().map(|tool| (*tool).to_owned()).collect(),
        }),
    }
}

#[test]
fn capability_is_framed_into_the_revision_only_when_declared() {
    let undeclared = host_local();
    let legacy_tuple = serde_json::to_vec(&(
        &undeclared.program,
        &undeclared.sandbox_program,
        &undeclared.omp,
        &undeclared.read_paths,
        &undeclared.write_paths,
        undeclared.output_limit_bytes,
        undeclared.allow_unchanged,
        &undeclared.worker_compute,
    ))
    .expect("legacy tuple");
    assert_eq!(
        undeclared.envelope.worker_profile,
        blake3::hash(&legacy_tuple).to_hex().to_string(),
        "requests without a declaration keep their revision bytes"
    );
    let bytes = serde_json::to_value(&undeclared).expect("request json");
    assert!(bytes.get("worker_capability").is_none(), "an absent declaration is not serialized");
    let declared = with_capability(host_local(), capability(&["read", "edit"]));
    declared.validate(1).expect("declared capability validates");
    assert_ne!(declared.envelope.worker_profile, undeclared.envelope.worker_profile);
    let reordered = with_capability(host_local(), capability(&["edit", "read"]));
    assert_eq!(reordered.envelope.worker_profile, declared.envelope.worker_profile, "order-insensitive");
    let widened = with_capability(host_local(), capability(&["read", "edit", "bash"]));
    assert_ne!(widened.envelope.worker_profile, declared.envelope.worker_profile, "new tools are a new revision");
    let mut swapped = declared.clone();
    swapped.worker_capability = Some(capability(&["read", "bash"]));
    assert!(swapped.validate(1).expect_err("changed tools").contains("stale, crossed"));
}

#[test]
fn malformed_capability_is_refused_before_effects() {
    let mut malformed = host_local();
    malformed.worker_capability = Some(capability(&["/usr/bin/bash"]));
    let error = malformed.validate(1).expect_err("path tool");
    assert!(error.contains("worker capability declaration is unsupported"), "{error}");
    assert!(!error.contains("/usr/bin/bash"), "{error}");
    let mut empty = host_local();
    empty.worker_capability = Some(capability(&[]));
    assert!(empty.validate(1).expect_err("empty set").contains("Empty"));
    let mut value = serde_json::to_value(with_capability(host_local(), capability(&["read"]))).expect("value");
    value["worker_capability"]["prompt"] = serde_json::Value::from("make the candidate");
    assert!(serde_json::from_value::<super::Request>(value).is_err(), "unknown capability fields are refused");
}

#[test]
fn prepare_normalizes_only_a_single_primary_route() {
    let mut request = host_local();
    let expected = super::binding::primary_route(&request).expect("derived route");
    if let lattice_core::execution_binding::request::RouteDeclaration::Routes { routes } =
        &mut request.execution_binding.model_routes
    {
        routes[0].route_identity = format!("blake3:{}", "0".repeat(64));
    }
    super::binding::normalize_primary_route(&mut request).expect("normalized");
    assert_eq!(
        request.execution_binding.model_routes,
        lattice_core::execution_binding::request::RouteDeclaration::Routes { routes: vec![expected] }
    );
    request.execution_binding.model_routes = lattice_core::execution_binding::request::RouteDeclaration::NoModel {};
    super::binding::normalize_primary_route(&mut request).expect("unchanged");
    assert!(request.validate(1).is_err());
}

#[test]
fn intent_is_one_shot_and_crossed_requests_are_refused() {
    let root = tempfile::tempdir().expect("scratch");
    let path = root.path().join("intent");
    assert!(super::artifacts::write_once(&path, b"exact").expect("first"));
    assert!(!super::artifacts::write_once(&path, b"exact").expect("replay"));
    assert!(super::artifacts::write_once(&path, b"other").is_err());
}
#[test]
fn unsafe_state_and_unbounded_requests_are_refused() {
    assert!(super::artifacts::private_directory(std::path::Path::new("relative")).is_err());
    let root = tempfile::tempdir().expect("scratch");
    let path = root.path().join("request");
    std::fs::write(&path, b"not-json").expect("write");
    assert!(super::artifacts::load_request(&path).is_err());
}

#[test]
fn recovery_requires_exact_bounded_raw_and_normalized_event_artifacts() {
    let root = tempfile::tempdir().expect("artifact directory");
    let raw = concat!(
        "{\"type\":\"session\",\"version\":3,\"id\":\"owned-session\",\"cwd\":\"/workspace\"}\n",
        "{\"type\":\"agent_start\"}\n",
        "{\"type\":\"message_end\",\"message\":{\"role\":\"assistant\",\"stopReason\":\"stop\"}}\n",
        "{\"type\":\"agent_end\",\"messages\":[{\"role\":\"assistant\",\"stopReason\":\"stop\"}]}\n",
    )
    .as_bytes();
    let digest = blake3::hash(raw).to_hex().to_string();
    let raw_path = root.path().join("raw-events.jsonl");
    let observation_path = root.path().join("omp-observation.json");
    assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    super::artifacts::write_once(&raw_path, raw).expect("durable raw events");
    assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    let observation = crate::candidate_worker::omp::normalize_jsonl(raw, 4096, "/workspace");
    assert!(observation.is_ok());
    let encoded = serde_json::to_vec(&observation).expect("encode normalized observation");
    super::artifacts::write_once(&observation_path, &encoded).expect("durable normalized observation");
    super::artifacts::verify_events(root.path(), 4096, &digest).expect("exact retained evidence");
    assert!(super::artifacts::verify_events(root.path(), raw.len() - 1, &digest).is_err());
    std::fs::write(&raw_path, b"substituted events\n").expect("substitute raw artifact");
    assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    std::fs::write(&raw_path, raw).expect("restore original raw artifact");
    std::fs::write(&observation_path, b"{\"Err\":\"forged completion\"}").expect("substitute normalized artifact");
    assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    #[cfg(unix)]
    {
        std::fs::remove_file(&raw_path).expect("remove raw artifact");
        std::os::unix::fs::symlink(&observation_path, &raw_path).expect("substitute artifact link");
        assert!(super::artifacts::verify_events(root.path(), 4096, &digest).is_err());
    }
}

#[test]
fn bounded_artifact_reads_accept_exact_bounds_and_reject_excess() {
    let root = tempfile::tempdir().expect("artifact directory");
    let path = root.path().join("bounded");
    std::fs::write(&path, b"exact").expect("write bounded artifact");
    assert_eq!(super::artifacts::read_bounded(&path, 5).expect("exact bound"), b"exact");
    assert!(super::artifacts::read_bounded(&path, 4).is_err());
    std::fs::write(&path, b"").expect("write empty artifact");
    assert_eq!(super::artifacts::read_bounded(&path, 0).expect("empty bound"), b"");
}

#[test]
fn concurrent_intent_writers_have_exactly_one_execution_owner() {
    let root = tempfile::tempdir().expect("state directory");
    let path = root.path().join("intent");
    let barrier = std::sync::Barrier::new(2);
    let outcomes = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            barrier.wait();
            super::artifacts::write_once(&path, b"exact")
        });
        let second = scope.spawn(|| {
            barrier.wait();
            super::artifacts::write_once(&path, b"exact")
        });
        [
            first.join().expect("first writer"),
            second.join().expect("second writer"),
        ]
    });
    assert_eq!(outcomes.iter().filter(|outcome| matches!(outcome, Ok(true))).count(), 1);
    assert_eq!(super::artifacts::read_bounded(&path, 5).expect("exact intent"), b"exact");
}
