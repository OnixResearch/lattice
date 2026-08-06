use super::Text;
use super::consts::CONTEXT_AGENT_FIELD;
use super::consts::CONTEXT_PANE_FIELD;
use super::consts::CONTEXT_PLUGIN_CONFIG_FIELD;
use super::consts::CONTEXT_PLUGIN_ROOT_FIELD;
use super::consts::CONTEXT_PLUGIN_STATE_FIELD;
use super::consts::CONTEXT_REQUEST_FIELD;
use super::consts::CONTEXT_SESSION_FIELD;
use super::consts::CONTEXT_SOURCE_METADATA_FIELD;
use super::consts::CONTEXT_TAB_FIELD;
use super::consts::CONTEXT_WORKSPACE_FIELD;
use super::consts::CONTEXT_WORKTREE_FIELD;
use super::consts::ROLE_KIND_ACTION;
use super::consts::ROLE_KIND_EVENT_HOOK;
use super::consts::ROLE_KIND_HOST_PANE_ENTRYPOINT;
use super::consts::ROLE_KIND_LINK_HANDLER;
use super::context::validate_invocation_label;
use super::types::PluginCompatibility;
use super::types::PluginDiagnostic;
use super::types::PluginDiagnosticKind;
use super::types::PluginDiagnosticSeverity;
use super::types::PluginInvocationContext;
use super::types::PluginInvocationContextDraft;
use super::types::PluginInvocationContextLeak;
use super::types::PluginRegistryOperationKind;
use super::types::PluginRegistryPlan;
use super::types::PluginRegistryPlanStatus;
use super::types::PluginRole;
use super::types::PluginRoleKind;

impl PluginCompatibility {
    /// Returns the minimum host version label.
    #[must_use]
    pub fn minimum_host_version(&self) -> &str {
        self.minimum_host_version.as_str()
    }

    /// Returns required host capability labels.
    #[must_use]
    pub fn required_host_capabilities(&self) -> &[Text] {
        self.required_host_capabilities.as_slice()
    }

    /// Returns supported platform labels.
    #[must_use]
    pub fn supported_platforms(&self) -> &[Text] {
        self.supported_platforms.as_slice()
    }
}

impl PluginRoleKind {
    /// Returns the stable role spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Action => ROLE_KIND_ACTION,
            Self::EventHook => ROLE_KIND_EVENT_HOOK,
            Self::HostPaneEntrypoint => ROLE_KIND_HOST_PANE_ENTRYPOINT,
            Self::LinkHandler => ROLE_KIND_LINK_HANDLER,
        }
    }
}

impl PluginRole {
    /// Returns the role kind.
    #[must_use]
    pub const fn kind(&self) -> PluginRoleKind {
        self.kind
    }

    /// Returns the role ID.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the human-facing role title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns safe context labels.
    #[must_use]
    pub fn context_labels(&self) -> &[Text] {
        self.context_labels.as_slice()
    }

    /// Returns the optional data-only command label.
    #[must_use]
    pub fn command_label(&self) -> Option<&str> {
        self.command_label.as_deref()
    }

    /// Returns the optional data-only artifact label.
    #[must_use]
    pub fn artifact_label(&self) -> Option<&str> {
        self.artifact_label.as_deref()
    }

    /// Returns the optional event name.
    #[must_use]
    pub fn event_name(&self) -> Option<&str> {
        self.event_name.as_deref()
    }

    /// Returns the optional link scheme.
    #[must_use]
    pub fn link_scheme(&self) -> Option<&str> {
        self.link_scheme.as_deref()
    }

    /// Returns effective role compatibility.
    #[must_use]
    pub const fn compatibility(&self) -> &PluginCompatibility {
        &self.compatibility
    }
}

impl PluginInvocationContextLeak {
    /// Returns the stable leak label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawPrompt => "raw_prompt",
            Self::Transcript => "transcript",
            Self::Credential => "credential",
            Self::FullPrivatePath => "full_private_path",
            Self::UnboundedLog => "unbounded_log",
            Self::ProviderPayload => "provider_payload",
        }
    }
}

impl PluginInvocationContext {
    /// Validates borrowed invocation context fields as safe labels.
    ///
    /// # Errors
    ///
    /// Returns [`PluginDescriptorError`] when a required context field is missing,
    /// invalid, or contains raw prompts, transcripts, credentials, private paths,
    /// unbounded logs, or provider payloads.
    pub fn validate(draft: &PluginInvocationContextDraft<'_>) -> Result<Self, super::super::PluginDescriptorError> {
        Ok(Self {
            workspace_label: validate_invocation_label(CONTEXT_WORKSPACE_FIELD, draft.workspace_label)?,
            tab_label: validate_invocation_label(CONTEXT_TAB_FIELD, draft.tab_label)?,
            pane_label: validate_invocation_label(CONTEXT_PANE_FIELD, draft.pane_label)?,
            agent_status_label: validate_invocation_label(CONTEXT_AGENT_FIELD, draft.agent_status_label)?,
            worktree_provenance_label: validate_invocation_label(
                CONTEXT_WORKTREE_FIELD,
                draft.worktree_provenance_label,
            )?,
            session_provenance_label: validate_invocation_label(CONTEXT_SESSION_FIELD, draft.session_provenance_label)?,
            request_identity: validate_invocation_label(CONTEXT_REQUEST_FIELD, draft.request_identity)?,
            plugin_root_label: validate_invocation_label(CONTEXT_PLUGIN_ROOT_FIELD, draft.plugin_root_label)?,
            plugin_config_label: validate_invocation_label(CONTEXT_PLUGIN_CONFIG_FIELD, draft.plugin_config_label)?,
            plugin_state_label: validate_invocation_label(CONTEXT_PLUGIN_STATE_FIELD, draft.plugin_state_label)?,
            source_metadata_label: validate_invocation_label(
                CONTEXT_SOURCE_METADATA_FIELD,
                draft.source_metadata_label,
            )?,
        })
    }

    /// Returns the safe workspace label.
    #[must_use]
    pub fn workspace_label(&self) -> &str {
        self.workspace_label.as_str()
    }

    /// Returns the safe request identity.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.request_identity.as_str()
    }
}

impl PluginDiagnosticSeverity {
    /// Returns true when this severity blocks host dispatch.
    #[must_use]
    pub const fn is_failure(self) -> bool {
        matches!(self, Self::Failure)
    }
}

impl PluginDiagnostic {
    /// Returns the diagnostic class.
    #[must_use]
    pub const fn kind(&self) -> PluginDiagnosticKind {
        self.kind
    }

    /// Returns the diagnostic severity.
    #[must_use]
    pub const fn severity(&self) -> PluginDiagnosticSeverity {
        self.severity
    }
}

impl PluginRegistryPlan {
    /// Returns the planned operation.
    #[must_use]
    pub const fn operation(&self) -> PluginRegistryOperationKind {
        self.operation
    }

    /// Returns the source manifest identity.
    #[must_use]
    pub fn manifest_identity(&self) -> &str {
        self.manifest_identity.as_str()
    }

    /// Returns the optional target entrypoint ID.
    #[must_use]
    pub fn entrypoint_id(&self) -> Option<&str> {
        self.entrypoint_id.as_deref()
    }

    /// Returns plan status.
    #[must_use]
    pub const fn status(&self) -> PluginRegistryPlanStatus {
        self.status
    }

    /// Returns retained diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[PluginDiagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns the optional data-only command label.
    #[must_use]
    pub fn command_label(&self) -> Option<&str> {
        self.command_label.as_deref()
    }

    /// Returns the optional data-only artifact label.
    #[must_use]
    pub fn artifact_label(&self) -> Option<&str> {
        self.artifact_label.as_deref()
    }
}
