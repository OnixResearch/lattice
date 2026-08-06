/// Output view source class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostOutputSource {
    /// Currently visible output.
    Visible,
    /// Recent wrapped output.
    Recent,
    /// Recent unwrapped output.
    RecentUnwrapped,
    /// Detector-safe output view.
    DetectorSafe,
}

impl HostOutputSource {
    /// Parses an output source label.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, HostControlError> {
        parse_value(SOURCE_FIELD, value, |normalized| match normalized {
            SOURCE_VISIBLE => Some(Self::Visible),
            SOURCE_RECENT => Some(Self::Recent),
            SOURCE_RECENT_UNWRAPPED => Some(Self::RecentUnwrapped),
            SOURCE_DETECTOR_SAFE => Some(Self::DetectorSafe),
            _unsupported => None,
        })
    }

    /// Returns the stable source label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Visible => SOURCE_VISIBLE,
            Self::Recent => SOURCE_RECENT,
            Self::RecentUnwrapped => SOURCE_RECENT_UNWRAPPED,
            Self::DetectorSafe => SOURCE_DETECTOR_SAFE,
        }
    }
}

/// Output truncation status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostTruncationStatus {
    /// Output view is complete.
    Complete,
    /// Output view is truncated.
    Truncated,
}

/// Output redaction status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostRedactionStatus {
    /// Output contains no redaction marker.
    Clean,
    /// Output was redacted by the host.
    Redacted,
}

impl HostRedactionStatus {
    /// Parses a redaction label.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, HostControlError> {
        parse_value(REDACTION_FIELD, value, |normalized| match normalized {
            REDACTION_CLEAN => Some(Self::Clean),
            REDACTION_REDACTED => Some(Self::Redacted),
            _unsupported => None,
        })
    }
}

/// Borrowed output read fields before bounding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostOutputReadDraft<'a> {
    /// Output source label.
    pub source: &'a str,
    /// Raw host output supplied by the host adapter.
    pub body: &'a str,
    /// Maximum visible lines allowed in the view.
    pub visible_line_budget: u32,
    /// Redaction status label.
    pub redaction_status: &'a str,
    /// Optional host-owned safe reference for full output.
    pub full_output_reference: Option<&'a str>,
}

/// Bounded host output view.
// r[impl agent_harness_runtime.host_control.bounded_reads]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostOutputView {
    source: HostOutputSource,
    visible_text: String,
    truncation_status: HostTruncationStatus,
    redaction_status: HostRedactionStatus,
    byte_count: u64,
    line_count: u64,
    content_hash: Option<String>,
    full_output_reference: Option<String>,
}

impl HostOutputView {
    /// Bounds host output for safe runtime and receipt use.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when source, bounds, references, or output are unsafe.
    pub fn new(draft: HostOutputReadDraft<'_>) -> Result<Self, HostControlError> {
        validate_line_budget(draft.visible_line_budget)?;
        crate::validation::validate_optional_adapter_field(OUTPUT_FIELD, draft.body)?;
        if SECRET_MARKERS
            .iter()
            .any(|marker| draft.body.contains(marker))
        {
            return Err(HostControlError::UnsafeOutput);
        }
        let source = HostOutputSource::parse(draft.source)?;
        let redaction_status = HostRedactionStatus::parse(draft.redaction_status)?;
        let line_count = count_lines(draft.body);
        let visible_line_budget = usize::try_from(draft.visible_line_budget)
            .map_err(|_error| HostControlError::UnboundedRead)?;
        let truncation_status = if line_count > visible_line_budget {
            HostTruncationStatus::Truncated
        } else {
            HostTruncationStatus::Complete
        };
        let visible_text = visible_lines(draft.body, visible_line_budget);
        let content_hash = (truncation_status == HostTruncationStatus::Truncated)
            .then(|| hash_parts(&[OUTPUT_HASH_SCHEMA, draft.body]));
        Ok(Self {
            source,
            visible_text,
            truncation_status,
            redaction_status,
            byte_count: usize_to_u64(draft.body.len())?,
            line_count: usize_to_u64(line_count)?,
            content_hash,
            full_output_reference: optional_reference(draft.full_output_reference)?,
        })
    }

    /// Returns the output source class.
    #[must_use]
    pub const fn source(&self) -> HostOutputSource {
        self.source
    }

    /// Returns bounded visible text.
    #[must_use]
    pub fn visible_text(&self) -> &str {
        self.visible_text.as_str()
    }

    /// Returns truncation status.
    #[must_use]
    pub const fn truncation_status(&self) -> HostTruncationStatus {
        self.truncation_status
    }

    /// Returns redaction status.
    #[must_use]
    pub const fn redaction_status(&self) -> HostRedactionStatus {
        self.redaction_status
    }

    /// Returns raw byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    /// Returns raw line count.
    #[must_use]
    pub const fn line_count(&self) -> u64 {
        self.line_count
    }

    /// Returns BLAKE3 content hash when truncation requires identity evidence.
    #[must_use]
    pub fn content_hash(&self) -> Option<&str> {
        self.content_hash.as_deref()
    }

    /// Returns safe full-output reference when supplied.
    #[must_use]
    pub fn full_output_reference(&self) -> Option<&str> {
        self.full_output_reference.as_deref()
    }
}
