use super::core::{error, hash_fields, is_blake3_identity};
use super::data::{
    TOOL_EXECUTION_STAGE_CONTRACT_IDENTITY, TOOL_EXECUTION_STAGE_NON_CLAIMS,
    TOOL_EXECUTION_STAGE_RECEIPT_SCHEMA, TOOL_EXECUTION_STAGE_REDACTION_MARKER,
    ToolExecutionStageError, ToolExecutionStageIssue, ToolExecutionStageOutcomeClass,
    ToolExecutionStagePlan, ToolExecutionStageReceipt, ToolExecutionStageSensitiveDraft,
    ToolExecutionStageState, ToolExecutionStageStatus,
};

const RECEIPT_IDENTITY_DOMAIN: &str = "animus.tool-execution-stages.receipt.v1";
const MAX_SAFE_LABEL_CHARS: usize = 128;
const PRIVATE_MARKERS: &[&str] = &[
    "RAW_ARGUMENTS:",
    "RAW_OUTPUT:",
    "RAW_HANDLER_ERROR:",
    "PROVIDER_PAYLOAD:",
    "CREDENTIAL:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
    "/home/",
    "/Users/",
    ":\\",
];

// r[impl agent_harness_runtime.tool_execution_stages.commit]
/// Projects one terminal metadata-only staged tool receipt.
///
/// # Errors
///
/// Returns a typed issue when execution is not terminal, private material was
/// supplied, counts overflow, or safe receipt fields are inconsistent.
pub fn project_tool_execution_stage_receipt(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    sensitive: ToolExecutionStageSensitiveDraft<'_>,
) -> Result<ToolExecutionStageReceipt, ToolExecutionStageError> {
    reject_sensitive(sensitive)?;
    if state.stage_plan_identity != plan.plan_identity
        || !matches!(
            state.status,
            ToolExecutionStageStatus::Denied | ToolExecutionStageStatus::Observed
        )
    {
        return Err(error(ToolExecutionStageIssue::UnsafeEvidence));
    }
    let handler_count = u32::try_from(plan.handlers.len())
        .map_err(|_overflow| error(ToolExecutionStageIssue::UnsafeEvidence))?;
    let mut receipt = ToolExecutionStageReceipt {
        schema: TOOL_EXECUTION_STAGE_RECEIPT_SCHEMA,
        contract_identity: TOOL_EXECUTION_STAGE_CONTRACT_IDENTITY,
        stage_plan_identity: plan.plan_identity.clone(),
        tool_name: plan.tool_name.clone(),
        schema_identity: plan.schema_identity.clone(),
        request_identity: plan.request_identity.clone(),
        authority_seal_identity: plan.authority_seal_identity.clone(),
        commit_owner_identity: plan.commit_owner_identity.clone(),
        status: state.status,
        outcome_class: state.outcome_class,
        handler_count,
        result_identity: state.result_identity.clone(),
        executor_receipt_identity: state.executor_receipt_identity.clone(),
        durable_head_identity: state.durable_head_identity.clone(),
        durable_commit_identity: state.durable_commit_identity.clone(),
        required_redaction: state.required_redaction,
        truncated: state.truncated,
        redaction_marker: TOOL_EXECUTION_STAGE_REDACTION_MARKER,
        non_claims: TOOL_EXECUTION_STAGE_NON_CLAIMS.to_vec(),
        receipt_identity: String::new(),
    };
    receipt.receipt_identity = receipt_identity(&receipt);
    validate_tool_execution_stage_receipt(&receipt)?;
    Ok(receipt)
}

/// Validates one metadata-only staged tool receipt.
///
/// # Errors
///
/// Returns a typed issue for malformed identities, unsafe labels, impossible
/// terminal combinations, changed non-claims, or receipt identity drift.
pub fn validate_tool_execution_stage_receipt(
    receipt: &ToolExecutionStageReceipt,
) -> Result<(), ToolExecutionStageError> {
    let scalar_fields_valid = receipt.schema == TOOL_EXECUTION_STAGE_RECEIPT_SCHEMA
        && receipt.contract_identity == TOOL_EXECUTION_STAGE_CONTRACT_IDENTITY
        && safe_label(receipt.tool_name.as_str())
        && safe_label(receipt.schema_identity.as_str())
        && receipt.redaction_marker == TOOL_EXECUTION_STAGE_REDACTION_MARKER
        && receipt.non_claims == TOOL_EXECUTION_STAGE_NON_CLAIMS;
    if !scalar_fields_valid {
        return Err(error(ToolExecutionStageIssue::UnsafeEvidence));
    }
    for identity in [
        Some(receipt.stage_plan_identity.as_str()),
        Some(receipt.request_identity.as_str()),
        Some(receipt.authority_seal_identity.as_str()),
        Some(receipt.commit_owner_identity.as_str()),
        receipt.result_identity.as_deref(),
        receipt.executor_receipt_identity.as_deref(),
        receipt.durable_head_identity.as_deref(),
        receipt.durable_commit_identity.as_deref(),
        Some(receipt.receipt_identity.as_str()),
    ]
    .into_iter()
    .flatten()
    {
        if !is_blake3_identity(identity) {
            return Err(error(ToolExecutionStageIssue::UnsafeEvidence));
        }
    }
    let terminal_fields_valid = match receipt.status {
        ToolExecutionStageStatus::Denied => {
            receipt.durable_head_identity.is_none()
                && receipt.durable_commit_identity.is_none()
                && receipt.outcome_class != ToolExecutionStageOutcomeClass::Succeeded
        }
        ToolExecutionStageStatus::Observed => {
            receipt.result_identity.is_some()
                && receipt.executor_receipt_identity.is_some()
                && receipt.durable_head_identity.is_some()
                && receipt.durable_commit_identity.is_some()
        }
        ToolExecutionStageStatus::Active
        | ToolExecutionStageStatus::AwaitingCommit
        | ToolExecutionStageStatus::Committed => false,
    };
    if !terminal_fields_valid || receipt.receipt_identity != receipt_identity(receipt) {
        return Err(error(ToolExecutionStageIssue::UnsafeEvidence));
    }
    Ok(())
}

fn reject_sensitive(
    sensitive: ToolExecutionStageSensitiveDraft<'_>,
) -> Result<(), ToolExecutionStageError> {
    if [
        sensitive.raw_arguments,
        sensitive.raw_output,
        sensitive.narrowed_output,
        sensitive.credential,
        sensitive.private_path,
        sensitive.provider_payload,
        sensitive.raw_handler_error,
    ]
    .into_iter()
    .any(|value| value.is_some())
    {
        return Err(error(ToolExecutionStageIssue::UnsafeEvidence));
    }
    Ok(())
}

fn receipt_identity(receipt: &ToolExecutionStageReceipt) -> String {
    let status = match receipt.status {
        ToolExecutionStageStatus::Active => "active",
        ToolExecutionStageStatus::Denied => "denied",
        ToolExecutionStageStatus::AwaitingCommit => "awaiting_commit",
        ToolExecutionStageStatus::Committed => "committed",
        ToolExecutionStageStatus::Observed => "observed",
    };
    let handler_count = receipt.handler_count.to_string();
    let required_redaction = receipt.required_redaction.to_string();
    let truncated = receipt.truncated.to_string();
    hash_fields(
        RECEIPT_IDENTITY_DOMAIN,
        &[
            receipt.schema.as_bytes(),
            receipt.contract_identity.as_bytes(),
            receipt.stage_plan_identity.as_bytes(),
            receipt.tool_name.as_bytes(),
            receipt.schema_identity.as_bytes(),
            receipt.request_identity.as_bytes(),
            receipt.authority_seal_identity.as_bytes(),
            receipt.commit_owner_identity.as_bytes(),
            status.as_bytes(),
            receipt.outcome_class.as_str().as_bytes(),
            handler_count.as_bytes(),
            receipt
                .result_identity
                .as_deref()
                .unwrap_or("none")
                .as_bytes(),
            receipt
                .executor_receipt_identity
                .as_deref()
                .unwrap_or("none")
                .as_bytes(),
            receipt
                .durable_head_identity
                .as_deref()
                .unwrap_or("none")
                .as_bytes(),
            receipt
                .durable_commit_identity
                .as_deref()
                .unwrap_or("none")
                .as_bytes(),
            required_redaction.as_bytes(),
            truncated.as_bytes(),
            receipt.redaction_marker.as_bytes(),
        ],
    )
}

fn safe_label(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.chars().count() <= MAX_SAFE_LABEL_CHARS
        && !PRIVATE_MARKERS
            .iter()
            .any(|marker| trimmed.contains(marker))
        && !trimmed.contains("http://")
        && !trimmed.contains("https://")
}
