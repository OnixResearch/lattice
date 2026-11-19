//! Field-level bounds of `lattice.typed-policy-profile.v1`.

const LABEL_PUNCTUATION: &[u8] = b"._:-";
const JSON_POINTER_PREFIX: char = '/';

/// Checks every field of a parsed profile.
///
/// # Errors
///
/// Returns the first [`super::admission::Refusal`].
// r[impl lattice.typed_policy.profile]
// r[impl lattice.typed_policy.stateless]
pub fn check(profile: &super::admission::Profile) -> Result<(), super::admission::Refusal> {
    check_header(profile)?;
    check_labels(&profile.labels)?;
    check_input(&profile.input)?;
    check_result(&profile.result)?;
    check_limits(profile.limits)?;
    check_cohort(&profile.cohort)?;
    check_fallback(&profile.fallback, &profile.labels)
}

fn check_header(profile: &super::admission::Profile) -> Result<(), super::admission::Refusal> {
    if profile.schema != super::admission::SCHEMA {
        return Err(super::admission::Refusal::Schema);
    }
    if profile.shape != super::admission::SELECT_SHAPE {
        return Err(super::admission::Refusal::Shape);
    }
    if profile.is_stateful {
        return Err(super::admission::Refusal::Stateful);
    }
    let fields = [
        profile.id.as_str(),
        profile.binding.workflow.as_str(),
        profile.binding.node_label.as_str(),
    ];
    if fields.iter().any(|field| field.trim().is_empty()) {
        return Err(super::admission::Refusal::Selector);
    }
    debug_assert_eq!(profile.schema, super::admission::SCHEMA);
    debug_assert!(!profile.is_stateful);
    Ok(())
}

fn check_labels(labels: &[String]) -> Result<(), super::admission::Refusal> {
    let count = u64::try_from(labels.len()).map_err(|_| super::admission::Refusal::Labels)?;
    if count == 0 || count > super::admission::MAX_LABEL_COUNT {
        return Err(super::admission::Refusal::Labels);
    }
    let unique = labels.iter().collect::<std::collections::BTreeSet<_>>();
    if unique.len() != labels.len() {
        return Err(super::admission::Refusal::Labels);
    }
    if labels.iter().all(|label| is_safe_label(label)) {
        return Ok(());
    }
    Err(super::admission::Refusal::Labels)
}

/// Whether `label` is non-empty, bounded and uses only `[A-Za-z0-9._:-]`.
#[must_use]
pub fn is_safe_label(label: &str) -> bool {
    let is_bounded =
        u64::try_from(label.len()).is_ok_and(|length| length > 0 && length <= super::admission::MAX_LABEL_BYTES);
    is_bounded && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || LABEL_PUNCTUATION.contains(&byte))
}

fn check_input(input: &super::admission::InputSchema) -> Result<(), super::admission::Refusal> {
    let count = u64::try_from(input.features.len()).map_err(|_| super::admission::Refusal::Input)?;
    if count == 0 || count > super::admission::MAX_FEATURE_COUNT {
        return Err(super::admission::Refusal::Input);
    }
    if input.max_bytes == 0 || input.max_bytes > super::admission::MAX_INPUT_BYTES {
        return Err(super::admission::Refusal::Input);
    }
    if input.schema.trim().is_empty() {
        return Err(super::admission::Refusal::Input);
    }
    if input.features.iter().all(is_valid_feature) {
        return Ok(());
    }
    Err(super::admission::Refusal::Input)
}

fn is_valid_feature(feature: &super::admission::Feature) -> bool {
    let is_named = !feature.name.trim().is_empty();
    is_named && feature.pointer.starts_with(JSON_POINTER_PREFIX) && feature.min < feature.max
}

fn check_result(result: &super::admission::ResultSchema) -> Result<(), super::admission::Refusal> {
    if result.schema.trim().is_empty() {
        return Err(super::admission::Refusal::Result);
    }
    if result.max_receipt_bytes == 0 || result.max_receipt_bytes > super::admission::MAX_RECEIPT_BYTES {
        return Err(super::admission::Refusal::Result);
    }
    Ok(())
}

fn check_limits(limits: super::admission::Limits) -> Result<(), super::admission::Refusal> {
    let operation_range = 1..=super::admission::MAX_OPERATION_COUNT;
    if !operation_range.contains(&limits.max_operation_count) {
        return Err(super::admission::Refusal::Limits);
    }
    if (1..=super::admission::MAX_DEADLINE_MILLIS).contains(&limits.deadline_millis) {
        return Ok(());
    }
    Err(super::admission::Refusal::Limits)
}

fn check_cohort(cohort: &super::admission::Cohort) -> Result<(), super::admission::Refusal> {
    if cohort.runtime_crate != animus_frozen_policy::RUNTIME_CRATE {
        return Err(super::admission::Refusal::Cohort);
    }
    if cohort.artifact_api != animus_frozen_policy::ARTIFACT_API || cohort.runtime_version.trim().is_empty() {
        return Err(super::admission::Refusal::Cohort);
    }
    let identities = [cohort.artifact.as_str(), cohort.model.as_str()];
    if !identities.iter().all(|identity| animus_frozen_policy::is_b3_identity(identity)) {
        return Err(super::admission::Refusal::Cohort);
    }
    debug_assert_eq!(cohort.runtime_crate, animus_frozen_policy::RUNTIME_CRATE);
    debug_assert!(animus_frozen_policy::is_b3_identity(&cohort.model));
    let schemas = [
        cohort.feature_schema.as_str(),
        cohort.action_schema.as_str(),
        cohort.policy_kind.as_str(),
    ];
    if schemas.iter().all(|schema| animus_frozen_policy::is_safe_identity(schema)) {
        return Ok(());
    }
    Err(super::admission::Refusal::Cohort)
}

fn check_fallback(fallback: &super::admission::Fallback, labels: &[String]) -> Result<(), super::admission::Refusal> {
    match fallback {
        super::admission::Fallback::Label { label } if labels.contains(label) => Ok(()),
        super::admission::Fallback::Label { .. } => Err(super::admission::Refusal::Fallback),
        super::admission::Fallback::DeclaredPredicates | super::admission::Fallback::Terminal => Ok(()),
    }
}
