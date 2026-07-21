use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use super::guidance::PromptResourceScope;

const SOURCE_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("prompt_source_label");
const SECTION_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("prompt_section_label");
const CONFLICT_REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("prompt_conflict_reason");
const MAX_PROMPT_SOURCES: usize = 32;
const MAX_PROMPT_SECTIONS: usize = 16;

const _: () = assert!(MAX_PROMPT_SOURCES > 0);
const _: () = assert!(MAX_PROMPT_SECTIONS > 0);

/// Stable prompt section ordering key.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PromptSectionKind {
    /// System/developer instruction section.
    System,
    /// Policy and operating profile section.
    Policy,
    /// Retrieved or supplied context section.
    Context,
    /// Tool metadata section.
    Tools,
    /// Current run objective section.
    Objective,
}

impl PromptSectionKind {
    /// Returns the stable section label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Policy => "policy",
            Self::Context => "context",
            Self::Tools => "tools",
            Self::Objective => "objective",
        }
    }
}

/// Prompt source kind admitted by the host.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PromptSourceKind {
    /// Direct host context supplied as data.
    HostContext,
    /// Policy profile or operating guidance.
    PolicyProfile,
    /// Progressive-disclosure skill metadata or loaded skill body.
    Skill,
    /// Memory-backed prompt source.
    Memory,
    /// Agent definition prompt fragment.
    AgentDefinition,
    /// Context reference expansion.
    ContextReference,
    /// Tool definition metadata.
    ToolMetadata,
    /// Current run objective or task summary.
    RunObjective,
    /// Source kind unsupported by this runtime.
    Unsupported,
}

impl PromptSourceKind {
    /// Returns the stable source-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HostContext => "host-context",
            Self::PolicyProfile => "policy-profile",
            Self::Skill => "skill",
            Self::Memory => "memory",
            Self::AgentDefinition => "agent-definition",
            Self::ContextReference => "context-reference",
            Self::ToolMetadata => "tool-metadata",
            Self::RunObjective => "run-objective",
            Self::Unsupported => "unsupported",
        }
    }

    const fn requires_discovery_adapter(self) -> bool {
        match self {
            Self::Memory | Self::ContextReference => true,
            Self::HostContext
            | Self::PolicyProfile
            | Self::Skill
            | Self::AgentDefinition
            | Self::ToolMetadata
            | Self::RunObjective
            | Self::Unsupported => false,
        }
    }
}

/// Trust status for a prompt source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptSourceTrustStatus {
    /// Source has been admitted for prompt influence.
    Trusted,
    /// Source has not been admitted for prompt influence.
    Untrusted,
}

/// Admission status for a prompt source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptSourceAdmissionStatus {
    /// Source may be considered for model-visible assembly.
    Admitted,
    /// Source was denied by host or policy.
    Denied,
    /// Required source or adapter was unavailable.
    Unavailable,
}

/// Source requirement rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptSourceRequirement {
    /// Missing or denied source is a required-source issue.
    Required,
    /// Missing or denied source is optional.
    Optional,
}

/// Redaction class for a prompt source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptRedactionClass {
    /// Source can produce model-visible prompt content.
    Public,
    /// Source can expose metadata only.
    MetadataOnly,
    /// Source cannot expose content or metadata to the model.
    Secret,
}

/// Prompt budget class for reporting and policy grouping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptBudgetClass {
    /// Small prompt contribution.
    Tiny,
    /// Normal prompt contribution.
    Normal,
    /// Large prompt contribution.
    Large,
}

/// Ordering rank for prompt source precedence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PromptSourcePriority(u16);

impl PromptSourcePriority {
    /// Builds a prompt source priority where lower ranks are assembled first.
    #[must_use]
    pub const fn new(rank: u16) -> Self {
        Self(rank)
    }

    /// Returns the stable priority rank.
    #[must_use]
    pub const fn rank(self) -> u16 {
        self.0
    }
}

/// Borrowed prompt-source fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromptSourceDraft<'a> {
    /// Source kind.
    pub kind: PromptSourceKind,
    /// Source scope.
    pub scope: PromptResourceScope,
    /// Trust/admission status.
    pub trust_status: PromptSourceTrustStatus,
    /// Admission status.
    pub admission_status: PromptSourceAdmissionStatus,
    /// Required or optional source rule.
    pub requirement: PromptSourceRequirement,
    /// Source priority.
    pub priority: PromptSourcePriority,
    /// Budget class.
    pub budget_class: PromptBudgetClass,
    /// Redaction class.
    pub redaction_class: PromptRedactionClass,
    /// Safe source label.
    pub label: &'a str,
    /// Prompt section target.
    pub section: PromptSectionKind,
    /// Lowercase BLAKE3 identity for the source body.
    pub body_hash: &'a str,
    /// Budget units requested by this source.
    pub requested_units: u32,
}

/// Validated typed prompt-source DTO.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptSource {
    kind: PromptSourceKind,
    scope: PromptResourceScope,
    trust_status: PromptSourceTrustStatus,
    admission_status: PromptSourceAdmissionStatus,
    requirement: PromptSourceRequirement,
    priority: PromptSourcePriority,
    budget_class: PromptBudgetClass,
    redaction_class: PromptRedactionClass,
    label: String,
    section: PromptSectionKind,
    body_hash: crate::artifact::Hash,
    requested_units: NonZeroU32,
}

