//! Pure session runtime authority-profile construction and comparison.

#![allow(missing_docs)]

// r[impl agent_harness_runtime.session_authority_profile.contract]

#[allow(unused_imports)]
use crate::prelude::*;

const FIELD_COMPONENTS: &str = "components";
const FIELD_IDENTITY: &str = "identity";
const FIELD_PAYLOAD: &str = "payload";
const FIELD_TRANSITION: &str = "transition";
const FIELD_ENFORCEMENT: &str = "enforcement";
const PROFILE_PAYLOAD_FIXED_FIELDS: usize = 2;
const BINDING_PAYLOAD_FIXED_FIELDS: usize = 6;
const COMPONENT_PAYLOAD_FIELDS: usize = 1;
const BINDING_SCHEMA_INDEX: usize = 0;
const BINDING_IDENTITY_INDEX: usize = 1;
const BINDING_AUTHORIZATION_INDEX: usize = 2;
const BINDING_BRANCH_INDEX: usize = 3;
const BINDING_PRIOR_HEAD_INDEX: usize = 4;
const BINDING_PRIOR_PROFILE_INDEX: usize = 5;
const PROFILE_IDENTITY_OFFSET: usize = 1;
const COMPONENT_ASSIGNMENT_SEPARATOR: char = '=';

#[derive(Clone, Copy)]
struct TransitionBindingInput<'a> {
    branch_id: &'a str,
    authorization_identity: &'a str,
    disposition: super::data::SessionAuthorityTransitionDisposition,
}

/// Builds one canonical metadata-only authority profile.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] when required classes are missing,
/// duplicated, or carry unsafe identities.
pub fn build_session_authority_profile(
    draft: super::data::SessionAuthorityProfileDraft<'_>,
) -> Result<super::data::SessionRuntimeAuthorityProfile, super::data::SessionAuthorityError> {
    if draft.components.len() != super::data::SessionAuthorityComponentClass::ALL.len() {
        return Err(error(
            super::data::SessionAuthorityIssue::MissingComponent,
            FIELD_COMPONENTS,
        ));
    }
    let mut components = Vec::with_capacity(draft.components.len());
    for component in draft.components {
        validate_hash(component.identity)?;
        if components
            .iter()
            .any(|existing: &super::data::SessionAuthorityComponent| {
                existing.class() == component.class
            })
        {
            return Err(error(
                super::data::SessionAuthorityIssue::DuplicateComponent,
                FIELD_COMPONENTS,
            ));
        }
        components.push(super::data::SessionAuthorityComponent {
            class: component.class,
            identity: String::from(component.identity),
        });
    }
    components.sort_by_key(super::data::SessionAuthorityComponent::class);
    for (expected, actual) in super::data::SessionAuthorityComponentClass::ALL
        .iter()
        .zip(components.iter())
    {
        if *expected != actual.class() {
            return Err(error(
                super::data::SessionAuthorityIssue::MissingComponent,
                FIELD_COMPONENTS,
            ));
        }
    }
    let profile_identity = profile_identity(components.as_slice());
    debug_assert_eq!(
        components.len(),
        super::data::SessionAuthorityComponentClass::ALL.len()
    );
    debug_assert_eq!(profile_identity.len(), super::data::HASH_HEX_BYTES);
    Ok(super::data::SessionRuntimeAuthorityProfile::from_parts(
        components,
        profile_identity,
    ))
}

/// Revalidates a canonical profile and its aggregate identity.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] when canonical order, coverage,
/// component identity, or aggregate identity is invalid.
pub fn validate_session_authority_profile(
    profile: &super::data::SessionRuntimeAuthorityProfile,
) -> Result<(), super::data::SessionAuthorityError> {
    if profile.components().len() != super::data::SessionAuthorityComponentClass::ALL.len() {
        return Err(error(
            super::data::SessionAuthorityIssue::MissingComponent,
            FIELD_COMPONENTS,
        ));
    }
    for (expected, component) in super::data::SessionAuthorityComponentClass::ALL
        .iter()
        .zip(profile.components().iter())
    {
        if *expected != component.class() {
            return Err(error(
                super::data::SessionAuthorityIssue::DuplicateComponent,
                FIELD_COMPONENTS,
            ));
        }
        validate_hash(component.identity())?;
    }
    let expected_identity = profile_identity(profile.components());
    if expected_identity != profile.profile_identity() {
        return Err(error(
            super::data::SessionAuthorityIssue::IdentityMismatch,
            FIELD_IDENTITY,
        ));
    }
    debug_assert_eq!(expected_identity.len(), super::data::HASH_HEX_BYTES);
    debug_assert_eq!(
        profile.components().len(),
        super::data::SessionAuthorityComponentClass::ALL.len()
    );
    Ok(())
}

/// Compares a committed profile, when present, with current admitted metadata.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] when either supplied profile is invalid.
pub fn compare_session_authority_profiles(
    committed: Option<&super::data::SessionRuntimeAuthorityProfile>,
    current: &super::data::SessionRuntimeAuthorityProfile,
) -> Result<super::data::SessionAuthorityComparison, super::data::SessionAuthorityError> {
    validate_session_authority_profile(current)?;
    let Some(committed_profile) = committed else {
        return Ok(super::data::SessionAuthorityComparison::from_parts(
            super::data::SessionAuthorityResumeStatus::LegacyUnbound,
            None,
            String::from(current.profile_identity()),
            super::data::SessionAuthorityComponentClass::ALL.to_vec(),
        ));
    };
    validate_session_authority_profile(committed_profile)?;
    let mut changed_components =
        Vec::with_capacity(super::data::SessionAuthorityComponentClass::ALL.len());
    for class in super::data::SessionAuthorityComponentClass::ALL {
        let committed_identity = committed_profile
            .component(class)
            .map(super::data::SessionAuthorityComponent::identity);
        let current_identity = current
            .component(class)
            .map(super::data::SessionAuthorityComponent::identity);
        if committed_identity != current_identity {
            changed_components.push(class);
        }
    }
    let status = if changed_components.is_empty()
        && committed_profile.profile_identity() == current.profile_identity()
    {
        super::data::SessionAuthorityResumeStatus::ExactMatch
    } else {
        super::data::SessionAuthorityResumeStatus::DriftBlocked
    };
    debug_assert!(
        changed_components.len() <= super::data::SessionAuthorityComponentClass::ALL.len()
    );
    debug_assert!(
        status != super::data::SessionAuthorityResumeStatus::ExactMatch
            || changed_components.is_empty()
    );
    Ok(super::data::SessionAuthorityComparison::from_parts(
        status,
        Some(String::from(committed_profile.profile_identity())),
        String::from(current.profile_identity()),
        changed_components,
    ))
}

/// Builds a committed profile binding tied to one branch and prior durable head.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] when profile or binding identities are invalid.
pub fn build_session_authority_binding(
    draft: super::data::SessionAuthorityBindingDraft<'_>,
) -> Result<super::data::SessionAuthorityBinding, super::data::SessionAuthorityError> {
    validate_session_authority_profile(draft.profile)?;
    validate_hash(draft.authorization_identity)?;
    validate_safe_label(draft.branch_id)?;
    validate_hash(draft.prior_head_hash)?;
    if let Some(prior_profile_identity) = draft.prior_profile_identity {
        validate_hash(prior_profile_identity)?;
        if prior_profile_identity == draft.profile.profile_identity() {
            return Err(error(
                super::data::SessionAuthorityIssue::InPlaceRewrite,
                FIELD_TRANSITION,
            ));
        }
    }
    let prior_profile_identity = draft
        .prior_profile_identity
        .unwrap_or(super::data::ABSENT_IDENTITY);
    let binding_identity = hash_parts(
        super::data::SESSION_AUTHORITY_BINDING_HASH_DOMAIN,
        &[
            draft.profile.profile_identity(),
            draft.authorization_identity,
            draft.branch_id,
            draft.prior_head_hash,
            prior_profile_identity,
        ],
    );
    debug_assert_ne!(binding_identity, draft.profile.profile_identity());
    debug_assert_eq!(binding_identity.len(), super::data::HASH_HEX_BYTES);
    Ok(super::data::SessionAuthorityBinding::from_validated_draft(
        draft,
        binding_identity,
    ))
}

/// Encodes one binding into deterministic metadata-only text.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] when the binding fails revalidation.
pub fn encode_session_authority_binding(
    binding: &super::data::SessionAuthorityBinding,
) -> Result<String, super::data::SessionAuthorityError> {
    validate_binding(binding)?;
    let component_count_items = binding.profile().components().len();
    let field_count_items = BINDING_PAYLOAD_FIXED_FIELDS
        .checked_add(PROFILE_PAYLOAD_FIXED_FIELDS)
        .and_then(|count| count.checked_add(component_count_items))
        .ok_or_else(|| {
            error(
                super::data::SessionAuthorityIssue::InvalidPayload,
                FIELD_PAYLOAD,
            )
        })?;
    let mut fields = Vec::with_capacity(field_count_items);
    fields.push(String::from(super::data::SESSION_AUTHORITY_BINDING_SCHEMA));
    fields.push(String::from(binding.binding_identity()));
    fields.push(String::from(binding.authorization_identity()));
    fields.push(String::from(binding.branch_id()));
    fields.push(String::from(binding.prior_head_hash()));
    fields.push(String::from(
        binding
            .prior_profile_identity()
            .unwrap_or(super::data::ABSENT_IDENTITY),
    ));
    fields.push(String::from(super::data::SESSION_AUTHORITY_PROFILE_SCHEMA));
    fields.push(String::from(binding.profile().profile_identity()));
    for component in binding.profile().components() {
        fields.push(format!(
            "{}{}{}",
            component.class().as_str(),
            COMPONENT_ASSIGNMENT_SEPARATOR,
            component.identity()
        ));
    }
    debug_assert_eq!(fields.len(), field_count_items);
    debug_assert!(fields.iter().all(|field| !field.is_empty()));
    Ok(fields.join(&super::data::PAYLOAD_SEPARATOR.to_string()))
}

/// Decodes and revalidates deterministic binding text.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] for malformed, unsafe, or stale identities.
pub fn decode_session_authority_binding(
    payload: &str,
) -> Result<super::data::SessionAuthorityBinding, super::data::SessionAuthorityError> {
    let fields = payload
        .split(super::data::PAYLOAD_SEPARATOR)
        .collect::<Vec<_>>();
    let expected_field_count_items = expected_binding_field_count()?;
    if fields.len() != expected_field_count_items {
        return Err(error(
            super::data::SessionAuthorityIssue::InvalidPayload,
            FIELD_PAYLOAD,
        ));
    }
    validate_binding_schemas(fields.as_slice())?;
    let profile = decode_binding_profile(fields.as_slice())?;
    let prior_profile_identity =
        if fields[BINDING_PRIOR_PROFILE_INDEX] == super::data::ABSENT_IDENTITY {
            None
        } else {
            Some(fields[BINDING_PRIOR_PROFILE_INDEX])
        };
    let binding = build_session_authority_binding(super::data::SessionAuthorityBindingDraft {
        profile: &profile,
        authorization_identity: fields[BINDING_AUTHORIZATION_INDEX],
        branch_id: fields[BINDING_BRANCH_INDEX],
        prior_head_hash: fields[BINDING_PRIOR_HEAD_INDEX],
        prior_profile_identity,
    })?;
    if binding.binding_identity() != fields[BINDING_IDENTITY_INDEX] {
        return Err(error(
            super::data::SessionAuthorityIssue::IdentityMismatch,
            FIELD_IDENTITY,
        ));
    }
    debug_assert_eq!(fields.len(), expected_field_count_items);
    debug_assert_eq!(
        binding.profile().profile_identity(),
        profile.profile_identity()
    );
    Ok(binding)
}

fn expected_binding_field_count() -> Result<usize, super::data::SessionAuthorityError> {
    BINDING_PAYLOAD_FIXED_FIELDS
        .checked_add(PROFILE_PAYLOAD_FIXED_FIELDS)
        .and_then(|count| count.checked_add(super::data::SessionAuthorityComponentClass::ALL.len()))
        .ok_or_else(|| {
            error(
                super::data::SessionAuthorityIssue::InvalidPayload,
                FIELD_PAYLOAD,
            )
        })
}

fn validate_binding_schemas(fields: &[&str]) -> Result<(), super::data::SessionAuthorityError> {
    if fields[BINDING_SCHEMA_INDEX] != super::data::SESSION_AUTHORITY_BINDING_SCHEMA
        || fields[BINDING_PAYLOAD_FIXED_FIELDS] != super::data::SESSION_AUTHORITY_PROFILE_SCHEMA
    {
        return Err(error(
            super::data::SessionAuthorityIssue::InvalidPayload,
            FIELD_PAYLOAD,
        ));
    }
    Ok(())
}

fn decode_binding_profile(
    fields: &[&str],
) -> Result<super::data::SessionRuntimeAuthorityProfile, super::data::SessionAuthorityError> {
    let component_start_index = BINDING_PAYLOAD_FIXED_FIELDS
        .checked_add(PROFILE_PAYLOAD_FIXED_FIELDS)
        .ok_or_else(|| {
            error(
                super::data::SessionAuthorityIssue::InvalidPayload,
                FIELD_PAYLOAD,
            )
        })?;
    let profile_identity_index = BINDING_PAYLOAD_FIXED_FIELDS
        .checked_add(PROFILE_IDENTITY_OFFSET)
        .ok_or_else(|| {
            error(
                super::data::SessionAuthorityIssue::InvalidPayload,
                FIELD_PAYLOAD,
            )
        })?;
    let mut component_drafts =
        Vec::with_capacity(super::data::SessionAuthorityComponentClass::ALL.len());
    for field in &fields[component_start_index..] {
        let (class, identity) = field
            .split_once(COMPONENT_ASSIGNMENT_SEPARATOR)
            .ok_or_else(|| {
                error(
                    super::data::SessionAuthorityIssue::InvalidPayload,
                    FIELD_PAYLOAD,
                )
            })?;
        let class =
            super::data::SessionAuthorityComponentClass::from_label(class).ok_or_else(|| {
                error(
                    super::data::SessionAuthorityIssue::InvalidPayload,
                    FIELD_PAYLOAD,
                )
            })?;
        component_drafts.push(super::data::SessionAuthorityComponentDraft { class, identity });
    }
    let profile = build_session_authority_profile(super::data::SessionAuthorityProfileDraft {
        components: component_drafts.as_slice(),
    })?;
    if profile.profile_identity() != fields[profile_identity_index] {
        return Err(error(
            super::data::SessionAuthorityIssue::IdentityMismatch,
            FIELD_IDENTITY,
        ));
    }
    debug_assert_eq!(
        component_drafts.len(),
        super::data::SessionAuthorityComponentClass::ALL.len()
    );
    debug_assert_eq!(profile.profile_identity(), fields[profile_identity_index]);
    Ok(profile)
}

/// Plans abandon, fork, or rebind without performing authorization or persistence.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] when a transition lacks explicit
/// authorization, attempts an in-place rewrite, or is inconsistent with comparison state.
pub fn plan_session_authority_transition(
    request: super::data::SessionAuthorityTransitionRequest<'_>,
) -> Result<super::data::SessionAuthorityTransitionPlan, super::data::SessionAuthorityError> {
    validate_hash(request.prior_head_hash)?;
    validate_session_authority_profile(request.requested_profile)?;
    debug_assert_eq!(
        request.comparison.current_profile_identity(),
        request.requested_profile.profile_identity()
    );
    debug_assert!(
        request.comparison.changed_components().len()
            <= super::data::SessionAuthorityComponentClass::ALL.len()
    );
    match request.action {
        super::data::SessionAuthorityTransitionAction::Abandon => {
            Ok(super::data::SessionAuthorityTransitionPlan::from_parts(
                super::data::SessionAuthorityTransitionDisposition::Abandoned,
                None,
            ))
        }
        super::data::SessionAuthorityTransitionAction::Fork {
            branch_id,
            authorization_identity,
        } => build_transition_binding(
            request,
            TransitionBindingInput {
                branch_id,
                authorization_identity,
                disposition: super::data::SessionAuthorityTransitionDisposition::ForkCommitRequired,
            },
        ),
        super::data::SessionAuthorityTransitionAction::Rebind {
            branch_id,
            authorization_identity,
        } => {
            if request.comparison.status() == super::data::SessionAuthorityResumeStatus::ExactMatch
            {
                return Err(error(
                    super::data::SessionAuthorityIssue::InPlaceRewrite,
                    FIELD_TRANSITION,
                ));
            }
            build_transition_binding(
                request,
                TransitionBindingInput {
                    branch_id,
                    authorization_identity,
                    disposition:
                        super::data::SessionAuthorityTransitionDisposition::RebindCommitRequired,
                },
            )
        }
    }
}

fn build_transition_binding(
    request: super::data::SessionAuthorityTransitionRequest<'_>,
    input: TransitionBindingInput<'_>,
) -> Result<super::data::SessionAuthorityTransitionPlan, super::data::SessionAuthorityError> {
    let prior_profile_identity = request.comparison.committed_profile_identity();
    let binding = build_session_authority_binding(super::data::SessionAuthorityBindingDraft {
        profile: request.requested_profile,
        authorization_identity: input.authorization_identity,
        branch_id: input.branch_id,
        prior_head_hash: request.prior_head_hash,
        prior_profile_identity,
    })?;
    debug_assert_eq!(binding.branch_id(), input.branch_id);
    debug_assert_eq!(
        binding.authorization_identity(),
        input.authorization_identity
    );
    Ok(super::data::SessionAuthorityTransitionPlan::from_parts(
        input.disposition,
        Some(binding),
    ))
}

/// Builds a current host enforcement observation separate from requested profile identity.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] when any observation identity is unsafe.
pub fn build_session_authority_enforcement_observation(
    draft: super::data::SessionAuthorityEnforcementDraft<'_>,
) -> Result<super::data::SessionAuthorityEnforcementObservation, super::data::SessionAuthorityError>
{
    let parts = [
        draft.profile_identity,
        draft.authorization_identity,
        draft.adapter_cohort_identity,
        draft.capability_observation_identity,
        draft.tool_readiness_identity,
        draft.provider_readiness_identity,
        draft.sandbox_observation_identity,
    ];
    for part in parts {
        validate_hash(part)?;
    }
    let status = if draft.admitted { "admitted" } else { "denied" };
    let observation_identity = hash_parts(
        super::data::SESSION_AUTHORITY_ENFORCEMENT_HASH_DOMAIN,
        &[
            draft.profile_identity,
            draft.authorization_identity,
            draft.adapter_cohort_identity,
            draft.capability_observation_identity,
            draft.tool_readiness_identity,
            draft.provider_readiness_identity,
            draft.sandbox_observation_identity,
            status,
        ],
    );
    debug_assert_eq!(observation_identity.len(), super::data::HASH_HEX_BYTES);
    debug_assert_ne!(observation_identity, draft.profile_identity);
    Ok(
        super::data::SessionAuthorityEnforcementObservation::from_draft(
            draft,
            observation_identity,
        ),
    )
}

/// Validates that a current observation is admitted and bound to the requested profile.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] for mismatched or unavailable enforcement.
pub fn validate_session_authority_enforcement(
    profile: &super::data::SessionRuntimeAuthorityProfile,
    observation: &super::data::SessionAuthorityEnforcementObservation,
) -> Result<(), super::data::SessionAuthorityError> {
    validate_session_authority_profile(profile)?;
    for part in observation.identity_parts() {
        if part == "admitted" || part == "denied" {
            continue;
        }
        validate_hash(part)?;
    }
    let expected_identity = hash_parts(
        super::data::SESSION_AUTHORITY_ENFORCEMENT_HASH_DOMAIN,
        observation.identity_parts().as_slice(),
    );
    if expected_identity != observation.observation_identity()
        || observation.profile_identity() != profile.profile_identity()
    {
        return Err(error(
            super::data::SessionAuthorityIssue::IdentityMismatch,
            FIELD_ENFORCEMENT,
        ));
    }
    if !observation.admitted() {
        return Err(error(
            super::data::SessionAuthorityIssue::EnforcementUnavailable,
            FIELD_ENFORCEMENT,
        ));
    }
    debug_assert_eq!(observation.profile_identity(), profile.profile_identity());
    debug_assert!(observation.admitted());
    Ok(())
}

fn validate_binding(
    binding: &super::data::SessionAuthorityBinding,
) -> Result<(), super::data::SessionAuthorityError> {
    let rebuilt = build_session_authority_binding(super::data::SessionAuthorityBindingDraft {
        profile: binding.profile(),
        authorization_identity: binding.authorization_identity(),
        branch_id: binding.branch_id(),
        prior_head_hash: binding.prior_head_hash(),
        prior_profile_identity: binding.prior_profile_identity(),
    })?;
    if rebuilt.binding_identity() != binding.binding_identity() {
        return Err(error(
            super::data::SessionAuthorityIssue::IdentityMismatch,
            FIELD_IDENTITY,
        ));
    }
    Ok(())
}

fn profile_identity(components: &[super::data::SessionAuthorityComponent]) -> String {
    let identity_part_count_items = components
        .len()
        .saturating_mul(COMPONENT_PAYLOAD_FIELDS.saturating_add(1));
    let mut parts = Vec::with_capacity(identity_part_count_items);
    for component in components {
        parts.push(component.class().as_str());
        parts.push(component.identity());
    }
    hash_parts(
        super::data::SESSION_AUTHORITY_PROFILE_HASH_DOMAIN,
        parts.as_slice(),
    )
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(super::data::HASH_SEPARATOR);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

pub(super) fn validate_hash(value: &str) -> Result<(), super::data::SessionAuthorityError> {
    let has_expected_length_bytes = value.len() == super::data::HASH_HEX_BYTES;
    let has_lowercase_hex = value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !has_expected_length_bytes || !has_lowercase_hex {
        return Err(error(
            super::data::SessionAuthorityIssue::UnsafeIdentity,
            FIELD_IDENTITY,
        ));
    }
    Ok(())
}

pub(super) fn validate_safe_label(value: &str) -> Result<(), super::data::SessionAuthorityError> {
    let has_expected_length_bytes =
        !value.is_empty() && value.len() <= super::data::MAX_SAFE_LABEL_BYTES;
    let has_safe_characters = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'));
    if !has_expected_length_bytes || !has_safe_characters {
        return Err(error(
            super::data::SessionAuthorityIssue::UnsafeIdentity,
            FIELD_IDENTITY,
        ));
    }
    Ok(())
}

pub(super) const fn error(
    issue: super::data::SessionAuthorityIssue,
    field: &'static str,
) -> super::data::SessionAuthorityError {
    super::data::SessionAuthorityError::new(issue, field)
}
