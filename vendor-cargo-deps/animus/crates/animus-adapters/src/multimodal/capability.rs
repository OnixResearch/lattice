/// Borrowed selected model capability metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelInputCapabilitiesDraft<'a> {
    /// Safe model label.
    pub model_label: &'a str,
    /// Whether text input is supported.
    pub supports_text: bool,
    /// Whether image input is supported.
    pub supports_images: bool,
    /// Whether an image-capable provider adapter is available.
    pub image_adapter_available: bool,
}

/// Validated selected model capability metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInputCapabilities {
    model_label: alloc::string::String,
    supports_text: bool,
    supports_images: bool,
    image_adapter_available: bool,
}

impl ModelInputCapabilities {
    /// Validates selected model capability metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the model label is unsafe.
    pub fn from_draft(draft: ModelInputCapabilitiesDraft<'_>) -> Result<Self, super::Error> {
        Ok(Self {
            model_label: super::check::normalized_safe_text(
                super::MODEL_LABEL_FIELD,
                draft.model_label,
            )?,
            supports_text: draft.supports_text,
            supports_images: draft.supports_images,
            image_adapter_available: draft.image_adapter_available,
        })
    }

    /// Returns the safe selected model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }

    /// Returns whether the selected model supports the modality.
    #[must_use]
    pub const fn supports(&self, modality: super::InputModality) -> bool {
        match modality {
            super::InputModality::Text => self.supports_text,
            super::InputModality::Image => self.supports_images,
        }
    }

    /// Returns whether an image adapter is available.
    #[must_use]
    pub const fn image_adapter_available(&self) -> bool {
        self.image_adapter_available
    }
}

/// Dispatch plan after provider/model capability gates pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchPlan {
    model_label: alloc::string::String,
    part_count: u64,
    image_count: u64,
}

impl DispatchPlan {
    /// Returns the safe model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }

    /// Returns the included part count.
    #[must_use]
    pub const fn part_count(&self) -> u64 {
        self.part_count
    }

    /// Returns the included image count.
    #[must_use]
    pub const fn image_count(&self) -> u64 {
        self.image_count
    }
}

/// Prepares dispatch only when selected model capabilities support all included modalities.
///
/// # Errors
///
/// Returns [`super::Error`] when text or image input is unsupported or the image adapter is unavailable.
pub fn prepare_dispatch(
    assembly: &super::PromptAssembly,
    capabilities: &ModelInputCapabilities,
) -> Result<DispatchPlan, super::Error> {
    let part_count = super::check::usize_to_u64(assembly.parts().len());
    let image_count = assembly.image_count();
    assert!(image_count <= part_count);
    assert!(part_count <= super::check::usize_to_u64(crate::validation::MAX_ADAPTER_FIELD_CHARS));
    for part in assembly.parts() {
        let modality = part.modality();
        if !capabilities.supports(modality) {
            return Err(super::Error::UnsupportedModality {
                model_label: alloc::string::String::from(capabilities.model_label()),
                modality,
            });
        }
    }
    reject_missing_image_adapter(assembly, capabilities)?;
    Ok(DispatchPlan {
        model_label: alloc::string::String::from(capabilities.model_label()),
        part_count,
        image_count,
    })
}

fn reject_missing_image_adapter(
    assembly: &super::PromptAssembly,
    capabilities: &ModelInputCapabilities,
) -> Result<(), super::Error> {
    if !assembly.has_images() {
        return Ok(());
    }
    if capabilities.image_adapter_available() {
        return Ok(());
    }
    Err(super::Error::AdapterUnavailable {
        label: alloc::string::String::from(capabilities.model_label()),
    })
}
