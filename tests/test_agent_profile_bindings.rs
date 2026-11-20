// r[verify lattice.agent_runtime.configured_bindings.scenario.compatibility]
// r[verify lattice.agent_runtime.configured_bindings.scenario.boundary]

use std::sync::Arc;

use lattice::agent_node::Adapter;
use lattice::agent_node::CompatibilitySupport;
use lattice::agent_node::ConfiguredBinding;
use lattice::agent_node::ExecutionContext;
use lattice::agent_node::OutputMode;
use lattice::agent_node::ProductionAdapter;
use lattice::agent_node::Request;
use lattice::agent_node::RuntimeOptions;
use lattice::agent_node::Target;

fn direct_config(target: &str) -> RuntimeOptions {
    serde_json::from_value(serde_json::json!({
        "runtimes": {},
        "openai_compatible": {
            (target): {"endpoint": "http://fixture.invalid/v1/chat/completions", "model": "fixture-model"}
        }
    }))
    .expect("configured fixture")
}

fn request(target: Target) -> Request {
    Request {
        target,
        prompt: "private dynamic prompt".to_string(),
        output_mode: OutputMode::Text,
        model_hint: None,
        tool_policy: String::new(),
        context_refs: Vec::new(),
        cwd: None,
        session: None,
        text_inputs: Vec::new(),
        image_inputs: Vec::new(),
        json_inputs: Vec::new(),
    }
}

#[test]
fn configured_profiles_are_bounded_material_free_and_never_capacity() {
    let mut config = RuntimeOptions::default();
    config.runtimes.get_mut("pi").expect("Pi").command = "/private/unqualified-worker".to_string();
    let profiles = config.configured_profiles().expect("declared command profiles");
    assert_eq!(profiles.len(), 2);
    for profile in &profiles {
        assert!(profile.binding.validate());
        assert_eq!(profile.model_service_bindings, CompatibilitySupport::Unsupported);
        assert_eq!(profile.compute_reservations, CompatibilitySupport::Unsupported);
        assert_eq!(profile.qualified_worker_telemetry, CompatibilitySupport::Unsupported);
        assert_eq!(profile.sessions, CompatibilitySupport::Unknown);
    }
    let inventory = serde_json::to_string(&profiles).expect("safe inventory");
    assert!(!inventory.contains("/private/"));
    assert!(!inventory.contains("prompt"));
    assert!(!inventory.contains("fixture.invalid"));
    assert!(config.configured_profile(Target::Animus).is_err());
}

#[test]
fn direct_adapter_projection_retains_limits_and_rejects_invalid_or_ambiguous_configuration() {
    for (label, target) in [
        ("pi", Target::Pi),
        ("clankers", Target::Clankers),
        ("animus", Target::Animus),
    ] {
        let mut config = direct_config(label);
        let profile = config.configured_profile(target).expect("valid declared direct route");
        assert_eq!(profile.output_modes, vec![OutputMode::Text, OutputMode::Json]);
        assert_eq!(profile.sessions, CompatibilitySupport::Unsupported);
        assert_eq!(profile.image_inputs, CompatibilitySupport::Unsupported);
        let inventory = serde_json::to_string(&profile).expect("metadata");
        assert!(!inventory.contains("fixture.invalid"));
        assert!(!inventory.contains("fixture-model"));
        config.openai_compatible.get_mut(label).expect("route").timeout_secs = 0;
        assert!(config.configured_profiles().is_err());
    }
    let mut config = direct_config("pi");
    config
        .runtimes
        .insert("pi".to_string(), RuntimeOptions::default().runtimes.remove("pi").expect("Pi"));
    assert!(config.configured_profiles().is_err());
    let mut config = RuntimeOptions::default();
    config.runtimes.get_mut("pi").expect("Pi").command.clear();
    assert!(config.configured_profiles().is_err());
}

struct FixtureTransport;

impl lattice::openai::Transport for FixtureTransport {
    fn execute(
        &self,
        _plan: &lattice::openai::ChatPlan,
        _bearer: Option<&str>,
    ) -> Result<lattice::openai::HttpResponse, lattice::agent_node::Error> {
        Ok(lattice::openai::HttpResponse {
            status: 200,
            body: br#"{"model":"fixture-model","choices":[{"message":{"content":"done"}}],"usage":{"total_tokens":7}}"#
                .to_vec(),
        })
    }
}

#[test]
fn production_metadata_reuses_admitted_identity_without_dynamic_input_or_service_claims() {
    let config = direct_config("pi");
    let profile = config.configured_profile(Target::Pi).expect("profile");
    let adapter = ProductionAdapter::new_with_openai_transport(config.clone(), Arc::new(FixtureTransport));
    let mut request = request(Target::Pi);
    let revision = adapter.execution_identity(&request).expect("admitted configuration");
    assert_eq!(profile.binding.revision, revision);
    request.prompt = "different dynamic prompt".to_string();
    request.cwd = Some("/private/workspace".to_string());
    assert_eq!(adapter.execution_identity(&request).expect("same configuration"), revision);
    let response = adapter.execute(&request, &ExecutionContext::default()).expect("injected response");
    let binding: ConfiguredBinding =
        serde_json::from_value(response.metadata["configured_runtime"].clone()).expect("host binding");
    assert_eq!(binding, profile.binding);
    assert!(response.metadata.get("compute_lease").is_none());
    assert!(response.metadata.get("model_service").is_none());
    let mut changed = config;
    changed.openai_compatible.get_mut("pi").expect("route").model = "changed-model".to_string();
    assert_ne!(changed.configured_profile(Target::Pi).expect("changed profile").binding, binding);
}

// r[verify lattice.execution_binding.profile_facts.scenario.compatibility]
#[test]
fn configured_routes_project_worker_facts_without_compute_service_or_qualification() {
    let unavailable = lattice::execution_binding::core::projection::SupportLevel::Unavailable;
    let unknown = lattice::execution_binding::core::projection::SupportLevel::Unknown;
    let mut config = RuntimeOptions::default();
    config.runtimes.get_mut("pi").expect("Pi").command = "/private/newly-advertised-worker".to_string();
    let profiles = config.configured_profiles().expect("command profiles");
    let facts = config.worker_profile_facts().expect("command facts");
    assert_eq!(facts.len(), profiles.len());
    for (fact, profile) in facts.iter().zip(&profiles) {
        assert_eq!(fact.revision, profile.binding.revision);
        assert_eq!(fact.compute.support, unavailable);
        assert_eq!(fact.model_routes.support, unavailable);
        assert_eq!(fact.executable_identity.support, unknown);
        assert_eq!(fact.sessions.support, unknown);
        assert_eq!(fact.adapter.value.as_deref(), Some("command"));
    }
    let text = serde_json::to_string(&facts).expect("facts json");
    assert!(!text.contains("\"support\":\"qualified\""), "configuration never qualifies an executable");
    assert!(!text.contains("/private/"));
    for (label, target) in [
        ("pi", Target::Pi),
        ("clankers", Target::Clankers),
        ("animus", Target::Animus),
    ] {
        let config = direct_config(label);
        let facts = config.worker_profile_facts().expect("direct facts");
        assert_eq!(facts.len(), 1);
        let fact = &facts[0];
        assert_eq!(fact.revision, config.configured_profile(target).expect("profile").binding.revision);
        assert_eq!(fact.sessions.support, unavailable);
        assert_eq!(fact.outputs.value.as_deref(), Some("text,json"));
        assert_eq!(fact.inputs.value.as_deref(), Some("text-prompt-without-images"));
        assert_eq!((fact.compute.support, fact.model_routes.support), (unavailable, unavailable));
        let text = serde_json::to_string(fact).expect("direct json");
        assert!(!text.contains("fixture.invalid"));
        assert!(!text.contains("fixture-model"));
    }
}

// r[verify lattice.worker_inventory.definition]
// r[verify lattice.worker_inventory.schema]
#[test]
fn configured_routes_are_owner_default_definitions_with_undeclared_capability() {
    use lattice::execution_binding::core::projection::SupportLevel;
    use lattice::execution_binding::core::projection::inventory::PROFILE_FACTS_SCHEMA;
    use lattice::execution_binding::core::projection::lineage::DefinitionBasis;
    let facts = direct_config("pi").worker_profile_facts().expect("direct facts");
    let fact = &facts[0];
    assert_eq!(fact.schema, PROFILE_FACTS_SCHEMA);
    assert_eq!(PROFILE_FACTS_SCHEMA, "lattice.worker-profile-facts.v2");
    assert_eq!((fact.definition.name.as_str(), fact.definition.basis), ("pi", DefinitionBasis::OwnerDefault));
    for names in [&fact.task_families, &fact.tools] {
        assert_eq!((names.support, names.value.as_ref()), (SupportLevel::Unknown, None));
    }
    let mut changed = direct_config("pi");
    changed.openai_compatible.get_mut("pi").expect("route").model = "changed-model".to_string();
    let changed = changed.worker_profile_facts().expect("changed facts");
    assert_ne!(changed[0].revision, fact.revision, "a changed route is a new revision");
    assert_eq!(changed[0].definition, fact.definition, "of the same definition");
    let clankers = direct_config("clankers").worker_profile_facts().expect("clankers facts");
    assert_ne!(clankers[0].definition.identity, fact.definition.identity, "targets are distinct definitions");
}
