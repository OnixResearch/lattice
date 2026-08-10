//! Pure expression parsing and evaluation for workflow data mapping.

use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::value::Value;
use crate::value::ValueData;

const OPEN_DELIMITER: &str = "{{";
const CLOSE_DELIMITER: &str = "}}";
const NODE_PREFIX: &str = "node:";
const PORT_SEPARATOR: &str = ":port:";
const TRANSFORM_SEPARATOR: char = '|';
const TRANSFORM_JSON_PREFIX: &str = "json:";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionTemplate {
    pub segments: Vec<ExpressionSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpressionSegment {
    Literal(String),
    Reference(ExpressionReference),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionReference {
    pub node_id: u64,
    pub port_id: rat_nodegraph::model::PortId,
    pub transforms: Vec<ExpressionTransform>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpressionTransform {
    Trim,
    Upper,
    Lower,
    JsonField(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionError {
    InvalidSyntax(String),
    MissingReference {
        node_id: u64,
        port_id: rat_nodegraph::model::PortId,
    },
    TypeMismatch {
        transform: String,
        actual: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionValue {
    pub text: String,
    pub sensitive_fragments: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExpressionContext {
    values: HashMap<(u64, rat_nodegraph::model::PortId), Value>,
}

impl ExpressionContext {
    pub fn new(values: HashMap<(u64, rat_nodegraph::model::PortId), Value>) -> Self {
        Self { values }
    }

    pub fn insert(&mut self, node_id: u64, port_id: rat_nodegraph::model::PortId, value: Value) {
        self.values.insert((node_id, port_id), value);
    }

    fn get(&self, reference: &ExpressionReference) -> Option<&Value> {
        self.values.get(&(reference.node_id, reference.port_id))
    }
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionError::InvalidSyntax(message) => write!(f, "invalid expression syntax: {message}"),
            ExpressionError::MissingReference { node_id, port_id } => {
                write!(f, "expression reference node:{node_id}:port:{} is unavailable", port_id.0)
            }
            ExpressionError::TypeMismatch { transform, actual } => {
                write!(f, "expression transform '{transform}' cannot be applied to {actual}")
            }
        }
    }
}

pub fn contains_expression(input: &str) -> bool {
    input.contains(OPEN_DELIMITER) || input.contains(CLOSE_DELIMITER)
}

// r[impl n8nclone.workflow_execution.expression_mapping.syntax]
pub fn parse_expression_template(input: &str) -> Result<ExpressionTemplate, ExpressionError> {
    let mut segments = Vec::new();
    let mut rest = input;
    loop {
        let Some(open_index) = rest.find(OPEN_DELIMITER) else {
            if !rest.is_empty() {
                segments.push(ExpressionSegment::Literal(rest.to_string()));
            }
            return Ok(ExpressionTemplate { segments });
        };

        let literal = &rest[..open_index];
        if !literal.is_empty() {
            segments.push(ExpressionSegment::Literal(literal.to_string()));
        }

        let after_open = &rest[open_index + OPEN_DELIMITER.len()..];
        let Some(close_index) = after_open.find(CLOSE_DELIMITER) else {
            return Err(ExpressionError::InvalidSyntax("missing closing delimiter".to_string()));
        };
        let body = after_open[..close_index].trim();
        if body.is_empty() {
            return Err(ExpressionError::InvalidSyntax("empty expression body".to_string()));
        }
        segments.push(ExpressionSegment::Reference(parse_reference(body)?));
        rest = &after_open[close_index + CLOSE_DELIMITER.len()..];
    }
}

fn parse_reference(body: &str) -> Result<ExpressionReference, ExpressionError> {
    let mut parts = body.split(TRANSFORM_SEPARATOR);
    let reference = parts.next().ok_or_else(|| ExpressionError::InvalidSyntax("missing reference".to_string()))?;
    let Some(after_node) = reference.strip_prefix(NODE_PREFIX) else {
        return Err(ExpressionError::InvalidSyntax("reference must start with node:".to_string()));
    };
    let Some((node, port)) = after_node.split_once(PORT_SEPARATOR) else {
        return Err(ExpressionError::InvalidSyntax("reference must contain :port:".to_string()));
    };
    let node_id = parse_identifier(node, "node id")?;
    let port_id = parse_identifier(port, "port id")?;
    let transforms = parts.map(parse_transform).collect::<Result<Vec<_>, _>>()?;
    Ok(ExpressionReference {
        node_id,
        port_id: rat_nodegraph::model::PortId(port_id),
        transforms,
    })
}

fn parse_identifier(value: &str, label: &str) -> Result<u64, ExpressionError> {
    value
        .parse::<u64>()
        .map_err(|_| ExpressionError::InvalidSyntax(format!("{label} must be an unsigned integer")))
}

fn parse_transform(value: &str) -> Result<ExpressionTransform, ExpressionError> {
    let value = value.trim();
    match value {
        "trim" => Ok(ExpressionTransform::Trim),
        "upper" => Ok(ExpressionTransform::Upper),
        "lower" => Ok(ExpressionTransform::Lower),
        _ => match value.strip_prefix(TRANSFORM_JSON_PREFIX) {
            Some(field) if !field.trim().is_empty() => Ok(ExpressionTransform::JsonField(field.trim().to_string())),
            _ => Err(ExpressionError::InvalidSyntax(format!("unsupported transform '{value}'"))),
        },
    }
}

// r[impl n8nclone.workflow_execution.expression_mapping.evaluate]
pub fn evaluate_expression_template(
    template: &ExpressionTemplate,
    context: &ExpressionContext,
) -> Result<ExpressionValue, ExpressionError> {
    let mut output = ExpressionValue::default();
    for segment in &template.segments {
        match segment {
            ExpressionSegment::Literal(literal) => output.text.push_str(literal),
            ExpressionSegment::Reference(reference) => {
                let value = evaluate_reference(reference, context)?;
                output.text.push_str(&value.text);
                output.sensitive_fragments.extend(value.sensitive_fragments);
            }
        }
    }
    Ok(output)
}

pub fn resolve_expression_string(input: &str, context: &ExpressionContext) -> Result<ExpressionValue, ExpressionError> {
    if !contains_expression(input) {
        return Ok(ExpressionValue {
            text: input.to_string(),
            sensitive_fragments: Vec::new(),
        });
    }
    let template = parse_expression_template(input)?;
    evaluate_expression_template(&template, context)
}

fn evaluate_reference(
    reference: &ExpressionReference,
    context: &ExpressionContext,
) -> Result<ExpressionValue, ExpressionError> {
    let value = context.get(reference).ok_or(ExpressionError::MissingReference {
        node_id: reference.node_id,
        port_id: reference.port_id,
    })?;
    let mut expression_value = ExpressionValue {
        text: value.as_display_string(),
        sensitive_fragments: value.sensitive_fragments.clone(),
    };
    for transform in &reference.transforms {
        apply_transform(&mut expression_value, value, transform)?;
    }
    Ok(expression_value)
}

fn apply_transform(
    expression_value: &mut ExpressionValue,
    source: &Value,
    transform: &ExpressionTransform,
) -> Result<(), ExpressionError> {
    match transform {
        ExpressionTransform::Trim => {
            expression_value.text = expression_value.text.trim().to_string();
            Ok(())
        }
        ExpressionTransform::Upper => {
            expression_value.text = expression_value.text.to_uppercase();
            Ok(())
        }
        ExpressionTransform::Lower => {
            expression_value.text = expression_value.text.to_lowercase();
            Ok(())
        }
        ExpressionTransform::JsonField(field) => match &source.data {
            ValueData::Json(json) => match json.get(field) {
                Some(value) => {
                    expression_value.text = json_value_to_field_string(value);
                    Ok(())
                }
                None => Err(ExpressionError::TypeMismatch {
                    transform: format!("json:{field}"),
                    actual: "json without requested field".to_string(),
                }),
            },
            other => Err(ExpressionError::TypeMismatch {
                transform: format!("json:{field}"),
                actual: value_type_name(other).to_string(),
            }),
        },
    }
}

fn json_value_to_field_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

fn value_type_name(value: &ValueData) -> &'static str {
    match value {
        ValueData::String(_) => "string",
        ValueData::Bytes(_) => "bytes",
        ValueData::Json(_) => "json",
        ValueData::Image(_) => "image",
        ValueData::ExitCode(_) => "exit code",
        ValueData::Error { .. } => "error",
    }
}

pub fn expression_error_fragments(input: &str) -> Vec<ExpressionError> {
    if contains_expression(input) {
        parse_expression_template(input).err().into_iter().collect()
    } else {
        Vec::new()
    }
}
