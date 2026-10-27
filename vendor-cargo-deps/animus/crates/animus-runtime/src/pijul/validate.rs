use std::collections::BTreeSet;

use animus_adapters::pijul::{
    PijulBackendResponse, PijulConfirmation, PijulExecutionStatus, PijulLoadedChannel,
    PijulLoadedDiffEntry, PijulLoadedLogEntry, PijulLoadedOutput, PijulOperation, PijulRequest,
    PijulSignatureStatus,
};

use super::{
    ALLOWED_DIFF_OPERATIONS, CHANGE_HASH_CHARS, PijulBounds, PijulProfile, PijulValidationError,
    PijulValidationIssue, STATE_ID_CHARS, ValidatedPijulChange, ValidatedPijulChannel,
    ValidatedPijulDiff, ValidatedPijulOutput, blake3_hex, validate_blake3_hex,
};

const MAX_CURRENT_CHANNELS: usize = 1;
const ZERO_EXIT_CODE: i32 = 0;
const WINDOWS_DRIVE_PREFIX_CHARS: usize = 3;
const WINDOWS_DRIVE_SEPARATOR_INDEX: usize = 1;
const WINDOWS_ROOT_SEPARATOR_INDEX: usize = 2;
const PROMPT_MARKERS: &[&str] = &[
    "password:",
    "passphrase:",
    "are you sure",
    "continue?",
    "confirm ",
    "press enter",
    "[y/n]",
    "(y/n)",
];

/// Validates a request before any host process or filesystem effect.
///
/// # Errors
///
/// Returns an error for an invalid identity, non-absolute explicit path, denied
/// channel, invalid selector, unsupported operation, or protected mutation.
pub fn validate_pijul_request(
    profile: &PijulProfile,
    request: &PijulRequest,
) -> Result<(), PijulValidationError> {
    validate_blake3_hex(&request.repository_identity, "repository_identity")?;
    if !is_explicit_absolute_path(&request.repository_path) {
        return Err(PijulValidationError::new(
            PijulValidationIssue::InvalidRequest,
            "repository_path",
        ));
    }
    if let Some(channel) = &request.channel {
        validate_channel_name(channel, profile.bounds())?;
        if !profile.channel_allowlist().contains(channel) {
            return Err(PijulValidationError::new(
                PijulValidationIssue::InvalidChannel,
                "channel_allowlist",
            ));
        }
    }
    if let Some(selector) = &request.change_selector {
        validate_hash_selector(selector, profile.bounds())?;
    }
    if request.operation.is_protected_mutation() {
        return Err(PijulValidationError::new(
            PijulValidationIssue::ProtectedMutationDenied,
            "protected_mutation",
        ));
    }
    if !profile.query_allowlist().contains(&request.operation)
        || profile.unsupported().contains(&request.operation)
    {
        return Err(PijulValidationError::new(
            PijulValidationIssue::UnsupportedOperation,
            "operation",
        ));
    }
    if request.confirmation != PijulConfirmation::NotRequired {
        return Err(PijulValidationError::new(
            PijulValidationIssue::InvalidRequest,
            "confirmation",
        ));
    }
    Ok(())
}

/// Revalidates one parsed executor response against the exact cohort and bounds.
///
/// # Errors
///
/// Returns an error for exit-status lies, truncation, over-limit bytes, protocol
/// mismatch, malformed hashes, unknown status classes, unsafe paths, prompt-like
/// diagnostics, duplicate records, ambiguous prefixes, or denied channels.
pub fn validate_pijul_response(
    profile: &PijulProfile,
    request: &PijulRequest,
    response: &PijulBackendResponse,
) -> Result<ValidatedPijulOutput, PijulValidationError> {
    validate_pijul_request(profile, request)?;
    validate_execution(profile.bounds(), response)?;
    if response.operation != request.operation {
        return Err(PijulValidationError::new(
            PijulValidationIssue::ProtocolMismatch,
            "response_operation",
        ));
    }
    let loaded = response.loaded.as_ref().ok_or_else(|| {
        PijulValidationError::new(PijulValidationIssue::ProtocolMismatch, "loaded_output")
    })?;
    match (request.operation, loaded) {
        (PijulOperation::Log, PijulLoadedOutput::Log(entries)) => {
            validate_log(profile, request, entries)
        }
        (PijulOperation::Diff, PijulLoadedOutput::Diff(entries)) => validate_diff(profile, entries),
        (PijulOperation::ChannelList, PijulLoadedOutput::Channels(entries)) => {
            validate_channels(profile, entries)
        }
        _ => Err(PijulValidationError::new(
            PijulValidationIssue::ProtocolMismatch,
            "loaded_variant",
        )),
    }
}

fn validate_execution(
    bounds: PijulBounds,
    response: &PijulBackendResponse,
) -> Result<(), PijulValidationError> {
    let output_bytes = u64::try_from(response.execution.standard_output.len()).unwrap_or(u64::MAX);
    let error_bytes = u64::try_from(response.execution.standard_error.len()).unwrap_or(u64::MAX);
    if response.execution.stdout_truncated
        || response.execution.stderr_truncated
        || output_bytes > bounds.max_stdout_bytes
        || error_bytes > bounds.max_stderr_bytes
    {
        return Err(PijulValidationError::new(
            PijulValidationIssue::BoundExceeded,
            "execution_output",
        ));
    }
    if contains_prompt_like_output(&response.execution.standard_error) {
        return Err(PijulValidationError::new(
            PijulValidationIssue::InteractiveOutput,
            "standard_error",
        ));
    }
    if response.execution.status == PijulExecutionStatus::Exited
        && response.execution.exit_code == Some(ZERO_EXIT_CODE)
    {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::InvalidExitStatus,
            "exit_status",
        ))
    }
}

fn validate_log(
    profile: &PijulProfile,
    request: &PijulRequest,
    entries: &[PijulLoadedLogEntry],
) -> Result<ValidatedPijulOutput, PijulValidationError> {
    validate_collection_bound(
        entries.len(),
        profile.bounds().max_log_entries,
        "log_entries",
    )?;
    let mut hashes = BTreeSet::new();
    let mut result = Vec::with_capacity(entries.len());
    for entry in entries {
        validate_exact_identity(&entry.hash, CHANGE_HASH_CHARS, "change_hash")?;
        if !hashes.insert(entry.hash.as_str()) {
            return Err(PijulValidationError::new(
                PijulValidationIssue::InvalidStatus,
                "duplicate_change_hash",
            ));
        }
        if let Some(state) = &entry.state {
            validate_exact_identity(state, STATE_ID_CHARS, "state_identity")?;
        }
        if entry.state.is_some()
            || !entry.authors.is_empty()
            || entry.timestamp.is_some()
            || entry.message.is_some()
            || entry.description.is_some()
            || !entry.paths.is_empty()
        {
            return Err(PijulValidationError::new(
                PijulValidationIssue::UnsafeEvidence,
                "log_content",
            ));
        }
        result.push(ValidatedPijulChange {
            hash: entry.hash.clone(),
            state: entry.state.clone(),
            author_count: u32::try_from(entry.authors.len()).unwrap_or(u32::MAX),
            timestamp_present: entry.timestamp.is_some(),
            signature_status: PijulSignatureStatus::NotReported,
        });
    }
    validate_selector_matches(request.change_selector.as_deref(), &result)?;
    Ok(ValidatedPijulOutput::Log(result))
}

fn validate_diff(
    profile: &PijulProfile,
    entries: &[PijulLoadedDiffEntry],
) -> Result<ValidatedPijulOutput, PijulValidationError> {
    validate_collection_bound(
        entries.len(),
        profile.bounds().max_diff_records,
        "diff_records",
    )?;
    let mut normalized = String::from("pijul-diff-json-v1-beta18");
    let mut conflict_count = 0_u32;
    for entry in entries {
        validate_relative_path(&entry.path, profile.bounds().max_path_bytes)?;
        if !ALLOWED_DIFF_OPERATIONS.contains(&entry.operation.as_str()) {
            return Err(PijulValidationError::new(
                PijulValidationIssue::InvalidStatus,
                "diff_operation",
            ));
        }
        if entry.operation.contains("conflict") {
            conflict_count = conflict_count.saturating_add(1);
        }
        normalized.push('\0');
        normalized.push_str(entry.operation.as_str());
        normalized.push('\0');
        match entry.line {
            Some(line) => normalized.push_str(line.to_string().as_str()),
            None => normalized.push_str("none"),
        }
    }
    Ok(ValidatedPijulOutput::Diff(ValidatedPijulDiff {
        record_count: u32::try_from(entries.len()).unwrap_or(u32::MAX),
        conflict_count,
        observation_identity: blake3_hex(normalized.as_bytes()),
    }))
}

fn validate_channels(
    profile: &PijulProfile,
    entries: &[PijulLoadedChannel],
) -> Result<ValidatedPijulOutput, PijulValidationError> {
    validate_collection_bound(entries.len(), profile.bounds().max_log_entries, "channels")?;
    let mut unique = BTreeSet::new();
    let mut current_count = 0_usize;
    let mut result = Vec::with_capacity(entries.len());
    for entry in entries {
        validate_channel_name(&entry.name, profile.bounds())?;
        if !profile.channel_allowlist().contains(&entry.name) || !unique.insert(entry.name.as_str())
        {
            return Err(PijulValidationError::new(
                PijulValidationIssue::InvalidChannel,
                "channel_entry",
            ));
        }
        if entry.is_current {
            current_count = current_count.saturating_add(1);
        }
        result.push(ValidatedPijulChannel {
            name: entry.name.clone(),
            is_current: entry.is_current,
        });
    }
    if current_count > MAX_CURRENT_CHANNELS {
        return Err(PijulValidationError::new(
            PijulValidationIssue::InvalidStatus,
            "current_channel_count",
        ));
    }
    Ok(ValidatedPijulOutput::Channels(result))
}

fn validate_selector_matches(
    selector: Option<&str>,
    changes: &[ValidatedPijulChange],
) -> Result<(), PijulValidationError> {
    let Some(selector) = selector else {
        return Ok(());
    };
    let matches = changes
        .iter()
        .filter(|change| change.hash.starts_with(selector))
        .count();
    if matches == 1 {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::AmbiguousPrefix,
            "change_selector",
        ))
    }
}

fn validate_collection_bound(
    actual: usize,
    maximum: u64,
    field: &'static str,
) -> Result<(), PijulValidationError> {
    if u64::try_from(actual).unwrap_or(u64::MAX) <= maximum {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::BoundExceeded,
            field,
        ))
    }
}

fn validate_exact_identity(
    value: &str,
    expected_chars: usize,
    field: &'static str,
) -> Result<(), PijulValidationError> {
    if value.len() == expected_chars && value.bytes().all(is_base32_identity_byte) {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::InvalidIdentity,
            field,
        ))
    }
}

fn validate_hash_selector(value: &str, bounds: PijulBounds) -> Result<(), PijulValidationError> {
    let chars = u64::try_from(value.len()).unwrap_or(u64::MAX);
    let maximum = u64::try_from(CHANGE_HASH_CHARS).unwrap_or(u64::MAX);
    if chars < bounds.minimum_hash_prefix_chars || chars > maximum {
        return Err(PijulValidationError::new(
            PijulValidationIssue::AmbiguousPrefix,
            "change_selector",
        ));
    }
    if value.bytes().all(is_base32_identity_byte) {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::InvalidIdentity,
            "change_selector",
        ))
    }
}

fn validate_channel_name(value: &str, bounds: PijulBounds) -> Result<(), PijulValidationError> {
    let bytes = u64::try_from(value.len()).unwrap_or(u64::MAX);
    if !value.is_empty()
        && bytes <= bounds.max_channel_bytes
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::InvalidChannel,
            "channel",
        ))
    }
}

fn validate_relative_path(value: &str, maximum: u64) -> Result<(), PijulValidationError> {
    let bytes = u64::try_from(value.len()).unwrap_or(u64::MAX);
    let unsafe_component = value
        .split('/')
        .any(|component| component.is_empty() || matches!(component, "." | ".."));
    if !value.is_empty()
        && bytes <= maximum
        && !value.starts_with('/')
        && !value.contains('\\')
        && !value.contains('\0')
        && !unsafe_component
    {
        Ok(())
    } else {
        Err(PijulValidationError::new(
            PijulValidationIssue::UnsafeEvidence,
            "relative_path",
        ))
    }
}

fn contains_prompt_like_output(bytes: &[u8]) -> bool {
    let text = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    PROMPT_MARKERS.iter().any(|marker| text.contains(marker))
}

fn is_base32_identity_byte(byte: u8) -> bool {
    byte.is_ascii_uppercase() || (b'2'..=b'7').contains(&byte)
}

fn is_explicit_absolute_path(value: &str) -> bool {
    value.starts_with('/') || is_windows_absolute_path(value)
}

fn is_windows_absolute_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= WINDOWS_DRIVE_PREFIX_CHARS
        && bytes[WINDOWS_DRIVE_SEPARATOR_INDEX] == b':'
        && matches!(bytes[WINDOWS_ROOT_SEPARATOR_INDEX], b'\\' | b'/')
        && bytes[0].is_ascii_alphabetic()
}
