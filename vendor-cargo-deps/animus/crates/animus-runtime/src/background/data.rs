/// Supported backend families for a backend-neutral start request.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProcessJobBackend {
    /// Built-in local process backend owned by a host shell.
    Native,
    /// Pueue queue backend owned by a host shell.
    Pueue,
    /// Systemd transient-unit backend owned by a host shell.
    Systemd,
}

impl ProcessJobBackend {
    /// Returns the stable backend label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Pueue => "pueue",
            Self::Systemd => "systemd",
        }
    }
}

/// Source precedence used to select profile manifests deterministically.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProcessJobProfileSourcePrecedence {
    /// User/global default profile source.
    Global,
    /// Workspace profile source.
    Workspace,
    /// Explicitly supplied source for this request.
    Explicit,
}

impl ProcessJobProfileSourcePrecedence {
    pub(crate) const fn rank(self) -> u8 {
        match self {
            Self::Global => super::PRECEDENCE_GLOBAL_RANK,
            Self::Workspace => super::PRECEDENCE_WORKSPACE_RANK,
            Self::Explicit => super::PRECEDENCE_EXPLICIT_RANK,
        }
    }
}

/// One loaded profile source supplied by a host shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessJobProfileSource {
    /// Deterministic precedence class for this source.
    pub precedence: ProcessJobProfileSourcePrecedence,
    /// Safe label for receipts; full paths and backend locators are rejected.
    pub safe_label: String,
    /// Already-loaded manifest data.
    pub manifest: ProcessJobProfileManifest,
}

impl ProcessJobProfileSource {
    /// Creates a loaded source and validates the safe source label.
    ///
    /// # Errors
    ///
    /// Returns [`super::ProcessJobProfileError`] when the source label is unsafe.
    pub fn new(
        precedence: ProcessJobProfileSourcePrecedence,
        safe_label: impl Into<String>,
        manifest: ProcessJobProfileManifest,
    ) -> Result<Self, super::ProcessJobProfileError> {
        let safe_label = safe_label.into();
        super::checks::validate_source_label(safe_label.as_str())?;
        Ok(Self {
            precedence,
            safe_label,
            manifest,
        })
    }
}

/// A loaded profile manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessJobProfileManifest {
    /// Manifest schema version.
    pub schema_version: u16,
    /// Named profiles in the manifest.
    pub profiles: Vec<NamedProcessJobProfile>,
}

impl ProcessJobProfileManifest {
    /// Creates a manifest with the current supported schema version.
    #[must_use]
    pub fn current(profiles: Vec<NamedProcessJobProfile>) -> Self {
        Self {
            schema_version: super::PROCESS_JOB_PROFILE_SCHEMA_VERSION,
            profiles,
        }
    }
}

/// A named profile entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedProcessJobProfile {
    /// Stable profile name.
    pub name: String,
    /// Profile body.
    pub profile: ProcessJobProfile,
}

/// A process-job profile body.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProcessJobProfile {
    /// Optional backend override; policy default is used when absent.
    pub backend: Option<ProcessJobBackend>,
    /// Shell command shape. Mutually exclusive with `program`.
    pub command: Option<String>,
    /// Direct program shape. Mutually exclusive with `command`.
    pub program: Option<String>,
    /// Program arguments; valid only when `program` is set.
    pub args: Vec<String>,
    /// Optional working directory selected by policy prefixes.
    pub cwd: Option<String>,
    /// Writable paths selected by policy prefixes.
    pub writable_paths: Vec<String>,
    /// Environment variables passed to the eventual host backend.
    pub env: Vec<ProcessJobEnvironmentVariable>,
    /// Runtime resource limits validated before dispatch.
    pub resources: ProcessJobResourceLimits,
    /// Completion notification preference.
    pub notification: ProcessJobNotificationPolicy,
    /// Safe identity metadata copied into identity receipts.
    pub metadata: Vec<ProcessJobMetadataField>,
}

/// Environment key/value pair for a start request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessJobEnvironmentVariable {
    /// Environment key.
    pub key: String,
    /// Environment value. Values are not copied into identity receipts.
    pub value: String,
}

/// Safe metadata field copied into identity envelopes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessJobMetadataField {
    /// Metadata key.
    pub key: String,
    /// Metadata value.
    pub value: String,
}

/// Resource limits validated before backend dispatch.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProcessJobResourceLimits {
    /// Optional timeout in seconds.
    pub timeout_seconds: Option<u32>,
    /// Optional memory cap in bytes.
    pub memory_bytes: Option<u64>,
    /// Optional CPU quota in milli-CPUs.
    pub cpu_millis: Option<u32>,
    /// Optional log cap in bytes.
    pub log_bytes: Option<u64>,
}

/// Notification policy copied into the backend-neutral start request.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProcessJobNotificationPolicy {
    /// Emit a completion notification after terminal status when the host supports it.
    pub notify_on_complete: bool,
}

/// Policy bounds for resolving process-job profiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessJobProfilePolicy {
    /// Default backend used when a profile omits one.
    pub default_backend: ProcessJobBackend,
    /// Backends allowed by policy.
    pub allowed_backends: Vec<ProcessJobBackend>,
    /// Allowed environment key prefixes.
    pub allowed_env_prefixes: Vec<String>,
    /// Allowed working-directory prefixes.
    pub allowed_cwd_prefixes: Vec<String>,
    /// Allowed writable-path prefixes.
    pub allowed_writable_path_prefixes: Vec<String>,
    /// Maximum timeout in seconds.
    pub max_timeout_seconds: Option<u32>,
    /// Maximum memory in bytes.
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU in milli-CPUs.
    pub max_cpu_millis: Option<u32>,
    /// Maximum retained log bytes.
    pub max_log_bytes: Option<u64>,
    /// Safe policy source label.
    pub policy_source: String,
    /// BLAKE3 hash of the exported policy data.
    pub policy_hash: String,
}

impl Default for ProcessJobProfilePolicy {
    fn default() -> Self {
        let mut policy = Self {
            default_backend: ProcessJobBackend::Native,
            allowed_backends: vec![ProcessJobBackend::Native],
            allowed_env_prefixes: Vec::new(),
            allowed_cwd_prefixes: Vec::new(),
            allowed_writable_path_prefixes: Vec::new(),
            max_timeout_seconds: None,
            max_memory_bytes: None,
            max_cpu_millis: None,
            max_log_bytes: None,
            policy_source: String::from(super::DEFAULT_POLICY_SOURCE),
            policy_hash: String::new(),
        };
        policy.policy_hash = super::digest::policy_hash(&policy);
        policy
    }
}

impl ProcessJobProfilePolicy {
    /// Returns policy with a deterministic hash derived from its public fields.
    #[must_use]
    pub fn with_computed_hash(mut self) -> Self {
        self.policy_hash = super::digest::policy_hash(&self);
        self
    }
}

/// Backend-neutral request returned by profile resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartProcessJobRequest {
    /// Selected backend.
    pub backend: ProcessJobBackend,
    /// Shell command, when this profile uses command shape.
    pub command: Option<String>,
    /// Program, when this profile uses direct-exec shape.
    pub program: Option<String>,
    /// Direct-exec arguments.
    pub args: Vec<String>,
    /// Working directory selected by policy.
    pub cwd: Option<String>,
    /// Writable paths selected by policy.
    pub writable_paths: Vec<String>,
    /// Environment variables for the eventual host backend.
    pub env: Vec<ProcessJobEnvironmentVariable>,
    /// Resource limits for the eventual host backend.
    pub resources: ProcessJobResourceLimits,
    /// Notification policy for the eventual host backend.
    pub notification: ProcessJobNotificationPolicy,
    /// Safe owner label for receipts.
    pub owner_label: String,
    /// Safe identity metadata. Raw environment values are excluded.
    pub metadata: std::collections::BTreeMap<String, String>,
}

/// Input for resolving one named process-job profile.
#[derive(Clone, Copy)]
pub struct ProcessJobProfileResolveRequest<'a> {
    /// Loaded sources, ordered independently from precedence semantics.
    pub sources: &'a [ProcessJobProfileSource],
    /// Requested profile name.
    pub profile_name: &'a str,
    /// Safe owner label for the backend-neutral request.
    pub owner_label: &'a str,
    /// Policy bounds for resolution.
    pub policy: &'a ProcessJobProfilePolicy,
}

/// Resolved profile output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProcessJobProfile {
    /// Backend-neutral start request.
    pub request: StartProcessJobRequest,
    /// Safe identity envelope for receipts/replay.
    pub identity: super::ProcessJobProfileIdentity,
}
