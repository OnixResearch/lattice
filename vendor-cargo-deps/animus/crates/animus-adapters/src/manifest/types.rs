type Text = super::Text;
type Collection<T> = super::Collection<T>;
type PluginCompatibility = super::PluginCompatibility;
type PluginCompatibilityDraft<'a> = super::PluginCompatibilityDraft<'a>;
type PluginRole = super::PluginRole;
type PluginRoleDraft<'a> = super::PluginRoleDraft<'a>;

/// Untrusted manifest-shaped data before descriptor validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginDescriptorDraft<'a> {
    /// Optional descriptor identity, required by validation.
    pub identity: Option<&'a str>,
    /// Optional descriptor version, required by validation.
    pub version: Option<&'a str>,
    /// Optional toolset-kind label, required by validation.
    pub toolset_kind: Option<&'a str>,
    /// Optional runtime-kind label, required by validation.
    pub runtime_kind: Option<&'a str>,
    /// Safe provenance label, required by validation.
    pub provenance: Option<&'a str>,
    /// Host trust status, required by validation.
    pub trust_status: Option<&'a str>,
    /// Declared tool descriptors owned by this manifest.
    pub declared_tools: &'a [ToolDescriptorDraft<'a>],
    /// Data-only action, event-hook, pane-entrypoint, and link-handler roles.
    pub declared_roles: &'a [PluginRoleDraft<'a>],
    /// Capability labels requested by this manifest.
    pub capabilities: &'a [&'a str],
    /// Minimum host compatibility required by this manifest.
    pub compatibility: PluginCompatibilityDraft<'a>,
    /// Launch-policy metadata. Validation treats this as data only.
    pub launch_policy: LaunchPolicyDraft<'a>,
    /// Sandbox and network claims. Validation does not enforce a sandbox.
    pub sandbox: SandboxRequirementsDraft<'a>,
    /// Redaction claims for future host-owned logging.
    pub redaction: RedactionPolicyDraft,
}

/// Untrusted manifest-shaped tool declaration data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolDescriptorDraft<'a> {
    /// Optional tool name, required by validation.
    pub name: Option<&'a str>,
    /// Optional human-facing descriptor text.
    pub description: Option<&'a str>,
    /// Optional parameter schema identity, required by validation.
    pub parameter_schema_identity: Option<&'a str>,
    /// Optional result contract identity, required by validation.
    pub result_contract_identity: Option<&'a str>,
    /// Capability labels required by this tool.
    pub capability_labels: &'a [&'a str],
}

/// Untrusted launch metadata from a descriptor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LaunchPolicyDraft<'a> {
    /// Optional command or artifact label. This is never executed by validation.
    pub command_label: Option<&'a str>,
    /// Optional data-only argument labels.
    pub arguments: &'a [&'a str],
    /// Optional working-directory mode label.
    pub working_directory: Option<&'a str>,
    /// Environment variable names that a future host may choose to expose.
    pub environment_allowlist: &'a [&'a str],
    /// Project-relative writable-root descriptors.
    pub writable_roots: &'a [&'a str],
}

/// Untrusted sandbox and network claims from a descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxRequirementsDraft<'a> {
    /// Optional sandbox mode, required by validation.
    pub mode: Option<&'a str>,
    /// Optional network policy, required by validation.
    pub network: Option<&'a str>,
}

/// Redaction claims from a descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedactionPolicyDraft {
    /// Whether host logs should redact transcript payloads.
    pub redact_transcripts: bool,
    /// Whether host logs should redact secret-looking payloads.
    pub redact_secrets: bool,
}

/// Validated descriptor manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginDescriptor {
    pub(super) identity: Text,
    pub(super) version: Text,
    pub(super) toolset_kind: ToolsetKind,
    pub(super) runtime_kind: RuntimeKind,
    pub(super) provenance: Text,
    pub(super) trust_status: TrustStatus,
    pub(super) declared_tools: Collection<ToolDescriptor>,
    pub(super) declared_roles: Collection<PluginRole>,
    pub(super) capabilities: Collection<Text>,
    pub(super) compatibility: PluginCompatibility,
    pub(super) launch_policy: LaunchPolicy,
    pub(super) sandbox: SandboxRequirements,
    pub(super) redaction: RedactionPolicy,
}

/// Toolset-kind descriptor labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolsetKind {
    /// Generic host-owned toolset.
    Generic,
    /// Hermes review-gate-style toolset.
    ReviewGate,
}

/// Host trust status for a descriptor manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustStatus {
    /// Host policy trusts the descriptor for active tool exposure.
    Trusted,
    /// Host policy withholds the descriptor from active tool exposure.
    Untrusted,
}

/// Runtime-kind descriptor labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeKind {
    /// Host-owned implementation; no launch command is required.
    Host,
    /// Stdio-shaped descriptor; validation records a command label but does not launch it.
    Stdio,
    /// Wasm-shaped descriptor; validation records an artifact label but does not load it.
    Wasm,
}

/// Validated tool declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDescriptor {
    pub(super) name: Text,
    pub(super) description: Text,
    pub(super) parameter_schema_identity: Text,
    pub(super) result_contract_identity: Text,
    pub(super) capability_labels: Collection<Text>,
}

/// Working-directory descriptor modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkingDirectoryMode {
    /// Future hosts may interpret paths relative to the project root.
    ProjectRoot,
    /// Future hosts may interpret paths relative to the descriptor location.
    ManifestDirectory,
}

/// Validated launch metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchPolicy {
    pub(super) command_label: Option<Text>,
    pub(super) arguments: Collection<Text>,
    pub(super) working_directory: WorkingDirectoryMode,
    pub(super) environment_allowlist: Collection<Text>,
    pub(super) writable_roots: Collection<Text>,
}

/// Sandbox-mode descriptor labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxMode {
    /// No sandbox claim is made by the descriptor.
    Unconfined,
    /// A future host must enforce isolation before running anything.
    Isolated,
}

/// Network-policy descriptor labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkPolicy {
    /// Network access is denied.
    Denied,
    /// Only loopback network access is claimed.
    Loopback,
    /// Outbound network access is claimed.
    Outbound,
}

/// Validated sandbox and network claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxRequirements {
    pub(super) mode: SandboxMode,
    pub(super) network: NetworkPolicy,
}

/// Validated redaction claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedactionPolicy {
    pub(super) redact_transcripts: bool,
    pub(super) redact_secrets: bool,
}
