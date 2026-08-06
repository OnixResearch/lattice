#[derive(Clone, Copy)]
struct CountLimit {
    field: crate::validation::FieldName,
    actual_items: usize,
    max_items: usize,
}

/// Borrowed descriptor for the reviewed tool call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolCallDescriptorDraft<'a> {
    /// Safe tool name or tool family label.
    pub tool_name: Option<&'a str>,
    /// Safe target/resource label, not a raw full path or secret-bearing URI.
    pub target_label: Option<&'a str>,
    /// Safe redacted argument labels available to deterministic scanners.
    pub safe_argument_labels: &'a [&'a str],
}

/// Validated data-only descriptor for a reviewed tool call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCallDescriptor {
    tool_name: super::Text,
    target_label: super::Text,
    safe_argument_labels: super::Collection<super::Text>,
}

impl ToolCallDescriptor {
    /// Validates a borrowed tool-call descriptor without executing the tool.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when labels are missing, malformed, too
    /// numerous, or contain raw sensitive material.
    pub fn validate(draft: &ToolCallDescriptorDraft<'_>) -> Result<Self, super::Error> {
        let tool_name = required_safe_text(super::TOOL_NAME_FIELD, draft.tool_name)?;
        let target_label = required_safe_text(super::TOOL_TARGET_FIELD, draft.target_label)?;
        let safe_argument_labels = required_safe_collection(
            super::ARGUMENT_LABEL_FIELD,
            draft.safe_argument_labels,
            super::MAX_ARGUMENT_LABEL_ITEMS,
        )?;
        Ok(Self {
            tool_name,
            target_label,
            safe_argument_labels,
        })
    }

    /// Returns the safe tool label.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the safe target label.
    #[must_use]
    pub fn target_label(&self) -> &str {
        self.target_label.as_str()
    }

    /// Returns safe redacted argument labels.
    #[must_use]
    pub fn safe_argument_labels(&self) -> &[super::Text] {
        self.safe_argument_labels.as_slice()
    }
}

/// Borrowed security review input before redaction and hashing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Original task labels or classes, never raw prompt text.
    pub task_labels: &'a [&'a str],
    /// Bounded recent context summaries after host redaction.
    pub recent_context_summaries: &'a [&'a str],
    /// Reviewed tool call descriptor.
    pub tool: ToolCallDescriptorDraft<'a>,
    /// Reviewed action class.
    pub action_class: super::ActionClass,
    /// Active policy labels that constrain the review.
    pub active_policy_labels: &'a [&'a str],
    /// Raw evidence values to hash and discard before storage.
    pub raw_evidence: &'a [&'a str],
}

/// Validated security review input with only safe labels, summaries, and hashes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Review {
    task_labels: super::Collection<super::Text>,
    recent_context_summaries: super::Collection<super::Text>,
    tool: ToolCallDescriptor,
    action_class: super::ActionClass,
    active_policy_labels: super::Collection<super::Text>,
    evidence_hashes: super::Collection<super::Text>,
}

impl Review {
    /// Validates and redacts a borrowed security review input.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when required safe labels are
    /// missing, unsafe material appears in safe fields, collections exceed their
    /// ceilings, or raw evidence cannot be hashed within adapter bounds.
    pub fn validate(draft: &Draft<'_>) -> Result<Self, super::Error> {
        let task_labels =
            required_safe_collection(super::TASK_LABEL_FIELD, draft.task_labels, super::MAX_TASK_LABEL_ITEMS)?;
        let recent_context_summaries = required_safe_collection(
            super::CONTEXT_SUMMARY_FIELD,
            draft.recent_context_summaries,
            super::MAX_CONTEXT_SUMMARY_ITEMS,
        )?;
        let tool = ToolCallDescriptor::validate(&draft.tool)?;
        let active_policy_labels = required_safe_collection(
            super::POLICY_LABEL_FIELD,
            draft.active_policy_labels,
            super::MAX_POLICY_LABEL_ITEMS,
        )?;
        let evidence_hashes =
            raw_evidence_hashes(super::RAW_EVIDENCE_FIELD, draft.raw_evidence, super::MAX_EVIDENCE_ITEMS)?;
        Ok(Self {
            task_labels,
            recent_context_summaries,
            tool,
            action_class: draft.action_class,
            active_policy_labels,
            evidence_hashes,
        })
    }

    /// Returns original task labels safe for review.
    #[must_use]
    pub fn task_labels(&self) -> &[super::Text] {
        self.task_labels.as_slice()
    }

    /// Returns bounded context summaries safe for review.
    #[must_use]
    pub fn recent_context_summaries(&self) -> &[super::Text] {
        self.recent_context_summaries.as_slice()
    }

    /// Returns the reviewed tool descriptor.
    #[must_use]
    pub const fn tool(&self) -> &ToolCallDescriptor {
        &self.tool
    }

    /// Returns the reviewed action class.
    #[must_use]
    pub const fn action_class(&self) -> super::ActionClass {
        self.action_class
    }

    /// Returns the derived risk class.
    #[must_use]
    pub const fn risk_class(&self) -> super::RiskClass {
        self.action_class.risk_class()
    }

    /// Returns active policy labels safe for review.
    #[must_use]
    pub fn active_policy_labels(&self) -> &[super::Text] {
        self.active_policy_labels.as_slice()
    }

    /// Returns redacted BLAKE3 evidence identities.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[super::Text] {
        self.evidence_hashes.as_slice()
    }

    pub(super) fn primary_evidence_hash(&self) -> &str {
        self.evidence_hashes[super::FIRST_EVIDENCE_INDEX].as_str()
    }
}

pub(super) fn required_safe_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<super::Text, super::Error> {
    let text = crate::validation::normalized_adapter_field(field, value.unwrap_or_default())?;
    reject_unsafe_text(field, text.as_str())?;
    Ok(text)
}

fn required_safe_collection(
    field: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
) -> Result<super::Collection<super::Text>, super::Error> {
    crate::validation::validate_required_adapter_field(field, required_marker(values))?;
    bounded_collection(field, values, max_items, true)
}

pub(super) fn raw_evidence_hashes(
    field: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
) -> Result<super::Collection<super::Text>, super::Error> {
    crate::validation::validate_required_adapter_field(field, required_marker(values))?;
    evidence_hashes(field, values, max_items)
}

pub(super) fn optional_raw_evidence_hashes(
    field: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
) -> Result<super::Collection<super::Text>, super::Error> {
    evidence_hashes(field, values, max_items)
}

fn reject_unsafe_text(field: crate::validation::FieldName, text: &str) -> Result<(), super::Error> {
    if let Some(leak) = super::evidence::detect_leak(text) {
        return Err(super::Error::UnsafeText {
            field: field.as_str(),
            leak,
        });
    }
    Ok(())
}

fn evidence_hashes(
    field: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
) -> Result<super::Collection<super::Text>, super::Error> {
    item_count(CountLimit {
        field,
        actual_items: values.len(),
        max_items,
    })?;
    let mut hashes = super::Collection::with_capacity(values.len());
    for value in values {
        crate::validation::validate_optional_adapter_field(field, value)?;
        hashes.push(super::hash_parts(&[super::HASH_SCHEMA, field.as_str(), value]));
    }
    Ok(hashes)
}

fn bounded_collection(
    field: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
    reject_leaks: bool,
) -> Result<super::Collection<super::Text>, super::Error> {
    item_count(CountLimit {
        field,
        actual_items: values.len(),
        max_items,
    })?;
    let mut normalized = super::Collection::with_capacity(values.len());
    for value in values {
        let text = crate::validation::normalized_adapter_field(field, value)?;
        if reject_leaks {
            reject_unsafe_text(field, text.as_str())?;
        }
        normalized.push(text);
    }
    Ok(normalized)
}

fn item_count(limit: CountLimit) -> Result<(), super::Error> {
    if limit.actual_items > limit.max_items {
        return Err(super::Error::CollectionTooLarge {
            field: limit.field.as_str(),
            max_items: limit.max_items,
            actual_items: limit.actual_items,
        });
    }
    Ok(())
}

fn required_marker(values: &[&str]) -> &'static str {
    if values.is_empty() { "" } else { "present" }
}
