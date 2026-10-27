use super::{Collection, Text};

/// Borrowed compatibility fields before manifest validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginCompatibilityDraft<'a> {
    /// Minimum host API or capability version required by the manifest.
    pub minimum_host_version: Option<&'a str>,
    /// Host capabilities that must be present before the manifest is available.
    pub required_host_capabilities: &'a [&'a str],
    /// Platform labels that may use the manifest.
    pub supported_platforms: &'a [&'a str],
}

/// Validated compatibility requirements for a manifest or role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginCompatibility {
    pub(super) minimum_host_version: Text,
    pub(super) required_host_capabilities: Collection<Text>,
    pub(super) supported_platforms: Collection<Text>,
}

/// Borrowed plugin role declaration before manifest validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginRoleDraft<'a> {
    /// Role kind label: action, event-hook, host-pane-entrypoint, or link-handler.
    pub kind: Option<&'a str>,
    /// Stable role ID used by host-owned registry planning.
    pub id: Option<&'a str>,
    /// Human-facing role title.
    pub title: Option<&'a str>,
    /// Safe context labels required by this role.
    pub context_labels: &'a [&'a str],
    /// Data-only command label for a future host executor.
    pub command_label: Option<&'a str>,
    /// Data-only artifact label for a future host executor.
    pub artifact_label: Option<&'a str>,
    /// Event label for event-hook roles.
    pub event_name: Option<&'a str>,
    /// Optional link scheme for link-handler roles.
    pub link_scheme: Option<&'a str>,
    /// Role-level compatibility overrides; omitted values inherit from the manifest.
    pub compatibility: PluginCompatibilityDraft<'a>,
}

/// Validated manifest role kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginRoleKind {
    /// User-visible workflow action declaration.
    Action,
    /// Host event-hook declaration.
    EventHook,
    /// Optional host-pane entrypoint declaration.
    HostPaneEntrypoint,
    /// Link-handler declaration.
    LinkHandler,
}

/// Validated data-only plugin role declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginRole {
    pub(super) kind: PluginRoleKind,
    pub(super) id: Text,
    pub(super) title: Text,
    pub(super) context_labels: Collection<Text>,
    pub(super) command_label: Option<Text>,
    pub(super) artifact_label: Option<Text>,
    pub(super) event_name: Option<Text>,
    pub(super) link_scheme: Option<Text>,
    pub(super) compatibility: PluginCompatibility,
}

/// Borrowed host compatibility facts supplied by the imperative shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginHostCompatibilityDraft<'a> {
    /// Current host API or capability version.
    pub host_version: Option<&'a str>,
    /// Capabilities available in the current host.
    pub capabilities: &'a [&'a str],
    /// Current host platform label.
    pub platform: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PluginHostCompatibility {
    pub(super) host_version: Text,
    pub(super) capabilities: Collection<Text>,
    pub(super) platform: Text,
}

/// Borrowed invocation context fields before redaction validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginInvocationContextDraft<'a> {
    /// Safe workspace label.
    pub workspace_label: Option<&'a str>,
    /// Safe tab label.
    pub tab_label: Option<&'a str>,
    /// Safe pane label.
    pub pane_label: Option<&'a str>,
    /// Safe agent status label.
    pub agent_status_label: Option<&'a str>,
    /// Safe worktree/session provenance label for workspace state.
    pub worktree_provenance_label: Option<&'a str>,
    /// Safe session provenance label.
    pub session_provenance_label: Option<&'a str>,
    /// Safe request identity.
    pub request_identity: Option<&'a str>,
    /// Safe plugin root label, never a full private path.
    pub plugin_root_label: Option<&'a str>,
    /// Safe plugin config label, never config contents.
    pub plugin_config_label: Option<&'a str>,
    /// Safe plugin state label, never state contents.
    pub plugin_state_label: Option<&'a str>,
    /// Safe source metadata label.
    pub source_metadata_label: Option<&'a str>,
}

/// Invocation context leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginInvocationContextLeak {
    /// Raw prompt text was supplied.
    RawPrompt,
    /// Raw transcript text was supplied.
    Transcript,
    /// Credential-looking text was supplied.
    Credential,
    /// Full private path text was supplied.
    FullPrivatePath,
    /// Unbounded log text was supplied.
    UnboundedLog,
    /// Provider payload text was supplied.
    ProviderPayload,
}

/// Validated safe invocation context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginInvocationContext {
    pub(super) workspace_label: Text,
    pub(super) tab_label: Text,
    pub(super) pane_label: Text,
    pub(super) agent_status_label: Text,
    pub(super) worktree_provenance_label: Text,
    pub(super) session_provenance_label: Text,
    pub(super) request_identity: Text,
    pub(super) plugin_root_label: Text,
    pub(super) plugin_config_label: Text,
    pub(super) plugin_state_label: Text,
    pub(super) source_metadata_label: Text,
}

/// Host-owned registry operation to plan as data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginRegistryOperationKind {
    /// Link a manifest into a host-owned registry.
    Link,
    /// Unlink a manifest from a host-owned registry.
    Unlink,
    /// Enable a linked manifest in host-owned state.
    Enable,
    /// Disable a linked manifest in host-owned state.
    Disable,
    /// List linked manifests.
    List,
    /// List available action roles.
    ActionList,
    /// Plan an action or link-handler invocation.
    PlanInvocation,
    /// Plan an event-hook invocation.
    PlanEvent,
    /// Plan opening a host-pane entrypoint.
    PlanPaneEntrypoint,
}

/// Borrowed planning request for host-owned registry operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginRegistryOperationDraft<'a> {
    /// Operation kind to plan.
    pub operation: PluginRegistryOperationKind,
    /// Optional role ID targeted by invocation or pane planning.
    pub entrypoint_id: Option<&'a str>,
    /// Optional event name targeted by event planning or link validation.
    pub event_name: Option<&'a str>,
    /// Whether host-owned registry state currently enables the plugin.
    pub plugin_enabled: bool,
    /// Whether host policy currently allows the requested operation.
    pub policy_allows: bool,
    /// Previously validated safe invocation context when planning dispatch-like operations.
    pub context: Option<&'a PluginInvocationContext>,
    /// Current host compatibility facts.
    pub host: PluginHostCompatibilityDraft<'a>,
}

/// Diagnostic severity for plugin manifest planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginDiagnosticSeverity {
    /// Non-blocking warning retained in the plan.
    Warning,
    /// Blocking failure that prevents host dispatch.
    Failure,
}

/// Diagnostic class for plugin manifest planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginDiagnosticKind {
    /// Link-time warning for forward-compatible data.
    LinkWarning,
    /// Unknown event warning.
    UnknownEvent,
    /// Optional metadata was incomplete.
    MissingOptionalMetadata,
    /// Plugin is disabled in host-owned registry state.
    DisabledPlugin,
    /// The requested role or entrypoint ID is unknown.
    UnknownEntrypoint,
    /// Host policy denied the operation.
    PolicyDenied,
    /// Invocation context was missing or malformed.
    MalformedContext,
    /// Event dispatch was unsupported.
    UnsupportedEvent,
    /// Host platform is unsupported.
    UnsupportedPlatform,
    /// Host version is older than the manifest requires.
    NewerHostRequired,
    /// Required host capability is missing.
    MissingHostCapability,
    /// Manifest data is malformed for the requested operation.
    MalformedManifest,
}

/// Typed plugin manifest planning diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginDiagnostic {
    pub(super) kind: PluginDiagnosticKind,
    pub(super) severity: PluginDiagnosticSeverity,
}

/// Status of a host-owned plugin registry plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginRegistryPlanStatus {
    /// The requested operation is available as a data-only plan.
    Planned,
    /// The requested operation is available with retained warnings.
    Warning,
    /// The requested operation is blocked with failure diagnostics.
    Blocked,
}

/// Data-only plan for a host-owned plugin registry operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginRegistryPlan {
    pub(super) operation: PluginRegistryOperationKind,
    pub(super) manifest_identity: Text,
    pub(super) entrypoint_id: Option<Text>,
    pub(super) status: PluginRegistryPlanStatus,
    pub(super) diagnostics: Collection<PluginDiagnostic>,
    pub(super) command_label: Option<Text>,
    pub(super) artifact_label: Option<Text>,
}
