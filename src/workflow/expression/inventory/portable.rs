//! Inventory projection for portable workflow configuration fields.

const MAX_EXPRESSION_FIELDS_PER_CONFIG: usize = 18;

pub(super) fn fields(
    configs: &[crate::workflow_format::PortableNodeConfig],
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let fields = configs.iter().flat_map(fields_for_config).collect::<Vec<_>>();
    let maximum_fields = configs.len().saturating_mul(MAX_EXPRESSION_FIELDS_PER_CONFIG);
    debug_assert!(fields.len() <= maximum_fields);
    debug_assert!(fields.iter().all(|field| configs.iter().any(|config| config.node_id == field.consumer_node)));
    fields
}

fn fields_for_config(
    config: &crate::workflow_format::PortableNodeConfig,
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let fields = match config.kind.as_str() {
        "shell" => shell_fields(config),
        "http" => http_fields(config),
        "file" => file_fields(config),
        "image_input" => image_fields(config),
        "agent" => agent_fields(config),
        _ => Vec::new(),
    };
    debug_assert!(fields.len() <= MAX_EXPRESSION_FIELDS_PER_CONFIG);
    debug_assert!(fields.iter().all(|field| field.consumer_node == config.node_id));
    fields
}

fn shell_fields(
    config: &crate::workflow_format::PortableNodeConfig,
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let command = field(config.node_id, "shell.command".to_string(), &config.command);
    let cwd = config.cwd.first().map(|value| field(config.node_id, "shell.cwd".to_string(), value));
    std::iter::once(command)
        .chain(cwd)
        .chain(config.env.iter().map(|environment| {
            field(config.node_id, format!("shell.env[{key:?}]", key = environment.key), &environment.value)
        }))
        .collect()
}

fn http_fields(
    config: &crate::workflow_format::PortableNodeConfig,
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let method = field(config.node_id, "http.method".to_string(), &config.http_method);
    let url = field(config.node_id, "http.url".to_string(), &config.url);
    let body = config.body.first().map(|value| field(config.node_id, "http.body".to_string(), value));
    [method, url]
        .into_iter()
        .chain(
            config
                .headers
                .iter()
                .map(|header| field(config.node_id, format!("http.headers[{key:?}]", key = header.key), &header.value)),
        )
        .chain(body)
        .collect()
}

fn file_fields(
    config: &crate::workflow_format::PortableNodeConfig,
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let path = field(config.node_id, "file.path".to_string(), &config.file_path);
    let content = config.file_content.first().map(|value| field(config.node_id, "file.content".to_string(), value));
    std::iter::once(path)
        .chain(content)
        .chain(
            config
                .allowed_roots
                .iter()
                .enumerate()
                .map(|(index, value)| field(config.node_id, format!("file.allowed_roots[{index}]"), value)),
        )
        .collect()
}

fn image_fields(
    config: &crate::workflow_format::PortableNodeConfig,
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let path = field(config.node_id, "image_input.path".to_string(), &config.image_path);
    std::iter::once(path)
        .chain(
            config
                .image_allowed_roots
                .iter()
                .enumerate()
                .map(|(index, value)| field(config.node_id, format!("image_input.allowed_roots[{index}]"), value)),
        )
        .collect()
}

fn agent_fields(
    config: &crate::workflow_format::PortableNodeConfig,
) -> Vec<lattice_core::expression_dependencies::ExpressionField> {
    let prompt = field(config.node_id, "agent.prompt".to_string(), &config.agent_prompt);
    let model = config
        .agent_model_hint
        .first()
        .map(|value| field(config.node_id, "agent.model_hint".to_string(), value));
    let cwd = config.agent_cwd.first().map(|value| field(config.node_id, "agent.cwd".to_string(), value));
    let session = config.agent_session.first().map(|value| field(config.node_id, "agent.session".to_string(), value));
    let fields = [
        prompt,
        field(config.node_id, "agent.tool_policy".to_string(), &config.agent_tool_policy),
    ]
    .into_iter()
    .chain(model)
    .chain(
        config
            .agent_context_refs
            .iter()
            .enumerate()
            .map(|(index, value)| field(config.node_id, format!("agent.context_refs[{index}]"), value)),
    )
    .chain(cwd)
    .chain(session)
    .collect::<Vec<_>>();
    debug_assert!(fields.len() <= MAX_EXPRESSION_FIELDS_PER_CONFIG);
    debug_assert!(fields.iter().all(|field| field.consumer_node == config.node_id));
    fields
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
