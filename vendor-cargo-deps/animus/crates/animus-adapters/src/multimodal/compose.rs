/// Prompt assembly outcome for an image part.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageOutcomeStatus {
    /// Image part was included in model-visible content.
    Included,
    /// Image part was rejected by validation or budget policy.
    Rejected,
    /// Image part was valid but unavailable from a host service.
    Unavailable,
}

impl ImageOutcomeStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Included => "included",
            Self::Rejected => "rejected",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Safe image assembly outcome metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageOutcome {
    status: ImageOutcomeStatus,
    safe_label: alloc::string::String,
    content_hash: alloc::string::String,
    reason: Option<alloc::string::String>,
}

impl ImageOutcome {
    pub(super) fn included(image: &super::ImagePart) -> Self {
        Self::new(ImageOutcomeStatus::Included, image, None)
    }

    pub(super) fn rejected(image: &super::ImagePart, reason: &str) -> Self {
        Self::new(ImageOutcomeStatus::Rejected, image, Some(reason))
    }

    fn new(status: ImageOutcomeStatus, image: &super::ImagePart, reason: Option<&str>) -> Self {
        Self {
            status,
            safe_label: alloc::string::String::from(image.safe_label()),
            content_hash: alloc::string::String::from(image.content_hash()),
            reason: reason.map(alloc::string::String::from),
        }
    }

    /// Returns the outcome status.
    #[must_use]
    pub const fn status(&self) -> ImageOutcomeStatus {
        self.status
    }
    /// Returns the safe image label.
    #[must_use]
    pub fn safe_label(&self) -> &str {
        self.safe_label.as_str()
    }
    /// Returns the safe image hash.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }
    /// Returns the optional safe reason label.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

/// Deterministic multimodal prompt assembly policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromptAssemblyPolicy {
    image_bytes: u64,
    video_bytes: u64,
    video_count: u64,
    aggregate_media_bytes: u64,
}

impl PromptAssemblyPolicy {
    /// Builds an image-compatible policy with default video bounds.
    #[must_use]
    pub const fn new(max_image_bytes: u64) -> Self {
        Self {
            image_bytes: max_image_bytes,
            video_bytes: super::DEFAULT_PROMPT_VIDEO_BYTE_BUDGET,
            video_count: super::DEFAULT_PROMPT_VIDEO_COUNT,
            aggregate_media_bytes: super::DEFAULT_AGGREGATE_MEDIA_BYTE_BUDGET,
        }
    }

    /// Sets explicit named video and aggregate bounds.
    #[must_use]
    pub const fn with_video_limits(
        mut self,
        max_video_bytes: u64,
        max_video_count: u64,
        max_aggregate_media_bytes: u64,
    ) -> Self {
        self.video_bytes = max_video_bytes;
        self.video_count = max_video_count;
        self.aggregate_media_bytes = max_aggregate_media_bytes;
        self
    }

    /// Returns maximum image bytes per image.
    #[must_use]
    pub const fn max_image_bytes(self) -> u64 {
        self.image_bytes
    }
    /// Returns maximum video bytes per video.
    #[must_use]
    pub const fn max_video_bytes(self) -> u64 {
        self.video_bytes
    }
    /// Returns maximum included video count.
    #[must_use]
    pub const fn max_video_count(self) -> u64 {
        self.video_count
    }
    /// Returns maximum aggregate included media bytes.
    #[must_use]
    pub const fn max_aggregate_media_bytes(self) -> u64 {
        self.aggregate_media_bytes
    }
}

impl Default for PromptAssemblyPolicy {
    fn default() -> Self {
        Self::new(super::DEFAULT_PROMPT_IMAGE_BYTE_BUDGET)
    }
}

/// Assembled model-visible content and ordered media outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptAssembly {
    parts: alloc::vec::Vec<super::ContentPart>,
    image_outcomes: alloc::vec::Vec<ImageOutcome>,
    video_outcomes: alloc::vec::Vec<super::VideoOutcome>,
    aggregate_media_bytes: u64,
}

impl PromptAssembly {
    /// Returns content parts in admitted request order.
    #[must_use]
    pub fn parts(&self) -> &[super::ContentPart] {
        self.parts.as_slice()
    }
    /// Returns image outcomes in request order.
    #[must_use]
    pub fn image_outcomes(&self) -> &[ImageOutcome] {
        self.image_outcomes.as_slice()
    }
    /// Returns video outcomes in request order.
    #[must_use]
    pub fn video_outcomes(&self) -> &[super::VideoOutcome] {
        self.video_outcomes.as_slice()
    }
    /// Returns true when an image is included.
    #[must_use]
    pub fn has_images(&self) -> bool {
        self.parts
            .iter()
            .any(|part| matches!(part, super::ContentPart::Image(_image)))
    }
    /// Returns true when a video is included.
    #[must_use]
    pub fn has_videos(&self) -> bool {
        self.parts
            .iter()
            .any(|part| matches!(part, super::ContentPart::Video(_video)))
    }
    /// Returns included image count.
    #[must_use]
    pub fn image_count(&self) -> u64 {
        super::check::usize_to_u64(
            self.parts
                .iter()
                .filter(|part| matches!(part, super::ContentPart::Image(_image)))
                .count(),
        )
    }
    /// Returns included video count.
    #[must_use]
    pub fn video_count(&self) -> u64 {
        super::check::usize_to_u64(
            self.parts
                .iter()
                .filter(|part| matches!(part, super::ContentPart::Video(_video)))
                .count(),
        )
    }
    /// Returns aggregate included image and video bytes.
    #[must_use]
    pub const fn aggregate_media_bytes(&self) -> u64 {
        self.aggregate_media_bytes
    }
}

/// Builds ordered text, image, and video content under explicit media budgets.
///
/// # Errors
///
/// Returns [`super::Error`] for malformed parts, duplicate video identities, or invalid policy bounds.
pub fn assemble_prompt_parts(
    drafts: &[super::ContentPartDraft<'_>],
    policy: PromptAssemblyPolicy,
) -> Result<PromptAssembly, super::Error> {
    validate_assembly_inputs(drafts, policy)?;
    let mut state = AssemblyState::new(drafts.len(), policy);
    for draft in drafts {
        state.push(super::ContentPart::from_draft(*draft)?)?;
    }
    Ok(state.finish(drafts.len()))
}

fn validate_assembly_inputs(
    drafts: &[super::ContentPartDraft<'_>],
    policy: PromptAssemblyPolicy,
) -> Result<(), super::Error> {
    if drafts.is_empty() {
        return Err(super::Error::EmptyParts);
    }
    if policy.max_image_bytes() > super::MAX_IMAGE_BYTE_COUNT {
        return Err(super::Error::InvalidMediaPolicy);
    }
    if policy.max_video_bytes() > super::MAX_VIDEO_BYTE_COUNT {
        return Err(super::Error::InvalidMediaPolicy);
    }
    if policy.max_video_count() == 0 {
        return Err(super::Error::InvalidMediaPolicy);
    }
    if policy.max_aggregate_media_bytes() == 0 {
        return Err(super::Error::InvalidMediaPolicy);
    }
    Ok(())
}

struct AssemblyState {
    policy: PromptAssemblyPolicy,
    parts: alloc::vec::Vec<super::ContentPart>,
    image_outcomes: alloc::vec::Vec<ImageOutcome>,
    video_outcomes: alloc::vec::Vec<super::VideoOutcome>,
    admitted_video_hashes: alloc::vec::Vec<alloc::string::String>,
    aggregate_media_bytes: u64,
    included_video_count: u64,
}

impl AssemblyState {
    fn new(capacity: usize, policy: PromptAssemblyPolicy) -> Self {
        Self {
            policy,
            parts: alloc::vec::Vec::with_capacity(capacity),
            image_outcomes: alloc::vec::Vec::with_capacity(capacity),
            video_outcomes: alloc::vec::Vec::with_capacity(capacity),
            admitted_video_hashes: alloc::vec::Vec::new(),
            aggregate_media_bytes: 0,
            included_video_count: 0,
        }
    }

    fn push(&mut self, part: super::ContentPart) -> Result<(), super::Error> {
        match part {
            super::ContentPart::Text(text) => self.parts.push(super::ContentPart::Text(text)),
            super::ContentPart::Image(image) => self.push_image(image),
            super::ContentPart::Video(video) => self.push_video(video)?,
        }
        Ok(())
    }

    fn push_image(&mut self, image: super::ImagePart) {
        let next_aggregate = self
            .aggregate_media_bytes
            .saturating_add(image.byte_count());
        if image.byte_count() > self.policy.max_image_bytes() {
            self.image_outcomes
                .push(ImageOutcome::rejected(&image, "image_byte_budget_exceeded"));
        } else if next_aggregate > self.policy.max_aggregate_media_bytes() {
            self.image_outcomes.push(ImageOutcome::rejected(
                &image,
                "aggregate_media_budget_exceeded",
            ));
        } else {
            self.aggregate_media_bytes = next_aggregate;
            self.image_outcomes.push(ImageOutcome::included(&image));
            self.parts.push(super::ContentPart::Image(image));
        }
    }

    fn push_video(&mut self, video: super::VideoPart) -> Result<(), super::Error> {
        self.reject_duplicate_video(&video)?;
        self.admitted_video_hashes
            .push(alloc::string::String::from(video.content_hash()));
        let next_aggregate = self
            .aggregate_media_bytes
            .saturating_add(video.byte_count());
        if video.byte_count() > self.policy.max_video_bytes() {
            self.video_outcomes.push(super::VideoOutcome::rejected(
                &video,
                "video_byte_budget_exceeded",
            ));
        } else if self.included_video_count >= self.policy.max_video_count() {
            self.video_outcomes.push(super::VideoOutcome::rejected(
                &video,
                "video_count_budget_exceeded",
            ));
        } else if next_aggregate > self.policy.max_aggregate_media_bytes() {
            self.video_outcomes.push(super::VideoOutcome::rejected(
                &video,
                "aggregate_media_budget_exceeded",
            ));
        } else {
            self.aggregate_media_bytes = next_aggregate;
            self.included_video_count = self.included_video_count.saturating_add(1);
            self.video_outcomes
                .push(super::VideoOutcome::included(&video));
            self.parts.push(super::ContentPart::Video(video));
        }
        Ok(())
    }

    fn reject_duplicate_video(&self, video: &super::VideoPart) -> Result<(), super::Error> {
        if self
            .admitted_video_hashes
            .iter()
            .any(|hash| hash == video.content_hash())
        {
            return Err(super::Error::DuplicateVideoIdentity {
                content_hash: alloc::string::String::from(video.content_hash()),
            });
        }
        Ok(())
    }

    fn finish(self, draft_count: usize) -> PromptAssembly {
        assert!(self.parts.len() <= draft_count);
        assert!(self.video_outcomes.len() <= draft_count);
        assert!(self.aggregate_media_bytes <= self.policy.max_aggregate_media_bytes());
        PromptAssembly {
            parts: self.parts,
            image_outcomes: self.image_outcomes,
            video_outcomes: self.video_outcomes,
            aggregate_media_bytes: self.aggregate_media_bytes,
        }
    }
}
