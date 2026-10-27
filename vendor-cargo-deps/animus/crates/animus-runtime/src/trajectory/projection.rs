use std::collections::BTreeMap;

use animus_adapters::{
    LedgerEntry,
    trajectory::{
        TRAJECTORY_HELPER_BLAKE3, TRAJECTORY_PACKAGE, TRAJECTORY_REVISION,
        TRAJECTORY_SCHEMA_BLAKE3, TRAJECTORY_VERSION, TrajectoryContentDisposition,
        TrajectoryPolicy, TrajectoryRecord,
    },
};

use super::{
    TrajectoryBatchDraft, TrajectoryIssue, TrajectoryOmission, TrajectoryProjection,
    TrajectoryReceipt, TrajectoryReceiptInput, TrajectoryRole, TrajectoryValidation,
};

const RECEIPT_SCHEMA: &str = "animus.trajectory-transcript-receipt.v1";
const PROJECTION_TARGET: &str = "ledger-and-batch-drafts";
const MAX_SERVICE_IDENTITY_BYTES: usize = 128;
const REQUIRED_NON_CLAIMS: [&str; 9] = [
    "not-source-authenticity",
    "not-transcript-completeness",
    "not-timestamp-truth",
    "not-tool-execution-truth",
    "not-semantic-equivalence",
    "not-privacy-proof",
    "not-model-correctness",
    "not-dataset-export-fitness",
    "not-release-eligibility",
];

/// Projects validated records into existing neutral ledger entries and batch drafts.
///
/// This function performs no persistence, dataset export, evaluation, or host effects.
///
/// # Errors
///
/// Returns a stable issue if a projected field violates an existing ledger contract
/// or an ordered position cannot fit in the bounded draft type.
pub fn project_trajectory(
    policy: &TrajectoryPolicy,
    validation: &TrajectoryValidation,
) -> Result<TrajectoryProjection, TrajectoryIssue> {
    let mut ledger_entries = Vec::new();
    let mut batch_drafts = Vec::with_capacity(validation.records.len());
    let mut omissions = vec![
        TrajectoryOmission::Reasoning,
        TrajectoryOmission::SourceMetadata,
    ];
    if policy.tool_arguments == TrajectoryContentDisposition::Omit {
        omissions.push(TrajectoryOmission::ToolArguments);
    }
    if policy.tool_results == TrajectoryContentDisposition::Omit {
        omissions.push(TrajectoryOmission::ToolResults);
    }

    let tool_names = collect_tool_names(validation);
    for (position, record) in validation.records.iter().enumerate() {
        let identity = projection_identity(validation, position, record);
        let role = record_role(record);
        let timestamp = record_timestamp(record).map(String::from);
        let content_omitted = content_omitted(policy, record);
        batch_drafts.push(TrajectoryBatchDraft {
            identity: identity.clone(),
            position: u32::try_from(position).map_err(|_error| TrajectoryIssue::ResultBound)?,
            role,
            descriptive_timestamp: timestamp,
            content_omitted,
        });
        if let Some(entry) = ledger_entry(policy, record, identity.as_str(), &tool_names)? {
            ledger_entries.push(entry);
        }
    }
    omissions.sort();
    omissions.dedup();
    Ok(TrajectoryProjection {
        ledger_entries,
        batch_drafts,
        omissions,
    })
}

/// Builds one metadata-only receipt over validated and projected facts.
///
/// # Errors
///
/// Returns [`TrajectoryIssue::Receipt`] if the caller supplies sensitive material,
/// unsafe claim flags, or an invalid service identity.
pub fn trajectory_receipt(
    input: TrajectoryReceiptInput<'_>,
) -> Result<TrajectoryReceipt, TrajectoryIssue> {
    if input.helper_identity != TRAJECTORY_HELPER_BLAKE3
        || !safe_service_identity(input.service_identity)
        || input.sensitive.transcript.is_some()
        || input.sensitive.reasoning.is_some()
        || input.sensitive.tool_body.is_some()
        || input.sensitive.diagnostic_message.is_some()
        || input.sensitive.path_metadata.is_some()
        || input.sensitive.credential.is_some()
        || input.sensitive.provider_payload.is_some()
        || input.sensitive.content_fingerprint.is_some()
        || input.sensitive.helper_output.is_some()
        || input.sensitive.claims_truth
    {
        return Err(TrajectoryIssue::Receipt);
    }
    let cohort_identity = cohort_identity();
    let receipt_identity = receipt_identity(cohort_identity.as_str(), &input);
    Ok(TrajectoryReceipt {
        schema: RECEIPT_SCHEMA,
        cohort_identity,
        policy_identity: input.validation.policy_identity.clone(),
        helper_identity: String::from(input.helper_identity),
        service_identity: String::from(input.service_identity),
        source: input.validation.source,
        role_counts: input.validation.role_counts,
        diagnostic_counts: input.validation.diagnostic_counts.clone(),
        disposition: input.validation.disposition,
        omissions: input.projection.omissions.clone(),
        projection_target: PROJECTION_TARGET,
        non_claims: REQUIRED_NON_CLAIMS,
        receipt_identity,
    })
}

fn collect_tool_names(validation: &TrajectoryValidation) -> BTreeMap<String, String> {
    let mut names = BTreeMap::new();
    for record in &validation.records {
        if let TrajectoryRecord::AssistantToolCalls { tool_calls, .. } = record {
            for call in tool_calls {
                names.insert(call.id.clone(), call.name.clone());
            }
        }
    }
    names
}

fn ledger_entry(
    policy: &TrajectoryPolicy,
    record: &TrajectoryRecord,
    identity: &str,
    tool_names: &BTreeMap<String, String>,
) -> Result<Option<LedgerEntry>, TrajectoryIssue> {
    let entry = match record {
        TrajectoryRecord::User { content, .. } => Some(LedgerEntry::user(identity, content)),
        TrajectoryRecord::Assistant { content, .. } => {
            Some(LedgerEntry::assistant(identity, content))
        }
        TrajectoryRecord::Tool {
            tool_call_id,
            content,
            ..
        } if policy.tool_results == TrajectoryContentDisposition::Include => {
            let tool_name = tool_names
                .get(tool_call_id)
                .ok_or(TrajectoryIssue::ToolResult)?;
            Some(LedgerEntry::tool(identity, tool_name.as_str(), content))
        }
        TrajectoryRecord::Meta { .. }
        | TrajectoryRecord::Reasoning { .. }
        | TrajectoryRecord::AssistantToolCalls { .. }
        | TrajectoryRecord::Tool { .. } => None,
    };
    entry
        .transpose()
        .map_err(|_error| TrajectoryIssue::LedgerProjection)
}

fn record_role(record: &TrajectoryRecord) -> TrajectoryRole {
    match record {
        TrajectoryRecord::Meta { .. } => TrajectoryRole::Meta,
        TrajectoryRecord::User { .. } => TrajectoryRole::User,
        TrajectoryRecord::Reasoning { .. } => TrajectoryRole::Reasoning,
        TrajectoryRecord::Assistant { .. } => TrajectoryRole::Assistant,
        TrajectoryRecord::AssistantToolCalls { .. } => TrajectoryRole::AssistantToolCalls,
        TrajectoryRecord::Tool { .. } => TrajectoryRole::Tool,
    }
}

fn record_timestamp(record: &TrajectoryRecord) -> Option<&str> {
    match record {
        TrajectoryRecord::Meta { .. } => None,
        TrajectoryRecord::User { timestamp, .. }
        | TrajectoryRecord::Reasoning { timestamp, .. }
        | TrajectoryRecord::Assistant { timestamp, .. }
        | TrajectoryRecord::AssistantToolCalls { timestamp, .. }
        | TrajectoryRecord::Tool { timestamp, .. } => Some(timestamp.as_str()),
    }
}

fn content_omitted(policy: &TrajectoryPolicy, record: &TrajectoryRecord) -> bool {
    match record {
        TrajectoryRecord::Meta { .. } | TrajectoryRecord::Reasoning { .. } => true,
        TrajectoryRecord::AssistantToolCalls { .. } => {
            policy.tool_arguments == TrajectoryContentDisposition::Omit
        }
        TrajectoryRecord::Tool { .. } => policy.tool_results == TrajectoryContentDisposition::Omit,
        TrajectoryRecord::User { .. } | TrajectoryRecord::Assistant { .. } => false,
    }
}

fn projection_identity(
    validation: &TrajectoryValidation,
    position: usize,
    record: &TrajectoryRecord,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"animus.trajectory-projection.v1\0");
    hasher.update(validation.source_generation_identity.as_bytes());
    hasher.update(&position.to_le_bytes());
    hash_record(&mut hasher, record);
    hasher.finalize().to_hex().to_string()
}

fn hash_record(hasher: &mut blake3::Hasher, record: &TrajectoryRecord) {
    hasher.update(record_role(record).as_str().as_bytes());
    match record {
        TrajectoryRecord::Meta {
            source,
            cwd,
            git_branch,
            model,
        } => {
            hash_text(hasher, source);
            hash_optional(hasher, cwd.as_deref());
            hash_optional(hasher, git_branch.as_deref());
            hash_optional(hasher, model.as_deref());
        }
        TrajectoryRecord::User { content, timestamp }
        | TrajectoryRecord::Reasoning { content, timestamp }
        | TrajectoryRecord::Assistant { content, timestamp } => {
            hash_text(hasher, content);
            hash_text(hasher, timestamp);
        }
        TrajectoryRecord::AssistantToolCalls {
            tool_calls,
            timestamp,
        } => {
            hash_text(hasher, timestamp);
            for call in tool_calls {
                hash_text(hasher, call.id.as_str());
                hash_text(hasher, call.name.as_str());
                hash_text(hasher, call.arguments.as_str());
            }
        }
        TrajectoryRecord::Tool {
            tool_call_id,
            content,
            timestamp,
        } => {
            hash_text(hasher, tool_call_id);
            hash_text(hasher, content);
            hash_text(hasher, timestamp);
        }
    }
}

fn hash_text(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&value.len().to_le_bytes());
    hasher.update(value.as_bytes());
}

fn hash_optional(hasher: &mut blake3::Hasher, value: Option<&str>) {
    match value {
        Some(value) => {
            hasher.update(&[1]);
            hash_text(hasher, value);
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

fn cohort_identity() -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"animus.trajectory-cohort.v1\0");
    for value in [
        TRAJECTORY_PACKAGE,
        TRAJECTORY_VERSION,
        TRAJECTORY_REVISION,
        TRAJECTORY_SCHEMA_BLAKE3,
        TRAJECTORY_HELPER_BLAKE3,
    ] {
        hash_text(&mut hasher, value);
    }
    hasher.finalize().to_hex().to_string()
}

fn receipt_identity(cohort: &str, input: &TrajectoryReceiptInput<'_>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"animus.trajectory-receipt.v1\0");
    for value in [
        cohort,
        input.validation.policy_identity.as_str(),
        input.helper_identity,
        input.service_identity,
        input.validation.source.as_str(),
    ] {
        hash_text(&mut hasher, value);
    }
    for count in [
        input.validation.role_counts.meta,
        input.validation.role_counts.user,
        input.validation.role_counts.reasoning,
        input.validation.role_counts.assistant,
        input.validation.role_counts.assistant_tool_calls,
        input.validation.role_counts.tool,
    ] {
        hasher.update(&count.to_le_bytes());
    }
    for (code, count) in &input.validation.diagnostic_counts {
        hash_text(&mut hasher, code.as_str());
        hasher.update(&count.to_le_bytes());
    }
    for omission in &input.projection.omissions {
        hash_text(&mut hasher, omission.as_str());
    }
    hasher.finalize().to_hex().to_string()
}

fn safe_service_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SERVICE_IDENTITY_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_' | b'.'))
}
