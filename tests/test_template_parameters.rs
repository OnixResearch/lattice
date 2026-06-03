//! Tests for Nickel node template parameters.

use std::collections::HashMap;

use n8nclone::config::NodeTemplate;
use n8nclone::config::PortTemplate;
use n8nclone::config::TemplateParameter;
use n8nclone::config::TemplateParameterDefault;
use n8nclone::config::TemplateParameterType;
use n8nclone::config::TemplateRenderError;
use n8nclone::config::load_node_templates_with_diagnostics;
use n8nclone::config::render_template_command;
use n8nclone::config::validate_template_parameters;
use pretty_assertions::assert_eq;

const TEMPLATE_NAME: &str = "grep";
const PARAM_PATTERN: &str = "pattern";
const PARAM_LIMIT: &str = "limit";
const PARAM_VERBOSE: &str = "verbose";
const COMMAND_TEMPLATE: &str = "grep -E '${pattern}' | head -n ${limit} # ${verbose}";
const PATTERN_VALUE: &str = "needle";
const LIMIT_VALUE: &str = "5";
const VERBOSE_VALUE: &str = "true";
const NUMBER_MIN: f64 = 1.0;
const NUMBER_MAX: f64 = 10.0;

fn parameterized_template() -> NodeTemplate {
    NodeTemplate {
        name: TEMPLATE_NAME.to_string(),
        description: "Filter lines".to_string(),
        kind: "shell".to_string(),
        command: Some(COMMAND_TEMPLATE.to_string()),
        parameters: vec![
            TemplateParameter {
                name: PARAM_PATTERN.to_string(),
                type_tag: TemplateParameterType::String,
                description: "Pattern".to_string(),
                default: None,
                required: true,
                min: None,
                max: None,
            },
            TemplateParameter {
                name: PARAM_LIMIT.to_string(),
                type_tag: TemplateParameterType::Number,
                description: "Line limit".to_string(),
                default: Some(TemplateParameterDefault::Number(NUMBER_MAX)),
                required: false,
                min: Some(NUMBER_MIN),
                max: Some(NUMBER_MAX),
            },
            TemplateParameter {
                name: PARAM_VERBOSE.to_string(),
                type_tag: TemplateParameterType::Bool,
                description: "Verbose marker".to_string(),
                default: Some(TemplateParameterDefault::Bool(false)),
                required: false,
                min: None,
                max: None,
            },
        ],
        input_ports: vec![PortTemplate {
            label: "stdin".to_string(),
            type_tag: "string".to_string(),
        }],
        output_ports: vec![PortTemplate {
            label: "stdout".to_string(),
            type_tag: "string".to_string(),
        }],
    }
}

// r[verify n8nclone.node_templates.parameters.schema]
// r[verify n8nclone.node_templates.parameters.schema.load]
#[test]
fn template_parameter_schema_validates_supported_types() {
    let template = parameterized_template();

    assert_eq!(validate_template_parameters(&template), Ok(()));
}

// r[verify n8nclone.node_templates.parameters.render]
// r[verify n8nclone.node_templates.parameters.render.command]
#[test]
fn render_template_command_replaces_validated_parameters() {
    let template = parameterized_template();
    let values = HashMap::from([
        (PARAM_PATTERN.to_string(), PATTERN_VALUE.to_string()),
        (PARAM_LIMIT.to_string(), LIMIT_VALUE.to_string()),
        (PARAM_VERBOSE.to_string(), VERBOSE_VALUE.to_string()),
    ]);

    let command = render_template_command(&template, &values).expect("template renders");

    assert_eq!(command, "grep -E 'needle' | head -n 5 # true");
}

// r[verify n8nclone.node_templates.parameters.render.missing]
#[test]
fn render_template_rejects_missing_required_parameter() {
    let template = parameterized_template();
    let values = HashMap::new();

    let error = render_template_command(&template, &values).expect_err("missing required parameter rejected");

    assert_eq!(error, TemplateRenderError::MissingRequired(PARAM_PATTERN.to_string()));
}

// r[verify n8nclone.node_templates.parameters.invalid]
// r[verify n8nclone.node_templates.parameters.invalid.type]
#[test]
fn render_template_rejects_invalid_number_parameter() {
    let template = parameterized_template();
    let values = HashMap::from([
        (PARAM_PATTERN.to_string(), PATTERN_VALUE.to_string()),
        (PARAM_LIMIT.to_string(), "not-a-number".to_string()),
    ]);

    let error = render_template_command(&template, &values).expect_err("invalid number rejected");

    assert_eq!(error, TemplateRenderError::InvalidNumber {
        name: PARAM_LIMIT.to_string(),
        value: "not-a-number".to_string()
    });
}

// r[verify n8nclone.node_templates.parameters.schema.invalid]
#[test]
fn loader_reports_invalid_parameter_schema_diagnostic() {
    let temp_dir = tempfile::tempdir().expect("temporary template directory");
    std::fs::write(
        temp_dir.path().join("bad.ncl"),
        r#"{
  name = "bad",
  description = "Bad template",
  kind = "shell",
  command = "echo ${count}",
  parameters = [
    { name = "count", type = "integer", required = true },
  ],
  input_ports = [],
  output_ports = [],
}
"#,
    )
    .expect("write bad template");

    let result = load_node_templates_with_diagnostics(temp_dir.path());

    assert!(result.templates.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("invalid template"));
}
