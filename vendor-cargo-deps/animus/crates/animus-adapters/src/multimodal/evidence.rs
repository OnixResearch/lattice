use alloc::string::ToString;

/// Borrowed safe image receipt metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageReceiptDraft<'a> {
    /// Image outcome status.
    pub status: super::ImageOutcomeStatus,
    /// Safe image source label.
    pub safe_label: &'a str,
    /// Image media type.
    pub media_type: &'a str,
    /// Image byte count.
    pub byte_count: u64,
    /// BLAKE3 content identity.
    pub content_hash: &'a str,
    /// Optional width in pixels.
    pub width_px: Option<u32>,
    /// Optional height in pixels.
    pub height_px: Option<u32>,
    /// Optional safe adapter class.
    pub adapter_class: Option<&'a str>,
    /// Safe redaction marker.
    pub redaction_marker: &'a str,
    /// Additional safe metadata fragments.
    pub metadata_fragments: &'a [&'a str],
}

/// Safe image receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageReceipt {
    modality: &'static str,
    status: super::ImageOutcomeStatus,
    safe_label: alloc::string::String,
    media_type: super::ImageMediaType,
    byte_count: u64,
    content_hash: alloc::string::String,
    width_px: Option<u32>,
    height_px: Option<u32>,
    adapter_class: Option<alloc::string::String>,
    redaction_marker: alloc::string::String,
    metadata_fragments: alloc::vec::Vec<alloc::string::String>,
    receipt_hash: alloc::string::String,
}

impl ImageReceipt {
    /// Validates safe image receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when receipt fields contain raw bytes, base64/data URLs, paths, credentials, or provider payloads.
    pub fn validate(draft: &ImageReceiptDraft<'_>) -> Result<Self, super::Error> {
        let safe_label =
            super::check::normalized_safe_text(super::SAFE_LABEL_FIELD, draft.safe_label)?;
        let media_type = super::ImageMediaType::parse(draft.media_type)?;
        super::check::validate_byte_count(super::check::ByteLimitInput {
            byte_count: draft.byte_count,
            max_bytes: super::MAX_IMAGE_BYTE_COUNT,
        })?;
        super::check::validate_dimensions(draft.width_px, draft.height_px)?;
        let content_hash = super::check::validate_content_hash(draft.content_hash)?;
        let adapter_class = draft
            .adapter_class
            .map(|raw| super::check::normalized_safe_text(super::ADAPTER_CLASS_FIELD, raw))
            .transpose()?;
        let redaction_marker = super::check::normalized_safe_text(
            super::REDACTION_MARKER_FIELD,
            draft.redaction_marker,
        )?;
        let metadata_fragments =
            super::check::validate_receipt_fragments(draft.metadata_fragments)?;
        let receipt_hash = image_receipt_hash(ImageReceiptHashParts {
            status: draft.status,
            safe_label: safe_label.as_str(),
            media_type,
            byte_count: draft.byte_count,
            content_hash: content_hash.as_str(),
            width_px: draft.width_px,
            height_px: draft.height_px,
            adapter_class: adapter_class.as_deref(),
            redaction_marker: redaction_marker.as_str(),
            metadata_fragments: metadata_fragments.as_slice(),
        });
        Ok(Self {
            modality: super::IMAGE_MODALITY,
            status: draft.status,
            safe_label,
            media_type,
            byte_count: draft.byte_count,
            content_hash,
            width_px: draft.width_px,
            height_px: draft.height_px,
            adapter_class,
            redaction_marker,
            metadata_fragments,
            receipt_hash,
        })
    }

    /// Returns the modality label recorded by all image receipts.
    #[must_use]
    pub const fn modality(&self) -> &'static str {
        self.modality
    }

    /// Returns the image outcome status.
    #[must_use]
    pub const fn status(&self) -> super::ImageOutcomeStatus {
        self.status
    }

    /// Returns the safe image label.
    #[must_use]
    pub fn safe_label(&self) -> &str {
        self.safe_label.as_str()
    }

    /// Returns the media type.
    #[must_use]
    pub const fn media_type(&self) -> super::ImageMediaType {
        self.media_type
    }

    /// Returns the image byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    /// Returns the BLAKE3 content hash.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
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

    /// Returns optional safe adapter class.
    #[must_use]
    pub fn adapter_class(&self) -> Option<&str> {
        self.adapter_class.as_deref()
    }

    /// Returns the safe redaction marker.
    #[must_use]
    pub fn redaction_marker(&self) -> &str {
        self.redaction_marker.as_str()
    }

    /// Returns additional safe metadata fragments.
    #[must_use]
    pub fn metadata_fragments(&self) -> &[alloc::string::String] {
        self.metadata_fragments.as_slice()
    }

    /// Returns the deterministic BLAKE3 receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

#[derive(Clone, Copy)]
struct ImageReceiptHashParts<'a> {
    status: super::ImageOutcomeStatus,
    safe_label: &'a str,
    media_type: super::ImageMediaType,
    byte_count: u64,
    content_hash: &'a str,
    width_px: Option<u32>,
    height_px: Option<u32>,
    adapter_class: Option<&'a str>,
    redaction_marker: &'a str,
    metadata_fragments: &'a [alloc::string::String],
}

fn image_receipt_hash(parts: ImageReceiptHashParts<'_>) -> alloc::string::String {
    assert!(!parts.safe_label.is_empty());
    assert_eq!(parts.content_hash.len(), super::BLAKE3_HEX_CHARS);
    let mut normalizer = HashNormalizer::new(super::RECEIPT_SCHEMA);
    normalizer.push_part(parts.status.as_str());
    normalizer.push_part(parts.safe_label);
    normalizer.push_part(parts.media_type.as_str());
    normalizer.push_part(parts.byte_count.to_string().as_str());
    normalizer.push_part(parts.content_hash);
    normalizer.push_optional_u32(parts.width_px);
    normalizer.push_optional_u32(parts.height_px);
    normalizer.push_part(parts.adapter_class.unwrap_or_default());
    normalizer.push_part(parts.redaction_marker);
    for fragment in parts.metadata_fragments {
        normalizer.push_part(fragment.as_str());
    }
    normalizer.finish()
}

struct HashNormalizer {
    value: alloc::string::String,
}

impl HashNormalizer {
    fn new(schema: &str) -> Self {
        Self {
            value: alloc::string::String::from(schema),
        }
    }

    fn push_part(&mut self, part: &str) {
        self.value.push(super::HASH_PART_SEPARATOR);
        self.value.push_str(part);
    }

    fn push_optional_u32(&mut self, value: Option<u32>) {
        match value {
            Some(raw) => self.push_part(raw.to_string().as_str()),
            None => self.push_part(""),
        }
    }

    fn finish(self) -> alloc::string::String {
        blake3::hash(self.value.as_bytes()).to_hex().to_string()
    }
}
