/// Text normalization observed by a hashed edit snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextNormalization {
    /// Snapshot text used LF line endings.
    Lf,
    /// Snapshot text used CRLF line endings.
    CrLf,
    /// Snapshot text had mixed endings normalized to LF before hashing tags.
    MixedNormalizedLf,
}

impl TextNormalization {
    /// Parses a text-normalization label.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the value is empty, malformed, or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::Error> {
        let normalized = super::required_label(super::NORMALIZATION_FIELD, value)?;
        match normalized.as_str() {
            super::NORMALIZATION_LF => Ok(Self::Lf),
            super::NORMALIZATION_CRLF => Ok(Self::CrLf),
            super::NORMALIZATION_MIXED_LF => Ok(Self::MixedNormalizedLf),
            _ => Err(super::Error::UnsupportedValue {
                field: super::NORMALIZATION_FIELD.as_str(),
                value: normalized,
            }),
        }
    }

    /// Returns the stable normalization label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => super::NORMALIZATION_LF,
            Self::CrLf => super::NORMALIZATION_CRLF,
            Self::MixedNormalizedLf => super::NORMALIZATION_MIXED_LF,
        }
    }
}

/// Newline behavior requested for validated replacement text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NewlinePolicy {
    /// Preserve the snapshot newline style selected by the host.
    PreserveSnapshot,
    /// Normalize replacements to LF before host mutation.
    NormalizeLf,
}

impl NewlinePolicy {
    /// Parses a newline-policy label.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the value is empty, malformed, or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::Error> {
        let normalized = super::required_label(super::NEWLINE_POLICY_FIELD, value)?;
        match normalized.as_str() {
            super::NEWLINE_POLICY_PRESERVE => Ok(Self::PreserveSnapshot),
            super::NEWLINE_POLICY_NORMALIZE_LF => Ok(Self::NormalizeLf),
            _ => Err(super::Error::UnsupportedValue {
                field: super::NEWLINE_POLICY_FIELD.as_str(),
                value: normalized,
            }),
        }
    }

    /// Returns the stable newline-policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreserveSnapshot => super::NEWLINE_POLICY_PRESERVE,
            Self::NormalizeLf => super::NEWLINE_POLICY_NORMALIZE_LF,
        }
    }
}

/// Resource-identity redaction class used by hashed edit receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceRedaction {
    /// Omit the resource identity from the receipt.
    Omitted,
    /// Store a caller-supplied relative path only.
    RelativePath,
    /// Store a BLAKE3 resource identity hash.
    ResourceHash,
}

impl ResourceRedaction {
    /// Parses a resource-redaction label.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the value is empty, malformed, or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::Error> {
        let normalized = super::required_label(super::RESOURCE_IDENTITY_FIELD, value)?;
        match normalized.as_str() {
            super::RESOURCE_REDACTION_OMITTED => Ok(Self::Omitted),
            super::RESOURCE_REDACTION_RELATIVE_PATH => Ok(Self::RelativePath),
            super::RESOURCE_REDACTION_RESOURCE_HASH => Ok(Self::ResourceHash),
            _ => Err(super::Error::UnsupportedValue {
                field: super::RESOURCE_IDENTITY_FIELD.as_str(),
                value: normalized,
            }),
        }
    }

    /// Returns the stable resource-redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Omitted => super::RESOURCE_REDACTION_OMITTED,
            Self::RelativePath => super::RESOURCE_REDACTION_RELATIVE_PATH,
            Self::ResourceHash => super::RESOURCE_REDACTION_RESOURCE_HASH,
        }
    }
}

/// Validated span with bounded replacement text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validated {
    pub(super) start_tag: super::Text,
    pub(super) end_tag: super::Text,
    pub(super) start_line: u32,
    pub(super) end_line: u32,
    pub(super) replacement: super::Text,
    pub(super) replacement_chars: u64,
    pub(super) replacement_hash: super::Text,
}

impl Validated {
    /// Returns the first selected tag.
    #[must_use]
    pub fn start_tag(&self) -> &str {
        self.start_tag.as_str()
    }

    /// Returns the last selected tag.
    #[must_use]
    pub fn end_tag(&self) -> &str {
        self.end_tag.as_str()
    }

    /// Returns the first selected line.
    #[must_use]
    pub const fn start_line(&self) -> u32 {
        self.start_line
    }

    /// Returns the last selected line.
    #[must_use]
    pub const fn end_line(&self) -> u32 {
        self.end_line
    }

    /// Returns the bounded replacement text for the host adapter.
    #[must_use]
    pub fn replacement(&self) -> &str {
        self.replacement.as_str()
    }

    /// Returns the replacement character count.
    #[must_use]
    pub const fn replacement_chars(&self) -> u64 {
        self.replacement_chars
    }

    /// Returns the BLAKE3 hash of the replacement operation.
    #[must_use]
    pub fn replacement_hash(&self) -> &str {
        self.replacement_hash.as_str()
    }
}

/// Pure edit plan that a host adapter may apply atomically.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    pub(super) resource_class: super::Text,
    pub(super) snapshot_hash: super::Text,
    pub(super) newline_policy: NewlinePolicy,
    pub(super) spans: super::Collection<Validated>,
    pub(super) hash: super::Text,
}

impl Plan {
    /// Returns the safe resource class for the target.
    #[must_use]
    pub fn resource_class(&self) -> &str {
        self.resource_class.as_str()
    }

    /// Returns the before-edit snapshot hash.
    #[must_use]
    pub fn snapshot_hash(&self) -> &str {
        self.snapshot_hash.as_str()
    }

    /// Returns the replacement newline policy.
    #[must_use]
    pub const fn newline_policy(&self) -> NewlinePolicy {
        self.newline_policy
    }

    /// Returns validated span edits in source order.
    #[must_use]
    pub fn spans(&self) -> &[Validated] {
        self.spans.as_slice()
    }

    /// Returns the BLAKE3 hash of the safe edit-plan fields.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.hash.as_str()
    }
}
