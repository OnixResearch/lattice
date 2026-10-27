//! Aggregate lifecycle acceptance, independent of transient TUI state.

use std::collections::BTreeMap;

use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentTarget;
use lattice::control_flow::ConditionBranch;
use lattice::control_flow::ConditionNodeConfig;
use lattice::control_flow::ConditionPredicate;
use lattice::control_flow::MergeNodeConfig;
use lattice::control_flow::MergePolicy;
use lattice::control_flow::SubworkflowNodeConfig;
use lattice::credentials::NodeCredentialRefs;
use lattice::credentials::SecretRef;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::NodeRunnerSettings;
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowTimestamp;
use lattice::file_node::FileOperationKind;
use lattice::operation;
use lattice::workflow::gate;
use lattice::workflow_format::load_workflow_nickel_file;
use lattice::workflow_format::workflow_to_nickel;
use lattice::workflow_format::workflow_to_portable;
use lattice::workflow_validation::WorkflowDiagnosticCode;
use lattice::workflow_validation::has_fatal_diagnostics;
use lattice::workflow_validation::validate_workflow;

fn condition_branch(label: &str) -> ConditionBranch {
    ConditionBranch {
        label: label.to_string(),
        predicate: ConditionPredicate::Always,
    }
}

fn approval_config() -> gate::ApprovalNodeConfig {
    gate::ApprovalNodeConfig {
        gate_id: gate::GateId::try_from_text("aggregate-review").expect("gate id"),
        safe_summary: "Review the aggregate".to_string(),
        selected_input_ports: vec!["input".to_string()],
        required_artifacts: Vec::new(),
        authority: gate::ApprovalAuthorityPolicy {
            profile: gate::AuthorityProfile::LocalOperator,
            local_actors: vec![gate::ActorId::try_from_text("reviewer").expect("actor")],
            accepted_abilities: vec![gate::DecisionAbility::Approve, gate::DecisionAbility::Reject],
            prohibit_self_approval: true,
            delegated_policy_reference: None,
        },
        expiry: gate::ApprovalExpiryPolicy {
            decision_ttl_millis: 60_000,
            resume_ttl_millis: 60_000,
            approval_must_be_fresh_at_resume: true,
            clock_source: gate::ClockSourceClass::UtcWallClock,
        },
        reason: gate::ApprovalReasonPolicy {
            required: false,
            identity_bound: true,
            max_bytes: 128,
        },
        resume: gate::ApprovalResumeSchema {
            values: Vec::new(),
            credential_bindings: Vec::new(),
        },
        terminal: gate::ApprovalTerminalPolicy {
            on_rejection: gate::TerminalOutcome::FailWorkflow,
            on_expiry: gate::TerminalOutcome::CancelWorkflow,
        },
        required_non_claims: gate::REQUIRED_APPROVAL_NON_CLAIMS.map(str::to_owned).to_vec(),
    }
}

// Planning consumes explicit metadata only: this fixture never resolves a secret,
// loads executable bytes, or invokes the operation.
fn operation_plan() -> operation::InvocationPlan {
    let bounds = operation::Bounds {
        timeout_seconds: 1,
        input_bytes: 4096,
        output_bytes: 16_384,
        artifact_bytes: 1024,
    };
    let executor_identity =
        operation::identity::hash_bytes("aggregate.lifecycle.fixture", b"read-only executable metadata");
    let method = |name: &str| operation::model::Method {
        name: name.to_string(),
        description: "Read repository metadata".to_string(),
        arguments: operation::model::RecordContract {
            fields: Vec::new(),
            deny_unknown_fields: true,
        },
        output: operation::model::RecordContract {
            fields: vec![operation::model::FieldContract {
                name: "valid".to_string(),
                value_type: operation::model::ValueType::Boolean,
                required: true,
                max_bytes: 128,
            }],
            deny_unknown_fields: false,
        },
        preflight: vec![
            operation::model::PreflightCheck::RuntimeAvailable,
            operation::model::PreflightCheck::ExecutableAvailable,
            operation::model::PreflightCheck::RepositoryRootAdmitted,
            operation::model::PreflightCheck::CredentialReferencesValid,
        ],
        capabilities: vec![operation::CapabilityClass::ProcessRead],
        side_effect: operation::model::SideEffectClass::ReadOnly,
    };
    let runtime = operation::model::RuntimeRequirement {
        kind: operation::runtime::CAIRN_RUNTIME_KIND.to_string(),
        key: operation::runtime::CAIRN_RUNTIME_KEY.to_string(),
        executor_identity: executor_identity.clone(),
    };
    let mut definition = operation::OperationType {
        id: "cairn.repository".to_string(),
        schema_version: operation::model::TYPE_SCHEMA_VERSION,
        revision: executor_identity.clone(),
        description: "Aggregate deletion operation fixture".to_string(),
        tags: vec!["read-only".to_string()],
        methods: vec![
            method(operation::runtime::CAIRN_CHANGE_LIST_METHOD),
            method(operation::runtime::CAIRN_VALIDATE_METHOD),
        ],
        runtime,
        execution_requirements: operation::ExecutionRequirements::cairn_read_only(),
        capabilities: vec![operation::CapabilityClass::ProcessRead],
        credential_bindings: Vec::new(),
        bounds,
        required_non_claims: operation::model::REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect(),
    };
    definition.revision = operation::identity::type_revision(&definition).expect("type revision");
    let catalog = operation::catalog::validate(operation::Catalog {
        schema: operation::model::CATALOG_SCHEMA.to_string(),
        types: vec![definition.clone()],
    })
    .expect("validated fixture catalog");
    let mut instances = operation::instance::validate(
        operation::InstanceFile {
            schema: operation::model::INSTANCE_SCHEMA.to_string(),
            instances: vec![operation::OperationInstance {
                id: "aggregate-repository".to_string(),
                display_alias: "Aggregate repository".to_string(),
                type_id: definition.id.clone(),
                type_revision: definition.revision.clone(),
                target: BTreeMap::from([(
                    operation::runtime::REPOSITORY_ROOT_TARGET.to_string(),
                    serde_json::json!("/aggregate/repository"),
                )]),
                tags: BTreeMap::new(),
                method_defaults: BTreeMap::new(),
                credentials: BTreeMap::new(),
            }],
        },
        &catalog,
    )
    .expect("validated fixture instance");
    let instance = instances.remove("aggregate-repository").expect("instance");
    operation::planning::plan(operation::PlanRequest {
        catalog: &catalog,
        instance: &instance,
        method: operation::runtime::CAIRN_CHANGE_LIST_METHOD,
        arguments: BTreeMap::new(),
        expected_catalog_identity: &catalog.identity,
        expected_instance_identity: &instance.identity,
        runtime: operation::RuntimeFact {
            kind: definition.runtime.kind,
            key: definition.runtime.key,
            executor_identity,
            executable_path: "/aggregate/bin/cairn".to_string(),
            available: true,
            maximum_bounds: bounds,
        },
        policy: operation::CapabilityPolicy {
            allowed: vec![operation::CapabilityClass::ProcessRead],
            allow_mutation: false,
        },
        preflight: operation::PreflightFacts {
            executable_available: true,
            repository_root_admitted: true,
            credential_references_valid: true,
        },
    })
    .expect("validated operation plan")
}

fn config_variants() -> Vec<NodeConfig> {
    vec![
        NodeConfig::shell("printf deleted"),
        NodeConfig::http("POST", "https://example.com/aggregate"),
        NodeConfig::file(FileOperationKind::Read, "/tmp/aggregate.txt"),
        NodeConfig::image_input("/tmp/aggregate.png"),
        NodeConfig::agent(AgentTarget::Pi, "Review the aggregate", AgentOutputMode::Text),
        NodeConfig::operation(operation_plan()),
        NodeConfig::condition(ConditionNodeConfig {
            branches: vec![condition_branch("output")],
        }),
        NodeConfig::merge(MergeNodeConfig {
            policy: MergePolicy::All,
            required_inputs: vec!["input".to_string()],
        }),
        NodeConfig::subworkflow(SubworkflowNodeConfig {
            workflow_name: "child".to_string(),
            input_mappings: Vec::new(),
            output_mappings: Vec::new(),
        }),
        NodeConfig::approval(approval_config()),
    ]
}

fn credentials(token: &str) -> NodeCredentialRefs {
    let mut refs = NodeCredentialRefs::default();
    refs.env.insert("TOKEN".to_string(), SecretRef::new("env", token, None));
    refs
}

fn snapshot(workflow: &Workflow) -> serde_json::Value {
    serde_json::to_value(workflow).expect("workflow snapshot")
}

#[test]
fn operation_runner_edit_cannot_replace_the_accepted_plan_runtime() {
    let mut workflow = Workflow::new("accepted operation runtime", WorkflowTimestamp::epoch());
    let plan = operation_plan();
    let accepted_binding = format!("{}|{}", plan.runtime.kind, plan.runtime.key);
    let owner = workflow.add_operation_node("operation", plan);
    lattice::tui::authoring::workflow::apply(&mut workflow, Some(owner.0), "runner.binding", &accepted_binding)
        .expect("the accepted operation runtime remains authorable");
    let before = snapshot(&workflow);
    assert!(
        lattice::tui::authoring::workflow::apply(&mut workflow, Some(owner.0), "runner.binding", "shell|default",)
            .is_err()
    );
    assert_eq!(snapshot(&workflow), before);
}

// r[verify lattice.workflow_validation.aggregate_mutation.scenario.delete]
// r[verify lattice.tui_ux.aggregate_node_deletion.scenario.delete]
#[test]
fn every_current_config_deletion_preserves_unrelated_aggregate_and_round_trips() {
    for config in config_variants() {
        let kind = config.kind_label();
        let mut workflow = Workflow::new("aggregate-round-trip", WorkflowTimestamp::epoch());
        workflow.meta.description = format!("Deleting {kind} preserves the other node");
        workflow.triggers.push(lattice::triggers::TriggerDefinition {
            id: "retained-manual".to_string(),
            workflow_name: workflow.meta.name.clone(),
            enabled: true,
            kind: lattice::triggers::TriggerKind::Manual,
            input_bindings: Vec::new(),
            overlap_policy: lattice::scheduler::OverlapPolicy::Forbid,
            idempotency_policy: lattice::triggers::IdempotencyPolicy::None,
        });
        let source = workflow.add_shell_node("retained source", "printf retained");
        let target = workflow.add_shell_node("retained target", "cat");
        workflow.graph.node_mut(target).expect("target").input_ports[0].type_tag = "any".to_string();
        let owner = workflow.try_add_configured_node(kind, config).expect("checked configured creation");
        workflow.set_node_credentials(owner, credentials("DELETED_TOKEN"));
        workflow.set_node_runner_settings(owner, NodeRunnerSettings::new("shell", "deleted-binding"));
        workflow.set_node_credentials(source, credentials("RETAINED_TOKEN"));
        workflow.set_node_runner_settings(source, NodeRunnerSettings::new("shell", "default"));
        let source_output = workflow.graph.node(source).expect("source").output_ports[0].id;
        let target_input = workflow.graph.node(target).expect("target").input_ports[0].id;
        let owner_node = workflow.graph.node(owner).expect("owner");
        let input = owner_node.input_ports.first().map(|port| port.id);
        let output = owner_node.output_ports.first().map(|port| port.id);
        let owned_ports = owner_node
            .input_ports
            .iter()
            .chain(&owner_node.output_ports)
            .map(|port| port.id)
            .collect::<Vec<_>>();
        if let Some(input) = input {
            workflow.graph.add_edge(source_output, input).expect("incoming edge");
        }
        if let Some(output) = output {
            workflow.graph.add_edge(output, target_input).expect("outgoing edge");
        }
        workflow.graph.add_edge(source_output, target_input).expect("unrelated edge");
        let retained_source = workflow.graph.node(source).cloned().expect("source");
        let retained_target = workflow.graph.node(target).cloned().expect("target");
        let retained_config = workflow.node_config(source).cloned().expect("source config");
        let retained_credentials = workflow.node_credentials(source).cloned().expect("source credentials");
        let retained_runner = workflow.node_runner_settings(source).cloned().expect("source runner");
        let metadata = serde_json::to_value(&workflow.meta).expect("metadata");
        let triggers = serde_json::to_value(&workflow.triggers).expect("triggers");

        workflow.remove_node(owner).expect("aggregate deletion");

        assert!(workflow.graph.node(owner).is_none(), "{kind}");
        assert!(owned_ports.iter().all(|port| workflow.graph.port(*port).is_none()), "{kind}");
        assert!(
            workflow.node_config(owner).is_none()
                && workflow.node_credentials(owner).is_none()
                && workflow.node_runner_settings(owner).is_none(),
            "{kind}"
        );
        assert_eq!(workflow.graph.node(source), Some(&retained_source), "{kind}");
        assert_eq!(workflow.graph.node(target), Some(&retained_target), "{kind}");
        assert_eq!(workflow.node_config(source), Some(&retained_config), "{kind}");
        assert_eq!(workflow.node_credentials(source), Some(&retained_credentials), "{kind}");
        assert_eq!(workflow.node_runner_settings(source), Some(&retained_runner), "{kind}");
        assert_eq!(
            workflow.graph.edges(),
            &[lattice::engine::workflow::Edge {
                source: source_output,
                target: target_input
            }],
            "{kind}"
        );
        assert_eq!(serde_json::to_value(&workflow.meta).expect("metadata"), metadata, "{kind}");
        assert_eq!(serde_json::to_value(&workflow.triggers).expect("triggers"), triggers, "{kind}");
        let diagnostics = validate_workflow(&workflow);
        assert!(!has_fatal_diagnostics(&diagnostics), "{kind}: {diagnostics:?}");

        let directory = tempfile::tempdir().expect("aggregate temporary directory");
        let store = lattice::store::Store::new(directory.path().join("aggregate.redb")).expect("store");
        store.save_workflow(&workflow).expect("save deleted aggregate");
        let loaded = store.load_workflow(&workflow.meta.name).expect("load aggregate").expect("saved workflow");
        assert_eq!(snapshot(&loaded), snapshot(&workflow), "{kind}: store persistence");
        let path = directory.path().join("aggregate.ncl");
        std::fs::write(&path, workflow_to_nickel(&loaded)).expect("write Nickel");
        let imported = load_workflow_nickel_file(&path).expect("import clean aggregate");
        assert_eq!(workflow_to_portable(&imported), workflow_to_portable(&workflow), "{kind}: Nickel meaning");
        assert!(!has_fatal_diagnostics(&validate_workflow(&imported)), "{kind}: imported aggregate");
    }
}

#[test]
fn checked_configured_creation_exhaustion_preserves_every_aggregate_field() {
    for config in config_variants() {
        for allocator in ["next_node_id", "next_port_id"] {
            let mut workflow = Workflow::new("exhausted configured creation", WorkflowTimestamp::epoch());
            let retained = workflow.add_shell_node("retained", "printf retained");
            workflow.set_node_credentials(retained, credentials("RETAINED_TOKEN"));
            workflow.set_node_runner_settings(retained, NodeRunnerSettings::new("shell", "default"));
            let mut graph = serde_json::to_value(&workflow.graph).expect("graph fixture");
            graph[allocator] = serde_json::json!(u64::MAX);
            workflow.graph = serde_json::from_value(graph).expect("exhausted allocator fixture");
            let before = snapshot(&workflow);
            let expected = if allocator == "next_node_id" {
                lattice::engine::workflow::GraphError::NodeIdExhausted
            } else {
                lattice::engine::workflow::GraphError::PortIdExhausted
            };
            assert_eq!(
                workflow.try_add_configured_node("rejected", config.clone()),
                Err(lattice::engine::workflow::MutationError::Graph(expected)),
                "{}: {allocator}",
                config.kind_label()
            );
            assert_eq!(snapshot(&workflow), before, "{}: {allocator}", config.kind_label());
        }
    }
}

#[test]
fn checked_configured_creation_rejects_duplicate_dynamic_ports_atomically() {
    let mut workflow = Workflow::new("duplicate configured ports", WorkflowTimestamp::epoch());
    workflow.add_shell_node("retained", "true");
    let before = snapshot(&workflow);
    let config = NodeConfig::condition(ConditionNodeConfig {
        branches: vec![condition_branch("duplicate"), condition_branch("duplicate")],
    });
    assert_eq!(
        workflow.try_add_configured_node("rejected", config),
        Err(lattice::engine::workflow::MutationError::Graph(
            lattice::engine::workflow::GraphError::DuplicatePortLabel("duplicate".to_string()),
        ))
    );
    assert_eq!(snapshot(&workflow), before);
}

// r[verify lattice.workflow_validation.aggregate_mutation.scenario.reject_orphan]
#[test]
fn runner_rows_without_owners_are_fatal_and_cannot_be_partially_deleted() {
    let mut workflow = Workflow::new("orphan runner", WorkflowTimestamp::epoch());
    let owner = workflow.add_shell_node("owner", "true");
    workflow.runner.insert(u64::MAX, NodeRunnerSettings::new("shell", "shell"));
    let before = snapshot(&workflow);
    let diagnostics = validate_workflow(&workflow);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::RunnerSettingsWithoutNode
                && diagnostic.node_id == Some(NodeId(u64::MAX)))
    );
    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(workflow.remove_node(owner).is_err());
    assert_eq!(snapshot(&workflow), before);
}

#[test]
fn duplicate_condition_outputs_are_fatal_and_reconciliation_cannot_drop_one_silently() {
    let mut workflow = Workflow::new("duplicate condition output", WorkflowTimestamp::epoch());
    let condition = workflow.add_condition_node("route", ConditionNodeConfig {
        branches: vec![condition_branch("keep")],
    });
    let duplicate = workflow.graph.node(condition).expect("condition").output_ports[0].clone();
    workflow.graph.node_mut(condition).expect("condition").output_ports.push(duplicate);
    let before = snapshot(&workflow);
    let diagnostics = validate_workflow(&workflow);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidConditionConfig
                && diagnostic.node_id == Some(condition))
    );
    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(workflow.replace_condition_branches(condition, vec![condition_branch("keep")]).is_err());
    assert_eq!(snapshot(&workflow), before);
}

#[test]
fn deleting_expression_producer_or_referenced_branch_rejects_without_orphaning_consumer() {
    let mut workflow = Workflow::new("expression lifecycle", WorkflowTimestamp::epoch());
    let condition = workflow.add_condition_node("producer", ConditionNodeConfig {
        branches: vec![condition_branch("keep"), condition_branch("referenced")],
    });
    let output = workflow.graph.node(condition).expect("producer").output_ports[1].id;
    let consumer =
        workflow.add_shell_node("consumer", format!("printf '{{{{node:{}:port:{}}}}}'", condition.0, output.0));
    let before = snapshot(&workflow);
    assert!(workflow.remove_node(condition).is_err());
    assert_eq!(snapshot(&workflow), before);
    assert!(workflow.replace_condition_branches(condition, vec![condition_branch("keep")]).is_err());
    assert_eq!(snapshot(&workflow), before);
    workflow
        .replace_condition_branches(condition, vec![condition_branch("referenced"), condition_branch("keep")])
        .expect("referenced identity survives reorder");
    assert!(workflow.graph.port(output).is_some());
    workflow.remove_node(consumer).expect("remove reference owner first");
    workflow.remove_node(condition).expect("unreferenced producer may be removed");
    assert!(workflow.configs.is_empty());
}

#[test]
fn reconciled_condition_typed_ports_and_edges_survive_store_and_nickel_round_trip() {
    let mut workflow = Workflow::new("condition-round-trip", WorkflowTimestamp::epoch());
    let condition = workflow.add_condition_node("route", ConditionNodeConfig {
        branches: vec![condition_branch("keep"), condition_branch("remove")],
    });
    let target = workflow.add_shell_node("target", "cat");
    workflow.graph.node_mut(condition).expect("condition").output_ports[0].type_tag = "string".to_string();
    let keep = workflow.graph.node(condition).expect("condition").output_ports[0].id;
    let input = workflow.graph.node(target).expect("target").input_ports[0].id;
    workflow.graph.add_edge(keep, input).expect("typed retained edge");
    workflow
        .replace_condition_branches(condition, vec![condition_branch("new"), condition_branch("keep")])
        .expect("reconcile");
    let directory = tempfile::tempdir().expect("directory");
    let store = lattice::store::Store::new(directory.path().join("condition.redb")).expect("store");
    store.save_workflow(&workflow).expect("save reconciled condition");
    let loaded = store.load_workflow(&workflow.meta.name).expect("load").expect("workflow");
    assert_eq!(snapshot(&loaded), snapshot(&workflow));
    let path = directory.path().join("condition.ncl");
    std::fs::write(&path, workflow_to_nickel(&loaded)).expect("Nickel");
    let imported = load_workflow_nickel_file(&path).expect("import reconciled topology");
    assert_eq!(workflow_to_portable(&imported), workflow_to_portable(&workflow));
    assert_eq!(imported.graph.port(keep).expect("stable port").type_tag, "string");
    assert_eq!(imported.graph.edges(), workflow.graph.edges());
    let diagnostics = validate_workflow(&imported);
    assert!(!has_fatal_diagnostics(&diagnostics), "{diagnostics:?}");
}
