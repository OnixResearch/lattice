use super::{
    Collection, ExternalSubagentDescriptor, ExternalSubagentDescriptorDraft, ID_FIELD,
    MIN_TURN_COUNT, MODEL_FIELD, PARAMETER_FIELD, PROFILE_FIELD, RECIPE_FIELD, ROLE_FIELD,
    SOURCE_USER, SUMMARY_SCHEMA_FIELD, SubagentDelegationSource, SubagentError, SubagentReturnMode,
    TASK_FIELD, TIMEOUT_FIELD, TOOL_FIELD, Text, WORKSPACE_FIELD, optional_text, required_text,
    text_collection,
};

/// Untrusted recipe-backed subagent delegation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentDelegationDraft<'a> {
    /// Child run identity.
    pub id: &'a str,
    /// Subagent role label.
    pub role: &'a str,
    /// Child task summary.
    pub task: &'a str,
    /// Optional admitted recipe identity.
    pub recipe_id: Option<&'a str>,
    /// Optional profile identity.
    pub profile_id: Option<&'a str>,
    /// Validated parameter binding labels.
    pub parameter_bindings: &'a [&'a str],
    /// Requested tool ceilings.
    pub tool_ceilings: &'a [&'a str],
    /// Workspace scope label.
    pub workspace_scope: &'a str,
    /// Model profile label.
    pub model_profile: &'a str,
    /// Timeout class label.
    pub timeout_class: &'a str,
    /// Maximum child turns.
    pub max_turns: u32,
    /// Return mode label.
    pub return_mode: &'a str,
    /// Summary schema label.
    pub summary_schema: &'a str,
    /// Delegation source label.
    pub delegation_source: &'a str,
    /// Whether this request is recursive from another subagent.
    pub recursive: bool,
    /// Recipe trust receipt when a recipe is referenced.
    pub recipe_trust: Option<&'a crate::RecipeTrustReceipt>,
    /// Whether automation source was admitted.
    pub automation_admitted: bool,
    /// Optional external subagent descriptor.
    pub external: Option<ExternalSubagentDescriptorDraft<'a>>,
}

/// Validated recipe-backed subagent delegation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentDelegation {
    id: Text,
    role: Text,
    task: Text,
    recipe_id: Option<Text>,
    profile_id: Option<Text>,
    parameter_bindings: Collection<Text>,
    tool_ceilings: Collection<Text>,
    workspace_scope: Text,
    model_profile: Text,
    timeout_class: Text,
    max_turns: u32,
    return_mode: SubagentReturnMode,
    summary_schema: Text,
    delegation_source: SubagentDelegationSource,
    external: Option<ExternalSubagentDescriptor>,
}

impl SubagentDelegation {
    /// Validates a subagent delegation request without starting a child run.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentError`] when delegation is implicit, recursive,
    /// untrusted, unbounded, or requests ambient external-agent authority.
    pub fn validate(draft: &SubagentDelegationDraft<'_>) -> Result<Self, SubagentError> {
        let delegation_source = SubagentDelegationSource::parse(draft.delegation_source)?;
        validate_delegation_admission(draft, delegation_source)?;
        let id = required_text(ID_FIELD, draft.id)?;
        let role = required_text(ROLE_FIELD, draft.role)?;
        let task = required_text(TASK_FIELD, draft.task)?;
        let recipe_id = optional_text(RECIPE_FIELD, draft.recipe_id)?;
        let profile_id = optional_text(PROFILE_FIELD, draft.profile_id)?;
        let parameter_bindings = text_collection(PARAMETER_FIELD, draft.parameter_bindings, false)?;
        let tool_ceilings = text_collection(TOOL_FIELD, draft.tool_ceilings, true)?;
        let workspace_scope = required_text(WORKSPACE_FIELD, draft.workspace_scope)?;
        let model_profile = required_text(MODEL_FIELD, draft.model_profile)?;
        let deadline_label = required_text(TIMEOUT_FIELD, draft.timeout_class)?;
        if draft.max_turns < MIN_TURN_COUNT {
            return Err(SubagentError::InvalidTurnCeiling);
        }
        let return_mode = SubagentReturnMode::parse(draft.return_mode)?;
        let summary_schema = required_text(SUMMARY_SCHEMA_FIELD, draft.summary_schema)?;
        let external = draft
            .external
            .as_ref()
            .map(ExternalSubagentDescriptor::validate)
            .transpose()?;
        Ok(Self {
            id,
            role,
            task,
            recipe_id,
            profile_id,
            parameter_bindings,
            tool_ceilings,
            workspace_scope,
            model_profile,
            timeout_class: deadline_label,
            max_turns: draft.max_turns,
            return_mode,
            summary_schema,
            delegation_source,
            external,
        })
    }

    /// Returns the child run identity.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the role label.
    #[must_use]
    pub fn role(&self) -> &str {
        self.role.as_str()
    }

    /// Returns the child task summary.
    #[must_use]
    pub fn task(&self) -> &str {
        self.task.as_str()
    }

    /// Returns the optional recipe identity.
    #[must_use]
    pub fn recipe_id(&self) -> Option<&str> {
        self.recipe_id.as_deref()
    }

    /// Returns the optional profile identity.
    #[must_use]
    pub fn profile_id(&self) -> Option<&str> {
        self.profile_id.as_deref()
    }

    /// Returns parameter binding labels.
    #[must_use]
    pub fn parameter_bindings(&self) -> &[Text] {
        self.parameter_bindings.as_slice()
    }

    /// Returns tool ceiling labels.
    #[must_use]
    pub fn tool_ceilings(&self) -> &[Text] {
        self.tool_ceilings.as_slice()
    }

    /// Returns the workspace scope label.
    #[must_use]
    pub fn workspace_scope(&self) -> &str {
        self.workspace_scope.as_str()
    }

    /// Returns the model profile label.
    #[must_use]
    pub fn model_profile(&self) -> &str {
        self.model_profile.as_str()
    }

    /// Returns the timeout class label.
    #[must_use]
    pub fn timeout_class(&self) -> &str {
        self.timeout_class.as_str()
    }

    /// Returns the maximum turn count.
    #[must_use]
    pub const fn max_turns(&self) -> u32 {
        self.max_turns
    }

    /// Returns the return mode.
    #[must_use]
    pub const fn return_mode(&self) -> SubagentReturnMode {
        self.return_mode
    }

    /// Returns the summary schema label.
    #[must_use]
    pub fn summary_schema(&self) -> &str {
        self.summary_schema.as_str()
    }

    /// Returns the delegation source.
    #[must_use]
    pub const fn delegation_source(&self) -> SubagentDelegationSource {
        self.delegation_source
    }

    /// Returns the external subagent descriptor, when any.
    #[must_use]
    pub const fn external(&self) -> Option<&ExternalSubagentDescriptor> {
        self.external.as_ref()
    }
}
fn validate_delegation_admission(
    draft: &SubagentDelegationDraft<'_>,
    source: SubagentDelegationSource,
) -> Result<(), SubagentError> {
    assert!(
        !draft.delegation_source.is_empty(),
        "delegation source was parsed"
    );
    assert!(!SOURCE_USER.is_empty(), "source labels are stable");
    if source == SubagentDelegationSource::Automatic {
        return Err(SubagentError::ImplicitDelegationDenied);
    }
    if draft.recursive {
        return Err(SubagentError::RecursiveDelegationDenied);
    }
    if source == SubagentDelegationSource::AdmittedAutomation && !draft.automation_admitted {
        return Err(SubagentError::AutomationDelegationDenied);
    }
    if draft.recipe_id.is_some() {
        let Some(receipt) = draft.recipe_trust else {
            return Err(SubagentError::UntrustedRecipeDenied);
        };
        if receipt.status() != crate::RecipeTrustStatus::Admitted {
            return Err(SubagentError::UntrustedRecipeDenied);
        }
    }
    Ok(())
}
