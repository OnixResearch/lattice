//! Pure separation-of-duty policy evaluation.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::types::{
    AGENT_ROLE_SEPARATION_HASH_DOMAIN, AgentRoleError, AgentRoleIssue,
    AgentRoleSeparationDimension, AgentRoleSeparationPolicy,
};
use super::validation::{ValidationValue, error, hash_parts, lowercase_blake3};

const FIELD_SEPARATION: &str = "agent_role_separation";

/// Host-supplied identities and observations considered by separation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleSeparationObservation<'a> {
    /// Candidate assignment or assignment-nonce identity.
    pub candidate_assignment_hash: &'a str,
    /// Assignment identity that produced or mutated the subject.
    pub subject_assignment_hash: &'a str,
    /// Candidate actor identity.
    pub candidate_actor_hash: &'a str,
    /// Subject-origin actor identity.
    pub subject_actor_hash: &'a str,
    /// Candidate run or thread identity.
    pub candidate_run_thread_hash: &'a str,
    /// Subject-origin run or thread identity.
    pub subject_run_thread_hash: &'a str,
    /// Candidate session identity.
    pub candidate_session_hash: &'a str,
    /// Subject-origin session identity.
    pub subject_session_hash: &'a str,
    /// Candidate model/runtime-profile identity.
    pub candidate_model_profile_hash: &'a str,
    /// Subject-origin model/runtime-profile identity.
    pub subject_model_profile_hash: &'a str,
    /// Candidate workspace identity.
    pub candidate_workspace_hash: &'a str,
    /// Subject-origin workspace identity.
    pub subject_workspace_hash: &'a str,
    /// Candidate mutation-lineage identity.
    pub candidate_mutation_lineage_hash: &'a str,
    /// Subject-origin mutation-lineage identity.
    pub subject_mutation_lineage_hash: &'a str,
    /// Pinned subject identity supplied to the role.
    pub subject_hash: &'a str,
    /// Subject identity observed when checking separation.
    pub observed_subject_hash: &'a str,
    /// Expected verifier/check identity when relevant.
    pub expected_verifier_hash: Option<&'a str>,
    /// Observed verifier/check identity when relevant.
    pub observed_verifier_hash: Option<&'a str>,
    /// Expected rubric identity when relevant.
    pub expected_rubric_hash: Option<&'a str>,
    /// Observed rubric identity when relevant.
    pub observed_rubric_hash: Option<&'a str>,
    /// Active policy identity.
    pub policy_hash: &'a str,
    /// Policy identity observed at the decision boundary.
    pub observed_policy_hash: &'a str,
    /// Host-affirmed state dimensions; identity distinctions are computed, not asserted.
    pub satisfied_state_dimensions: &'a [AgentRoleSeparationDimension],
}

/// Bounded result of explicit separation-policy evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleSeparationDecision {
    policy_id: String,
    subject_hash: String,
    passed: bool,
    failed_dimensions: Vec<AgentRoleSeparationDimension>,
    decision_hash: String,
}

impl AgentRoleSeparationDecision {
    /// Returns the exact separation policy identity.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns the pinned subject identity.
    #[must_use]
    pub fn subject_hash(&self) -> &str {
        self.subject_hash.as_str()
    }

    /// Returns whether every required observed dimension passed.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.passed
    }

    /// Returns failed checked dimensions.
    #[must_use]
    pub fn failed_dimensions(&self) -> &[AgentRoleSeparationDimension] {
        self.failed_dimensions.as_slice()
    }

    /// Returns the BLAKE3 separation-decision identity.
    #[must_use]
    pub fn decision_hash(&self) -> &str {
        self.decision_hash.as_str()
    }
}

/// Evaluates exactly the dimensions selected by a named separation policy.
///
/// # Errors
///
/// Returns [`AgentRoleError`] for malformed or stale subject, verifier, rubric,
/// or policy identities. A well-formed policy failure is returned as a bounded
/// decision with `passed() == false`.
// r[impl agent_harness_runtime.agent_role_profiles.separation]
pub fn evaluate_agent_role_separation(
    policy: &AgentRoleSeparationPolicy,
    observation: &AgentRoleSeparationObservation<'_>,
) -> Result<AgentRoleSeparationDecision, AgentRoleError> {
    validate_observation_hashes(observation)?;
    validate_state_dimensions(observation.satisfied_state_dimensions)?;
    validate_identity_bindings(policy, observation)?;
    let failed_dimensions = policy
        .required_dimensions()
        .iter()
        .copied()
        .filter(|dimension| !dimension_passes(*dimension, observation))
        .collect::<Vec<_>>();
    let is_passed = failed_dimensions.is_empty();
    let hash_material =
        separation_hash_material(policy, observation, &failed_dimensions, is_passed);
    let decision_hash = hash_parts(AGENT_ROLE_SEPARATION_HASH_DOMAIN, hash_material.as_slice());
    debug_assert_eq!(is_passed, failed_dimensions.is_empty());
    debug_assert!(!decision_hash.is_empty());
    Ok(AgentRoleSeparationDecision {
        policy_id: policy.policy_id().to_string(),
        subject_hash: observation.subject_hash.to_string(),
        passed: is_passed,
        failed_dimensions,
        decision_hash,
    })
}

fn validate_identity_bindings(
    policy: &AgentRoleSeparationPolicy,
    observation: &AgentRoleSeparationObservation<'_>,
) -> Result<(), AgentRoleError> {
    if observation.subject_hash != observation.observed_subject_hash
        || observation.policy_hash != observation.observed_policy_hash
    {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_SEPARATION,
            "separation observation changed subject or policy identity",
        ));
    }
    validate_optional_identity_pair(
        observation.expected_verifier_hash,
        observation.observed_verifier_hash,
        policy.requires(AgentRoleSeparationDimension::VerifierImmutable),
    )?;
    validate_optional_identity_pair(
        observation.expected_rubric_hash,
        observation.observed_rubric_hash,
        policy.requires(AgentRoleSeparationDimension::RubricImmutable),
    )?;
    debug_assert_eq!(observation.subject_hash, observation.observed_subject_hash);
    debug_assert_eq!(observation.policy_hash, observation.observed_policy_hash);
    Ok(())
}

fn validate_state_dimensions(
    dimensions: &[AgentRoleSeparationDimension],
) -> Result<(), AgentRoleError> {
    let mut sorted = dimensions.to_vec();
    sorted.sort_unstable();
    let has_duplicate = sorted.windows(2).any(|pair| pair[0] == pair[1]);
    let has_identity_claim = sorted.iter().any(|dimension| {
        !matches!(
            dimension,
            AgentRoleSeparationDimension::CleanContext
                | AgentRoleSeparationDimension::SubjectImmutable
                | AgentRoleSeparationDimension::VerifierImmutable
                | AgentRoleSeparationDimension::RubricImmutable
        )
    });
    if has_duplicate || has_identity_claim {
        return Err(error(
            AgentRoleIssue::InvalidCollection,
            FIELD_SEPARATION,
            "state observations must be unique and cannot assert computed identity dimensions",
        ));
    }
    debug_assert!(!has_duplicate);
    debug_assert!(!has_identity_claim);
    Ok(())
}

fn dimension_passes(
    dimension: AgentRoleSeparationDimension,
    observation: &AgentRoleSeparationObservation<'_>,
) -> bool {
    match dimension {
        AgentRoleSeparationDimension::Assignment => {
            observation.candidate_assignment_hash != observation.subject_assignment_hash
        }
        AgentRoleSeparationDimension::Actor => {
            observation.candidate_actor_hash != observation.subject_actor_hash
        }
        AgentRoleSeparationDimension::RunThread => {
            observation.candidate_run_thread_hash != observation.subject_run_thread_hash
        }
        AgentRoleSeparationDimension::Session => {
            observation.candidate_session_hash != observation.subject_session_hash
        }
        AgentRoleSeparationDimension::ModelProfile => {
            observation.candidate_model_profile_hash != observation.subject_model_profile_hash
        }
        AgentRoleSeparationDimension::Workspace => {
            observation.candidate_workspace_hash != observation.subject_workspace_hash
        }
        AgentRoleSeparationDimension::MutationLineage => {
            observation.candidate_mutation_lineage_hash != observation.subject_mutation_lineage_hash
        }
        AgentRoleSeparationDimension::CleanContext
        | AgentRoleSeparationDimension::SubjectImmutable
        | AgentRoleSeparationDimension::VerifierImmutable
        | AgentRoleSeparationDimension::RubricImmutable => {
            observation.satisfied_state_dimensions.contains(&dimension)
        }
    }
}

fn separation_hash_material(
    policy: &AgentRoleSeparationPolicy,
    observation: &AgentRoleSeparationObservation<'_>,
    failed_dimensions: &[AgentRoleSeparationDimension],
    is_passed: bool,
) -> Vec<String> {
    let mut material = vec![
        policy.policy_id().to_string(),
        observation.subject_hash.to_string(),
        observation.observed_subject_hash.to_string(),
        observation.candidate_assignment_hash.to_string(),
        observation.subject_assignment_hash.to_string(),
        observation.candidate_actor_hash.to_string(),
        observation.subject_actor_hash.to_string(),
        observation.candidate_run_thread_hash.to_string(),
        observation.subject_run_thread_hash.to_string(),
        observation.candidate_session_hash.to_string(),
        observation.subject_session_hash.to_string(),
        observation.candidate_model_profile_hash.to_string(),
        observation.subject_model_profile_hash.to_string(),
        observation.candidate_workspace_hash.to_string(),
        observation.subject_workspace_hash.to_string(),
        observation.candidate_mutation_lineage_hash.to_string(),
        observation.subject_mutation_lineage_hash.to_string(),
        observation
            .expected_verifier_hash
            .unwrap_or("none")
            .to_string(),
        observation
            .observed_verifier_hash
            .unwrap_or("none")
            .to_string(),
        observation
            .expected_rubric_hash
            .unwrap_or("none")
            .to_string(),
        observation
            .observed_rubric_hash
            .unwrap_or("none")
            .to_string(),
        observation.policy_hash.to_string(),
        observation.observed_policy_hash.to_string(),
        is_passed.to_string(),
    ];
    material.extend(
        policy
            .required_dimensions()
            .iter()
            .map(|dimension| format!("required:{}", dimension.as_str())),
    );
    material.extend(
        observation
            .satisfied_state_dimensions
            .iter()
            .map(|dimension| format!("satisfied:{}", dimension.as_str())),
    );
    material.extend(
        failed_dimensions
            .iter()
            .map(|dimension| format!("failed:{}", dimension.as_str())),
    );
    debug_assert!(!material.is_empty());
    debug_assert_eq!(
        material.first().map(String::as_str),
        Some(policy.policy_id())
    );
    material
}

fn validate_observation_hashes(
    observation: &AgentRoleSeparationObservation<'_>,
) -> Result<(), AgentRoleError> {
    for (field, hash) in [
        (
            "candidate_assignment_hash",
            observation.candidate_assignment_hash,
        ),
        (
            "subject_assignment_hash",
            observation.subject_assignment_hash,
        ),
        ("candidate_actor_hash", observation.candidate_actor_hash),
        ("subject_actor_hash", observation.subject_actor_hash),
        (
            "candidate_run_thread_hash",
            observation.candidate_run_thread_hash,
        ),
        (
            "subject_run_thread_hash",
            observation.subject_run_thread_hash,
        ),
        ("candidate_session_hash", observation.candidate_session_hash),
        ("subject_session_hash", observation.subject_session_hash),
        (
            "candidate_model_profile_hash",
            observation.candidate_model_profile_hash,
        ),
        (
            "subject_model_profile_hash",
            observation.subject_model_profile_hash,
        ),
        (
            "candidate_workspace_hash",
            observation.candidate_workspace_hash,
        ),
        ("subject_workspace_hash", observation.subject_workspace_hash),
        (
            "candidate_mutation_lineage_hash",
            observation.candidate_mutation_lineage_hash,
        ),
        (
            "subject_mutation_lineage_hash",
            observation.subject_mutation_lineage_hash,
        ),
        ("subject_hash", observation.subject_hash),
        ("observed_subject_hash", observation.observed_subject_hash),
        ("policy_hash", observation.policy_hash),
        ("observed_policy_hash", observation.observed_policy_hash),
    ] {
        lowercase_blake3(ValidationValue { field, value: hash })?;
    }
    for hash in [
        observation.expected_verifier_hash,
        observation.observed_verifier_hash,
        observation.expected_rubric_hash,
        observation.observed_rubric_hash,
    ]
    .into_iter()
    .flatten()
    {
        lowercase_blake3(ValidationValue {
            field: FIELD_SEPARATION,
            value: hash,
        })?;
    }
    debug_assert!(!observation.subject_hash.is_empty());
    debug_assert!(!observation.policy_hash.is_empty());
    Ok(())
}

fn validate_optional_identity_pair(
    expected: Option<&str>,
    observed: Option<&str>,
    is_required: bool,
) -> Result<(), AgentRoleError> {
    if is_required && expected.is_none() {
        return Err(error(
            AgentRoleIssue::MissingReference,
            FIELD_SEPARATION,
            "required separation identity is missing",
        ));
    }
    if is_required && expected != observed {
        return Err(error(
            AgentRoleIssue::BindingMismatch,
            FIELD_SEPARATION,
            "required separation identity changed",
        ));
    }
    Ok(())
}
