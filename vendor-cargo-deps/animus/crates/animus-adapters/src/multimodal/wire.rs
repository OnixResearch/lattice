/// Provider request serialization mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderSerializationMode {
    /// Exact reviewed provider-native fixture serialization.
    NativeFixture,
    /// Generic conversion, rejected for image or video parts.
    GenericJson,
}

/// Support state for one exact provider projection fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderProjectionSupport {
    /// The exact reviewed profile represents the fact.
    Supported,
    /// The exact reviewed profile does not represent the fact.
    Unsupported,
}

impl ProviderProjectionSupport {
    const fn is_supported(self) -> bool {
        matches!(self, Self::Supported)
    }
}

/// Exact reviewed video projection facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderVideoProjection {
    /// Declared dimensions, duration, frame rate, codec, and container support.
    pub declared_metadata: ProviderProjectionSupport,
    /// Audio-presence intent support.
    pub audio_marker: ProviderProjectionSupport,
    /// Detail-preference support.
    pub detail: ProviderProjectionSupport,
    /// Mixed-part ordering support.
    pub ordering: ProviderProjectionSupport,
}

/// Borrowed provider serialization policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderSerializationPolicyDraft<'a> {
    /// Exact safe provider profile label.
    pub provider_label: &'a str,
    /// Serialization mode.
    pub mode: ProviderSerializationMode,
    /// Reviewed video projection facts; absence means video is unsupported.
    pub video_projection: Option<ProviderVideoProjection>,
}

/// Validated provider serialization policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSerializationPolicy {
    provider_label: alloc::string::String,
    mode: ProviderSerializationMode,
    video_projection: Option<ProviderVideoProjection>,
}

impl ProviderSerializationPolicy {
    /// Validates provider serialization policy.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the provider profile label is unsafe.
    pub fn from_draft(draft: ProviderSerializationPolicyDraft<'_>) -> Result<Self, super::Error> {
        Ok(Self {
            provider_label: super::check::normalized_safe_text(
                super::PROVIDER_LABEL_FIELD,
                draft.provider_label,
            )?,
            mode: draft.mode,
            video_projection: draft.video_projection,
        })
    }
    /// Returns exact provider profile label.
    #[must_use]
    pub fn provider_label(&self) -> &str {
        self.provider_label.as_str()
    }
    /// Returns serialization mode.
    #[must_use]
    pub const fn mode(&self) -> ProviderSerializationMode {
        self.mode
    }
    /// Returns reviewed video projection facts.
    #[must_use]
    pub const fn video_projection(&self) -> Option<ProviderVideoProjection> {
        self.video_projection
    }
}

/// Provider-native request fixture part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderNativePart {
    /// Native text request part.
    Text { text: alloc::string::String },
    /// Native image request part represented by a safe provider handle.
    Image {
        media_type: super::ImageMediaType,
        handle_kind: super::ImagePayloadHandleKind,
        safe_handle: alloc::string::String,
        content_hash: alloc::string::String,
        detail: Option<super::ImageDetailPreference>,
    },
    /// Native video request part represented by safe provider-handle metadata.
    Video {
        media_type: super::VideoMediaType,
        handle_kind: super::VideoPayloadHandleKind,
        safe_handle: alloc::string::String,
        content_hash: alloc::string::String,
        detail: Option<super::VideoDetailPreference>,
    },
}

/// Provider-native request fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderNativeRequest {
    provider_label: alloc::string::String,
    parts: alloc::vec::Vec<ProviderNativePart>,
}

impl ProviderNativeRequest {
    /// Returns exact provider profile label.
    #[must_use]
    pub fn provider_label(&self) -> &str {
        self.provider_label.as_str()
    }
    /// Returns provider-native parts.
    #[must_use]
    pub fn parts(&self) -> &[ProviderNativePart] {
        self.parts.as_slice()
    }
}

/// Serializes text and image fixtures through the compatibility image-service seam.
///
/// # Errors
///
/// Returns [`super::Error`] when the assembly contains video or unsafe/lossy image projection.
pub fn serialize_provider_native_fixture(
    assembly: &super::PromptAssembly,
    policy: &ProviderSerializationPolicy,
    image_service: Option<&dyn super::ImagePayloadService>,
) -> Result<ProviderNativeRequest, super::Error> {
    serialize_provider_native_video_fixture(assembly, policy, image_service, None)
}

/// Serializes an exact provider-native fixture through injected image and video services.
///
/// # Errors
///
/// Returns [`super::Error`] for generic conversion, missing services, live calls, unsafe handles, or visible projection loss.
pub fn serialize_provider_native_video_fixture(
    assembly: &super::PromptAssembly,
    policy: &ProviderSerializationPolicy,
    image_service: Option<&dyn super::ImagePayloadService>,
    video_service: Option<&dyn super::VideoPayloadService>,
) -> Result<ProviderNativeRequest, super::Error> {
    reject_lossy_or_missing_services(assembly, policy, image_service, video_service)?;
    let mut parts = alloc::vec::Vec::with_capacity(assembly.parts().len());
    for part in assembly.parts() {
        parts.push(native_part(part, policy, image_service, video_service)?);
    }
    assert_eq!(parts.len(), assembly.parts().len());
    Ok(ProviderNativeRequest {
        provider_label: alloc::string::String::from(policy.provider_label()),
        parts,
    })
}

fn reject_lossy_or_missing_services(
    assembly: &super::PromptAssembly,
    policy: &ProviderSerializationPolicy,
    image_service: Option<&dyn super::ImagePayloadService>,
    video_service: Option<&dyn super::VideoPayloadService>,
) -> Result<(), super::Error> {
    assert!(!policy.provider_label().is_empty());
    assert!(assembly.parts().len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS);
    if (assembly.has_images() || assembly.has_videos())
        && policy.mode() == ProviderSerializationMode::GenericJson
    {
        return Err(super::Error::LossyGenericConversion);
    }
    if assembly.has_images() && image_service.is_none() {
        return Err(super::Error::MissingImageService);
    }
    if assembly.has_videos() && video_service.is_none() {
        return Err(super::Error::MissingVideoService);
    }
    if assembly.has_videos() && policy.video_projection().is_none() {
        return Err(super::Error::UnsupportedModality {
            model_label: alloc::string::String::from(policy.provider_label()),
            modality: super::InputModality::Video,
        });
    }
    Ok(())
}

fn native_part(
    part: &super::ContentPart,
    policy: &ProviderSerializationPolicy,
    image_service: Option<&dyn super::ImagePayloadService>,
    video_service: Option<&dyn super::VideoPayloadService>,
) -> Result<ProviderNativePart, super::Error> {
    match part {
        super::ContentPart::Text(text) => Ok(ProviderNativePart::Text {
            text: alloc::string::String::from(text.text()),
        }),
        super::ContentPart::Image(image) => native_image_part(image_service, image),
        super::ContentPart::Video(video) => native_video_part(video_service, policy, video),
    }
}

fn native_image_part(
    service: Option<&dyn super::ImagePayloadService>,
    image: &super::ImagePart,
) -> Result<ProviderNativePart, super::Error> {
    let handle = resolve_image_handle(service, image)?;
    ensure_image_handle_matches(&handle, image)?;
    Ok(ProviderNativePart::Image {
        media_type: image.media_type(),
        handle_kind: handle.kind(),
        safe_handle: alloc::string::String::from(handle.safe_handle()),
        content_hash: alloc::string::String::from(handle.content_hash()),
        detail: image.metadata().and_then(super::ImageMetadata::detail),
    })
}

fn native_video_part(
    service: Option<&dyn super::VideoPayloadService>,
    policy: &ProviderSerializationPolicy,
    video: &super::VideoPart,
) -> Result<ProviderNativePart, super::Error> {
    reject_video_projection_loss(policy, video)?;
    let handle = resolve_video_handle(service, video)?;
    ensure_video_handle_matches(&handle, video)?;
    Ok(ProviderNativePart::Video {
        media_type: video.media_type(),
        handle_kind: handle.kind(),
        safe_handle: alloc::string::String::from(handle.safe_handle()),
        content_hash: alloc::string::String::from(handle.content_hash()),
        detail: video.metadata().and_then(super::VideoMetadata::detail),
    })
}

fn reject_video_projection_loss(
    policy: &ProviderSerializationPolicy,
    video: &super::VideoPart,
) -> Result<(), super::Error> {
    assert!(video.byte_count() > 0);
    assert_eq!(video.content_hash().len(), super::BLAKE3_HEX_CHARS);
    let Some(projection) = policy.video_projection() else {
        return Err(super::Error::UnsupportedModality {
            model_label: alloc::string::String::from(policy.provider_label()),
            modality: super::InputModality::Video,
        });
    };
    if !projection.ordering.is_supported() {
        return Err(super::Error::UnrepresentableVideoField { field: "ordering" });
    }
    let Some(metadata) = video.metadata() else {
        return Ok(());
    };
    let has_declared_metadata = metadata.duration_ms().is_some()
        || metadata.width_px().is_some()
        || metadata.height_px().is_some()
        || metadata.frame_rate_millihz().is_some()
        || metadata.codec().is_some()
        || metadata.container().is_some();
    if has_declared_metadata && !projection.declared_metadata.is_supported() {
        return Err(super::Error::UnrepresentableVideoField {
            field: "declared_metadata",
        });
    }
    if metadata.has_audio().is_some() && !projection.audio_marker.is_supported() {
        return Err(super::Error::UnrepresentableVideoField {
            field: "audio_presence",
        });
    }
    if metadata.detail().is_some() && !projection.detail.is_supported() {
        return Err(super::Error::UnrepresentableVideoField {
            field: "detail_preference",
        });
    }
    Ok(())
}

fn resolve_image_handle(
    service: Option<&dyn super::ImagePayloadService>,
    image: &super::ImagePart,
) -> Result<super::ImagePayloadHandle, super::Error> {
    let Some(service) = service else {
        return Err(super::Error::MissingImageService);
    };
    service
        .resolve_image_payload(image)
        .map_err(image_payload_error)
}

fn resolve_video_handle(
    service: Option<&dyn super::VideoPayloadService>,
    video: &super::VideoPart,
) -> Result<super::VideoPayloadHandle, super::Error> {
    let Some(service) = service else {
        return Err(super::Error::MissingVideoService);
    };
    service
        .resolve_video_payload(video)
        .map_err(video_payload_error)
}

fn image_payload_error(error: super::ImagePayloadError) -> super::Error {
    match error {
        super::ImagePayloadError::Unavailable => super::Error::AdapterUnavailable {
            label: alloc::string::String::from("image_payload_service"),
        },
        super::ImagePayloadError::LiveProviderCallRequired => {
            super::Error::LiveProviderCallRequired
        }
        super::ImagePayloadError::InvalidHandle { field, reason } => {
            super::Error::InvalidImageHandle { field, reason }
        }
    }
}

fn video_payload_error(error: super::VideoPayloadError) -> super::Error {
    match error {
        super::VideoPayloadError::Unavailable => super::Error::AdapterUnavailable {
            label: alloc::string::String::from("video_payload_service"),
        },
        super::VideoPayloadError::LiveProviderCallRequired => {
            super::Error::LiveProviderCallRequired
        }
        super::VideoPayloadError::UploadTooLarge => super::Error::VideoUploadTooLarge,
        super::VideoPayloadError::InvalidHandle { field, reason } => {
            super::Error::InvalidVideoHandle { field, reason }
        }
    }
}

fn ensure_image_handle_matches(
    handle: &super::ImagePayloadHandle,
    image: &super::ImagePart,
) -> Result<(), super::Error> {
    if handle.content_hash() != image.content_hash() {
        return Err(super::Error::MismatchedImageHandle {
            field: super::CONTENT_HASH_FIELD.as_str(),
        });
    }
    if handle.media_type() != image.media_type() {
        return Err(super::Error::MismatchedImageHandle {
            field: super::MEDIA_TYPE_FIELD.as_str(),
        });
    }
    Ok(())
}

fn ensure_video_handle_matches(
    handle: &super::VideoPayloadHandle,
    video: &super::VideoPart,
) -> Result<(), super::Error> {
    if handle.content_hash() != video.content_hash() {
        return Err(super::Error::MismatchedVideoHandle {
            field: "video_content_hash",
        });
    }
    if handle.media_type() != video.media_type() {
        return Err(super::Error::MismatchedVideoHandle {
            field: "video_media_type",
        });
    }
    Ok(())
}
