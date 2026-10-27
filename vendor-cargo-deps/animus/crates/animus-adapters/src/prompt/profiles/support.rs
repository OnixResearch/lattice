use alloc::string::ToString;

pub(super) type Text = alloc::string::String;
pub(super) type Collection<T> = alloc::vec::Vec<T>;

pub(super) const PROFILE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("prompt_profile_id");
pub(super) const PROFILE_PURPOSE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("prompt_profile_purpose");
pub(super) const PROFILE_SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("prompt_profile_source_identity");
pub(super) const PROFILE_BODY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("prompt_profile_body");
pub(super) const PROFILE_EXTRA_INSTRUCTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("prompt_profile_extra_instruction");

pub(super) const MIN_PRECEDENCE: u16 = 1;
pub(super) const MAX_PRECEDENCE: u16 = 1_000;
pub(super) const MAX_BODY_CHARS: usize = 4_096;
pub(super) const MAX_EXTRA_INSTRUCTION_CHARS: usize = 512;
pub(super) const MAX_ASSEMBLY_PROFILES: usize = 64;
pub(super) const MAX_DIRECTIVE_TOKENS: usize = 8;
pub(super) const MAX_CHAIN_TARGETS: usize = 16;
pub(super) const HASH_HEX_CHARS: usize = 64;

const BLAKE3_HEX_MULTIPLIER: usize = 2;
const HASH_SCHEMA: &str = "animus.prompt-profile.v1";
const HASH_SEPARATOR: &str = "|";

const _: () = assert!(MIN_PRECEDENCE > 0);
const _: () = assert!(MAX_PRECEDENCE >= MIN_PRECEDENCE);
const _: () = assert!(MAX_BODY_CHARS > 0);
const _: () = assert!(MAX_EXTRA_INSTRUCTION_CHARS > 0);
const _: () = assert!(MAX_ASSEMBLY_PROFILES > 0);
const _: () = assert!(MAX_DIRECTIVE_TOKENS > 0);
const _: () = assert!(MAX_CHAIN_TARGETS > 0);
const _: () = assert!(HASH_HEX_CHARS == blake3::OUT_LEN.saturating_mul(BLAKE3_HEX_MULTIPLIER));

pub(super) fn required_text(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<Text, super::types::Error> {
    crate::validation::normalized_adapter_field(field, value).map_err(super::types::Error::from)
}

pub(super) fn validate_precedence(precedence: u16) -> Result<(), super::types::Error> {
    if (MIN_PRECEDENCE..=MAX_PRECEDENCE).contains(&precedence) {
        Ok(())
    } else {
        Err(super::types::Error::PrecedenceOutOfRange {
            min: MIN_PRECEDENCE,
            max: MAX_PRECEDENCE,
            actual: precedence,
        })
    }
}

pub(super) fn validate_capabilities(
    capabilities: &[super::types::Capability],
) -> Result<(), super::types::Error> {
    if capabilities.is_empty() {
        Err(super::types::Error::MissingCapabilities)
    } else {
        Ok(())
    }
}

pub(super) fn validate_body(body: &str) -> Result<(), super::types::Error> {
    crate::validation::validate_required_adapter_field(PROFILE_BODY_FIELD, body)?;
    let actual_chars = body.chars().count();
    if actual_chars > MAX_BODY_CHARS {
        Err(super::types::Error::BodyTooLong {
            max_chars: MAX_BODY_CHARS,
            actual_chars,
        })
    } else {
        Ok(())
    }
}

pub(super) fn normalize_extra_instruction(
    value: Option<&str>,
) -> Result<Option<Text>, super::types::Error> {
    match value {
        Some(raw) => normalize_present_extra_instruction(raw),
        None => Ok(None),
    }
}

fn normalize_present_extra_instruction(raw: &str) -> Result<Option<Text>, super::types::Error> {
    crate::validation::validate_optional_adapter_field(PROFILE_EXTRA_INSTRUCTION_FIELD, raw)?;
    let actual_chars = raw.chars().count();
    if actual_chars > MAX_EXTRA_INSTRUCTION_CHARS {
        return Err(super::types::Error::ExtraInstructionTooLong {
            max_chars: MAX_EXTRA_INSTRUCTION_CHARS,
            actual_chars,
        });
    }

    let normalized = raw.trim();
    if normalized.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Text::from(normalized)))
    }
}

pub(super) fn parse_mode(label: &str) -> Option<super::types::Mode> {
    match label.trim() {
        "ask" => Some(super::types::Mode::Ask),
        "brainstorm" => Some(super::types::Mode::Brainstorm),
        "plan" => Some(super::types::Mode::Plan),
        "code" => Some(super::types::Mode::Code),
        "debug" => Some(super::types::Mode::Debug),
        "review" => Some(super::types::Mode::Review),
        "simplify" => Some(super::types::Mode::Simplify),
        "write-prompt" => Some(super::types::Mode::WritePrompt),
        _ => None,
    }
}

pub(super) fn hash_parts(parts: &[&str]) -> Text {
    let mut joined = Text::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            joined.push_str(HASH_SEPARATOR);
        }
        joined.push_str(part);
    }
    let rendered = blake3::hash(joined.as_bytes()).to_hex().to_string();
    assert!(
        rendered.len() == HASH_HEX_CHARS,
        "BLAKE3 hex length drifted"
    );
    rendered
}

#[derive(Clone, Copy)]
pub(super) struct BodyHashInput<'a> {
    pub(super) profile_id: &'a str,
    pub(super) source_identity: &'a str,
    pub(super) body: &'a str,
}

pub(super) fn profile_body_hash(input: BodyHashInput<'_>) -> Text {
    hash_parts(&[
        HASH_SCHEMA,
        input.profile_id,
        input.source_identity,
        input.body,
    ])
}

pub(super) fn count_to_u64(value: usize) -> u64 {
    value as u64
}
