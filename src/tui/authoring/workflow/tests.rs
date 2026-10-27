use super::*;
use crate::engine::workflow::WorkflowTimestamp;

fn apply(workflow: &mut Workflow, node_id: Option<u64>, id: &str, value: &str) -> Result<(), String> {
    crate::tui::authoring::test_support::edit(workflow, node_id, id, value)
}

fn fixture() -> (Workflow, u64, u64) {
    let mut workflow = Workflow::new("authoring", WorkflowTimestamp::epoch());
    let first = workflow.add_shell_node("source", "printf data").0;
    let second = workflow.add_shell_node("sink", "cat").0;
    (workflow, first, second)
}

fn snapshot(workflow: &Workflow) -> serde_json::Value {
    serde_json::to_value(workflow).unwrap()
}

fn rejects(workflow: &mut Workflow, node: Option<u64>, id: &str, value: &str) {
    let before = snapshot(workflow);
    assert!(apply(workflow, node, id, value).is_err(), "unexpectedly accepted {id}={value}");
    assert_eq!(snapshot(workflow), before, "invalid edit mutated {id}");
}

fn source_target(workflow: &Workflow, first: u64, second: u64) -> (u64, u64) {
    (
        workflow.graph.node(NodeId(first)).unwrap().output_ports[0].id.0,
        workflow.graph.node(NodeId(second)).unwrap().input_ports[0].id.0,
    )
}

// r[verify lattice.tui_ux.portable_authoring_parity.workflow_graph.scenario.edit]
#[test]
fn metadata_edits_are_explicit_and_rename_owned_triggers() {
    let (mut workflow, _, _) = fixture();
    apply(&mut workflow, None, "trigger.add", "manual|manual").unwrap();
    apply(&mut workflow, None, "workflow.description", "operator-authored\nmetadata").unwrap();
    apply(&mut workflow, None, "workflow.created", "2026-10-15T20:00:00-04:00").unwrap();
    apply(&mut workflow, None, "workflow.modified", "2026-10-16T01:00:00Z").unwrap();
    let created = workflow.meta.created;
    let modified = workflow.meta.modified;
    apply(&mut workflow, None, "workflow.name", "renamed").unwrap();
    assert_eq!(workflow.meta.name, "renamed");
    assert_eq!(workflow.triggers[0].workflow_name, "renamed");
    assert_eq!(created.to_rfc3339(), "2026-10-16T00:00:00+00:00");
    assert_eq!(workflow.meta.created, created);
    assert_eq!(workflow.meta.modified, modified);
    apply(&mut workflow, None, "workflow.description", "").unwrap();
    assert_eq!(workflow.meta.description, "");
    rejects(&mut workflow, None, "workflow.name", " ");
    rejects(&mut workflow, None, "workflow.created", "tomorrow");
    rejects(&mut workflow, None, "workflow.modified", "2026-99-99T00:00:00Z");
}

#[test]
fn selected_node_metadata_is_bounded_and_scoped() {
    let (mut workflow, first, second) = fixture();
    let untouched = workflow.graph.node(NodeId(second)).unwrap().clone();
    apply(&mut workflow, Some(first), "graph.label", "renamed source").unwrap();
    apply(&mut workflow, Some(first), "graph.x", &i32::MIN.to_string()).unwrap();
    apply(&mut workflow, Some(first), "graph.y", &i32::MAX.to_string()).unwrap();
    let node = workflow.graph.node(NodeId(first)).unwrap();
    assert_eq!((node.id.0, node.label.as_str(), node.x, node.y), (first, "renamed source", i32::MIN, i32::MAX));
    assert_eq!(workflow.graph.node(NodeId(second)).unwrap(), &untouched);
    rejects(&mut workflow, Some(first), "graph.x", "2147483648");
    rejects(&mut workflow, Some(first), "graph.y", "-2147483649");
    rejects(&mut workflow, Some(first), "graph.label", "");
    rejects(&mut workflow, None, "graph.label", "missing selection");
    rejects(&mut workflow, Some(u64::MAX), "graph.label", "stale selection");
}

#[test]
fn port_relabel_preserves_identity_order_and_compatible_edges() {
    let (mut workflow, first, second) = fixture();
    let (source, target) = source_target(&workflow, first, second);
    apply(&mut workflow, None, "graph.edge.add", &format!("{source}|{target}")).unwrap();
    let ids = workflow
        .graph
        .node(NodeId(first))
        .unwrap()
        .output_ports
        .iter()
        .map(|port| port.id)
        .collect::<Vec<_>>();
    apply(
        &mut workflow,
        Some(first),
        &format!("graph.port.{source}"),
        &format!("{source}|output|renamed stdout|string"),
    )
    .unwrap();
    let node = workflow.graph.node(NodeId(first)).unwrap();
    assert_eq!(node.output_ports.iter().map(|port| port.id).collect::<Vec<_>>(), ids);
    assert_eq!(node.output_ports[0].label, "renamed stdout");
    assert_eq!(workflow.graph.edges(), &[crate::engine::workflow::Edge {
        source: PortId(source),
        target: PortId(target)
    }]);
    rejects(
        &mut workflow,
        Some(first),
        &format!("graph.port.{source}"),
        &format!("{source}|output|renamed stdout|bytes"),
    );
}

#[test]
fn removing_port_cleans_only_incident_edges_without_reusing_id() {
    let (mut workflow, first, second) = fixture();
    let (source, target) = source_target(&workflow, first, second);
    apply(&mut workflow, Some(first), "graph.port.add", "auto|output|extra|string").unwrap();
    let extra = workflow.graph.node(NodeId(first)).unwrap().output_ports.last().unwrap().id.0;
    apply(&mut workflow, None, "graph.edge.add", &format!("{source}|{target}")).unwrap();
    apply(&mut workflow, None, "graph.edge.add", &format!("{extra}|{target}")).unwrap();
    apply(&mut workflow, Some(first), &format!("graph.port.{extra}"), "").unwrap();
    assert!(workflow.graph.port(PortId(extra)).is_none());
    assert_eq!(workflow.graph.edges(), &[crate::engine::workflow::Edge {
        source: PortId(source),
        target: PortId(target)
    }]);
    apply(&mut workflow, Some(first), "graph.port.add", "auto|output|replacement|string").unwrap();
    assert!(workflow.graph.node(NodeId(first)).unwrap().output_ports.last().unwrap().id.0 > extra);
}

#[test]
fn explicit_port_id_and_direction_changes_remove_incident_edges() {
    let (mut workflow, first, second) = fixture();
    let (source, target) = source_target(&workflow, first, second);
    apply(&mut workflow, None, "graph.edge.add", &format!("{source}|{target}")).unwrap();
    apply(&mut workflow, Some(first), &format!("graph.port.{source}"), "1000|input|response|string").unwrap();
    assert_eq!(workflow.graph.port_owner(PortId(1000)), Some(NodeId(first)));
    assert_eq!(workflow.graph.port(PortId(1000)).unwrap().direction, PortDirection::Input);
    assert!(workflow.graph.port(PortId(source)).is_none());
    assert!(workflow.graph.edges().is_empty());
    apply(&mut workflow, Some(second), "graph.port.add", "auto|output|after explicit id|string").unwrap();
    assert!(workflow.graph.node(NodeId(second)).unwrap().output_ports.last().unwrap().id.0 > 1000);
}

// r[verify lattice.tui_ux.portable_authoring_parity.workflow_graph.scenario.invalid]
#[test]
fn port_collision_direction_duplicate_escape_and_overflow_are_atomic() {
    let (mut workflow, first, second) = fixture();
    let (source, target) = source_target(&workflow, first, second);
    for value in [
        format!("{target}|output|collision|string"),
        "18446744073709551615|output|overflow|string".into(),
        "auto|sideways|invalid|string".into(),
        "auto|output|stdout|string".into(),
        "auto|output||string".into(),
        "auto|output|empty type|".into(),
        "auto|output|bad\\x|string".into(),
    ] {
        rejects(&mut workflow, Some(first), "graph.port.add", &value);
    }
    rejects(&mut workflow, Some(second), &format!("graph.port.{source}"), "");
    apply(&mut workflow, Some(first), "graph.port.add", "auto|output|a\\|b\\\\c|string").unwrap();
    let added = workflow.graph.node(NodeId(first)).unwrap().output_ports.last().unwrap().clone();
    assert_eq!(added.label, "a|b\\c");
    let descriptor = fields(&workflow, Some(first))
        .into_iter()
        .find(|field| field.id == format!("graph.port.{}", added.id.0))
        .unwrap();
    apply(&mut workflow, Some(first), &descriptor.id, &descriptor.value).unwrap();
    assert_eq!(workflow.graph.port(added.id).unwrap(), &added);
}

#[test]
fn edge_add_edit_remove_rejects_wrong_direction_self_type_cycle_and_duplicates() {
    let (mut workflow, first, second) = fixture();
    let (source, target) = source_target(&workflow, first, second);
    let first_input = workflow.graph.node(NodeId(first)).unwrap().input_ports[0].id.0;
    let second_output = workflow.graph.node(NodeId(second)).unwrap().output_ports[0].id.0;
    let exit = workflow.graph.node(NodeId(first)).unwrap().output_ports[2].id.0;
    for value in [
        format!("{target}|{source}"),
        format!("{source}|{first_input}"),
        format!("{exit}|{target}"),
        format!("{}|{target}", u64::MAX),
    ] {
        rejects(&mut workflow, None, "graph.edge.add", &value);
    }
    apply(&mut workflow, None, "graph.edge.add", &format!("{source}|{target}")).unwrap();
    rejects(&mut workflow, None, "graph.edge.add", &format!("{source}|{target}"));
    rejects(&mut workflow, None, "graph.edge.add", &format!("{second_output}|{first_input}"));
    let edge_id = format!("graph.edge.{source}.{target}");
    rejects(&mut workflow, None, &edge_id, "not|ids");
    apply(&mut workflow, None, &edge_id, &format!("{second_output}|{first_input}")).unwrap();
    assert_eq!(workflow.graph.edges(), &[crate::engine::workflow::Edge {
        source: PortId(second_output),
        target: PortId(first_input)
    }]);
    apply(&mut workflow, None, &format!("graph.edge.{second_output}.{first_input}"), "").unwrap();
    assert!(workflow.graph.edges().is_empty());
}

#[test]
fn credential_rows_are_opaque_scoped_and_have_no_add_name_collision() {
    let (mut workflow, first, second) = fixture();
    apply(&mut workflow, Some(first), "credential.env.add", "add|unconfigured-provider|not-loaded|").unwrap();
    apply(&mut workflow, Some(second), "credential.env.add", "OTHER|other-provider|unavailable|token").unwrap();
    let other = workflow.credentials[&second].clone();
    assert!(fields(&workflow, Some(first)).iter().any(|field| field.id == "credential.env.row.add"));
    apply(
        &mut workflow,
        Some(first),
        "credential.env.row.add",
        "TOKEN|unconfigured-provider|not-loaded|password",
    )
    .unwrap();
    assert_eq!(
        workflow.credentials[&first].env["TOKEN"],
        SecretRef::new("unconfigured-provider", "not-loaded", Some("password".into()))
    );
    assert_eq!(workflow.credentials[&second], other);
    for value in [
        "TOKEN|duplicate|key|",
        "BAD-NAME|provider|key|",
        "EMPTY||key|",
        "EMPTY|provider||",
        "EMPTY|provider|key| ",
        "not a typed row",
    ] {
        rejects(&mut workflow, Some(first), "credential.env.add", value);
    }
    apply(&mut workflow, Some(first), "credential.env.row.TOKEN", "").unwrap();
    assert!(!workflow.credentials.contains_key(&first));
    assert_eq!(workflow.credentials[&second], other);
}

// r[verify lattice.tui_ux.portable_authoring_parity.runner.scenario.edit]
#[test]
fn registered_runtime_pair_and_canonical_policies_roundtrip_and_reset() {
    let (mut workflow, first, _) = fixture();
    apply(&mut workflow, Some(first), "runner.binding", "file|read").unwrap();
    assert_eq!(workflow.runner[&first].runtime_kind, "file");
    assert_eq!(workflow.runner[&first].runtime_key, "read");
    apply(&mut workflow, Some(first), "runner.effect_policy", "filesystem").unwrap();
    for (id, value) in [
        ("runner.cache_policy", "disabled"),
        ("runner.retry_policy", "none"),
        ("runner.artifact_policy", "metadata-only"),
    ] {
        apply(&mut workflow, Some(first), id, value).unwrap();
    }
    let portable = crate::workflow_format::workflow_to_portable(&workflow);
    let restored = crate::workflow_format::portable_to_workflow(portable.clone()).unwrap();
    assert_eq!(crate::workflow_format::workflow_to_portable(&restored), portable);
    for id in [
        "runner.effect_policy",
        "runner.cache_policy",
        "runner.retry_policy",
        "runner.artifact_policy",
    ] {
        apply(&mut workflow, Some(first), id, "").unwrap();
    }
    apply(&mut workflow, Some(first), "runner.binding", "").unwrap();
    assert_eq!(workflow.runner[&first], NodeRunnerSettings::new("shell", "default"));
    apply(&mut workflow, Some(first), "runner.remove", "remove").unwrap();
    assert!(!workflow.runner.contains_key(&first));
}

// r[verify lattice.tui_ux.portable_authoring_parity.runner.scenario.invalid]
#[test]
fn runtime_validation_reuses_binding_and_effect_admission_and_rejects_unknown_policies() {
    let (mut workflow, first, _) = fixture();
    for (id, value) in [
        ("runner.runtime_kind", "unknown"),
        ("runner.runtime_key", "missing"),
        ("runner.binding", "file|default"),
        ("runner.binding", "file"),
        ("runner.effect_policy", "network"),
        ("runner.effect_policy", "process,network"),
        ("runner.cache_policy", "enabled"),
        ("runner.retry_policy", "3|exponential"),
        ("runner.artifact_policy", "unbounded"),
        ("runner.retry_limit", "1"),
        ("runner.remove", "yes"),
    ] {
        rejects(&mut workflow, Some(first), id, value);
    }
    apply(&mut workflow, Some(first), "runner.effect_policy", "process").unwrap();
    assert_eq!(workflow.runner[&first].effect_policy, "process");
}

// r[verify lattice.tui_ux.portable_authoring_parity.triggers.scenario.edit]
#[test]
fn manual_schedule_and_webhook_definitions_author_without_effects() {
    let (mut workflow, _, _) = fixture();
    apply(&mut workflow, None, "trigger.add", "manual|manual").unwrap();
    apply(&mut workflow, None, "trigger.add", "schedule|schedule|90|-1000|skip").unwrap();
    apply(&mut workflow, None, "trigger.add", "hook|webhook|post|/inert-hook").unwrap();
    assert!(workflow.triggers.iter().all(|trigger| !trigger.enabled));
    apply(&mut workflow, None, "trigger.0.enabled", "true").unwrap();
    apply(&mut workflow, None, "trigger.0.overlap_policy", "allow").unwrap();
    apply(&mut workflow, None, "trigger.0.idempotency_policy", "event_key").unwrap();
    apply(&mut workflow, None, "trigger.1.schedule_interval_secs", "120").unwrap();
    apply(&mut workflow, None, "trigger.1.schedule_next_run_epoch_ms", "10000").unwrap();
    apply(&mut workflow, None, "trigger.1.schedule_missed_run_policy", "run_once").unwrap();
    apply(&mut workflow, None, "trigger.2.webhook_method", "put").unwrap();
    apply(&mut workflow, None, "trigger.2.webhook_path", "/typed").unwrap();
    apply(&mut workflow, None, "trigger.2.webhook_content_types.add", "application/json").unwrap();
    apply(&mut workflow, None, "trigger.2.auth", "bearer|unconfigured-provider|unavailable|token").unwrap();
    apply(&mut workflow, None, "trigger.2.idempotency_policy", "payload_json_pointer|/event~1id").unwrap();
    assert_eq!(workflow.triggers[0].overlap_policy, OverlapPolicy::Allow);
    assert_eq!(workflow.triggers[0].idempotency_policy, IdempotencyPolicy::EventKey);
    assert!(matches!(&workflow.triggers[1].kind, TriggerKind::Schedule {
        recurrence: RecurrenceRule::EverySeconds { interval_secs: 120 },
        missed_run_policy: MissedRunPolicy::RunOnce,
        next_run_epoch_ms: 10000
    }));
    let TriggerKind::Webhook(webhook) = &workflow.triggers[2].kind else {
        panic!("expected webhook");
    };
    assert_eq!((webhook.method.as_str(), webhook.path.as_str()), ("PUT", "/typed"));
    assert_eq!(webhook.accepted_content_types, ["application/json"]);
    assert_eq!(webhook.auth_policy, WebhookAuthPolicy::Bearer {
        secret_ref: SecretRef::new("unconfigured-provider", "unavailable", Some("token".into()))
    });
    assert_eq!(workflow.triggers[2].idempotency_policy, IdempotencyPolicy::PayloadJsonPointer("/event~1id".into()));
    let portable = crate::workflow_format::workflow_to_portable(&workflow);
    let restored = crate::workflow_format::portable_to_workflow(portable.clone()).unwrap();
    assert_eq!(crate::workflow_format::workflow_to_portable(&restored), portable);
}

// r[verify lattice.tui_ux.portable_authoring_parity.triggers.scenario.invalid]
#[test]
fn trigger_identity_schedule_webhook_and_policy_failures_are_atomic() {
    let (mut workflow, _, _) = fixture();
    for value in [
        "bad|schedule|0|0|skip",
        "bad|schedule|18446744073709551615|0|run_once",
        "bad|schedule|1|9223372036854775807|run_once",
        "bad|webhook|TRACEINVALID|/path",
        "bad|webhook|POST|relative",
        "bad|webhook|POST|//host/path",
        "bad|webhook|POST|/path?q=1",
        "bad|webhook|POST|/space here",
        "bad|future",
        "|manual",
    ] {
        rejects(&mut workflow, None, "trigger.add", value);
    }
    apply(&mut workflow, None, "trigger.add", "hook|webhook|POST|/path").unwrap();
    rejects(&mut workflow, None, "trigger.add", "hook|manual");
    for (id, value) in [
        ("trigger.0.id", ""),
        ("trigger.0.workflow_name", "another workflow"),
        ("trigger.0.enabled", "yes"),
        ("trigger.0.kind", "unknown"),
        ("trigger.0.overlap_policy", "queue"),
        ("trigger.0.idempotency_policy", "unknown"),
        ("trigger.0.idempotency_policy", "payload_json_pointer|bad"),
        ("trigger.0.idempotency_policy", "payload_json_pointer|/bad~2"),
        ("trigger.0.auth", "bearer||key|"),
        ("trigger.0.auth", "blake3_signature|provider|key||bad header"),
        ("trigger.0.webhook_content_types.add", "invalid"),
        ("trigger.0.schedule_interval_secs", "3"),
        ("trigger.0.remove", "true"),
    ] {
        rejects(&mut workflow, None, id, value);
    }
    apply(&mut workflow, None, "trigger.0.webhook_content_types.add", "application/json").unwrap();
    rejects(&mut workflow, None, "trigger.0.webhook_content_types.add", "Application/JSON");
    apply(&mut workflow, None, "trigger.0.kind", "schedule").unwrap();
    for (id, value) in [
        ("trigger.0.schedule_interval_secs", "0"),
        ("trigger.0.schedule_interval_secs", "9223372036854776"),
        ("trigger.0.schedule_next_run_epoch_ms", "9223372036854775808"),
        ("trigger.0.schedule_next_run_epoch_ms", "9223372036854775807"),
        ("trigger.0.schedule_missed_run_policy", "catch_up"),
        ("trigger.0.webhook_path", "/not-a-webhook"),
    ] {
        rejects(&mut workflow, None, id, value);
    }
}

#[test]
fn trigger_input_auth_and_content_rows_edit_remove_and_reset_independently() {
    let (mut workflow, _, _) = fixture();
    apply(&mut workflow, None, "trigger.add", "hook|webhook|POST|/path").unwrap();
    for binding in [
        "payload|json_pointer|/data|json|true",
        "query|query_param|q|string|false",
        "header|header|x-event|string|true",
        "body|raw_body||bytes|false",
    ] {
        apply(&mut workflow, None, "trigger.0.input_bindings.add", binding).unwrap();
    }
    apply(&mut workflow, None, "trigger.0.input_bindings.0", "renamed|json_pointer||any|false").unwrap();
    assert_eq!(workflow.triggers[0].input_bindings[0], TriggerInputBinding {
        input_label: "renamed".into(),
        source: TriggerPayloadSource::JsonPointer("".into()),
        type_tag: "any".into(),
        required: false
    });
    let others = workflow.triggers[0].input_bindings[1..].to_vec();
    apply(&mut workflow, None, "trigger.0.input_bindings.0", "").unwrap();
    assert_eq!(workflow.triggers[0].input_bindings, others);
    apply(&mut workflow, None, "trigger.0.auth", "blake3_signature|vault|webhook|token|x-signature").unwrap();
    let TriggerKind::Webhook(webhook) = &workflow.triggers[0].kind else {
        panic!("expected webhook");
    };
    assert_eq!(webhook.auth_policy, WebhookAuthPolicy::Blake3Signature {
        secret_ref: SecretRef::new("vault", "webhook", Some("token".into())),
        header: "x-signature".into()
    });
    apply(&mut workflow, None, "trigger.0.auth", "").unwrap();
    apply(&mut workflow, None, "trigger.0.webhook_content_types.add", "application/json").unwrap();
    apply(&mut workflow, None, "trigger.0.webhook_content_types.0", "text/plain").unwrap();
    apply(&mut workflow, None, "trigger.0.webhook_content_types.0", "").unwrap();
    let TriggerKind::Webhook(webhook) = &workflow.triggers[0].kind else {
        panic!("expected webhook");
    };
    assert_eq!(webhook.auth_policy, WebhookAuthPolicy::NoAuthLocalOnly);
    assert!(webhook.accepted_content_types.is_empty());
    apply(&mut workflow, None, "trigger.0.enabled", "false").unwrap();
    apply(&mut workflow, None, "trigger.0.remove", "remove").unwrap();
    assert!(workflow.triggers.is_empty());
}

#[test]
fn malformed_duplicate_or_incompatible_trigger_bindings_leave_prior_rows_unchanged() {
    let (mut workflow, _, _) = fixture();
    apply(&mut workflow, None, "trigger.add", "manual|manual").unwrap();
    apply(&mut workflow, None, "trigger.0.input_bindings.add", "existing|json_pointer|/valid|json|true").unwrap();
    for value in [
        "existing|json_pointer|/other|json|false",
        "|json_pointer|/other|json|true",
        "other|json_pointer|bad|json|true",
        "other|json_pointer|/bad~|json|true",
        "other|query_param||string|true",
        "other|header|bad header|string|true",
        "other|raw_body|name|bytes|true",
        "other|raw_body||string|true",
        "other|header|x-name|json|true",
        "other|json_pointer|/name|bytes|true",
        "other|query_param|q|string|yes",
        "other|unknown|q|string|true",
        "not|enough|cells",
    ] {
        rejects(&mut workflow, None, "trigger.0.input_bindings.add", value);
    }
    rejects(&mut workflow, None, "trigger.0.input_bindings.99", "");
    rejects(&mut workflow, None, "trigger.99.enabled", "true");
}

// r[verify lattice.tui_ux.portable_authoring_parity.catalog.scenario.complete]
#[test]
fn descriptors_expose_existing_rows_and_add_actions_without_claiming_absent_runner_fields() {
    let (mut workflow, first, second) = fixture();
    apply(&mut workflow, Some(first), "credential.env.add", "TOKEN|vault|key|").unwrap();
    apply(&mut workflow, None, "trigger.add", "hook|webhook|POST|/hook").unwrap();
    apply(&mut workflow, None, "trigger.0.webhook_content_types.add", "application/json").unwrap();
    apply(&mut workflow, None, "trigger.0.input_bindings.add", "event|header|x-event|string|true").unwrap();
    let (source, target) = source_target(&workflow, first, second);
    apply(&mut workflow, None, "graph.edge.add", &format!("{source}|{target}")).unwrap();
    let descriptors = fields(&workflow, Some(first));
    for id in [
        "credential.env.row.TOKEN".into(),
        "credential.env.add".into(),
        format!("graph.port.{source}"),
        "graph.port.add".into(),
        format!("graph.edge.{source}.{target}"),
        "graph.edge.add".into(),
        "trigger.0.webhook_content_types.0".into(),
        "trigger.0.webhook_content_types.add".into(),
        "trigger.0.input_bindings.0".into(),
        "trigger.0.input_bindings.add".into(),
        "trigger.add".into(),
        "runner.binding".into(),
    ] {
        assert!(descriptors.iter().any(|descriptor| descriptor.id == id), "missing editable row/action {id}");
    }
    let ids = capability_ids();
    assert_eq!(ids.iter().copied().collect::<HashSet<_>>().len(), ids.len());
    assert!(ids.contains(&"trigger.input_bindings.source_name"));
    assert!(ids.contains(&"credential.env.field"));
    assert!(!ids.iter().any(|id| id.contains("backoff")
        || id.contains("retry_count")
        || id.contains("ability")
        || id.contains("limit")));
}

#[test]
fn exhausted_port_allocator_rejects_add_without_changing_graph_or_owned_state() {
    let (mut workflow, first, _) = fixture();
    apply(
        &mut workflow,
        Some(first),
        "graph.port.add",
        &format!("{}|output|last allocatable|string", u64::MAX - 1),
    )
    .unwrap();
    rejects(&mut workflow, Some(first), "graph.port.add", "auto|output|exhausted|string");
    let last = u64::MAX - 1;
    apply(&mut workflow, Some(first), &format!("graph.port.{last}"), "").unwrap();
    rejects(&mut workflow, Some(first), "graph.port.add", "auto|output|still exhausted|string");
}
