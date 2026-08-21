use lattice_workflow_contract::*;
use lattice_workflow_transport::*;

const TEST_MAXIMUM_NODES: u32 = 8;
const TEST_MAXIMUM_EDGES: u32 = 16;
const TEST_MAXIMUM_LOG_REFERENCES: u32 = 8;
const TEST_MAXIMUM_LOG_READ_BYTES: u64 = 4_096;
const TEST_MAXIMUM_TOTAL_LOG_BYTES: u64 = 16_384;
const TEST_FRAME_BYTES: u32 = 1_048_576;
const TEST_LOG_BYTES: u32 = 65_536;
const TEST_CONNECTIONS: u32 = 8;
const TEST_ACTIVE_EXECUTIONS: u32 = 2;
const TEST_DEADLINE_MILLISECONDS: u64 = 5_000;
const TEST_USER_ID: u32 = 1_000;
const TEST_IDENTITY_DIGITS: usize = 64;
const TEST_WORKFLOW: &str = "exchange-test";
const TEST_COMMAND: &str = "printf exchange";
const TEST_SOCKET: &str = "/tmp/lattice-workflow-exchange-test.sock";
const SOCKET_READY_ATTEMPTS: usize = 100;
const SOCKET_READY_DELAY_MILLISECONDS: u64 = 10;
const RUNNING_OBSERVATION_ATTEMPTS: usize = 100;
const RUNNING_OBSERVATION_DELAY_MILLISECONDS: u64 = 10;

fn identity(digit: char) -> Identity {
    Identity::new(format!("{BLAKE3_PREFIX}{}", digit.to_string().repeat(TEST_IDENTITY_DIGITS)))
        .expect("test identity is valid")
}

fn label(value: &str) -> Label {
    Label::new(value.to_string()).expect("test label is valid")
}

fn source_value(value: &str) -> SourceValue {
    SourceValue::new(value.to_string()).expect("test source value is valid")
}

fn contract_profile() -> CapabilityProfile {
    CapabilityProfile {
        schema: PROFILE_SCHEMA.to_string(),
        graph_schema: GRAPH_SCHEMA.to_string(),
        platforms: vec![label("linux-x86_64")],
        node_kinds: vec![label("build")],
        effect_classes: vec![EffectClass::Build],
        adapter_kinds: vec![label("local")],
        controls: ControlCapabilities {
            cancel: true,
            pause: true,
            resume: true,
        },
        limits: ContractLimits {
            maximum_nodes: TEST_MAXIMUM_NODES,
            maximum_edges: TEST_MAXIMUM_EDGES,
            maximum_log_references: TEST_MAXIMUM_LOG_REFERENCES,
            maximum_log_read_bytes: TEST_MAXIMUM_LOG_READ_BYTES,
            maximum_total_log_bytes: TEST_MAXIMUM_TOTAL_LOG_BYTES,
        },
        non_claims: lattice_workflow_contract::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

fn request(profile: &CapabilityProfile, operation_identity: Identity) -> WorkflowRequest {
    let mut graph = WorkflowGraph {
        schema: GRAPH_SCHEMA.to_string(),
        identity: identity('0'),
        nodes: vec![WorkflowNode {
            id: label("build"),
            kind: label("build"),
            effect_class: EffectClass::Build,
            adapter_kind: label("local"),
            operation_identity,
        }],
        edges: Vec::new(),
    };
    graph.identity = graph_identity(&graph);
    let mut request = WorkflowRequest {
        schema: REQUEST_SCHEMA.to_string(),
        identity: identity('0'),
        idempotency_key: identity('2'),
        profile_identity: profile_identity(profile),
        platform: label("linux-x86_64"),
        kiln: KilnLinkage {
            effect_identity: identity('3'),
            attempt_identity: identity('4'),
            candidate_identity: identity('5'),
            pipeline_revision_identity: identity('6'),
            workflow_policy_identity: identity('7'),
        },
        source: None,
        graph,
        controls: RequestedControls {
            cancel: true,
            pause: true,
            resume: true,
        },
        observation: ObservationPolicy {
            maximum_log_references: TEST_MAXIMUM_LOG_REFERENCES,
            maximum_log_read_bytes: TEST_MAXIMUM_LOG_READ_BYTES,
            maximum_total_log_bytes: TEST_MAXIMUM_TOTAL_LOG_BYTES,
        },
    };
    request.identity = lattice_workflow_contract::request_identity(&request);
    request
}

fn transport_profile() -> TransportProfile {
    TransportProfile {
        schema: TRANSPORT_PROFILE_SCHEMA.to_string(),
        limits: TransportLimits {
            maximum_frame_bytes: TEST_FRAME_BYTES,
            maximum_log_bytes: TEST_LOG_BYTES,
            maximum_connections: TEST_CONNECTIONS,
            read_deadline_milliseconds: TEST_DEADLINE_MILLISECONDS,
            write_deadline_milliseconds: TEST_DEADLINE_MILLISECONDS,
        },
        peers: PeerPolicy {
            allowed_user_ids: vec![TEST_USER_ID],
        },
        non_claims: lattice_workflow_transport::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

fn fixture(
    command: &str,
) -> (
    tempfile::TempDir,
    crate::store::Store,
    crate::engine::workflow::Workflow,
    CapabilityProfile,
    WorkflowRequest,
    super::HandlerProfile,
) {
    let directory = tempfile::tempdir().expect("temporary directory exists");
    let store = crate::store::Store::new(directory.path().join("lattice.redb")).expect("fixture store opens");
    let mut workflow =
        crate::engine::workflow::Workflow::new(TEST_WORKFLOW, crate::engine::workflow::WorkflowTimestamp::epoch());
    let workflow_node = workflow.add_shell_node("build", command);
    store.save_workflow(&workflow).expect("fixture workflow saves");
    let contract = contract_profile();
    let operation_identity = identity('1');
    let request = request(&contract, operation_identity.clone());
    let facts = super::application::workflow_facts(&workflow).expect("workflow facts are valid");
    let profile = super::HandlerProfile {
        schema: super::HANDLER_PROFILE_SCHEMA.to_string(),
        socket_path: std::path::PathBuf::from(TEST_SOCKET),
        transport: transport_profile(),
        maximum_active_executions: TEST_ACTIVE_EXECUTIONS,
        routes: vec![super::WorkflowRoute {
            profile_identity: request.profile_identity.clone(),
            graph_identity: request.graph.identity.clone(),
            workflow_name: TEST_WORKFLOW.to_string(),
            workflow_revision: facts.revision,
            effect_authority: super::EffectAuthorityMode::RouteExact,
            source_binding: super::SourceBindingMode::Unavailable,
            nodes: vec![super::RouteNode {
                request_node_id: request.graph.nodes[0].id.clone(),
                operation_identity,
                workflow_node_id: workflow_node.0,
            }],
        }],
        non_claims: super::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    };
    (directory, store, workflow, contract, request, profile)
}

fn accepted_from_response(response: ExchangeResponse) -> WorkflowAccepted {
    match response.result {
        ExchangeResult::Success(success) => match *success {
            ExchangeSuccess::Accepted(accepted) => accepted,
            _ => panic!("expected accepted response"),
        },
        ExchangeResult::Rejected(rejection) => panic!("dispatch rejected: {rejection:?}"),
    }
}

fn observed(response: ExchangeResponse) -> ExecutionObservation {
    match response.result {
        ExchangeResult::Success(success) => match *success {
            ExchangeSuccess::Observation(observation) => observation,
            _ => panic!("expected observation response"),
        },
        ExchangeResult::Rejected(rejection) => panic!("observation rejected: {rejection:?}"),
    }
}

// r[verify lattice.workflow_interchange.application_profile]
// r[verify lattice.workflow_interchange.application_profile.scenario.admitted]
// r[verify lattice.workflow_interchange.application_profile.scenario.drift]
#[test]
fn handler_profile_and_exact_route_reject_revision_drift() {
    let (_directory, _store, workflow, _contract, request, mut profile) = fixture(TEST_COMMAND);
    assert_eq!(super::validate_handler_profile(&profile), Ok(()));
    let facts = super::application::workflow_facts(&workflow).expect("workflow facts are valid");
    assert_eq!(super::validate_route(&profile.routes[0], &request, &facts), Ok(()));

    profile.routes[0].workflow_revision = identity('f');
    assert_eq!(
        super::validate_route(&profile.routes[0], &request, &facts),
        Err(super::RouteError::WorkflowRevision)
    );
}

// r[verify lattice.workflow_interchange.source_binding]
// r[verify lattice.workflow_interchange.source_binding.scenario.admitted]
// r[verify lattice.workflow_interchange.source_binding.scenario.collision]
// r[verify lattice.workflow_interchange.source_authority]
// r[verify lattice.workflow_interchange.source_authority.scenario.drift]
#[test]
fn source_context_instantiates_exact_shell_environment_and_revision() {
    const SOURCE_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let command = format!("test \"$LATTICE_SOURCE_REVISION\" = {SOURCE_REVISION}");
    let (_directory, store, mut workflow, contract, mut request, mut profile) = fixture(&command);
    request.source = Some(SourceContext {
        provider: label("radicle"),
        repository: source_value("rad:z2wsvXm5S2sJGvuV1k5JHiwi1PbKE"),
        revision: source_value(SOURCE_REVISION),
    });
    request.identity = lattice_workflow_contract::request_identity(&request);
    profile.routes[0].source_binding = super::SourceBindingMode::ShellEnvironment;
    let route = profile.routes[0].clone();
    let (instantiated, instantiated_revision) =
        super::instantiate_workflow(&route, &request, workflow.clone()).expect("source binding is admitted");
    assert_ne!(instantiated_revision, route.workflow_revision);
    let config = instantiated.configs.get(&route.nodes[0].workflow_node_id).expect("mapped node config exists");
    let crate::engine::workflow::NodeConfig::Shell { env, .. } = config else {
        panic!("mapped node must remain a shell node");
    };
    assert_eq!(env.get(super::SOURCE_REPOSITORY_ENV).map(String::as_str), Some("rad:z2wsvXm5S2sJGvuV1k5JHiwi1PbKE"));
    if let Some(crate::engine::workflow::NodeConfig::Shell { env, .. }) =
        workflow.configs.get_mut(&route.nodes[0].workflow_node_id)
    {
        env.insert(super::SOURCE_REVISION_ENV.to_string(), "collision".to_string());
    }
    assert_eq!(
        super::instantiate_workflow(&route, &request, workflow).map(|_| ()),
        Err(super::RouteError::SourceCollision)
    );

    let handler = super::ApplicationHandler::new(store, profile, crate::agent_node::AgentRuntimeConfig::default())
        .expect("handler starts");
    let mut service = WorkflowExchangeService::new(handler);
    let dispatch = new_request(ExchangeCommand::Dispatch {
        profile: contract.clone(),
        request: request.clone(),
    })
    .expect("dispatch envelope is valid");
    let accepted = accepted_from_response(service.exchange(&dispatch).expect("dispatch succeeds"));
    assert_eq!(accepted.workflow_identity, instantiated_revision);
    service.handler_mut().wait_for_workers();
    let observe = new_request(ExchangeCommand::Observe {
        profile: contract,
        request,
        accepted: accepted.clone(),
    })
    .expect("observe envelope is valid");
    let observation = observed(service.exchange(&observe).expect("observe succeeds"));
    assert_eq!(observation.workflow_identity, accepted.workflow_identity);
    assert_eq!(observation.state, ExecutionState::Completed);
}

// r[verify lattice.workflow_interchange.application_admission]
// r[verify lattice.workflow_interchange.application_admission.scenario.recovery]
// r[verify lattice.workflow_interchange.application_execution]
// r[verify lattice.workflow_interchange.application_execution.scenario.completed]
// r[verify lattice.workflow_interchange.application_recovery]
// r[verify lattice.workflow_interchange.application_logs]
// r[verify lattice.workflow_interchange.application_logs.scenario.linked]
// r[verify lattice.workflow_interchange.application_logs.scenario.absent]
// r[verify lattice.workflow_interchange.application_verification]
// r[verify lattice.workflow_interchange.application_verification.scenario.focused]
#[test]
fn dispatch_executes_once_recovers_acceptance_and_reads_bounded_log() {
    let (_directory, store, _workflow, contract, request, profile) = fixture(TEST_COMMAND);
    let handler = super::ApplicationHandler::new(store, profile, crate::agent_node::AgentRuntimeConfig::default())
        .expect("handler starts");
    let mut service = WorkflowExchangeService::new(handler);
    let dispatch = new_request(ExchangeCommand::Dispatch {
        profile: contract.clone(),
        request: request.clone(),
    })
    .expect("dispatch envelope is valid");
    let accepted = accepted_from_response(service.exchange(&dispatch).expect("dispatch succeeds"));
    service.handler_mut().wait_for_workers();

    let duplicate =
        accepted_from_response(service.exchange(&dispatch).expect("duplicate dispatch returns the durable acceptance"));
    assert_eq!(duplicate, accepted);
    let recovery = new_request(ExchangeCommand::RecoverDispatch {
        profile: contract.clone(),
        request: request.clone(),
    })
    .expect("recovery envelope is valid");
    assert_eq!(accepted_from_response(service.exchange(&recovery).expect("recovery succeeds")), accepted);

    let observe = new_request(ExchangeCommand::Observe {
        profile: contract.clone(),
        request: request.clone(),
        accepted: accepted.clone(),
    })
    .expect("observe envelope is valid");
    let observation = observed(service.exchange(&observe).expect("observe succeeds"));
    assert_eq!(observation.state, ExecutionState::Completed);
    assert!(observation.successful);
    let log = observation.logs.first().expect("terminal diagnostic log is retained");
    let log_read = LogReadRequest {
        schema: LOG_READ_SCHEMA.to_string(),
        request_identity: request.identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        node_id: log.node_id.clone(),
        stream: log.stream,
        cursor: log.cursor,
        maximum_bytes: TEST_MAXIMUM_LOG_READ_BYTES,
    };
    let read = new_request(ExchangeCommand::ReadLog {
        profile: contract,
        request,
        accepted,
        log_read,
    })
    .expect("log envelope is valid");
    let response = service.exchange(&read).expect("log read succeeds");
    assert!(matches!(
        response.result,
        ExchangeResult::Success(success) if matches!(success.as_ref(), ExchangeSuccess::Log { bytes, .. } if bytes == b"completed\n")
    ));

    let mut missing = read;
    if let ExchangeCommand::ReadLog { log_read, .. } = &mut missing.command {
        log_read.cursor = log_read.cursor.saturating_add(1);
    }
    missing.identity =
        lattice_workflow_transport::request_identity(&missing.command).expect("changed log request identity is valid");
    let response = service.exchange(&missing).expect("missing log returns a closed response");
    assert!(matches!(response.result, ExchangeResult::Rejected(_)));
}

// r[verify lattice.workflow_interchange.application_admission.scenario.conflict]
#[test]
fn reused_idempotency_key_with_changed_request_is_rejected() {
    let (_directory, store, _workflow, contract, request, profile) = fixture(TEST_COMMAND);
    let route = profile.routes[0].clone();
    let profile_identity = super::handler_profile_identity(&profile).expect("handler profile identity exists");
    let first = store
        .admit_workflow_exchange(request.clone(), &route, route.workflow_revision.clone(), profile_identity.clone())
        .expect("first request is admitted");
    assert!(matches!(first, super::AdmissionResolution::Inserted(_)));

    let mut changed = request;
    changed.kiln.attempt_identity = identity('a');
    changed.identity = lattice_workflow_contract::request_identity(&changed);
    let error = store
        .admit_workflow_exchange(changed, &route, route.workflow_revision.clone(), profile_identity)
        .expect_err("changed request cannot reuse idempotency");
    assert!(matches!(error, crate::store::StoreError::Conflict(_)));
    assert_eq!(validate_profile(&contract), Ok(()));
}

// r[verify lattice.workflow_interchange.application_lifecycle]
// r[verify lattice.workflow_interchange.application_lifecycle.scenario.cancel]
// r[verify lattice.workflow_interchange.application_lifecycle.scenario.stale]
#[test]
fn stale_cancel_and_unsupported_pause_fail_closed() {
    let (_directory, store, _workflow, contract, request, mut profile) = fixture("sleep 2");
    profile.maximum_active_executions = 1;
    let handler = super::ApplicationHandler::new(store, profile, crate::agent_node::AgentRuntimeConfig::default())
        .expect("handler starts");
    let mut service = WorkflowExchangeService::new(handler);
    let dispatch = new_request(ExchangeCommand::Dispatch {
        profile: contract.clone(),
        request: request.clone(),
    })
    .expect("dispatch envelope is valid");
    let accepted = accepted_from_response(service.exchange(&dispatch).expect("dispatch succeeds"));
    let observe = new_request(ExchangeCommand::Observe {
        profile: contract.clone(),
        request: request.clone(),
        accepted: accepted.clone(),
    })
    .expect("observe envelope is valid");
    let mut current = observed(service.exchange(&observe).expect("observe succeeds"));
    for _attempt in 0..RUNNING_OBSERVATION_ATTEMPTS {
        if current.state == ExecutionState::Running {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(RUNNING_OBSERVATION_DELAY_MILLISECONDS));
        current = observed(service.exchange(&observe).expect("running observation succeeds"));
    }
    assert_eq!(current.state, ExecutionState::Running);

    let mut concurrent = request.clone();
    concurrent.idempotency_key = identity('e');
    concurrent.identity = lattice_workflow_contract::request_identity(&concurrent);
    let concurrent = new_request(ExchangeCommand::Dispatch {
        profile: contract.clone(),
        request: concurrent,
    })
    .expect("concurrent dispatch envelope is valid");
    let response = service.exchange(&concurrent).expect("active limit returns a closed response");
    assert!(matches!(response.result, ExchangeResult::Rejected(_)));

    let stale_control = ControlRequest {
        schema: CONTROL_SCHEMA.to_string(),
        request_identity: request.identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        expected_sequence: current.sequence.saturating_add(1),
        action: ControlAction::Cancel,
    };
    let stale = new_request(ExchangeCommand::Control {
        profile: contract.clone(),
        request: request.clone(),
        accepted: accepted.clone(),
        current_sequence: current.sequence,
        control: stale_control,
    })
    .expect("stale control envelope is encodable");
    let response = service.exchange(&stale).expect("stale control returns a closed response");
    assert!(matches!(response.result, ExchangeResult::Rejected(_)));

    let pause = ControlRequest {
        schema: CONTROL_SCHEMA.to_string(),
        request_identity: request.identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        expected_sequence: current.sequence,
        action: ControlAction::Pause,
    };
    let pause = new_request(ExchangeCommand::Control {
        profile: contract.clone(),
        request: request.clone(),
        accepted: accepted.clone(),
        current_sequence: current.sequence,
        control: pause,
    })
    .expect("pause envelope is valid");
    let response = service.exchange(&pause).expect("pause returns a closed response");
    assert!(matches!(response.result, ExchangeResult::Rejected(_)));

    let cancel = ControlRequest {
        schema: CONTROL_SCHEMA.to_string(),
        request_identity: request.identity.clone(),
        workflow_identity: accepted.workflow_identity.clone(),
        execution_identity: accepted.execution_identity.clone(),
        expected_sequence: current.sequence,
        action: ControlAction::Cancel,
    };
    let cancel = new_request(ExchangeCommand::Control {
        profile: contract,
        request,
        accepted,
        current_sequence: current.sequence,
        control: cancel,
    })
    .expect("cancel envelope is valid");
    let response = service.exchange(&cancel).expect("cancel is admitted");
    assert!(matches!(response.result, ExchangeResult::Success(_)));
    service.handler_mut().wait_for_workers();
}

// r[verify lattice.workflow_interchange.application_recovery.scenario.restart]
#[test]
fn accepted_execution_without_live_worker_becomes_unknown_without_redispatch() {
    let (_directory, store, _workflow, contract, request, profile) = fixture(TEST_COMMAND);
    let route = profile.routes[0].clone();
    let profile_identity = super::handler_profile_identity(&profile).expect("handler profile identity exists");
    let admitted = store
        .admit_workflow_exchange(request.clone(), &route, route.workflow_revision.clone(), profile_identity)
        .expect("request is durably admitted");
    let super::AdmissionResolution::Inserted(record) = admitted else {
        panic!("fixture must insert one execution");
    };
    let handler = super::ApplicationHandler::new(store, profile, crate::agent_node::AgentRuntimeConfig::default())
        .expect("replacement handler starts");
    let mut service = WorkflowExchangeService::new(handler);
    let observe = new_request(ExchangeCommand::Observe {
        profile: contract,
        request,
        accepted: record.accepted,
    })
    .expect("observe envelope is valid");
    let observation = observed(service.exchange(&observe).expect("unknown observation succeeds"));
    assert_eq!(observation.state, ExecutionState::Unknown);
    assert!(!observation.terminal);
    assert!(!observation.successful);
}

// r[verify lattice.workflow_interchange.application_execution.scenario.authority]
#[test]
fn unavailable_route_authority_denies_effects_without_fallback() {
    let (_directory, store, _workflow, contract, request, mut profile) = fixture(TEST_COMMAND);
    profile.routes[0].effect_authority = super::EffectAuthorityMode::Unavailable;
    let handler = super::ApplicationHandler::new(store, profile, crate::agent_node::AgentRuntimeConfig::default())
        .expect("handler starts");
    let mut service = WorkflowExchangeService::new(handler);
    let dispatch = new_request(ExchangeCommand::Dispatch {
        profile: contract.clone(),
        request: request.clone(),
    })
    .expect("dispatch envelope is valid");
    let accepted = accepted_from_response(service.exchange(&dispatch).expect("dispatch is accepted"));
    service.handler_mut().wait_for_workers();
    let observe = new_request(ExchangeCommand::Observe {
        profile: contract,
        request,
        accepted,
    })
    .expect("observe envelope is valid");
    let observation = observed(service.exchange(&observe).expect("denied execution is observed"));
    assert_eq!(observation.state, ExecutionState::Failed);
    assert!(!observation.successful);
}

// r[verify lattice.workflow_interchange.application_command.scenario.invalid]
#[test]
fn typed_handler_profile_loads_and_negative_profiles_fail() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let valid = root.join("tests/fixtures/workflow-exchange-handler/profile-valid.ncl");
    let zero_active = root.join("tests/fixtures/workflow-exchange-handler/negative/profile-zero-active.ncl");
    let empty_routes = root.join("tests/fixtures/workflow-exchange-handler/negative/profile-empty-routes.ncl");
    assert!(super::load_profile(&valid).is_ok());
    assert_eq!(super::load_profile(&zero_active), Err(super::ApplicationError::Profile));
    assert_eq!(super::load_profile(&empty_routes), Err(super::ApplicationError::Profile));
}

// r[verify lattice.workflow_interchange.application_command]
// r[verify lattice.workflow_interchange.application_command.scenario.valid]
#[cfg(target_os = "linux")]
#[test]
fn unix_service_composes_the_application_handler_and_bounded_transport() {
    let (directory, store, _workflow, contract, request, mut profile) = fixture(TEST_COMMAND);
    let socket_path = directory.path().join("workflow.sock");
    profile.socket_path = socket_path.clone();
    profile.transport.limits.maximum_connections = 1;
    profile.maximum_active_executions = 1;
    profile.transport.peers.allowed_user_ids = vec![nix::unistd::getuid().as_raw()];
    let client_profile = profile.transport.clone();
    let server =
        std::thread::spawn(move || super::serve(store, profile, crate::agent_node::AgentRuntimeConfig::default()));
    for _attempt in 0..SOCKET_READY_ATTEMPTS {
        if socket_path.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(SOCKET_READY_DELAY_MILLISECONDS));
    }
    assert!(socket_path.exists());
    let dispatch = new_request(ExchangeCommand::Dispatch {
        profile: contract,
        request,
    })
    .expect("dispatch envelope is valid");
    let endpoint = UnixEndpoint {
        socket_path: &socket_path,
        profile: &client_profile,
    };
    let response = UnixClient::new(endpoint).exchange(&dispatch).expect("Unix application exchange succeeds");
    assert!(matches!(
        response.result,
        ExchangeResult::Success(success) if matches!(success.as_ref(), ExchangeSuccess::Accepted(_))
    ));
    let report = server.join().expect("server thread joins").expect("server finishes cleanly");
    assert_eq!(report.accepted_connections, 1);
    assert_eq!(report.completed_exchanges, 1);
}
