use alloc::{string::ToString, vec::Vec};

use super::*;

const IMPLEMENTATION: &str = "implementation.eval.v1";
const POLICY: &str = "policy.eval.v1";
const CATALOG: &str = "catalog.eval.v1";
const RESULT: &str = "result.eval.v1";
const REQUESTED_ITEMS: u32 = 8;
const REQUESTED_BYTES: u64 = 4_096;
const SERVICE_ITEMS: u32 = 16;
const SERVICE_BYTES: u64 = 8_192;
const INPUT_TOKENS: u64 = 100;
const OUTPUT_TOKENS: u64 = 20;
const CACHED_TOKENS: u64 = 5;
const REASONING_TOKENS: u64 = 3;
const LATER_ATTEMPT: u32 = 2;

fn binding<'a>() -> EvaluationBinding<'a> {
    EvaluationBinding {
        evaluation_identity: "evaluation.alpha",
        plan_identity: "plan.alpha",
        task_identity: "task.alpha",
        episode_identity: "episode.alpha",
        run_identity: "run.alpha",
        role_identity: "role.solver",
        model_identity: "model.alpha",
        harness_identity: "harness.alpha",
        environment_identity: "environment.solver",
        solver_environment_identity: "environment.solver",
        tool_identity: "tools.alpha",
        verifier_identity: "verifier.alpha",
        rubric_identity: "rubric.alpha",
        artifact_identity: "artifact.manifest.alpha",
        policy_identity: POLICY,
        cancellation_identity: "cancel.alpha",
        deadline_identity: "deadline.alpha",
        attempt: 1,
        attempt_lineage_identity: "attempt.lineage.alpha",
        correlation_identity: "correlation.alpha",
    }
}

fn descriptor(kind: ServiceKind) -> ServiceDescriptor<'static> {
    let capability = match kind {
        ServiceKind::TaskSource => &[ServiceCapability::TaskSource][..],
        ServiceKind::AgentExecution => &[ServiceCapability::AgentExecution][..],
        ServiceKind::VerifierExecution => &[ServiceCapability::VerifierExecution][..],
        ServiceKind::ArtifactCollect => &[ServiceCapability::ArtifactCollect][..],
        ServiceKind::ArtifactRestore => &[ServiceCapability::ArtifactRestore][..],
        ServiceKind::ModelCallCapture => &[ServiceCapability::ModelCallCapture][..],
        ServiceKind::ResultAppend => &[ServiceCapability::ResultAppend][..],
        ServiceKind::ResultReplace => &[ServiceCapability::ResultReplace][..],
        ServiceKind::ResultReopen => &[ServiceCapability::ResultReopen][..],
    };
    ServiceDescriptor {
        kind,
        implementation_identity: IMPLEMENTATION,
        capabilities: capability,
        availability: ServiceAvailability::Available,
        bounds: ServiceResourceBounds {
            max_items: SERVICE_ITEMS,
            max_bytes: SERVICE_BYTES,
        },
        policy_identity: POLICY,
    }
}

fn admission(kind: ServiceKind) -> ServiceAdmission {
    let services = [descriptor(kind)];
    admit_service(
        &ServiceCatalog {
            catalog_identity: CATALOG,
            services: &services,
        },
        &ServiceRequirement {
            kind,
            implementation_identity: Some(IMPLEMENTATION),
            required_items: REQUESTED_ITEMS,
            required_bytes: REQUESTED_BYTES,
            policy_identity: POLICY,
        },
    )
    .expect("valid service must be admitted")
}

fn request(kind: ServiceKind) -> ServiceRequest<'static> {
    let admission = admission(kind);
    let mut exact_binding = binding();
    if kind == ServiceKind::VerifierExecution {
        exact_binding.environment_identity = "environment.verifier";
    }
    build_service_request(
        kind,
        &admission.identity,
        IMPLEMENTATION,
        &exact_binding,
        REQUESTED_ITEMS,
        REQUESTED_BYTES,
        &admission,
    )
    .expect("valid request must bind")
}

fn success(request: &ServiceRequest<'_>) -> ServiceObservation {
    ServiceObservation {
        service: request.service,
        request_identity: request.identity().expect("request identity"),
        binding_identity: binding_identity(&request.binding).expect("binding identity"),
        correlation_identity: request.binding.correlation_identity.to_string(),
        implementation_identity: request.implementation_identity.clone(),
        attempt: request.binding.attempt,
        outcome: ObservationClass::Success,
        result_identity: Some(RESULT.to_string()),
        item_count: 1,
        byte_count: 1,
    }
}

#[derive(Default)]
struct FakePorts {
    calls: Vec<ServiceKind>,
}

impl FakePorts {
    fn observe(&mut self, request: &ServiceRequest<'_>) -> ServiceObservation {
        self.calls.push(request.service());
        success(request)
    }
}

impl TaskSourcePort for FakePorts {
    fn materialize(&mut self, request: &TaskSourceRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}
impl AgentExecutionPort for FakePorts {
    fn execute_agent(&mut self, request: &AgentExecutionRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}
impl VerifierExecutionPort for FakePorts {
    fn execute_verifier(&mut self, request: &VerifierExecutionRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}
impl ArtifactCollectPort for FakePorts {
    fn collect_artifacts(&mut self, request: &ArtifactCollectRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}
impl ArtifactRestorePort for FakePorts {
    fn restore_artifacts(&mut self, request: &ArtifactRestoreRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}
impl ModelCallCapturePort for FakePorts {
    fn capture_model_call(&mut self, request: &ModelCallCaptureRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}
impl ResultAppendPort for FakePorts {
    fn append_result(&mut self, request: &ResultAppendRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}
impl ResultReplacePort for FakePorts {
    fn replace_result(&mut self, request: &ResultReplaceRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}
impl ResultReopenPort for FakePorts {
    fn reopen_result(&mut self, request: &ResultReopenRequest<'_>) -> ServiceObservation {
        self.observe(request.service())
    }
}

#[test]
fn every_separate_port_returns_exact_validated_observation() {
    let mut ports = FakePorts::default();
    let task = TaskSourceRequest::new(request(ServiceKind::TaskSource)).expect("task request");
    let agent =
        AgentExecutionRequest::new(request(ServiceKind::AgentExecution)).expect("agent request");
    let verifier = VerifierExecutionRequest::new(request(ServiceKind::VerifierExecution))
        .expect("verifier request");
    let collect = ArtifactCollectRequest::new(request(ServiceKind::ArtifactCollect))
        .expect("artifact collect request");
    let restore = ArtifactRestoreRequest::new(request(ServiceKind::ArtifactRestore))
        .expect("artifact restore request");
    let capture = ModelCallCaptureRequest::new(request(ServiceKind::ModelCallCapture))
        .expect("model capture request");
    let append = ResultAppendRequest::new(request(ServiceKind::ResultAppend))
        .expect("result append request");
    let replace = ResultReplaceRequest::new(request(ServiceKind::ResultReplace))
        .expect("result replace request");
    let reopen = ResultReopenRequest::new(request(ServiceKind::ResultReopen))
        .expect("result reopen request");
    let observations = [
        ports.materialize(&task),
        ports.execute_agent(&agent),
        ports.execute_verifier(&verifier),
        ports.collect_artifacts(&collect),
        ports.restore_artifacts(&restore),
        ports.capture_model_call(&capture),
        ports.append_result(&append),
        ports.replace_result(&replace),
        ports.reopen_result(&reopen),
    ];
    let requests = [
        task.into_service(),
        agent.into_service(),
        verifier.into_service(),
        collect.into_service(),
        restore.into_service(),
        capture.into_service(),
        append.into_service(),
        replace.into_service(),
        reopen.into_service(),
    ];
    assert_eq!(observations.len(), requests.len());
    for (service_request, observation) in requests.iter().zip(&observations) {
        let validated = validate_service_observation(service_request, observation)
            .expect("matching fake observation must validate");
        assert_eq!(
            reduce_observation(&validated),
            ObservationDisposition::Eligible
        );
        assert_eq!(validated.result_identity.as_deref(), Some(RESULT));
    }
    assert_eq!(ports.calls.len(), requests.len());
}

#[test]
fn catalog_fails_closed_for_absence_widening_and_availability() {
    let empty = ServiceCatalog {
        catalog_identity: CATALOG,
        services: &[],
    };
    assert_eq!(
        validate_service_catalog(&empty),
        Err(AdmissionError::Catalog)
    );

    let widened = ServiceDescriptor {
        capabilities: &[ServiceCapability::ProviderTransport],
        ..descriptor(ServiceKind::TaskSource)
    };
    assert_eq!(
        validate_service_catalog(&ServiceCatalog {
            catalog_identity: CATALOG,
            services: &[widened],
        }),
        Err(AdmissionError::CapabilityWidening)
    );

    for (availability, expected) in [
        (ServiceAvailability::Absent, AdmissionError::Absent),
        (ServiceAvailability::Denied, AdmissionError::Denied),
        (
            ServiceAvailability::Unavailable,
            AdmissionError::Unavailable,
        ),
        (
            ServiceAvailability::Unsupported,
            AdmissionError::Unsupported,
        ),
        (ServiceAvailability::Stale, AdmissionError::Stale),
    ] {
        let unavailable = ServiceDescriptor {
            availability,
            ..descriptor(ServiceKind::TaskSource)
        };
        assert_eq!(
            admit_service(
                &ServiceCatalog {
                    catalog_identity: CATALOG,
                    services: &[unavailable],
                },
                &ServiceRequirement {
                    kind: ServiceKind::TaskSource,
                    implementation_identity: Some(IMPLEMENTATION),
                    required_items: REQUESTED_ITEMS,
                    required_bytes: REQUESTED_BYTES,
                    policy_identity: POLICY,
                },
            ),
            Err(expected)
        );
    }

    let available = descriptor(ServiceKind::TaskSource);
    assert_eq!(
        admit_service(
            &ServiceCatalog {
                catalog_identity: CATALOG,
                services: &[available],
            },
            &ServiceRequirement {
                kind: ServiceKind::AgentExecution,
                implementation_identity: Some(IMPLEMENTATION),
                required_items: REQUESTED_ITEMS,
                required_bytes: REQUESTED_BYTES,
                policy_identity: POLICY,
            },
        ),
        Err(AdmissionError::Absent)
    );
}

#[test]
fn binding_fails_closed_for_stale_isolation_artifact_and_storage_mismatch() {
    assert!(matches!(
        TaskSourceRequest::new(request(ServiceKind::AgentExecution)),
        Err(BindingError::Mismatch)
    ));
    let verifier_admission = admission(ServiceKind::VerifierExecution);
    assert_eq!(
        build_service_request(
            ServiceKind::VerifierExecution,
            &verifier_admission.identity,
            IMPLEMENTATION,
            &binding(),
            REQUESTED_ITEMS,
            REQUESTED_BYTES,
            &verifier_admission,
        ),
        Err(BindingError::Isolation)
    );

    let agent_request = request(ServiceKind::AgentExecution);
    let mut observation = success(&agent_request);
    observation.correlation_identity = "correlation.stale".to_string();
    assert_eq!(
        validate_service_observation(&agent_request, &observation),
        Err(BindingError::Mismatch)
    );
    observation = success(&agent_request);
    observation.attempt += 1;
    assert_eq!(
        validate_service_observation(&agent_request, &observation),
        Err(BindingError::Mismatch)
    );
    observation = success(&agent_request);
    observation.result_identity = None;
    assert_eq!(
        validate_service_observation(&agent_request, &observation),
        Err(BindingError::Incomplete)
    );

    let original = success(&agent_request);
    let mut cross_plan = agent_request.clone();
    cross_plan.binding.plan_identity = "plan.other";
    assert_eq!(
        validate_service_observation(&cross_plan, &original),
        Err(BindingError::Mismatch)
    );
    let mut widened_artifact = agent_request.clone();
    widened_artifact.binding.artifact_identity = "artifact.manifest.widened";
    assert_eq!(
        validate_service_observation(&widened_artifact, &original),
        Err(BindingError::Mismatch)
    );

    let storage = request(ServiceKind::ResultAppend);
    let mut storage_mismatch = success(&storage);
    storage_mismatch.outcome = ObservationClass::InfrastructureFailure;
    assert_eq!(
        validate_service_observation(&storage, &storage_mismatch),
        Err(BindingError::Contradiction)
    );
}

#[test]
fn gateway_projection_commit_order_and_coalescing_are_exact() {
    let projection = project_model_call(&ModelCallProjectionDraft {
        dialect: ModelDialect::OpenAiResponses,
        endpoint: EndpointClass::Responses,
        effective_model_identity: "model.effective",
        sampling_identity: "sampling.alpha",
        tool_catalog_identity: "tools.alpha",
        request_identity: "request.alpha",
        run_identity: "run.alpha",
        policy_identity: POLICY,
        attempt: 1,
        streaming: true,
        usage: ModelUsage {
            input_tokens: INPUT_TOKENS,
            output_tokens: OUTPUT_TOKENS,
            cached_input_tokens: CACHED_TOKENS,
            reasoning_tokens: REASONING_TOKENS,
        },
        finish: ModelFinishClass::Stop,
        error: ModelErrorClass::None,
        retains_raw_provider_body: false,
        rewrites_response: false,
        shares_model_and_state_capability: false,
        unbounded_material: false,
    })
    .expect("safe metadata projection must pass");
    assert_eq!(projection.endpoint, EndpointClass::Responses);

    let terminal = TerminalStreamEvidence {
        run_identity: "run.alpha",
        request_identity: "request.alpha",
        turn_identity: "turn.alpha",
        policy_identity: POLICY,
        attempt: 1,
        result_identity: RESULT,
        commit_identity: "commit.alpha",
    };
    let mut commit = TurnCommitEvidence {
        run_identity: "run.alpha",
        request_identity: "request.alpha",
        turn_identity: "turn.alpha",
        policy_identity: POLICY,
        attempt: 1,
        commit_identity: "commit.alpha",
        result_identity: RESULT,
        complete: false,
    };
    assert_eq!(
        admit_terminal_after_commit(&terminal, &commit),
        Err(GatewayError::CommitOrder)
    );
    commit.complete = true;
    assert!(admit_terminal_after_commit(&terminal, &commit).is_ok());

    let exact = CoalescingEvidence {
        first_request_identity: "request.alpha",
        later_request_identity: "request.alpha",
        first_run_identity: "run.alpha",
        later_run_identity: "run.alpha",
        first_model_identity: "model.alpha",
        later_model_identity: "model.alpha",
        first_sampling_identity: "sampling.alpha",
        later_sampling_identity: "sampling.alpha",
        first_policy_identity: POLICY,
        later_policy_identity: POLICY,
        first_turn_identity: "turn.alpha",
        later_turn_identity: "turn.alpha",
        attempt_lineage_identity: "attempt.lineage.alpha",
        first_result_identity: RESULT,
        first_attempt: 1,
        later_attempt: LATER_ATTEMPT,
        identical_bounded_request: true,
        non_streaming: true,
    };
    assert!(admit_identical_request_coalescing(&exact).is_ok());
    let content_only = CoalescingEvidence {
        later_policy_identity: "policy.other",
        ..exact
    };
    assert_eq!(
        admit_identical_request_coalescing(&content_only),
        Err(GatewayError::Coalescing)
    );
}

#[test]
fn unsafe_gateway_behavior_is_rejected() {
    let unsafe_gateway = ModelCallProjectionDraft {
        dialect: ModelDialect::OpenAiChatCompletions,
        endpoint: EndpointClass::ChatCompletions,
        effective_model_identity: "model.alpha",
        sampling_identity: "sampling.alpha",
        tool_catalog_identity: "tools.alpha",
        request_identity: "request.alpha",
        run_identity: "run.alpha",
        policy_identity: POLICY,
        attempt: 1,
        streaming: false,
        usage: ModelUsage {
            input_tokens: 1,
            output_tokens: 1,
            cached_input_tokens: 0,
            reasoning_tokens: 0,
        },
        finish: ModelFinishClass::Stop,
        error: ModelErrorClass::None,
        retains_raw_provider_body: true,
        rewrites_response: false,
        shares_model_and_state_capability: false,
        unbounded_material: false,
    };
    assert_eq!(
        project_model_call(&unsafe_gateway),
        Err(GatewayError::Unsafe)
    );
    for unsafe_variant in [
        ModelCallProjectionDraft {
            retains_raw_provider_body: false,
            rewrites_response: true,
            ..unsafe_gateway
        },
        ModelCallProjectionDraft {
            retains_raw_provider_body: false,
            shares_model_and_state_capability: true,
            ..unsafe_gateway
        },
        ModelCallProjectionDraft {
            retains_raw_provider_body: false,
            unbounded_material: true,
            ..unsafe_gateway
        },
    ] {
        assert_eq!(
            project_model_call(&unsafe_variant),
            Err(GatewayError::Unsafe)
        );
    }
}

#[test]
fn receipt_content_and_overclaims_are_rejected() {
    let safe = AdapterReceiptDraft {
        kind: AdapterReceiptKind::ServiceObservation,
        evaluation_identity: "evaluation.alpha",
        plan_identity: "plan.alpha",
        subject_identity: "observation.alpha",
        policy_identity: POLICY,
        implementation_identity: IMPLEMENTATION,
        status: AdapterReceiptStatus::Observed,
        labels: &["metadata_only"],
        item_count: 1,
        redaction_count: 1,
        leaks: AdapterReceiptLeakFlags::default(),
        claims: &[AdapterReceiptClaim::BoundedObservation],
    };
    let receipt = project_adapter_receipt(&safe).expect("safe receipt must pass");
    assert!(receipt.non_claims.contains(&"not-authoritative-completion"));

    let leaking = AdapterReceiptDraft {
        leaks: AdapterReceiptLeakFlags {
            prompt: true,
            ..AdapterReceiptLeakFlags::default()
        },
        ..safe
    };
    assert_eq!(
        project_adapter_receipt(&leaking),
        Err(AdapterReceiptError::Leak)
    );
    let overclaim = AdapterReceiptDraft {
        claims: &[AdapterReceiptClaim::ReleaseEligibility],
        ..safe
    };
    assert_eq!(
        project_adapter_receipt(&overclaim),
        Err(AdapterReceiptError::Overclaim)
    );

    let leak_cases = [
        AdapterReceiptLeakFlags {
            hidden_test: true,
            ..AdapterReceiptLeakFlags::default()
        },
        AdapterReceiptLeakFlags {
            reasoning: true,
            ..AdapterReceiptLeakFlags::default()
        },
        AdapterReceiptLeakFlags {
            provider_body: true,
            ..AdapterReceiptLeakFlags::default()
        },
        AdapterReceiptLeakFlags {
            tool_body: true,
            ..AdapterReceiptLeakFlags::default()
        },
        AdapterReceiptLeakFlags {
            credential: true,
            ..AdapterReceiptLeakFlags::default()
        },
        AdapterReceiptLeakFlags {
            environment_value: true,
            ..AdapterReceiptLeakFlags::default()
        },
        AdapterReceiptLeakFlags {
            physical_path: true,
            ..AdapterReceiptLeakFlags::default()
        },
        AdapterReceiptLeakFlags {
            unbounded_log: true,
            ..AdapterReceiptLeakFlags::default()
        },
    ];
    for leaks in leak_cases {
        assert_eq!(
            project_adapter_receipt(&AdapterReceiptDraft { leaks, ..safe }),
            Err(AdapterReceiptError::Leak)
        );
    }

    let overclaims = [
        AdapterReceiptClaim::ExactlyOnceRemoteEffects,
        AdapterReceiptClaim::VerifierSoundness,
        AdapterReceiptClaim::ArtifactAuthenticity,
        AdapterReceiptClaim::CompleteCapture,
        AdapterReceiptClaim::BenchmarkValidity,
        AdapterReceiptClaim::AuthoritativeCompletion,
        AdapterReceiptClaim::ProductionReadiness,
        AdapterReceiptClaim::ReleaseEligibility,
    ];
    for claim in overclaims {
        assert_eq!(
            project_adapter_receipt(&AdapterReceiptDraft {
                claims: &[claim],
                ..safe
            }),
            Err(AdapterReceiptError::Overclaim)
        );
    }
}
