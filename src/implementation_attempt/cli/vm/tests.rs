//! Q0: VM-bound request validation and inspect facts.

// r[verify lattice.vm_profile.request]

type Request = super::super::Request;

fn fixture<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/vm-profile").join(relative);
    serde_json::from_slice(&std::fs::read(path).expect(relative)).expect(relative)
}

fn host_local() -> Request {
    super::super::tests::host_local()
}

/// A host-local request re-admitted as a guest attempt of the published profile.
fn bound() -> Request {
    let mut request = host_local();
    request.network = Some(false);
    request.omp.model_broker.credential = None;
    rebound(request)
}

/// Re-admit `request` as a dispatcher would, then bind it to the profile.
fn rebound(mut request: Request) -> Request {
    request.execution_binding.model_routes = lattice_core::execution_binding::request::RouteDeclaration::Routes {
        routes: vec![super::super::binding::primary_route(&request).expect("primary route")],
    };
    request.envelope.worker_profile = request.worker_profile_identity().expect("profile revision");
    request.admitted_envelope = request.envelope.identity();
    let profile: crate::vm_profile::ExecutionProfile = fixture("execution-profile/admitted.json");
    let mut binding: crate::vm_profile::AttemptBinding = fixture("attempt-binding/admitted.json");
    binding.envelope = request.admitted_envelope.clone();
    binding.attempt_id = request.envelope.attempt_id.clone();
    binding.fence = request.envelope.fence;
    request.omp.model_broker.forwarded = Some(crate::candidate_worker::broker::Forwarded {
        schema: crate::candidate_worker::broker::FORWARDED_SCHEMA.into(),
        vsock_port: profile.gateway.model_vsock_port,
        binding: crate::vm_profile::binding_identity(&binding).expect("binding identity"),
    });
    request.vm_binding = Some(binding);
    request.vm_profile = Some(profile);
    request
}

// r[verify lattice.vm_profile.request.scenario.absent]
#[test]
fn an_absent_binding_keeps_request_bytes_and_a_bound_one_keeps_the_profile_revision() {
    let local = host_local();
    let bytes = serde_json::to_vec(&local).expect("request bytes");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("request JSON");
    for key in ["vm_binding", "vm_profile"] {
        assert!(value.get(key).is_none(), "{key} is absent from serialization");
    }
    assert!(value["omp"]["model_broker"].get("forwarded").is_none());
    let decoded: Request = serde_json::from_slice(&bytes).expect("decodes");
    assert_eq!(serde_json::to_vec(&decoded).expect("bytes"), bytes, "byte-identical round trip");
    local.validate(1).expect("local request");
    let request = bound();
    request.validate(1).expect("VM-bound request");
    let mut unbound = request.clone();
    unbound.vm_binding = None;
    unbound.vm_profile = None;
    unbound.omp.model_broker.forwarded = None;
    assert_eq!(unbound.worker_profile_identity(), request.worker_profile_identity());
    unbound.validate(1).expect("the same request without its binding is a local request");
}

// r[verify lattice.vm_profile.request.scenario.crossed]
#[test]
fn crossed_bindings_and_missing_forwarding_are_refused_before_effect() {
    let other = format!("b3:{}", "6".repeat(64));
    let cases: [(&str, fn(&mut Request)); 12] = [
        ("network", |request| request.network = None),
        ("network", |request| request.network = Some(true)),
        ("crossed_envelope", |request| {
            request.vm_binding.as_mut().expect("binding").envelope = "9".repeat(64)
        }),
        ("crossed_attempt", |request| {
            request.vm_binding.as_mut().expect("binding").attempt_id = "other".into()
        }),
        ("crossed_fence", |request| request.vm_binding.as_mut().expect("binding").fence = 8),
        ("mobility_not_offered", |request| {
            request.vm_binding.as_mut().expect("binding").requested_mobility = crate::vm_profile::Mobility::LivePrecopy;
        }),
        ("crossed_profile", |request| request.vm_profile.as_mut().expect("profile").resources.vcpus = 4),
        ("guest_network_not_none", |request| {
            request.vm_profile.as_mut().expect("profile").guest_network = "tap".into();
        }),
        ("present together", |request| request.omp.model_broker.forwarded = None),
        ("present together", |request| request.vm_profile = None),
        ("present together", |request| request.vm_binding = None),
        ("another VM binding", |request| {
            request.omp.model_broker.forwarded.as_mut().expect("forwarded").vsock_port =
                crate::vm_profile::VsockPort(10242);
        }),
    ];
    for (expected, change) in cases {
        let mut request = bound();
        change(&mut request);
        let error = request.validate(1).expect_err(expected);
        assert!(error.contains(expected), "{expected}: {error}");
    }
    let mut crossed = bound();
    crossed.omp.model_broker.forwarded.as_mut().expect("forwarded").binding = other;
    assert!(crossed.validate(1).expect_err("crossed forwarding").contains("another VM binding"));
    let mut credentialed = bound();
    credentialed.omp.model_broker.credential = Some(crate::credentials::SecretRef::new("env", "MODEL_TOKEN", None));
    let error = rebound(credentialed).validate(1).expect_err("credential in the guest");
    assert!(error.contains("no model credential"), "{error}");
}

// r[verify lattice.vm_profile.request.scenario.inspect]
#[test]
fn inspect_reports_guest_facts_only_when_bound() {
    let readiness = crate::execution_binding::serving::readiness::observe(
        None,
        "attempt",
        &mut crate::execution_binding::serving::UnavailableOwner,
    )
    .expect("readiness");
    let facts = |request: &Request| {
        let facts = super::super::inspect::profile_facts(request, &readiness, None).expect("profile facts");
        serde_json::to_value(facts).expect("facts JSON")
    };
    let local = facts(&host_local());
    let bound_request = bound();
    let guest = facts(&bound_request);
    assert_eq!(local["isolation"]["value"], "required-sandbox");
    assert_eq!(local["network"]["value"], "attempt-local-model-broker");
    assert_eq!(local["checkpoint"]["support"], "unavailable");
    assert_eq!(guest["isolation"]["value"], "vm-guest");
    assert_eq!(guest["network"]["value"], "host-gateway-model-broker");
    assert_eq!(guest["checkpoint"]["value"], "workflow-node-boundary-v1");
    let mut pinned = bound_request;
    pinned.vm_profile = Some(fixture("execution-profile/admitted-pinned.json"));
    assert_eq!(facts(&pinned)["checkpoint"]["support"], "unavailable");
}
