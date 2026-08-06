/// Borrowed optional host-supplied image metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageMetadataDraft<'a> {
    /// Optional width in pixels.
    pub width_px: Option<u32>,
    /// Optional height in pixels.
    pub height_px: Option<u32>,
    /// Optional host-declared detail preference.
    pub detail: Option<super::ImageDetailPreference>,
    /// Optional safe adapter class label.
    pub adapter_class: Option<&'a str>,
}

/// Validated optional host-supplied image metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageMetadata {
    width_px: Option<u32>,
    height_px: Option<u32>,
    detail: Option<super::ImageDetailPreference>,
    adapter_class: Option<alloc::string::String>,
}

impl ImageMetadata {
    /// Validates optional host-supplied image metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when dimensions are zero, implausibly large, or labels are unsafe.
    pub fn from_draft(draft: ImageMetadataDraft<'_>) -> Result<Self, super::Error> {
        super::check::validate_dimensions(draft.width_px, draft.height_px)?;
        let adapter_class = draft
            .adapter_class
            .map(|raw| super::check::normalized_safe_text(super::ADAPTER_CLASS_FIELD, raw))
            .transpose()?;
        Ok(Self {
            width_px: draft.width_px,
            height_px: draft.height_px,
            detail: draft.detail,
            adapter_class,
        })
    }

    /// Returns optional width in pixels.
    #[must_use]
    pub const fn width_px(&self) -> Option<u32> {
        self.width_px
    }

    /// Returns optional height in pixels.
    #[must_use]
    pub const fn height_px(&self) -> Option<u32> {
        self.height_px
    }

    /// Returns optional detail preference.
    #[must_use]
    pub const fn detail(&self) -> Option<super::ImageDetailPreference> {
        self.detail
    }

    /// Returns optional safe adapter class.
    #[must_use]
    pub fn adapter_class(&self) -> Option<&str> {
        self.adapter_class.as_deref()
    }
}

/// Borrowed image content-part descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImagePartDraft<'a> {
    /// Safe source label supplied by the host.
    pub safe_label: &'a str,
    /// Declared image media type.
    pub media_type: &'a str,
    /// Image byte count supplied by the host.
    pub byte_count: u64,
    /// BLAKE3 content identity for the image bytes held by the host.
    pub content_hash: &'a str,
    /// Source trust metadata.
    pub source_trust: super::ImageSourceTrust,
    /// Optional host-supplied metadata.
    pub metadata: Option<ImageMetadataDraft<'a>>,
    /// Optional safe descriptor hints; unsafe raw material is rejected.
    pub descriptor_hints: &'a [&'a str],
}

/// Validated image content part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImagePart {
    safe_label: alloc::string::String,
    media_type: super::ImageMediaType,
    byte_count: u64,
    content_hash: alloc::string::String,
    source_trust: super::ImageSourceTrust,
    metadata: Option<ImageMetadata>,
    descriptor_hints: alloc::vec::Vec<alloc::string::String>,
}

impl ImagePart {
    /// Validates an image descriptor for reusable runtime use.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when required metadata is missing, unsupported, oversized, or
    /// unsafe.
    pub fn from_draft(draft: ImagePartDraft<'_>) -> Result<Self, super::Error> {
        let safe_label = super::check::normalized_safe_text(super::SAFE_LABEL_FIELD, draft.safe_label)?;
        let media_type = super::ImageMediaType::parse(draft.media_type)?;
        super::check::validate_byte_count(super::check::ByteLimitInput {
            byte_count: draft.byte_count,
            max_bytes: super::MAX_IMAGE_BYTE_COUNT,
        })?;
        let content_hash = super::check::validate_content_hash(draft.content_hash)?;
        let metadata = draft.metadata.map(ImageMetadata::from_draft).transpose()?;
        let descriptor_hints = super::check::validate_hints(draft.descriptor_hints)?;
        Ok(Self {
            safe_label,
            media_type,
            byte_count: draft.byte_count,
            content_hash,
            source_trust: draft.source_trust,
            metadata,
            descriptor_hints,
        })
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn safe_label(&self) -> &str {
        self.safe_label.as_str()
    }

    /// Returns the media type.
    #[must_use]
    pub const fn media_type(&self) -> super::ImageMediaType {
        self.media_type
    }

    /// Returns the host-supplied byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    /// Returns the BLAKE3 content identity.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }

    /// Returns source trust metadata.
    #[must_use]
    pub const fn source_trust(&self) -> super::ImageSourceTrust {
        self.source_trust
    }

    /// Returns optional host-supplied metadata.
    #[must_use]
    pub const fn metadata(&self) -> Option<&ImageMetadata> {
        self.metadata.as_ref()
    }

    /// Returns safe descriptor hints.
    #[must_use]
    pub fn descriptor_hints(&self) -> &[alloc::string::String] {
        self.descriptor_hints.as_slice()
    }
}
