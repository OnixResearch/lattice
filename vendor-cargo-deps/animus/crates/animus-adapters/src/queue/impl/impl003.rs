/// Borrowed queue group status input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueGroupStatusDraft<'a> {
    /// Queue group label.
    pub group: &'a str,
    /// Host-configured parallelism limit for this group.
    pub parallelism_limit: u32,
    /// Stable status counts.
    pub counts: QueueStatusCounts,
    /// Backend-specific safe labels.
    pub backend_labels: &'a [&'a str],
}

/// Validated queue group status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueGroupStatus {
    group: String,
    parallelism_limit: u32,
    counts: QueueStatusCounts,
    backend_labels: Vec<String>,
}

impl QueueGroupStatus {
    /// Validates group status metadata.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError`] when the group label is unsafe or the
    /// parallelism limit is zero.
    pub fn new(draft: QueueGroupStatusDraft<'_>) -> Result<Self, QueueContractError> {
        if draft.parallelism_limit < MIN_POSITIVE_COUNT {
            return Err(QueueContractError::NonPositiveCount {
                field: PARALLELISM_FIELD,
            });
        }
        let group = crate::validation::normalized_adapter_field(GROUP_FIELD, draft.group)?;
        let backend_labels = normalize_label_slice(BACKEND_LABEL_FIELD, draft.backend_labels)?;
        Ok(Self {
            group,
            parallelism_limit: draft.parallelism_limit,
            counts: draft.counts,
            backend_labels,
        })
    }

    /// Returns the group label.
    #[must_use]
    pub fn group(&self) -> &str {
        self.group.as_str()
    }

    /// Returns the configured parallelism limit.
    #[must_use]
    pub const fn parallelism_limit(&self) -> u32 {
        self.parallelism_limit
    }

    /// Returns stable status counts.
    #[must_use]
    pub const fn counts(&self) -> QueueStatusCounts {
        self.counts
    }

    /// Returns backend-specific safe labels.
    #[must_use]
    pub fn backend_labels(&self) -> &[String] {
        self.backend_labels.as_slice()
    }
}

/// Borrowed log viewport input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueLogViewportDraft<'a> {
    /// Task ID whose log is represented.
    pub task_id: &'a str,
    /// Visible bounded log text.
    pub visible_text: &'a str,
    /// Total line count reported by the host.
    pub total_line_count: u32,
    /// Total byte count reported by the host.
    pub total_byte_count: u32,
    /// Visible line count included in this viewport.
    pub visible_line_count: u32,
    /// Visible byte count included in this viewport.
    pub visible_byte_count: u32,
    /// True when visible text omits hidden log content.
    pub truncated: bool,
    /// Optional BLAKE3 hash of the full log content.
    pub full_output_hash: Option<&'a str>,
    /// Optional safe host reference for out-of-band inspection.
    pub safe_reference: Option<&'a str>,
}

/// Validated bounded log viewport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueLogViewport {
    task_id: String,
    visible_text: String,
    total_line_count: u32,
    total_byte_count: u32,
    visible_line_count: u32,
    visible_byte_count: u32,
    truncated: bool,
    full_output_hash: Option<crate::SafeHash>,
    safe_reference: Option<String>,
}

impl QueueLogViewport {
    /// Validates a model-visible bounded log viewport.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError`] when counts exceed configured bounds, when
    /// visible counts exceed totals, or when partial output lacks truncation.
    pub fn new(
        draft: QueueLogViewportDraft<'_>,
        bounds: QueueLogBounds,
    ) -> Result<Self, QueueContractError> {
        validate_log_counts(&draft, bounds)?;
        let task_id = crate::validation::normalized_adapter_field(TASK_ID_FIELD, draft.task_id)?;
        crate::validation::validate_optional_adapter_field(LOG_TEXT_FIELD, draft.visible_text)?;
        let full_output_hash = draft
            .full_output_hash
            .map(crate::SafeHash::new)
            .transpose()?;
        let safe_reference = draft
            .safe_reference
            .map(|reference| {
                crate::validation::normalized_adapter_field(SAFE_REFERENCE_FIELD, reference)
            })
            .transpose()?;
        Ok(Self {
            task_id,
            visible_text: String::from(draft.visible_text),
            total_line_count: draft.total_line_count,
            total_byte_count: draft.total_byte_count,
            visible_line_count: draft.visible_line_count,
            visible_byte_count: draft.visible_byte_count,
            truncated: draft.truncated,
            full_output_hash,
            safe_reference,
        })
    }

    /// Returns the task ID.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns the bounded visible log text.
    #[must_use]
    pub fn visible_text(&self) -> &str {
        self.visible_text.as_str()
    }

    /// Returns the total line count.
    #[must_use]
    pub const fn total_line_count(&self) -> u32 {
        self.total_line_count
    }

    /// Returns the total byte count.
    #[must_use]
    pub const fn total_byte_count(&self) -> u32 {
        self.total_byte_count
    }

    /// Returns the visible line count.
    #[must_use]
    pub const fn visible_line_count(&self) -> u32 {
        self.visible_line_count
    }

    /// Returns the visible byte count.
    #[must_use]
    pub const fn visible_byte_count(&self) -> u32 {
        self.visible_byte_count
    }

    /// Returns whether the viewport is truncated.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the optional full-output hash.
    #[must_use]
    pub const fn full_output_hash(&self) -> Option<&crate::SafeHash> {
        self.full_output_hash.as_ref()
    }

    /// Returns the optional safe reference.
    #[must_use]
    pub fn safe_reference(&self) -> Option<&str> {
        self.safe_reference.as_deref()
    }
}
