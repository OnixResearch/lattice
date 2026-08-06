pub(super) fn validate(draft: &super::Draft<'_>) -> Result<super::Manifest, super::super::Error> {
    let fields = RequiredFields::from_draft(draft)?;
    let source_scope =
        source_scope_from_label(fields.source_text.as_str()).ok_or_else(|| super::super::Error::UnsupportedValue {
            field: super::super::SOURCE_FIELD.as_str(),
            value: fields.source_text.clone(),
        })?;
    let trust_status =
        trust_status_from_label(fields.trust_text.as_str()).ok_or_else(|| super::super::Error::UnsupportedValue {
            field: super::super::TRUST_FIELD.as_str(),
            value: fields.trust_text.clone(),
        })?;
    validate_prompt_budget(&fields.prompt_fragments)?;
    Ok(fields.into_manifest(source_scope, trust_status))
}

struct RequiredFields {
    name: super::super::Text,
    description: super::super::Text,
    model_preference: super::super::Text,
    prompt_fragments: super::super::Collection<super::super::Text>,
    capability_ceilings: super::super::Collection<super::super::Text>,
    toolset_limits: super::super::Collection<super::super::Text>,
    memory_scopes: super::super::Collection<super::super::Text>,
    context_reference_scopes: super::super::Collection<super::super::Text>,
    workflow_abilities: super::super::Collection<super::super::Text>,
    source_text: super::super::Text,
    trust_text: super::super::Text,
    manifest_identity: super::super::Text,
}

impl RequiredFields {
    fn from_draft(draft: &super::Draft<'_>) -> Result<Self, super::super::Error> {
        Ok(Self {
            name: super::super::required_text(super::super::NAME_FIELD, draft.name)?,
            description: super::super::required_text(super::super::DESCRIPTION_FIELD, draft.description)?,
            model_preference: super::super::required_text(super::super::MODEL_FIELD, draft.model_preference)?,
            prompt_fragments: super::super::required_collection(
                super::super::PROMPT_FRAGMENT_FIELD,
                draft.prompt_fragments,
            )?,
            capability_ceilings: super::super::required_label_set(
                super::super::CAPABILITY_FIELD,
                draft.capability_ceilings,
            )?,
            toolset_limits: super::super::required_label_set(super::super::TOOLSET_FIELD, draft.toolset_limits)?,
            memory_scopes: super::super::optional_label_set(super::super::MEMORY_FIELD, draft.memory_scopes)?,
            context_reference_scopes: super::super::optional_label_set(
                super::super::CONTEXT_FIELD,
                draft.context_reference_scopes,
            )?,
            workflow_abilities: super::super::optional_label_set(
                super::super::WORKFLOW_FIELD,
                draft.workflow_abilities,
            )?,
            source_text: super::super::required_text(super::super::SOURCE_FIELD, draft.source_scope)?,
            trust_text: super::super::required_text(super::super::TRUST_FIELD, draft.trust_status)?,
            manifest_identity: super::super::required_text(
                super::super::MANIFEST_IDENTITY_FIELD,
                draft.manifest_identity,
            )?,
        })
    }

    fn into_manifest(self, source_scope: super::SourceScope, trust_status: super::TrustStatus) -> super::Manifest {
        super::Manifest {
            name: self.name,
            description: self.description,
            model_preference: self.model_preference,
            prompt_fragments: self.prompt_fragments,
            capability_ceilings: self.capability_ceilings,
            toolset_limits: self.toolset_limits,
            memory_scopes: self.memory_scopes,
            context_reference_scopes: self.context_reference_scopes,
            workflow_abilities: self.workflow_abilities,
            source_scope,
            trust_status,
            identity: self.manifest_identity,
        }
    }
}

fn validate_prompt_budget(prompt_fragments: &[super::super::Text]) -> Result<(), super::super::Error> {
    let mut prompt_chars = 0usize;
    for fragment in prompt_fragments {
        prompt_chars = prompt_chars.saturating_add(fragment.chars().count());
    }
    if prompt_chars > super::super::MAX_PROMPT_CHARS {
        return Err(super::super::Error::PromptTooLong {
            max_chars: super::super::MAX_PROMPT_CHARS,
            actual_chars: prompt_chars,
        });
    }
    Ok(())
}

fn source_scope_from_label(value: &str) -> Option<super::SourceScope> {
    match value {
        super::super::USER_SOURCE_LABEL => Some(super::SourceScope::User),
        super::super::PROJECT_SOURCE_LABEL => Some(super::SourceScope::Project),
        super::super::HOST_SOURCE_LABEL => Some(super::SourceScope::ExplicitHost),
        _ => None,
    }
}

fn trust_status_from_label(value: &str) -> Option<super::TrustStatus> {
    match value {
        super::super::TRUSTED_LABEL => Some(super::TrustStatus::Trusted),
        super::super::UNTRUSTED_LABEL => Some(super::TrustStatus::Untrusted),
        _ => None,
    }
}
