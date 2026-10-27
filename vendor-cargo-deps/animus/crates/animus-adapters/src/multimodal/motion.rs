use alloc::string::ToString;

const VIDEO_SAFE_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_safe_label");
const VIDEO_MEDIA_TYPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_media_type");
const VIDEO_CONTENT_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_content_hash");
const VIDEO_CODEC_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_codec");
const VIDEO_CONTAINER_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_container");
const VIDEO_HANDLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_payload_handle");
const VIDEO_PROVIDER_PROFILE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_provider_profile");
const VIDEO_SERVICE_CLASS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_service_class");
const VIDEO_RECEIPT_FRAGMENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("video_receipt_fragment");
const VIDEO_RECEIPT_SCHEMA: &str = "animus-provider-video-receipt-v1";
const VIDEO_MODALITY: &str = "video";
const BYTES_PER_KIB: u64 = 1_024;
const KIB_PER_MIB: u64 = 1_024;
const MAX_VIDEO_MIB: u64 = 512;
const DEFAULT_VIDEO_MIB: u64 = 100;
const DEFAULT_AGGREGATE_MEDIA_MIB: u64 = 128;
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const SECONDS_PER_MINUTE: u64 = 60;
const MAX_VIDEO_MINUTES: u64 = 20;
const MAX_FRAME_RATE_MILLIHZ: u32 = 120_000;
const MIN_DECLARED_VALUE: u64 = 1;
const DEFAULT_MAX_VIDEO_COUNT: u64 = 4;

/// Largest admitted provider-video descriptor byte count.
pub const MAX_VIDEO_BYTE_COUNT: u64 = mib_to_bytes(MAX_VIDEO_MIB);
/// Default per-video assembly budget.
pub const DEFAULT_PROMPT_VIDEO_BYTE_BUDGET: u64 = mib_to_bytes(DEFAULT_VIDEO_MIB);
/// Default aggregate image and video assembly budget.
pub const DEFAULT_AGGREGATE_MEDIA_BYTE_BUDGET: u64 = mib_to_bytes(DEFAULT_AGGREGATE_MEDIA_MIB);
/// Largest admitted declared video duration.
pub const MAX_VIDEO_DURATION_MS: u64 = MAX_VIDEO_MINUTES
    .saturating_mul(SECONDS_PER_MINUTE)
    .saturating_mul(MILLISECONDS_PER_SECOND);
/// Default maximum number of included video parts.
pub const DEFAULT_PROMPT_VIDEO_COUNT: u64 = DEFAULT_MAX_VIDEO_COUNT;

const fn mib_to_bytes(mib: u64) -> u64 {
    mib.saturating_mul(KIB_PER_MIB)
        .saturating_mul(BYTES_PER_KIB)
}

/// Provider-neutral video media type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoMediaType {
    /// `video/mp4`.
    Mp4,
    /// `video/webm`.
    Webm,
    /// `video/quicktime`.
    QuickTime,
    /// `video/mpeg`.
    Mpeg,
}

impl VideoMediaType {
    /// Parses an admitted video media type.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] for unsafe or unsupported values.
    pub fn parse(raw: &str) -> Result<Self, super::Error> {
        let normalized = super::check::normalized_safe_text(VIDEO_MEDIA_TYPE_FIELD, raw)?;
        match normalized.as_str() {
            "video/mp4" => Ok(Self::Mp4),
            "video/webm" => Ok(Self::Webm),
            "video/quicktime" => Ok(Self::QuickTime),
            "video/mpeg" => Ok(Self::Mpeg),
            _unsupported => Err(super::Error::UnsupportedVideoMediaType { value: normalized }),
        }
    }

    /// Returns the stable media type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mp4 => "video/mp4",
            Self::Webm => "video/webm",
            Self::QuickTime => "video/quicktime",
            Self::Mpeg => "video/mpeg",
        }
    }
}

/// Trust classification for a video source descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoSourceTrust {
    /// User explicitly supplied the video.
    UserProvided,
    /// The host admitted a workspace artifact.
    WorkspaceArtifact,
    /// A trusted host transform produced the video.
    HostGenerated,
    /// The source is external and untrusted.
    ExternalUntrusted,
}

impl VideoSourceTrust {
    /// Returns the stable trust label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserProvided => "user_provided",
            Self::WorkspaceArtifact => "workspace_artifact",
            Self::HostGenerated => "host_generated",
            Self::ExternalUntrusted => "external_untrusted",
        }
    }
}

/// Host-declared video detail preference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoDetailPreference {
    /// Use the reviewed provider default.
    Auto,
    /// Prefer bounded low-detail processing.
    Low,
    /// Prefer high-detail processing when admitted.
    High,
}

impl VideoDetailPreference {
    /// Returns the stable detail label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Low => "low",
            Self::High => "high",
        }
    }
}

/// Borrowed host-declared video metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VideoMetadataDraft<'a> {
    /// Declared duration in milliseconds.
    pub duration_ms: Option<u64>,
    /// Declared width in pixels.
    pub width_px: Option<u32>,
    /// Declared height in pixels.
    pub height_px: Option<u32>,
    /// Declared frame rate in milli-hertz.
    pub frame_rate_millihz: Option<u32>,
    /// Optional safe codec label.
    pub codec: Option<&'a str>,
    /// Optional safe container label.
    pub container: Option<&'a str>,
    /// Optional declared audio-presence marker.
    pub has_audio: Option<bool>,
    /// Optional detail preference.
    pub detail: Option<VideoDetailPreference>,
}

/// Validated host-declared video metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoMetadata {
    duration_ms: Option<u64>,
    width_px: Option<u32>,
    height_px: Option<u32>,
    frame_rate_millihz: Option<u32>,
    codec: Option<alloc::string::String>,
    container: Option<alloc::string::String>,
    has_audio: Option<bool>,
    detail: Option<VideoDetailPreference>,
}

impl VideoMetadata {
    /// Validates declared metadata without inspecting media bytes.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] for invalid bounds or unsafe labels.
    pub fn from_draft(draft: VideoMetadataDraft<'_>) -> Result<Self, super::Error> {
        validate_video_metadata(&draft)?;
        Ok(Self {
            duration_ms: draft.duration_ms,
            width_px: draft.width_px,
            height_px: draft.height_px,
            frame_rate_millihz: draft.frame_rate_millihz,
            codec: draft
                .codec
                .map(|raw| super::check::normalized_safe_text(VIDEO_CODEC_FIELD, raw))
                .transpose()?,
            container: draft
                .container
                .map(|raw| super::check::normalized_safe_text(VIDEO_CONTAINER_FIELD, raw))
                .transpose()?,
            has_audio: draft.has_audio,
            detail: draft.detail,
        })
    }

    /// Returns declared duration.
    #[must_use]
    pub const fn duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }
    /// Returns declared width.
    #[must_use]
    pub const fn width_px(&self) -> Option<u32> {
        self.width_px
    }
    /// Returns declared height.
    #[must_use]
    pub const fn height_px(&self) -> Option<u32> {
        self.height_px
    }
    /// Returns declared frame rate.
    #[must_use]
    pub const fn frame_rate_millihz(&self) -> Option<u32> {
        self.frame_rate_millihz
    }
    /// Returns the safe codec label.
    #[must_use]
    pub fn codec(&self) -> Option<&str> {
        self.codec.as_deref()
    }
    /// Returns the safe container label.
    #[must_use]
    pub fn container(&self) -> Option<&str> {
        self.container.as_deref()
    }
    /// Returns the audio marker.
    #[must_use]
    pub const fn has_audio(&self) -> Option<bool> {
        self.has_audio
    }
    /// Returns the detail preference.
    #[must_use]
    pub const fn detail(&self) -> Option<VideoDetailPreference> {
        self.detail
    }
}

fn validate_video_metadata(draft: &VideoMetadataDraft<'_>) -> Result<(), super::Error> {
    super::check::validate_dimensions(draft.width_px, draft.height_px)?;
    if let Some(duration_ms) = draft.duration_ms {
        if !(MIN_DECLARED_VALUE..=MAX_VIDEO_DURATION_MS).contains(&duration_ms) {
            return Err(super::Error::InvalidVideoDuration { duration_ms });
        }
    }
    if let Some(frame_rate_millihz) = draft.frame_rate_millihz {
        if frame_rate_millihz == 0 || frame_rate_millihz > MAX_FRAME_RATE_MILLIHZ {
            return Err(super::Error::InvalidVideoFrameRate { frame_rate_millihz });
        }
    }
    Ok(())
}

/// Borrowed provider-neutral video descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VideoPartDraft<'a> {
    /// Safe source label.
    pub safe_label: &'a str,
    /// Declared media type.
    pub media_type: &'a str,
    /// Host-observed byte count.
    pub byte_count: u64,
    /// BLAKE3 identity of host-owned bytes.
    pub content_hash: &'a str,
    /// Source trust classification.
    pub source_trust: VideoSourceTrust,
    /// Optional declared metadata.
    pub metadata: Option<VideoMetadataDraft<'a>>,
    /// Safe descriptor hints only.
    pub descriptor_hints: &'a [&'a str],
}

/// Validated provider-neutral video content part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoPart {
    safe_label: alloc::string::String,
    media_type: VideoMediaType,
    byte_count: u64,
    content_hash: alloc::string::String,
    source_trust: VideoSourceTrust,
    metadata: Option<VideoMetadata>,
    descriptor_hints: alloc::vec::Vec<alloc::string::String>,
}

impl VideoPart {
    /// Validates a video descriptor without media I/O.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] for malformed, unsafe, or over-limit metadata.
    pub fn from_draft(draft: VideoPartDraft<'_>) -> Result<Self, super::Error> {
        let safe_label =
            super::check::normalized_safe_text(VIDEO_SAFE_LABEL_FIELD, draft.safe_label)?;
        let media_type = VideoMediaType::parse(draft.media_type)?;
        if draft.byte_count == 0 {
            return Err(super::Error::MissingByteCount);
        }
        if draft.byte_count > MAX_VIDEO_BYTE_COUNT {
            return Err(super::Error::VideoTooLarge {
                max_bytes: MAX_VIDEO_BYTE_COUNT,
                actual_bytes: draft.byte_count,
            });
        }
        let content_hash = validate_video_hash(draft.content_hash)?;
        let metadata = draft.metadata.map(VideoMetadata::from_draft).transpose()?;
        let descriptor_hints = validate_video_hints(draft.descriptor_hints)?;
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

    /// Returns the safe label.
    #[must_use]
    pub fn safe_label(&self) -> &str {
        self.safe_label.as_str()
    }
    /// Returns the media type.
    #[must_use]
    pub const fn media_type(&self) -> VideoMediaType {
        self.media_type
    }
    /// Returns the byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }
    /// Returns the BLAKE3 content identity.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }
    /// Returns source trust.
    #[must_use]
    pub const fn source_trust(&self) -> VideoSourceTrust {
        self.source_trust
    }
    /// Returns declared metadata.
    #[must_use]
    pub const fn metadata(&self) -> Option<&VideoMetadata> {
        self.metadata.as_ref()
    }
    /// Returns safe hints.
    #[must_use]
    pub fn descriptor_hints(&self) -> &[alloc::string::String] {
        self.descriptor_hints.as_slice()
    }
}

fn validate_video_hash(raw: &str) -> Result<alloc::string::String, super::Error> {
    let normalized = super::check::normalized_safe_text(VIDEO_CONTENT_HASH_FIELD, raw)?;
    let is_valid = normalized.len() == super::BLAKE3_HEX_CHARS
        && normalized
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
    if !is_valid {
        return Err(super::Error::InvalidContentHash {
            field: VIDEO_CONTENT_HASH_FIELD.as_str(),
        });
    }
    Ok(normalized)
}

fn validate_video_hints(
    raw: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, super::Error> {
    let mut result = alloc::vec::Vec::with_capacity(raw.len());
    for hint in raw {
        result.push(super::check::normalized_safe_text(
            super::SOURCE_HINT_FIELD,
            hint,
        )?);
    }
    Ok(result)
}

/// Video assembly status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoOutcomeStatus {
    /// Video was included.
    Included,
    /// Video was rejected by validation or budget policy.
    Rejected,
    /// Explicit policy omitted an optional video.
    Omitted,
    /// Selected capability cannot represent the video.
    Unsupported,
}

impl VideoOutcomeStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Included => "included",
            Self::Rejected => "rejected",
            Self::Omitted => "omitted",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Safe ordered outcome for one video draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoOutcome {
    status: VideoOutcomeStatus,
    safe_label: alloc::string::String,
    content_hash: alloc::string::String,
    reason: Option<alloc::string::String>,
}

impl VideoOutcome {
    pub(super) fn included(video: &VideoPart) -> Self {
        Self::new(VideoOutcomeStatus::Included, video, None)
    }
    pub(super) fn rejected(video: &VideoPart, reason: &str) -> Self {
        Self::new(VideoOutcomeStatus::Rejected, video, Some(reason))
    }
    fn new(status: VideoOutcomeStatus, video: &VideoPart, reason: Option<&str>) -> Self {
        Self {
            status,
            safe_label: alloc::string::String::from(video.safe_label()),
            content_hash: alloc::string::String::from(video.content_hash()),
            reason: reason.map(alloc::string::String::from),
        }
    }
    /// Returns status.
    #[must_use]
    pub const fn status(&self) -> VideoOutcomeStatus {
        self.status
    }
    /// Returns safe label.
    #[must_use]
    pub fn safe_label(&self) -> &str {
        self.safe_label.as_str()
    }
    /// Returns content identity.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }
    /// Returns safe reason.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

/// Provider-native video handle class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoPayloadHandleKind {
    /// Provider file identifier.
    ProviderFileId,
    /// Approved provider-owned remote handle.
    ApprovedRemoteHandle,
    /// Bounded ephemeral upload handle label.
    EphemeralUploadHandle,
}

impl VideoPayloadHandleKind {
    /// Returns the stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderFileId => "provider_file_id",
            Self::ApprovedRemoteHandle => "approved_remote_handle",
            Self::EphemeralUploadHandle => "ephemeral_upload_handle",
        }
    }
}

/// Borrowed safe video payload handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VideoPayloadHandleDraft<'a> {
    /// Handle class.
    pub kind: VideoPayloadHandleKind,
    /// Safe handle label, not a URL or credential.
    pub safe_handle: &'a str,
    /// BLAKE3 content identity.
    pub content_hash: &'a str,
    /// Video media type.
    pub media_type: &'a str,
    /// Whether the host reports this handle as expired.
    pub expired: bool,
}

/// Validated safe video payload handle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoPayloadHandle {
    kind: VideoPayloadHandleKind,
    safe_handle: alloc::string::String,
    content_hash: alloc::string::String,
    media_type: VideoMediaType,
}

impl VideoPayloadHandle {
    /// Validates host-returned safe handle metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] for expired or unsafe handles.
    pub fn from_draft(draft: VideoPayloadHandleDraft<'_>) -> Result<Self, super::Error> {
        if draft.expired {
            return Err(super::Error::ExpiredVideoHandle);
        }
        let safe_handle = super::check::normalized_safe_text(VIDEO_HANDLE_FIELD, draft.safe_handle)
            .map_err(|error| match error {
                super::Error::UnsafeDescriptor { reason, .. } => super::Error::InvalidVideoHandle {
                    field: VIDEO_HANDLE_FIELD.as_str(),
                    reason,
                },
                other => other,
            })?;
        Ok(Self {
            kind: draft.kind,
            safe_handle,
            content_hash: validate_video_hash(draft.content_hash)?,
            media_type: VideoMediaType::parse(draft.media_type)?,
        })
    }
    /// Returns handle class.
    #[must_use]
    pub const fn kind(&self) -> VideoPayloadHandleKind {
        self.kind
    }
    /// Returns safe handle label.
    #[must_use]
    pub fn safe_handle(&self) -> &str {
        self.safe_handle.as_str()
    }
    /// Returns content identity.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }
    /// Returns media type.
    #[must_use]
    pub const fn media_type(&self) -> VideoMediaType {
        self.media_type
    }
}

/// Host video payload service error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoPayloadError {
    /// Service is unavailable.
    Unavailable,
    /// Fixture resolution would require a live call.
    LiveProviderCallRequired,
    /// Upload exceeds a service limit.
    UploadTooLarge,
    /// Service returned unsafe metadata.
    InvalidHandle {
        field: &'static str,
        reason: super::UnsafeContentReason,
    },
}

/// Injected host boundary for media I/O, inspection, upload, and provider calls.
pub trait VideoPayloadService {
    /// Resolves an admitted descriptor to safe provider-handle metadata.
    ///
    /// # Errors
    ///
    /// Returns [`VideoPayloadError`] without exposing provider bodies or credentials.
    fn resolve_video_payload(
        &self,
        video: &VideoPart,
    ) -> Result<VideoPayloadHandle, VideoPayloadError>;
}

/// Provider projection loss class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoLossClass {
    /// Projection is lossless for admitted fields.
    None,
    /// Declared metadata cannot be represented.
    DeclaredMetadata,
    /// Audio-presence intent cannot be represented.
    AudioPresence,
    /// Detail preference cannot be represented.
    DetailPreference,
    /// Ordering cannot be preserved.
    Ordering,
}

impl VideoLossClass {
    /// Returns stable loss label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::DeclaredMetadata => "declared_metadata",
            Self::AudioPresence => "audio_presence",
            Self::DetailPreference => "detail_preference",
            Self::Ordering => "ordering",
        }
    }
}

/// Borrowed safe video receipt fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VideoReceiptDraft<'a> {
    /// Outcome status.
    pub status: VideoOutcomeStatus,
    /// Safe label.
    pub safe_label: &'a str,
    /// Media type.
    pub media_type: &'a str,
    /// Byte count.
    pub byte_count: u64,
    /// BLAKE3 content identity.
    pub content_hash: &'a str,
    /// Declared duration.
    pub duration_ms: Option<u64>,
    /// Declared width.
    pub width_px: Option<u32>,
    /// Declared height.
    pub height_px: Option<u32>,
    /// Declared frame rate.
    pub frame_rate_millihz: Option<u32>,
    /// Safe service class.
    pub service_class: &'a str,
    /// Exact reviewed provider profile.
    pub provider_profile: &'a str,
    /// Redaction marker.
    pub redaction_marker: &'a str,
    /// Projection loss classes.
    pub loss_classes: &'a [VideoLossClass],
    /// Explicit non-claim labels.
    pub non_claims: &'a [&'a str],
    /// Additional bounded safe metadata.
    pub metadata_fragments: &'a [&'a str],
}

/// Deterministic metadata-only video receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoReceipt {
    modality: &'static str,
    status: VideoOutcomeStatus,
    safe_label: alloc::string::String,
    media_type: VideoMediaType,
    byte_count: u64,
    content_hash: alloc::string::String,
    service_class: alloc::string::String,
    provider_profile: alloc::string::String,
    redaction_marker: alloc::string::String,
    loss_classes: alloc::vec::Vec<VideoLossClass>,
    non_claims: alloc::vec::Vec<alloc::string::String>,
    receipt_hash: alloc::string::String,
}

impl VideoReceipt {
    /// Validates and hashes metadata-only receipt fields.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when any field contains content or authority material.
    pub fn validate(draft: &VideoReceiptDraft<'_>) -> Result<Self, super::Error> {
        let metadata = VideoMetadataDraft {
            duration_ms: draft.duration_ms,
            width_px: draft.width_px,
            height_px: draft.height_px,
            frame_rate_millihz: draft.frame_rate_millihz,
            codec: None,
            container: None,
            has_audio: None,
            detail: None,
        };
        validate_video_metadata(&metadata)?;
        if draft.byte_count == 0 || draft.byte_count > MAX_VIDEO_BYTE_COUNT {
            return Err(super::Error::VideoTooLarge {
                max_bytes: MAX_VIDEO_BYTE_COUNT,
                actual_bytes: draft.byte_count,
            });
        }
        let safe_label =
            super::check::normalized_safe_text(VIDEO_SAFE_LABEL_FIELD, draft.safe_label)?;
        let media_type = VideoMediaType::parse(draft.media_type)?;
        let content_hash = validate_video_hash(draft.content_hash)?;
        let service_class =
            super::check::normalized_safe_text(VIDEO_SERVICE_CLASS_FIELD, draft.service_class)?;
        let provider_profile = super::check::normalized_safe_text(
            VIDEO_PROVIDER_PROFILE_FIELD,
            draft.provider_profile,
        )?;
        let redaction_marker = super::check::normalized_safe_text(
            super::REDACTION_MARKER_FIELD,
            draft.redaction_marker,
        )?;
        let non_claims = validate_fragments(draft.non_claims)?;
        let fragments = validate_fragments(draft.metadata_fragments)?;
        let receipt_hash = receipt_hash(&ReceiptHashInput {
            draft,
            safe_label: safe_label.as_str(),
            media_type,
            content_hash: content_hash.as_str(),
            service_class: service_class.as_str(),
            provider_profile: provider_profile.as_str(),
            redaction_marker: redaction_marker.as_str(),
            non_claims: non_claims.as_slice(),
            fragments: fragments.as_slice(),
        });
        Ok(Self {
            modality: VIDEO_MODALITY,
            status: draft.status,
            safe_label,
            media_type,
            byte_count: draft.byte_count,
            content_hash,
            service_class,
            provider_profile,
            redaction_marker,
            loss_classes: draft.loss_classes.to_vec(),
            non_claims,
            receipt_hash,
        })
    }
    /// Returns modality label.
    #[must_use]
    pub const fn modality(&self) -> &'static str {
        self.modality
    }
    /// Returns outcome status.
    #[must_use]
    pub const fn status(&self) -> VideoOutcomeStatus {
        self.status
    }
    /// Returns safe label.
    #[must_use]
    pub fn safe_label(&self) -> &str {
        self.safe_label.as_str()
    }
    /// Returns media type.
    #[must_use]
    pub const fn media_type(&self) -> VideoMediaType {
        self.media_type
    }
    /// Returns byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }
    /// Returns content identity.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }
    /// Returns service class.
    #[must_use]
    pub fn service_class(&self) -> &str {
        self.service_class.as_str()
    }
    /// Returns provider profile.
    #[must_use]
    pub fn provider_profile(&self) -> &str {
        self.provider_profile.as_str()
    }
    /// Returns redaction marker.
    #[must_use]
    pub fn redaction_marker(&self) -> &str {
        self.redaction_marker.as_str()
    }
    /// Returns loss classes.
    #[must_use]
    pub fn loss_classes(&self) -> &[VideoLossClass] {
        self.loss_classes.as_slice()
    }
    /// Returns non-claims.
    #[must_use]
    pub fn non_claims(&self) -> &[alloc::string::String] {
        self.non_claims.as_slice()
    }
    /// Returns deterministic receipt identity.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

fn validate_fragments(
    raw: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, super::Error> {
    let mut result = alloc::vec::Vec::with_capacity(raw.len());
    for item in raw {
        result.push(super::check::normalized_safe_text(
            VIDEO_RECEIPT_FRAGMENT_FIELD,
            item,
        )?);
    }
    Ok(result)
}

struct ReceiptHashInput<'a> {
    draft: &'a VideoReceiptDraft<'a>,
    safe_label: &'a str,
    media_type: VideoMediaType,
    content_hash: &'a str,
    service_class: &'a str,
    provider_profile: &'a str,
    redaction_marker: &'a str,
    non_claims: &'a [alloc::string::String],
    fragments: &'a [alloc::string::String],
}

fn receipt_hash(input: &ReceiptHashInput<'_>) -> alloc::string::String {
    assert!(!input.safe_label.is_empty());
    assert_eq!(input.content_hash.len(), super::BLAKE3_HEX_CHARS);
    let draft = input.draft;
    let mut value = alloc::string::String::from(VIDEO_RECEIPT_SCHEMA);
    let mut push = |part: &str| {
        value.push(super::HASH_PART_SEPARATOR);
        value.push_str(part);
    };
    push(draft.status.as_str());
    push(input.safe_label);
    push(input.media_type.as_str());
    push(draft.byte_count.to_string().as_str());
    push(input.content_hash);
    push(
        draft
            .duration_ms
            .map_or_else(alloc::string::String::new, |raw| raw.to_string())
            .as_str(),
    );
    push(
        draft
            .width_px
            .map_or_else(alloc::string::String::new, |raw| raw.to_string())
            .as_str(),
    );
    push(
        draft
            .height_px
            .map_or_else(alloc::string::String::new, |raw| raw.to_string())
            .as_str(),
    );
    push(
        draft
            .frame_rate_millihz
            .map_or_else(alloc::string::String::new, |raw| raw.to_string())
            .as_str(),
    );
    push(input.service_class);
    push(input.provider_profile);
    push(input.redaction_marker);
    for loss in draft.loss_classes {
        push(loss.as_str());
    }
    for item in input.non_claims {
        push(item.as_str());
    }
    for item in input.fragments {
        push(item.as_str());
    }
    blake3::hash(value.as_bytes()).to_hex().to_string()
}
