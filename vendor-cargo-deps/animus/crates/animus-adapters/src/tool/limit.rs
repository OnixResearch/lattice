/// Tool-output hash requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashRequirement {
    /// Hash only when truncation makes identity evidence necessary.
    Optional,
    /// Always include a BLAKE3 content hash.
    Required,
}

/// Tool-output truncation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputPolicy {
    /// Maximum model-visible character count.
    pub visible_char_budget: u32,
}

/// Borrowed tool output before bounding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputDraft<'a> {
    /// Raw executor output body.
    pub body: &'a str,
    /// Optional host-owned reference for full output.
    pub full_output_reference: Option<&'a str>,
    /// Whether callers require a hash even when output is complete.
    pub hash_requirement: HashRequirement,
}

/// Bounded output status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputStatus {
    /// The visible text contains the complete output.
    Complete,
    /// The visible text is a bounded prefix of a larger output.
    Truncated,
}

/// Preserved bounded-output metadata without the raw full body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedOutputMetadataDraft<'a> {
    /// Bounded model-visible text.
    pub visible_text: &'a str,
    /// Completion/truncation status.
    pub status: OutputStatus,
    /// Original output byte count.
    pub byte_count: u64,
    /// Original output character count.
    pub char_count: u64,
    /// Original output line count.
    pub line_count: u64,
    /// Optional BLAKE3 content hash.
    pub content_hash: Option<&'a str>,
    /// Optional host-owned full-output reference.
    pub full_output_reference: Option<&'a str>,
}

/// Bounded tool output safe for model-visible context and receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedOutput {
    visible_text: super::Text,
    status: OutputStatus,
    byte_count: u64,
    char_count: u64,
    line_count: u64,
    content_hash: Option<super::Text>,
    full_output_reference: Option<super::Text>,
}

impl BoundedOutput {
    /// Bounds raw tool output according to model-visible policy.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractError`] when the budget is invalid, output
    /// contains a NUL character, or a full-output reference is unsafe.
    pub fn new(
        draft: &OutputDraft<'_>,
        policy: OutputPolicy,
    ) -> Result<Self, super::ContractError> {
        let visible_char_budget = validate_budget(policy.visible_char_budget)?;
        super::validate_no_nul(super::OUTPUT_FIELD, draft.body)?;
        let char_count = draft.body.chars().count();
        let extent = Extent {
            char_count,
            visible_char_budget,
        };
        let status = status_for_extent(extent);
        let visible_text = visible_text(draft.body, visible_char_budget, status);
        let content_hash = content_hash_for(draft, status);
        let full_output_reference = optional_safe_reference(draft.full_output_reference)?;
        Ok(Self {
            visible_text,
            status,
            byte_count: super::usize_to_u64(draft.body.len()),
            char_count: super::usize_to_u64(char_count),
            line_count: super::usize_to_u64(count_lines(draft.body)),
            content_hash,
            full_output_reference,
        })
    }

    /// Reconstructs preserved bounded metadata without requiring the raw full body.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractError`] when visible text contains a NUL,
    /// optional hash/reference metadata is malformed, or counts contradict the
    /// visible text.
    pub fn from_metadata(
        draft: &BoundedOutputMetadataDraft<'_>,
    ) -> Result<Self, super::ContractError> {
        super::validate_no_nul(super::OUTPUT_FIELD, draft.visible_text)?;
        validate_preserved_counts(draft)?;
        let content_hash = optional_content_hash(draft.content_hash)?;
        let full_output_reference = optional_safe_reference(draft.full_output_reference)?;
        Ok(Self {
            visible_text: super::Text::from(draft.visible_text),
            status: draft.status,
            byte_count: draft.byte_count,
            char_count: draft.char_count,
            line_count: draft.line_count,
            content_hash,
            full_output_reference,
        })
    }

    /// Returns the bounded model-visible text.
    #[must_use]
    pub fn visible_text(&self) -> &str {
        self.visible_text.as_str()
    }

    /// Returns whether output was complete or truncated.
    #[must_use]
    pub const fn status(&self) -> OutputStatus {
        self.status
    }

    /// Returns the original byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    /// Returns the original character count.
    #[must_use]
    pub const fn char_count(&self) -> u64 {
        self.char_count
    }

    /// Returns the original line count.
    #[must_use]
    pub const fn line_count(&self) -> u64 {
        self.line_count
    }

    /// Returns the BLAKE3 content hash when required by policy/status.
    #[must_use]
    pub fn content_hash(&self) -> Option<&str> {
        self.content_hash.as_deref()
    }

    /// Returns the host-owned full-output reference when present.
    #[must_use]
    pub fn full_output_reference(&self) -> Option<&str> {
        self.full_output_reference.as_deref()
    }
}

#[derive(Clone, Copy)]
struct Extent {
    char_count: usize,
    visible_char_budget: usize,
}

fn validate_budget(visible_char_budget: u32) -> Result<usize, super::ContractError> {
    if (super::MIN_VISIBLE_OUTPUT_CHARS..=super::MAX_VISIBLE_OUTPUT_CHARS)
        .contains(&visible_char_budget)
    {
        return match usize::try_from(visible_char_budget) {
            Ok(converted) => Ok(converted),
            Err(_overflow) => Err(invalid_budget(visible_char_budget)),
        };
    }
    Err(invalid_budget(visible_char_budget))
}

const fn invalid_budget(actual_chars: u32) -> super::ContractError {
    super::ContractError::InvalidVisibleBudget {
        min_chars: super::MIN_VISIBLE_OUTPUT_CHARS,
        max_chars: super::MAX_VISIBLE_OUTPUT_CHARS,
        actual_chars,
    }
}

const fn status_for_extent(extent: Extent) -> OutputStatus {
    if extent.char_count > extent.visible_char_budget {
        OutputStatus::Truncated
    } else {
        OutputStatus::Complete
    }
}

fn visible_text(body: &str, visible_char_budget: usize, status: OutputStatus) -> super::Text {
    match status {
        OutputStatus::Complete => super::Text::from(body),
        OutputStatus::Truncated => body.chars().take(visible_char_budget).collect(),
    }
}

fn content_hash_for(draft: &OutputDraft<'_>, status: OutputStatus) -> Option<super::Text> {
    if status == OutputStatus::Truncated || draft.hash_requirement == HashRequirement::Required {
        return Some(super::hash_parts(&[super::HASH_SCHEMA, draft.body]));
    }
    None
}

fn validate_preserved_counts(
    draft: &BoundedOutputMetadataDraft<'_>,
) -> Result<(), super::ContractError> {
    let visible_bytes = super::usize_to_u64(draft.visible_text.len());
    let visible_chars = super::usize_to_u64(draft.visible_text.chars().count());
    if draft.byte_count < visible_bytes || draft.char_count < visible_chars {
        return Err(super::ContractError::InconsistentOutputMetadata);
    }
    Ok(())
}

fn optional_content_hash(hash: Option<&str>) -> Result<Option<super::Text>, super::ContractError> {
    hash.map(|value| super::required_text_value(super::OUTPUT_HASH_FIELD, value))
        .transpose()
}

fn optional_safe_reference(
    reference: Option<&str>,
) -> Result<Option<super::Text>, super::ContractError> {
    match reference {
        Some(raw) => {
            let normalized = super::required_text_value(super::OUTPUT_REFERENCE_FIELD, raw)?;
            if is_safe_reference(&normalized) {
                Ok(Some(normalized))
            } else {
                Err(super::ContractError::UnsafeReference {
                    reference: normalized,
                })
            }
        }
        None => Ok(None),
    }
}

fn is_safe_reference(reference: &str) -> bool {
    !reference.starts_with(super::ABSOLUTE_PATH_PREFIX)
        && !reference.starts_with(super::HOME_PATH_PREFIX)
        && !reference.contains(super::URI_SCHEME_MARKER)
        && !reference.contains(super::WINDOWS_SEPARATOR_MARKER)
}

fn count_lines(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    text.lines().count()
}
