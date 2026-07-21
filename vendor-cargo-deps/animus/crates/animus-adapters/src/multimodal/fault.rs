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
    /// Optional dimensions were invalid.
    InvalidDimensions {
        /// Invalid width in pixels.
        width_px: Option<u32>,
        /// Invalid height in pixels.
        height_px: Option<u32>,
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
    /// Image request serialization tried to use a lossy generic conversion.
    LossyGenericConversion,
    /// Image request serialization required a live provider call.
    LiveProviderCallRequired,
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
}
