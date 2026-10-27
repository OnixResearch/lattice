// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
use super::{
    BLAKE3_ALGORITHM, DATASET_HASH_DOMAIN, FIELD_POLICY, FIELD_REPLAY, FIELD_ROW,
    HASH_PART_SEPARATOR, REQUIRED_COUNT, ROLE_ASSISTANT, ROLE_SYSTEM, ROLE_TOOL, ROLE_USER,
    data::{
        DatasetChatMessage, DatasetChatRow, DatasetDiagnostic, DatasetDiagnosticKind,
        DatasetExportPolicy, DatasetExportResult, DatasetFingerprint, DatasetInspectionReport,
        DatasetMetadataProjection, DatasetPreferenceExample, DatasetPreferenceRow,
        DatasetReplayRole, DatasetRlStepKind, DatasetRlTrajectory, DatasetRlTrajectoryRow,
        DatasetSessionSource, DatasetSkippedRow, DatasetTarget,
    },
};

const RAW_ENVIRONMENT_MARKER: &str = "env_secret:";
const RAW_PRIVATE_MARKER: &str = "/home/";
const RAW_PROVENANCE_MARKER: &str = "private_provenance:";
const RAW_REASONING_MARKER: &str = "hidden_reasoning:";
const RAW_TOOL_MARKER: &str = "tool_output:";

/// Computes a dataset export BLAKE3 fingerprint.
#[must_use]
pub fn blake3_dataset_export_fingerprint(label: &str, bytes: &[u8]) -> DatasetFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DATASET_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    DatasetFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Exports a Hugging Face-compatible chat messages row.
///
/// # Errors
///
/// Returns diagnostics for invalid session sources, empty conversations, invalid
/// roles, empty assistant targets, unsupported content, malformed tool ordering,
/// or leakage rejected by policy.
pub fn export_chat_messages_row(
    source: &DatasetSessionSource,
    policy: &DatasetExportPolicy,
) -> DatasetExportResult<DatasetChatRow> {
    validate_chat_source(source)?;
    let mut messages = Vec::new();
    let mut source_spans = Vec::new();
    let mut saw_assistant = false;
    let mut tool_call_open = false;
    for entry in &source.replay {
        match entry.role {
            DatasetReplayRole::System => {
                if policy.system_text == DatasetMetadataProjection::SafeLabels {
                    reject_private_payload(&entry.content)?;
                    push_message(&mut messages, ROLE_SYSTEM, &entry.content);
                    source_spans.push(entry.source_span.clone());
                }
            }
            DatasetReplayRole::User => {
                reject_private_payload(&entry.content)?;
                push_message(&mut messages, ROLE_USER, &entry.content);
                source_spans.push(entry.source_span.clone());
            }
            DatasetReplayRole::Assistant => {
                reject_private_payload(&entry.content)?;
                if entry.content.chars().count() < policy.min_assistant_chars {
                    return Err(vec![DatasetDiagnostic::new(
                        DatasetDiagnosticKind::BadChatRow,
                        FIELD_ROW,
                        "assistant target is empty or below quality threshold",
                    )]);
                }
                push_message(&mut messages, ROLE_ASSISTANT, &entry.content);
                source_spans.push(entry.source_span.clone());
                saw_assistant = true;
                tool_call_open = false;
            }
            DatasetReplayRole::ToolCall => {
                if policy.tool_calls == DatasetMetadataProjection::SafeLabels {
                    push_message(&mut messages, ROLE_ASSISTANT, "[tool-call]");
                    source_spans.push(entry.source_span.clone());
                    tool_call_open = true;
                }
            }
            DatasetReplayRole::ToolResult => {
                if policy.tool_results == DatasetMetadataProjection::SafeLabels {
                    if !tool_call_open {
                        return Err(vec![DatasetDiagnostic::new(
                            DatasetDiagnosticKind::BadChatRow,
                            FIELD_REPLAY,
                            "tool result appears without an admitted tool call",
                        )]);
                    }
                    push_message(&mut messages, ROLE_TOOL, "[tool-result-redacted]");
                    source_spans.push(entry.source_span.clone());
                    tool_call_open = false;
                }
            }
            DatasetReplayRole::HiddenReasoning => {
                if policy.hidden_reasoning == DatasetMetadataProjection::SafeLabels {
                    push_message(&mut messages, ROLE_ASSISTANT, "[reasoning-redacted]");
                    source_spans.push(entry.source_span.clone());
                }
            }
            DatasetReplayRole::Unsupported => {
                return Err(vec![DatasetDiagnostic::new(
                    DatasetDiagnosticKind::BadChatRow,
                    FIELD_REPLAY,
                    "unsupported replay content block",
                )]);
            }
        }
    }
    if messages.is_empty() || !saw_assistant {
        return Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::BadChatRow,
            FIELD_ROW,
            "chat export requires messages and an assistant target",
        )]);
    }
    Ok(DatasetChatRow {
        row_id: row_id(&source.session_id, &source.branch_id, &messages),
        messages,
        source_spans,
    })
}

/// Exports a validated preference row.
///
/// # Errors
///
/// Returns diagnostics for missing prompt, duplicate chosen/rejected candidates,
/// missing outcome labels, or unsafe metadata.
pub fn export_preference_row(
    example: &DatasetPreferenceExample,
) -> DatasetExportResult<DatasetPreferenceRow> {
    if example.prompt.is_empty()
        || example.prompt.iter().any(invalid_chat_prompt_message)
        || example.chosen.trim().is_empty()
        || example.rejected.trim().is_empty()
        || example.chosen == example.rejected
        || example.outcome_label.trim().is_empty()
        || example
            .metadata
            .iter()
            .any(|item| contains_private_marker(item))
    {
        return Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::MalformedPreference,
            FIELD_ROW,
            "preference row requires prompt, distinct candidates, outcome label, and safe metadata",
        )]);
    }
    Ok(DatasetPreferenceRow {
        row_id: blake3_dataset_export_fingerprint("preference", example.outcome_label.as_bytes()),
        prompt: example.prompt.clone(),
        chosen: example.chosen.clone(),
        rejected: example.rejected.clone(),
        metadata: example.metadata.clone(),
    })
}

/// Exports a validated RL trajectory row.
///
/// # Errors
///
/// Returns diagnostics for missing rewards, missing terminal outcome, orphaned
/// observations, invalid action links, or unsafe metadata.
pub fn export_rl_trajectory_row(
    trajectory: &DatasetRlTrajectory,
) -> DatasetExportResult<DatasetRlTrajectoryRow> {
    let has_prompt = trajectory
        .steps
        .iter()
        .any(|step| step.kind == DatasetRlStepKind::Prompt);
    let has_action = trajectory
        .steps
        .iter()
        .any(|step| step.kind == DatasetRlStepKind::Action);
    let action_ids: Vec<&str> = trajectory
        .steps
        .iter()
        .filter(|step| step.kind == DatasetRlStepKind::Action)
        .map(|step| step.step_id.as_str())
        .collect();
    let invalid_step = trajectory
        .steps
        .iter()
        .any(|step| step.step_id.trim().is_empty() || step.content.trim().is_empty());
    let invalid_observation_link = trajectory.steps.iter().any(|step| {
        step.kind == DatasetRlStepKind::Observation
            && step
                .link_id
                .as_deref()
                .is_none_or(|link| !action_ids.contains(&link))
    });
    let invalid_non_observation_link = trajectory
        .steps
        .iter()
        .any(|step| step.kind != DatasetRlStepKind::Observation && step.link_id.is_some());
    if !has_prompt
        || !has_action
        || invalid_step
        || invalid_observation_link
        || invalid_non_observation_link
        || trajectory
            .reward_label
            .as_ref()
            .is_none_or(|reward| reward.trim().is_empty())
        || trajectory
            .terminal_outcome
            .as_ref()
            .is_none_or(|outcome| outcome.trim().is_empty())
        || trajectory
            .metadata
            .iter()
            .any(|item| contains_private_marker(item))
    {
        return Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::IncompleteTrajectory,
            FIELD_ROW,
            "RL trajectory requires prompt, action, linked observations, reward, terminal outcome, and safe metadata",
        )]);
    }
    Ok(DatasetRlTrajectoryRow {
        row_id: blake3_dataset_export_fingerprint(
            "rl-trajectory",
            trajectory.trajectory_id.as_bytes(),
        ),
        steps: trajectory.steps.clone(),
        reward_label: trajectory.reward_label.clone().unwrap_or_default(),
        terminal_outcome: trajectory.terminal_outcome.clone().unwrap_or_default(),
        metadata: trajectory.metadata.clone(),
    })
}

/// Builds a structured dataset inspection report.
#[must_use]
pub fn inspect_dataset_exports(
    exported_rows: u32,
    diagnostics: &[DatasetDiagnostic],
    policy: &DatasetExportPolicy,
) -> DatasetInspectionReport {
    let skipped_rows = diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| DatasetSkippedRow {
            row_label: format!("row-{index}"),
            reason: format!("{:?}", diagnostic.kind),
        })
        .collect();
    let mut policy_decisions = vec![format!("hidden_reasoning={:?}", policy.hidden_reasoning)];
    policy_decisions.push(format!("tool_results={:?}", policy.tool_results));
    let quality_warning_count = count_diagnostics(diagnostics, DatasetDiagnosticKind::BadChatRow);
    let leakage_warning_count = count_diagnostics(diagnostics, DatasetDiagnosticKind::Leakage);
    DatasetInspectionReport {
        exported_rows,
        skipped_rows,
        warning_classes: diagnostics
            .iter()
            .map(|diagnostic| format!("{:?}", diagnostic.kind))
            .collect(),
        policy_decisions,
        quality_warning_count,
        leakage_warning_count,
        usage_summary: None,
    }
}

fn contains_private_marker(text: &str) -> bool {
    text.contains(RAW_ENVIRONMENT_MARKER)
        || text.contains(RAW_PRIVATE_MARKER)
        || text.contains(RAW_PROVENANCE_MARKER)
        || text.contains(RAW_REASONING_MARKER)
        || text.contains(RAW_TOOL_MARKER)
}

fn reject_private_payload(text: &str) -> DatasetExportResult<()> {
    if contains_private_marker(text) {
        Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::Leakage,
            FIELD_POLICY,
            "dataset export rejected private payload marker",
        )])
    } else {
        Ok(())
    }
}

fn invalid_chat_prompt_message(message: &DatasetChatMessage) -> bool {
    !matches!(
        message.role.as_str(),
        ROLE_SYSTEM | ROLE_USER | ROLE_ASSISTANT | ROLE_TOOL
    ) || message.content.trim().is_empty()
        || contains_private_marker(&message.content)
}

fn push_message(messages: &mut Vec<DatasetChatMessage>, role: &str, content: &str) {
    messages.push(DatasetChatMessage {
        role: role.to_string(),
        content: content.to_string(),
    });
}

fn row_id(
    session_id: &str,
    branch_id: &str,
    messages: &[DatasetChatMessage],
) -> DatasetFingerprint {
    let mut material = format!("{session_id}:{branch_id}");
    for message in messages {
        material.push_str(&message.role);
        material.push_str(&message.content);
    }
    blake3_dataset_export_fingerprint("chat-row", material.as_bytes())
}

fn validate_chat_source(source: &DatasetSessionSource) -> DatasetExportResult<()> {
    if source.target != DatasetTarget::ChatMessages
        || source.session_id.trim().is_empty()
        || source.branch_id.trim().is_empty()
        || source.replay.len() < REQUIRED_COUNT
        || source
            .replay
            .iter()
            .any(|entry| entry.entry_id.trim().is_empty() || entry.source_span.trim().is_empty())
        || has_duplicate_source_spans(source)
    {
        Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::InvalidSource,
            FIELD_REPLAY,
            "dataset export requires a validated chat session, branch, replay path, and unique source spans",
        )])
    } else {
        Ok(())
    }
}

fn has_duplicate_source_spans(source: &DatasetSessionSource) -> bool {
    source.replay.iter().enumerate().any(|(index, entry)| {
        source
            .replay
            .iter()
            .skip(index + REQUIRED_COUNT)
            .any(|candidate| candidate.source_span == entry.source_span)
    })
}

fn count_diagnostics(diagnostics: &[DatasetDiagnostic], kind: DatasetDiagnosticKind) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind == kind)
        .count()
}
