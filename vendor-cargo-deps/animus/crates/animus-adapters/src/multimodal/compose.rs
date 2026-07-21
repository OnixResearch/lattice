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
        let reason = reason.map(alloc::string::String::from);
        Self {
            status,
            safe_label: alloc::string::String::from(image.safe_label()),
            content_hash: alloc::string::String::from(image.content_hash()),
            reason,
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

/// Deterministic prompt assembly policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromptAssemblyPolicy {
    max_image_bytes: u64,
}

impl PromptAssemblyPolicy {
    /// Builds a prompt assembly policy with a maximum image byte budget per image.
    #[must_use]
    pub const fn new(max_image_bytes: u64) -> Self {
        Self { max_image_bytes }
    }

    /// Returns the maximum image byte budget.
    #[must_use]
    pub const fn max_image_bytes(self) -> u64 {
        self.max_image_bytes
    }
}

impl Default for PromptAssemblyPolicy {
    fn default() -> Self {
        Self::new(super::DEFAULT_PROMPT_IMAGE_BYTE_BUDGET)
    }
}

/// Assembled model-visible content plus safe image outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptAssembly {
    parts: alloc::vec::Vec<super::ContentPart>,
    image_outcomes: alloc::vec::Vec<ImageOutcome>,
}

impl PromptAssembly {
    /// Returns model-visible content parts in deterministic order.
    #[must_use]
    pub fn parts(&self) -> &[super::ContentPart] {
        self.parts.as_slice()
    }

    /// Returns image outcomes in request order.
    #[must_use]
    pub fn image_outcomes(&self) -> &[ImageOutcome] {
        self.image_outcomes.as_slice()
    }

    /// Returns true when at least one image part is included.
    #[must_use]
    pub fn has_images(&self) -> bool {
        self.parts
            .iter()
            .any(|part| matches!(part, super::ContentPart::Image(_image)))
    }

    /// Returns the included image count.
    #[must_use]
    pub fn image_count(&self) -> u64 {
        super::check::usize_to_u64(
            self.parts
                .iter()
                .filter(|part| matches!(part, super::ContentPart::Image(_image)))
                .count(),
        )
    }
}

/// Builds ordered multimodal prompt content after deterministic image budget decisions.
///
/// # Errors
///
/// Returns [`super::Error`] when any part is malformed or no parts are provided.
pub fn assemble_prompt_parts(
    drafts: &[super::ContentPartDraft<'_>],
    policy: PromptAssemblyPolicy,
) -> Result<PromptAssembly, super::Error> {
    if drafts.is_empty() {
        return Err(super::Error::EmptyParts);
    }
    assert!(drafts.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS);
    assert!(policy.max_image_bytes() <= super::MAX_IMAGE_BYTE_COUNT);
    let mut parts = alloc::vec::Vec::with_capacity(drafts.len());
    let mut image_outcomes = alloc::vec::Vec::with_capacity(drafts.len());
    for draft in drafts {
        let part = super::ContentPart::from_draft(*draft)?;
        match part {
            super::ContentPart::Text(text) => parts.push(super::ContentPart::Text(text)),
            super::ContentPart::Image(image) => {
                if image.byte_count() > policy.max_image_bytes() {
                    image_outcomes
                        .push(ImageOutcome::rejected(&image, "image_byte_budget_exceeded"));
                } else {
                    image_outcomes.push(ImageOutcome::included(&image));
                    parts.push(super::ContentPart::Image(image));
                }
            }
        }
    }
    assert!(parts.len() <= drafts.len());
    assert!(image_outcomes.len() <= drafts.len());
    Ok(PromptAssembly {
        parts,
        image_outcomes,
    })
}
