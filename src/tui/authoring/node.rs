//! Pure typed node configuration drafts. Accepted operation plans remain immutable.
use std::collections::BTreeSet;
use std::collections::HashMap;

use super::Field;
use super::field;
use crate::control_flow::ConditionBranch;
use crate::control_flow::ConditionPredicate;
use crate::control_flow::MergePolicy;
use crate::control_flow::SubworkflowInputMapping;
use crate::control_flow::SubworkflowOutputMapping;
use crate::control_flow::{self};
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::NodeId;
use crate::engine::workflow::PortDirection;
use crate::engine::workflow::PortSpec;
use crate::engine::workflow::Workflow;
use crate::workflow::gate::*;

const OPTIONAL: &str = "Blank resets to absent; no source or credential is loaded.";
const ROW: &str = "Edit this row; blank removes it. Use the + row to add.";
const PREDICATE: &str =
    "always, exists, eq:text, contains:text, gt:finite-number, json_exists:/pointer, bool:true|false";

pub fn fields(workflow: &Workflow, node_id: u64) -> Vec<Field> {
    if workflow.graph.node(NodeId(node_id)).is_none() {
        return Vec::new();
    }
    let Some(config) = workflow.configs.get(&node_id) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let kind = crate::node_kind_provider::provider_for_config(config).portable_kind();
    let prefix = format!("config.{kind}");
    match config {
        NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            pipe_stdin,
        } => {
            scalar(
                &mut out,
                &prefix,
                "command",
                "Command",
                "Non-empty shell command; authoring never executes it.",
                command,
            );
            optional(&mut out, &prefix, "cwd", "Working directory", cwd);
            map_fields(&mut out, &format!("{prefix}.env"), env);
            scalar(
                &mut out,
                &prefix,
                "timeout_secs",
                "Timeout seconds",
                "Unsigned 64-bit seconds; 0 disables timeout.",
                timeout_secs.to_string(),
            );
            scalar(&mut out, &prefix, "pipe_stdin", "Pipe stdin", "true or false", pipe_stdin.to_string());
        }
        NodeConfig::Http {
            method,
            url,
            headers,
            body,
            timeout_secs,
        } => {
            scalar(
                &mut out,
                &prefix,
                "method",
                "HTTP method",
                "GET, POST, PUT, PATCH, DELETE, HEAD, or an expression",
                method,
            );
            scalar(&mut out, &prefix, "url", "URL", "HTTP(S) URL or expression", url);
            map_fields(&mut out, &format!("{prefix}.headers"), headers);
            optional(&mut out, &prefix, "body", "Body", body);
            scalar(
                &mut out,
                &prefix,
                "timeout_secs",
                "Timeout seconds",
                "Unsigned 64-bit seconds; 0 disables timeout.",
                timeout_secs.to_string(),
            );
        }
        NodeConfig::File {
            operation,
            path,
            content,
            overwrite,
            allowed_roots,
        } => {
            scalar(&mut out, &prefix, "operation", "File operation", "read, write, append", match operation {
                crate::file_node::FileOperationKind::Read => "read",
                crate::file_node::FileOperationKind::Write => "write",
                crate::file_node::FileOperationKind::Append => "append",
            });
            scalar(&mut out, &prefix, "path", "Path", "Non-empty lexical path; no filesystem access", path);
            optional(&mut out, &prefix, "content", "Content", content);
            scalar(&mut out, &prefix, "overwrite", "Overwrite", "true or false", overwrite.to_string());
            list_fields(&mut out, &format!("{prefix}.allowed_roots"), allowed_roots.iter().map(String::as_str));
        }
        NodeConfig::ImageInput {
            path,
            allowed_roots,
            max_bytes,
        } => {
            scalar(&mut out, &prefix, "path", "Image path", "PNG/JPEG path or expression; no filesystem access", path);
            list_fields(&mut out, &format!("{prefix}.allowed_roots"), allowed_roots.iter().map(String::as_str));
            scalar(
                &mut out,
                &prefix,
                "max_bytes",
                "Maximum bytes",
                "Positive platform-sized unsigned integer",
                max_bytes.to_string(),
            );
        }
        NodeConfig::Agent(settings) => {
            scalar(
                &mut out,
                &prefix,
                "target",
                "Agent target",
                "pi, clankers, animus",
                crate::agent_node::agent_target_label(settings.target),
            );
            scalar(&mut out, &prefix, "prompt", "Prompt", "Non-empty inline prompt; no execution", &settings.prompt);
            scalar(
                &mut out,
                &prefix,
                "prompt_file",
                "Prompt source path",
                "Bundle-relative .md path with normal components only; blank resets. Never reads the source.",
                settings.prompt_file.as_deref().unwrap_or_default(),
            );
            scalar(
                &mut out,
                &prefix,
                "output_mode",
                "Output mode",
                "text, json, image, artifact",
                settings.output_mode.as_str(),
            );
            optional(&mut out, &prefix, "model_hint", "Model hint", &settings.model_hint);
            scalar(
                &mut out,
                &prefix,
                "tool_policy",
                "Tool policy",
                "Opaque provider policy text; blank resets",
                &settings.tool_policy,
            );
            list_fields(&mut out, &format!("{prefix}.context_refs"), settings.context_refs.iter().map(String::as_str));
            optional(&mut out, &prefix, "cwd", "Working directory", &settings.cwd);
            optional(&mut out, &prefix, "session", "Session", &settings.session);
        }
        NodeConfig::Condition(config) => {
            for (index, branch) in config.branches.iter().enumerate() {
                scalar(&mut out, &prefix, &format!("branches.{index}.label"), "Branch label", ROW, &branch.label);
                scalar(
                    &mut out,
                    &prefix,
                    &format!("branches.{index}.predicate"),
                    "Predicate",
                    PREDICATE,
                    predicate_text(&branch.predicate),
                );
                scalar(
                    &mut out,
                    &prefix,
                    &format!("branches.{index}.move"),
                    "Move branch",
                    "Zero-based destination row; preserves branch identity",
                    index.to_string(),
                );
            }
            scalar(&mut out, &prefix, "branches.add", "+ branch", "label=predicate; split at first =", "");
        }
        NodeConfig::Merge(config) => {
            scalar(&mut out, &prefix, "policy", "Merge policy", "all, first, last, collect", match config.policy {
                MergePolicy::All => "all",
                MergePolicy::First => "first",
                MergePolicy::Last => "last",
                MergePolicy::Collect => "collect",
            });
            list_fields(
                &mut out,
                &format!("{prefix}.required_inputs"),
                config.required_inputs.iter().map(String::as_str),
            );
        }
        NodeConfig::Subworkflow(config) => {
            scalar(
                &mut out,
                &prefix,
                "workflow_name",
                "Child workflow",
                "Non-empty reference; authoring never loads the workflow",
                &config.workflow_name,
            );
            for (index, mapping) in config.input_mappings.iter().enumerate() {
                scalar(
                    &mut out,
                    &prefix,
                    &format!("input_mappings.{index}.parent_input"),
                    "Parent input",
                    ROW,
                    &mapping.parent_input,
                );
                scalar(
                    &mut out,
                    &prefix,
                    &format!("input_mappings.{index}.child_input"),
                    "Child input",
                    ROW,
                    &mapping.child_input,
                );
            }
            scalar(&mut out, &prefix, "input_mappings.add", "+ input mapping", "parent-input=child-input", "");
            for (index, mapping) in config.output_mappings.iter().enumerate() {
                scalar(
                    &mut out,
                    &prefix,
                    &format!("output_mappings.{index}.child_output"),
                    "Child output",
                    ROW,
                    &mapping.child_output,
                );
                scalar(
                    &mut out,
                    &prefix,
                    &format!("output_mappings.{index}.parent_output"),
                    "Parent output",
                    ROW,
                    &mapping.parent_output,
                );
            }
            scalar(&mut out, &prefix, "output_mappings.add", "+ output mapping", "child-output=parent-output", "");
        }
        NodeConfig::Approval(config) => approval_fields(&mut out, &prefix, config),
        NodeConfig::Operation(_) => {}
    }
    out
}

fn scalar(out: &mut Vec<Field>, prefix: &str, id: &str, label: &str, help: &str, value: impl Into<String>) {
    out.push(field(&format!("{prefix}.{id}"), label, help, value));
}
fn optional(out: &mut Vec<Field>, prefix: &str, id: &str, label: &str, value: &Option<String>) {
    scalar(out, prefix, id, label, OPTIONAL, value.as_deref().unwrap_or_default());
}
fn list_fields<'a>(out: &mut Vec<Field>, prefix: &str, values: impl Iterator<Item = &'a str>) {
    for (index, value) in values.enumerate() {
        out.push(field(&format!("{prefix}.{index}"), prefix, ROW, value));
        out.push(field(
            &format!("{prefix}.{index}.move"),
            "Move row",
            "Zero-based destination row",
            index.to_string(),
        ));
    }
    out.push(field(&format!("{prefix}.add"), &format!("+ {prefix}"), "Add one non-empty unique row", ""));
}
fn map_fields(out: &mut Vec<Field>, prefix: &str, values: &HashMap<String, String>) {
    let mut keys = values.keys().collect::<Vec<_>>();
    keys.sort();
    for (index, key) in keys.into_iter().enumerate() {
        out.push(field(
            &format!("{prefix}.{index}.key"),
            "Key",
            "Blank removes this row; duplicate keys rejected",
            key,
        ));
        out.push(field(
            &format!("{prefix}.{index}.value"),
            "Value",
            "Value text (including empty); use key field to remove",
            &values[key],
        ));
    }
    out.push(field(
        &format!("{prefix}.add"),
        &format!("+ {prefix}"),
        "key=value; first = separates key from value",
        "",
    ));
}
fn predicate_text(value: &ConditionPredicate) -> String {
    match value {
        ConditionPredicate::Always => "always".into(),
        ConditionPredicate::Exists => "exists".into(),
        ConditionPredicate::StringEquals(value) => format!("eq:{value}"),
        ConditionPredicate::StringContains(value) => format!("contains:{value}"),
        ConditionPredicate::NumberGreaterThan(value) => format!("gt:{value}"),
        ConditionPredicate::JsonPointerExists(value) => format!("json_exists:{value}"),
        ConditionPredicate::BoolEquals(value) => format!("bool:{value}"),
    }
}

fn approval_fields(out: &mut Vec<Field>, prefix: &str, config: &ApprovalNodeConfig) {
    scalar(
        out,
        prefix,
        "gate_id",
        "Gate identity",
        "Checked non-empty identity, unique within workflow",
        config.gate_id.as_str(),
    );
    scalar(out, prefix, "safe_summary", "Safe summary", "Non-empty; at most 512 bytes", &config.safe_summary);
    list_fields(
        out,
        &format!("{prefix}.selected_input_ports"),
        config.selected_input_ports.iter().map(String::as_str),
    );
    list_fields(
        out,
        &format!("{prefix}.required_artifacts"),
        config.required_artifacts.iter().map(ApprovalArtifactId::as_str),
    );
    scalar(
        out,
        prefix,
        "authority.profile",
        "Authority profile",
        "local_operator|actor or basalt_ucan_verified|policy-ref transitions the authority policy atomically; current profile alone preserves its facts",
        match config.authority.profile {
            AuthorityProfile::LocalOperator => "local_operator",
            AuthorityProfile::BasaltUcanVerified => "basalt_ucan_verified",
        },
    );
    list_fields(
        out,
        &format!("{prefix}.authority.local_actors"),
        config.authority.local_actors.iter().map(ActorId::as_str),
    );
    list_fields(
        out,
        &format!("{prefix}.authority.accepted_abilities"),
        config.authority.accepted_abilities.iter().map(|ability| match ability {
            DecisionAbility::Approve => "approve",
            DecisionAbility::Reject => "reject",
        }),
    );
    scalar(
        out,
        prefix,
        "authority.prohibit_self_approval",
        "Prohibit self approval",
        "true or false",
        config.authority.prohibit_self_approval.to_string(),
    );
    optional(
        out,
        prefix,
        "authority.delegated_policy_reference",
        "Delegated policy reference",
        &config.authority.delegated_policy_reference,
    );
    scalar(
        out,
        prefix,
        "expiry.decision_ttl_millis",
        "Decision TTL milliseconds",
        "1 through 31622400000",
        config.expiry.decision_ttl_millis.to_string(),
    );
    scalar(
        out,
        prefix,
        "expiry.resume_ttl_millis",
        "Resume TTL milliseconds",
        "1 through 31622400000",
        config.expiry.resume_ttl_millis.to_string(),
    );
    scalar(
        out,
        prefix,
        "expiry.approval_must_be_fresh_at_resume",
        "Fresh approval at resume",
        "true or false",
        config.expiry.approval_must_be_fresh_at_resume.to_string(),
    );
    scalar(out, prefix, "expiry.clock_source", "Clock source", "utc_wall_clock", "utc_wall_clock");
    scalar(
        out,
        prefix,
        "reason.required",
        "Reason required",
        "true or false",
        config.reason.required.to_string(),
    );
    scalar(
        out,
        prefix,
        "reason.identity_bound",
        "Reason identity bound",
        "true or false",
        config.reason.identity_bound.to_string(),
    );
    scalar(
        out,
        prefix,
        "reason.max_bytes",
        "Reason maximum bytes",
        "1 through 4096",
        config.reason.max_bytes.to_string(),
    );
    for (index, value) in config.resume.values.iter().enumerate() {
        scalar(out, prefix, &format!("resume.values.{index}.name"), "Resume value name", ROW, value.name.as_str());
        scalar(
            out,
            prefix,
            &format!("resume.values.{index}.type_tag"),
            "Resume value type",
            "string, bytes, json, image, exitcode, any",
            &value.type_tag,
        );
    }
    scalar(out, prefix, "resume.values.add", "+ resume value", "name=type-tag", "");
    list_fields(
        out,
        &format!("{prefix}.resume.credential_bindings"),
        config.resume.credential_bindings.iter().map(ResumeBindingName::as_str),
    );
    scalar(
        out,
        prefix,
        "terminal.on_rejection",
        "On rejection",
        "fail_workflow or cancel_workflow",
        terminal_text(config.terminal.on_rejection),
    );
    scalar(
        out,
        prefix,
        "terminal.on_expiry",
        "On expiry",
        "fail_workflow or cancel_workflow",
        terminal_text(config.terminal.on_expiry),
    );
    list_fields(out, &format!("{prefix}.required_non_claims"), config.required_non_claims.iter().map(String::as_str));
    scalar(
        out,
        prefix,
        "required_non_claims.reset",
        "Reset non-claims",
        "Enter reset to restore the exact reviewed invariant set",
        "",
    );
}
fn terminal_text(value: TerminalOutcome) -> &'static str {
    match value {
        TerminalOutcome::FailWorkflow => "fail_workflow",
        TerminalOutcome::CancelWorkflow => "cancel_workflow",
    }
}

/// Apply only after parsing, provider validation and topology reconciliation succeed.
/// Files, workflow references, prompt sources and credentials are never resolved here.
pub fn apply(workflow: &mut Workflow, node_id: u64, id: &str, value: &str) -> Result<(), String> {
    let node = workflow.graph.node(NodeId(node_id)).ok_or("Node no longer exists")?;
    let mut config = workflow.configs.get(&node_id).ok_or("Node configuration no longer exists")?.clone();
    let provider = crate::node_kind_provider::provider_for_config(&config);
    let prefix = format!("config.{}.", provider.portable_kind());
    let key = id.strip_prefix(&prefix).ok_or("Field does not belong to this node kind")?;
    edit_config(&mut config, key, value)?;
    let issues = provider.validate_config(&node.label, &config);
    if !issues.is_empty() {
        return Err(issues.into_iter().map(|issue| issue.message).collect::<Vec<_>>().join("; "));
    }
    validate_nested(&config)?;
    if let NodeConfig::Approval(approval) = &config {
        if workflow.configs.iter().any(|(other, config)| {
            *other != node_id && matches!(config, NodeConfig::Approval(other) if other.gate_id == approval.gate_id)
        }) {
            return Err("Approval gate identity already exists".into());
        }
    }
    let mut candidate = workflow.clone();
    reconcile(&mut candidate, node_id, &config, key)?;
    candidate.configs.insert(node_id, config);
    *workflow = candidate;
    Ok(())
}

fn edit_config(config: &mut NodeConfig, key: &str, value: &str) -> Result<(), String> {
    match config {
        NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            pipe_stdin,
        } => match key {
            "command" => *command = required(value)?,
            "cwd" => *cwd = optional_path(value)?,
            "timeout_secs" => *timeout_secs = number(value)?,
            "pipe_stdin" => *pipe_stdin = boolean(value)?,
            _ => edit_map(env, key.strip_prefix("env.").ok_or("Unknown shell field")?, value, false)?,
        },
        NodeConfig::Http {
            method,
            url,
            headers,
            body,
            timeout_secs,
        } => match key {
            "method" => {
                *method = required(value)?;
                if !crate::expressions::contains_expression(value) {
                    *method = crate::http::normalized_method(value);
                }
            }
            "url" => *url = required(value)?,
            "body" => *body = optional_text(value),
            "timeout_secs" => *timeout_secs = number(value)?,
            _ => edit_map(headers, key.strip_prefix("headers.").ok_or("Unknown HTTP field")?, value, true)?,
        },
        NodeConfig::File {
            operation,
            path,
            content,
            overwrite,
            allowed_roots,
        } => match key {
            "operation" => {
                *operation = match value {
                    "read" => crate::file_node::FileOperationKind::Read,
                    "write" => crate::file_node::FileOperationKind::Write,
                    "append" => crate::file_node::FileOperationKind::Append,
                    _ => return Err("Expected read, write or append".into()),
                }
            }
            "path" => *path = path_text(value)?,
            "content" => *content = optional_text(value),
            "overwrite" => *overwrite = boolean(value)?,
            _ => edit_list(
                allowed_roots,
                key.strip_prefix("allowed_roots.").ok_or("Unknown file field")?,
                value,
                path_text,
            )?,
        },
        NodeConfig::ImageInput {
            path,
            allowed_roots,
            max_bytes,
        } => match key {
            "path" => *path = path_text(value)?,
            "max_bytes" => *max_bytes = positive(value)?,
            _ => edit_list(
                allowed_roots,
                key.strip_prefix("allowed_roots.").ok_or("Unknown image field")?,
                value,
                path_text,
            )?,
        },
        NodeConfig::Agent(settings) => match key {
            "target" => {
                settings.target = crate::agent_node::parse_agent_target(value).map_err(|error| error.to_string())?
            }
            "prompt" => settings.prompt = required(value)?,
            "prompt_file" => settings.prompt_file = optional_prompt_path(value)?,
            "output_mode" => {
                settings.output_mode =
                    crate::agent_node::parse_agent_output_mode(value).map_err(|error| error.to_string())?
            }
            "model_hint" => settings.model_hint = optional_text(value),
            "tool_policy" => settings.tool_policy = value.into(),
            "cwd" => settings.cwd = optional_path(value)?,
            "session" => settings.session = optional_text(value),
            _ => edit_list(
                &mut settings.context_refs,
                key.strip_prefix("context_refs.").ok_or("Unknown agent field")?,
                value,
                required,
            )?,
        },
        NodeConfig::Condition(config) => {
            edit_branches(&mut config.branches, key.strip_prefix("branches.").ok_or("Unknown condition field")?, value)?
        }
        NodeConfig::Merge(config) => match key {
            "policy" => {
                config.policy = match value {
                    "all" => MergePolicy::All,
                    "first" => MergePolicy::First,
                    "last" => MergePolicy::Last,
                    "collect" => MergePolicy::Collect,
                    _ => return Err("Expected all, first, last or collect".into()),
                }
            }
            _ => edit_list(
                &mut config.required_inputs,
                key.strip_prefix("required_inputs.").ok_or("Unknown merge field")?,
                value,
                required,
            )?,
        },
        NodeConfig::Subworkflow(config) => edit_subworkflow(config, key, value)?,
        NodeConfig::Approval(config) => edit_approval(config, key, value)?,
        NodeConfig::Operation(_) => {
            return Err(
                "Accepted operation plans are immutable; select a newly admitted plan outside the pure editor".into()
            );
        }
    }
    Ok(())
}
fn required(value: &str) -> Result<String, String> {
    if value.trim().is_empty() || value.contains('\0') {
        Err("Value must be non-empty and contain no NUL".into())
    } else {
        Ok(value.into())
    }
}
fn optional_text(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.into())
}
pub(crate) fn path_text(value: &str) -> Result<String, String> {
    let value = required(value)?;
    if std::path::Path::new(&value)
        .components()
        .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err("Parent traversal is not an authorable path".into());
    }
    Ok(value)
}
fn optional_path(value: &str) -> Result<Option<String>, String> {
    if value.is_empty() {
        Ok(None)
    } else {
        path_text(value).map(Some)
    }
}
fn optional_prompt_path(value: &str) -> Result<Option<String>, String> {
    if value.is_empty() {
        return Ok(None);
    }
    required(value)?;
    let path = std::path::Path::new(value);
    if path.is_absolute()
        || path.extension().and_then(|extension| extension.to_str()) != Some("md")
        || path.components().any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err("Prompt source must be a bundle-relative .md path with normal components only".into());
    }
    Ok(Some(value.into()))
}
fn boolean(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("Expected true or false".into()),
    }
}
fn number<T: std::str::FromStr>(value: &str) -> Result<T, String> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Expected unsigned decimal integer".into());
    }
    value.parse().map_err(|_| "Integer exceeds supported bound".into())
}
fn positive(value: &str) -> Result<usize, String> {
    let parsed = number(value)?;
    if parsed == 0 {
        Err("Value must be positive".into())
    } else {
        Ok(parsed)
    }
}
fn pair(value: &str) -> Result<(&str, &str), String> {
    let (left, right) = value.split_once('=').ok_or("Expected left=right row syntax")?;
    required(left)?;
    Ok((left, right))
}
fn index(value: &str, length: usize) -> Result<usize, String> {
    let index = number(value)?;
    if index >= length {
        Err("Row no longer exists".into())
    } else {
        Ok(index)
    }
}
fn row_leaf(key: &str, length: usize) -> Result<(usize, &str), String> {
    let (row, leaf) = key.split_once('.').ok_or("Expected row index and field")?;
    Ok((index(row, length)?, leaf))
}
fn edit_list<T: PartialEq>(
    values: &mut Vec<T>,
    key: &str,
    value: &str,
    parse: impl Fn(&str) -> Result<T, String>,
) -> Result<(), String> {
    if let Some(row) = key.strip_suffix(".move") {
        let source = index(row, values.len())?;
        let destination = index(value, values.len())?;
        let item = values.remove(source);
        values.insert(destination, item);
        return Ok(());
    }
    if key == "add" {
        let item = parse(value)?;
        if values.contains(&item) {
            return Err("Duplicate row".into());
        }
        values.push(item);
    } else {
        let row = index(key, values.len())?;
        if value.is_empty() {
            values.remove(row);
        } else {
            let item = parse(value)?;
            if values.iter().enumerate().any(|(other, prior)| other != row && prior == &item) {
                return Err("Duplicate row".into());
            }
            values[row] = item;
        }
    }
    Ok(())
}
fn map_key(value: &str, header: bool) -> Result<String, String> {
    required(value)?;
    if header {
        if !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)) {
            return Err("Invalid HTTP header name".into());
        }
    } else if value.contains('=') || value.chars().any(char::is_control) {
        return Err("Invalid environment variable name".into());
    }
    Ok(value.into())
}
fn edit_map(values: &mut HashMap<String, String>, key: &str, value: &str, header: bool) -> Result<(), String> {
    let duplicate = |values: &HashMap<String, String>, candidate: &str, except: Option<&str>| {
        values.keys().any(|key| {
            Some(key.as_str()) != except
                && if header {
                    key.eq_ignore_ascii_case(candidate)
                } else {
                    key == candidate
                }
        })
    };
    let validate_value = |value: &str| {
        if value.contains('\0') || header && value.contains(['\r', '\n']) {
            Err("Invalid map value".to_string())
        } else {
            Ok(())
        }
    };
    if key == "add" {
        let (name, content) = pair(value)?;
        let name = map_key(name, header)?;
        validate_value(content)?;
        if duplicate(values, &name, None) {
            return Err("Duplicate map key".into());
        }
        values.insert(name, content.into());
    } else {
        let mut keys = values.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        let (row, leaf) = row_leaf(key, keys.len())?;
        let prior = &keys[row];
        match leaf {
            "key" if value.is_empty() => {
                values.remove(prior);
            }
            "key" => {
                let name = map_key(value, header)?;
                if duplicate(values, &name, Some(prior)) {
                    return Err("Duplicate map key".into());
                }
                let content = values.remove(prior).ok_or("Map row no longer exists")?;
                values.insert(name, content);
            }
            "value" => {
                validate_value(value)?;
                values.insert(prior.clone(), value.into());
            }
            _ => return Err("Unknown map row field".into()),
        }
    }
    Ok(())
}

fn edit_branches(branches: &mut Vec<ConditionBranch>, key: &str, value: &str) -> Result<(), String> {
    if key == "add" {
        let (label, predicate) = pair(value)?;
        branches.push(ConditionBranch {
            label: required(label)?,
            predicate: control_flow::parse_condition_predicate(predicate).map_err(|error| error.to_string())?,
        });
    } else {
        let (row, leaf) = row_leaf(key, branches.len())?;
        match leaf {
            "label" if value.is_empty() => {
                branches.remove(row);
            }
            "label" => branches[row].label = required(value)?,
            "predicate" => {
                branches[row].predicate =
                    control_flow::parse_condition_predicate(value).map_err(|error| error.to_string())?
            }
            "move" => {
                let destination = index(value, branches.len())?;
                let branch = branches.remove(row);
                branches.insert(destination, branch);
            }
            _ => return Err("Unknown branch field".into()),
        }
    }
    Ok(())
}
fn edit_subworkflow(config: &mut control_flow::SubworkflowNodeConfig, key: &str, value: &str) -> Result<(), String> {
    if key == "workflow_name" {
        config.workflow_name = required(value)?;
    } else if let Some(key) = key.strip_prefix("input_mappings.") {
        if key == "add" {
            let (parent, child) = pair(value)?;
            config.input_mappings.push(SubworkflowInputMapping {
                parent_input: required(parent)?,
                child_input: required(child)?,
            });
        } else {
            let (row, leaf) = row_leaf(key, config.input_mappings.len())?;
            if !matches!(leaf, "parent_input" | "child_input") {
                return Err("Unknown input mapping field".into());
            }
            if value.is_empty() {
                config.input_mappings.remove(row);
            } else if leaf == "parent_input" {
                config.input_mappings[row].parent_input = required(value)?;
            } else {
                config.input_mappings[row].child_input = required(value)?;
            }
        }
    } else if let Some(key) = key.strip_prefix("output_mappings.") {
        if key == "add" {
            let (child, parent) = pair(value)?;
            config.output_mappings.push(SubworkflowOutputMapping {
                child_output: required(child)?,
                parent_output: required(parent)?,
            });
        } else {
            let (row, leaf) = row_leaf(key, config.output_mappings.len())?;
            if !matches!(leaf, "child_output" | "parent_output") {
                return Err("Unknown output mapping field".into());
            }
            if value.is_empty() {
                config.output_mappings.remove(row);
            } else if leaf == "parent_output" {
                config.output_mappings[row].parent_output = required(value)?;
            } else {
                config.output_mappings[row].child_output = required(value)?;
            }
        }
    } else {
        return Err("Unknown subworkflow field".into());
    }
    Ok(())
}
fn edit_approval(config: &mut ApprovalNodeConfig, key: &str, value: &str) -> Result<(), String> {
    match key {
        "gate_id" => config.gate_id = GateId::try_from_text(value).map_err(|error| error.to_string())?,
        "safe_summary" => config.safe_summary = required(value)?,
        "authority.profile" => {
            let (profile, fact) = value.split_once('|').map_or((value, None), |(profile, fact)| (profile, Some(fact)));
            match profile {
                "local_operator" => {
                    if let Some(actor) = fact {
                        config.authority.local_actors =
                            vec![ActorId::try_from_text(actor).map_err(|error| error.to_string())?];
                        config.authority.delegated_policy_reference = None;
                    }
                    config.authority.profile = AuthorityProfile::LocalOperator;
                }
                "basalt_ucan_verified" => {
                    if let Some(reference) = fact {
                        config.authority.delegated_policy_reference = Some(required(reference)?);
                        config.authority.local_actors.clear();
                    }
                    config.authority.profile = AuthorityProfile::BasaltUcanVerified;
                }
                _ => return Err("Unknown authority profile".into()),
            }
        }
        "authority.prohibit_self_approval" => config.authority.prohibit_self_approval = boolean(value)?,
        "authority.delegated_policy_reference" => config.authority.delegated_policy_reference = optional_text(value),
        "expiry.decision_ttl_millis" => config.expiry.decision_ttl_millis = number(value)?,
        "expiry.resume_ttl_millis" => config.expiry.resume_ttl_millis = number(value)?,
        "expiry.approval_must_be_fresh_at_resume" => config.expiry.approval_must_be_fresh_at_resume = boolean(value)?,
        "expiry.clock_source" => {
            config.expiry.clock_source = match value {
                "utc_wall_clock" => ClockSourceClass::UtcWallClock,
                _ => return Err("Expected utc_wall_clock".into()),
            }
        }
        "reason.required" => config.reason.required = boolean(value)?,
        "reason.identity_bound" => config.reason.identity_bound = boolean(value)?,
        "reason.max_bytes" => config.reason.max_bytes = positive(value)?,
        "terminal.on_rejection" => config.terminal.on_rejection = parse_terminal(value)?,
        "terminal.on_expiry" => config.terminal.on_expiry = parse_terminal(value)?,
        "required_non_claims.reset" => {
            if value != "reset" {
                return Err("Enter reset to restore reviewed non-claims".into());
            }
            config.required_non_claims = REQUIRED_APPROVAL_NON_CLAIMS.map(str::to_owned).to_vec();
        }
        _ => {
            if let Some(key) = key.strip_prefix("selected_input_ports.") {
                edit_list(&mut config.selected_input_ports, key, value, required)?;
            } else if let Some(key) = key.strip_prefix("required_artifacts.") {
                edit_list(&mut config.required_artifacts, key, value, |value| {
                    ApprovalArtifactId::try_from_hex(value).map_err(|error| error.to_string())
                })?;
            } else if let Some(key) = key.strip_prefix("authority.local_actors.") {
                edit_list(&mut config.authority.local_actors, key, value, |value| {
                    ActorId::try_from_text(value).map_err(|error| error.to_string())
                })?;
            } else if let Some(key) = key.strip_prefix("authority.accepted_abilities.") {
                edit_list(&mut config.authority.accepted_abilities, key, value, |value| match value {
                    "approve" => Ok(DecisionAbility::Approve),
                    "reject" => Ok(DecisionAbility::Reject),
                    _ => Err("Expected approve or reject".into()),
                })?;
            } else if let Some(key) = key.strip_prefix("resume.credential_bindings.") {
                edit_list(&mut config.resume.credential_bindings, key, value, |value| {
                    ResumeBindingName::try_from_text(value).map_err(|error| error.to_string())
                })?;
            } else if let Some(key) = key.strip_prefix("required_non_claims.") {
                edit_list(&mut config.required_non_claims, key, value, required)?;
            } else if let Some(key) = key.strip_prefix("resume.values.") {
                if key == "add" {
                    let (name, type_tag) = pair(value)?;
                    config.resume.values.push(ResumeValueField {
                        name: ResumeBindingName::try_from_text(name).map_err(|error| error.to_string())?,
                        type_tag: required(type_tag)?,
                    });
                } else {
                    let (row, leaf) = row_leaf(key, config.resume.values.len())?;
                    match leaf {
                        "name" if value.is_empty() => {
                            config.resume.values.remove(row);
                        }
                        "name" => {
                            config.resume.values[row].name =
                                ResumeBindingName::try_from_text(value).map_err(|error| error.to_string())?
                        }
                        "type_tag" => config.resume.values[row].type_tag = required(value)?,
                        _ => return Err("Unknown resume value field".into()),
                    }
                }
            } else {
                return Err("Unknown approval field".into());
            }
        }
    }
    Ok(())
}
fn parse_terminal(value: &str) -> Result<TerminalOutcome, String> {
    match value {
        "fail_workflow" => Ok(TerminalOutcome::FailWorkflow),
        "cancel_workflow" => Ok(TerminalOutcome::CancelWorkflow),
        _ => Err("Expected fail_workflow or cancel_workflow".into()),
    }
}
fn unique<'a>(values: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err("Duplicate row endpoint".into());
        }
    }
    Ok(())
}
fn validate_nested(config: &NodeConfig) -> Result<(), String> {
    match config {
        NodeConfig::Subworkflow(config) => {
            unique(config.input_mappings.iter().map(|mapping| mapping.parent_input.as_str()))?;
            unique(config.input_mappings.iter().map(|mapping| mapping.child_input.as_str()))?;
            unique(config.output_mappings.iter().map(|mapping| mapping.parent_output.as_str()))?;
            unique(config.output_mappings.iter().map(|mapping| mapping.child_output.as_str()))?;
        }
        NodeConfig::Approval(config) => unique(config.required_non_claims.iter().map(String::as_str))?,
        _ => {}
    }
    Ok(())
}
fn reconcile(workflow: &mut Workflow, node_id: u64, config: &NodeConfig, key: &str) -> Result<(), String> {
    match config {
        NodeConfig::Condition(config) => workflow
            .replace_condition_branches(NodeId(node_id), config.branches.clone())
            .map_err(|error| format!("{error:?}"))?,
        NodeConfig::Merge(config) if key.starts_with("required_inputs.") => {
            reconcile_labels(workflow, node_id, PortDirection::Input, &config.required_inputs)?
        }
        NodeConfig::Subworkflow(config) if key.starts_with("input_mappings.") => {
            let inputs = config.input_mappings.iter().map(|mapping| mapping.parent_input.clone()).collect::<Vec<_>>();
            reconcile_labels(workflow, node_id, PortDirection::Input, &inputs)?;
        }
        NodeConfig::Subworkflow(config) if key.starts_with("output_mappings.") => {
            let outputs = if config.output_mappings.is_empty() {
                vec![control_flow::subworkflow_default_output_port_label().into()]
            } else {
                config.output_mappings.iter().map(|mapping| mapping.parent_output.clone()).collect()
            };
            reconcile_labels(workflow, node_id, PortDirection::Output, &outputs)?;
        }
        NodeConfig::Approval(config) if key.starts_with("selected_input_ports.") => {
            reconcile_labels(workflow, node_id, PortDirection::Input, &config.selected_input_ports)?;
        }
        NodeConfig::Approval(config) if key.starts_with("resume.") => {
            let mut outputs = config
                .resume
                .values
                .iter()
                .map(|value| PortSpec::new(value.name.as_str(), &value.type_tag))
                .collect::<Vec<_>>();
            outputs.extend(
                config
                    .resume
                    .credential_bindings
                    .iter()
                    .map(|name| PortSpec::new(format!("credential:{name}"), "string")),
            );
            workflow
                .graph
                .reconcile_node_ports(NodeId(node_id), PortDirection::Output, &outputs)
                .map_err(|error| error.to_string())?;
        }
        _ => {}
    }
    Ok(())
}
fn reconcile_labels(
    workflow: &mut Workflow,
    node_id: u64,
    direction: PortDirection,
    labels: &[String],
) -> Result<(), String> {
    let node = workflow.graph.node(NodeId(node_id)).ok_or("Node no longer exists")?;
    let prior = match direction {
        PortDirection::Input => &node.input_ports,
        PortDirection::Output => &node.output_ports,
    };
    let specs = labels
        .iter()
        .map(|label| {
            PortSpec::new(
                label,
                prior.iter().find(|port| &port.label == label).map_or("any", |port| port.type_tag.as_str()),
            )
        })
        .collect::<Vec<_>>();
    workflow
        .graph
        .reconcile_node_ports(NodeId(node_id), direction, &specs)
        .map_err(|error| error.to_string())?;
    Ok(())
}

/// Select an already admitted plan with its reproducible catalog/instance evidence.
/// This does not manufacture an invocation identity or resolve any authority.
pub fn replace_operation_plan(
    workflow: &mut Workflow,
    node_id: u64,
    plan: crate::operation::InvocationPlan,
    catalog: &crate::operation::ValidatedCatalog,
    instance: &crate::operation::ValidatedInstance,
) -> Result<(), String> {
    if workflow.graph.node(NodeId(node_id)).is_none()
        || !matches!(workflow.configs.get(&node_id), Some(NodeConfig::Operation(_)))
    {
        return Err("Operation node no longer exists".into());
    }
    crate::operation::planning::validate_plan_admission(&plan, catalog, instance).map_err(|error| error.to_string())?;
    workflow.configs.insert(node_id, NodeConfig::operation(plan));
    Ok(())
}

/// Create a local-operator declaration from explicitly supplied actor facts.
pub fn create_approval(workflow: &mut Workflow, label: &str, gate_id: &str, actor: &str) -> Result<u64, String> {
    const DEFAULT_APPROVAL_TTL_MILLIS: u64 = 60_000;
    const DEFAULT_REASON_MAX_BYTES: usize = 128;
    let label = required(label)?;
    let config = ApprovalNodeConfig {
        gate_id: GateId::try_from_text(gate_id).map_err(|error| error.to_string())?,
        safe_summary: format!("Review {gate_id}"),
        selected_input_ports: Vec::new(),
        required_artifacts: Vec::new(),
        authority: ApprovalAuthorityPolicy {
            profile: AuthorityProfile::LocalOperator,
            local_actors: vec![ActorId::try_from_text(actor).map_err(|error| error.to_string())?],
            accepted_abilities: vec![DecisionAbility::Approve, DecisionAbility::Reject],
            prohibit_self_approval: true,
            delegated_policy_reference: None,
        },
        expiry: ApprovalExpiryPolicy {
            decision_ttl_millis: DEFAULT_APPROVAL_TTL_MILLIS,
            resume_ttl_millis: DEFAULT_APPROVAL_TTL_MILLIS,
            approval_must_be_fresh_at_resume: true,
            clock_source: ClockSourceClass::UtcWallClock,
        },
        reason: ApprovalReasonPolicy {
            required: false,
            identity_bound: true,
            max_bytes: DEFAULT_REASON_MAX_BYTES,
        },
        resume: ApprovalResumeSchema {
            values: Vec::new(),
            credential_bindings: Vec::new(),
        },
        terminal: ApprovalTerminalPolicy {
            on_rejection: TerminalOutcome::FailWorkflow,
            on_expiry: TerminalOutcome::CancelWorkflow,
        },
        required_non_claims: REQUIRED_APPROVAL_NON_CLAIMS.map(str::to_owned).to_vec(),
    };
    if workflow
        .configs
        .values()
        .any(|other| matches!(other, NodeConfig::Approval(other) if other.gate_id == config.gate_id))
    {
        return Err("Approval gate identity already exists".into());
    }
    let issues = crate::workflow::gate::core::validate_node_config(&config);
    if !issues.is_empty() {
        return Err(issues.into_iter().map(|issue| issue.message).collect::<Vec<_>>().join("; "));
    }
    let mut candidate = workflow.clone();
    let node_id = candidate
        .try_add_configured_node(label, NodeConfig::Approval(config))
        .map_err(|error| format!("{error:?}"))?;
    *workflow = candidate;
    Ok(node_id.0)
}

/// Create from an admitted plan without synthesizing authority or identity.
pub fn create_operation(
    workflow: &mut Workflow,
    label: &str,
    plan: crate::operation::InvocationPlan,
    catalog: &crate::operation::ValidatedCatalog,
    instance: &crate::operation::ValidatedInstance,
) -> Result<u64, String> {
    let label = required(label)?;
    crate::operation::planning::validate_plan_admission(&plan, catalog, instance).map_err(|error| error.to_string())?;
    let mut candidate = workflow.clone();
    let node_id = candidate
        .try_add_configured_node(label, NodeConfig::operation(plan))
        .map_err(|error| format!("{error:?}"))?;
    *workflow = candidate;
    Ok(node_id.0)
}

#[cfg(test)]
mod tests;
