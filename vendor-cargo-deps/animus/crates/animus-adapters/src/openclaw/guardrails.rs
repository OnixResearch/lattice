//! OpenClaw-inspired security audit and plugin guardrail DTOs.
//!
//! Guardrails consume host-supplied snapshots and manifest/provenance metadata.
//! Reusable crates classify risks and validate safe receipts only; hosts own
//! config loading, filesystem scanning, package installation, plugin loading,
//! daemon restarts, Doctor UI, and auto-fix execution.

const SNAPSHOT_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_snapshot_id");
const SAFE_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_safe_label");
const REMEDIATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_remediation_label");
const AUTOFIX_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_autofix_label");
const PLUGIN_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_plugin_id");
const CHANNEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_channel_id");
const PROVIDER_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_provider_id");
const SLOT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_slot_id");
const SKILL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_skill_id");
const UI_HINT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_ui_hint");
const RECEIPT_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_receipt_summary");
const SOURCE_CLASS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_source_class");
const PROVENANCE_CLASS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("guardrail_provenance_class");
const LEAK_RAW_CONFIG: &str = "RAW_CONFIG:";
const LEAK_TOKEN: &str = "TOKEN=";
const LEAK_CREDENTIAL: &str = "CREDENTIAL:";
const LEAK_FULL_PATH: &str = "/home/";
const LEAK_PROVIDER_PAYLOAD: &str = "PROVIDER_PAYLOAD:";
const LEAK_RAW_PROMPT: &str = "RAW_PROMPT:";
const LEAK_PLUGIN_CODE: &str = "PLUGIN_CODE:";
const LEAK_UNBOUNDED_LOG: &str = "UNBOUNDED_LOG:";
const UNSAFE_TEXT_MARKERS: &[&str] = &[
    LEAK_RAW_CONFIG,
    LEAK_TOKEN,
    LEAK_CREDENTIAL,
    LEAK_FULL_PATH,
    LEAK_PROVIDER_PAYLOAD,
    LEAK_RAW_PROMPT,
    LEAK_PLUGIN_CODE,
    LEAK_UNBOUNDED_LOG,
];

/// Guardrail validation or classification error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Generic adapter field validation failed.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 identity failed validation.
    Artifact(crate::ArtifactValidationError),
    /// Reusable validation was asked to inspect host state directly.
    AmbientHostInspection,
    /// A required collection was empty.
    EmptyCollection {
        /// Field that contained no items.
        field: &'static str,
    },
    /// Plugin manifest was missing or untrusted for an enabled plugin.
    MissingManifest,
    /// Plugin code execution was requested before manifest validation.
    PluginExecutionRequested,
    /// Required plugin config schema is absent.
    MissingConfigSchema,
    /// Host config references an undeclared channel.
    UnknownChannel,
    /// Host config references an undeclared provider.
    UnknownProvider,
    /// Host config references an undeclared slot.
    UnknownSlot,
    /// Plugin policy denied admission.
    PluginDenied,
    /// Plugin path summary reports a symlink escape.
    PathEscape,
    /// Plugin root writability is unsafe.
    WorldWritableRoot,
    /// Plugin ownership summary is suspicious.
    SuspiciousOwnership,
    /// Registry-only install claim is unsupported by supplied provenance.
    RegistryOnlyClaimMismatch,
    /// Plugin lifecycle scripts are risky for admission.
    LifecycleScriptRisk,
    /// Text contained raw config, credentials, paths, provider payloads, prompts, plugin code, or unbounded logs.
    UnsafePayload {
        /// Field that contained unsafe text.
        field: &'static str,
        /// Marker that caused rejection.
        marker: &'static str,
    },
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<crate::ArtifactValidationError> for Error {
    fn from(error: crate::ArtifactValidationError) -> Self {
        Self::Artifact(error)
    }
}

/// Gateway bind posture supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindPosture {
    /// Local-only bind posture.
    LocalOnly,
    /// Remote-accessible bind posture.
    RemoteBound,
    /// Public internet exposure.
    PublicInternet,
}

/// Admission/auth posture supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionPosture {
    /// Admission is required and enforced.
    Required,
    /// Admission is optional.
    Optional,
    /// Admission is missing.
    Missing,
}

/// Channel or direct-message access posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelPosture {
    /// Private direct access only.
    PrivateOnly,
    /// Open direct messages.
    OpenDirectMessages,
    /// Open group or public channel access.
    OpenGroup,
}

/// Tool blast-radius profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolProfile {
    /// Read-only tools only.
    ReadOnly,
    /// Filesystem tools are available.
    Filesystem,
    /// Runtime or process tools are available.
    Runtime,
    /// Elevated tools are available.
    Elevated,
}

impl ToolProfile {
    const fn is_broad(self) -> bool {
        matches!(self, Self::Filesystem | Self::Runtime | Self::Elevated)
    }
}

/// Sandbox enforcement posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxPosture {
    /// Sandbox is enforced by the selected shell/platform.
    Enforced,
    /// Config says sandboxed, but enforcement is absent or drifted.
    ConfiguredNotEnforced,
    /// Sandbox is disabled.
    Disabled,
}

impl SandboxPosture {
    const fn is_enforced(self) -> bool {
        matches!(self, Self::Enforced)
    }
}

/// Plugin admission policy posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginPolicyPosture {
    /// Strict allowlist admission.
    Strict,
    /// Prompt or review admission.
    ReviewRequired,
    /// Permissive plugin admission.
    Permissive,
}

/// Model/tool risk compatibility posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRiskPosture {
    /// Low model risk.
    Low,
    /// Elevated model risk.
    Elevated,
}

/// Guardrail finding ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FindingId {
    /// Remote exposure lacks admission/auth.
    RemoteExposureWithoutAdmission,
    /// Open messaging context has broad tools.
    OpenMessagingWithBroadTools,
    /// Sandbox configuration drift.
    SandboxDrift,
    /// Plugin policy is permissive.
    PermissivePluginPolicy,
    /// Model/tool risk mismatch.
    ModelToolRiskMismatch,
    /// Unknown channel reference.
    UnknownChannel,
    /// Unsafe plugin path.
    UnsafePluginPath,
    /// Plugin lifecycle script risk.
    LifecycleScriptRisk,
    /// Receipt leak fixture.
    ReceiptLeak,
}

impl FindingId {
    /// Returns stable finding label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RemoteExposureWithoutAdmission => "remote_exposure_without_admission",
            Self::OpenMessagingWithBroadTools => "open_messaging_with_broad_tools",
            Self::SandboxDrift => "sandbox_drift",
            Self::PermissivePluginPolicy => "permissive_plugin_policy",
            Self::ModelToolRiskMismatch => "model_tool_risk_mismatch",
            Self::UnknownChannel => "unknown_channel",
            Self::UnsafePluginPath => "unsafe_plugin_path",
            Self::LifecycleScriptRisk => "lifecycle_script_risk",
            Self::ReceiptLeak => "receipt_leak",
        }
    }
}

/// Finding severity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    /// Informational finding.
    Info,
    /// Warning finding.
    Warning,
    /// High severity finding.
    High,
    /// Critical severity finding.
    Critical,
}

/// Optional autofix plan descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutofixDescriptor {
    label: alloc::string::String,
}

impl AutofixDescriptor {
    /// Builds a safe autofix descriptor label without executing it.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the label is missing or unsafe.
    pub fn new(label: &str) -> Result<Self, Error> {
        Ok(Self {
            label: safe_text(AUTOFIX_FIELD, label)?,
        })
    }

    /// Returns the safe autofix label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
}

/// Deterministic guardrail finding.
// r[impl agent_harness_runtime.openclaw_guardrails.audit]
// r[impl agent_harness_runtime.openclaw_guardrails.exposure]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailFinding {
    id: FindingId,
    severity: Severity,
    affected_labels: alloc::vec::Vec<alloc::string::String>,
    remediation_labels: alloc::vec::Vec<alloc::string::String>,
    evidence_hash: crate::ArtifactHash,
    autofix: Option<AutofixDescriptor>,
}

impl GuardrailFinding {
    /// Returns finding ID.
    #[must_use]
    pub const fn id(&self) -> FindingId {
        self.id
    }

    /// Returns severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns affected safe labels.
    #[must_use]
    pub fn affected_labels(&self) -> &[alloc::string::String] {
        self.affected_labels.as_slice()
    }

    /// Returns remediation labels.
    #[must_use]
    pub fn remediation_labels(&self) -> &[alloc::string::String] {
        self.remediation_labels.as_slice()
    }

    /// Returns evidence identity.
    #[must_use]
    pub const fn evidence_hash(&self) -> &crate::ArtifactHash {
        &self.evidence_hash
    }

    /// Returns optional autofix descriptor.
    #[must_use]
    pub const fn autofix(&self) -> Option<&AutofixDescriptor> {
        self.autofix.as_ref()
    }
}

/// Borrowed guardrail audit snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditSnapshotDraft<'a> {
    /// Stable snapshot identity label.
    pub snapshot_id: &'a str,
    /// Gateway bind posture.
    pub bind_posture: BindPosture,
    /// Admission posture.
    pub admission_posture: AdmissionPosture,
    /// Channel posture.
    pub channel_posture: ChannelPosture,
    /// Tool profile.
    pub tool_profile: ToolProfile,
    /// Sandbox posture.
    pub sandbox_posture: SandboxPosture,
    /// Plugin policy posture.
    pub plugin_policy: PluginPolicyPosture,
    /// Model/tool risk posture.
    pub model_risk: ModelRiskPosture,
    /// BLAKE3 identity of host-held snapshot evidence.
    pub evidence_hash: &'a str,
    /// Whether reusable code was asked to inspect host config/files/network.
    pub ambient_host_inspection: bool,
}

/// Validated audit snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditSnapshot {
    snapshot_id: alloc::string::String,
    bind_posture: BindPosture,
    admission_posture: AdmissionPosture,
    channel_posture: ChannelPosture,
    tool_profile: ToolProfile,
    sandbox_posture: SandboxPosture,
    plugin_policy: PluginPolicyPosture,
    model_risk: ModelRiskPosture,
    evidence_hash: crate::ArtifactHash,
}

impl AuditSnapshot {
    /// Builds a host-supplied audit snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when snapshot identity/evidence is invalid or ambient host inspection is requested.
    pub fn new(draft: &AuditSnapshotDraft<'_>) -> Result<Self, Error> {
        if draft.ambient_host_inspection {
            return Err(Error::AmbientHostInspection);
        }
        Ok(Self {
            snapshot_id: safe_text(SNAPSHOT_ID_FIELD, draft.snapshot_id)?,
            bind_posture: draft.bind_posture,
            admission_posture: draft.admission_posture,
            channel_posture: draft.channel_posture,
            tool_profile: draft.tool_profile,
            sandbox_posture: draft.sandbox_posture,
            plugin_policy: draft.plugin_policy,
            model_risk: draft.model_risk,
            evidence_hash: crate::ArtifactHash::new(draft.evidence_hash)?,
        })
    }

    /// Returns snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        self.snapshot_id.as_str()
    }
}

/// Classifies guardrail findings over a host-supplied snapshot.
///
/// # Errors
///
/// Returns [`Error`] only when safe labels or hashes fail validation.
///
/// # Panics
///
/// Panics if a previously validated snapshot loses its identity or evidence hash.
pub fn classify_audit(
    snapshot: &AuditSnapshot,
) -> Result<alloc::vec::Vec<GuardrailFinding>, Error> {
    assert!(!snapshot.snapshot_id().is_empty());
    assert!(!snapshot.evidence_hash.as_str().is_empty());
    let mut findings = alloc::vec::Vec::new();
    if matches!(
        snapshot.bind_posture,
        BindPosture::RemoteBound | BindPosture::PublicInternet
    ) && snapshot.admission_posture != AdmissionPosture::Required
    {
        findings.push(finding(&FindingDraft {
            id: FindingId::RemoteExposureWithoutAdmission,
            severity: Severity::Critical,
            snapshot,
            affected_label: "remote_bind",
            remediation_label: "require_admission",
            autofix_label: Some("disable_remote_or_require_auth"),
        })?);
    }
    if matches!(
        snapshot.channel_posture,
        ChannelPosture::OpenDirectMessages | ChannelPosture::OpenGroup
    ) && snapshot.tool_profile.is_broad()
        && !snapshot.sandbox_posture.is_enforced()
    {
        findings.push(finding(&FindingDraft {
            id: FindingId::OpenMessagingWithBroadTools,
            severity: Severity::High,
            snapshot,
            affected_label: "open_messaging_tools",
            remediation_label: "restrict_tools_or_enforce_sandbox",
            autofix_label: None,
        })?);
    }
    if snapshot.sandbox_posture == SandboxPosture::ConfiguredNotEnforced {
        findings.push(finding(&FindingDraft {
            id: FindingId::SandboxDrift,
            severity: Severity::High,
            snapshot,
            affected_label: "sandbox_drift",
            remediation_label: "repair_sandbox_enforcement",
            autofix_label: None,
        })?);
    }
    if snapshot.plugin_policy == PluginPolicyPosture::Permissive {
        findings.push(finding(&FindingDraft {
            id: FindingId::PermissivePluginPolicy,
            severity: Severity::Warning,
            snapshot,
            affected_label: "plugin_policy",
            remediation_label: "require_plugin_allowlist",
            autofix_label: None,
        })?);
    }
    if snapshot.model_risk == ModelRiskPosture::Elevated && snapshot.tool_profile.is_broad() {
        findings.push(finding(&FindingDraft {
            id: FindingId::ModelToolRiskMismatch,
            severity: Severity::High,
            snapshot,
            affected_label: "model_tool_risk",
            remediation_label: "downgrade_tools_or_route_model",
            autofix_label: None,
        })?);
    }
    Ok(findings)
}

/// Plugin manifest diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginDiagnostic {
    /// Plugin is disabled and should not be executed.
    DisabledPlugin,
}

/// Borrowed plugin manifest guardrail input.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginManifestDraft<'a> {
    /// Plugin identity.
    pub plugin_id: &'a str,
    /// Whether the manifest is trusted.
    pub trusted: bool,
    /// Whether plugin is enabled.
    pub enabled: bool,
    /// Whether a config schema is present before execution.
    pub has_config_schema: bool,
    /// Declared channel IDs.
    pub declared_channels: &'a [&'a str],
    /// Host config channel IDs.
    pub config_channels: &'a [&'a str],
    /// Declared provider IDs.
    pub declared_providers: &'a [&'a str],
    /// Host config provider IDs.
    pub config_providers: &'a [&'a str],
    /// Declared slot IDs.
    pub slots: &'a [&'a str],
    /// Host config slot IDs.
    pub config_slots: &'a [&'a str],
    /// Declared skill IDs.
    pub skills: &'a [&'a str],
    /// Safe UI hint labels.
    pub ui_hints: &'a [&'a str],
    /// Whether plugin code execution was requested before validation.
    pub execution_requested: bool,
}

/// Validated plugin manifest guardrail result.
// r[impl agent_harness_runtime.openclaw_guardrails.plugin_manifest]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginManifestValidation {
    plugin_id: alloc::string::String,
    enabled: bool,
    diagnostics: alloc::vec::Vec<PluginDiagnostic>,
}

impl PluginManifestValidation {
    /// Returns plugin identity.
    #[must_use]
    pub fn plugin_id(&self) -> &str {
        self.plugin_id.as_str()
    }

    /// Returns enabled state.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns manifest diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[PluginDiagnostic] {
        self.diagnostics.as_slice()
    }
}

/// Validates plugin manifest data before code execution.
///
/// # Errors
///
/// Returns [`Error`] when manifest data is missing, unknown, unsafe, or tries to execute plugin code.
///
/// # Panics
///
/// Panics if plugin execution or config-schema post-conditions are violated after validation.
pub fn validate_plugin_manifest(
    draft: &PluginManifestDraft<'_>,
) -> Result<PluginManifestValidation, Error> {
    if draft.execution_requested {
        return Err(Error::PluginExecutionRequested);
    }
    if !draft.trusted && draft.enabled {
        return Err(Error::MissingManifest);
    }
    if !draft.has_config_schema {
        return Err(Error::MissingConfigSchema);
    }
    assert!(!draft.execution_requested);
    assert!(draft.has_config_schema);
    let plugin_id = safe_text(PLUGIN_ID_FIELD, draft.plugin_id)?;
    validate_labels(CHANNEL_FIELD, draft.declared_channels)?;
    validate_labels(PROVIDER_FIELD, draft.declared_providers)?;
    validate_labels(SLOT_FIELD, draft.slots)?;
    validate_labels(SKILL_FIELD, draft.skills)?;
    validate_labels(UI_HINT_FIELD, draft.ui_hints)?;
    reject_unknown(
        CHANNEL_FIELD,
        draft.config_channels,
        draft.declared_channels,
        &Error::UnknownChannel,
    )?;
    reject_unknown(
        PROVIDER_FIELD,
        draft.config_providers,
        draft.declared_providers,
        &Error::UnknownProvider,
    )?;
    reject_unknown(
        SLOT_FIELD,
        draft.config_slots,
        draft.slots,
        &Error::UnknownSlot,
    )?;
    let mut diagnostics = alloc::vec::Vec::new();
    if !draft.enabled {
        diagnostics.push(PluginDiagnostic::DisabledPlugin);
    }
    Ok(PluginManifestValidation {
        plugin_id,
        enabled: draft.enabled,
        diagnostics,
    })
}

/// Plugin allow/deny policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginTrustPolicy {
    /// Plugin is allowed by host policy.
    Allow,
    /// Plugin is denied by host policy.
    Deny,
}

/// Install source class supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallSourceClass {
    /// Registry install source.
    Registry,
    /// Local path install source.
    LocalPath,
    /// Unknown install source.
    Unknown,
}

/// Path-safety class supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathSafetyClass {
    /// Path is within declared plugin root.
    InRoot,
    /// Path resolves outside declared plugin root.
    SymlinkEscape,
}

/// Writability class supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WritabilityClass {
    /// Not world-writable.
    NotWorldWritable,
    /// World-writable root or parent.
    WorldWritable,
}

/// Ownership class supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipClass {
    /// Expected owner summary.
    Expected,
    /// Suspicious owner summary.
    Suspicious,
}

/// Lifecycle script risk supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleScriptRisk {
    /// No lifecycle-script risk.
    None,
    /// Lifecycle scripts are present.
    Present,
}

/// Borrowed plugin trust summary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginTrustDraft<'a> {
    /// Plugin identity.
    pub plugin_id: &'a str,
    /// Allow/deny posture.
    pub policy: PluginTrustPolicy,
    /// Install source class.
    pub install_source: InstallSourceClass,
    /// Path safety class.
    pub path_safety: PathSafetyClass,
    /// Writability class.
    pub writability: WritabilityClass,
    /// Ownership class.
    pub ownership: OwnershipClass,
    /// Whether host config claims registry-only install.
    pub registry_only_claim: bool,
    /// Lifecycle script risk.
    pub lifecycle_script_risk: LifecycleScriptRisk,
    /// BLAKE3 identity of host-held provenance summary.
    pub provenance_hash: &'a str,
}

/// Validated plugin trust summary.
// r[impl agent_harness_runtime.openclaw_guardrails.plugin_trust]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginTrustValidation {
    plugin_id: alloc::string::String,
    provenance_hash: crate::ArtifactHash,
}

impl PluginTrustValidation {
    /// Returns plugin identity.
    #[must_use]
    pub fn plugin_id(&self) -> &str {
        self.plugin_id.as_str()
    }

    /// Returns provenance identity.
    #[must_use]
    pub const fn provenance_hash(&self) -> &crate::ArtifactHash {
        &self.provenance_hash
    }
}

/// Validates host-supplied plugin trust/path summaries without host inspection.
///
/// # Errors
///
/// Returns [`Error`] for denied policy, unsafe path, writability, ownership, registry-only, or lifecycle risk.
///
/// # Panics
///
/// Panics if allow/path-safety post-conditions are violated after validation.
pub fn validate_plugin_trust(draft: &PluginTrustDraft<'_>) -> Result<PluginTrustValidation, Error> {
    if draft.policy == PluginTrustPolicy::Deny {
        return Err(Error::PluginDenied);
    }
    if draft.path_safety == PathSafetyClass::SymlinkEscape {
        return Err(Error::PathEscape);
    }
    if draft.writability == WritabilityClass::WorldWritable {
        return Err(Error::WorldWritableRoot);
    }
    if draft.ownership == OwnershipClass::Suspicious {
        return Err(Error::SuspiciousOwnership);
    }
    if draft.registry_only_claim && draft.install_source != InstallSourceClass::Registry {
        return Err(Error::RegistryOnlyClaimMismatch);
    }
    if draft.lifecycle_script_risk == LifecycleScriptRisk::Present {
        return Err(Error::LifecycleScriptRisk);
    }
    assert_eq!(draft.policy, PluginTrustPolicy::Allow);
    assert_ne!(draft.path_safety, PathSafetyClass::SymlinkEscape);
    Ok(PluginTrustValidation {
        plugin_id: safe_text(PLUGIN_ID_FIELD, draft.plugin_id)?,
        provenance_hash: crate::ArtifactHash::new(draft.provenance_hash)?,
    })
}

/// Borrowed safe guardrail receipt before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuardrailReceiptDraft<'a> {
    /// Finding ID.
    pub finding_id: FindingId,
    /// Finding severity.
    pub severity: Severity,
    /// Source class label.
    pub source_class: &'a str,
    /// Affected safe label.
    pub affected_label: &'a str,
    /// Remediation label.
    pub remediation_label: &'a str,
    /// BLAKE3 policy identity.
    pub policy_hash: &'a str,
    /// Optional plugin identity.
    pub plugin_id: Option<&'a str>,
    /// Provenance class label.
    pub provenance_class: &'a str,
    /// Path-safety class.
    pub path_safety: PathSafetyClass,
    /// BLAKE3 receipt identity.
    pub receipt_hash: &'a str,
    /// Safe receipt summary.
    pub summary: &'a str,
}

/// Safe guardrail receipt metadata.
// r[impl agent_harness_runtime.openclaw_guardrails.receipts]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardrailReceipt {
    finding_id: FindingId,
    severity: Severity,
    source_class: alloc::string::String,
    affected_label: alloc::string::String,
    remediation_label: alloc::string::String,
    policy_hash: crate::ArtifactHash,
    plugin_id: Option<alloc::string::String>,
    provenance_class: alloc::string::String,
    path_safety: PathSafetyClass,
    receipt_hash: crate::ArtifactHash,
    summary: alloc::string::String,
}

impl GuardrailReceipt {
    /// Builds a safe guardrail receipt.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when metadata is malformed or unsafe.
    pub fn new(draft: &GuardrailReceiptDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            finding_id: draft.finding_id,
            severity: draft.severity,
            source_class: safe_text(SOURCE_CLASS_FIELD, draft.source_class)?,
            affected_label: safe_text(SAFE_LABEL_FIELD, draft.affected_label)?,
            remediation_label: safe_text(REMEDIATION_FIELD, draft.remediation_label)?,
            policy_hash: crate::ArtifactHash::new(draft.policy_hash)?,
            plugin_id: draft
                .plugin_id
                .map(|plugin_id| safe_text(PLUGIN_ID_FIELD, plugin_id))
                .transpose()?,
            provenance_class: safe_text(PROVENANCE_CLASS_FIELD, draft.provenance_class)?,
            path_safety: draft.path_safety,
            receipt_hash: crate::ArtifactHash::new(draft.receipt_hash)?,
            summary: safe_text(RECEIPT_SUMMARY_FIELD, draft.summary)?,
        })
    }

    /// Returns finding ID.
    #[must_use]
    pub const fn finding_id(&self) -> FindingId {
        self.finding_id
    }

    /// Returns severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns source class.
    #[must_use]
    pub fn source_class(&self) -> &str {
        self.source_class.as_str()
    }

    /// Returns affected label.
    #[must_use]
    pub fn affected_label(&self) -> &str {
        self.affected_label.as_str()
    }

    /// Returns remediation label.
    #[must_use]
    pub fn remediation_label(&self) -> &str {
        self.remediation_label.as_str()
    }

    /// Returns policy identity.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::ArtifactHash {
        &self.policy_hash
    }

    /// Returns optional plugin identity.
    #[must_use]
    pub fn plugin_id(&self) -> Option<&str> {
        self.plugin_id.as_deref()
    }

    /// Returns provenance class.
    #[must_use]
    pub fn provenance_class(&self) -> &str {
        self.provenance_class.as_str()
    }

    /// Returns path-safety class.
    #[must_use]
    pub const fn path_safety(&self) -> PathSafetyClass {
        self.path_safety
    }

    /// Returns receipt identity.
    #[must_use]
    pub const fn receipt_hash(&self) -> &crate::ArtifactHash {
        &self.receipt_hash
    }

    /// Returns safe summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

struct FindingDraft<'a> {
    id: FindingId,
    severity: Severity,
    snapshot: &'a AuditSnapshot,
    affected_label: &'a str,
    remediation_label: &'a str,
    autofix_label: Option<&'a str>,
}

fn finding(draft: &FindingDraft<'_>) -> Result<GuardrailFinding, Error> {
    Ok(GuardrailFinding {
        id: draft.id,
        severity: draft.severity,
        affected_labels: alloc::vec![safe_text(SAFE_LABEL_FIELD, draft.affected_label)?],
        remediation_labels: alloc::vec![safe_text(REMEDIATION_FIELD, draft.remediation_label)?],
        evidence_hash: draft.snapshot.evidence_hash.clone(),
        autofix: draft
            .autofix_label
            .map(AutofixDescriptor::new)
            .transpose()?,
    })
}

fn validate_labels(field_name: crate::validation::FieldName, values: &[&str]) -> Result<(), Error> {
    for value in values {
        safe_text(field_name, value)?;
    }
    Ok(())
}

fn reject_unknown(
    field_name: crate::validation::FieldName,
    requested: &[&str],
    declared: &[&str],
    error: &Error,
) -> Result<(), Error> {
    for value in requested {
        safe_text(field_name, value)?;
        if !declared.contains(value) {
            return Err(error.clone());
        }
    }
    Ok(())
}

fn safe_text(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<alloc::string::String, Error> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    reject_unsafe_text(field_name, normalized.as_str())?;
    Ok(normalized)
}

fn reject_unsafe_text(field_name: crate::validation::FieldName, value: &str) -> Result<(), Error> {
    let lower = value.to_ascii_lowercase();
    for marker in UNSAFE_TEXT_MARKERS {
        if lower.contains(&marker.to_ascii_lowercase()) {
            return Err(Error::UnsafePayload {
                field: field_name.as_str(),
                marker,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNAPSHOT_ID: &str = "snapshot-main";
    const PLUGIN_ID: &str = "plugin-main";
    const CHANNEL_ID: &str = "channel-main";
    const PROVIDER_ID: &str = "provider-main";
    const SLOT_ID: &str = "slot-main";
    const SKILL_ID: &str = "skill-main";
    const UI_HINT: &str = "safe-ui-hint";
    const SAFE_SOURCE: &str = "audit-snapshot";
    const SAFE_AFFECTED: &str = "gateway-bind";
    const SAFE_REMEDIATION: &str = "require-admission";
    const SAFE_PROVENANCE: &str = "host-supplied-summary";
    const SAFE_SUMMARY: &str = "safe guardrail receipt";
    const UNSAFE_SUMMARY: &str = "TOKEN=raw-secret";
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn hardened_baseline_valid_plugin_disabled_warning_and_safe_receipt_pass() {
        // r[verify agent_harness_runtime.openclaw_guardrails.audit]
        // r[verify agent_harness_runtime.openclaw_guardrails.plugin_manifest]
        // r[verify agent_harness_runtime.openclaw_guardrails.plugin_trust]
        // r[verify agent_harness_runtime.openclaw_guardrails.receipts]
        // r[verify agent_harness_runtime.openclaw_guardrails.verification]
        let snapshot = AuditSnapshot::new(&AuditSnapshotDraft {
            snapshot_id: SNAPSHOT_ID,
            bind_posture: BindPosture::LocalOnly,
            admission_posture: AdmissionPosture::Required,
            channel_posture: ChannelPosture::PrivateOnly,
            tool_profile: ToolProfile::ReadOnly,
            sandbox_posture: SandboxPosture::Enforced,
            plugin_policy: PluginPolicyPosture::Strict,
            model_risk: ModelRiskPosture::Low,
            evidence_hash: HASH_A,
            ambient_host_inspection: false,
        })
        .expect("snapshot should validate");
        let findings = classify_audit(&snapshot).expect("audit should classify");
        let disabled = validate_plugin_manifest(&PluginManifestDraft {
            enabled: false,
            ..valid_manifest()
        })
        .expect("disabled trusted plugin should validate with warning");
        let manifest =
            validate_plugin_manifest(&valid_manifest()).expect("manifest should validate");
        let trust = validate_plugin_trust(&valid_trust()).expect("trust should validate");
        let receipt = GuardrailReceipt::new(&valid_receipt()).expect("receipt should validate");

        assert!(findings.is_empty());
        assert!(
            disabled
                .diagnostics()
                .contains(&PluginDiagnostic::DisabledPlugin)
        );
        assert_eq!(manifest.plugin_id(), PLUGIN_ID);
        assert_eq!(trust.plugin_id(), PLUGIN_ID);
        assert_eq!(
            receipt.finding_id(),
            FindingId::RemoteExposureWithoutAdmission
        );
    }

    #[test]
    fn exposure_sandbox_drift_and_model_tool_mismatch_emit_findings() {
        // r[verify agent_harness_runtime.openclaw_guardrails.exposure]
        // r[verify agent_harness_runtime.openclaw_guardrails.verification]
        let snapshot = AuditSnapshot::new(&AuditSnapshotDraft {
            snapshot_id: SNAPSHOT_ID,
            bind_posture: BindPosture::PublicInternet,
            admission_posture: AdmissionPosture::Missing,
            channel_posture: ChannelPosture::OpenGroup,
            tool_profile: ToolProfile::Runtime,
            sandbox_posture: SandboxPosture::ConfiguredNotEnforced,
            plugin_policy: PluginPolicyPosture::Permissive,
            model_risk: ModelRiskPosture::Elevated,
            evidence_hash: HASH_A,
            ambient_host_inspection: false,
        })
        .expect("risky snapshot should validate");
        let findings = classify_audit(&snapshot).expect("findings should classify");

        assert!(findings.iter().any(|finding| {
            finding.id() == FindingId::RemoteExposureWithoutAdmission
                && finding.severity() == Severity::Critical
        }));
        assert!(
            findings
                .iter()
                .any(|finding| finding.id() == FindingId::OpenMessagingWithBroadTools)
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.id() == FindingId::SandboxDrift)
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.id() == FindingId::ModelToolRiskMismatch)
        );
    }

    #[test]
    fn unknown_manifest_path_execution_ambient_and_receipt_leaks_fail_closed() {
        // r[verify agent_harness_runtime.openclaw_guardrails.audit]
        // r[verify agent_harness_runtime.openclaw_guardrails.plugin_manifest]
        // r[verify agent_harness_runtime.openclaw_guardrails.plugin_trust]
        // r[verify agent_harness_runtime.openclaw_guardrails.receipts]
        // r[verify agent_harness_runtime.openclaw_guardrails.verification]
        let ambient = AuditSnapshot::new(&AuditSnapshotDraft {
            ambient_host_inspection: true,
            ..safe_snapshot()
        })
        .expect_err("ambient host inspection should fail");
        let unknown_channel = validate_plugin_manifest(&PluginManifestDraft {
            config_channels: &["missing-channel"],
            ..valid_manifest()
        })
        .expect_err("unknown channel should fail");
        let execution = validate_plugin_manifest(&PluginManifestDraft {
            execution_requested: true,
            ..valid_manifest()
        })
        .expect_err("plugin execution before validation should fail");
        let missing_schema = validate_plugin_manifest(&PluginManifestDraft {
            has_config_schema: false,
            ..valid_manifest()
        })
        .expect_err("missing schema should fail");
        let path_escape = validate_plugin_trust(&PluginTrustDraft {
            path_safety: PathSafetyClass::SymlinkEscape,
            ..valid_trust()
        })
        .expect_err("symlink escape summary should fail");
        let lifecycle = validate_plugin_trust(&PluginTrustDraft {
            lifecycle_script_risk: LifecycleScriptRisk::Present,
            ..valid_trust()
        })
        .expect_err("lifecycle scripts should fail");
        let unsafe_receipt = GuardrailReceipt::new(&GuardrailReceiptDraft {
            summary: UNSAFE_SUMMARY,
            ..valid_receipt()
        })
        .expect_err("receipt leak should fail");

        assert_eq!(ambient, Error::AmbientHostInspection);
        assert_eq!(unknown_channel, Error::UnknownChannel);
        assert_eq!(execution, Error::PluginExecutionRequested);
        assert_eq!(missing_schema, Error::MissingConfigSchema);
        assert_eq!(path_escape, Error::PathEscape);
        assert_eq!(lifecycle, Error::LifecycleScriptRisk);
        assert!(matches!(unsafe_receipt, Error::UnsafePayload { .. }));
    }

    const fn safe_snapshot() -> AuditSnapshotDraft<'static> {
        AuditSnapshotDraft {
            snapshot_id: SNAPSHOT_ID,
            bind_posture: BindPosture::LocalOnly,
            admission_posture: AdmissionPosture::Required,
            channel_posture: ChannelPosture::PrivateOnly,
            tool_profile: ToolProfile::ReadOnly,
            sandbox_posture: SandboxPosture::Enforced,
            plugin_policy: PluginPolicyPosture::Strict,
            model_risk: ModelRiskPosture::Low,
            evidence_hash: HASH_A,
            ambient_host_inspection: false,
        }
    }

    const fn valid_manifest() -> PluginManifestDraft<'static> {
        PluginManifestDraft {
            plugin_id: PLUGIN_ID,
            trusted: true,
            enabled: true,
            has_config_schema: true,
            declared_channels: &[CHANNEL_ID],
            config_channels: &[CHANNEL_ID],
            declared_providers: &[PROVIDER_ID],
            config_providers: &[PROVIDER_ID],
            slots: &[SLOT_ID],
            config_slots: &[SLOT_ID],
            skills: &[SKILL_ID],
            ui_hints: &[UI_HINT],
            execution_requested: false,
        }
    }

    const fn valid_trust() -> PluginTrustDraft<'static> {
        PluginTrustDraft {
            plugin_id: PLUGIN_ID,
            policy: PluginTrustPolicy::Allow,
            install_source: InstallSourceClass::Registry,
            path_safety: PathSafetyClass::InRoot,
            writability: WritabilityClass::NotWorldWritable,
            ownership: OwnershipClass::Expected,
            registry_only_claim: true,
            lifecycle_script_risk: LifecycleScriptRisk::None,
            provenance_hash: HASH_A,
        }
    }

    const fn valid_receipt() -> GuardrailReceiptDraft<'static> {
        GuardrailReceiptDraft {
            finding_id: FindingId::RemoteExposureWithoutAdmission,
            severity: Severity::Critical,
            source_class: SAFE_SOURCE,
            affected_label: SAFE_AFFECTED,
            remediation_label: SAFE_REMEDIATION,
            policy_hash: HASH_A,
            plugin_id: Some(PLUGIN_ID),
            provenance_class: SAFE_PROVENANCE,
            path_safety: PathSafetyClass::InRoot,
            receipt_hash: HASH_B,
            summary: SAFE_SUMMARY,
        }
    }
}
