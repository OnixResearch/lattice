//! Property editor field vocabulary and validation.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyEditField {
    Label,
    Command,
    TimeoutSecs,
    ImagePath,
    AgentPrompt,
    AgentTarget,
    AgentOutputMode,
    AgentModelHint,
    AgentToolPolicy,
    AgentContextRefs,
    AgentCwd,
    AgentSession,
    Credentials,
    ConditionBranches,
}

impl PropertyEditField {
    pub(in crate::tui::app) fn label(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::Command => "command",
            Self::TimeoutSecs => "timeout",
            Self::ImagePath => "image path",
            Self::AgentPrompt => "agent prompt",
            Self::AgentTarget => "agent target",
            Self::AgentOutputMode => "agent output mode",
            Self::AgentModelHint => "agent model hint",
            Self::AgentToolPolicy => "agent tool policy",
            Self::AgentContextRefs => "agent context refs",
            Self::AgentCwd => "agent cwd",
            Self::AgentSession => "agent session",
            Self::Credentials => "credentials",
            Self::ConditionBranches => "condition branches",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyEditState {
    pub node_id: NodeId,
    pub field: PropertyEditField,
    pub original: String,
    pub workflow_snapshot: crate::workflow_format::PortableWorkflow,
    pub draft: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyEditError {
    MissingSelection,
    NodeNotFound,
    MissingConfig,
    InvalidTimeout,
    InvalidAgentTarget,
    InvalidAgentOutputMode,
    InvalidAgentCredentials,
    InvalidConditionBranches,
    InvalidMergePolicy,
    InvalidHttpConfig,
    InvalidFileConfig,
    Typed(String),
}

pub fn validate_timeout_secs(input: &str) -> Result<u64, PropertyEditError> {
    input.trim().parse::<u64>().map_err(|_| PropertyEditError::InvalidTimeout)
}

pub fn validate_agent_target(input: &str) -> Result<crate::agent_node::Target, PropertyEditError> {
    agent_node::parse_target(input.trim()).map_err(|_| PropertyEditError::InvalidAgentTarget)
}

pub fn validate_agent_output_mode(input: &str) -> Result<crate::agent_node::OutputMode, PropertyEditError> {
    agent_node::parse_output_mode(input.trim()).map_err(|_| PropertyEditError::InvalidAgentOutputMode)
}
