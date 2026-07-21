//! Metadata-only model runtime profile and invocation receipts.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::composition::{ModelInvocationOmissionClass, ModelInvocationPlan};
use super::types::{
    MODEL_INVOCATION_RECEIPT_HASH_DOMAIN, ModelMemoryAccessMode, ModelProfileSourceScope,
    ModelProfileSpecificity, ModelProfileTrustStatus, ModelRuntimeProfileError,
    ModelRuntimeProfileIssue, NativeToolCallingMode, ResolvedModelRuntimeProfile,
};
use super::validation::{blake3_parts, error};

const FIELD_RECEIPT: &str = "model_invocation_receipt";
const REDACTION_MARKER: &str = "raw-model-context-omitted";

/// Forbidden durable material class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelInvocationReceiptLeak {
    /// Raw prompt body or hidden directive.
    RawPrompt,
    /// Full skill body.
    SkillBody,
    /// Raw memory body or generated memory candidate.
    MemoryBody,
    /// Skill script or asset body.
    ScriptOrAsset,
    /// Tool arguments.
    ToolArguments,
    /// Tool output.
    ToolOutput,
    /// Provider request or response payload.
    ProviderPayload,
    /// Credential or secret value.
    Credential,
    /// Environment value.
    EnvironmentValue,
    /// Private or full host path.
    FullPath,
    /// Endpoint body or private URL.
    Endpoint,
    /// Hidden reasoning.
    HiddenReasoning,
    /// Unbounded execution trace.
    UnboundedTrace,
}

/// Candidate sensitive fields that must all be absent from durable evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ModelInvocationReceiptSensitiveDraft<'a> {
    /// Raw prompt body or hidden directive.
    pub raw_prompt: Option<&'a str>,
    /// Full skill body.
    pub skill_body: Option<&'a str>,
    /// Raw memory body or generated memory candidate.
    pub memory_body: Option<&'a str>,
    /// Skill script or asset body.
    pub script_or_asset: Option<&'a str>,
    /// Tool arguments.
    pub tool_arguments: Option<&'a str>,
    /// Tool output.
    pub tool_output: Option<&'a str>,
    /// Provider request or response payload.
    pub provider_payload: Option<&'a str>,
    /// Credential or secret value.
    pub credential: Option<&'a str>,
    /// Environment value.
    pub environment_value: Option<&'a str>,
    /// Private or full host path.
    pub full_path: Option<&'a str>,
    /// Endpoint body or private URL.
    pub endpoint: Option<&'a str>,
    /// Hidden reasoning.
    pub hidden_reasoning: Option<&'a str>,
    /// Unbounded execution trace.
    pub unbounded_trace: Option<&'a str>,
}

/// Safe receipt entry for one advertised or selected skill.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInvocationSkillReceiptEntry {
    name: String,
    content_identity: String,
    selected: bool,
}

impl ModelInvocationSkillReceiptEntry {
    /// Returns the exact safe skill name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the BLAKE3 skill body identity.
    #[must_use]
    pub fn content_identity(&self) -> &str {
        self.content_identity.as_str()
    }

    /// Returns whether a separately admitted body was selected transiently.
    #[must_use]
    pub const fn selected(&self) -> bool {
        self.selected
    }
}

/// Safe receipt entry for one canonical model-visible tool declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInvocationToolReceiptEntry {
    name: String,
    schema_identity: String,
}

impl ModelInvocationToolReceiptEntry {
    /// Returns the exact safe tool name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the exact parameter schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }
}

/// Bounded metadata-only invocation receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInvocationReceipt {
    provider_id: String,
    model_id: String,
    router_role: Option<String>,
    target_hash: String,
    profile_id: String,
    profile_hash: String,
    specificity: ModelProfileSpecificity,
    source_scope: ModelProfileSourceScope,
    trust_status: ModelProfileTrustStatus,
    prompt_mode_ref: String,
    prompt_plan_hash: String,
    skills: Vec<ModelInvocationSkillReceiptEntry>,
    skill_plan_hash: String,
    memory_access_mode: ModelMemoryAccessMode,
    memory_policy_hash: String,
    memory_use_enabled: bool,
    memory_generation_enabled: bool,
    memory_plan_hash: String,
    tools: Vec<ModelInvocationToolReceiptEntry>,
    tool_calling_mode: NativeToolCallingMode,
    tool_plan_hash: String,
    tool_catalog_identity: String,
    capability_plan_hash: String,
    policy_hash: String,
    plan_hash: String,
    omission_classes: Vec<ModelInvocationOmissionClass>,
    redaction_marker: &'static str,
    receipt_hash: String,
}

impl ModelInvocationReceipt {
    /// Returns the safe provider ID.
    #[must_use]
    pub fn provider_id(&self) -> &str {
        self.provider_id.as_str()
    }

    /// Returns the safe model ID.
    #[must_use]
    pub fn model_id(&self) -> &str {
        self.model_id.as_str()
    }

    /// Returns the optional router role.
    #[must_use]
    pub fn router_role(&self) -> Option<&str> {
        self.router_role.as_deref()
    }

    /// Returns the selected target identity.
    #[must_use]
    pub fn target_hash(&self) -> &str {
        self.target_hash.as_str()
    }

    /// Returns the selected profile ID.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the profile content identity.
    #[must_use]
    pub fn profile_hash(&self) -> &str {
        self.profile_hash.as_str()
    }

    /// Returns the profile resolution specificity.
    #[must_use]
    pub const fn specificity(&self) -> ModelProfileSpecificity {
        self.specificity
    }

    /// Returns the profile source scope.
    #[must_use]
    pub const fn source_scope(&self) -> ModelProfileSourceScope {
        self.source_scope
    }

    /// Returns the profile trust status.
    #[must_use]
    pub const fn trust_status(&self) -> ModelProfileTrustStatus {
        self.trust_status
    }

    /// Returns the prompt-mode reference.
    #[must_use]
    pub fn prompt_mode_ref(&self) -> &str {
        self.prompt_mode_ref.as_str()
    }

    /// Returns the deterministic prompt plan identity.
    #[must_use]
    pub fn prompt_plan_hash(&self) -> &str {
        self.prompt_plan_hash.as_str()
    }

    /// Returns safe skill metadata entries.
    #[must_use]
    pub fn skills(&self) -> &[ModelInvocationSkillReceiptEntry] {
        self.skills.as_slice()
    }

    /// Returns the deterministic skill plan identity.
    #[must_use]
    pub fn skill_plan_hash(&self) -> &str {
        self.skill_plan_hash.as_str()
    }

    /// Returns the effective model-specific memory access posture.
    #[must_use]
    pub const fn memory_access_mode(&self) -> ModelMemoryAccessMode {
        self.memory_access_mode
    }

    /// Returns the exact existing memory-policy identity.
    #[must_use]
    pub fn memory_policy_hash(&self) -> &str {
        self.memory_policy_hash.as_str()
    }

    /// Returns whether memory use was admitted.
    #[must_use]
    pub const fn memory_use_enabled(&self) -> bool {
        self.memory_use_enabled
    }

    /// Returns whether memory generation was admitted.
    #[must_use]
    pub const fn memory_generation_enabled(&self) -> bool {
        self.memory_generation_enabled
    }

    /// Returns the deterministic effective memory-plan identity.
    #[must_use]
    pub fn memory_plan_hash(&self) -> &str {
        self.memory_plan_hash.as_str()
    }

    /// Returns safe tool metadata entries.
    #[must_use]
    pub fn tools(&self) -> &[ModelInvocationToolReceiptEntry] {
        self.tools.as_slice()
    }

    /// Returns the explicit native tool-calling mode.
    #[must_use]
    pub const fn tool_calling_mode(&self) -> NativeToolCallingMode {
        self.tool_calling_mode
    }

    /// Returns the deterministic tool plan identity.
    #[must_use]
    pub fn tool_plan_hash(&self) -> &str {
        self.tool_plan_hash.as_str()
    }

    /// Returns the trusted tool catalog identity.
    #[must_use]
    pub fn tool_catalog_identity(&self) -> &str {
        self.tool_catalog_identity.as_str()
    }

    /// Returns the capability decision identity.
    #[must_use]
    pub fn capability_plan_hash(&self) -> &str {
        self.capability_plan_hash.as_str()
    }

    /// Returns the active policy identity.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the immutable invocation plan identity.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }

    /// Returns bounded omission classes.
    #[must_use]
    pub fn omission_classes(&self) -> &[ModelInvocationOmissionClass] {
        self.omission_classes.as_slice()
    }

    /// Returns the durable redaction marker.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }

    /// Returns the BLAKE3 receipt identity.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Projects metadata-only evidence from an admitted invocation plan.
///
/// # Errors
///
/// Returns [`ModelRuntimeProfileError`] when resolved and plan identities differ
/// or any forbidden sensitive candidate field is present.
///
/// # Panics
///
/// Panics if a receipt constructed after exact binding validation does not
/// preserve the invocation plan or resolved profile identity.
// r[impl agent_harness_runtime.model_runtime_profiles.receipts]
pub fn project_model_invocation_receipt(
    resolved: &ResolvedModelRuntimeProfile,
    plan: &ModelInvocationPlan,
    sensitive: ModelInvocationReceiptSensitiveDraft<'_>,
) -> Result<ModelInvocationReceipt, ModelRuntimeProfileError> {
    reject_sensitive_receipt_fields(sensitive)?;
    validate_plan_resolution_binding(resolved, plan)?;
    let skills: Vec<ModelInvocationSkillReceiptEntry> = plan
        .skills()
        .iter()
        .map(|skill| ModelInvocationSkillReceiptEntry {
            name: skill.name().to_string(),
            content_identity: skill.content_identity().to_string(),
            selected: skill.selected_body().is_some(),
        })
        .collect();
    let tools: Vec<ModelInvocationToolReceiptEntry> = plan
        .tools()
        .iter()
        .map(|tool| ModelInvocationToolReceiptEntry {
            name: tool.definition().name().to_string(),
            schema_identity: tool.definition().parameter_schema_identity().to_string(),
        })
        .collect();
    let omission_classes = distinct_omission_classes(plan);
    let receipt_hash = receipt_hash(ReceiptHashInput {
        resolved,
        plan,
        skills: skills.as_slice(),
        tools: tools.as_slice(),
        omissions: omission_classes.as_slice(),
    });
    let receipt = ModelInvocationReceipt {
        provider_id: plan.target().provider_id().to_string(),
        model_id: plan.target().model_id().to_string(),
        router_role: plan.target().router_role().map(str::to_string),
        target_hash: plan.target().target_hash().to_string(),
        profile_id: plan.profile_id().to_string(),
        profile_hash: plan.profile_hash().to_string(),
        specificity: resolved.specificity(),
        source_scope: resolved.profile().source_scope(),
        trust_status: resolved.profile().trust_status(),
        prompt_mode_ref: plan.prompt_mode_ref().to_string(),
        prompt_plan_hash: plan.prompt_plan_hash().to_string(),
        skills,
        skill_plan_hash: plan.skill_plan_hash().to_string(),
        memory_access_mode: plan.memory_access_mode(),
        memory_policy_hash: plan.memory_policy_hash().to_string(),
        memory_use_enabled: plan.memory_use_enabled(),
        memory_generation_enabled: plan.memory_generation_enabled(),
        memory_plan_hash: plan.memory_plan_hash().to_string(),
        tools,
        tool_calling_mode: plan.tool_calling_mode(),
        tool_plan_hash: plan.tool_plan_hash().to_string(),
        tool_catalog_identity: plan.tool_catalog_identity().to_string(),
        capability_plan_hash: plan.capability_plan_hash().to_string(),
        policy_hash: plan.policy_hash().to_string(),
        plan_hash: plan.plan_hash().to_string(),
        omission_classes,
        redaction_marker: REDACTION_MARKER,
        receipt_hash,
    };
    assert_eq!(receipt.plan_hash(), plan.plan_hash());
    assert_eq!(receipt.profile_hash(), resolved.profile().profile_hash());
    Ok(receipt)
}

fn reject_sensitive_receipt_fields(
    sensitive: ModelInvocationReceiptSensitiveDraft<'_>,
) -> Result<(), ModelRuntimeProfileError> {
    let leaks = [
        (sensitive.raw_prompt, ModelInvocationReceiptLeak::RawPrompt),
        (sensitive.skill_body, ModelInvocationReceiptLeak::SkillBody),
        (
            sensitive.memory_body,
            ModelInvocationReceiptLeak::MemoryBody,
        ),
        (
            sensitive.script_or_asset,
            ModelInvocationReceiptLeak::ScriptOrAsset,
        ),
        (
            sensitive.tool_arguments,
            ModelInvocationReceiptLeak::ToolArguments,
        ),
        (
            sensitive.tool_output,
            ModelInvocationReceiptLeak::ToolOutput,
        ),
        (
            sensitive.provider_payload,
            ModelInvocationReceiptLeak::ProviderPayload,
        ),
        (sensitive.credential, ModelInvocationReceiptLeak::Credential),
        (
            sensitive.environment_value,
            ModelInvocationReceiptLeak::EnvironmentValue,
        ),
        (sensitive.full_path, ModelInvocationReceiptLeak::FullPath),
        (sensitive.endpoint, ModelInvocationReceiptLeak::Endpoint),
        (
            sensitive.hidden_reasoning,
            ModelInvocationReceiptLeak::HiddenReasoning,
        ),
        (
            sensitive.unbounded_trace,
            ModelInvocationReceiptLeak::UnboundedTrace,
        ),
    ];
    if leaks.iter().any(|(candidate, _)| candidate.is_some()) {
        return Err(error(
            ModelRuntimeProfileIssue::ReceiptLeak,
            FIELD_RECEIPT,
            "receipt candidate contains forbidden raw or sensitive material",
        ));
    }
    assert!(sensitive.raw_prompt.is_none());
    assert!(sensitive.credential.is_none());
    Ok(())
}

fn validate_plan_resolution_binding(
    resolved: &ResolvedModelRuntimeProfile,
    plan: &ModelInvocationPlan,
) -> Result<(), ModelRuntimeProfileError> {
    let is_binding_exact = resolved.target() == plan.target()
        && resolved.profile().profile_id() == plan.profile_id()
        && resolved.profile().profile_hash() == plan.profile_hash()
        && resolved.resolution_hash() == plan.resolution_hash();
    if !is_binding_exact {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_RECEIPT,
            "receipt plan does not match the supplied profile resolution",
        ));
    }
    Ok(())
}

fn distinct_omission_classes(plan: &ModelInvocationPlan) -> Vec<ModelInvocationOmissionClass> {
    let mut classes = Vec::with_capacity(plan.diagnostics().len());
    for diagnostic in plan.diagnostics() {
        if !classes.contains(&diagnostic.class()) {
            classes.push(diagnostic.class());
        }
    }
    classes
}

#[derive(Clone, Copy)]
struct ReceiptHashInput<'a> {
    resolved: &'a ResolvedModelRuntimeProfile,
    plan: &'a ModelInvocationPlan,
    skills: &'a [ModelInvocationSkillReceiptEntry],
    tools: &'a [ModelInvocationToolReceiptEntry],
    omissions: &'a [ModelInvocationOmissionClass],
}

fn receipt_hash(input: ReceiptHashInput<'_>) -> String {
    assert!(!input.plan.plan_hash().is_empty());
    assert_eq!(input.resolved.target(), input.plan.target());
    let mut parts = vec![
        input.plan.target().target_hash().to_string(),
        input.plan.profile_hash().to_string(),
        input.resolved.specificity().as_str().to_string(),
        input.resolved.profile().source_scope().as_str().to_string(),
        input.resolved.profile().trust_status().as_str().to_string(),
        input.plan.prompt_mode_ref().to_string(),
        input.plan.prompt_plan_hash().to_string(),
        input.plan.skill_plan_hash().to_string(),
        input.plan.memory_access_mode().as_str().to_string(),
        input.plan.memory_policy_hash().to_string(),
        input.plan.memory_use_enabled().to_string(),
        input.plan.memory_generation_enabled().to_string(),
        input.plan.memory_plan_hash().to_string(),
        input.plan.capability_plan_hash().to_string(),
        input.plan.tool_plan_hash().to_string(),
        input.plan.tool_catalog_identity().to_string(),
        input.plan.policy_hash().to_string(),
        input.plan.plan_hash().to_string(),
        REDACTION_MARKER.to_string(),
    ];
    for skill in input.skills {
        parts.push(format!(
            "skill:{}:{}:{}",
            skill.name, skill.content_identity, skill.selected
        ));
    }
    for tool in input.tools {
        parts.push(format!("tool:{}:{}", tool.name, tool.schema_identity));
    }
    for omission in input.omissions {
        parts.push(format!("omission:{}", omission.as_str()));
    }
    blake3_parts(MODEL_INVOCATION_RECEIPT_HASH_DOMAIN, parts.as_slice())
}
