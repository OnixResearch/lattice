impl super::PluginDescriptor {
    /// Returns the normalized descriptor identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the descriptor version.
    #[must_use]
    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    /// Returns the descriptor toolset-kind label.
    #[must_use]
    pub const fn toolset_kind(&self) -> super::ToolsetKind {
        self.toolset_kind
    }

    /// Returns the descriptor runtime-kind label.
    #[must_use]
    pub const fn runtime_kind(&self) -> super::RuntimeKind {
        self.runtime_kind
    }

    /// Returns safe manifest provenance.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }

    /// Returns host trust status.
    #[must_use]
    pub const fn trust_status(&self) -> super::TrustStatus {
        self.trust_status
    }

    /// Returns declared tools after validation and collision checks.
    #[must_use]
    pub fn declared_tools(&self) -> &[super::ToolDescriptor] {
        self.declared_tools.as_slice()
    }

    /// Returns data-only action, event-hook, pane-entrypoint, and link-handler roles.
    #[must_use]
    pub fn declared_roles(&self) -> &[super::PluginRole] {
        self.declared_roles.as_slice()
    }

    /// Returns manifest-level compatibility requirements.
    #[must_use]
    pub const fn compatibility(&self) -> &super::PluginCompatibility {
        &self.compatibility
    }

    /// Returns normalized capability labels.
    #[must_use]
    pub fn capabilities(&self) -> &[super::Text] {
        self.capabilities.as_slice()
    }

    /// Returns validated launch-policy metadata.
    #[must_use]
    pub const fn launch_policy(&self) -> &super::LaunchPolicy {
        &self.launch_policy
    }

    /// Returns validated sandbox and network claims.
    #[must_use]
    pub const fn sandbox(&self) -> &super::SandboxRequirements {
        &self.sandbox
    }

    /// Returns validated redaction claims.
    #[must_use]
    pub const fn redaction(&self) -> &super::RedactionPolicy {
        &self.redaction
    }
}

impl super::ToolsetKind {
    /// Returns the stable descriptor spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Generic => super::TOOLSET_KIND_GENERIC,
            Self::ReviewGate => super::TOOLSET_KIND_REVIEW_GATE,
        }
    }
}

impl super::TrustStatus {
    /// Returns the stable descriptor spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => super::TRUST_STATUS_TRUSTED,
            Self::Untrusted => super::TRUST_STATUS_UNTRUSTED,
        }
    }
}

impl super::RuntimeKind {
    /// Returns the stable descriptor spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Host => super::RUNTIME_KIND_HOST,
            Self::Stdio => super::RUNTIME_KIND_STDIO,
            Self::Wasm => super::RUNTIME_KIND_WASM,
        }
    }

    pub(super) const fn requires_launch_command(self) -> bool {
        match self {
            Self::Host => false,
            Self::Stdio | Self::Wasm => true,
        }
    }
}

impl super::ToolDescriptor {
    /// Returns the normalized tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns descriptor text.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the parameter schema identity.
    #[must_use]
    pub fn parameter_schema_identity(&self) -> &str {
        self.parameter_schema_identity.as_str()
    }

    /// Returns the result contract identity.
    #[must_use]
    pub fn result_contract_identity(&self) -> &str {
        self.result_contract_identity.as_str()
    }

    /// Returns tool-specific capability labels.
    #[must_use]
    pub fn capability_labels(&self) -> &[super::Text] {
        self.capability_labels.as_slice()
    }

    pub(super) fn capability_label_strs(&self) -> super::Collection<&str> {
        self.capability_labels.iter().map(alloc::string::String::as_str).collect()
    }
}

impl super::WorkingDirectoryMode {
    /// Returns the stable descriptor spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProjectRoot => super::WORKING_DIRECTORY_PROJECT_ROOT,
            Self::ManifestDirectory => super::WORKING_DIRECTORY_MANIFEST_DIRECTORY,
        }
    }
}

impl super::LaunchPolicy {
    /// Returns the optional command or artifact label.
    #[must_use]
    pub fn command_label(&self) -> Option<&str> {
        self.command_label.as_deref()
    }

    /// Returns data-only argument labels.
    #[must_use]
    pub fn arguments(&self) -> &[super::Text] {
        self.arguments.as_slice()
    }

    /// Returns the working-directory descriptor mode.
    #[must_use]
    pub const fn working_directory(&self) -> super::WorkingDirectoryMode {
        self.working_directory
    }

    /// Returns environment variable names only.
    #[must_use]
    pub fn environment_allowlist(&self) -> &[super::Text] {
        self.environment_allowlist.as_slice()
    }

    /// Returns project-relative writable-root descriptors.
    #[must_use]
    pub fn writable_roots(&self) -> &[super::Text] {
        self.writable_roots.as_slice()
    }
}

impl super::SandboxMode {
    /// Returns the stable descriptor spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unconfined => super::SANDBOX_MODE_UNCONFINED,
            Self::Isolated => super::SANDBOX_MODE_ISOLATED,
        }
    }
}

impl super::NetworkPolicy {
    /// Returns the stable descriptor spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Denied => super::NETWORK_POLICY_DENIED,
            Self::Loopback => super::NETWORK_POLICY_LOOPBACK,
            Self::Outbound => super::NETWORK_POLICY_OUTBOUND,
        }
    }
}

impl super::SandboxRequirements {
    /// Returns the sandbox-mode claim.
    #[must_use]
    pub const fn mode(self) -> super::SandboxMode {
        self.mode
    }

    /// Returns the network-policy claim.
    #[must_use]
    pub const fn network(self) -> super::NetworkPolicy {
        self.network
    }
}

impl super::RedactionPolicy {
    /// Returns whether future host logs should redact transcript payloads.
    #[must_use]
    pub const fn redact_transcripts(self) -> bool {
        self.redact_transcripts
    }

    /// Returns whether future host logs should redact secret-looking payloads.
    #[must_use]
    pub const fn redact_secrets(self) -> bool {
        self.redact_secrets
    }
}
