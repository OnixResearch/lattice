use super::EXTERNAL_PROTOCOL_FIELD;
use super::PROTOCOL_ACP;
use super::PROTOCOL_MCP;
use super::RETURN_FULL_DETAIL_BOUNDED;
use super::RETURN_MODE_FIELD;
use super::RETURN_STATUS_PLUS_SUMMARY;
use super::RETURN_SUMMARY_ONLY;
use super::SOURCE_ADMITTED_AUTOMATION;
use super::SOURCE_AUTOMATIC;
use super::SOURCE_FIELD;
use super::SOURCE_TRUSTED_SKILL;
use super::SOURCE_USER;
use super::SubagentError;
use super::parse_value;

/// Source that explicitly delegated a subagent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentDelegationSource {
    /// User requested a child run.
    User,
    /// Trusted skill requested a child run.
    TrustedSkill,
    /// Automation policy admitted the child run.
    AdmittedAutomation,
    /// Automatic spawning request, always denied by validation.
    Automatic,
}

impl SubagentDelegationSource {
    /// Parses a stable delegation source label.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, SubagentError> {
        parse_value(SOURCE_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            SOURCE_USER => Some(Self::User),
            SOURCE_TRUSTED_SKILL => Some(Self::TrustedSkill),
            SOURCE_ADMITTED_AUTOMATION => Some(Self::AdmittedAutomation),
            SOURCE_AUTOMATIC => Some(Self::Automatic),
            _ => None,
        }
    }

    /// Returns the stable source label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => SOURCE_USER,
            Self::TrustedSkill => SOURCE_TRUSTED_SKILL,
            Self::AdmittedAutomation => SOURCE_ADMITTED_AUTOMATION,
            Self::Automatic => SOURCE_AUTOMATIC,
        }
    }
}

/// Parent handoff return mode for subagent results.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentReturnMode {
    /// Parent receives only bounded summary and evidence references.
    SummaryOnly,
    /// Parent receives status and bounded summary.
    StatusPlusSummary,
    /// Parent may receive a bounded detail reference.
    FullDetailWithBounds,
}

impl SubagentReturnMode {
    /// Parses a stable return mode label.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, SubagentError> {
        parse_value(RETURN_MODE_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            RETURN_SUMMARY_ONLY => Some(Self::SummaryOnly),
            RETURN_STATUS_PLUS_SUMMARY => Some(Self::StatusPlusSummary),
            RETURN_FULL_DETAIL_BOUNDED => Some(Self::FullDetailWithBounds),
            _ => None,
        }
    }

    /// Returns the stable return mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SummaryOnly => RETURN_SUMMARY_ONLY,
            Self::StatusPlusSummary => RETURN_STATUS_PLUS_SUMMARY,
            Self::FullDetailWithBounds => RETURN_FULL_DETAIL_BOUNDED,
        }
    }
}

/// External subagent protocol kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalSubagentProtocol {
    /// Agent Client Protocol adapter.
    Acp,
    /// Model Context Protocol adapter.
    Mcp,
}

impl ExternalSubagentProtocol {
    /// Parses a stable external subagent protocol label.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, SubagentError> {
        parse_value(EXTERNAL_PROTOCOL_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            PROTOCOL_ACP => Some(Self::Acp),
            PROTOCOL_MCP => Some(Self::Mcp),
            _ => None,
        }
    }
}
