/// Validated text content part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextPart {
    text: alloc::string::String,
}

impl TextPart {
    /// Validates a text content part.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the text part is empty, too long, or contains a NUL character.
    pub fn new(raw: &str) -> Result<Self, super::Error> {
        Ok(Self {
            text: crate::validation::normalized_adapter_field(super::TEXT_PART_FIELD, raw)?,
        })
    }

    /// Returns the text body.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}

/// Borrowed content-part draft before multimodal validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentPartDraft<'a> {
    /// Text content part.
    Text {
        /// Text body.
        text: &'a str,
    },
    /// Image content part.
    Image(super::ImagePartDraft<'a>),
    /// Video content part.
    Video(super::VideoPartDraft<'a>),
}

/// Validated provider-neutral content part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentPart {
    /// Text content part.
    Text(TextPart),
    /// Image content part.
    Image(super::ImagePart),
    /// Video content part.
    Video(super::VideoPart),
}

impl ContentPart {
    /// Validates a content-part draft.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the draft is invalid or unsafe.
    pub fn from_draft(draft: ContentPartDraft<'_>) -> Result<Self, super::Error> {
        match draft {
            ContentPartDraft::Text { text } => Ok(Self::Text(TextPart::new(text)?)),
            ContentPartDraft::Image(image) => Ok(Self::Image(super::ImagePart::from_draft(image)?)),
            ContentPartDraft::Video(video) => Ok(Self::Video(super::VideoPart::from_draft(video)?)),
        }
    }

    /// Returns the part modality.
    #[must_use]
    pub const fn modality(&self) -> super::InputModality {
        match self {
            Self::Text(_part) => super::InputModality::Text,
            Self::Image(_part) => super::InputModality::Image,
            Self::Video(_part) => super::InputModality::Video,
        }
    }

    /// Returns the image part when this is an image.
    #[must_use]
    pub const fn as_image(&self) -> Option<&super::ImagePart> {
        match self {
            Self::Image(image) => Some(image),
            Self::Text(_) | Self::Video(_) => None,
        }
    }

    /// Returns the video part when this is video.
    #[must_use]
    pub const fn as_video(&self) -> Option<&super::VideoPart> {
        match self {
            Self::Video(video) => Some(video),
            Self::Text(_) | Self::Image(_) => None,
        }
    }

    /// Returns the text part when this is text.
    #[must_use]
    pub const fn as_text(&self) -> Option<&TextPart> {
        match self {
            Self::Text(text) => Some(text),
            Self::Image(_) | Self::Video(_) => None,
        }
    }
}
