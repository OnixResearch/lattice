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
    BLAKE3_ALGORITHM, BLAKE3_HEX_LENGTH, FIELD_MESSAGE, FIELD_ROW, FIELD_ROW_ID, FIELD_SOURCE_SPAN,
    REQUIRED_COUNT, ROLE_ASSISTANT, ROLE_SYSTEM, ROLE_TOOL, ROLE_USER,
    data::{
        DatasetChatMessage, DatasetChatRow, DatasetDiagnostic, DatasetDiagnosticKind,
        DatasetExportResult,
    },
};

const JSON_KEY_ALGORITHM: &str = "algorithm";
const JSON_KEY_CONTENT: &str = "content";
const JSON_KEY_MESSAGES: &str = "messages";
const JSON_KEY_ROLE: &str = "role";
const JSON_KEY_ROW_ID: &str = "row_id";
const JSON_KEY_SOURCE_SPANS: &str = "source_spans";
const JSON_KEY_VALUE_HEX: &str = "value_hex";
const RAW_ENVIRONMENT_MARKER: &str = "env_secret:";
const RAW_PRIVATE_MARKER: &str = "/home/";
const RAW_PROVENANCE_MARKER: &str = "private_provenance:";
const RAW_REASONING_MARKER: &str = "hidden_reasoning:";
const RAW_TOOL_MARKER: &str = "tool_output:";

/// Renders one supervised Hugging Face chat dataset row as a JSONL record.
///
/// The renderer is a host-facing projection only: it performs no filesystem I/O,
/// calls no Hugging Face libraries, and refuses publicly constructed rows whose
/// identity, role ordering, content, or source-span metadata is unsafe.
///
/// # Errors
///
/// Returns diagnostics when the row identity is not a BLAKE3 fingerprint, the
/// message/source-span shape is malformed, a role is unsupported, content is
/// empty, or a row contains raw private payload markers.
pub fn render_chat_row_jsonl(row: &DatasetChatRow) -> DatasetExportResult<String> {
    validate_renderable_chat_row(row)?;
    let mut output = String::new();
    output.push('{');
    push_json_key(&mut output, JSON_KEY_ROW_ID);
    output.push('{');
    push_json_string_field(
        &mut output,
        JSON_KEY_ALGORITHM,
        row.row_id.algorithm.as_str(),
    );
    output.push(',');
    push_json_string_field(
        &mut output,
        JSON_KEY_VALUE_HEX,
        row.row_id.value_hex.as_str(),
    );
    output.push('}');
    output.push(',');
    push_json_key(&mut output, JSON_KEY_MESSAGES);
    push_messages(&mut output, &row.messages);
    output.push(',');
    push_json_key(&mut output, JSON_KEY_SOURCE_SPANS);
    push_strings(&mut output, &row.source_spans);
    output.push('}');
    Ok(output)
}

/// Renders supervised Hugging Face chat rows as newline-delimited JSON records.
///
/// # Errors
///
/// Returns diagnostics when no rows are supplied or when any row fails
/// [`render_chat_row_jsonl`] validation.
pub fn render_chat_rows_jsonl(rows: &[DatasetChatRow]) -> DatasetExportResult<String> {
    if rows.is_empty() {
        return Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::BadChatRow,
            FIELD_ROW,
            "JSONL rendering requires at least one supervised chat row",
        )]);
    }
    let mut output = String::new();
    let mut first_row = true;
    for row in rows {
        let line = render_chat_row_jsonl(row)?;
        if first_row {
            first_row = false;
        } else {
            output.push('\n');
        }
        output.push_str(&line);
    }
    Ok(output)
}

fn validate_renderable_chat_row(row: &DatasetChatRow) -> DatasetExportResult<()> {
    if row.row_id.algorithm != BLAKE3_ALGORITHM
        || row.row_id.value_hex.len() != BLAKE3_HEX_LENGTH
        || !row
            .row_id
            .value_hex
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::BadChatRow,
            FIELD_ROW_ID,
            "chat row JSONL rendering requires a BLAKE3 row identity",
        )]);
    }
    if row.messages.len() < REQUIRED_COUNT || row.source_spans.len() != row.messages.len() {
        return Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::BadChatRow,
            FIELD_ROW,
            "chat row JSONL rendering requires messages paired with source spans",
        )]);
    }
    if row.messages.iter().any(invalid_message) {
        return Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::BadChatRow,
            FIELD_MESSAGE,
            "chat row JSONL rendering requires safe role/content messages",
        )]);
    }
    if row.source_spans.iter().any(|span| {
        span.trim().is_empty()
            || contains_private_marker(span)
            || span.chars().any(char::is_control)
    }) {
        return Err(vec![DatasetDiagnostic::new(
            DatasetDiagnosticKind::Leakage,
            FIELD_SOURCE_SPAN,
            "chat row JSONL rendering requires safe source span labels",
        )]);
    }
    Ok(())
}

fn invalid_message(message: &DatasetChatMessage) -> bool {
    !matches!(
        message.role.as_str(),
        ROLE_SYSTEM | ROLE_USER | ROLE_ASSISTANT | ROLE_TOOL
    ) || message.content.trim().is_empty()
        || contains_private_marker(message.content.as_str())
}

fn contains_private_marker(text: &str) -> bool {
    text.contains(RAW_ENVIRONMENT_MARKER)
        || text.contains(RAW_PRIVATE_MARKER)
        || text.contains(RAW_PROVENANCE_MARKER)
        || text.contains(RAW_REASONING_MARKER)
        || text.contains(RAW_TOOL_MARKER)
}

fn push_messages(output: &mut String, messages: &[DatasetChatMessage]) {
    output.push('[');
    let mut first_message = true;
    for message in messages {
        if first_message {
            first_message = false;
        } else {
            output.push(',');
        }
        output.push('{');
        push_json_string_field(output, JSON_KEY_ROLE, message.role.as_str());
        output.push(',');
        push_json_string_field(output, JSON_KEY_CONTENT, message.content.as_str());
        output.push('}');
    }
    output.push(']');
}

fn push_strings(output: &mut String, values: &[String]) {
    output.push('[');
    let mut first_value = true;
    for value in values {
        if first_value {
            first_value = false;
        } else {
            output.push(',');
        }
        output.push('"');
        push_json_escaped(output, value);
        output.push('"');
    }
    output.push(']');
}

fn push_json_string_field(output: &mut String, key: &str, value: &str) {
    push_json_key(output, key);
    output.push('"');
    push_json_escaped(output, value);
    output.push('"');
}

fn push_json_key(output: &mut String, key: &str) {
    output.push('"');
    output.push_str(key);
    output.push_str("\":");
}

fn push_json_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            safe => output.push(safe),
        }
    }
}
