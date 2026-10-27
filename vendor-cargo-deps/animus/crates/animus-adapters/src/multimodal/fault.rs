/// Multimodal adapter-boundary validation failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A reused adapter-boundary field failed generic text validation.
    AdapterField {
        /// Inner adapter contract error.
        source: crate::AdapterContractError,
    },
    /// A field contained unsafe descriptor or receipt material.
    UnsafeDescriptor {
        /// Field name.
        field: &'static str,
        /// Leak or unsafe material class.
        reason: UnsafeContentReason,
    },
    /// The image media type is not in the supported input set.
    UnsupportedMediaType {
        /// Unsupported media type value after normalization.
        value: alloc::string::String,
    },
    /// The video media type is not in the admitted set.
    UnsupportedVideoMediaType {
        /// Unsupported normalized value.
        value: alloc::string::String,
    },
    /// A BLAKE3 content identity was malformed.
    InvalidContentHash {
        /// Field name.
        field: &'static str,
    },
    /// Required byte-count metadata was missing or zero.
    MissingByteCount,
    /// The image descriptor exceeded an accepted byte budget.
    ImageTooLarge {
        /// Maximum accepted bytes for the current boundary.
        max_bytes: u64,
        /// Actual descriptor byte count.
        actual_bytes: u64,
    },
    /// The video descriptor exceeded an accepted byte budget.
    VideoTooLarge {
        /// Maximum accepted bytes.
        max_bytes: u64,
        /// Actual byte count.
        actual_bytes: u64,
    },
    /// Optional dimensions were invalid.
    InvalidDimensions {
        /// Invalid width in pixels.
        width_px: Option<u32>,
        /// Invalid height in pixels.
        height_px: Option<u32>,
    },
    /// Declared video duration was invalid.
    InvalidVideoDuration {
        /// Invalid duration.
        duration_ms: u64,
    },
    /// Declared video frame rate was invalid.
    InvalidVideoFrameRate {
        /// Invalid milli-hertz value.
        frame_rate_millihz: u32,
    },
    /// Prompt media policy was internally invalid.
    InvalidMediaPolicy,
    /// Two video drafts used the same content identity.
    DuplicateVideoIdentity {
        /// Duplicate BLAKE3 identity.
        content_hash: alloc::string::String,
    },
    /// No content parts were supplied for prompt assembly.
    EmptyParts,
    /// A requested input modality was not supported by the selected model.
    UnsupportedModality {
        /// Safe selected model label.
        model_label: alloc::string::String,
        /// Rejected modality.
        modality: super::InputModality,
    },
    /// The selected adapter or image service is unavailable.
    AdapterUnavailable {
        /// Safe selected provider or model label.
        label: alloc::string::String,
    },
    /// Image parts required an injected image payload service, but none was supplied.
    MissingImageService,
    /// Video parts required an injected video payload service, but none was supplied.
    MissingVideoService,
    /// Image request serialization tried to use a lossy generic conversion.
    LossyGenericConversion,
    /// Image or video fixture serialization required a live provider call.
    LiveProviderCallRequired,
    /// Host reported an expired video handle.
    ExpiredVideoHandle,
    /// Provider projection cannot represent admitted video intent.
    UnrepresentableVideoField {
        /// Exact field or ordering class.
        field: &'static str,
    },
    /// Injected video service rejected an oversized upload.
    VideoUploadTooLarge,
    /// The injected image service returned an invalid safe handle.
    InvalidImageHandle {
        /// Field name.
        field: &'static str,
        /// Leak or mismatch class.
        reason: UnsafeContentReason,
    },
    /// The injected image service returned a handle for a different image descriptor.
    MismatchedImageHandle {
        /// Field name that differed.
        field: &'static str,
    },
    /// The injected video service returned an invalid safe handle.
    InvalidVideoHandle {
        /// Field name.
        field: &'static str,
        /// Leak or mismatch class.
        reason: UnsafeContentReason,
    },
    /// The injected video service returned a handle for different media.
    MismatchedVideoHandle {
        /// Field name that differed.
        field: &'static str,
    },
}

impl From<crate::AdapterContractError> for Error {
    fn from(source: crate::AdapterContractError) -> Self {
        Self::AdapterField { source }
    }
}

/// Unsafe descriptor or receipt material class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsafeContentReason {
    /// Raw byte material was present.
    RawBytes,
    /// Base64 payload material was present.
    Base64Payload,
    /// A data URL was present.
    DataUrl,
    /// A private absolute path was present.
    AbsolutePath,
    /// A credential-looking value was present.
    Credential,
    /// A provider-native payload body was present.
    ProviderPayload,
    /// Unbounded payload material was present.
    UnboundedPayload,
    /// A remote URL was present where only an approved handle is allowed.
    RemoteUrl,
    /// Frame or audio body material was present.
    MediaBody,
}
