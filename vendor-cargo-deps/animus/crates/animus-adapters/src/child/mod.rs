//! Recipe-backed subagent delegation DTOs and pure validation helpers.

#![allow(non_trait_imports)]

const ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_id");
const ROLE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_role");
const TASK_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_task");
const RECIPE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_recipe");
const PROFILE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_profile");
const PARAMETER_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_parameter_binding");
const TOOL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_tool");
const WORKSPACE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_workspace_scope");
const MODEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_model_profile");
const TIMEOUT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_timeout");
const RETURN_MODE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_return_mode");
const SUMMARY_SCHEMA_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_summary_schema");
const SOURCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_delegation_source");
const CAPABILITY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_capability");
const EXTERNAL_PROTOCOL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("subagent_external_protocol");
const EXTERNAL_SERVICE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("subagent_external_service");
const HANDOFF_STATUS_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_handoff_status");
const HANDOFF_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("subagent_handoff_summary");
const HANDOFF_DETAIL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("subagent_handoff_detail_ref");
const OUTCOME_KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("subagent_outcome_kind");

const SOURCE_USER: &str = "user";
const SOURCE_TRUSTED_SKILL: &str = "trusted_skill";
const SOURCE_ADMITTED_AUTOMATION: &str = "admitted_automation";
const SOURCE_AUTOMATIC: &str = "automatic";
const RETURN_SUMMARY_ONLY: &str = "summary_only";
const RETURN_STATUS_PLUS_SUMMARY: &str = "status_plus_summary";
const RETURN_FULL_DETAIL_BOUNDED: &str = "full_detail_with_bounds";
const PROTOCOL_ACP: &str = "acp";
const PROTOCOL_MCP: &str = "mcp";
const OUTCOME_SUCCESS: &str = "success";
const OUTCOME_FAILURE: &str = "failure";
const OUTCOME_TIMEOUT: &str = "timeout";
const OUTCOME_CANCELLATION: &str = "cancellation";
const OUTCOME_MALFORMED_SUMMARY: &str = "malformed_summary";
const OUTCOME_DENIED_TOOL: &str = "denied_tool";
const OUTCOME_PARTIAL_PARALLEL_SUCCESS: &str = "partial_parallel_success";
const OUTCOME_WRITE_CONFLICT: &str = "write_conflict";
const MIN_TURN_COUNT: u32 = 1;
const MIN_DETAIL_BOUND_CHARS: u32 = 1;

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

/// Error raised while validating recipe-backed subagent data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubagentError {
    /// Generic adapter contract validation failed.
    Contract(crate::AdapterContractError),
    /// An enum-like subagent field contained an unsupported value.
    UnsupportedValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: Text,
    },
    /// A required collection was empty.
    EmptyCollection {
        /// Boundary collection field name.
        field: &'static str,
    },
    /// A collection contained a duplicate value.
    DuplicateValue {
        /// Boundary collection field name.
        field: &'static str,
        /// Duplicated normalized value.
        value: Text,
    },
    /// Delegation attempted automatic spawning.
    ImplicitDelegationDenied,
    /// Delegation attempted recursive subagent spawning.
    RecursiveDelegationDenied,
    /// Automation delegation was not admitted by policy.
    AutomationDelegationDenied,
    /// A recipe reference was missing an admitted trust receipt.
    UntrustedRecipeDenied,
    /// Turn ceiling was zero or unbounded.
    InvalidTurnCeiling,
    /// A requested capability was denied by an intersection input.
    CapabilityDenied {
        /// Requested capability label.
        capability: Text,
    },
    /// Full detail handoff omitted its bounded detail reference.
    MissingDetailReference,
    /// Full detail handoff exceeded its configured bound.
    DetailBoundsExceeded,
    /// External subagent requested ambient CLI discovery.
    AmbientExternalDiscoveryDenied,
    /// External subagent requested inherited credentials.
    InheritedCredentialDenied,
    /// External subagent descriptor omitted an injected service label.
    MissingExternalService,
}

impl From<crate::AdapterContractError> for SubagentError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

mod access;
mod enums;
mod external;
mod parent;
mod request;
mod summary;

#[cfg(test)]
mod tests;

pub use access::SubagentCapabilityDenial;
pub use access::SubagentCapabilityInput;
pub use access::SubagentCapabilityResolution;
pub use access::resolve_subagent_capabilities;
pub use enums::ExternalSubagentProtocol;
pub use enums::SubagentDelegationSource;
pub use enums::SubagentReturnMode;
pub use external::ExternalSubagentDescriptor;
pub use external::ExternalSubagentDescriptorDraft;
pub use parent::SubagentHandoff;
pub use parent::SubagentHandoffDraft;
pub use parent::build_subagent_handoff;
pub use request::SubagentDelegation;
pub use request::SubagentDelegationDraft;
pub use summary::SubagentAggregateReport;
pub use summary::SubagentEvent;
pub use summary::SubagentEventDraft;
pub use summary::SubagentEventKind;
pub use summary::SubagentOutcomeDraft;
pub use summary::SubagentOutcomeKind;
pub use summary::aggregate_subagent_outcomes;
fn parse_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, SubagentError> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or(SubagentError::UnsupportedValue {
        field: field_name.as_str(),
        value: normalized,
    })
}

fn required_text(field_name: crate::validation::FieldName, value: &str) -> Result<Text, SubagentError> {
    crate::validation::normalized_adapter_field(field_name, value).map_err(SubagentError::from)
}

fn optional_text(field_name: crate::validation::FieldName, value: Option<&str>) -> Result<Option<Text>, SubagentError> {
    value.map(|text| required_text(field_name, text)).transpose()
}

fn text_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
    required: bool,
) -> Result<Collection<Text>, SubagentError> {
    if required && values.is_empty() {
        return Err(SubagentError::EmptyCollection {
            field: field_name.as_str(),
        });
    }
    let mut normalized = Collection::with_capacity(values.len());
    for value in values {
        let text = required_text(field_name, value)?;
        if normalized.iter().any(|existing| existing == &text) {
            return Err(SubagentError::DuplicateValue {
                field: field_name.as_str(),
                value: text,
            });
        }
        normalized.push(text);
    }
    Ok(normalized)
}

#[derive(Clone, Copy)]
struct CapabilitySetCheck<'a> {
    capability: &'a str,
    parent_policy: &'a [Text],
    recipe_or_profile: &'a [Text],
    workspace_scope: &'a [Text],
    host_admission: &'a [Text],
}

fn contains_all(input: CapabilitySetCheck<'_>) -> bool {
    contains_text(input.parent_policy, input.capability)
        && contains_text(input.recipe_or_profile, input.capability)
        && contains_text(input.workspace_scope, input.capability)
        && contains_text(input.host_admission, input.capability)
}

fn contains_text(values: &[Text], needle: &str) -> bool {
    values.iter().any(|value| value.as_str() == needle)
}
