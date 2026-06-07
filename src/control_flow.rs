//! Pure control-flow and subworkflow admission core.

use std::collections::BTreeSet;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::engine::Value;
use crate::engine::ValueData;

const PREDICATE_ALWAYS: &str = "always";
const PREDICATE_EXISTS: &str = "exists";
const PREDICATE_EQ_PREFIX: &str = "eq:";
const PREDICATE_CONTAINS_PREFIX: &str = "contains:";
const PREDICATE_GT_PREFIX: &str = "gt:";
const PREDICATE_JSON_EXISTS_PREFIX: &str = "json_exists:";
const PREDICATE_BOOL_PREFIX: &str = "bool:";
const MERGE_OUTPUT_PORT_LABEL: &str = "merged";
const SUBWORKFLOW_DEFAULT_OUTPUT_PORT_LABEL: &str = "result";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionPredicate {
    Always,
    Exists,
    StringEquals(String),
    StringContains(String),
    NumberGreaterThan(f64),
    JsonPointerExists(String),
    BoolEquals(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionBranch {
    pub label: String,
    pub predicate: ConditionPredicate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionNodeConfig {
    pub branches: Vec<ConditionBranch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergePolicy {
    All,
    First,
    Last,
    Collect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeNodeConfig {
    pub policy: MergePolicy,
    pub required_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubworkflowInputMapping {
    pub parent_input: String,
    pub child_input: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubworkflowOutputMapping {
    pub child_output: String,
    pub parent_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubworkflowNodeConfig {
    pub workflow_name: String,
    pub input_mappings: Vec<SubworkflowInputMapping>,
    pub output_mappings: Vec<SubworkflowOutputMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlFlowError {
    EmptyBranchLabel,
    DuplicateBranchLabel(String),
    InvalidPredicate(String),
    EmptyMergeInput,
    DuplicateMergeInput(String),
    MissingMergeInput(String),
    EmptySubworkflowName,
    MissingSubworkflow(String),
    RecursiveSubworkflow(String),
    InvalidMapping(String),
}

impl std::fmt::Display for ControlFlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyBranchLabel => write!(f, "condition branch label is empty"),
            Self::DuplicateBranchLabel(label) => write!(f, "duplicate condition branch label: {label}"),
            Self::InvalidPredicate(predicate) => write!(f, "invalid condition predicate: {predicate}"),
            Self::EmptyMergeInput => write!(f, "merge input label is empty"),
            Self::DuplicateMergeInput(input) => write!(f, "duplicate merge input label: {input}"),
            Self::MissingMergeInput(input) => write!(f, "merge input is not ready: {input}"),
            Self::EmptySubworkflowName => write!(f, "subworkflow name is empty"),
            Self::MissingSubworkflow(name) => write!(f, "missing subworkflow reference: {name}"),
            Self::RecursiveSubworkflow(name) => write!(f, "recursive subworkflow reference: {name}"),
            Self::InvalidMapping(message) => write!(f, "invalid subworkflow mapping: {message}"),
        }
    }
}

impl std::error::Error for ControlFlowError {}

#[derive(Debug, Clone)]
pub struct MergeInputValue {
    pub label: String,
    pub sequence: u64,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct MergeDecision {
    pub ready: bool,
    pub selected: Vec<MergeInputValue>,
    pub missing_inputs: Vec<String>,
}

// r[impl n8nclone.workflow_execution.control_flow.condition]
pub fn parse_condition_predicate(input: &str) -> Result<ConditionPredicate, ControlFlowError> {
    let trimmed = input.trim();
    if trimmed == PREDICATE_ALWAYS {
        return Ok(ConditionPredicate::Always);
    }
    if trimmed == PREDICATE_EXISTS {
        return Ok(ConditionPredicate::Exists);
    }
    if let Some(value) = trimmed.strip_prefix(PREDICATE_EQ_PREFIX) {
        return Ok(ConditionPredicate::StringEquals(value.to_string()));
    }
    if let Some(value) = trimmed.strip_prefix(PREDICATE_CONTAINS_PREFIX) {
        return Ok(ConditionPredicate::StringContains(value.to_string()));
    }
    if let Some(value) = trimmed.strip_prefix(PREDICATE_GT_PREFIX) {
        return value
            .parse::<f64>()
            .map(ConditionPredicate::NumberGreaterThan)
            .map_err(|_| ControlFlowError::InvalidPredicate(input.to_string()));
    }
    if let Some(value) = trimmed.strip_prefix(PREDICATE_JSON_EXISTS_PREFIX) {
        return Ok(ConditionPredicate::JsonPointerExists(value.to_string()));
    }
    if let Some(value) = trimmed.strip_prefix(PREDICATE_BOOL_PREFIX) {
        return match value {
            "true" => Ok(ConditionPredicate::BoolEquals(true)),
            "false" => Ok(ConditionPredicate::BoolEquals(false)),
            _ => Err(ControlFlowError::InvalidPredicate(input.to_string())),
        };
    }
    Err(ControlFlowError::InvalidPredicate(input.to_string()))
}

// r[impl n8nclone.workflow_execution.control_flow.condition]
pub fn evaluate_condition(config: &ConditionNodeConfig, value: Option<&Value>) -> Vec<String> {
    config
        .branches
        .iter()
        .filter(|branch| predicate_matches(&branch.predicate, value))
        .map(|branch| branch.label.clone())
        .collect()
}

fn predicate_matches(predicate: &ConditionPredicate, value: Option<&Value>) -> bool {
    match predicate {
        ConditionPredicate::Always => true,
        ConditionPredicate::Exists => value.is_some(),
        ConditionPredicate::StringEquals(expected) => {
            value.and_then(value_as_string).is_some_and(|value| value == *expected)
        }
        ConditionPredicate::StringContains(needle) => {
            value.and_then(value_as_string).is_some_and(|value| value.contains(needle))
        }
        ConditionPredicate::NumberGreaterThan(threshold) => {
            value.and_then(value_as_number).is_some_and(|value| value > *threshold)
        }
        ConditionPredicate::JsonPointerExists(pointer) => {
            value.and_then(value_as_json).is_some_and(|value| value.pointer(pointer).is_some())
        }
        ConditionPredicate::BoolEquals(expected) => value.and_then(value_as_bool) == Some(*expected),
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    match &value.data {
        ValueData::String(value) => Some(value.clone()),
        ValueData::Json(value) => value.as_str().map(ToOwned::to_owned),
        _ => None,
    }
}

fn value_as_number(value: &Value) -> Option<f64> {
    match &value.data {
        ValueData::Json(value) => value.as_f64(),
        ValueData::String(value) => value.parse::<f64>().ok(),
        _ => None,
    }
}

fn value_as_json(value: &Value) -> Option<&serde_json::Value> {
    match &value.data {
        ValueData::Json(value) => Some(value),
        _ => None,
    }
}

fn value_as_bool(value: &Value) -> Option<bool> {
    match &value.data {
        ValueData::Json(value) => value.as_bool(),
        ValueData::String(value) => match value.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

// r[impl n8nclone.workflow_validation.control_flow]
pub fn validate_condition_config(config: &ConditionNodeConfig) -> Result<(), ControlFlowError> {
    let mut seen = BTreeSet::new();
    for branch in &config.branches {
        if branch.label.trim().is_empty() {
            return Err(ControlFlowError::EmptyBranchLabel);
        }
        if !seen.insert(branch.label.clone()) {
            return Err(ControlFlowError::DuplicateBranchLabel(branch.label.clone()));
        }
    }
    Ok(())
}

// r[impl n8nclone.workflow_execution.control_flow.merge]
pub fn merge_decision(
    config: &MergeNodeConfig,
    available_inputs: Vec<MergeInputValue>,
) -> Result<MergeDecision, ControlFlowError> {
    validate_merge_config(config)?;
    let available_by_label =
        available_inputs.into_iter().map(|input| (input.label.clone(), input)).collect::<HashMap<_, _>>();
    let missing_inputs = config
        .required_inputs
        .iter()
        .filter(|label| !available_by_label.contains_key(*label))
        .cloned()
        .collect::<Vec<_>>();
    if config.policy == MergePolicy::All && !missing_inputs.is_empty() {
        return Ok(MergeDecision {
            ready: false,
            selected: Vec::new(),
            missing_inputs,
        });
    }
    let mut available = available_by_label.into_values().collect::<Vec<_>>();
    available.sort_by_key(|input| input.sequence);
    let selected = match config.policy {
        MergePolicy::All | MergePolicy::Collect => available,
        MergePolicy::First => available.into_iter().take(1).collect(),
        MergePolicy::Last => available.into_iter().rev().take(1).collect(),
    };
    Ok(MergeDecision {
        ready: !selected.is_empty(),
        selected,
        missing_inputs,
    })
}

// r[impl n8nclone.workflow_validation.control_flow]
pub fn validate_merge_config(config: &MergeNodeConfig) -> Result<(), ControlFlowError> {
    let mut seen = BTreeSet::new();
    for input in &config.required_inputs {
        if input.trim().is_empty() {
            return Err(ControlFlowError::EmptyMergeInput);
        }
        if !seen.insert(input.clone()) {
            return Err(ControlFlowError::DuplicateMergeInput(input.clone()));
        }
    }
    Ok(())
}

// r[impl n8nclone.workflow_execution.subworkflows.invocation]
pub fn validate_subworkflow_admission(
    config: &SubworkflowNodeConfig,
    ancestry: &[String],
) -> Result<(), ControlFlowError> {
    validate_subworkflow_config(config)?;
    if ancestry.iter().any(|name| name == &config.workflow_name) {
        return Err(ControlFlowError::RecursiveSubworkflow(config.workflow_name.clone()));
    }
    Ok(())
}

// r[impl n8nclone.workflow_validation.subworkflows]
pub fn validate_subworkflow_reference(
    config: &SubworkflowNodeConfig,
    available_workflows: &[String],
    ancestry: &[String],
) -> Result<(), ControlFlowError> {
    validate_subworkflow_admission(config, ancestry)?;
    if !available_workflows.iter().any(|name| name == &config.workflow_name) {
        return Err(ControlFlowError::MissingSubworkflow(config.workflow_name.clone()));
    }
    Ok(())
}

// r[impl n8nclone.workflow_validation.subworkflows]
pub fn validate_subworkflow_config(config: &SubworkflowNodeConfig) -> Result<(), ControlFlowError> {
    if config.workflow_name.trim().is_empty() {
        return Err(ControlFlowError::EmptySubworkflowName);
    }
    for mapping in &config.input_mappings {
        if mapping.parent_input.trim().is_empty() || mapping.child_input.trim().is_empty() {
            return Err(ControlFlowError::InvalidMapping("input mapping contains an empty endpoint".to_string()));
        }
    }
    for mapping in &config.output_mappings {
        if mapping.parent_output.trim().is_empty() || mapping.child_output.trim().is_empty() {
            return Err(ControlFlowError::InvalidMapping("output mapping contains an empty endpoint".to_string()));
        }
    }
    Ok(())
}

pub fn merge_output_port_label() -> &'static str {
    MERGE_OUTPUT_PORT_LABEL
}

pub fn subworkflow_default_output_port_label() -> &'static str {
    SUBWORKFLOW_DEFAULT_OUTPUT_PORT_LABEL
}
