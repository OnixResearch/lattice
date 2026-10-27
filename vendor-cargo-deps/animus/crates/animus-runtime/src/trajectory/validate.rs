use std::collections::BTreeSet;

use animus_adapters::trajectory::{
    DiagnosticDisposition, TRAJECTORY_HELPER_BLAKE3, TRAJECTORY_SCHEMA_BLAKE3,
    TrajectoryContentDisposition, TrajectoryDiagnosticCode, TrajectoryNormalizeRequest,
    TrajectoryNormalizeResult, TrajectoryPolicy, TrajectoryRecord, TrajectorySource,
};

use super::{TrajectoryIssue, TrajectoryRole, TrajectoryRoleCounts, TrajectoryValidation};

const REQUIRED_SOURCE_COUNT: usize = 7;
const REQUIRED_DIAGNOSTIC_COUNT: usize = 15;
const MAX_SAFE_IDENTITY_BYTES: usize = 256;
const MIN_TIMESTAMP_BYTES: usize = 20;
const DATE_TIME_SEPARATOR_INDEX: usize = 10;
const YEAR_SEPARATOR_INDEX: usize = 4;
const MONTH_SEPARATOR_INDEX: usize = 7;
const HOUR_SEPARATOR_INDEX: usize = 13;
const MINUTE_SEPARATOR_INDEX: usize = 16;

type DiagnosticSummary = (Vec<(TrajectoryDiagnosticCode, u32)>, DiagnosticDisposition);

/// Revalidates the generated policy before any host effect.
///
/// # Errors
///
/// Returns a stable issue when the cohort, source set, bounds, diagnostics, or
/// mandatory privacy defaults differ from the reviewed contract.
pub fn validate_policy(policy: &TrajectoryPolicy) -> Result<String, TrajectoryIssue> {
    if policy.helper_identity != TRAJECTORY_HELPER_BLAKE3
        || policy.schema_identity != TRAJECTORY_SCHEMA_BLAKE3
    {
        return Err(TrajectoryIssue::Cohort);
    }
    validate_sources(policy)?;
    validate_bounds(policy)?;
    validate_diagnostic_policy(policy)?;
    if policy.reasoning != TrajectoryContentDisposition::Omit
        || policy.path_metadata != TrajectoryContentDisposition::Omit
    {
        return Err(TrajectoryIssue::Cohort);
    }
    Ok(policy_identity(policy))
}

/// Revalidates one loaded trajectory-v1 result without host effects.
///
/// # Errors
///
/// Returns a stable issue when request admission, record shape, order, content,
/// tool correlation, diagnostics, or bounds fail.
pub fn validate_trajectory(
    policy: &TrajectoryPolicy,
    request: &TrajectoryNormalizeRequest,
    result: &TrajectoryNormalizeResult,
) -> Result<TrajectoryValidation, TrajectoryIssue> {
    let computed_policy_identity = validate_policy(policy)?;
    validate_request(policy, request, computed_policy_identity.as_str())?;
    if result.records.is_empty()
        || result.records.len() > policy.bounds.max_records
        || result.diagnostics.len() > policy.bounds.max_diagnostics
    {
        return Err(TrajectoryIssue::ResultBound);
    }

    let mut role_counts = TrajectoryRoleCounts::default();
    let mut calls = BTreeSet::new();
    let mut results = BTreeSet::new();
    let mut tool_count = 0usize;
    for (position, record) in result.records.iter().enumerate() {
        validate_record(
            policy,
            request.source,
            position,
            record,
            &mut role_counts,
            &mut calls,
            &mut results,
            &mut tool_count,
        )?;
    }
    if role_counts.meta != 1 {
        return Err(TrajectoryIssue::Meta);
    }

    let (diagnostic_counts, disposition) = validate_diagnostics(policy, result)?;
    Ok(TrajectoryValidation {
        source: request.source,
        source_generation_identity: request.source_generation_identity.clone(),
        policy_identity: request.policy_identity.clone(),
        records: result.records.clone(),
        role_counts,
        diagnostic_counts,
        disposition,
    })
}

fn validate_sources(policy: &TrajectoryPolicy) -> Result<(), TrajectoryIssue> {
    let expected = [
        TrajectorySource::ClaudeCode,
        TrajectorySource::Codex,
        TrajectorySource::Hermes,
        TrajectorySource::LettaCode,
        TrajectorySource::OpenClaw,
        TrajectorySource::OpenHands,
        TrajectorySource::Pi,
    ];
    let unique = policy.sources.iter().copied().collect::<BTreeSet<_>>();
    if policy.sources.len() != REQUIRED_SOURCE_COUNT
        || unique.len() != REQUIRED_SOURCE_COUNT
        || expected.iter().any(|source| !unique.contains(source))
    {
        return Err(TrajectoryIssue::SourcePolicy);
    }
    Ok(())
}

fn validate_bounds(policy: &TrajectoryPolicy) -> Result<(), TrajectoryIssue> {
    let bounds = policy.bounds;
    let values = [
        bounds.max_transcript_bytes,
        bounds.max_output_bytes,
        bounds.max_records,
        bounds.max_content_bytes,
        bounds.max_tool_calls,
        bounds.max_tool_argument_bytes,
        bounds.max_tool_result_bytes,
        bounds.max_diagnostics,
    ];
    if values.contains(&0)
        || bounds.max_content_bytes > bounds.max_output_bytes
        || bounds.max_tool_argument_bytes > bounds.max_output_bytes
        || bounds.max_tool_result_bytes > bounds.max_output_bytes
    {
        return Err(TrajectoryIssue::BoundPolicy);
    }
    Ok(())
}

fn validate_diagnostic_policy(policy: &TrajectoryPolicy) -> Result<(), TrajectoryIssue> {
    let unique = policy
        .diagnostics
        .iter()
        .map(|row| row.code)
        .collect::<BTreeSet<_>>();
    if policy.diagnostics.len() != REQUIRED_DIAGNOSTIC_COUNT
        || unique.len() != REQUIRED_DIAGNOSTIC_COUNT
    {
        return Err(TrajectoryIssue::DiagnosticPolicy);
    }
    Ok(())
}

fn validate_request(
    policy: &TrajectoryPolicy,
    request: &TrajectoryNormalizeRequest,
    computed_policy_identity: &str,
) -> Result<(), TrajectoryIssue> {
    if !policy.sources.contains(&request.source)
        || request.helper_identity != policy.helper_identity
        || request.policy_identity != computed_policy_identity
        || request.transcript.len() > policy.bounds.max_transcript_bytes
        || !safe_identity(request.source_generation_identity.as_str())
    {
        return Err(TrajectoryIssue::Request);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_record(
    policy: &TrajectoryPolicy,
    source: TrajectorySource,
    position: usize,
    record: &TrajectoryRecord,
    counts: &mut TrajectoryRoleCounts,
    calls: &mut BTreeSet<String>,
    results: &mut BTreeSet<String>,
    tool_count: &mut usize,
) -> Result<(), TrajectoryIssue> {
    if position == 0 {
        match record {
            TrajectoryRecord::Meta { source: actual, .. } if actual == source.as_str() => {
                counts.add(TrajectoryRole::Meta)?;
                return Ok(());
            }
            _ => return Err(TrajectoryIssue::Meta),
        }
    }
    if matches!(record, TrajectoryRecord::Meta { .. }) {
        return Err(TrajectoryIssue::Meta);
    }

    match record {
        TrajectoryRecord::Meta { .. } => Err(TrajectoryIssue::Meta),
        TrajectoryRecord::User { content, timestamp } => {
            validate_content(policy, content, timestamp, false)?;
            counts.add(TrajectoryRole::User)
        }
        TrajectoryRecord::Reasoning { content, timestamp } => {
            validate_content(policy, content, timestamp, false)?;
            counts.add(TrajectoryRole::Reasoning)
        }
        TrajectoryRecord::Assistant { content, timestamp } => {
            validate_content(policy, content, timestamp, false)?;
            counts.add(TrajectoryRole::Assistant)
        }
        TrajectoryRecord::AssistantToolCalls {
            tool_calls,
            timestamp,
        } => {
            if tool_calls.is_empty() || !valid_timestamp(timestamp) {
                return Err(TrajectoryIssue::Content);
            }
            *tool_count = tool_count
                .checked_add(tool_calls.len())
                .ok_or(TrajectoryIssue::ResultBound)?;
            if *tool_count > policy.bounds.max_tool_calls {
                return Err(TrajectoryIssue::ResultBound);
            }
            for call in tool_calls {
                if call.id.is_empty() || call.name.is_empty() || !calls.insert(call.id.clone()) {
                    return Err(TrajectoryIssue::DuplicateToolCall);
                }
                if call.arguments.len() > policy.bounds.max_tool_argument_bytes {
                    return Err(TrajectoryIssue::ResultBound);
                }
                let value: serde_json::Value = serde_json::from_str(call.arguments.as_str())
                    .map_err(|_error| TrajectoryIssue::ToolArguments)?;
                if !value.is_object() {
                    return Err(TrajectoryIssue::ToolArguments);
                }
            }
            counts.add(TrajectoryRole::AssistantToolCalls)
        }
        TrajectoryRecord::Tool {
            tool_call_id,
            content,
            timestamp,
        } => {
            validate_content(policy, content, timestamp, true)?;
            if !calls.contains(tool_call_id) || !results.insert(tool_call_id.clone()) {
                return Err(TrajectoryIssue::ToolResult);
            }
            counts.add(TrajectoryRole::Tool)
        }
    }
}

fn validate_content(
    policy: &TrajectoryPolicy,
    content: &str,
    timestamp: &str,
    tool_result: bool,
) -> Result<(), TrajectoryIssue> {
    if content.is_empty() || !valid_timestamp(timestamp) {
        return Err(TrajectoryIssue::Content);
    }
    let limit = if tool_result {
        policy.bounds.max_tool_result_bytes
    } else {
        policy.bounds.max_content_bytes
    };
    if content.len() > limit {
        return Err(TrajectoryIssue::ResultBound);
    }
    Ok(())
}

fn valid_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < MIN_TIMESTAMP_BYTES
        || bytes.get(YEAR_SEPARATOR_INDEX) != Some(&b'-')
        || bytes.get(MONTH_SEPARATOR_INDEX) != Some(&b'-')
        || bytes.get(DATE_TIME_SEPARATOR_INDEX) != Some(&b'T')
        || bytes.get(HOUR_SEPARATOR_INDEX) != Some(&b':')
        || bytes.get(MINUTE_SEPARATOR_INDEX) != Some(&b':')
    {
        return false;
    }
    value.ends_with('Z')
        || value
            .get(MIN_TIMESTAMP_BYTES - 1..)
            .is_some_and(|tail| tail.contains('+') || tail.contains('-'))
}

fn validate_diagnostics(
    policy: &TrajectoryPolicy,
    result: &TrajectoryNormalizeResult,
) -> Result<DiagnosticSummary, TrajectoryIssue> {
    let mut totals: Vec<(TrajectoryDiagnosticCode, u32)> = Vec::new();
    let mut disposition = DiagnosticDisposition::AcceptedCleanup;
    for diagnostic in &result.diagnostics {
        let row = policy
            .diagnostics
            .iter()
            .find(|row| row.code == diagnostic.code)
            .ok_or(TrajectoryIssue::Diagnostic)?;
        if row.disposition == DiagnosticDisposition::Rejected {
            return Err(TrajectoryIssue::Diagnostic);
        }
        if row.disposition == DiagnosticDisposition::Lossy {
            disposition = DiagnosticDisposition::Lossy;
        }
        let count = diagnostic.count.unwrap_or(1);
        if count == 0 {
            return Err(TrajectoryIssue::Diagnostic);
        }
        match totals
            .iter_mut()
            .find(|(code, _count)| *code == diagnostic.code)
        {
            Some((_code, total)) => {
                *total = total
                    .checked_add(count)
                    .ok_or(TrajectoryIssue::ResultBound)?;
            }
            None => totals.push((diagnostic.code, count)),
        }
    }
    totals.sort_by_key(|(code, _count)| *code);
    Ok((totals, disposition))
}

fn safe_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SAFE_IDENTITY_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_' | b'.'))
}

fn policy_identity(policy: &TrajectoryPolicy) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"animus.trajectory-policy.v1\0");
    hasher.update(policy.helper_identity.as_bytes());
    hasher.update(policy.schema_identity.as_bytes());
    for source in &policy.sources {
        hasher.update(source.as_str().as_bytes());
        hasher.update(&[0]);
    }
    for value in [
        policy.bounds.max_transcript_bytes,
        policy.bounds.max_output_bytes,
        policy.bounds.max_records,
        policy.bounds.max_content_bytes,
        policy.bounds.max_tool_calls,
        policy.bounds.max_tool_argument_bytes,
        policy.bounds.max_tool_result_bytes,
        policy.bounds.max_diagnostics,
    ] {
        hasher.update(&value.to_le_bytes());
    }
    for row in &policy.diagnostics {
        hasher.update(row.code.as_str().as_bytes());
        hasher.update(&[row.disposition as u8]);
    }
    hasher.update(&[
        policy.reasoning as u8,
        policy.tool_arguments as u8,
        policy.tool_results as u8,
        policy.path_metadata as u8,
    ]);
    hasher.finalize().to_hex().to_string()
}
