//! Typed, effect-free workflow, topology, credential, runner and trigger drafts.
//!
//! Compound rows use escaped pipe-separated cells: \|, \\, \n, \r and \t.
//! Empty optional scalar values reset; empty existing rows remove that row.

use std::collections::HashSet;

use super::Field;
use super::field;
use crate::credentials::NodeCredentialRefs;
use crate::credentials::SecretRef;
use crate::engine::runner;
use crate::engine::workflow::NodeId;
use crate::engine::workflow::NodeRunnerSettings;
use crate::engine::workflow::Port;
use crate::engine::workflow::PortDirection;
use crate::engine::workflow::PortId;
use crate::engine::workflow::PortSpec;
use crate::engine::workflow::Workflow;
use crate::scheduler::MissedRunPolicy;
use crate::scheduler::OverlapPolicy;
use crate::scheduler::RecurrenceRule;
use crate::scheduler::ScheduleDefinition;
use crate::triggers::IdempotencyPolicy;
use crate::triggers::TriggerDefinition;
use crate::triggers::TriggerInputBinding;
use crate::triggers::TriggerKind;
use crate::triggers::TriggerPayloadSource;
use crate::triggers::WebhookAuthPolicy;
use crate::triggers::WebhookTrigger;

const ROW_HELP: &str =
    "Pipe-separated cells; escape pipe/backslash as \\|/\\\\ and newline as \\n. Blank existing row removes it.";
const PORT_HELP: &str = "id|input or output|label|type_tag; use auto for new id. Blank removes port and incident edges; changing id/direction removes incident edges. Unchanged rows keep ids.";
const BINDING_HELP: &str = "label|json_pointer, query_param, header or raw_body|source_name|type_tag|required (true/false). Raw body has empty source_name. Blank removes row.";
const AUTH_HELP: &str = "none; bearer|provider|key|optional field; blake3_signature|provider|key|optional field|header. Blank resets to local-only no auth. Opaque references only; never resolved.";
const DEFAULT_INTERVAL_SECS: u64 = 60;
const MAX_INTERVAL_SECS: u64 = (i64::MAX / 1_000) as u64;

// r[impl lattice.tui_ux.portable_authoring_parity.catalog]
pub fn capability_ids() -> Vec<&'static str> {
    vec![
        "workflow.name",
        "workflow.description",
        "workflow.created",
        "workflow.modified",
        "graph.label",
        "graph.x",
        "graph.y",
        "graph.port.id",
        "graph.port.direction",
        "graph.port.label",
        "graph.port.type_tag",
        "graph.edge.source",
        "graph.edge.target",
        "credential.env.name",
        "credential.env.provider",
        "credential.env.key",
        "credential.env.field",
        "runner.runtime_kind",
        "runner.runtime_key",
        "runner.effect_policy",
        "runner.cache_policy",
        "runner.retry_policy",
        "runner.artifact_policy",
        "trigger.id",
        "trigger.workflow_name",
        "trigger.enabled",
        "trigger.kind",
        "trigger.schedule_interval_secs",
        "trigger.schedule_next_run_epoch_ms",
        "trigger.schedule_missed_run_policy",
        "trigger.webhook_method",
        "trigger.webhook_path",
        "trigger.webhook_content_types",
        "trigger.auth_kind",
        "trigger.auth_provider",
        "trigger.auth_key",
        "trigger.auth_field",
        "trigger.auth_header",
        "trigger.input_bindings.input_label",
        "trigger.input_bindings.source_kind",
        "trigger.input_bindings.source_name",
        "trigger.input_bindings.type_tag",
        "trigger.input_bindings.required",
        "trigger.overlap_policy",
        "trigger.idempotency_policy",
        "trigger.idempotency_json_pointer",
    ]
}

pub fn fields(workflow: &Workflow, node_id: Option<u64>) -> Vec<Field> {
    let mut result = vec![
        field(
            "workflow.name",
            "Workflow name",
            "Required name; owned trigger workflow references are renamed atomically.",
            &workflow.meta.name,
        ),
        field("workflow.description", "Description", "Free text; blank clears.", &workflow.meta.description),
        field(
            "workflow.created",
            "Created timestamp",
            "Explicit RFC3339 timestamp normalized to UTC; never reads the clock.",
            workflow.meta.created.to_rfc3339(),
        ),
        field(
            "workflow.modified",
            "Modified timestamp",
            "Explicit RFC3339 timestamp normalized to UTC; no automatic timestamp changes.",
            workflow.meta.modified.to_rfc3339(),
        ),
    ];
    if let Some(node_id) = node_id.filter(|id| workflow.graph.node(NodeId(*id)).is_some()) {
        node_fields(workflow, node_id, &mut result);
    }
    for edge in workflow.graph.edges() {
        result.push(field(&format!("graph.edge.{}.{}", edge.source.0, edge.target.0), "Edge", "source port id|target port id; blank removes. Direction, owner, type, cycles and duplicates are validated.", format!("{}|{}", edge.source.0, edge.target.0)));
    }
    result.push(field(
        "graph.edge.add",
        "+ Edge",
        "source port id|target port id; compatible output to input only.",
        "",
    ));
    for (index, trigger) in workflow.triggers.iter().enumerate() {
        trigger_fields(index, trigger, &mut result);
    }
    result.push(field("trigger.add", "+ Trigger", "id|manual; id|schedule|interval seconds|next epoch milliseconds|run_once or skip; id|webhook|method|path. Created disabled; authoring never activates a scheduler/listener.", ""));
    result
}

// r[impl lattice.tui_ux.portable_authoring_parity.workflow_graph]
fn node_fields(workflow: &Workflow, node_id: u64, result: &mut Vec<Field>) {
    let Some(node) = workflow.graph.node(NodeId(node_id)) else {
        return;
    };
    result.extend([
        field("graph.label", "Node label", "Required label; stable node identity is not changed.", &node.label),
        field("graph.x", "Node X", "Signed 32-bit canvas coordinate.", node.x.to_string()),
        field("graph.y", "Node Y", "Signed 32-bit canvas coordinate.", node.y.to_string()),
    ]);
    for port in node.input_ports.iter().chain(&node.output_ports) {
        result.push(field(
            &format!("graph.port.{}", port.id.0),
            "Typed port",
            PORT_HELP,
            row(&[
                &port.id.0.to_string(),
                direction_name(port.direction),
                &port.label,
                &port.type_tag,
            ]),
        ));
    }
    result.push(field("graph.port.add", "+ Typed port", PORT_HELP, ""));
    if let Some(refs) = workflow.credentials.get(&node_id) {
        let mut names = refs.env.keys().collect::<Vec<_>>();
        names.sort();
        for name in names {
            let reference = &refs.env[name];
            result.push(field(&format!("credential.env.row.{name}"), "Credential reference", "name|provider|key|optional field; blank removes. Opaque validated provider reference, never secret material or provider configuration.", row(&[name, &reference.provider, &reference.key, reference.field.as_deref().unwrap_or("")])));
        }
    }
    result.push(field(
        "credential.env.add",
        "+ Credential reference",
        "name|provider|key|optional field; environment name must be an identifier. Opaque reference only.",
        "",
    ));
    if let Some(settings) = effective_runner(workflow, node_id) {
        result.push(field(
            "runner.binding",
            "Runtime binding",
            "kind|key; changes both registered binding parts atomically. Blank restores the node config binding.",
            row(&[&settings.runtime_kind, &settings.runtime_key]),
        ));
        for (id, label, help, value) in [
            (
                "runner.runtime_kind",
                "Runtime kind",
                "Registered runtime kind; blank restores this node config's binding kind.",
                settings.runtime_kind,
            ),
            (
                "runner.runtime_key",
                "Runtime key",
                "Registered runtime key; blank restores this node config's binding key.",
                settings.runtime_key,
            ),
            (
                "runner.effect_policy",
                "Effect policy",
                "Blank resets; runtime-owned process/network/filesystem/model/pure policy is validated by runner admission. No ability-list grammar exists in the portable contract.",
                settings.effect_policy,
            ),
            (
                "runner.cache_policy",
                "Cache policy",
                "disabled or blank reset. Other policies have no producer parser and fail closed.",
                settings.cache_policy,
            ),
            (
                "runner.retry_policy",
                "Retry policy",
                "none or blank reset. The current portable contract has no retry count, backoff, or named limits.",
                settings.retry_policy,
            ),
            (
                "runner.artifact_policy",
                "Artifact policy",
                "metadata-only or blank reset. No artifact limits/reference grammar is provided by the current portable contract.",
                settings.artifact_policy,
            ),
        ] {
            result.push(field(id, label, help, value));
        }
        result.push(field(
            "runner.remove",
            "Reset runner",
            "Enter remove to remove the node runner override and use its config binding; blank leaves it unchanged.",
            "",
        ));
    }
}

// r[impl lattice.tui_ux.portable_authoring_parity.triggers]
fn trigger_fields(index: usize, trigger: &TriggerDefinition, result: &mut Vec<Field>) {
    let prefix = format!("trigger.{index}.");
    let mut add = |id: &str, label: &str, help: &str, value: String| {
        result.push(field(&format!("{prefix}{id}"), label, help, value))
    };
    add("id", "Trigger id", "Required unique identity.", trigger.id.clone());
    add(
        "workflow_name",
        "Trigger workflow",
        "Must equal the current workflow name; workflow rename updates owned references.",
        trigger.workflow_name.clone(),
    );
    add(
        "enabled",
        "Trigger enabled",
        "true or false; stores definition only, never activates it.",
        trigger.enabled.to_string(),
    );
    add(
        "kind",
        "Trigger kind",
        "manual, schedule, webhook. Changing kind resets only kind-specific settings to valid deterministic defaults.",
        kind_name(&trigger.kind).into(),
    );
    add(
        "overlap_policy",
        "Overlap policy",
        "forbid (blank reset) or allow.",
        match trigger.overlap_policy {
            OverlapPolicy::Forbid => "forbid",
            OverlapPolicy::Allow => "allow",
        }
        .into(),
    );
    add(
        "idempotency_policy",
        "Idempotency",
        "none (blank reset), event_key, or payload_json_pointer|RFC6901 pointer.",
        match &trigger.idempotency_policy {
            IdempotencyPolicy::None => "none".into(),
            IdempotencyPolicy::EventKey => "event_key".into(),
            IdempotencyPolicy::PayloadJsonPointer(pointer) => row(&["payload_json_pointer", pointer]),
        },
    );
    match &trigger.kind {
        TriggerKind::Manual => {}
        TriggerKind::Schedule {
            recurrence,
            missed_run_policy,
            next_run_epoch_ms,
        } => {
            add(
                "schedule_interval_secs",
                "Interval seconds",
                "Positive integer representable in signed milliseconds; overflow rejected.",
                recurrence.interval_secs().to_string(),
            );
            add(
                "schedule_next_run_epoch_ms",
                "Next run epoch ms",
                "Signed 64-bit explicit timestamp; must leave room for one interval. No clock reads.",
                next_run_epoch_ms.to_string(),
            );
            add(
                "schedule_missed_run_policy",
                "Missed run",
                "run_once (blank reset) or skip.",
                missed_name(*missed_run_policy).into(),
            );
        }
        TriggerKind::Webhook(webhook) => {
            add(
                "webhook_method",
                "Webhook method",
                "Supported HTTP method; normalized uppercase.",
                webhook.method.clone(),
            );
            add(
                "webhook_path",
                "Webhook path",
                "Absolute local URL path beginning /; no whitespace, query, fragment or authority.",
                webhook.path.clone(),
            );
            add("auth", "Webhook auth reference", AUTH_HELP, auth_row(&webhook.auth_policy));
            for (row_index, content_type) in webhook.accepted_content_types.iter().enumerate() {
                add(
                    &format!("webhook_content_types.{row_index}"),
                    "Accepted content type",
                    "MIME type/subtype; blank removes row. Empty list accepts any content type.",
                    content_type.clone(),
                );
            }
            add(
                "webhook_content_types.add",
                "+ Content type",
                "MIME type/subtype; duplicates rejected.",
                String::new(),
            );
        }
    }
    for (binding_index, binding) in trigger.input_bindings.iter().enumerate() {
        add(
            &format!("input_bindings.{binding_index}"),
            "Trigger input binding",
            BINDING_HELP,
            binding_row(binding),
        );
    }
    add("input_bindings.add", "+ Trigger input binding", BINDING_HELP, String::new());
    add(
        "remove",
        "Remove trigger",
        "Enter remove to remove definition; blank leaves unchanged. Disabling uses enabled=false.",
        String::new(),
    );
}

// r[impl lattice.tui_ux.portable_authoring_parity.workflow_graph.scenario.invalid]
// r[impl lattice.tui_ux.portable_authoring_parity.triggers.scenario.invalid]
pub fn apply(workflow: &mut Workflow, node_id: Option<u64>, id: &str, value: &str) -> Result<(), String> {
    let mut candidate = workflow.clone();
    apply_candidate(&mut candidate, node_id, id, value)?;
    validate(&candidate)?;
    *workflow = candidate;
    Ok(())
}

fn apply_candidate(workflow: &mut Workflow, node_id: Option<u64>, id: &str, value: &str) -> Result<(), String> {
    match id {
        "workflow.name" => {
            required(value, "workflow name")?;
            let old_name = std::mem::replace(&mut workflow.meta.name, value.into());
            for trigger in &mut workflow.triggers {
                if trigger.workflow_name == old_name {
                    trigger.workflow_name = value.into();
                }
            }
            return Ok(());
        }
        "workflow.description" => {
            workflow.meta.description = value.into();
            return Ok(());
        }
        "workflow.created" | "workflow.modified" => {
            let timestamp = chrono::DateTime::parse_from_rfc3339(value)
                .map_err(|error| error.to_string())?
                .with_timezone(&chrono::Utc);
            if id == "workflow.created" {
                workflow.meta.created = timestamp;
            } else {
                workflow.meta.modified = timestamp;
            }
            return Ok(());
        }
        _ => {}
    }
    if let Some(edge_id) = id.strip_prefix("graph.edge.") {
        return edit_edge(workflow, edge_id, value);
    }
    if let Some(trigger_id) = id.strip_prefix("trigger.") {
        return edit_trigger(workflow, trigger_id, value);
    }
    let node_id = node_id.ok_or("select a node to edit this field")?;
    if workflow.graph.node(NodeId(node_id)).is_none() {
        return Err("selected node no longer exists".into());
    }
    match id {
        "graph.label" => {
            required(value, "node label")?;
            workflow.graph.node_mut(NodeId(node_id)).ok_or("node missing")?.label = value.into();
            Ok(())
        }
        "graph.x" | "graph.y" => {
            let coordinate = number::<i32>(value, "signed 32-bit coordinate")?;
            let node = workflow.graph.node_mut(NodeId(node_id)).ok_or("node missing")?;
            if id == "graph.x" {
                node.x = coordinate;
            } else {
                node.y = coordinate;
            }
            Ok(())
        }
        _ if id.starts_with("graph.port.") => edit_port(workflow, node_id, &id[11..], value),
        _ if id.starts_with("credential.env.") => edit_credential(workflow, node_id, &id[15..], value),
        _ if id.starts_with("runner.") => edit_runner(workflow, node_id, &id[7..], value),
        _ => Err(format!("unknown workflow field: {id}")),
    }
}

fn edit_port(workflow: &mut Workflow, node_id: u64, key: &str, value: &str) -> Result<(), String> {
    let old_id = if key == "add" {
        None
    } else {
        Some(PortId(number(key, "port id")?))
    };
    let node = workflow.graph.node(NodeId(node_id)).ok_or("node missing")?;
    let prior = old_id
        .map(|id| {
            node.input_ports
                .iter()
                .chain(&node.output_ports)
                .find(|port| port.id == id)
                .cloned()
                .ok_or("port is not owned by selected node")
        })
        .transpose()?;
    if value.is_empty() {
        let prior = prior.ok_or("new port cannot be blank")?;
        remove_incident(workflow, prior.id)?;
        let node = workflow.graph.node_mut(NodeId(node_id)).ok_or("node missing")?;
        node.input_ports.retain(|port| port.id != prior.id);
        node.output_ports.retain(|port| port.id != prior.id);
        return workflow.graph.restore_indexes_preserving_allocators().map_err(|error| error.to_string());
    }
    let cells = cells(value, 4)?;
    let direction = match cells[1].as_str() {
        "input" => PortDirection::Input,
        "output" => PortDirection::Output,
        _ => return Err("direction must be input or output".into()),
    };
    required(&cells[2], "port label")?;
    required(&cells[3], "port type tag")?;
    let ports = match direction {
        PortDirection::Input => &node.input_ports,
        PortDirection::Output => &node.output_ports,
    };
    if ports.iter().any(|port| Some(port.id) != old_id && port.label == cells[2]) {
        return Err("duplicate port label in direction".into());
    }
    let new_id = if cells[0] == "auto" {
        if let Some(prior) = &prior {
            prior.id
        } else {
            let mut specs = ports.iter().map(|port| PortSpec::new(&port.label, &port.type_tag)).collect::<Vec<_>>();
            specs.push(PortSpec::new(&cells[2], &cells[3]));
            let ids = workflow
                .graph
                .reconcile_node_ports(NodeId(node_id), direction, &specs)
                .map_err(|error| error.to_string())?;
            *ids.last().ok_or("port allocator returned no id")?
        }
    } else {
        let id = PortId(number(&cells[0], "port id")?);
        id.0.checked_add(1).ok_or("port id exhausts identifier space")?;
        if Some(id) != old_id && workflow.graph.port(id).is_some() {
            return Err("port id already exists".into());
        }
        id
    };
    if let Some(prior) = prior {
        if prior.id != new_id || prior.direction != direction {
            remove_incident(workflow, prior.id)?;
        }
        if prior.direction != direction {
            let node = workflow.graph.node_mut(NodeId(node_id)).ok_or("node missing")?;
            node.input_ports.retain(|port| port.id != prior.id);
            node.output_ports.retain(|port| port.id != prior.id);
        }
    }
    let node = workflow.graph.node_mut(NodeId(node_id)).ok_or("node missing")?;
    let ports = match direction {
        PortDirection::Input => &mut node.input_ports,
        PortDirection::Output => &mut node.output_ports,
    };
    let replacement = Port {
        id: new_id,
        label: cells[2].clone(),
        type_tag: cells[3].clone(),
        direction,
    };
    if let Some(existing) = ports.iter_mut().find(|port| port.id == new_id || Some(port.id) == old_id) {
        *existing = replacement;
    } else {
        ports.push(replacement);
    }
    workflow.graph.restore_indexes_preserving_allocators().map_err(|error| error.to_string())
}

fn remove_incident(workflow: &mut Workflow, port_id: PortId) -> Result<(), String> {
    let incident = workflow
        .graph
        .edges()
        .iter()
        .filter(|edge| edge.source == port_id || edge.target == port_id)
        .cloned()
        .collect::<Vec<_>>();
    for edge in incident {
        workflow.graph.remove_edge(edge.source, edge.target).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn edit_edge(workflow: &mut Workflow, key: &str, value: &str) -> Result<(), String> {
    if key != "add" {
        let (source, target) = key.split_once('.').ok_or("invalid edge identity")?;
        workflow
            .graph
            .remove_edge(PortId(number(source, "source port id")?), PortId(number(target, "target port id")?))
            .map_err(|error| error.to_string())?;
        if value.is_empty() {
            return Ok(());
        }
    }
    let cells = cells(value, 2)?;
    let source = PortId(number(&cells[0], "source port id")?);
    let target = PortId(number(&cells[1], "target port id")?);
    if workflow.graph.edges().iter().any(|edge| edge.source == source && edge.target == target) {
        return Err("duplicate edge".into());
    }
    workflow.graph.add_edge(source, target).map_err(|error| error.to_string())
}

fn edit_credential(workflow: &mut Workflow, node_id: u64, key: &str, value: &str) -> Result<(), String> {
    let adding = key == "add";
    let key = if adding {
        ""
    } else {
        key.strip_prefix("row.").ok_or("invalid credential row identity")?
    };
    if !adding && !workflow.credentials.get(&node_id).is_some_and(|refs| refs.env.contains_key(key)) {
        return Err("credential row no longer exists".into());
    }
    if value.is_empty() {
        if adding {
            return Err("new credential cannot be blank".into());
        }
        if let Some(refs) = workflow.credentials.get_mut(&node_id) {
            refs.env.remove(key);
        }
    } else {
        let cells = cells(value, 4)?;
        if !env_name(&cells[0]) {
            return Err("environment name must match [A-Za-z_][A-Za-z0-9_]*".into());
        }
        let reference = secret_ref(&cells[1], &cells[2], &cells[3])?;
        let refs = workflow.credentials.entry(node_id).or_insert_with(NodeCredentialRefs::default);
        if cells[0] != key && refs.env.contains_key(&cells[0]) {
            return Err("duplicate credential environment name".into());
        }
        if !adding {
            refs.env.remove(key);
        }
        refs.env.insert(cells[0].clone(), reference);
    }
    if workflow.credentials.get(&node_id).is_some_and(NodeCredentialRefs::is_empty) {
        workflow.credentials.remove(&node_id);
    }
    Ok(())
}

fn effective_runner(workflow: &Workflow, node_id: u64) -> Option<NodeRunnerSettings> {
    workflow.runner.get(&node_id).cloned().or_else(|| {
        workflow.configs.get(&node_id).map(|config| {
            let binding = runner::runtime_binding_for_config(config);
            NodeRunnerSettings::new(binding.kind, binding.key)
        })
    })
}

// r[impl lattice.tui_ux.portable_authoring_parity.runner]
fn edit_runner(workflow: &mut Workflow, node_id: u64, key: &str, value: &str) -> Result<(), String> {
    if key == "remove" {
        if value == "remove" {
            workflow.runner.remove(&node_id);
            return Ok(());
        }
        if value.is_empty() {
            return Ok(());
        }
        return Err("enter remove to reset runner".into());
    }
    let mut settings = effective_runner(workflow, node_id).ok_or("node config missing")?;
    match key {
        "binding" => {
            if value.is_empty() {
                let default =
                    runner::runtime_binding_for_config(workflow.configs.get(&node_id).ok_or("node config missing")?);
                settings.runtime_kind = default.kind;
                settings.runtime_key = default.key;
            } else {
                let parts = cells(value, 2)?;
                required(&parts[0], "runtime kind")?;
                required(&parts[1], "runtime key")?;
                settings.runtime_kind = parts[0].clone();
                settings.runtime_key = parts[1].clone();
            }
        }
        "runtime_kind" | "runtime_key" => {
            let config = workflow.configs.get(&node_id).ok_or("node config missing")?;
            let default = runner::runtime_binding_for_config(config);
            if key == "runtime_kind" {
                settings.runtime_kind = if value.is_empty() { default.kind } else { value.into() };
            } else {
                settings.runtime_key = if value.is_empty() { default.key } else { value.into() };
            }
        }
        "effect_policy" => settings.effect_policy = value.into(),
        "cache_policy" => {
            canonical_policy(value, "disabled")?;
            settings.cache_policy = value.into();
        }
        "retry_policy" => {
            canonical_policy(value, "none")?;
            settings.retry_policy = value.into();
        }
        "artifact_policy" => {
            canonical_policy(value, "metadata-only")?;
            settings.artifact_policy = value.into();
        }
        _ => return Err(format!("unknown runner field: {key}")),
    }
    workflow.runner.insert(node_id, settings);
    Ok(())
}

fn canonical_policy(value: &str, supported: &str) -> Result<(), String> {
    if value.is_empty() || value == supported {
        Ok(())
    } else {
        Err(format!("unsupported policy: only {supported} or blank reset has a producer contract"))
    }
}

fn edit_trigger(workflow: &mut Workflow, key: &str, value: &str) -> Result<(), String> {
    if key == "add" {
        let parts = split_row(value)?;
        let [id, kind, rest @ ..] = parts.as_slice() else {
            return Err("trigger needs id|kind and kind-specific cells".into());
        };
        required(id, "trigger id")?;
        let trigger_kind = match (kind.as_str(), rest) {
            ("manual", []) => TriggerKind::Manual,
            ("schedule", [interval, next, missed]) => TriggerKind::Schedule {
                recurrence: RecurrenceRule::EverySeconds {
                    interval_secs: interval_secs(interval)?,
                },
                missed_run_policy: parse_missed(missed)?,
                next_run_epoch_ms: number(next, "signed epoch milliseconds")?,
            },
            ("webhook", [method, path]) => TriggerKind::Webhook(WebhookTrigger {
                method: crate::http::normalized_method(method),
                path: path.clone(),
                accepted_content_types: Vec::new(),
                auth_policy: WebhookAuthPolicy::NoAuthLocalOnly,
            }),
            _ => {
                return Err(
                    "expected id|manual, id|schedule|seconds|epoch_ms|run_once or skip, or id|webhook|method|path"
                        .into(),
                );
            }
        };
        workflow.triggers.push(TriggerDefinition {
            id: id.clone(),
            workflow_name: workflow.meta.name.clone(),
            enabled: false,
            kind: trigger_kind,
            input_bindings: Vec::new(),
            overlap_policy: OverlapPolicy::Forbid,
            idempotency_policy: IdempotencyPolicy::None,
        });
        return Ok(());
    }
    let (index, property) = key.split_once('.').ok_or("trigger field requires row index")?;
    let index = number::<usize>(index, "trigger row index")?;
    if property == "remove" {
        if value.is_empty() {
            return Ok(());
        }
        if value != "remove" {
            return Err("enter remove to delete trigger".into());
        }
        if index >= workflow.triggers.len() {
            return Err("trigger no longer exists".into());
        }
        workflow.triggers.remove(index);
        return Ok(());
    }
    let trigger = workflow.triggers.get_mut(index).ok_or("trigger no longer exists")?;
    match property {
        "id" => {
            required(value, "trigger id")?;
            trigger.id = value.into();
        }
        "workflow_name" => {
            if value != workflow.meta.name {
                return Err("trigger must target its owning workflow".into());
            }
            trigger.workflow_name = value.into();
        }
        "enabled" => trigger.enabled = boolean(value)?,
        "kind" => {
            if value != kind_name(&trigger.kind) {
                trigger.kind = match value {
                    "manual" => TriggerKind::Manual,
                    "schedule" => TriggerKind::Schedule {
                        recurrence: RecurrenceRule::EverySeconds {
                            interval_secs: DEFAULT_INTERVAL_SECS,
                        },
                        missed_run_policy: MissedRunPolicy::RunOnce,
                        next_run_epoch_ms: 0,
                    },
                    "webhook" => TriggerKind::Webhook(WebhookTrigger {
                        method: "POST".into(),
                        path: "/hook".into(),
                        accepted_content_types: Vec::new(),
                        auth_policy: WebhookAuthPolicy::NoAuthLocalOnly,
                    }),
                    _ => return Err("trigger kind must be manual, schedule or webhook".into()),
                };
            }
        }
        "overlap_policy" => {
            trigger.overlap_policy = match value {
                "" | "forbid" => OverlapPolicy::Forbid,
                "allow" => OverlapPolicy::Allow,
                _ => return Err("overlap policy must be forbid or allow".into()),
            }
        }
        "idempotency_policy" => trigger.idempotency_policy = parse_idempotency(value)?,
        "schedule_interval_secs" => {
            let TriggerKind::Schedule { recurrence, .. } = &mut trigger.kind else {
                return Err("selected trigger is not scheduled".into());
            };
            *recurrence = RecurrenceRule::EverySeconds {
                interval_secs: interval_secs(value)?,
            };
        }
        "schedule_next_run_epoch_ms" => {
            let TriggerKind::Schedule { next_run_epoch_ms, .. } = &mut trigger.kind else {
                return Err("selected trigger is not scheduled".into());
            };
            *next_run_epoch_ms = number(value, "signed epoch milliseconds")?;
        }
        "schedule_missed_run_policy" => {
            let TriggerKind::Schedule { missed_run_policy, .. } = &mut trigger.kind else {
                return Err("selected trigger is not scheduled".into());
            };
            *missed_run_policy = parse_missed(value)?;
        }
        "webhook_method" => webhook_mut(trigger)?.method = crate::http::normalized_method(value),
        "webhook_path" => webhook_mut(trigger)?.path = value.into(),
        "auth" => webhook_mut(trigger)?.auth_policy = parse_auth(value)?,
        _ if property.starts_with("webhook_content_types.") => {
            let webhook = webhook_mut(trigger)?;
            edit_rows(&mut webhook.accepted_content_types, &property[22..], value, |value| Ok(value.to_string()))?;
        }
        _ if property.starts_with("input_bindings.") => {
            edit_rows(&mut trigger.input_bindings, &property[15..], value, parse_binding)?
        }
        _ => return Err(format!("unknown trigger field: {property}")),
    }
    Ok(())
}

fn edit_rows<T>(
    rows: &mut Vec<T>,
    key: &str,
    value: &str,
    parse: impl FnOnce(&str) -> Result<T, String>,
) -> Result<(), String> {
    if key == "add" {
        rows.push(parse(value)?);
    } else {
        let index = number::<usize>(key, "row index")?;
        if index >= rows.len() {
            return Err("row no longer exists".into());
        }
        if value.is_empty() {
            rows.remove(index);
        } else {
            rows[index] = parse(value)?;
        }
    }
    Ok(())
}

fn webhook_mut(trigger: &mut TriggerDefinition) -> Result<&mut WebhookTrigger, String> {
    match &mut trigger.kind {
        TriggerKind::Webhook(webhook) => Ok(webhook),
        _ => Err("selected trigger is not a webhook".into()),
    }
}

fn parse_auth(value: &str) -> Result<WebhookAuthPolicy, String> {
    if value.is_empty() || value == "none" {
        return Ok(WebhookAuthPolicy::NoAuthLocalOnly);
    }
    let parts = split_row(value)?;
    match parts.as_slice() {
        [kind, provider, key, field] if kind == "bearer" => Ok(WebhookAuthPolicy::Bearer {
            secret_ref: secret_ref(provider, key, field)?,
        }),
        [kind, provider, key, field, header] if kind == "blake3_signature" => {
            header_name(header)?;
            Ok(WebhookAuthPolicy::Blake3Signature {
                secret_ref: secret_ref(provider, key, field)?,
                header: header.clone(),
            })
        }
        _ => Err("auth must be none, bearer|provider|key|field, or blake3_signature|provider|key|field|header".into()),
    }
}

fn secret_ref(provider: &str, key: &str, field: &str) -> Result<SecretRef, String> {
    let reference = SecretRef::new(provider, key, if field.is_empty() { None } else { Some(field.into()) });
    crate::credentials::validate_secret_ref(&reference).map_err(|error| error.to_string())?;
    Ok(reference)
}

fn parse_binding(value: &str) -> Result<TriggerInputBinding, String> {
    let parts = cells(value, 5)?;
    required(&parts[0], "input label")?;
    let source = match parts[1].as_str() {
        "json_pointer" => {
            json_pointer(&parts[2])?;
            TriggerPayloadSource::JsonPointer(parts[2].clone())
        }
        "query_param" => {
            required(&parts[2], "query parameter")?;
            TriggerPayloadSource::QueryParam(parts[2].clone())
        }
        "header" => {
            header_name(&parts[2])?;
            TriggerPayloadSource::Header(parts[2].clone())
        }
        "raw_body" if parts[2].is_empty() => TriggerPayloadSource::RawBody,
        _ => return Err("invalid input source kind/name".into()),
    };
    if !matches!(parts[3].as_str(), "any" | "string" | "bytes" | "json") {
        return Err("trigger type must be any, string, bytes or json".into());
    }
    let compatible = match &source {
        TriggerPayloadSource::RawBody => matches!(parts[3].as_str(), "any" | "bytes"),
        TriggerPayloadSource::QueryParam(_) | TriggerPayloadSource::Header(_) => {
            matches!(parts[3].as_str(), "any" | "string")
        }
        TriggerPayloadSource::JsonPointer(_) => parts[3] != "bytes",
    };
    if !compatible {
        return Err("input source cannot produce the declared type".into());
    }
    Ok(TriggerInputBinding {
        input_label: parts[0].clone(),
        source,
        type_tag: parts[3].clone(),
        required: boolean(&parts[4])?,
    })
}

fn parse_idempotency(value: &str) -> Result<IdempotencyPolicy, String> {
    match value {
        "" | "none" => Ok(IdempotencyPolicy::None),
        "event_key" => Ok(IdempotencyPolicy::EventKey),
        _ => {
            let parts = cells(value, 2)?;
            if parts[0] != "payload_json_pointer" {
                return Err("unknown idempotency policy".into());
            }
            json_pointer(&parts[1])?;
            Ok(IdempotencyPolicy::PayloadJsonPointer(parts[1].clone()))
        }
    }
}

fn validate(workflow: &Workflow) -> Result<(), String> {
    workflow.validate_aggregate_ownership().map_err(|error| format!("{error:?}"))?;
    for (node_id, settings) in &workflow.runner {
        if let Some(crate::engine::workflow::NodeConfig::Operation(plan)) = workflow.configs.get(node_id)
            && (settings.runtime_kind != plan.runtime.kind || settings.runtime_key != plan.runtime.key)
        {
            return Err("accepted operation runtime binding cannot be overridden; author a new accepted plan".into());
        }
    }
    required(&workflow.meta.name, "workflow name")?;
    let diagnostics = crate::workflow_validation::validate_workflow(workflow);
    if let Some(diagnostic) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == crate::workflow_validation::DiagnosticSeverity::Fatal)
    {
        return Err(crate::workflow_validation::format_diagnostic(diagnostic));
    }
    let names = HashSet::from([workflow.meta.name.clone()]);
    let mut ids = HashSet::new();
    for trigger in &workflow.triggers {
        if !ids.insert(&trigger.id) {
            return Err("duplicate trigger id".into());
        }
        if trigger.workflow_name != workflow.meta.name {
            return Err("trigger must target its owning workflow".into());
        }
        let mut labels = HashSet::new();
        for binding in &trigger.input_bindings {
            if !labels.insert(&binding.input_label) {
                return Err("duplicate trigger input label".into());
            }
        }
        match &trigger.kind {
            TriggerKind::Manual => {}
            TriggerKind::Schedule {
                recurrence,
                missed_run_policy,
                next_run_epoch_ms,
            } => {
                let interval = interval_secs(&recurrence.interval_secs().to_string())?;
                let interval_ms = i64::try_from(interval)
                    .map_err(|_| "schedule interval overflow")?
                    .checked_mul(1_000)
                    .ok_or("schedule interval overflow")?;
                next_run_epoch_ms
                    .checked_add(interval_ms)
                    .ok_or("next schedule occurrence overflows epoch milliseconds")?;
                let schedule = ScheduleDefinition {
                    id: trigger.id.clone(),
                    workflow_name: trigger.workflow_name.clone(),
                    recurrence: *recurrence,
                    enabled: trigger.enabled,
                    overlap_policy: trigger.overlap_policy,
                    missed_run_policy: *missed_run_policy,
                    next_run_epoch_ms: *next_run_epoch_ms,
                };
                if let Some(diagnostic) = crate::scheduler::validate_schedule(&schedule, &names).first() {
                    return Err(diagnostic.to_string());
                }
            }
            TriggerKind::Webhook(webhook) => {
                if !crate::http::validate_http_method(&webhook.method) {
                    return Err("unsupported webhook HTTP method".into());
                }
                if !webhook.path.starts_with('/')
                    || webhook.path.starts_with("//")
                    || webhook.path.chars().any(|character| {
                        character.is_whitespace() || character.is_control() || matches!(character, '?' | '#')
                    })
                {
                    return Err("webhook path must be an absolute local path without query/fragment/whitespace".into());
                }
                let mut content_types = HashSet::new();
                for content_type in &webhook.accepted_content_types {
                    let Some((major, minor)) = content_type.split_once('/') else {
                        return Err("content type must be type/subtype".into());
                    };
                    if major.is_empty()
                        || minor.is_empty()
                        || content_type.chars().any(|character| character.is_whitespace() || character.is_control())
                    {
                        return Err("invalid content type".into());
                    }
                    if !content_types.insert(content_type.to_ascii_lowercase()) {
                        return Err("duplicate webhook content type".into());
                    }
                }
                if let WebhookAuthPolicy::Blake3Signature { header, .. } = &webhook.auth_policy {
                    header_name(header)?;
                }
            }
        }
    }
    Ok(())
}

fn interval_secs(value: &str) -> Result<u64, String> {
    let seconds = number::<u64>(value, "positive interval seconds")?;
    if seconds == 0 || seconds > MAX_INTERVAL_SECS {
        return Err("schedule seconds must be positive and fit signed milliseconds".into());
    }
    Ok(seconds)
}

fn parse_missed(value: &str) -> Result<MissedRunPolicy, String> {
    match value {
        "" | "run_once" => Ok(MissedRunPolicy::RunOnce),
        "skip" => Ok(MissedRunPolicy::Skip),
        _ => Err("missed-run policy must be run_once or skip".into()),
    }
}

fn boolean(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("expected true or false".into()),
    }
}

fn number<T: std::str::FromStr>(value: &str, description: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("expected {description}; invalid or overflowing number"))
}

fn required(value: &str, name: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{name} cannot be blank"))
    } else {
        Ok(())
    }
}

fn env_name(value: &str) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn header_name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_\x60|~".contains(&byte))
    {
        return Err("invalid HTTP header name".into());
    }
    Ok(())
}

fn json_pointer(value: &str) -> Result<(), String> {
    if !value.is_empty() && !value.starts_with('/') {
        return Err("JSON pointer must be empty or begin /".into());
    }
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character == '~' && !matches!(characters.next(), Some('0' | '1')) {
            return Err("JSON pointer escape must be ~0 or ~1".into());
        }
    }
    Ok(())
}

fn direction_name(direction: PortDirection) -> &'static str {
    match direction {
        PortDirection::Input => "input",
        PortDirection::Output => "output",
    }
}
fn kind_name(kind: &TriggerKind) -> &'static str {
    match kind {
        TriggerKind::Manual => "manual",
        TriggerKind::Schedule { .. } => "schedule",
        TriggerKind::Webhook(_) => "webhook",
    }
}
fn missed_name(policy: MissedRunPolicy) -> &'static str {
    match policy {
        MissedRunPolicy::RunOnce => "run_once",
        MissedRunPolicy::Skip => "skip",
    }
}

fn auth_row(auth: &WebhookAuthPolicy) -> String {
    match auth {
        WebhookAuthPolicy::NoAuthLocalOnly => "none".into(),
        WebhookAuthPolicy::Bearer { secret_ref } => row(&[
            "bearer",
            &secret_ref.provider,
            &secret_ref.key,
            secret_ref.field.as_deref().unwrap_or(""),
        ]),
        WebhookAuthPolicy::Blake3Signature { secret_ref, header } => row(&[
            "blake3_signature",
            &secret_ref.provider,
            &secret_ref.key,
            secret_ref.field.as_deref().unwrap_or(""),
            header,
        ]),
    }
}

fn binding_row(binding: &TriggerInputBinding) -> String {
    let (kind, name) = match &binding.source {
        TriggerPayloadSource::JsonPointer(name) => ("json_pointer", name.as_str()),
        TriggerPayloadSource::QueryParam(name) => ("query_param", name.as_str()),
        TriggerPayloadSource::Header(name) => ("header", name.as_str()),
        TriggerPayloadSource::RawBody => ("raw_body", ""),
    };
    row(&[
        &binding.input_label,
        kind,
        name,
        &binding.type_tag,
        if binding.required { "true" } else { "false" },
    ])
}

fn cells(value: &str, count: usize) -> Result<Vec<String>, String> {
    let parts = split_row(value)?;
    if parts.len() != count {
        return Err(format!("expected {count} pipe-separated cells. {ROW_HELP}"));
    }
    Ok(parts)
}

pub(crate) fn split_row(value: &str) -> Result<Vec<String>, String> {
    let mut parts = vec![String::new()];
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character == '|' {
            parts.push(String::new());
            continue;
        }
        let character = if character == '\\' {
            match characters.next() {
                Some('n') => '\n',
                Some('r') => '\r',
                Some('t') => '\t',
                Some('\\') => '\\',
                Some('|') => '|',
                _ => return Err("invalid row escape; use \\|, \\\\, \\n, \\r or \\t".into()),
            }
        } else {
            character
        };
        parts.last_mut().ok_or("missing row cell")?.push(character);
    }
    Ok(parts)
}

pub(crate) fn row(cells: &[&str]) -> String {
    cells
        .iter()
        .map(|cell| {
            cell.replace('\\', "\\\\")
                .replace('|', "\\|")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t")
        })
        .collect::<Vec<_>>()
        .join("|")
}

#[cfg(test)]
mod tests;
