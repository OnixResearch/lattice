use super::*;
use crate::engine::workflow::WorkflowTimestamp;

fn workflow() -> Workflow {
    Workflow::new("typed-authoring", WorkflowTimestamp::epoch())
}
fn edit(workflow: &mut Workflow, node: NodeId, id: &str, value: &str) {
    crate::tui::authoring::test_support::edit(workflow, Some(node.0), id, value)
        .unwrap_or_else(|error| panic!("{id}={value}: {error}"));
}
fn rejected(workflow: &mut Workflow, node: NodeId, id: &str, value: &str) {
    let before = serde_json::to_value(&*workflow).unwrap();
    assert!(
        crate::tui::authoring::test_support::edit(workflow, Some(node.0), id, value).is_err(),
        "accepted {id}={value}"
    );
    assert_eq!(serde_json::to_value(&*workflow).unwrap(), before);
}
fn rejected_pure(workflow: &mut Workflow, node: NodeId, id: &str, value: &str) {
    let before = serde_json::to_value(&*workflow).unwrap();
    assert!(apply(workflow, node.0, id, value).is_err(), "accepted {id}={value}");
    assert_eq!(serde_json::to_value(&*workflow).unwrap(), before);
}

#[test]
fn shell_typed_fields_map_rows_reset_and_unrelated_state() {
    let mut w = workflow();
    let node = w.add_shell_node("shell", "old");
    let other = w.add_shell_node("untouched", "preserve");
    let untouched = serde_json::to_value(w.configs.get(&other.0)).unwrap();
    for (id, value) in [
        ("command", "printf ok"),
        ("cwd", "/tmp"),
        ("env.add", "A=one=two"),
        ("env.0.key", "B"),
        ("env.0.value", ""),
        ("timeout_secs", "18446744073709551615"),
        ("pipe_stdin", "false"),
    ] {
        edit(&mut w, node, &format!("config.shell.{id}"), value);
    }
    match &w.configs[&node.0] {
        NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            pipe_stdin,
        } => {
            assert_eq!(command, "printf ok");
            assert_eq!(cwd.as_deref(), Some("/tmp"));
            assert_eq!(env, &HashMap::from([("B".into(), "".into())]));
            assert_eq!(*timeout_secs, u64::MAX);
            assert!(!pipe_stdin);
        }
        _ => panic!("wrong config"),
    }
    for (id, value) in [
        ("command", ""),
        ("cwd", "../escape"),
        ("timeout_secs", "18446744073709551616"),
        ("timeout_secs", "-1"),
        ("timeout_secs", "+1"),
        ("pipe_stdin", "yes"),
        ("env.add", "B=duplicate"),
        ("env.add", "missing-separator"),
        ("env.99.value", "stale"),
        ("unknown", "value"),
    ] {
        rejected(&mut w, node, &format!("config.shell.{id}"), value);
    }
    edit(&mut w, node, "config.shell.env.0.key", "");
    edit(&mut w, node, "config.shell.cwd", "");
    edit(&mut w, node, "config.shell.timeout_secs", "0");
    match &w.configs[&node.0] {
        NodeConfig::Shell {
            cwd, env, timeout_secs, ..
        } => {
            assert!(cwd.is_none());
            assert!(env.is_empty());
            assert_eq!(*timeout_secs, 0);
        }
        _ => unreachable!(),
    }
    assert_eq!(serde_json::to_value(w.configs.get(&other.0)).unwrap(), untouched);
}

#[test]
fn http_methods_url_headers_optional_body_and_timeout_are_typed() {
    let mut w = workflow();
    let node = w.add_http_node("http", "GET", "https://example.test");
    for method in ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"] {
        edit(&mut w, node, "config.http.method", method);
        assert!(matches!(&w.configs[&node.0], NodeConfig::Http { method: actual, .. } if actual == method));
    }
    for (id, value) in [
        ("url", "https://example.test/new"),
        ("body", "payload"),
        ("headers.add", "X-Test=a=b"),
        ("headers.0.key", "X-Other"),
        ("headers.0.value", "changed"),
        ("timeout_secs", "0"),
    ] {
        edit(&mut w, node, &format!("config.http.{id}"), value);
    }
    match &w.configs[&node.0] {
        NodeConfig::Http {
            url,
            body,
            headers,
            timeout_secs,
            ..
        } => {
            assert_eq!(url, "https://example.test/new");
            assert_eq!(body.as_deref(), Some("payload"));
            assert_eq!(headers["X-Other"], "changed");
            assert_eq!(*timeout_secs, 0);
        }
        _ => unreachable!(),
    }
    for (id, value) in [
        ("method", "CONNECT"),
        ("url", "not a URL"),
        ("headers.add", "x-other=duplicate"),
        ("headers.add", "Bad Name=value"),
        ("headers.0.value", "bad\r\nInjected: value"),
        ("timeout_secs", "1.5"),
    ] {
        rejected(&mut w, node, &format!("config.http.{id}"), value);
    }
    edit(&mut w, node, "config.http.body", "");
    edit(&mut w, node, "config.http.headers.0.key", "");
    assert!(matches!(&w.configs[&node.0], NodeConfig::Http { body: None, headers, .. } if headers.is_empty()));
}

#[test]
fn file_and_image_fields_validate_paths_rows_enums_and_platform_bounds() {
    let mut w = workflow();
    let file = w.add_file_node("file", crate::file_node::FileOperationKind::Read, "/tmp/file");
    for operation in ["read", "write", "append"] {
        edit(&mut w, file, "config.file.operation", operation);
    }
    for (id, value) in [
        ("path", "/tmp/new"),
        ("content", "content"),
        ("overwrite", "true"),
        ("allowed_roots.add", "/tmp"),
    ] {
        edit(&mut w, file, &format!("config.file.{id}"), value);
    }
    match &w.configs[&file.0] {
        NodeConfig::File {
            operation,
            path,
            content,
            overwrite,
            allowed_roots,
        } => {
            assert_eq!(*operation, crate::file_node::FileOperationKind::Append);
            assert_eq!(path, "/tmp/new");
            assert_eq!(content.as_deref(), Some("content"));
            assert!(*overwrite);
            assert_eq!(allowed_roots, &["/tmp"]);
        }
        _ => unreachable!(),
    }
    for (id, value) in [
        ("operation", "delete"),
        ("path", "../escape"),
        ("path", "/outside/file"),
        ("overwrite", "1"),
        ("allowed_roots.add", "/tmp"),
    ] {
        rejected(&mut w, file, &format!("config.file.{id}"), value);
    }
    edit(&mut w, file, "config.file.allowed_roots.0", "/");
    edit(&mut w, file, "config.file.allowed_roots.0", "");
    edit(&mut w, file, "config.file.content", "");
    assert!(
        matches!(&w.configs[&file.0], NodeConfig::File { content: None, allowed_roots, .. } if allowed_roots.is_empty())
    );
    let image = w.add_image_input_node("image", "/tmp/image.png");
    edit(&mut w, image, "config.image_input.path", "/tmp/new.jpg");
    edit(&mut w, image, "config.image_input.allowed_roots.add", "/tmp");
    edit(&mut w, image, "config.image_input.max_bytes", &usize::MAX.to_string());
    assert!(
        matches!(&w.configs[&image.0], NodeConfig::ImageInput { path, max_bytes, .. } if path == "/tmp/new.jpg" && *max_bytes == usize::MAX)
    );
    for (id, value) in [
        ("path", "/tmp/unsupported.gif"),
        ("path", "../image.png"),
        ("max_bytes", "0"),
        ("max_bytes", "999999999999999999999999999999"),
        ("allowed_roots.add", "/tmp"),
    ] {
        rejected(&mut w, image, &format!("config.image_input.{id}"), value);
    }
    edit(&mut w, image, "config.image_input.allowed_roots.0", "/");
    edit(&mut w, image, "config.image_input.allowed_roots.0", "");
    assert!(matches!(&w.configs[&image.0], NodeConfig::ImageInput { allowed_roots, .. } if allowed_roots.is_empty()));
}

#[test]
fn agent_source_is_opaque_and_all_fields_preserve_typed_values_without_io() {
    let mut w = workflow();
    let node = w.add_agent_node(
        "agent",
        crate::agent_node::AgentTarget::Pi,
        "prompt",
        crate::agent_node::AgentOutputMode::Text,
    );
    for target in ["pi", "clankers", "animus"] {
        edit(&mut w, node, "config.agent.target", target);
        assert!(
            matches!(&w.configs[&node.0], NodeConfig::Agent(settings) if crate::agent_node::agent_target_label(settings.target) == target)
        );
    }
    for mode in ["text", "json", "image", "artifact"] {
        edit(&mut w, node, "config.agent.output_mode", mode);
        assert!(matches!(&w.configs[&node.0], NodeConfig::Agent(settings) if settings.output_mode.as_str() == mode));
    }
    for (id, value) in [
        ("prompt", "new prompt"),
        ("prompt_file", "prompt.md"),
        ("model_hint", "model"),
        ("tool_policy", "read-only"),
        ("context_refs.add", "opaque:context"),
        ("context_refs.0", "opaque:updated"),
        ("cwd", "/work"),
        ("session", "session"),
    ] {
        edit(&mut w, node, &format!("config.agent.{id}"), value);
    }
    match &w.configs[&node.0] {
        NodeConfig::Agent(settings) => {
            assert_eq!(settings.prompt, "new prompt");
            assert_eq!(settings.prompt_file.as_deref(), Some("prompt.md"));
            assert_eq!(settings.model_hint.as_deref(), Some("model"));
            assert_eq!(settings.tool_policy, "read-only");
            assert_eq!(settings.context_refs, ["opaque:updated"]);
            assert_eq!(settings.cwd.as_deref(), Some("/work"));
            assert_eq!(settings.session.as_deref(), Some("session"));
        }
        _ => unreachable!(),
    }
    for (id, value) in [
        ("target", "unknown"),
        ("output_mode", "raw"),
        ("prompt", ""),
        ("prompt_file", "../source.md"),
        ("prompt_file", "/absolute/prompt.md"),
        ("prompt_file", "prompt.txt"),
        ("prompt_file", "./prompt.md"),
        ("context_refs.add", "opaque:updated"),
    ] {
        rejected(&mut w, node, &format!("config.agent.{id}"), value);
    }
    for id in [
        "prompt_file",
        "model_hint",
        "cwd",
        "session",
        "tool_policy",
        "context_refs.0",
    ] {
        edit(&mut w, node, &format!("config.agent.{id}"), "");
    }
    assert!(
        matches!(&w.configs[&node.0], NodeConfig::Agent(settings) if settings.prompt_file.is_none() && settings.model_hint.is_none() && settings.cwd.is_none() && settings.session.is_none() && settings.context_refs.is_empty() && settings.tool_policy.is_empty())
    );
}

#[test]
fn condition_rows_reconcile_by_label_preserving_tags_edges_and_reorder() {
    let mut w = workflow();
    let node = w.add_condition_node("condition", control_flow::ConditionNodeConfig {
        branches: vec![ConditionBranch {
            label: "keep".into(),
            predicate: ConditionPredicate::Always,
        }],
    });
    let sink = w.add_shell_node("sink", "cat");
    w.graph.node_mut(node).unwrap().output_ports[0].type_tag = "string".into();
    let source = w.graph.node(node).unwrap().output_ports[0].id;
    let target = w.graph.node(sink).unwrap().input_ports[0].id;
    w.graph.add_edge(source, target).unwrap();
    edit(&mut w, node, "config.condition.branches.add", "added=gt:3.5");
    edit(&mut w, node, "config.condition.branches.1.move", "0");
    let ports = &w.graph.node(node).unwrap().output_ports;
    assert_eq!(ports[1].id, source);
    assert_eq!(ports[1].type_tag, "string");
    assert_eq!(ports[0].label, "added");
    assert_eq!(w.graph.edges(), &[crate::engine::workflow::Edge { source, target }]);
    for predicate in [
        "exists",
        "eq:x=y",
        "contains:x",
        "gt:-3.25",
        "json_exists:/a~1b",
        "bool:false",
        "always",
    ] {
        edit(&mut w, node, "config.condition.branches.0.predicate", predicate);
    }
    for (id, value) in [
        ("branches.add", "keep=always"),
        ("branches.add", "=always"),
        ("branches.add", "bad=invalid"),
        ("branches.0.predicate", "gt:NaN"),
        ("branches.0.predicate", "gt:inf"),
        ("branches.0.predicate", "json_exists:relative"),
        ("branches.0.predicate", "json_exists:/bad~2"),
        ("branches.0.move", "99"),
    ] {
        rejected(&mut w, node, &format!("config.condition.{id}"), value);
    }
    edit(&mut w, node, "config.condition.branches.1.label", "renamed");
    assert_ne!(w.graph.node(node).unwrap().output_ports[1].id, source);
    assert!(w.graph.edges().is_empty());
    edit(&mut w, node, "config.condition.branches.0.label", "");
    assert_eq!(w.graph.node(node).unwrap().output_ports[0].label, "renamed");
    let mut encoded = serde_json::to_value(&w).unwrap();
    encoded["graph"]["next_port_id"] = serde_json::json!(u64::MAX);
    let mut exhausted: Workflow = serde_json::from_value(encoded).unwrap();
    rejected_pure(&mut exhausted, node, "config.condition.branches.add", "overflow=always");
}

#[test]
fn merge_and_subworkflow_rows_reconcile_inputs_outputs_without_identity_retargeting() {
    let mut w = workflow();
    let merge = w.add_merge_node("merge", control_flow::MergeNodeConfig {
        policy: MergePolicy::All,
        required_inputs: vec!["keep".into()],
    });
    let keep = w.graph.node(merge).unwrap().input_ports[0].id;
    for policy in ["all", "first", "last", "collect"] {
        edit(&mut w, merge, "config.merge.policy", policy);
    }
    edit(&mut w, merge, "config.merge.required_inputs.add", "added");
    assert_eq!(w.graph.node(merge).unwrap().input_ports[0].id, keep);
    rejected(&mut w, merge, "config.merge.policy", "minimum");
    rejected(&mut w, merge, "config.merge.required_inputs.add", "keep");
    edit(&mut w, merge, "config.merge.required_inputs.1", "renamed");
    edit(&mut w, merge, "config.merge.required_inputs.1", "");
    assert!(
        matches!(&w.configs[&merge.0], NodeConfig::Merge(config) if config.policy == MergePolicy::Collect && config.required_inputs == ["keep"])
    );
    let child = w.add_subworkflow_node("child", control_flow::SubworkflowNodeConfig {
        workflow_name: "old".into(),
        input_mappings: Vec::new(),
        output_mappings: Vec::new(),
    });
    edit(&mut w, child, "config.subworkflow.workflow_name", "new");
    edit(&mut w, child, "config.subworkflow.input_mappings.add", "parent=child");
    edit(&mut w, child, "config.subworkflow.output_mappings.add", "child-out=parent-out");
    let input = w.graph.node(child).unwrap().input_ports[0].id;
    let output = w.graph.node(child).unwrap().output_ports[0].id;
    edit(&mut w, child, "config.subworkflow.input_mappings.0.child_input", "changed-child");
    edit(&mut w, child, "config.subworkflow.output_mappings.0.child_output", "changed-out");
    assert_eq!(w.graph.node(child).unwrap().input_ports[0].id, input);
    assert_eq!(w.graph.node(child).unwrap().output_ports[0].id, output);
    edit(&mut w, child, "config.subworkflow.input_mappings.0.parent_input", "renamed-parent");
    edit(&mut w, child, "config.subworkflow.output_mappings.0.parent_output", "renamed-output");
    assert_ne!(w.graph.node(child).unwrap().input_ports[0].id, input);
    assert_ne!(w.graph.node(child).unwrap().output_ports[0].id, output);
    for (id, value) in [
        ("workflow_name", ""),
        ("input_mappings.add", "broken"),
        ("input_mappings.add", "renamed-parent=duplicate"),
        ("output_mappings.add", "duplicate=renamed-output"),
    ] {
        rejected(&mut w, child, &format!("config.subworkflow.{id}"), value);
    }
    edit(&mut w, child, "config.subworkflow.input_mappings.0.child_input", "");
    edit(&mut w, child, "config.subworkflow.output_mappings.0.parent_output", "");
    assert!(w.graph.node(child).unwrap().input_ports.is_empty());
    assert_eq!(w.graph.node(child).unwrap().output_ports[0].label, "result");
}

#[test]
fn approval_nested_fields_create_transition_validate_and_reconcile_without_authority_effects() {
    let mut w = workflow();
    let node = NodeId(create_approval(&mut w, "approval", "gate", "operator").unwrap());
    for (id, value) in [
        ("gate_id", "renamed-gate"),
        ("safe_summary", "Review the proposed change"),
        ("selected_input_ports.add", "request"),
        ("selected_input_ports.0", "renamed-request"),
        ("authority.local_actors.add", "reviewer"),
        ("authority.local_actors.1", "reviewer-two"),
        ("authority.local_actors.1", ""),
        ("authority.accepted_abilities.1", ""),
        ("authority.accepted_abilities.0", "reject"),
        ("authority.accepted_abilities.add", "approve"),
        ("authority.prohibit_self_approval", "false"),
        ("expiry.decision_ttl_millis", "1"),
        ("expiry.resume_ttl_millis", "31622400000"),
        ("expiry.approval_must_be_fresh_at_resume", "false"),
        ("expiry.clock_source", "utc_wall_clock"),
        ("reason.required", "true"),
        ("reason.identity_bound", "false"),
        ("reason.max_bytes", "4096"),
        ("resume.values.add", "answer=string"),
        ("resume.values.0.name", "approved-answer"),
        ("resume.values.0.type_tag", "json"),
        ("resume.credential_bindings.add", "token"),
        ("resume.credential_bindings.0", "renamed-token"),
        ("terminal.on_rejection", "cancel_workflow"),
        ("terminal.on_expiry", "fail_workflow"),
        ("required_non_claims.reset", "reset"),
        ("required_non_claims.0.move", "1"),
    ] {
        edit(&mut w, node, &format!("config.approval.{id}"), value);
    }
    let digest = "a".repeat(64);
    edit(&mut w, node, "config.approval.required_artifacts.add", &digest);
    edit(&mut w, node, "config.approval.required_artifacts.0", &"b".repeat(64));
    let inputs = &w.graph.node(node).unwrap().input_ports;
    assert_eq!(inputs[0].label, "renamed-request");
    let outputs = &w.graph.node(node).unwrap().output_ports;
    assert_eq!(outputs[0].label, "approved-answer");
    assert_eq!(outputs[0].type_tag, "json");
    assert_eq!(outputs[1].label, "credential:renamed-token");
    match &w.configs[&node.0] {
        NodeConfig::Approval(config) => {
            assert_eq!(config.gate_id.as_str(), "renamed-gate");
            assert_eq!(config.safe_summary, "Review the proposed change");
            assert_eq!(config.authority.local_actors[0].as_str(), "operator");
            assert!(!config.authority.prohibit_self_approval);
            assert_eq!(config.authority.accepted_abilities, [DecisionAbility::Reject, DecisionAbility::Approve]);
            assert_eq!(config.expiry.decision_ttl_millis, 1);
            assert_eq!(config.expiry.resume_ttl_millis, 31_622_400_000);
            assert!(!config.expiry.approval_must_be_fresh_at_resume);
            assert!(config.reason.required);
            assert!(!config.reason.identity_bound);
            assert_eq!(config.reason.max_bytes, 4096);
            assert_eq!(config.terminal.on_rejection, TerminalOutcome::CancelWorkflow);
            assert_eq!(config.terminal.on_expiry, TerminalOutcome::FailWorkflow);
            assert_eq!(config.required_artifacts[0].as_str(), "b".repeat(64));
            assert_eq!(config.required_non_claims[1], REQUIRED_APPROVAL_NON_CLAIMS[0]);
        }
        _ => unreachable!(),
    }
    for (id, value) in [
        ("gate_id", ""),
        ("safe_summary", ""),
        ("selected_input_ports.add", "renamed-request"),
        ("required_artifacts.add", "malformed"),
        ("authority.local_actors.add", "operator"),
        ("authority.accepted_abilities.add", "approve"),
        ("authority.accepted_abilities.add", "unknown"),
        ("authority.profile", "unknown"),
        ("authority.profile", "basalt_ucan_verified"),
        ("authority.prohibit_self_approval", "yes"),
        ("authority.delegated_policy_reference", "invalid for local"),
        ("expiry.decision_ttl_millis", "0"),
        ("expiry.resume_ttl_millis", "31622400001"),
        ("expiry.decision_ttl_millis", "18446744073709551616"),
        ("expiry.clock_source", "monotonic"),
        ("reason.max_bytes", "0"),
        ("reason.max_bytes", "4097"),
        ("resume.values.add", "approved-answer=string"),
        ("resume.values.0.type_tag", "secret"),
        ("resume.credential_bindings.add", "renamed-token"),
        ("terminal.on_expiry", "continue"),
        ("required_non_claims.0", "unreviewed claim"),
        ("required_non_claims.add", "unreviewed claim"),
        ("required_non_claims.reset", "wrong"),
    ] {
        rejected(&mut w, node, &format!("config.approval.{id}"), value);
    }
    rejected(&mut w, node, "config.approval.safe_summary", &"x".repeat(513));
    edit(&mut w, node, "config.approval.authority.profile", "basalt_ucan_verified|reviewed-policy");
    edit(&mut w, node, "config.approval.authority.delegated_policy_reference", "other-reviewed-policy");
    assert!(
        matches!(&w.configs[&node.0], NodeConfig::Approval(config) if config.authority.profile == AuthorityProfile::BasaltUcanVerified && config.authority.local_actors.is_empty() && config.authority.delegated_policy_reference.as_deref() == Some("other-reviewed-policy"))
    );
    edit(&mut w, node, "config.approval.authority.profile", "local_operator|another-operator");
    for id in [
        "required_artifacts.0",
        "selected_input_ports.0",
        "resume.values.0.name",
        "resume.credential_bindings.0",
    ] {
        edit(&mut w, node, &format!("config.approval.{id}"), "");
    }
    assert!(w.graph.node(node).unwrap().input_ports.is_empty());
    assert!(w.graph.node(node).unwrap().output_ports.is_empty());
    let before = serde_json::to_value(&w).unwrap();
    assert!(create_approval(&mut w, "duplicate", "renamed-gate", "operator").is_err());
    assert!(create_approval(&mut w, "invalid", "other", "").is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
    let other = NodeId(create_approval(&mut w, "other", "other-gate", "operator").unwrap());
    rejected(&mut w, other, "config.approval.gate_id", "renamed-gate");
}

#[test]
fn stale_wrong_owner_and_exhausted_creation_preserve_the_aggregate() {
    let mut w = workflow();
    let shell = w.add_shell_node("shell", "true");
    rejected_pure(&mut w, shell, "config.http.url", "https://example.test");
    rejected_pure(&mut w, NodeId(u64::MAX), "config.shell.command", "new");
    assert!(fields(&w, u64::MAX).is_empty());
    let mut encoded = serde_json::to_value(&w).unwrap();
    encoded["graph"]["next_node_id"] = serde_json::json!(u64::MAX);
    let mut exhausted: Workflow = serde_json::from_value(encoded).unwrap();
    let before = serde_json::to_value(&exhausted).unwrap();
    assert!(create_approval(&mut exhausted, "gate", "gate", "actor").is_err());
    assert_eq!(serde_json::to_value(&exhausted).unwrap(), before);
}

fn admitted_operation() -> (
    crate::operation::InvocationPlan,
    crate::operation::ValidatedCatalog,
    crate::operation::ValidatedInstance,
) {
    use std::collections::BTreeMap;

    use crate::operation as op;
    let bounds = op::Bounds {
        timeout_seconds: 60,
        input_bytes: 4096,
        output_bytes: 4096,
        artifact_bytes: 4096,
    };
    let mut operation_type = op::OperationType {
        id: "cairn.repository".into(),
        schema_version: op::model::TYPE_SCHEMA_VERSION,
        revision: op::model::BLAKE3_PREFIX.to_string() + &"0".repeat(op::model::BLAKE3_HEX_LENGTH),
        description: "Pure admitted selection fixture".into(),
        tags: Vec::new(),
        methods: vec![op::model::Method {
            name: op::runtime::CAIRN_CHANGE_LIST_METHOD.into(),
            description: "Read changes".into(),
            arguments: op::model::RecordContract {
                fields: Vec::new(),
                deny_unknown_fields: true,
            },
            output: op::model::RecordContract {
                fields: vec![op::model::FieldContract {
                    name: "valid".into(),
                    value_type: op::model::ValueType::Boolean,
                    required: true,
                    max_bytes: 64,
                }],
                deny_unknown_fields: false,
            },
            preflight: vec![
                op::model::PreflightCheck::RuntimeAvailable,
                op::model::PreflightCheck::ExecutableAvailable,
                op::model::PreflightCheck::RepositoryRootAdmitted,
                op::model::PreflightCheck::CredentialReferencesValid,
            ],
            capabilities: vec![op::CapabilityClass::ProcessRead],
            side_effect: op::model::SideEffectClass::ReadOnly,
        }],
        runtime: op::model::RuntimeRequirement {
            kind: op::runtime::CAIRN_RUNTIME_KIND.into(),
            key: op::runtime::CAIRN_RUNTIME_KEY.into(),
            executor_identity: op::identity::hash_bytes("lattice.test.executor.v1", b"typed authoring executor fact"),
        },
        execution_requirements: op::ExecutionRequirements::cairn_read_only(),
        capabilities: vec![op::CapabilityClass::ProcessRead],
        credential_bindings: Vec::new(),
        bounds,
        required_non_claims: op::model::REQUIRED_NON_CLAIMS.iter().map(|value| (*value).into()).collect(),
    };
    operation_type.revision = op::identity::type_revision(&operation_type).unwrap();
    let catalog = op::catalog::validate(op::Catalog {
        schema: op::model::CATALOG_SCHEMA.into(),
        types: vec![operation_type],
    })
    .unwrap();
    let operation_type = &catalog.types["cairn.repository"].definition;
    let instance = op::instance::validate(
        op::InstanceFile {
            schema: op::model::INSTANCE_SCHEMA.into(),
            instances: vec![op::OperationInstance {
                id: "repository".into(),
                display_alias: "Repository".into(),
                type_id: operation_type.id.clone(),
                type_revision: operation_type.revision.clone(),
                target: BTreeMap::from([(
                    op::runtime::REPOSITORY_ROOT_TARGET.into(),
                    serde_json::json!("/opaque/repository"),
                )]),
                tags: BTreeMap::new(),
                method_defaults: BTreeMap::new(),
                credentials: BTreeMap::new(),
            }],
        },
        &catalog,
    )
    .unwrap()
    .remove("repository")
    .unwrap();
    let plan = op::planning::plan(op::PlanRequest {
        catalog: &catalog,
        instance: &instance,
        method: op::runtime::CAIRN_CHANGE_LIST_METHOD,
        arguments: BTreeMap::new(),
        expected_catalog_identity: &catalog.identity,
        expected_instance_identity: &instance.identity,
        runtime: op::RuntimeFact {
            kind: operation_type.runtime.kind.clone(),
            key: operation_type.runtime.key.clone(),
            executor_identity: operation_type.runtime.executor_identity.clone(),
            executable_path: "/opaque/cairn".into(),
            available: true,
            maximum_bounds: bounds,
        },
        policy: op::CapabilityPolicy {
            allowed: vec![op::CapabilityClass::ProcessRead],
            allow_mutation: false,
        },
        preflight: op::PreflightFacts {
            executable_available: true,
            repository_root_admitted: true,
            credential_references_valid: true,
        },
    })
    .unwrap();
    (plan, catalog, instance)
}

#[test]
fn operation_selection_requires_reproducible_admission_and_never_forges_identity() {
    let (plan, catalog, instance) = admitted_operation();
    let mut w = workflow();
    let node = NodeId(create_operation(&mut w, "operation", plan.clone(), &catalog, &instance).unwrap());
    assert!(fields(&w, node.0).is_empty());
    rejected_pure(&mut w, node, "config.operation.method", "forged-method");
    let ports = w.graph.node(node).unwrap().output_ports.clone();
    replace_operation_plan(&mut w, node.0, plan.clone(), &catalog, &instance).unwrap();
    assert_eq!(w.graph.node(node).unwrap().output_ports, ports);
    assert!(matches!(&w.configs[&node.0], NodeConfig::Operation(actual) if actual.as_ref() == &plan));
    let before = serde_json::to_value(&w).unwrap();
    let mut forged = plan.clone();
    forged.method = "forged".into();
    assert!(replace_operation_plan(&mut w, node.0, forged.clone(), &catalog, &instance).is_err());
    assert!(create_operation(&mut w, "forged", forged, &catalog, &instance).is_err());
    let mut stale_instance = instance.clone();
    stale_instance.definition.target.insert("repository_root".into(), serde_json::json!("/other"));
    assert!(replace_operation_plan(&mut w, node.0, plan, &catalog, &stale_instance).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
}

#[test]
fn approval_named_size_limits_and_supported_resume_types_fail_closed() {
    let mut w = workflow();
    let node = NodeId(create_approval(&mut w, "gate", "bounded", "operator").unwrap());
    edit(&mut w, node, "config.approval.safe_summary", &"s".repeat(512));
    edit(
        &mut w,
        node,
        "config.approval.authority.profile",
        &format!("basalt_ucan_verified|{}", "p".repeat(512)),
    );
    rejected(&mut w, node, "config.approval.authority.delegated_policy_reference", &"p".repeat(513));
    edit(&mut w, node, "config.approval.authority.profile", "local_operator|operator");
    edit(&mut w, node, "config.approval.resume.values.add", "value=string");
    for type_tag in ["string", "bytes", "json", "image", "exitcode", "any"] {
        edit(&mut w, node, "config.approval.resume.values.0.type_tag", type_tag);
        assert_eq!(w.graph.node(node).unwrap().output_ports[0].type_tag, type_tag);
    }
    for row in 0..128 {
        edit(&mut w, node, "config.approval.selected_input_ports.add", &format!("input-{row}"));
        edit(&mut w, node, "config.approval.required_artifacts.add", &format!("{row:064x}"));
        edit(&mut w, node, "config.approval.resume.credential_bindings.add", &format!("credential-{row}"));
        if row > 0 {
            edit(&mut w, node, "config.approval.authority.local_actors.add", &format!("actor-{row}"));
            edit(&mut w, node, "config.approval.resume.values.add", &format!("value-{row}=json"));
        }
    }
    for (id, value) in [
        ("selected_input_ports.add", "overflow"),
        ("required_artifacts.add", &format!("{:064x}", 128)),
        ("resume.credential_bindings.add", "overflow"),
        ("authority.local_actors.add", "overflow"),
        ("resume.values.add", "overflow=json"),
    ] {
        rejected(&mut w, node, &format!("config.approval.{id}"), value);
    }
}

#[test]
fn non_topology_config_edits_preserve_unrelated_custom_graph_declarations() {
    let mut w = workflow();
    let merge = w.add_merge_node("merge", control_flow::MergeNodeConfig {
        policy: MergePolicy::All,
        required_inputs: vec!["required".into()],
    });
    w.graph
        .reconcile_node_ports(merge, PortDirection::Input, &[
            PortSpec::new("required", "string"),
            PortSpec::new("extra", "json"),
        ])
        .unwrap();
    let before = w.graph.clone();
    edit(&mut w, merge, "config.merge.policy", "collect");
    assert_eq!(w.graph, before);
    let child = w.add_subworkflow_node("child", control_flow::SubworkflowNodeConfig {
        workflow_name: "old".into(),
        input_mappings: Vec::new(),
        output_mappings: Vec::new(),
    });
    w.graph
        .reconcile_node_ports(child, PortDirection::Input, &[PortSpec::new("custom", "json")])
        .unwrap();
    let before = w.graph.clone();
    edit(&mut w, child, "config.subworkflow.workflow_name", "new");
    assert_eq!(w.graph, before);
    let gate = NodeId(create_approval(&mut w, "gate", "custom", "operator").unwrap());
    w.graph
        .reconcile_node_ports(gate, PortDirection::Output, &[PortSpec::new("custom", "json")])
        .unwrap();
    let before = w.graph.clone();
    edit(&mut w, gate, "config.approval.safe_summary", "new summary");
    assert_eq!(w.graph, before);
}
