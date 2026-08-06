const PRESENTATION_TITLE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("presentation_title");
const DISPLAY_AGENT_NAME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("display_agent_name");
const ACTIVITY_TEXT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("activity_text");
const CUSTOM_STATE_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("custom_state_label");

/// Presentation-only status metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Presentation {
    title: Option<alloc::string::String>,
    display_agent_name: Option<alloc::string::String>,
    activity_text: Option<alloc::string::String>,
    custom_state_label: Option<alloc::string::String>,
}

impl Presentation {
    /// Builds empty presentation metadata with every optional field absent.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            title: None,
            display_agent_name: None,
            activity_text: None,
            custom_state_label: None,
        }
    }

    /// Builds validated presentation metadata.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when any supplied field is too
    /// long or contains a NUL character.
    pub fn new(draft: PresentationDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            title: optional_field(PRESENTATION_TITLE_FIELD, draft.title)?,
            display_agent_name: optional_field(DISPLAY_AGENT_NAME_FIELD, draft.display_agent_name)?,
            activity_text: optional_field(ACTIVITY_TEXT_FIELD, draft.activity_text)?,
            custom_state_label: optional_field(CUSTOM_STATE_LABEL_FIELD, draft.custom_state_label)?,
        })
    }

    /// Returns the optional presentation title.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the optional display agent name.
    #[must_use]
    pub fn display_agent_name(&self) -> Option<&str> {
        self.display_agent_name.as_deref()
    }

    /// Returns the optional compact activity text.
    #[must_use]
    pub fn activity_text(&self) -> Option<&str> {
        self.activity_text.as_deref()
    }

    /// Returns the optional visual state label.
    #[must_use]
    pub fn custom_state_label(&self) -> Option<&str> {
        self.custom_state_label.as_deref()
    }
}

/// Borrowed presentation metadata input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PresentationDraft<'a> {
    /// Optional title shown by presentation hosts.
    pub title: Option<&'a str>,
    /// Optional display-only agent name.
    pub display_agent_name: Option<&'a str>,
    /// Optional compact activity text.
    pub activity_text: Option<&'a str>,
    /// Optional display-only state label.
    pub custom_state_label: Option<&'a str>,
}

pub(super) fn optional_field(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, crate::AdapterContractError> {
    value.map_or(Ok(None), |text| {
        crate::validation::validate_optional_adapter_field(field_name, text)?;
        Ok(Some(alloc::string::String::from(text)))
    })
}
