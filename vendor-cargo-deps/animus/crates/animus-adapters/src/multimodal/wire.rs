/// Provider request serialization mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderSerializationMode {
    /// Provider-native fixture serialization only.
    NativeFixture,
    /// Lossy generic JSON conversion, rejected when image parts are present.
    GenericJson,
}

/// Borrowed provider serialization policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderSerializationPolicyDraft<'a> {
    /// Safe provider label.
    pub provider_label: &'a str,
    /// Serialization mode.
    pub mode: ProviderSerializationMode,
}

/// Validated provider serialization policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSerializationPolicy {
    provider_label: alloc::string::String,
    mode: ProviderSerializationMode,
}

impl ProviderSerializationPolicy {
    /// Validates provider serialization policy.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the provider label is unsafe.
    pub fn from_draft(draft: ProviderSerializationPolicyDraft<'_>) -> Result<Self, super::Error> {
        Ok(Self {
            provider_label: super::check::normalized_safe_text(super::PROVIDER_LABEL_FIELD, draft.provider_label)?,
            mode: draft.mode,
        })
    }

    /// Returns the safe provider label.
    #[must_use]
    pub fn provider_label(&self) -> &str {
        self.provider_label.as_str()
    }

    /// Returns serialization mode.
    #[must_use]
    pub const fn mode(&self) -> ProviderSerializationMode {
        self.mode
    }
}

/// Provider-native request fixture part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderNativePart {
    /// Native text request part.
    Text {
        /// Text body.
        text: alloc::string::String,
    },
    /// Native image request part represented by a safe provider handle.
    Image {
        /// Image media type.
        media_type: super::ImageMediaType,
        /// Handle kind.
        handle_kind: super::ImagePayloadHandleKind,
        /// Safe provider-native handle label.
        safe_handle: alloc::string::String,
        /// BLAKE3 content identity.
        content_hash: alloc::string::String,
        /// Optional detail preference.
        detail: Option<super::ImageDetailPreference>,
    },
}

/// Provider-native request fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderNativeRequest {
    provider_label: alloc::string::String,
    parts: alloc::vec::Vec<ProviderNativePart>,
}

impl ProviderNativeRequest {
    /// Returns the safe provider label.
    #[must_use]
    pub fn provider_label(&self) -> &str {
        self.provider_label.as_str()
    }

    /// Returns provider-native request parts.
    #[must_use]
    pub fn parts(&self) -> &[ProviderNativePart] {
        self.parts.as_slice()
    }
}

/// Serializes an assembled request into a provider-native fixture using an injected image service.
///
/// # Errors
///
/// Returns [`super::Error`] when image support would require a generic conversion, missing service,
/// live provider call, or unsafe handle.
pub fn serialize_provider_native_fixture(
    assembly: &super::PromptAssembly,
    policy: &ProviderSerializationPolicy,
    service: Option<&dyn super::ImagePayloadService>,
) -> Result<ProviderNativeRequest, super::Error> {
    let has_images = assembly.has_images();
    debug_assert!(u32::try_from(super::check::usize_to_u64(assembly.parts().len())).is_ok());
    debug_assert!(assembly.image_count() <= super::check::usize_to_u64(assembly.parts().len()));
    reject_lossy_or_missing_image_service(has_images, policy, service)?;
    let mut parts = alloc::vec::Vec::with_capacity(assembly.parts().len());
    for part in assembly.parts() {
        parts.push(native_part(part, service)?);
    }
    Ok(ProviderNativeRequest {
        provider_label: alloc::string::String::from(policy.provider_label()),
        parts,
    })
}

fn reject_lossy_or_missing_image_service(
    has_images: bool,
    policy: &ProviderSerializationPolicy,
    service: Option<&dyn super::ImagePayloadService>,
) -> Result<(), super::Error> {
    if !has_images {
        return Ok(());
    }
    if policy.mode() == ProviderSerializationMode::GenericJson {
        return Err(super::Error::LossyGenericConversion);
    }
    if service.is_none() {
        return Err(super::Error::MissingImageService);
    }
    Ok(())
}

fn native_part(
    part: &super::ContentPart,
    service: Option<&dyn super::ImagePayloadService>,
) -> Result<ProviderNativePart, super::Error> {
    match part {
        super::ContentPart::Text(text) => Ok(ProviderNativePart::Text {
            text: alloc::string::String::from(text.text()),
        }),
        super::ContentPart::Image(image) => native_image_part(service, image),
    }
}

fn native_image_part(
    service: Option<&dyn super::ImagePayloadService>,
    image: &super::ImagePart,
) -> Result<ProviderNativePart, super::Error> {
    let handle = resolve_handle(service, image)?;
    ensure_handle_matches_image(&handle, image)?;
    Ok(ProviderNativePart::Image {
        media_type: image.media_type(),
        handle_kind: handle.kind(),
        safe_handle: alloc::string::String::from(handle.safe_handle()),
        content_hash: alloc::string::String::from(handle.content_hash()),
        detail: image.metadata().and_then(super::ImageMetadata::detail),
    })
}

fn resolve_handle(
    service: Option<&dyn super::ImagePayloadService>,
    image: &super::ImagePart,
) -> Result<super::ImagePayloadHandle, super::Error> {
    let Some(service) = service else {
        return Err(super::Error::MissingImageService);
    };
    service.resolve_image_payload(image).map_err(image_payload_error)
}

fn image_payload_error(error: super::ImagePayloadError) -> super::Error {
    match error {
        super::ImagePayloadError::Unavailable => super::Error::AdapterUnavailable {
            label: alloc::string::String::from("image_payload_service"),
        },
        super::ImagePayloadError::LiveProviderCallRequired => super::Error::LiveProviderCallRequired,
        super::ImagePayloadError::InvalidHandle { field, reason } => super::Error::InvalidImageHandle { field, reason },
    }
}

fn ensure_handle_matches_image(
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
