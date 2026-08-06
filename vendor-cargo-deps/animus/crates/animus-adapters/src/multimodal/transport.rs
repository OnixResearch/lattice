/// Provider-native image handle kind supplied by an injected host image service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImagePayloadHandleKind {
    /// Provider-native file identifier or upload handle.
    ProviderFileId,
    /// Approved remote handle label owned by the provider adapter.
    ApprovedRemoteHandle,
    /// Ephemeral host-upload token label.
    EphemeralUploadToken,
}

impl ImagePayloadHandleKind {
    /// Returns the stable handle-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderFileId => "provider_file_id",
            Self::ApprovedRemoteHandle => "approved_remote_handle",
            Self::EphemeralUploadToken => "ephemeral_upload_token",
        }
    }
}

/// Borrowed image payload handle returned by an injected image service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImagePayloadHandleDraft<'a> {
    /// Provider-native safe handle kind.
    pub kind: ImagePayloadHandleKind,
    /// Safe handle label; raw bytes, URLs, base64, and credentials are rejected.
    pub safe_handle: &'a str,
    /// BLAKE3 content hash for the image represented by this handle.
    pub content_hash: &'a str,
    /// Media type for the image represented by this handle.
    pub media_type: &'a str,
}

/// Validated image payload handle returned by an injected image service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImagePayloadHandle {
    kind: ImagePayloadHandleKind,
    safe_handle: alloc::string::String,
    content_hash: alloc::string::String,
    media_type: super::ImageMediaType,
}

impl ImagePayloadHandle {
    /// Validates an injected image-service handle.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the handle is unsafe or points to malformed metadata.
    pub fn from_draft(draft: ImagePayloadHandleDraft<'_>) -> Result<Self, super::Error> {
        Ok(Self {
            kind: draft.kind,
            safe_handle: safe_handle_text(draft.safe_handle)?,
            content_hash: super::check::validate_content_hash(draft.content_hash)?,
            media_type: super::ImageMediaType::parse(draft.media_type)?,
        })
    }

    /// Returns the handle kind.
    #[must_use]
    pub const fn kind(&self) -> ImagePayloadHandleKind {
        self.kind
    }

    /// Returns the safe provider-native handle label.
    #[must_use]
    pub fn safe_handle(&self) -> &str {
        self.safe_handle.as_str()
    }

    /// Returns the BLAKE3 content hash for the represented image.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }

    /// Returns the media type for the represented image.
    #[must_use]
    pub const fn media_type(&self) -> super::ImageMediaType {
        self.media_type
    }
}

/// Error returned by a host-injected image payload service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImagePayloadError {
    /// Service was unavailable for this image.
    Unavailable,
    /// Service refused to return a provider-native fixture without a live provider call.
    LiveProviderCallRequired,
    /// Service returned invalid safe metadata.
    InvalidHandle {
        /// Field name.
        field: &'static str,
        /// Unsafe material class.
        reason: super::UnsafeContentReason,
    },
}

/// Host-injected image payload service boundary.
pub trait ImagePayloadService {
    /// Resolves an approved image descriptor to a provider-native safe handle.
    ///
    /// # Errors
    ///
    /// Returns [`ImagePayloadError`] when the service is unavailable, would require a live call, or
    /// cannot return safe metadata.
    fn resolve_image_payload(&self, image: &super::ImagePart) -> Result<ImagePayloadHandle, ImagePayloadError>;
}

fn safe_handle_text(raw: &str) -> Result<alloc::string::String, super::Error> {
    super::check::normalized_safe_text(super::HANDLE_FIELD, raw).map_err(|error| match error {
        super::Error::UnsafeDescriptor { reason, .. } => super::Error::InvalidImageHandle {
            field: super::HANDLE_FIELD.as_str(),
            reason,
        },
        other => other,
    })
}
