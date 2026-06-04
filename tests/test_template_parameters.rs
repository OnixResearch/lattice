//! Tests for Nickel node template parameters.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use n8nclone::config::DEFAULT_NODE_TEMPLATE_FILES;
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
const HEAD_TEMPLATE_NAME: &str = "head";
const REPLACE_TEMPLATE_NAME: &str = "replace";
const WORD_COUNT_TEMPLATE_NAME: &str = "word-count";
const PARAM_PATTERN: &str = "pattern";
const PARAM_REPLACEMENT: &str = "replacement";
const PARAM_LIMIT: &str = "limit";
const PARAM_VERBOSE: &str = "verbose";
const COMMAND_TEMPLATE: &str = "grep -E '${pattern}' | head -n ${limit} # ${verbose}";
const PATTERN_VALUE: &str = "needle";
const REPLACEMENT_VALUE: &str = "thread";
const LIMIT_VALUE: &str = "5";
const VERBOSE_VALUE: &str = "true";
const INVALID_NUMBER_VALUE: &str = "not-a-number";
const INVALID_BOOL_VALUE: &str = "yes";
const NUMBER_MIN: f64 = 1.0;
const NUMBER_MAX: f64 = 10.0;
const ABOVE_NUMBER_MAX_VALUE: &str = "11";
const RENDERED_PARAMETERIZED_COMMAND: &str = "grep -E 'needle' | head -n 5 # true";
const RENDERED_DEFAULT_HEAD_COMMAND: &str = "head -n 10";
const RENDERED_DEFAULT_WORD_COUNT_COMMAND: &str = "wc -l";
const RENDERED_REPLACE_COMMAND: &str = "sed 's/needle/thread/g'";
const CONFIG_FILE_NAME: &str = "config.ncl";
const TEMPLATES_DIR_NAME: &str = "templates";
const BAD_TEMPLATE_FILE_NAME: &str = "bad.ncl";
const MISMATCHED_DEFAULT_FILE_NAME: &str = "mismatched-default.ncl";
const SINGLE_DIAGNOSTIC_COUNT: usize = 1;
const DEFAULT_TEMPLATE_NAMES: &[&str] = &[
    TEMPLATE_NAME,
    HEAD_TEMPLATE_NAME,
    REPLACE_TEMPLATE_NAME,
    WORD_COUNT_TEMPLATE_NAME,
];

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

fn write_default_template_catalog(dir: &Path) {
    for template_file in DEFAULT_NODE_TEMPLATE_FILES {
        std::fs::write(dir.join(template_file.file_name), template_file.contents).expect("write default template");
    }
}

fn sorted_template_names(templates: &[NodeTemplate]) -> Vec<&str> {
    let mut names = templates.iter().map(|template| template.name.as_str()).collect::<Vec<_>>();
    names.sort();
    names
}

fn template_by_name<'a>(templates: &'a [NodeTemplate], name: &str) -> &'a NodeTemplate {
    templates.iter().find(|template| template.name == name).expect("default template should load")
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

    assert_eq!(command, RENDERED_PARAMETERIZED_COMMAND);
}

// r[verify n8nclone.node_templates.catalog]
// r[verify n8nclone.node_templates.catalog.load]
#[test]
fn default_template_catalog_loads_all_templates_without_diagnostics() {
    let temp_dir = tempfile::tempdir().expect("temporary template directory");
    write_default_template_catalog(temp_dir.path());

    let result = load_node_templates_with_diagnostics(temp_dir.path());

    assert!(result.diagnostics.is_empty());
    assert_eq!(sorted_template_names(&result.templates), DEFAULT_TEMPLATE_NAMES);
}

// r[verify n8nclone.node_templates.catalog]
// r[verify n8nclone.node_templates.catalog.render]
#[test]
fn default_template_catalog_renders_default_and_required_values() {
    let temp_dir = tempfile::tempdir().expect("temporary template directory");
    write_default_template_catalog(temp_dir.path());
    let result = load_node_templates_with_diagnostics(temp_dir.path());
    let head = template_by_name(&result.templates, HEAD_TEMPLATE_NAME);
    let word_count = template_by_name(&result.templates, WORD_COUNT_TEMPLATE_NAME);
    let replace = template_by_name(&result.templates, REPLACE_TEMPLATE_NAME);
    let replace_values = HashMap::from([
        (PARAM_PATTERN.to_string(), PATTERN_VALUE.to_string()),
        (PARAM_REPLACEMENT.to_string(), REPLACEMENT_VALUE.to_string()),
    ]);

    assert_eq!(render_template_command(head, &HashMap::new()), Ok(RENDERED_DEFAULT_HEAD_COMMAND.to_string()));
    assert_eq!(
        render_template_command(word_count, &HashMap::new()),
        Ok(RENDERED_DEFAULT_WORD_COUNT_COMMAND.to_string())
    );
    assert_eq!(render_template_command(replace, &replace_values), Ok(RENDERED_REPLACE_COMMAND.to_string()));
}

// r[verify n8nclone.node_templates.catalog]
// r[verify n8nclone.node_templates.catalog.init]
#[test]
fn cli_init_writes_default_template_catalog() {
    let temp_dir = tempfile::tempdir().expect("temporary cli config directory");
    let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
    let templates_dir = temp_dir.path().join(TEMPLATES_DIR_NAME);

    let output = Command::new(env!("CARGO_BIN_EXE_n8nclone"))
        .arg("--config")
        .arg(&config_path)
        .arg("--init")
        .output()
        .expect("run init command");

    assert!(output.status.success());
    for template_file in DEFAULT_NODE_TEMPLATE_FILES {
        assert!(templates_dir.join(template_file.file_name).exists());
    }
    let result = load_node_templates_with_diagnostics(&templates_dir);
    assert!(result.diagnostics.is_empty());
    assert_eq!(sorted_template_names(&result.templates), DEFAULT_TEMPLATE_NAMES);
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
        (PARAM_LIMIT.to_string(), INVALID_NUMBER_VALUE.to_string()),
    ]);

    let error = render_template_command(&template, &values).expect_err("invalid number rejected");

    assert_eq!(error, TemplateRenderError::InvalidNumber {
        name: PARAM_LIMIT.to_string(),
        value: INVALID_NUMBER_VALUE.to_string()
    });
}

// r[verify n8nclone.node_templates.parameters.invalid]
// r[verify n8nclone.node_templates.parameters.invalid.type]
#[test]
fn render_template_rejects_invalid_bool_parameter() {
    let template = parameterized_template();
    let values = HashMap::from([
        (PARAM_PATTERN.to_string(), PATTERN_VALUE.to_string()),
        (PARAM_VERBOSE.to_string(), INVALID_BOOL_VALUE.to_string()),
    ]);

    let error = render_template_command(&template, &values).expect_err("invalid bool rejected");

    assert_eq!(error, TemplateRenderError::InvalidBool {
        name: PARAM_VERBOSE.to_string(),
        value: INVALID_BOOL_VALUE.to_string()
    });
}

// r[verify n8nclone.node_templates.parameters.invalid]
// r[verify n8nclone.node_templates.parameters.invalid.bounds]
#[test]
fn render_template_rejects_number_above_maximum() {
    let template = parameterized_template();
    let values = HashMap::from([
        (PARAM_PATTERN.to_string(), PATTERN_VALUE.to_string()),
        (PARAM_LIMIT.to_string(), ABOVE_NUMBER_MAX_VALUE.to_string()),
    ]);

    let error = render_template_command(&template, &values).expect_err("above maximum rejected");

    assert_eq!(error, TemplateRenderError::NumberAboveMaximum {
        name: PARAM_LIMIT.to_string(),
        max: NUMBER_MAX.to_string()
    });
}

// r[verify n8nclone.node_templates.parameters.schema.invalid]
#[test]
fn loader_reports_invalid_parameter_schema_diagnostic() {
    let temp_dir = tempfile::tempdir().expect("temporary template directory");
    std::fs::write(
        temp_dir.path().join(BAD_TEMPLATE_FILE_NAME),
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
    assert_eq!(result.diagnostics.len(), SINGLE_DIAGNOSTIC_COUNT);
    assert!(result.diagnostics[0].message.contains("invalid template"));
}

// r[verify n8nclone.node_templates.parameters.schema.invalid]
#[test]
fn loader_reports_mismatched_default_type_diagnostic() {
    let temp_dir = tempfile::tempdir().expect("temporary template directory");
    std::fs::write(
        temp_dir.path().join(MISMATCHED_DEFAULT_FILE_NAME),
        r#"{
  name = "mismatched-default",
  description = "Bad default type",
  kind = "shell",
  command = "head -n ${count}",
  parameters = [
    { name = "count", type = "number", default = "ten", required = false },
  ],
  input_ports = [],
  output_ports = [],
}
"#,
    )
    .expect("write mismatched default template");

    let result = load_node_templates_with_diagnostics(temp_dir.path());

    assert!(result.templates.is_empty());
    assert_eq!(result.diagnostics.len(), SINGLE_DIAGNOSTIC_COUNT);
    assert!(result.diagnostics[0].message.contains("default value type"));
}
