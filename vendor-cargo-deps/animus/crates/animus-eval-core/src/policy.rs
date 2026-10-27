//! Typed evaluation policy validation.

use alloc::{format, string::String, vec::Vec};

use crate::{RecipeKind, hash_identity, validate_blake3_identity, validate_identity};

/// Evaluation policy schema.
pub const EVALUATION_POLICY_SCHEMA: &str = "animus.native-evaluation-policy.v1";
/// Maximum admitted tasks in one evaluation plan.
pub const MAX_POLICY_TASKS: u32 = 4_096;
/// Maximum admitted runs in one evaluation plan.
pub const MAX_POLICY_RUNS: u32 = 1_024;
/// Maximum admitted reward dimensions.
pub const MAX_POLICY_REWARD_DIMENSIONS: u32 = 128;
/// Maximum admitted artifact references.
pub const MAX_POLICY_ARTIFACTS: u32 = 256;
/// Maximum bounded diagnostic bytes.
pub const MAX_DIAGNOSTIC_BYTES: usize = 192;

/// Default network posture for evaluation plans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationNetworkPosture {
    /// No network authority is admitted.
    None,
    /// A host may interpret an explicit private-network request.
    Private,
    /// Public network authority, which this core rejects as a default.
    Public,
}

/// Missing reward value policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MissingRewardPolicy {
    /// Reject a missing value.
    Error,
    /// Keep the dimension visible and exclude it from the aggregate.
    Exclude,
    /// Use an explicit zero contribution.
    Zero,
}

/// Resume mismatch policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeMismatchPolicy {
    /// Rerun only the mismatched run.
    Rerun,
    /// Block until a host resolves the evidence conflict.
    Block,
    /// Redo every run for the affected task.
    RedoWholeTask,
}

/// Borrowed, already-loaded evaluation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationPolicyDraft<'a> {
    /// Exact schema identifier.
    pub schema: &'a str,
    /// BLAKE3 identity supplied for the reviewed Nickel source.
    pub source_identity: &'a str,
    /// Human-reviewable profile identifier.
    pub profile_id: &'a str,
    /// Maximum selected tasks.
    pub max_tasks: u32,
    /// Maximum planned runs.
    pub max_runs: u32,
    /// Maximum reward dimensions.
    pub max_reward_dimensions: u32,
    /// Maximum artifact references.
    pub max_artifacts: u32,
    /// Default network posture.
    pub network_default: EvaluationNetworkPosture,
    /// Whether secret values can enter reusable evaluation values.
    pub secret_values_allowed: bool,
    /// Whether transcript bodies can enter episode or receipt values.
    pub transcript_bodies_allowed: bool,
    /// Missing reward policy.
    pub missing_reward: MissingRewardPolicy,
    /// Resume mismatch policy.
    pub resume_mismatch: ResumeMismatchPolicy,
    /// Explicit supported recipe kinds.
    pub supported_recipes: &'a [RecipeKind],
    /// Required evidence redaction label.
    pub evidence_redaction: &'a str,
}

/// Validated, owned evaluation policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationPolicy {
    /// Reviewed source identity.
    pub source_identity: String,
    /// Profile identifier.
    pub profile_id: String,
    /// Canonical normalized policy identity.
    pub identity: String,
    /// Maximum selected tasks.
    pub max_tasks: u32,
    /// Maximum planned runs.
    pub max_runs: u32,
    /// Maximum reward dimensions.
    pub max_reward_dimensions: u32,
    /// Maximum artifact references.
    pub max_artifacts: u32,
    /// Missing reward policy.
    pub missing_reward: MissingRewardPolicy,
    /// Resume mismatch policy.
    pub resume_mismatch: ResumeMismatchPolicy,
    /// Supported recipes in review order.
    pub supported_recipes: Vec<RecipeKind>,
}

/// Policy validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyError {
    /// The schema is unsupported.
    Schema,
    /// The source identity is malformed or stale.
    SourceIdentity,
    /// A public metadata identity is malformed.
    Identity,
    /// A required bound is zero or exceeds its hard ceiling.
    Bound,
    /// Policy grants unsafe default authority.
    UnsafeDefault,
    /// Recipe support is empty or duplicated.
    Recipe,
    /// Evidence is not metadata-only.
    Redaction,
}

/// Validate and normalize an already-loaded policy.
///
/// `expected_source_identity` is supplied by the host's checked export path.
/// This core does not read the Nickel source or evaluate configuration.
///
/// # Errors
/// Returns a bounded policy class before any evaluation plan can be produced.
pub fn validate_evaluation_policy(
    draft: &EvaluationPolicyDraft<'_>,
    expected_source_identity: &str,
) -> Result<EvaluationPolicy, PolicyError> {
    if draft.schema != EVALUATION_POLICY_SCHEMA {
        return Err(PolicyError::Schema);
    }
    validate_blake3_identity(draft.source_identity).map_err(|_| PolicyError::SourceIdentity)?;
    validate_blake3_identity(expected_source_identity).map_err(|_| PolicyError::SourceIdentity)?;
    if draft.source_identity != expected_source_identity {
        return Err(PolicyError::SourceIdentity);
    }
    validate_identity(draft.profile_id).map_err(|_| PolicyError::Identity)?;
    if draft.max_tasks == 0
        || draft.max_tasks > MAX_POLICY_TASKS
        || draft.max_runs == 0
        || draft.max_runs > MAX_POLICY_RUNS
        || draft.max_reward_dimensions == 0
        || draft.max_reward_dimensions > MAX_POLICY_REWARD_DIMENSIONS
        || draft.max_artifacts == 0
        || draft.max_artifacts > MAX_POLICY_ARTIFACTS
    {
        return Err(PolicyError::Bound);
    }
    if draft.network_default == EvaluationNetworkPosture::Public
        || draft.secret_values_allowed
        || draft.transcript_bodies_allowed
    {
        return Err(PolicyError::UnsafeDefault);
    }
    if draft.evidence_redaction != "metadata_only" {
        return Err(PolicyError::Redaction);
    }
    if draft.supported_recipes.is_empty() {
        return Err(PolicyError::Recipe);
    }
    for (index, recipe) in draft.supported_recipes.iter().enumerate() {
        if draft.supported_recipes[..index].contains(recipe) {
            return Err(PolicyError::Recipe);
        }
    }

    let max_tasks = format!("{}", draft.max_tasks);
    let max_runs = format!("{}", draft.max_runs);
    let max_rewards = format!("{}", draft.max_reward_dimensions);
    let max_artifacts = format!("{}", draft.max_artifacts);
    let recipe_labels = draft
        .supported_recipes
        .iter()
        .map(|recipe| recipe.label())
        .collect::<Vec<_>>();
    let recipe_material = recipe_labels.join(",");
    let identity = hash_identity(
        "animus.native-evaluation-policy.normalized.v1",
        &[
            draft.schema,
            draft.source_identity,
            draft.profile_id,
            &max_tasks,
            &max_runs,
            &max_rewards,
            &max_artifacts,
            network_label(draft.network_default),
            missing_label(draft.missing_reward),
            mismatch_label(draft.resume_mismatch),
            &recipe_material,
            draft.evidence_redaction,
        ],
    );
    Ok(EvaluationPolicy {
        source_identity: draft.source_identity.into(),
        profile_id: draft.profile_id.into(),
        identity,
        max_tasks: draft.max_tasks,
        max_runs: draft.max_runs,
        max_reward_dimensions: draft.max_reward_dimensions,
        max_artifacts: draft.max_artifacts,
        missing_reward: draft.missing_reward,
        resume_mismatch: draft.resume_mismatch,
        supported_recipes: draft.supported_recipes.to_vec(),
    })
}

fn network_label(value: EvaluationNetworkPosture) -> &'static str {
    match value {
        EvaluationNetworkPosture::None => "none",
        EvaluationNetworkPosture::Private => "private",
        EvaluationNetworkPosture::Public => "public",
    }
}

fn missing_label(value: MissingRewardPolicy) -> &'static str {
    match value {
        MissingRewardPolicy::Error => "error",
        MissingRewardPolicy::Exclude => "exclude",
        MissingRewardPolicy::Zero => "zero",
    }
}

fn mismatch_label(value: ResumeMismatchPolicy) -> &'static str {
    match value {
        ResumeMismatchPolicy::Rerun => "rerun",
        ResumeMismatchPolicy::Block => "block",
        ResumeMismatchPolicy::RedoWholeTask => "redo_whole_task",
    }
}
