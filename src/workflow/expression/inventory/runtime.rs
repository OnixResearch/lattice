//! Exhaustive inventory of expression-bearing runtime workflow fields.

const SHELL_SCALAR_FIELD_COUNT: usize = 2;
const HTTP_SCALAR_FIELD_COUNT: usize = 3;
const FILE_SCALAR_FIELD_COUNT: usize = 2;
const IMAGE_SCALAR_FIELD_COUNT: usize = 1;
const AGENT_SCALAR_FIELD_COUNT: usize = 5;

struct HttpText<'a> {
    method: &'a str,
    url: &'a str,
    body: Option<&'a str>,
}

pub(super) fn fields(
    workflow: &crate::engine::workflow::Workflow,
) -> Result<
    Vec<lattice_core::expression_dependencies::ExpressionField>,
    Vec<lattice_core::expression_dependencies::ExpressionDependencyIssue>,
> {
    let grouped = workflow
        .graph
        .nodes()
        .filter_map(|node| workflow.node_config(node.id).map(|config| (node.id.0, config)))
        .map(|(consumer_node, config)| fields_for_config(consumer_node, config))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(grouped.into_iter().flatten().collect())
}

// r[impl lattice.workflow_validation.explicit_dependencies.inventory]
fn fields_for_config(
    consumer_node: u64,
    config: &crate::engine::workflow::NodeConfig,
) -> Result<
    Vec<lattice_core::expression_dependencies::ExpressionField>,
    Vec<lattice_core::expression_dependencies::ExpressionDependencyIssue>,
> {
    match config {
        crate::engine::workflow::NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            pipe_stdin,
        } => {
            let _ = (timeout_secs, pipe_stdin);
            Ok(shell_fields(consumer_node, command, cwd.as_deref(), env))
        }
        crate::engine::workflow::NodeConfig::Http {
            method,
            url,
            headers,
            body,
            timeout_secs,
        } => {
            let _ = timeout_secs;
            Ok(http_fields(
                consumer_node,
                HttpText {
                    method,
                    url,
                    body: body.as_deref(),
                },
                headers,
            ))
        }
        crate::engine::workflow::NodeConfig::File {
            operation,
            path,
            content,
            overwrite,
            allowed_roots,
        } => {
            let _ = (operation, overwrite);
            Ok(file_fields(consumer_node, path, content.as_deref(), allowed_roots))
        }
        crate::engine::workflow::NodeConfig::ImageInput {
            path,
            allowed_roots,
            max_bytes,
        } => {
            let _ = max_bytes;
            Ok(image_fields(consumer_node, path, allowed_roots))
        }
        crate::engine::workflow::NodeConfig::Agent(settings) => agent_fields(consumer_node, settings),
        crate::engine::workflow::NodeConfig::Operation(_)
        | crate::engine::workflow::NodeConfig::Condition(_)
        | crate::engine::workflow::NodeConfig::Merge(_)
        | crate::engine::workflow::NodeConfig::Subworkflow(_)
        | crate::engine::workflow::NodeConfig::Approval(_) => Ok(Vec::new()),
    }
}

fn shell_fields(
    consumer_node: u64,
    command: &str,
    cwd: Option<&str>,
    environment: &std::collections::HashMap<String, String>,
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let mut fields = Vec::with_capacity(SHELL_SCALAR_FIELD_COUNT.saturating_add(environment.len()));
    fields.push(field(consumer_node, "shell.command".to_string(), command));
    fields.extend(cwd.map(|value| field(consumer_node, "shell.cwd".to_string(), value)));
    fields.extend(environment.iter().map(|(name, value)| field(consumer_node, format!("shell.env[{name:?}]"), value)));
    fields
}

fn http_fields(
    consumer_node: u64,
    text: HttpText<'_>,
    headers: &std::collections::HashMap<String, String>,
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let mut fields = Vec::with_capacity(HTTP_SCALAR_FIELD_COUNT.saturating_add(headers.len()));
    fields.push(field(consumer_node, "http.method".to_string(), text.method));
    fields.push(field(consumer_node, "http.url".to_string(), text.url));
    fields.extend(headers.iter().map(|(name, value)| field(consumer_node, format!("http.headers[{name:?}]"), value)));
    fields.extend(text.body.map(|value| field(consumer_node, "http.body".to_string(), value)));
    fields
}

fn file_fields(
    consumer_node: u64,
    path: &str,
    content: Option<&str>,
    allowed_roots: &[String],
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let mut fields = Vec::with_capacity(FILE_SCALAR_FIELD_COUNT.saturating_add(allowed_roots.len()));
    fields.push(field(consumer_node, "file.path".to_string(), path));
    fields.extend(content.map(|value| field(consumer_node, "file.content".to_string(), value)));
    fields.extend(
        allowed_roots
            .iter()
            .enumerate()
            .map(|(index, value)| field(consumer_node, format!("file.allowed_roots[{index}]"), value)),
    );
    fields
}

fn image_fields(
    consumer_node: u64,
    path: &str,
    allowed_roots: &[String],
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let mut fields = Vec::with_capacity(IMAGE_SCALAR_FIELD_COUNT.saturating_add(allowed_roots.len()));
    fields.push(field(consumer_node, "image_input.path".to_string(), path));
    fields.extend(
        allowed_roots
            .iter()
            .enumerate()
            .map(|(index, value)| field(consumer_node, format!("image_input.allowed_roots[{index}]"), value)),
    );
    fields
}

fn agent_fields(
    consumer_node: u64,
    settings: &crate::agent_node::AgentNodeSettings,
) -> Result<
    Vec<lattice_core::expression_dependencies::ExpressionField>,
    Vec<lattice_core::expression_dependencies::ExpressionDependencyIssue>,
> {
    let crate::agent_node::AgentNodeSettings {
        target,
        prompt,
        prompt_file,
        output_mode,
        model_hint,
        tool_policy,
        context_refs,
        cwd,
        session,
    } = settings;
    if prompt_file.as_deref().is_some_and(crate::expressions::contains_expression) {
        return Err(vec![unsupported(consumer_node, "agent.prompt_file")]);
    }
    let maximum = AGENT_SCALAR_FIELD_COUNT.saturating_add(context_refs.len());
    let mut fields = Vec::with_capacity(maximum);
    fields.push(field(consumer_node, "agent.prompt".to_string(), prompt));
    fields.extend(model_hint.as_deref().map(|value| field(consumer_node, "agent.model_hint".to_string(), value)));
    fields.push(field(consumer_node, "agent.tool_policy".to_string(), tool_policy));
    fields.extend(
        context_refs
            .iter()
            .enumerate()
            .map(|(index, value)| field(consumer_node, format!("agent.context_refs[{index}]"), value)),
    );
    fields.extend(cwd.as_deref().map(|value| field(consumer_node, "agent.cwd".to_string(), value)));
    fields.extend(session.as_deref().map(|value| field(consumer_node, "agent.session".to_string(), value)));
    let _ = (target, output_mode);
    debug_assert!(fields.len() <= maximum);
    debug_assert!(fields.iter().all(|field| field.consumer_node == consumer_node));
    Ok(fields)
}

fn field(
    consumer_node: u64,
    field_path: String,
    value: &str,
) -> lattice_core::expression_dependencies::ExpressionField {
    lattice_core::expression_dependencies::ExpressionField {
        consumer_node,
        field_path,
        value: value.to_string(),
    }
}

fn unsupported(
    consumer_node: u64,
    field_path: &str,
) -> lattice_core::expression_dependencies::ExpressionDependencyIssue {
    lattice_core::expression_dependencies::ExpressionDependencyIssue {
        code: lattice_core::expression_dependencies::ExpressionDependencyIssueCode::UnsupportedField,
        consumer_node,
        config_field_path: field_path.to_string(),
        producer_node: None,
        producer_output_port: None,
        message: "field does not support Lattice expression syntax".to_string(),
    }
}
