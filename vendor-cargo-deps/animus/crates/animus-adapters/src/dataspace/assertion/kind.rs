const FIELD: crate::validation::FieldName = crate::validation::FieldName::new("dataspace_assertion_kind");

const RUNTIME_EVENT: &str = "runtime_event";
const TOOL_STATE: &str = "tool_state";
const BACKGROUND_WORK: &str = "background_work";
const SESSION_SUMMARY: &str = "session_summary";
const PROVENANCE_SUMMARY: &str = "provenance_summary";
const AGENT_LIFECYCLE: &str = "agent_lifecycle";
const SUBAGENT_LIFECYCLE: &str = "subagent_lifecycle";
const CANCELLATION: &str = "cancellation";
const OWNERSHIP_TRANSFER: &str = "ownership_transfer";

/// Bounded assertion classes published through an optional dataspace adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    /// Redacted runtime event projection.
    RuntimeEvent,
    /// Redacted tool-state projection.
    ToolState,
    /// Redacted background-work projection.
    BackgroundWork,
    /// Redacted session summary projection.
    SessionSummary,
    /// Redacted provenance summary projection.
    ProvenanceSummary,
    /// Redacted top-level agent lifecycle projection.
    AgentLifecycle,
    /// Redacted subagent lifecycle projection.
    SubagentLifecycle,
    /// Cancellation assertion or lifecycle update.
    Cancellation,
    /// Ownership-transfer assertion or lifecycle update.
    OwnershipTransfer,
}

impl Value {
    /// Parses a stable assertion-kind label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(FIELD, value, Self::from_normalized)
    }

    pub(super) fn parse_for_field(
        field_name: crate::validation::FieldName,
        value: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(field_name, value, Self::from_normalized)
    }

    fn from_normalized(normalized: &str) -> Option<Self> {
        match normalized {
            RUNTIME_EVENT => Some(Self::RuntimeEvent),
            TOOL_STATE => Some(Self::ToolState),
            BACKGROUND_WORK => Some(Self::BackgroundWork),
            SESSION_SUMMARY => Some(Self::SessionSummary),
            PROVENANCE_SUMMARY => Some(Self::ProvenanceSummary),
            AGENT_LIFECYCLE => Some(Self::AgentLifecycle),
            SUBAGENT_LIFECYCLE => Some(Self::SubagentLifecycle),
            CANCELLATION => Some(Self::Cancellation),
            OWNERSHIP_TRANSFER => Some(Self::OwnershipTransfer),
            _ => None,
        }
    }
}
