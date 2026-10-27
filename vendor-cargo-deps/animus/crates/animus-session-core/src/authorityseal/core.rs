//! Pure runtime-authority unit validation and transitions.

#![allow(missing_docs)]

#[allow(unused_imports)]
use crate::prelude::*;

const FIELD_SESSION: &str = "session";
const FIELD_DISPATCH: &str = "dispatch";
const FIELD_IDENTITY: &str = "identity";
const FIELD_OPERATIONS: &str = "operation_identities";
const FIELD_RESOURCES: &str = "resource_identities";
const FIELD_EXPECTED_VIEWS: &str = "expected_views";
const FIELD_OBSERVED_VIEWS: &str = "observed_views";
const FIELD_STATE: &str = "state";
const FIELD_RECEIPT: &str = "receipt";
const FIELD_RECHECK: &str = "current_recheck";

/// Prepare one immutable dispatch authority unit and its pre-effect receipt.
///
/// # Errors
///
/// Returns deterministic issues when labels, identities, views, scope, or receipt fields fail.
pub fn prepare_runtime_authority_seal(
    draft: super::data::RuntimeAuthorityUnitDraft<'_>,
    receipt: super::data::RuntimeAuthorityReceiptDraft<'_>,
) -> Result<super::data::RuntimeAuthoritySealPlan, super::data::RuntimeAuthoritySealError> {
    let mut issues = Vec::with_capacity(super::data::MAX_RUNTIME_AUTHORITY_ISSUES);
    if !is_safe_label(draft.session_id) {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::UnsafeLabel,
            FIELD_SESSION,
        ));
    }
    if !is_safe_label(draft.dispatch_id) {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::UnsafeLabel,
            FIELD_DISPATCH,
        ));
    }
    for identity in fixed_identities(draft) {
        if !is_valid_hash(identity) {
            issues.push(issue(
                super::data::RuntimeAuthorityIssueCode::UnsafeIdentity,
                FIELD_IDENTITY,
            ));
        }
    }
    let (operation_identities, operation_issues) = normalize_hashes(
        draft.operation_identities,
        HashListPolicy {
            field: FIELD_OPERATIONS,
            maximum: super::data::MAX_RUNTIME_AUTHORITY_OPERATIONS,
            require_non_empty: true,
            duplicate_code: super::data::RuntimeAuthorityIssueCode::DuplicateOperation,
        },
    );
    issues.extend(operation_issues);
    let (resource_identities, resource_issues) = normalize_hashes(
        draft.resource_identities,
        HashListPolicy {
            field: FIELD_RESOURCES,
            maximum: super::data::MAX_RUNTIME_AUTHORITY_RESOURCES,
            require_non_empty: false,
            duplicate_code: super::data::RuntimeAuthorityIssueCode::DuplicateResource,
        },
    );
    issues.extend(resource_issues);
    let (runtime_views, runtime_view_issues) = compare_runtime_views(draft);
    issues.extend(runtime_view_issues);
    if !issues.is_empty() {
        return Err(super::data::RuntimeAuthoritySealError::from_issues(issues));
    }
    let unit = unit_from_validated_collections(
        draft,
        ValidatedAuthorityCollections {
            operation_identities,
            resource_identities,
            runtime_views,
        },
    );
    let receipt = super::evidence::project_runtime_authority_seal_receipt(&unit, receipt)?;
    debug_assert_eq!(
        unit.state(),
        super::data::RuntimeAuthorityUnitState::Preparing
    );
    debug_assert_eq!(unit.unit_identity().len(), super::data::HASH_HEX_BYTES);
    Ok(super::data::RuntimeAuthoritySealPlan::from_parts(
        unit, receipt,
    ))
}

struct ValidatedAuthorityCollections {
    operation_identities: Vec<String>,
    resource_identities: Vec<String>,
    runtime_views: Vec<super::data::RuntimeAuthorityViewEntry>,
}

fn unit_from_validated_collections(
    draft: super::data::RuntimeAuthorityUnitDraft<'_>,
    collections: ValidatedAuthorityCollections,
) -> super::data::RuntimeAuthorityUnit {
    debug_assert!(
        collections.operation_identities.len() <= super::data::MAX_RUNTIME_AUTHORITY_OPERATIONS
    );
    debug_assert!(
        collections.resource_identities.len() <= super::data::MAX_RUNTIME_AUTHORITY_RESOURCES
    );
    debug_assert_eq!(
        collections.runtime_views.len(),
        super::data::RuntimeAuthorityViewClass::ALL.len()
    );
    let unit_identity = unit_identity(
        draft,
        collections.operation_identities.as_slice(),
        collections.resource_identities.as_slice(),
        collections.runtime_views.as_slice(),
    );
    super::data::RuntimeAuthorityUnit::from_parts(super::data::RuntimeAuthorityUnitParts {
        session_id: String::from(draft.session_id),
        dispatch_id: String::from(draft.dispatch_id),
        session_lineage_identity: String::from(draft.session_lineage_identity),
        durable_binding_identity: String::from(draft.durable_binding_identity),
        authority_profile_identity: String::from(draft.authority_profile_identity),
        enforcement_observation_identity: String::from(draft.enforcement_observation_identity),
        operation_set_identity: String::from(draft.operation_set_identity),
        adapter_cohort_identity: String::from(draft.adapter_cohort_identity),
        resource_profile_identity: String::from(draft.resource_profile_identity),
        resource_budget_identity: String::from(draft.resource_budget_identity),
        runtime_services_identity: String::from(draft.runtime_services_identity),
        operation_identities: collections.operation_identities,
        resource_identities: collections.resource_identities,
        runtime_views: collections.runtime_views,
        unit_identity,
    })
}

/// Mark the unit sealed after the shell confirms the exact receipt identity.
///
/// # Errors
///
/// Returns issues when the plan state or committed receipt identity does not match.
pub fn commit_runtime_authority_seal(
    plan: &super::data::RuntimeAuthoritySealPlan,
    committed_receipt_identity: &str,
) -> Result<super::data::RuntimeAuthorityUnit, super::data::RuntimeAuthoritySealError> {
    let mut issues = Vec::new();
    if !is_valid_hash(committed_receipt_identity) {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::UnsafeIdentity,
            FIELD_RECEIPT,
        ));
    }
    if plan.unit().state() != super::data::RuntimeAuthorityUnitState::Preparing {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::InvalidStateTransition,
            FIELD_STATE,
        ));
    }
    if committed_receipt_identity != plan.receipt().receipt_identity {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::ReceiptMismatch,
            FIELD_RECEIPT,
        ));
    }
    if !issues.is_empty() {
        return Err(super::data::RuntimeAuthoritySealError::from_issues(issues));
    }
    let sealed = plan.unit().with_state(
        super::data::RuntimeAuthorityUnitState::Sealed,
        Some(String::from(committed_receipt_identity)),
    );
    debug_assert_eq!(sealed.unit_identity(), plan.unit().unit_identity());
    debug_assert_eq!(
        sealed.seal_receipt_identity(),
        Some(committed_receipt_identity)
    );
    Ok(sealed)
}

/// Recheck current enforcement and move one sealed unit to the running state.
///
/// # Errors
///
/// Returns issues when the unit is not sealed or the current view widens, narrows, or denies scope.
pub fn start_runtime_authority_effect(
    unit: &super::data::RuntimeAuthorityUnit,
    recheck: super::data::RuntimeAuthorityRecheckDraft<'_>,
) -> Result<super::data::RuntimeAuthorityUnit, super::data::RuntimeAuthoritySealError> {
    let mut issues = validate_runtime_authority_recheck(unit, recheck)
        .err()
        .map_or_else(Vec::new, |error| error.issues().to_vec());
    if unit.state() != super::data::RuntimeAuthorityUnitState::Sealed {
        issues.push(issue(
            if unit.state().is_terminal() {
                super::data::RuntimeAuthorityIssueCode::StaleUnitReuse
            } else {
                super::data::RuntimeAuthorityIssueCode::InvalidStateTransition
            },
            FIELD_STATE,
        ));
    }
    if unit.seal_receipt_identity().is_none() {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::ReceiptMismatch,
            FIELD_RECEIPT,
        ));
    }
    if !issues.is_empty() {
        return Err(super::data::RuntimeAuthoritySealError::from_issues(issues));
    }
    let running = unit.with_state(
        super::data::RuntimeAuthorityUnitState::Running,
        unit.seal_receipt_identity().map(String::from),
    );
    debug_assert_eq!(running.unit_identity(), unit.unit_identity());
    Ok(running)
}

/// Compare the current enforcement view with the sealed unit.
///
/// # Errors
///
/// Returns issues for invalid identities, mismatch, denial, narrowing, or widening.
pub fn validate_runtime_authority_recheck(
    unit: &super::data::RuntimeAuthorityUnit,
    recheck: super::data::RuntimeAuthorityRecheckDraft<'_>,
) -> Result<(), super::data::RuntimeAuthoritySealError> {
    debug_assert!(
        unit.operation_identities()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    debug_assert!(
        unit.resource_identities()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    let mut issues = recheck_identity_issues(recheck);
    let (current_operations, operation_issues) = normalize_hashes(
        recheck.operation_identities,
        HashListPolicy {
            field: FIELD_OPERATIONS,
            maximum: super::data::MAX_RUNTIME_AUTHORITY_OPERATIONS,
            require_non_empty: false,
            duplicate_code: super::data::RuntimeAuthorityIssueCode::DuplicateOperation,
        },
    );
    issues.extend(operation_issues);
    let (current_resources, resource_issues) = normalize_hashes(
        recheck.resource_identities,
        HashListPolicy {
            field: FIELD_RESOURCES,
            maximum: super::data::MAX_RUNTIME_AUTHORITY_RESOURCES,
            require_non_empty: false,
            duplicate_code: super::data::RuntimeAuthorityIssueCode::DuplicateResource,
        },
    );
    issues.extend(resource_issues);
    if !is_exact_recheck_identity_match(unit, recheck) {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::CurrentEnforcementMismatch,
            FIELD_RECHECK,
        ));
    }
    let is_operations_subset =
        is_subset(current_operations.as_slice(), unit.operation_identities());
    let is_resources_subset = is_subset(current_resources.as_slice(), unit.resource_identities());
    if !is_operations_subset || !is_resources_subset {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::PostSealWidening,
            FIELD_RECHECK,
        ));
    }
    let is_exact_dispatch_scope = current_operations == unit.operation_identities()
        && current_resources == unit.resource_identities();
    if !recheck.admitted || !is_exact_dispatch_scope {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::CurrentEnforcementDenied,
            FIELD_RECHECK,
        ));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(super::data::RuntimeAuthoritySealError::from_issues(issues))
    }
}

fn recheck_identity_issues(
    recheck: super::data::RuntimeAuthorityRecheckDraft<'_>,
) -> Vec<super::data::RuntimeAuthorityIssue> {
    let identities = [
        recheck.session_lineage_identity,
        recheck.authority_profile_identity,
        recheck.durable_binding_identity,
        recheck.enforcement_observation_identity,
        recheck.operation_set_identity,
        recheck.adapter_cohort_identity,
        recheck.resource_profile_identity,
        recheck.resource_budget_identity,
        recheck.runtime_services_identity,
    ];
    debug_assert_eq!(
        identities.len(),
        super::data::FIXED_AUTHORITY_IDENTITY_COUNT
    );
    debug_assert!(!FIELD_RECHECK.is_empty());
    identities
        .into_iter()
        .filter(|identity| !is_valid_hash(identity))
        .map(|_| {
            issue(
                super::data::RuntimeAuthorityIssueCode::UnsafeIdentity,
                FIELD_RECHECK,
            )
        })
        .collect()
}

fn is_exact_recheck_identity_match(
    unit: &super::data::RuntimeAuthorityUnit,
    recheck: super::data::RuntimeAuthorityRecheckDraft<'_>,
) -> bool {
    recheck.session_lineage_identity == unit.session_lineage_identity()
        && recheck.authority_profile_identity == unit.authority_profile_identity()
        && recheck.durable_binding_identity == unit.durable_binding_identity()
        && recheck.enforcement_observation_identity == unit.enforcement_observation_identity()
        && recheck.operation_set_identity == unit.operation_set_identity()
        && recheck.adapter_cohort_identity == unit.adapter_cohort_identity()
        && recheck.resource_profile_identity == unit.resource_profile_identity()
        && recheck.resource_budget_identity == unit.resource_budget_identity()
        && recheck.runtime_services_identity == unit.runtime_services_identity()
}

/// Check that a proposed scope is equal to or narrower than the sealed ceiling.
///
/// # Errors
///
/// Returns issues when an identity is invalid or the proposed scope widens authority.
pub fn validate_runtime_authority_narrowing(
    unit: &super::data::RuntimeAuthorityUnit,
    operation_identities: &[&str],
    resource_identities: &[&str],
) -> Result<(), super::data::RuntimeAuthoritySealError> {
    debug_assert!(
        unit.operation_identities()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    debug_assert!(
        unit.resource_identities()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    let mut issues = Vec::new();
    let (operations, operation_issues) = normalize_hashes(
        operation_identities,
        HashListPolicy {
            field: FIELD_OPERATIONS,
            maximum: super::data::MAX_RUNTIME_AUTHORITY_OPERATIONS,
            require_non_empty: false,
            duplicate_code: super::data::RuntimeAuthorityIssueCode::DuplicateOperation,
        },
    );
    issues.extend(operation_issues);
    let (resources, resource_issues) = normalize_hashes(
        resource_identities,
        HashListPolicy {
            field: FIELD_RESOURCES,
            maximum: super::data::MAX_RUNTIME_AUTHORITY_RESOURCES,
            require_non_empty: false,
            duplicate_code: super::data::RuntimeAuthorityIssueCode::DuplicateResource,
        },
    );
    issues.extend(resource_issues);
    if !is_subset(operations.as_slice(), unit.operation_identities())
        || !is_subset(resources.as_slice(), unit.resource_identities())
    {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::PostSealWidening,
            FIELD_STATE,
        ));
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(super::data::RuntimeAuthoritySealError::from_issues(issues))
    }
}

/// Apply one terminal transition or reject mutation and stale reuse.
///
/// # Errors
///
/// Returns issues when the transition is invalid, widens authority, or reuses a terminal unit.
pub fn transition_runtime_authority_unit(
    unit: &super::data::RuntimeAuthorityUnit,
    transition: super::data::RuntimeAuthorityTransition,
) -> Result<super::data::RuntimeAuthorityUnit, super::data::RuntimeAuthoritySealError> {
    let next_state = match (unit.state(), transition) {
        (
            super::data::RuntimeAuthorityUnitState::Running,
            super::data::RuntimeAuthorityTransition::Complete,
        ) => super::data::RuntimeAuthorityUnitState::Completed,
        (
            super::data::RuntimeAuthorityUnitState::Sealed
            | super::data::RuntimeAuthorityUnitState::Running,
            super::data::RuntimeAuthorityTransition::Cancel,
        ) => super::data::RuntimeAuthorityUnitState::Cancelled,
        (
            super::data::RuntimeAuthorityUnitState::Preparing
            | super::data::RuntimeAuthorityUnitState::Sealed
            | super::data::RuntimeAuthorityUnitState::Running,
            super::data::RuntimeAuthorityTransition::Fail,
        ) => super::data::RuntimeAuthorityUnitState::Failed,
        (
            super::data::RuntimeAuthorityUnitState::Sealed
            | super::data::RuntimeAuthorityUnitState::Running,
            super::data::RuntimeAuthorityTransition::AttemptMutation(_),
        ) => {
            return Err(error(
                super::data::RuntimeAuthorityIssueCode::PostSealWidening,
                FIELD_STATE,
            ));
        }
        (
            super::data::RuntimeAuthorityUnitState::Completed
            | super::data::RuntimeAuthorityUnitState::Cancelled
            | super::data::RuntimeAuthorityUnitState::Failed,
            _,
        ) => {
            return Err(error(
                super::data::RuntimeAuthorityIssueCode::StaleUnitReuse,
                FIELD_STATE,
            ));
        }
        (
            _,
            super::data::RuntimeAuthorityTransition::StartEffect
            | super::data::RuntimeAuthorityTransition::Complete
            | super::data::RuntimeAuthorityTransition::Cancel,
        )
        | (
            super::data::RuntimeAuthorityUnitState::Preparing,
            super::data::RuntimeAuthorityTransition::AttemptMutation(_),
        ) => {
            return Err(error(
                super::data::RuntimeAuthorityIssueCode::InvalidStateTransition,
                FIELD_STATE,
            ));
        }
    };
    let next = unit.with_state(next_state, unit.seal_receipt_identity().map(String::from));
    debug_assert_eq!(next.unit_identity(), unit.unit_identity());
    debug_assert!(next.state().is_terminal());
    Ok(next)
}

fn fixed_identities(
    draft: super::data::RuntimeAuthorityUnitDraft<'_>,
) -> [&str; super::data::FIXED_AUTHORITY_IDENTITY_COUNT] {
    [
        draft.session_lineage_identity,
        draft.durable_binding_identity,
        draft.authority_profile_identity,
        draft.enforcement_observation_identity,
        draft.operation_set_identity,
        draft.adapter_cohort_identity,
        draft.resource_profile_identity,
        draft.resource_budget_identity,
        draft.runtime_services_identity,
    ]
}

type ExpectedViewMap<'a> =
    alloc::collections::BTreeMap<super::data::RuntimeAuthorityViewClass, &'a str>;
type ObservedViewMap<'a> = alloc::collections::BTreeMap<
    super::data::RuntimeAuthorityViewClass,
    (&'a str, super::data::RuntimeAuthorityObservationStatus),
>;

fn compare_runtime_views(
    draft: super::data::RuntimeAuthorityUnitDraft<'_>,
) -> (
    Vec<super::data::RuntimeAuthorityViewEntry>,
    Vec<super::data::RuntimeAuthorityIssue>,
) {
    debug_assert_eq!(
        super::data::RuntimeAuthorityViewClass::ALL.len(),
        super::data::FIXED_AUTHORITY_IDENTITY_COUNT
    );
    debug_assert!(
        super::data::MAX_RUNTIME_AUTHORITY_VIEWS
            >= super::data::RuntimeAuthorityViewClass::ALL.len()
    );
    let (expected_map, mut issues) = expected_view_map(draft.expected_views);
    let (observed_map, observed_issues) = observed_view_map(draft.observed_views);
    issues.extend(observed_issues);
    if draft.expected_views.len() > super::data::MAX_RUNTIME_AUTHORITY_VIEWS
        || draft.observed_views.len() > super::data::MAX_RUNTIME_AUTHORITY_VIEWS
    {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::ItemLimitExceeded,
            FIELD_EXPECTED_VIEWS,
        ));
    }
    for class in super::data::RuntimeAuthorityViewClass::ALL {
        if !expected_map.contains_key(&class) {
            issues.push(view_issue(
                super::data::RuntimeAuthorityIssueCode::MissingView,
                FIELD_EXPECTED_VIEWS,
                class,
            ));
        }
    }
    issues.extend(compare_current_views(draft, &expected_map, &observed_map));
    for class in observed_map.keys() {
        if !expected_map.contains_key(class) {
            issues.push(view_issue(
                super::data::RuntimeAuthorityIssueCode::UnexpectedView,
                FIELD_OBSERVED_VIEWS,
                *class,
            ));
        }
    }
    let entries = expected_map
        .into_iter()
        .map(|(class, identity)| {
            super::data::RuntimeAuthorityViewEntry::new(class, String::from(identity))
        })
        .collect();
    (entries, issues)
}

fn expected_view_map<'a>(
    views: &[super::data::RuntimeAuthorityExpectedViewDraft<'a>],
) -> (ExpectedViewMap<'a>, Vec<super::data::RuntimeAuthorityIssue>) {
    let mut issues = Vec::with_capacity(super::data::MAX_RUNTIME_AUTHORITY_VIEWS);
    let mut view_map = alloc::collections::BTreeMap::new();
    for view in views.iter().take(super::data::MAX_RUNTIME_AUTHORITY_VIEWS) {
        if !is_valid_hash(view.identity) {
            issues.push(issue(
                super::data::RuntimeAuthorityIssueCode::UnsafeIdentity,
                FIELD_EXPECTED_VIEWS,
            ));
        }
        if view_map.insert(view.class, view.identity).is_some() {
            issues.push(view_issue(
                super::data::RuntimeAuthorityIssueCode::DuplicateView,
                FIELD_EXPECTED_VIEWS,
                view.class,
            ));
        }
    }
    debug_assert!(view_map.len() <= views.len());
    debug_assert!(
        issues
            .iter()
            .all(|issue| issue.field() == FIELD_EXPECTED_VIEWS)
    );
    (view_map, issues)
}

fn observed_view_map<'a>(
    views: &[super::data::RuntimeAuthorityObservedViewDraft<'a>],
) -> (ObservedViewMap<'a>, Vec<super::data::RuntimeAuthorityIssue>) {
    let mut issues = Vec::with_capacity(super::data::MAX_RUNTIME_AUTHORITY_VIEWS);
    let mut view_map = alloc::collections::BTreeMap::new();
    for view in views.iter().take(super::data::MAX_RUNTIME_AUTHORITY_VIEWS) {
        if !is_valid_hash(view.identity) {
            issues.push(issue(
                super::data::RuntimeAuthorityIssueCode::UnsafeIdentity,
                FIELD_OBSERVED_VIEWS,
            ));
        }
        if view_map
            .insert(view.class, (view.identity, view.status))
            .is_some()
        {
            issues.push(view_issue(
                super::data::RuntimeAuthorityIssueCode::DuplicateView,
                FIELD_OBSERVED_VIEWS,
                view.class,
            ));
        }
    }
    debug_assert!(view_map.len() <= views.len());
    debug_assert!(
        issues
            .iter()
            .all(|issue| issue.field() == FIELD_OBSERVED_VIEWS)
    );
    (view_map, issues)
}

fn compare_current_views(
    draft: super::data::RuntimeAuthorityUnitDraft<'_>,
    expected: &ExpectedViewMap<'_>,
    observed: &ObservedViewMap<'_>,
) -> Vec<super::data::RuntimeAuthorityIssue> {
    debug_assert!(expected.len() <= draft.expected_views.len());
    debug_assert!(observed.len() <= draft.observed_views.len());
    let mut issues = Vec::with_capacity(super::data::MAX_RUNTIME_AUTHORITY_VIEWS);
    for (class, expected_identity) in expected {
        if *expected_identity != declared_view_identity(draft, *class) {
            issues.push(view_issue(
                super::data::RuntimeAuthorityIssueCode::ViewIdentityMismatch,
                FIELD_EXPECTED_VIEWS,
                *class,
            ));
        }
        let Some((observed_identity, status)) = observed.get(class) else {
            issues.push(view_issue(
                super::data::RuntimeAuthorityIssueCode::MissingView,
                FIELD_OBSERVED_VIEWS,
                *class,
            ));
            continue;
        };
        if let Some(code) = observation_issue_code(*status) {
            issues.push(view_issue(code, FIELD_OBSERVED_VIEWS, *class));
        }
        if expected_identity != observed_identity {
            issues.push(view_issue(
                super::data::RuntimeAuthorityIssueCode::ViewIdentityMismatch,
                FIELD_OBSERVED_VIEWS,
                *class,
            ));
        }
    }
    issues
}

const fn observation_issue_code(
    status: super::data::RuntimeAuthorityObservationStatus,
) -> Option<super::data::RuntimeAuthorityIssueCode> {
    match status {
        super::data::RuntimeAuthorityObservationStatus::Current => None,
        super::data::RuntimeAuthorityObservationStatus::Stale => {
            Some(super::data::RuntimeAuthorityIssueCode::StaleView)
        }
        super::data::RuntimeAuthorityObservationStatus::Unknown => {
            Some(super::data::RuntimeAuthorityIssueCode::UnknownView)
        }
        super::data::RuntimeAuthorityObservationStatus::Unsupported => {
            Some(super::data::RuntimeAuthorityIssueCode::UnsupportedView)
        }
    }
}

fn declared_view_identity(
    draft: super::data::RuntimeAuthorityUnitDraft<'_>,
    class: super::data::RuntimeAuthorityViewClass,
) -> &str {
    match class {
        super::data::RuntimeAuthorityViewClass::SessionLineage => draft.session_lineage_identity,
        super::data::RuntimeAuthorityViewClass::DurableBinding => draft.durable_binding_identity,
        super::data::RuntimeAuthorityViewClass::AuthorityProfile => {
            draft.authority_profile_identity
        }
        super::data::RuntimeAuthorityViewClass::EnforcementObservation => {
            draft.enforcement_observation_identity
        }
        super::data::RuntimeAuthorityViewClass::OperationSet => draft.operation_set_identity,
        super::data::RuntimeAuthorityViewClass::AdapterCohort => draft.adapter_cohort_identity,
        super::data::RuntimeAuthorityViewClass::ResourceProfile => draft.resource_profile_identity,
        super::data::RuntimeAuthorityViewClass::ResourceBudget => draft.resource_budget_identity,
        super::data::RuntimeAuthorityViewClass::RuntimeServices => draft.runtime_services_identity,
    }
}

#[derive(Clone, Copy)]
struct HashListPolicy {
    field: &'static str,
    maximum: usize,
    require_non_empty: bool,
    duplicate_code: super::data::RuntimeAuthorityIssueCode,
}

fn normalize_hashes(
    values: &[&str],
    policy: HashListPolicy,
) -> (Vec<String>, Vec<super::data::RuntimeAuthorityIssue>) {
    debug_assert!(!policy.field.is_empty());
    debug_assert!(policy.maximum > 0);
    let mut issues = Vec::with_capacity(policy.maximum);
    if values.len() > policy.maximum {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::ItemLimitExceeded,
            policy.field,
        ));
    }
    if policy.require_non_empty && values.is_empty() {
        issues.push(issue(
            super::data::RuntimeAuthorityIssueCode::EmptyOperationSet,
            policy.field,
        ));
    }
    for value in values.iter().take(policy.maximum) {
        if !is_valid_hash(value) {
            issues.push(issue(
                super::data::RuntimeAuthorityIssueCode::UnsafeIdentity,
                policy.field,
            ));
        }
    }
    let normalized = values
        .iter()
        .take(policy.maximum)
        .copied()
        .collect::<alloc::collections::BTreeSet<_>>();
    if normalized.len() != values.len() {
        issues.push(issue(policy.duplicate_code, policy.field));
    }
    (normalized.into_iter().map(String::from).collect(), issues)
}

fn unit_identity(
    draft: super::data::RuntimeAuthorityUnitDraft<'_>,
    operations: &[String],
    resources: &[String],
    views: &[super::data::RuntimeAuthorityViewEntry],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(super::data::RUNTIME_AUTHORITY_SEAL_HASH_DOMAIN.as_bytes());
    let fixed = [
        super::data::RUNTIME_AUTHORITY_SEAL_SCHEMA,
        draft.session_id,
        draft.dispatch_id,
        draft.session_lineage_identity,
        draft.durable_binding_identity,
        draft.authority_profile_identity,
        draft.enforcement_observation_identity,
        draft.operation_set_identity,
        draft.adapter_cohort_identity,
        draft.resource_profile_identity,
        draft.resource_budget_identity,
        draft.runtime_services_identity,
    ];
    for part in fixed {
        update_hash(&mut hasher, part);
    }
    for operation in operations {
        update_hash(&mut hasher, operation);
    }
    for resource in resources {
        update_hash(&mut hasher, resource);
    }
    for view in views {
        update_hash(&mut hasher, view.class().as_str());
        update_hash(&mut hasher, view.identity());
    }
    let identity = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(identity.len(), super::data::HASH_HEX_BYTES);
    identity
}

pub(super) fn receipt_identity(receipt: &super::data::RuntimeAuthoritySealReceipt) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(super::data::RUNTIME_AUTHORITY_SEAL_RECEIPT_HASH_DOMAIN.as_bytes());
    let fixed = [
        receipt.schema,
        receipt.unit_identity.as_str(),
        receipt.session_id.as_str(),
        receipt.dispatch_id.as_str(),
        receipt.session_lineage_identity.as_str(),
        receipt.authority_profile_identity.as_str(),
        receipt.durable_binding_identity.as_str(),
        receipt.enforcement_observation_identity.as_str(),
        receipt.operation_set_identity.as_str(),
        receipt.adapter_cohort_identity.as_str(),
        receipt.resource_profile_identity.as_str(),
        receipt.resource_budget_identity.as_str(),
        receipt.runtime_services_identity.as_str(),
        receipt.state.as_str(),
        if receipt.commit_required_before_effect {
            "commit-required"
        } else {
            "commit-not-required"
        },
        receipt.redaction_marker,
    ];
    for part in fixed {
        update_hash(&mut hasher, part);
    }
    for operation in &receipt.operation_identities {
        update_hash(&mut hasher, operation);
    }
    for resource in &receipt.resource_identities {
        update_hash(&mut hasher, resource);
    }
    for view in &receipt.runtime_views {
        update_hash(&mut hasher, view.class().as_str());
        update_hash(&mut hasher, view.identity());
    }
    for non_claim in &receipt.non_claims {
        update_hash(&mut hasher, non_claim);
    }
    let identity = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(identity.len(), super::data::HASH_HEX_BYTES);
    identity
}

fn update_hash(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(super::data::HASH_SEPARATOR);
    hasher.update(value.as_bytes());
}

fn is_subset(values: &[String], ceiling: &[String]) -> bool {
    values.iter().all(|value| ceiling.contains(value))
}

fn is_valid_hash(value: &str) -> bool {
    let has_expected_hex_bytes = value.len() == super::data::HASH_HEX_BYTES;
    let is_lowercase_hex = value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    has_expected_hex_bytes && is_lowercase_hex
}

fn is_safe_label(value: &str) -> bool {
    let has_safe_label_bytes =
        !value.is_empty() && value.len() <= super::data::MAX_SAFE_LABEL_BYTES;
    let has_safe_characters = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'));
    has_safe_label_bytes && has_safe_characters
}

fn issue(
    code: super::data::RuntimeAuthorityIssueCode,
    field: &'static str,
) -> super::data::RuntimeAuthorityIssue {
    super::data::RuntimeAuthorityIssue::new(code, field, None)
}

fn view_issue(
    code: super::data::RuntimeAuthorityIssueCode,
    field: &'static str,
    class: super::data::RuntimeAuthorityViewClass,
) -> super::data::RuntimeAuthorityIssue {
    super::data::RuntimeAuthorityIssue::new(code, field, Some(class))
}

fn error(
    code: super::data::RuntimeAuthorityIssueCode,
    field: &'static str,
) -> super::data::RuntimeAuthoritySealError {
    super::data::RuntimeAuthoritySealError::from_issues(vec![issue(code, field)])
}
