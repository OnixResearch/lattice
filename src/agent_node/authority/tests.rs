use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use parking_lot::Mutex;

use super::*;
use crate::agent_node::AgentOutputMode;
use crate::agent_node::AgentRuntimeConfig;
use crate::agent_node::AgentTarget;
use crate::agent_node::ProductionAgentAdapter;
use crate::credentials::CredentialError;
use crate::credentials::SecretRef;
use crate::credentials::SecretValue;
use crate::engine::effect::authority;
use crate::engine::effect::dispatch;
use crate::engine::effect::runtime;
use crate::engine::executor::Executor;
use crate::engine::executor::NodeStatus;
use crate::engine::workflow::Workflow;
use crate::engine::workflow::WorkflowTimestamp;

#[derive(Clone)]
struct Provider {
    material: Arc<Mutex<String>>,
    changed_policy: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}

impl Provider {
    fn new() -> Self {
        Self {
            material: Arc::new(Mutex::new("sentinel-agent-token".to_owned())),
            changed_policy: Arc::new(AtomicBool::new(false)),
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl CredentialProvider for Provider {
    fn resolve(&self, _reference: &SecretRef) -> Result<SecretValue, CredentialError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(SecretValue::new(self.material.lock().clone()))
    }

    fn resolution_policy_identity(&self, _reference: &SecretRef) -> Result<Option<&str>, CredentialError> {
        Ok(Some(if self.changed_policy.load(Ordering::SeqCst) {
            "policy-v2"
        } else {
            "policy-v1"
        }))
    }
}

#[derive(Clone, Default)]
struct EchoTransport(Arc<AtomicUsize>);

impl crate::openai::Transport for EchoTransport {
    fn execute(
        &self,
        _plan: &crate::openai::ChatPlan,
        bearer: Option<&str>,
    ) -> Result<crate::openai::HttpResponse, AgentError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let token = bearer.expect("authorized credential reaches selected transport");
        Ok(crate::openai::HttpResponse {
            status: 200,
            body: serde_json::to_vec(&serde_json::json!({
                "model": token,
                "choices": [{"message": {"content": format!("echo {token}")}, "finish_reason": "stop"}],
                "usage": {"echo": token},
            }))
            .unwrap(),
        })
    }
}

fn network_config(target: AgentTarget) -> AgentRuntimeConfig {
    let mut config = AgentRuntimeConfig::default();
    config.runtimes.clear();
    config.openai_compatible.insert(
        super::super::agent_target_label(target).to_owned(),
        crate::openai::RuntimeConfig {
            endpoint: "https://agent.example.test/chat/completions".to_owned(),
            model: "test-model".to_owned(),
            api_key_env: Some("TOKEN".to_owned()),
            timeout_secs: 5,
            max_tokens: 128,
            temperature_milliunits: 0,
            max_request_bytes: 4096,
            max_response_bytes: 4096,
            enable_thinking: Some(false),
            system_prompt: String::new(),
        },
    );
    config
}

fn request(target: AgentTarget) -> AgentRequest {
    AgentRequest {
        target,
        prompt: "say hello".to_owned(),
        output_mode: AgentOutputMode::Text,
        model_hint: None,
        tool_policy: "default".to_owned(),
        context_refs: Vec::new(),
        cwd: None,
        session: None,
        text_inputs: Vec::new(),
        image_inputs: Vec::new(),
        json_inputs: Vec::new(),
    }
}

fn references(key: &str) -> NodeCredentialRefs {
    NodeCredentialRefs {
        env: HashMap::from([("TOKEN".to_owned(), SecretRef::new("env", key, None))]),
    }
}

fn binding(payload: &str, node: u64) -> authority::IntentBinding {
    authority::IntentBinding {
        schema: authority::CHOREGRAPH_EFFECT_AUTHORITY_BINDING_SCHEMA.to_owned(),
        machine: authority::MachineIdentity::new("test-machine").unwrap(),
        workflow_revision: authority::WorkflowRevisionIdentity::new("test-revision").unwrap(),
        run: authority::RunIdentity::new("test-run").unwrap(),
        node: authority::NodeIdentity::new(format!("lattice-node/{node}")).unwrap(),
        transition: authority::TransitionIdentity::new("ready->start").unwrap(),
        operation: authority::OperationIdentity::new("lattice/agent").unwrap(),
        resource: authority::ResourceIdentity::new(format!("lattice://agent/{node}")).unwrap(),
        ability: authority::AbilityIdentity::new("agent/execute").unwrap(),
        request: authority::RequestIdentity::new(runtime::dispatch_request_identity(payload)).unwrap(),
        payload: authority::PayloadIdentity::new(payload).unwrap(),
        policy: authority::PolicyIdentity::new(authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF).unwrap(),
        authority: authority::AuthorityIdentity::new("test-operator").unwrap(),
        intent: authority::IntentIdentity::new(format!("test-intent/{node}")).unwrap(),
        scope: authority::CollaborationScopeBinding::Unscoped,
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Allow,
    Deny,
    Unavailable,
}
struct Facts(Mode);

impl dispatch::AuthorityFactsProvider for Facts {
    fn load(&self, binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, Error> {
        if matches!(self.0, Mode::Unavailable) {
            return Err(Error::AuthorityUnavailable);
        }
        Ok(authority::AuthorizationFacts {
            binding: binding.clone(),
            evidence: authority::Evidence::LocalOperator(authority::LocalOperatorGrant {
                authority_ref: binding.authority.as_str().to_owned(),
                operation: binding.operation.as_str().to_owned(),
                request: basalt::authority_core::AuthorityRequest {
                    contract_id: "lattice-agent".to_owned(),
                    resource: binding.resource.as_str().to_owned(),
                    ability: binding.ability.as_str().to_owned(),
                    request_ref: binding.request.as_str().to_owned(),
                },
            }),
            trusted_policy_ref: dispatch::trusted_policy_ref("lattice-agent"),
            scope_requirement: authority::ScopeRequirement::UnscopedPermitted,
            scope_snapshot: None,
            revocation: if matches!(self.0, Mode::Deny) {
                authority::RevocationDisposition::Revoked
            } else {
                authority::RevocationDisposition::Current
            },
        })
    }
}

struct Scope;
impl dispatch::ScopeSnapshotProvider for Scope {
    fn load_current(
        &self,
        _binding: &authority::IntentBinding,
    ) -> Result<Option<authority::ScopeSnapshotFacts>, Error> {
        Ok(None)
    }
}
struct Attempts;
impl dispatch::EffectAttemptStore for Attempts {
    fn reserve(&self, binding: authority::IntentBinding) -> Result<authority::Attempt, Error> {
        Ok(authority::Attempt {
            identity: authority::AttemptIdentity::new("test-attempt").unwrap(),
            binding,
            sequence: 1,
        })
    }
    fn record_outcome(&self, _event: &authority::AuthorizationOutcomeEvent) -> Result<(), Error> {
        Ok(())
    }
}
fn shell(mode: Mode) -> dispatch::Shell {
    dispatch::Shell::new(
        Box::new(Facts(mode)),
        Box::new(Scope),
        Box::new(Attempts),
        Box::new(runtime::ConfiguredRoots::new(Default::default())),
    )
}
fn dispatch_recipe(recipe: &Recipe<'_>, binding: authority::IntentBinding, mode: Mode) -> Result<AgentResponse, Error> {
    let shell = shell(mode);
    let http = crate::http::NativeHttpClient;
    let process = crate::process::core::RuntimeConfig::default();
    shell.dispatch(&runtime::Adapters::new(&shell, &http, &process), binding, recipe)
}

#[test]
fn exact_agent_authority_gates_material_and_transport_and_redacts_echoes() {
    for (mode, expected) in [
        (Mode::Deny, Some(Error::AuthorityDenied)),
        (Mode::Unavailable, Some(Error::AuthorityUnavailable)),
        (Mode::Allow, None),
    ] {
        let provider = Provider::new();
        let transport = EchoTransport::default();
        let adapter = ProductionAgentAdapter::new_with_openai_transport(
            network_config(AgentTarget::Pi),
            Arc::new(transport.clone()),
        );
        let cancellation = AtomicBool::new(false);
        let recipe = Recipe::new(
            request(AgentTarget::Pi),
            &adapter,
            Some(&references("first")),
            Arc::new(provider.clone()),
            &cancellation,
        )
        .unwrap();
        let result = dispatch_recipe(&recipe, binding(recipe.payload(), 0), mode);
        if let Some(expected) = expected {
            assert_eq!(result, Err(expected));
            assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
            assert_eq!(transport.0.load(Ordering::SeqCst), 0);
        } else {
            let response = result.unwrap();
            assert_eq!(response.output, AgentOutput::Text("echo <redacted>".to_owned()));
            assert_eq!(response.metadata["model"], "<redacted>");
            assert_eq!(response.metadata["usage"]["echo"], "<redacted>");
            assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
            assert_eq!(transport.0.load(Ordering::SeqCst), 1);
            assert_eq!(
                dispatch_recipe(&recipe, binding(recipe.payload(), 0), Mode::Allow),
                Err(Error::AdapterRejected)
            );
        }
    }
}

#[test]
fn stale_agent_request_reference_policy_or_backend_cannot_reuse_authority_but_material_can_rotate() {
    let provider = Provider::new();
    let transport = EchoTransport::default();
    let config = network_config(AgentTarget::Pi);
    let adapter = ProductionAgentAdapter::new_with_openai_transport(config.clone(), Arc::new(transport.clone()));
    let cancellation = AtomicBool::new(false);
    let original = Recipe::new(
        request(AgentTarget::Pi),
        &adapter,
        Some(&references("first")),
        Arc::new(provider.clone()),
        &cancellation,
    )
    .unwrap();
    let sealed = binding(original.payload(), 0);
    let mut changed_request = request(AgentTarget::Pi);
    changed_request.prompt = "different request".to_owned();
    let changed_request =
        Recipe::new(changed_request, &adapter, Some(&references("first")), Arc::new(provider.clone()), &cancellation)
            .unwrap();
    let changed_reference = Recipe::new(
        request(AgentTarget::Pi),
        &adapter,
        Some(&references("second")),
        Arc::new(provider.clone()),
        &cancellation,
    )
    .unwrap();
    let mut changed_config = config;
    changed_config.openai_compatible.get_mut("pi").unwrap().endpoint = "https://different.example.test/chat".to_owned();
    let changed_adapter =
        ProductionAgentAdapter::new_with_openai_transport(changed_config, Arc::new(transport.clone()));
    let changed_backend = Recipe::new(
        request(AgentTarget::Pi),
        &changed_adapter,
        Some(&references("first")),
        Arc::new(provider.clone()),
        &cancellation,
    )
    .unwrap();
    for recipe in [&changed_request, &changed_reference, &changed_backend] {
        assert_eq!(dispatch_recipe(recipe, sealed.clone(), Mode::Allow), Err(Error::AdapterRejected));
    }
    provider.changed_policy.store(true, Ordering::SeqCst);
    assert_eq!(dispatch_recipe(&original, sealed.clone(), Mode::Allow), Err(Error::AdapterRejected));
    let changed_policy = Recipe::new(
        request(AgentTarget::Pi),
        &adapter,
        Some(&references("first")),
        Arc::new(provider.clone()),
        &cancellation,
    )
    .unwrap();
    assert_eq!(dispatch_recipe(&changed_policy, sealed.clone(), Mode::Allow), Err(Error::AdapterRejected));
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    assert_eq!(transport.0.load(Ordering::SeqCst), 0);
    provider.changed_policy.store(false, Ordering::SeqCst);
    *provider.material.lock() = "rotated-agent-token".to_owned();
    let rotated = Recipe::new(
        request(AgentTarget::Pi),
        &adapter,
        Some(&references("first")),
        Arc::new(provider.clone()),
        &cancellation,
    )
    .unwrap();
    let response = dispatch_recipe(&rotated, sealed, Mode::Allow).unwrap();
    assert_eq!(response.output, AgentOutput::Text("echo <redacted>".to_owned()));
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    assert_eq!(transport.0.load(Ordering::SeqCst), 1);
}

#[test]
fn selected_backend_not_target_determines_agent_identity() {
    for target in [AgentTarget::Pi, AgentTarget::Clankers] {
        let provider = Provider::new();
        let cancellation = AtomicBool::new(false);
        let command = ProductionAgentAdapter::default();
        let network = ProductionAgentAdapter::new(network_config(target));
        let command_recipe = Recipe::new(
            request(target),
            &command,
            Some(&references("first")),
            Arc::new(provider.clone()),
            &cancellation,
        )
        .unwrap();
        let network_recipe = Recipe::new(
            request(target),
            &network,
            Some(&references("first")),
            Arc::new(provider.clone()),
            &cancellation,
        )
        .unwrap();
        assert_eq!(
            dispatch_recipe(&network_recipe, binding(command_recipe.payload(), 0), Mode::Allow),
            Err(Error::AdapterRejected)
        );
        let mut ambiguous = network_config(target);
        ambiguous.runtimes = AgentRuntimeConfig::default().runtimes;
        assert!(
            Recipe::new(
                request(target),
                &ProductionAgentAdapter::new(ambiguous),
                Some(&references("first")),
                Arc::new(provider.clone()),
                &cancellation
            )
            .is_err()
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    }
    let provider = Provider::new();
    let cancellation = AtomicBool::new(false);
    let adapter = ProductionAgentAdapter::new(network_config(AgentTarget::Pi));
    assert!(
        Recipe::new(
            request(AgentTarget::Animus),
            &adapter,
            Some(&references("first")),
            Arc::new(provider),
            &cancellation
        )
        .is_err()
    );
}

#[tokio::test]
async fn exact_executor_agent_path_publishes_only_redacted_outputs() {
    let provider = Provider::new();
    let transport = EchoTransport::default();
    let adapter =
        ProductionAgentAdapter::new_with_openai_transport(network_config(AgentTarget::Pi), Arc::new(transport.clone()));
    let mut workflow = Workflow::new("authorized-agent", WorkflowTimestamp::epoch());
    let node = workflow.add_agent_node("agent", AgentTarget::Pi, "say hello", AgentOutputMode::Text);
    workflow.set_node_credentials(node, references("first"));
    let settings = match workflow.node_config(node).unwrap() {
        crate::engine::workflow::NodeConfig::Agent(settings) => settings,
        _ => unreachable!(),
    };
    let mut public = request(AgentTarget::Pi);
    public.tool_policy.clone_from(&settings.tool_policy);
    let cancellation = AtomicBool::new(false);
    let sealed = {
        let recipe =
            Recipe::new(public, &adapter, Some(&references("first")), Arc::new(provider.clone()), &cancellation)
                .unwrap();
        binding(recipe.payload(), node.0)
    };
    let executor =
        Executor::new_with_agent_adapter(Box::new(adapter)).with_credential_provider(Box::new(provider.clone()));
    for mode in [Mode::Deny, Mode::Unavailable, Mode::Allow] {
        let step = executor.execute_authorized_node(&workflow, node, &HashMap::new(), &sealed, &shell(mode)).await;
        match (mode, step.status) {
            (Mode::Allow, NodeStatus::Completed { outputs, .. }) => {
                let text_port = workflow.graph.node(node).unwrap().output_ports[0].id;
                assert_eq!(outputs[&text_port].as_display_string(), "echo <redacted>");
                assert!(!format!("{outputs:?}").contains("sentinel-agent-token"));
            }
            (Mode::Deny | Mode::Unavailable, NodeStatus::Failed { .. }) => {
                assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
                assert_eq!(transport.0.load(Ordering::SeqCst), 0);
            }
            _ => panic!("unexpected exact Agent result"),
        }
    }
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    assert_eq!(transport.0.load(Ordering::SeqCst), 1);
}

#[test]
fn all_selected_network_agent_routes_execute_with_exact_authority() {
    for target in [AgentTarget::Pi, AgentTarget::Clankers, AgentTarget::Animus] {
        let provider = Provider::new();
        let transport = EchoTransport::default();
        let adapter =
            ProductionAgentAdapter::new_with_openai_transport(network_config(target), Arc::new(transport.clone()));
        let cancellation = AtomicBool::new(false);
        let recipe = Recipe::new(
            request(target),
            &adapter,
            Some(&references("first")),
            Arc::new(provider.clone()),
            &cancellation,
        )
        .unwrap();
        let response = dispatch_recipe(&recipe, binding(recipe.payload(), 0), Mode::Allow).unwrap();
        assert_eq!(response.output, AgentOutput::Text("echo <redacted>".to_owned()));
        assert!(!response.metadata.to_string().contains("sentinel-agent-token"));
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        assert_eq!(transport.0.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn selected_agent_projection_and_bounds_are_authority_bound() {
    let provider = Provider::new();
    let cancellation = AtomicBool::new(false);
    let adapter = ProductionAgentAdapter::new(network_config(AgentTarget::Pi));
    let original = Recipe::new(
        request(AgentTarget::Pi),
        &adapter,
        Some(&references("first")),
        Arc::new(provider.clone()),
        &cancellation,
    )
    .unwrap();
    let sealed = binding(original.payload(), 0);
    let mut global_bound = network_config(AgentTarget::Pi);
    global_bound.max_output_bytes += 1;
    let mut network_bound = network_config(AgentTarget::Pi);
    network_bound.openai_compatible.get_mut("pi").unwrap().max_response_bytes += 1;
    let mut local_bound = network_config(AgentTarget::Pi);
    local_bound.local_process.limits.stdout_max_bytes += 1;
    let mut projection = network_config(AgentTarget::Pi);
    projection.structured_input_projection.pi = Some(crate::agent_prompt::StructuredInputProjection::JsonCompact);
    for config in [global_bound, network_bound, local_bound, projection] {
        let changed = ProductionAgentAdapter::new(config);
        let recipe = Recipe::new(
            request(AgentTarget::Pi),
            &changed,
            Some(&references("first")),
            Arc::new(provider.clone()),
            &cancellation,
        )
        .unwrap();
        assert_eq!(dispatch_recipe(&recipe, sealed.clone(), Mode::Allow), Err(Error::AdapterRejected));
    }
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn binary_and_structured_echoes_redact_overlapping_values_without_losing_public_bytes() {
    let context = ExecutionContext::new(HashMap::from([
        ("SHORT".to_owned(), "abc".to_owned()),
        ("LONG".to_owned(), "abcde".to_owned()),
    ]));
    let mut bytes = b"\xff:abcde:abc:\x00".to_vec();
    redact_bytes(&mut bytes, &context);
    assert_eq!(bytes, b"\xff:<redacted>:<redacted>:\x00");
    let mut json = serde_json::json!({"abcde": ["abcde", "abc", "public"]});
    redact_json(&mut json, &context);
    assert_eq!(json, serde_json::json!({"<redacted>": ["<redacted>", "<redacted>", "public"]}));
}

#[tokio::test]
async fn product_agent_session_requires_authority_before_provider_or_transport() {
    for mode in [Mode::Deny, Mode::Unavailable, Mode::Allow] {
        let provider = Provider::new();
        let transport = EchoTransport::default();
        let adapter = ProductionAgentAdapter::new_with_openai_transport(
            network_config(AgentTarget::Pi),
            Arc::new(transport.clone()),
        );
        let mut workflow = Workflow::new("product-agent", WorkflowTimestamp::epoch());
        let node = workflow.add_agent_node("agent", AgentTarget::Pi, "say hello", AgentOutputMode::Text);
        workflow.set_node_credentials(node, references("first"));
        let mut executor =
            Executor::new_with_agent_adapter(Box::new(adapter)).with_credential_provider(Box::new(provider.clone()));
        let context = crate::engine::effect::execution::RunContext::new(
            authority::RunIdentity::new("test-run").unwrap(),
            authority::AuthorityIdentity::new("test-operator").unwrap(),
            authority::CollaborationScopeBinding::Unscoped,
            Arc::new(shell(mode)),
        );
        let (_, steps) = executor.run_authorized(&workflow, &HashMap::new(), &context).await;
        let step = steps.iter().find(|step| step.node_id == node).expect("Agent step is observed");
        if matches!(mode, Mode::Allow) {
            let NodeStatus::Completed { outputs, .. } = &step.status else {
                panic!("authorized Agent must complete: {:?}", step.status);
            };
            let port = workflow.graph.node(node).unwrap().output_ports[0].id;
            assert_eq!(outputs[&port].as_display_string(), "echo <redacted>");
            assert!(!format!("{steps:?}").contains("sentinel-agent-token"));
            assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
            assert_eq!(transport.0.load(Ordering::SeqCst), 1);
        } else {
            assert!(matches!(step.status, NodeStatus::Failed { .. }));
            assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
            assert_eq!(transport.0.load(Ordering::SeqCst), 0);
        }
    }
}
