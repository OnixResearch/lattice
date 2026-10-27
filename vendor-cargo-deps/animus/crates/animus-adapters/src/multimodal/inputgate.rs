/// Borrowed exact video capability facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VideoInputCapabilitiesDraft<'a> {
    /// Whether video input is independently supported.
    pub supported: bool,
    /// Whether an injected video payload service is available.
    pub service_available: bool,
    /// Whether an exact reviewed provider projection is available.
    pub projection_available: bool,
    /// Admitted video media types.
    pub media_types: &'a [super::VideoMediaType],
    /// Provider/model video byte limit.
    pub byte_limit: u64,
    /// Provider/model declared duration limit.
    pub duration_limit_ms: u64,
}

/// Validated exact video capability facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoInputCapabilities {
    supported: bool,
    service_available: bool,
    projection_available: bool,
    media_types: alloc::vec::Vec<super::VideoMediaType>,
    byte_limit: u64,
    duration_limit_ms: u64,
}

impl VideoInputCapabilities {
    fn from_draft(draft: VideoInputCapabilitiesDraft<'_>) -> Result<Self, super::Error> {
        if draft.supported
            && (draft.media_types.is_empty()
                || draft.byte_limit == 0
                || draft.byte_limit > super::MAX_VIDEO_BYTE_COUNT
                || draft.duration_limit_ms == 0
                || draft.duration_limit_ms > super::MAX_VIDEO_DURATION_MS)
        {
            return Err(super::Error::InvalidMediaPolicy);
        }
        Ok(Self {
            supported: draft.supported,
            service_available: draft.service_available,
            projection_available: draft.projection_available,
            media_types: draft.media_types.to_vec(),
            byte_limit: draft.byte_limit,
            duration_limit_ms: draft.duration_limit_ms,
        })
    }

    /// Returns whether the exact video class is supported.
    #[must_use]
    pub const fn supported(&self) -> bool {
        self.supported
    }
    /// Returns payload-service availability.
    #[must_use]
    pub const fn service_available(&self) -> bool {
        self.service_available
    }
    /// Returns provider-projection availability.
    #[must_use]
    pub const fn projection_available(&self) -> bool {
        self.projection_available
    }
    /// Returns whether the exact descriptor fits the selected profile.
    #[must_use]
    pub fn admits(&self, video: &super::VideoPart) -> bool {
        if !self.supported {
            return false;
        }
        if !self.media_types.contains(&video.media_type()) {
            return false;
        }
        if video.byte_count() > self.byte_limit {
            return false;
        }
        let duration_ms = video.metadata().and_then(super::VideoMetadata::duration_ms);
        duration_ms.is_none_or(|value_ms| value_ms <= self.duration_limit_ms)
    }
}

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
    /// Exact video capability facts.
    pub video: VideoInputCapabilitiesDraft<'a>,
}

/// Validated selected model capability metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInputCapabilities {
    model_label: alloc::string::String,
    supports_text: bool,
    supports_images: bool,
    image_adapter_available: bool,
    video: VideoInputCapabilities,
}

impl ModelInputCapabilities {
    /// Validates selected model capability metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when labels or video limits are invalid.
    pub fn from_draft(draft: ModelInputCapabilitiesDraft<'_>) -> Result<Self, super::Error> {
        Ok(Self {
            model_label: super::check::normalized_safe_text(
                super::MODEL_LABEL_FIELD,
                draft.model_label,
            )?,
            supports_text: draft.supports_text,
            supports_images: draft.supports_images,
            image_adapter_available: draft.image_adapter_available,
            video: VideoInputCapabilities::from_draft(draft.video)?,
        })
    }

    /// Returns selected model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }
    /// Returns whether a modality is independently supported.
    #[must_use]
    pub const fn supports(&self, modality: super::InputModality) -> bool {
        match modality {
            super::InputModality::Text => self.supports_text,
            super::InputModality::Image => self.supports_images,
            super::InputModality::Video => self.video.supported(),
        }
    }
    /// Returns image adapter availability.
    #[must_use]
    pub const fn image_adapter_available(&self) -> bool {
        self.image_adapter_available
    }
    /// Returns exact video capability facts.
    #[must_use]
    pub const fn video(&self) -> &VideoInputCapabilities {
        &self.video
    }
}

/// Explicit policy for unsupported included video.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoCapabilityPolicy {
    /// Every included video is required.
    Required,
    /// A caller-reviewed policy can omit unsupported videos visibly.
    OptionalOmissionAllowed,
}

/// Visible video capability result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoCapabilityOutcome {
    /// No included video was present.
    NotRequested,
    /// All included videos are admitted.
    Supported,
    /// Explicit policy omitted unsupported video.
    OmittedByPolicy,
}

/// Dispatch plan after provider/model capability gates pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchPlan {
    model_label: alloc::string::String,
    part_count: u64,
    image_count: u64,
    video_count: u64,
    video_outcome: VideoCapabilityOutcome,
}

impl DispatchPlan {
    /// Returns model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }
    /// Returns admitted part count.
    #[must_use]
    pub const fn part_count(&self) -> u64 {
        self.part_count
    }
    /// Returns included image count.
    #[must_use]
    pub const fn image_count(&self) -> u64 {
        self.image_count
    }
    /// Returns included video count after policy.
    #[must_use]
    pub const fn video_count(&self) -> u64 {
        self.video_count
    }
    /// Returns visible video capability result.
    #[must_use]
    pub const fn video_outcome(&self) -> VideoCapabilityOutcome {
        self.video_outcome
    }
}

/// Prepares dispatch with required-video policy.
///
/// # Errors
///
/// Returns [`super::Error`] when any included modality or service is unsupported.
pub fn prepare_dispatch(
    assembly: &super::PromptAssembly,
    capabilities: &ModelInputCapabilities,
) -> Result<DispatchPlan, super::Error> {
    prepare_dispatch_with_video_policy(assembly, capabilities, VideoCapabilityPolicy::Required)
}

/// Prepares dispatch under explicit unsupported-video policy.
///
/// # Errors
///
/// Returns [`super::Error`] for unsupported required media or unavailable services.
pub fn prepare_dispatch_with_video_policy(
    assembly: &super::PromptAssembly,
    capabilities: &ModelInputCapabilities,
    video_policy: VideoCapabilityPolicy,
) -> Result<DispatchPlan, super::Error> {
    reject_unsupported_text_or_image(assembly, capabilities)?;
    reject_missing_image_adapter(assembly, capabilities)?;
    let (video_count, omitted_video_count, video_outcome) =
        plan_video_capability(assembly, capabilities, video_policy)?;
    let all_part_count = super::check::usize_to_u64(assembly.parts().len());
    let part_count = all_part_count.saturating_sub(omitted_video_count);
    let image_count = assembly.image_count();
    assert!(image_count.saturating_add(video_count) <= part_count);
    Ok(DispatchPlan {
        model_label: alloc::string::String::from(capabilities.model_label()),
        part_count,
        image_count,
        video_count,
        video_outcome,
    })
}

fn reject_unsupported_text_or_image(
    assembly: &super::PromptAssembly,
    capabilities: &ModelInputCapabilities,
) -> Result<(), super::Error> {
    for part in assembly.parts() {
        let modality = part.modality();
        if modality == super::InputModality::Video {
            continue;
        }
        if !capabilities.supports(modality) {
            return Err(super::Error::UnsupportedModality {
                model_label: alloc::string::String::from(capabilities.model_label()),
                modality,
            });
        }
    }
    Ok(())
}

fn reject_missing_image_adapter(
    assembly: &super::PromptAssembly,
    capabilities: &ModelInputCapabilities,
) -> Result<(), super::Error> {
    if !assembly.has_images() || capabilities.image_adapter_available() {
        return Ok(());
    }
    Err(super::Error::AdapterUnavailable {
        label: alloc::string::String::from(capabilities.model_label()),
    })
}

fn plan_video_capability(
    assembly: &super::PromptAssembly,
    capabilities: &ModelInputCapabilities,
    policy: VideoCapabilityPolicy,
) -> Result<(u64, u64, VideoCapabilityOutcome), super::Error> {
    if !assembly.has_videos() {
        return Ok((0, 0, VideoCapabilityOutcome::NotRequested));
    }
    assert!(assembly.video_count() <= super::check::usize_to_u64(assembly.parts().len()));
    assert!(!capabilities.model_label().is_empty());
    let video = capabilities.video();
    let is_every_part_admitted = assembly
        .parts()
        .iter()
        .filter_map(super::ContentPart::as_video)
        .all(|part| video.admits(part));
    if is_every_part_admitted && video.service_available() && video.projection_available() {
        return Ok((assembly.video_count(), 0, VideoCapabilityOutcome::Supported));
    }
    if policy == VideoCapabilityPolicy::OptionalOmissionAllowed {
        return Ok((
            0,
            assembly.video_count(),
            VideoCapabilityOutcome::OmittedByPolicy,
        ));
    }
    Err(super::Error::UnsupportedModality {
        model_label: alloc::string::String::from(capabilities.model_label()),
        modality: super::InputModality::Video,
    })
}
