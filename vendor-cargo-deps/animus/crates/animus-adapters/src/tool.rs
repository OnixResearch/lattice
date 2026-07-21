//! Pure tool adapter contracts, validation, output bounding, mutation planning, and dispatch gates.

mod args;
mod defs;
mod fault;
mod gate;
pub mod hashed;
mod limit;
mod message;
mod planning;
mod schedule;

#[cfg(test)]
mod tests;

pub use args::{
    ArgumentCompatibilityRule, ArgumentField, ArgumentFieldDraft, ArgumentPreparationRequest,
    PreparedArguments,
};
pub use defs::{
    Definition, DefinitionDraft, DefinitionStatus, PromptContextEntry, PromptContextKind,
    build_prompt_context,
};
pub use fault::ContractError;
pub use gate::{
    CapabilityStatus, DispatchDecision, DispatchRequest, DispatchStatus, ExecutorStatus,
    FailureClass, OutputBoundaryStatus, evaluate_dispatch,
};
pub use limit::{
    BoundedOutput, BoundedOutputMetadataDraft, HashRequirement, OutputDraft, OutputPolicy,
    OutputStatus,
};
pub use message::{ApprovalPosture, OutputRedactionClass, Request, RequestDraft, Response};
// Internal planning names stay short under `tool::planning`; these aliases preserve
// the root-level public DTO names documented for embedders.
#[cfg_attr(animus_octet, allow(tigerstyle::renamed_imports))]
pub use planning::{
    DiscoveryCeilings as ToolDiscoveryCeilings, DiscoveryDetail as ToolDiscoveryDetail,
    DiscoveryRequestDraft as ToolDiscoveryRequestDraft, DiscoverySet as ToolDiscoverySet,
    DiscoverySummary as ToolDiscoverySummary, Plan as ToolPlan,
    PlanAdmissionContext as ToolPlanAdmissionContext, PlanBatchPolicy as ToolPlanBatchPolicy,
    PlanCancellationPosture as ToolPlanCancellationPosture, PlanDraft as ToolPlanDraft,
    PlanFabricAdmission as ToolPlanFabricAdmission, PlanFabricDraft as ToolPlanFabricDraft,
    PlanFabricIssue as ToolPlanFabricIssue, PlanFabricKind as ToolPlanFabricKind,
    PlanFabricStatus as ToolPlanFabricStatus,
    PlanIntermediateBinding as ToolPlanIntermediateBinding,
    PlanIntermediateBindingDraft as ToolPlanIntermediateBindingDraft,
    PlanIntermediateKind as ToolPlanIntermediateKind,
    PlanIntermediateUse as ToolPlanIntermediateUse, PlanIssue as ToolPlanIssue,
    PlanReceipt as ToolPlanReceipt, PlanReceiptDraft as ToolPlanReceiptDraft,
    PlanStatus as ToolPlanStatus, PlanStepAdmission as ToolPlanStepAdmission,
    PlanStepDraft as ToolPlanStepDraft, PlanTimeoutPosture as ToolPlanTimeoutPosture,
    TOOL_DISCOVERY_CONTRACT_IDENTITY, TOOL_PLAN_CONTRACT_IDENTITY,
    TOOL_PLAN_RECEIPT_CONTRACT_IDENTITY, admit_plan as admit_tool_plan,
    admit_plan_fabric as admit_tool_plan_fabric, discover_tools,
};
pub use schedule::{
    MutationCallDraft, MutationWindow, MutationWindowStatus, plan_mutation_windows,
};

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

const NAME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("tool_name");
const OUTPUT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("tool_output");
const LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("tool_label");
const DESCRIPTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_description");
const SCHEMA_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_parameter_schema_identity");
const PROMPT_SNIPPET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_prompt_snippet");
const PROMPT_GUIDELINE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_prompt_guideline");
const SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_source_identity");
const PROVENANCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_provenance");
const CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_capability_label");
const RESULT_SHAPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_result_metadata_shape");
const ARGUMENT_NAME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_argument_name");
const ARGUMENT_VALUE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_argument_value");
const MUTATION_CALL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_mutation_call_id");
const MUTATION_TARGET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_mutation_target");
const OUTPUT_REFERENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_output_reference");
const OUTPUT_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_output_content_hash");
const CORRELATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_call_correlation_id");
const APPROVAL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_approval_posture");

const MAX_TOOL_METADATA_ITEMS: usize = 32;
const MIN_VISIBLE_OUTPUT_CHARS: u32 = 1;
const MAX_VISIBLE_OUTPUT_CHARS: u32 = 8_192;
const FIRST_COLLECTION_INDEX: usize = 0;
const ARGUMENT_SUMMARY_SEPARATOR: &str = ",";
const ARGUMENT_SUMMARY_ASSIGNMENT: &str = "=";
const APPROVAL_PRE_APPROVED: &str = "pre_approved";
const APPROVAL_REQUIRES_APPROVAL: &str = "requires_approval";
const APPROVAL_DENIED: &str = "denied";
const REDACTION_SAFE: &str = "safe";
const REDACTION_REDACTED: &str = "redacted";
const REDACTION_SECRET_ADJACENT: &str = "secret_adjacent";
// r[impl agent_harness_runtime.boundary_versions.contract_identities]
/// Stable contract identity for public tool request DTOs.
pub const TOOL_REQUEST_CONTRACT_IDENTITY: &str = "animus.tool-request.v1";
/// Stable contract identity for public tool response DTOs.
pub const TOOL_RESPONSE_CONTRACT_IDENTITY: &str = "animus.tool-response.v1";

const HASH_SCHEMA: &str = "animus.pi-tool-contract.v1";
const HASH_SEPARATOR: &str = "|";
const ABSOLUTE_PATH_PREFIX: &str = "/";
const HOME_PATH_PREFIX: &str = "~";
const URI_SCHEME_MARKER: &str = "://";
const WINDOWS_SEPARATOR_MARKER: char = '\\';
const NUL_CHARACTER: char = '\0';

const _: () = assert!(MAX_TOOL_METADATA_ITEMS > 0);
const _: () = assert!(MAX_VISIBLE_OUTPUT_CHARS >= MIN_VISIBLE_OUTPUT_CHARS);

fn required_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, ContractError> {
    match value {
        Some(raw) => required_text_value(field, raw),
        None => Err(ContractError::MissingField {
            field: field.as_str(),
        }),
    }
}

fn required_text_value(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<Text, ContractError> {
    crate::validation::normalized_adapter_field(field, value).map_err(ContractError::from)
}

fn required_texts(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, ContractError> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: field.as_str(),
        }
        .into());
    }
    collect_texts(field, values)
}

fn normalized_label_set(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, ContractError> {
    let mut labels = collect_texts(field, values)?;
    labels.sort();
    labels.dedup();
    Ok(labels)
}

fn collect_texts(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, ContractError> {
    validate_item_count(field, values.len())?;
    let mut normalized = Collection::with_capacity(values.len());
    for value in values {
        normalized.push(required_text_value(field, value)?);
    }
    Ok(normalized)
}

fn validate_item_count(
    field: crate::validation::FieldName,
    item_count: usize,
) -> Result<(), ContractError> {
    if item_count > MAX_TOOL_METADATA_ITEMS {
        return Err(ContractError::TooManyItems {
            field: field.as_str(),
            max_items: MAX_TOOL_METADATA_ITEMS,
            actual_items: item_count,
        });
    }
    Ok(())
}

fn validate_no_nul(field: crate::validation::FieldName, value: &str) -> Result<(), ContractError> {
    if let Some((byte_index, _character)) = value
        .char_indices()
        .find(|(_byte_index, character)| *character == NUL_CHARACTER)
    {
        return Err(crate::AdapterContractError::NulCharacter {
            field: field.as_str(),
            byte_index,
        }
        .into());
    }
    Ok(())
}

fn hash_parts(parts: &[&str]) -> Text {
    let mut text = Text::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    alloc::format!("{}", blake3::hash(text.as_bytes()).to_hex())
}

fn usize_to_u64(value: usize) -> u64 {
    value as u64
}
