#![allow(clippy::struct_excessive_bools, clippy::struct_field_names)]

use alloc::{format, string::String, vec::Vec};

use crate::{
    ATIF_SCHEMA, AtifDiagnostic, AtifField, AtifIssueClass, AtifLocation, CODEC_IDENTITY,
    HARBOR_RELEASE, HARBOR_REVISION, SOURCE_COHORT_IDENTITY, hash_strings,
    required_non_claims_present, required_receipt_fields_present,
};

/// Policy schema.
pub const POLICY_SCHEMA: &str = "animus.harbor-atif-profile.v1";
/// Hard upper bound for caller-supplied JSON bytes.
pub const HARD_MAX_INPUT_BYTES: u64 = 16 * 1_024 * 1_024;
/// Hard upper bound for trajectories in one graph.
pub const HARD_MAX_TRAJECTORIES: usize = 1_024;
/// Hard upper bound for embedded depth.
pub const HARD_MAX_DEPTH: usize = 32;
/// Hard upper bound for steps in one trajectory.
pub const HARD_MAX_STEPS: usize = 16_384;
/// Hard upper bound for aggregate steps.
pub const HARD_MAX_AGGREGATE_STEPS: usize = 65_536;
/// Hard upper bound for one string.
pub const HARD_MAX_STRING_BYTES: usize = 1_048_576;
/// Hard upper bound for one step-local item collection.
pub const HARD_MAX_STEP_ITEMS: usize = 4_096;
/// Hard upper bound for token or log-probability arrays.
pub const HARD_MAX_TOKEN_ITEMS: usize = 1_048_576;
/// Hard upper bound for extension nodes.
pub const HARD_MAX_EXTENSION_NODES: usize = 65_536;
/// Hard upper bound for extension depth.
pub const HARD_MAX_EXTENSION_DEPTH: usize = 32;
/// Hard upper bound for extension string bytes.
pub const HARD_MAX_EXTENSION_BYTES: usize = 4 * 1_024 * 1_024;
/// Hard upper bound for non-claim labels.
pub const HARD_MAX_NON_CLAIMS: usize = 64;

/// Explicit field disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(rename_all = "snake_case"))]
pub enum FieldDisposition {
    /// Include admitted material in a private projection.
    Include,
    /// Omit material and report the loss.
    Omit,
    /// Replace material with a redaction marker.
    Redact,
    /// Retain only an approved domain-separated identity.
    Hash,
    /// Reject the document or projection.
    Reject,
    /// Retain only an unresolved descriptor classification.
    Unresolved,
}

/// External descriptor handling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(rename_all = "snake_case"))]
pub enum ExternalReferencePolicy {
    /// Preserve only an unresolved descriptor class.
    Unresolved,
    /// Reject external descriptors.
    Reject,
}

/// Named finite limits.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct AtifBounds {
    /// Maximum input JSON bytes.
    pub max_input_bytes: u64,
    /// Maximum trajectories in the embedded graph.
    pub max_trajectories: usize,
    /// Maximum embedded trajectory depth.
    pub max_depth: usize,
    /// Maximum steps in one trajectory.
    pub max_steps: usize,
    /// Maximum aggregate steps.
    pub max_aggregate_steps: usize,
    /// Maximum bytes in one string.
    pub max_string_bytes: usize,
    /// Maximum content parts in one message.
    pub max_content_parts: usize,
    /// Maximum tool definitions.
    pub max_tool_definitions: usize,
    /// Maximum calls in one step.
    pub max_tool_calls: usize,
    /// Maximum results in one step.
    pub max_observation_results: usize,
    /// Maximum subagent refs in one result.
    pub max_subagent_refs: usize,
    /// Maximum token IDs or log probabilities in one array.
    pub max_token_items: usize,
    /// Maximum metrics extension entries.
    pub max_metrics_entries: usize,
    /// Maximum extension nodes across the document.
    pub max_extension_nodes: usize,
    /// Maximum extension nesting depth.
    pub max_extension_depth: usize,
    /// Maximum aggregate extension string bytes.
    pub max_extension_bytes: usize,
    /// Maximum loss classes in one projection.
    pub max_loss_classes: usize,
}

/// Import and export privacy posture.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct ProjectionPolicy {
    /// Message text.
    pub messages: FieldDisposition,
    /// Explicit reasoning.
    pub reasoning: FieldDisposition,
    /// Tool argument objects.
    pub tool_arguments: FieldDisposition,
    /// Tool result content.
    pub tool_results: FieldDisposition,
    /// Image descriptors.
    pub images: FieldDisposition,
    /// Token IDs.
    pub token_ids: FieldDisposition,
    /// Log probabilities.
    pub logprobs: FieldDisposition,
    /// Notes.
    pub notes: FieldDisposition,
    /// Provider and model labels.
    pub provider_model: FieldDisposition,
    /// Extension objects.
    pub extensions: FieldDisposition,
    /// Copied-context dataset eligibility.
    pub copied_context_dataset: FieldDisposition,
}

/// Explicit export defaults. Every field must remain false.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct AtifExportDefaults {
    /// Do not invent timestamps.
    pub invent_timestamps: bool,
    /// Do not invent metrics.
    pub invent_metrics: bool,
    /// Do not invent reasoning.
    pub invent_reasoning: bool,
    /// Do not invent tool outcomes.
    pub invent_tool_outcomes: bool,
    /// Do not invent external resources.
    pub invent_external_resources: bool,
    /// Do not invent copied-context status.
    pub invent_copied_context: bool,
    /// Do not invent LLM-call counts.
    pub invent_llm_call_count: bool,
    /// Do not invent terminal trial state.
    pub invent_terminal_state: bool,
}

/// Strict checked ATIF policy.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "json-codec", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "json-codec", serde(deny_unknown_fields))]
pub struct AtifPolicy {
    /// Policy schema.
    pub schema: String,
    /// Profile identity.
    pub profile_id: String,
    /// Harbor release.
    pub harbor_release: String,
    /// Harbor revision.
    pub harbor_revision: String,
    /// ATIF schema.
    pub atif_schema: String,
    /// Codec identity.
    pub codec_identity: String,
    /// Reviewed source cohort identity.
    pub source_cohort_identity: String,
    /// Reviewed source repository.
    pub source_repository: String,
    /// Reviewed source license.
    pub source_license: String,
    /// Ordered reviewed source paths.
    pub source_paths: Vec<String>,
    /// Named limits.
    pub bounds: AtifBounds,
    /// Projection posture.
    pub projection: ProjectionPolicy,
    /// External descriptor policy.
    pub external_references: ExternalReferencePolicy,
    /// Scalar extension key allowlist.
    pub extension_scalar_allowlist: Vec<String>,
    /// Receipt metadata field allowlist.
    pub receipt_fields: Vec<String>,
    /// Required non-claims.
    pub required_non_claims: Vec<String>,
    /// Explicit export defaults.
    pub export_defaults: AtifExportDefaults,
    /// Ambient resolution remains denied.
    pub ambient_resolution: bool,
    /// Automatic persistence remains denied.
    pub automatic_persistence: bool,
    /// Automatic dataset promotion remains denied.
    pub automatic_dataset_promotion: bool,
    /// Automatic evaluation promotion remains denied.
    pub automatic_evaluation_promotion: bool,
}

/// Validated policy and identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedAtifPolicy {
    value: AtifPolicy,
    identity: String,
}

impl ValidatedAtifPolicy {
    /// Return the checked policy.
    #[must_use]
    pub const fn value(&self) -> &AtifPolicy {
        &self.value
    }

    /// Return the policy identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

/// Validate exact cohort, finite bounds, privacy defaults, and authority denials.
///
/// # Errors
/// Returns a bounded policy diagnostic before any decode or projection.
pub fn validate_atif_policy(policy: AtifPolicy) -> Result<ValidatedAtifPolicy, AtifDiagnostic> {
    validate_exact_cohort(&policy)?;
    validate_bounds(&policy.bounds)?;
    validate_authority_posture(&policy)?;
    validate_policy_lists(&policy)?;
    let bounds = bounds_material(&policy.bounds);
    let source_paths = policy.source_paths.join(",");
    let projection = projection_material(&policy.projection);
    let non_claims = policy.required_non_claims.join(",");
    let receipt_fields = policy.receipt_fields.join(",");
    let extensions = policy.extension_scalar_allowlist.join(",");
    let external = match policy.external_references {
        ExternalReferencePolicy::Unresolved => "unresolved",
        ExternalReferencePolicy::Reject => "reject",
    };
    let identity = hash_strings(
        "animus.harbor-atif-policy.v1",
        &[
            &policy.schema,
            &policy.profile_id,
            &policy.harbor_release,
            &policy.harbor_revision,
            &policy.atif_schema,
            &policy.codec_identity,
            &policy.source_cohort_identity,
            &policy.source_repository,
            &policy.source_license,
            &source_paths,
            &bounds,
            &projection,
            external,
            &extensions,
            &receipt_fields,
            &non_claims,
        ],
    );
    Ok(ValidatedAtifPolicy {
        value: policy,
        identity,
    })
}

fn validate_exact_cohort(policy: &AtifPolicy) -> Result<(), AtifDiagnostic> {
    let valid = policy.schema == POLICY_SCHEMA
        && policy.harbor_release == HARBOR_RELEASE
        && policy.harbor_revision == HARBOR_REVISION
        && policy.atif_schema == ATIF_SCHEMA
        && policy.codec_identity == CODEC_IDENTITY
        && policy.source_cohort_identity == SOURCE_COHORT_IDENTITY
        && safe_label(&policy.profile_id)
        && policy.source_repository == "laude-institute/harbor"
        && policy.source_license == "Apache-2.0"
        && !policy.source_paths.is_empty()
        && !policy.source_paths.iter().any(String::is_empty)
        && !has_duplicates(&policy.source_paths);
    if !valid {
        return Err(policy_diagnostic());
    }
    Ok(())
}

fn validate_authority_posture(policy: &AtifPolicy) -> Result<(), AtifDiagnostic> {
    let export = &policy.export_defaults;
    let projection = &policy.projection;
    let unsafe_posture = policy.ambient_resolution
        || policy.automatic_persistence
        || policy.automatic_dataset_promotion
        || policy.automatic_evaluation_promotion
        || export.invent_timestamps
        || export.invent_metrics
        || export.invent_reasoning
        || export.invent_tool_outcomes
        || export.invent_external_resources
        || export.invent_copied_context
        || export.invent_llm_call_count
        || export.invent_terminal_state
        || projection.reasoning == FieldDisposition::Include
        || projection.token_ids == FieldDisposition::Include
        || projection.logprobs == FieldDisposition::Include
        || projection.notes == FieldDisposition::Include
        || projection.extensions == FieldDisposition::Include
        || projection.copied_context_dataset != FieldDisposition::Reject;
    if unsafe_posture {
        return Err(AtifDiagnostic::new(
            AtifIssueClass::Authority,
            AtifLocation::Profile,
            AtifField::Projection,
        ));
    }
    Ok(())
}

fn validate_policy_lists(policy: &AtifPolicy) -> Result<(), AtifDiagnostic> {
    let invalid = policy.required_non_claims.is_empty()
        || policy.required_non_claims.len() > HARD_MAX_NON_CLAIMS
        || policy
            .required_non_claims
            .iter()
            .any(|value| !safe_label(value))
        || has_duplicates(&policy.required_non_claims)
        || policy.receipt_fields.is_empty()
        || policy.receipt_fields.iter().any(|value| !safe_label(value))
        || has_duplicates(&policy.receipt_fields)
        || !required_receipt_fields_present(&policy.receipt_fields)
        || !required_non_claims_present(&policy.required_non_claims)
        || policy
            .extension_scalar_allowlist
            .iter()
            .any(|value| !safe_label(value))
        || has_duplicates(&policy.extension_scalar_allowlist);
    if invalid {
        return Err(policy_diagnostic());
    }
    Ok(())
}

const fn policy_diagnostic() -> AtifDiagnostic {
    AtifDiagnostic::new(
        AtifIssueClass::Policy,
        AtifLocation::Profile,
        AtifField::Identity,
    )
}

fn validate_bounds(bounds: &AtifBounds) -> Result<(), AtifDiagnostic> {
    let valid = within(bounds.max_input_bytes, HARD_MAX_INPUT_BYTES)
        && within(bounds.max_trajectories, HARD_MAX_TRAJECTORIES)
        && within(bounds.max_depth, HARD_MAX_DEPTH)
        && within(bounds.max_steps, HARD_MAX_STEPS)
        && within(bounds.max_aggregate_steps, HARD_MAX_AGGREGATE_STEPS)
        && bounds.max_aggregate_steps >= bounds.max_steps
        && within(bounds.max_string_bytes, HARD_MAX_STRING_BYTES)
        && within(bounds.max_content_parts, HARD_MAX_STEP_ITEMS)
        && within(bounds.max_tool_definitions, HARD_MAX_STEP_ITEMS)
        && within(bounds.max_tool_calls, HARD_MAX_STEP_ITEMS)
        && within(bounds.max_observation_results, HARD_MAX_STEP_ITEMS)
        && within(bounds.max_subagent_refs, HARD_MAX_STEP_ITEMS)
        && within(bounds.max_token_items, HARD_MAX_TOKEN_ITEMS)
        && within(bounds.max_metrics_entries, HARD_MAX_STEP_ITEMS)
        && within(bounds.max_extension_nodes, HARD_MAX_EXTENSION_NODES)
        && within(bounds.max_extension_depth, HARD_MAX_EXTENSION_DEPTH)
        && within(bounds.max_extension_bytes, HARD_MAX_EXTENSION_BYTES)
        && within(bounds.max_loss_classes, HARD_MAX_STEP_ITEMS);
    if !valid {
        return Err(AtifDiagnostic::new(
            AtifIssueClass::Bound,
            AtifLocation::Profile,
            AtifField::Bound,
        ));
    }
    Ok(())
}

fn within<T>(value: T, hard: T) -> bool
where
    T: Copy + Ord + From<u8>,
{
    value >= T::from(1) && value <= hard
}

fn has_duplicates(values: &[String]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].iter().any(|prior| prior == value))
}

fn safe_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= HARD_MAX_STRING_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-'))
}

fn bounds_material(bounds: &AtifBounds) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        bounds.max_input_bytes,
        bounds.max_trajectories,
        bounds.max_depth,
        bounds.max_steps,
        bounds.max_aggregate_steps,
        bounds.max_string_bytes,
        bounds.max_content_parts,
        bounds.max_tool_definitions,
        bounds.max_tool_calls,
        bounds.max_observation_results,
        bounds.max_subagent_refs,
        bounds.max_token_items,
        bounds.max_metrics_entries,
        bounds.max_extension_nodes,
        bounds.max_extension_depth,
        bounds.max_extension_bytes,
        bounds.max_loss_classes,
    )
}

fn projection_material(policy: &ProjectionPolicy) -> String {
    let values = [
        policy.messages,
        policy.reasoning,
        policy.tool_arguments,
        policy.tool_results,
        policy.images,
        policy.token_ids,
        policy.logprobs,
        policy.notes,
        policy.provider_model,
        policy.extensions,
        policy.copied_context_dataset,
    ];
    values
        .iter()
        .map(|value| disposition_label(*value))
        .collect::<Vec<_>>()
        .join(":")
}

pub(crate) const fn disposition_label(value: FieldDisposition) -> &'static str {
    match value {
        FieldDisposition::Include => "include",
        FieldDisposition::Omit => "omit",
        FieldDisposition::Redact => "redact",
        FieldDisposition::Hash => "hash",
        FieldDisposition::Reject => "reject",
        FieldDisposition::Unresolved => "unresolved",
    }
}
