use alloc::string::ToString;

use super::base::CURRENT_DIR_PREFIX;
use super::base::DIFF_PREFIX;
use super::base::EMPTY_HASH_INPUT;
use super::base::FTP_PREFIX;
use super::base::HOME_DIR_PREFIX;
use super::base::HTTP_PREFIX;
use super::base::HTTPS_PREFIX;
use super::base::IMAGE_PREFIX;
use super::base::LOCAL_RANGE_SEPARATOR;
use super::base::MALFORMED_RANGE_REASON;
use super::base::PARENT_DIR_SEGMENT;
use super::base::PATH_ESCAPE_REASON;
use super::base::PATH_SEPARATOR;
use super::base::RANGE_SPAN_SEPARATOR;
use super::base::REFERENCE_SIGIL;
use super::base::UNSUPPORTED_KIND_REASON;
use super::base::URL_CREDENTIAL_REASON;
use super::base::URL_LABEL_PREFIX;
use super::base::WINDOWS_PATH_SEPARATOR;
use super::base::hash_text;
use super::data::ContextLineRange;
use super::data::ContextReference;
use super::data::ContextReferenceError;
use super::data::ContextReferenceKind;
use super::data::ContextRenderMode;
use super::data::ContextSourceTrust;

/// Parses a raw prompt context reference into a typed DTO.
///
/// # Errors
///
/// Returns [`ContextReferenceError`] when the raw label is empty, lacks the `@`
/// sigil, has an unsupported kind, escapes the relative target boundary, or has
/// a malformed line range.
pub fn parse_context_reference(raw: &str) -> Result<ContextReference, ContextReferenceError> {
    let trimmed = raw.trim();
    let body = trimmed.strip_prefix(REFERENCE_SIGIL).ok_or(ContextReferenceError::MissingSigil)?.trim();
    if body.is_empty() {
        return Err(ContextReferenceError::EmptyReference);
    }
    parse_context_body(BodyParseInput {
        raw_label: trimmed,
        body,
    })
}

#[derive(Clone, Copy)]
struct BodyParseInput<'a> {
    raw_label: &'a str,
    body: &'a str,
}

fn parse_context_body(input: BodyParseInput<'_>) -> Result<ContextReference, ContextReferenceError> {
    assert!(!input.raw_label.is_empty(), "raw label is validated by parser");
    assert!(!input.body.is_empty(), "body is validated by parser");
    if input.body.starts_with(HTTP_PREFIX) || input.body.starts_with(HTTPS_PREFIX) {
        return parse_url_reference(TargetParseInput {
            raw_label: input.raw_label,
            target: input.body,
        });
    }
    if input.body.starts_with(FTP_PREFIX) {
        return Err(ContextReferenceError::UnsupportedKind {
            reason: UNSUPPORTED_KIND_REASON,
        });
    }
    if let Some(target) = input.body.strip_prefix(DIFF_PREFIX) {
        return parse_prefixed_reference(PrefixedParseInput {
            raw_label: input.raw_label,
            target,
            kind: ContextReferenceKind::Diff,
            render_mode: ContextRenderMode::Diff,
        });
    }
    if let Some(target) = input.body.strip_prefix(IMAGE_PREFIX) {
        return parse_prefixed_reference(PrefixedParseInput {
            raw_label: input.raw_label,
            target,
            kind: ContextReferenceKind::Image,
            render_mode: ContextRenderMode::Image,
        });
    }
    parse_local_reference(input)
}

#[derive(Clone, Copy)]
struct TargetParseInput<'a> {
    raw_label: &'a str,
    target: &'a str,
}

fn parse_url_reference(input: TargetParseInput<'_>) -> Result<ContextReference, ContextReferenceError> {
    reject_url_credentials(input.target)?;
    let hash = hash_text(input.target);
    let safe_target_label = alloc::format!("{URL_LABEL_PREFIX}{}", hash.as_str());
    Ok(ContextReference::new(super::data::forms::ContextReferenceInput {
        raw_label: input.raw_label,
        normalized_target: safe_target_label.clone(),
        safe_target_label,
        kind: ContextReferenceKind::Url,
        range: None,
        render_mode: ContextRenderMode::Text,
        source_trust: ContextSourceTrust::UserSupplied,
    }))
}

#[derive(Clone, Copy)]
struct PrefixedParseInput<'a> {
    raw_label: &'a str,
    target: &'a str,
    kind: ContextReferenceKind,
    render_mode: ContextRenderMode,
}

fn parse_prefixed_reference(input: PrefixedParseInput<'_>) -> Result<ContextReference, ContextReferenceError> {
    let normalized_target = normalize_local_target(input.target)?;
    Ok(ContextReference::new(super::data::forms::ContextReferenceInput {
        raw_label: input.raw_label,
        safe_target_label: safe_label_for_local(input.kind, normalized_target.as_str()),
        normalized_target,
        kind: input.kind,
        range: None,
        render_mode: input.render_mode,
        source_trust: ContextSourceTrust::UserSupplied,
    }))
}

fn parse_local_reference(input: BodyParseInput<'_>) -> Result<ContextReference, ContextReferenceError> {
    assert!(!input.raw_label.is_empty(), "raw label is validated by parser");
    assert!(!input.body.is_empty(), "body is validated by parser");
    let (target, range) = split_target_range(input.body)?;
    let kind = if target.ends_with(PATH_SEPARATOR) {
        ContextReferenceKind::Directory
    } else {
        ContextReferenceKind::File
    };
    let render_mode = if kind == ContextReferenceKind::Directory {
        ContextRenderMode::DirectoryListing
    } else {
        ContextRenderMode::Text
    };
    let normalized_target = normalize_local_target(target)?;
    Ok(ContextReference::new(super::data::forms::ContextReferenceInput {
        raw_label: input.raw_label,
        safe_target_label: safe_label_for_local(kind, normalized_target.as_str()),
        normalized_target,
        kind,
        range,
        render_mode,
        source_trust: ContextSourceTrust::UserSupplied,
    }))
}

fn split_target_range(body: &str) -> Result<(&str, Option<ContextLineRange>), ContextReferenceError> {
    let Some(separator_index) = body.rfind(LOCAL_RANGE_SEPARATOR) else {
        return Ok((body, None));
    };
    let after_separator =
        separator_index
            .checked_add(LOCAL_RANGE_SEPARATOR.len_utf8())
            .ok_or(ContextReferenceError::MalformedRange {
                reason: MALFORMED_RANGE_REASON,
            })?;
    let target = &body[..separator_index];
    let range_text = &body[after_separator..];
    if target.is_empty() || range_text.is_empty() {
        return Err(ContextReferenceError::MalformedRange {
            reason: MALFORMED_RANGE_REASON,
        });
    }
    Ok((target, Some(parse_line_range(range_text)?)))
}

fn parse_line_range(text: &str) -> Result<ContextLineRange, ContextReferenceError> {
    assert!(!text.is_empty(), "range text is validated before parsing");
    let Some(separator_index) = text.find(RANGE_SPAN_SEPARATOR) else {
        let start = parse_line_number(text)?;
        return ContextLineRange::new(start, Some(start));
    };
    assert!(separator_index < text.len(), "range separator index stays within range text");
    let after_separator =
        separator_index
            .checked_add(RANGE_SPAN_SEPARATOR.len_utf8())
            .ok_or(ContextReferenceError::MalformedRange {
                reason: MALFORMED_RANGE_REASON,
            })?;
    let start_text = &text[..separator_index];
    let end_text = &text[after_separator..];
    let start_line = parse_line_number(start_text)?;
    let end_line = if end_text.is_empty() {
        None
    } else {
        Some(parse_line_number(end_text)?)
    };
    ContextLineRange::new(start_line, end_line)
}

fn parse_line_number(text: &str) -> Result<u32, ContextReferenceError> {
    if text.is_empty() || !text.chars().all(|character| character.is_ascii_digit()) {
        return Err(ContextReferenceError::MalformedRange {
            reason: MALFORMED_RANGE_REASON,
        });
    }
    text.parse::<u32>().map_err(|_error| ContextReferenceError::MalformedRange {
        reason: MALFORMED_RANGE_REASON,
    })
}

fn normalize_local_target(target: &str) -> Result<alloc::string::String, ContextReferenceError> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return Err(ContextReferenceError::EmptyReference);
    }
    if is_path_escape(trimmed) {
        return Err(ContextReferenceError::PathEscape {
            reason: PATH_ESCAPE_REASON,
        });
    }
    let normalized = trimmed.strip_prefix(CURRENT_DIR_PREFIX).unwrap_or(trimmed);
    if normalized.is_empty() {
        return Err(ContextReferenceError::EmptyReference);
    }
    Ok(normalized.to_string())
}

fn is_path_escape(target: &str) -> bool {
    target.starts_with(PATH_SEPARATOR)
        || target.starts_with(WINDOWS_PATH_SEPARATOR)
        || target.starts_with(HOME_DIR_PREFIX)
        || target.split(is_path_separator).any(|segment| segment == PARENT_DIR_SEGMENT)
}

fn is_path_separator(character: char) -> bool {
    character == PATH_SEPARATOR || character == WINDOWS_PATH_SEPARATOR
}

fn reject_url_credentials(target: &str) -> Result<(), ContextReferenceError> {
    let authority = url_authority(target);
    if authority.contains('@') {
        return Err(ContextReferenceError::UnsafeTarget {
            reason: URL_CREDENTIAL_REASON,
        });
    }
    Ok(())
}

fn url_authority(target: &str) -> &str {
    let after_scheme = target.strip_prefix(HTTP_PREFIX).or_else(|| target.strip_prefix(HTTPS_PREFIX)).unwrap_or(target);
    after_scheme.split(PATH_SEPARATOR).next().unwrap_or(EMPTY_HASH_INPUT)
}

fn safe_label_for_local(kind: ContextReferenceKind, target: &str) -> alloc::string::String {
    alloc::format!("{}:{target}", kind.as_str())
}
