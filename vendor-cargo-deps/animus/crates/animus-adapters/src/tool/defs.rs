/// Activation status for model-facing tool metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefinitionStatus {
    /// Tool metadata may be shown to the model and considered for dispatch.
    Active,
    /// Tool metadata is registered but not model-visible or dispatch-admitted.
    Inactive,
    /// Tool metadata is present but not trusted for prompt or dispatch admission.
    Untrusted,
}

/// Borrowed tool definition data before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefinitionDraft<'a> {
    /// Optional tool name, required by validation.
    pub name: Option<&'a str>,
    /// Optional human-facing label, required by validation.
    pub label: Option<&'a str>,
    /// Optional model-facing description, required by validation.
    pub description: Option<&'a str>,
    /// Optional parameter schema identity, required by validation.
    pub parameter_schema_identity: Option<&'a str>,
    /// Optional prompt snippet attributed to this tool.
    pub prompt_snippet: Option<&'a str>,
    /// Tool-specific prompt guidelines.
    pub prompt_guidelines: &'a [&'a str],
    /// Optional source identity, required by validation.
    pub source_identity: Option<&'a str>,
    /// Optional provenance label, required by validation.
    pub provenance: Option<&'a str>,
    /// Capability labels required by the tool.
    pub capability_labels: &'a [&'a str],
    /// Optional result metadata shape identity, required by validation.
    pub result_metadata_shape: Option<&'a str>,
    /// Whether the definition is active for prompt construction.
    pub status: DefinitionStatus,
}

/// Validated tool definition metadata with no executor implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    pub(super) name: super::Text,
    label: super::Text,
    description: super::Text,
    parameter_schema_identity: super::Text,
    pub(super) prompt_snippet: super::Text,
    pub(super) prompt_guidelines: super::Collection<super::Text>,
    source_identity: super::Text,
    provenance: super::Text,
    capability_labels: super::Collection<super::Text>,
    result_metadata_shape: super::Text,
    status: DefinitionStatus,
}

impl Definition {
    /// Validates model-facing and runtime-facing tool definition metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractError`] when required fields are missing, field
    /// text is malformed, or metadata collections exceed the bounded contract size.
    pub fn validate(draft: &DefinitionDraft<'_>) -> Result<Self, super::ContractError> {
        Ok(Self {
            name: super::required_text(super::NAME_FIELD, draft.name)?,
            label: super::required_text(super::LABEL_FIELD, draft.label)?,
            description: super::required_text(super::DESCRIPTION_FIELD, draft.description)?,
            parameter_schema_identity: super::required_text(super::SCHEMA_FIELD, draft.parameter_schema_identity)?,
            prompt_snippet: super::required_text(super::PROMPT_SNIPPET_FIELD, draft.prompt_snippet)?,
            prompt_guidelines: super::required_texts(super::PROMPT_GUIDELINE_FIELD, draft.prompt_guidelines)?,
            source_identity: super::required_text(super::SOURCE_FIELD, draft.source_identity)?,
            provenance: super::required_text(super::PROVENANCE_FIELD, draft.provenance)?,
            capability_labels: super::normalized_label_set(super::CAPABILITY_FIELD, draft.capability_labels)?,
            result_metadata_shape: super::required_text(super::RESULT_SHAPE_FIELD, draft.result_metadata_shape)?,
            status: draft.status,
        })
    }

    /// Returns the validated tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the human-facing label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the model-facing description.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the parameter schema identity.
    #[must_use]
    pub fn parameter_schema_identity(&self) -> &str {
        self.parameter_schema_identity.as_str()
    }

    /// Returns the prompt snippet.
    #[must_use]
    pub fn prompt_snippet(&self) -> &str {
        self.prompt_snippet.as_str()
    }

    /// Returns attributed prompt guidelines.
    #[must_use]
    pub fn prompt_guidelines(&self) -> &[super::Text] {
        self.prompt_guidelines.as_slice()
    }

    /// Returns source identity metadata.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }

    /// Returns provenance metadata.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }

    /// Returns capability labels.
    #[must_use]
    pub fn capability_labels(&self) -> &[super::Text] {
        self.capability_labels.as_slice()
    }

    /// Returns result metadata shape identity.
    #[must_use]
    pub fn result_metadata_shape(&self) -> &str {
        self.result_metadata_shape.as_str()
    }

    /// Returns whether this definition is active for prompt construction.
    #[must_use]
    pub const fn status(&self) -> DefinitionStatus {
        self.status
    }
}

/// Model-facing prompt context entry kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptContextKind {
    /// Tool prompt snippet.
    Snippet,
    /// Tool-specific prompt guideline.
    Guideline,
}

/// One model-facing prompt context entry attributed to a tool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptContextEntry {
    tool_name: super::Text,
    kind: PromptContextKind,
    text: super::Text,
}

impl PromptContextEntry {
    /// Returns the attributed tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns whether this entry is a snippet or guideline.
    #[must_use]
    pub const fn kind(&self) -> PromptContextKind {
        self.kind
    }

    /// Returns the model-facing text.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
}

/// Builds model-facing prompt context from active tool metadata only.
///
/// # Panics
///
/// Panics when callers pass more than the bounded tool metadata count.
#[must_use]
pub fn build_prompt_context(definitions: &[Definition]) -> alloc::vec::Vec<PromptContextEntry> {
    assert!(
        definitions.len() <= super::MAX_TOOL_METADATA_ITEMS,
        "tool definitions are bounded before prompt assembly"
    );
    let entry_count = prompt_context_entry_count(definitions);
    assert!(entry_count <= max_prompt_context_entries(), "prompt context capacity is bounded");
    let mut entries = super::Collection::with_capacity(entry_count);
    for definition in definitions {
        push_active_prompt_entries(&mut entries, definition);
    }
    entries
}

fn prompt_context_entry_count(definitions: &[Definition]) -> usize {
    let mut entry_count = 0usize;
    for definition in definitions {
        if definition.status() == DefinitionStatus::Active {
            entry_count = entry_count.saturating_add(1);
            entry_count = entry_count.saturating_add(definition.prompt_guidelines().len());
        }
    }
    entry_count
}

const fn max_prompt_context_entries() -> usize {
    super::MAX_TOOL_METADATA_ITEMS.saturating_mul(super::MAX_TOOL_METADATA_ITEMS.saturating_add(1))
}

fn push_active_prompt_entries(entries: &mut super::Collection<PromptContextEntry>, definition: &Definition) {
    if definition.status() != DefinitionStatus::Active {
        return;
    }
    entries.push(PromptContextEntry {
        tool_name: definition.name.clone(),
        kind: PromptContextKind::Snippet,
        text: definition.prompt_snippet.clone(),
    });
    for guideline in definition.prompt_guidelines() {
        entries.push(PromptContextEntry {
            tool_name: definition.name.clone(),
            kind: PromptContextKind::Guideline,
            text: guideline.clone(),
        });
    }
}
