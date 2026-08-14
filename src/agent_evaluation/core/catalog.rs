mod live;

const CASE_DOMAIN: &str = "lattice.agent-evaluation.case-revision.v1";
const CATALOG_DOMAIN: &str = "lattice.agent-evaluation.catalog.v1";
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "agent_correctness_not_proven",
    "verifier_soundness_not_proven",
    "judge_correctness_not_proven",
    "benchmark_superiority_not_proven",
    "deployment_correctness_not_proven",
    "release_eligibility_not_proven",
];

// r[impl lattice.agent_evaluation.catalog]
// r[impl lattice.agent_evaluation.case_contract]
pub fn validate(
    catalog: super::model::Catalog,
    assets: &[super::model::AssetFact],
) -> Result<super::model::AdmittedCatalog, super::Error> {
    debug_assert!(!super::model::CATALOG_SCHEMA.is_empty());
    let is_invalid_schema = catalog.schema != super::model::CATALOG_SCHEMA;
    let is_invalid_case_count = catalog.cases.is_empty() || catalog.cases.len() > super::model::MAX_CASES;
    if is_invalid_schema || is_invalid_case_count {
        return Err(super::Error::InvalidCatalog);
    }
    debug_assert!(!catalog.cases.is_empty());
    debug_assert!(catalog.cases.len() <= super::model::MAX_CASES);
    if assets.is_empty() || assets.len() > super::model::MAX_ASSETS {
        return Err(super::Error::AssetParity);
    }
    let mut case_ids = std::collections::BTreeSet::new();
    let mut case_revisions = std::collections::BTreeSet::new();
    let mut claimed_assets = std::collections::BTreeSet::new();
    let mut admitted = Vec::with_capacity(catalog.cases.len());
    for case in catalog.cases {
        validate_case(&case)?;
        if !case_ids.insert(case.id.clone()) || !claimed_assets.insert(case.asset_path.clone()) {
            return Err(super::Error::Duplicate);
        }
        let asset =
            assets.iter().find(|fact| fact.relative_path == case.asset_path).ok_or(super::Error::AssetParity)?;
        validate_asset(asset, &case.asset_identity)?;
        let revision_identity = super::identity::hash_value(CASE_DOMAIN, &case)?;
        if !case_revisions.insert(revision_identity.clone()) {
            return Err(super::Error::Duplicate);
        }
        admitted.push(super::model::AdmittedCase {
            definition: case,
            revision_identity,
        });
    }
    let has_claim_count_mismatch = claimed_assets.len() != assets.len();
    let has_unclaimed_asset = assets.iter().any(|asset| !claimed_assets.contains(&asset.relative_path));
    if has_claim_count_mismatch || has_unclaimed_asset {
        return Err(super::Error::AssetParity);
    }
    admitted.sort_by(|left, right| left.definition.id.cmp(&right.definition.id));
    let identity_material = admitted.iter().map(|case| case.revision_identity.as_str()).collect::<Vec<_>>();
    let identity = super::identity::hash_value(CATALOG_DOMAIN, &identity_material)?;
    debug_assert_eq!(admitted.len(), case_ids.len());
    debug_assert_eq!(admitted.len(), claimed_assets.len());
    Ok(super::model::AdmittedCatalog {
        cases: admitted,
        identity,
    })
}

fn validate_case(case: &super::model::CaseDefinition) -> Result<(), super::Error> {
    debug_assert!(!case.id.is_empty());
    debug_assert!(!case.asset_path.is_empty());
    if !super::identity::valid_public_label(&case.id) || !super::identity::valid_safe_path(&case.asset_path) {
        return Err(super::Error::InvalidCase);
    }
    validate_identity_fields(case)?;
    validate_budget(case.budget)?;
    validate_subject(case)?;
    validate_environment(case)?;
    validate_oracles(case)?;
    validate_artifact_policy(case)?;
    live::validate_posture(case)?;
    let has_too_few_non_claims = case.non_claims.len() < REQUIRED_NON_CLAIMS.len();
    let has_too_many_non_claims = case.non_claims.len() > super::model::MAX_NON_CLAIMS;
    if has_too_few_non_claims || has_too_many_non_claims {
        return Err(super::Error::NonClaim);
    }
    for required in REQUIRED_NON_CLAIMS {
        if !case.non_claims.iter().any(|claim| claim == required) {
            return Err(super::Error::NonClaim);
        }
    }
    debug_assert!(case.non_claims.len() >= REQUIRED_NON_CLAIMS.len());
    debug_assert!(case.non_claims.len() <= super::model::MAX_NON_CLAIMS);
    Ok(())
}

fn validate_identity_fields(case: &super::model::CaseDefinition) -> Result<(), super::Error> {
    for identity in [
        case.asset_identity.as_str(),
        case.subject.repository_identity.as_str(),
        case.subject.workload_revision.as_str(),
        case.subject.fixture_identity.as_str(),
        case.subject.scaffold_identity.as_str(),
        case.subject.initial_state_identity.as_str(),
        case.agent_profile_identity.as_str(),
        case.environment.identity.as_str(),
        case.policy_identity.as_str(),
        case.artifacts.identity.as_str(),
    ] {
        super::identity::validate(identity)?;
    }
    Ok(())
}

fn validate_budget(budget: super::model::TrialBudget) -> Result<(), super::Error> {
    let is_invalid_trial_count = budget.trials == 0 || budget.trials > super::model::MAX_TRIALS_PER_CASE;
    let is_invalid_concurrency = budget.concurrency == 0
        || budget.concurrency > super::model::MAX_CONCURRENCY
        || budget.concurrency > budget.trials;
    let is_invalid_token_count = budget.max_tokens == 0 || budget.max_tokens > super::model::MAX_TOKENS;
    let is_invalid_duration = budget.max_duration_ms == 0 || budget.max_duration_ms > super::model::MAX_DURATION_MS;
    let is_invalid_deployment_count = budget.max_deployments > super::model::MAX_DEPLOYMENTS;
    let has_invalid_budget = [
        is_invalid_trial_count,
        is_invalid_concurrency,
        is_invalid_token_count,
        is_invalid_duration,
        is_invalid_deployment_count,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_budget {
        return Err(super::Error::InvalidBudget);
    }
    debug_assert!(budget.trials <= super::model::MAX_TRIALS_PER_CASE);
    debug_assert!(budget.concurrency <= budget.trials);
    debug_assert!(budget.max_tokens <= super::model::MAX_TOKENS);
    debug_assert!(budget.max_duration_ms <= super::model::MAX_DURATION_MS);
    debug_assert!(budget.max_deployments <= super::model::MAX_DEPLOYMENTS);
    Ok(())
}

fn validate_subject(case: &super::model::CaseDefinition) -> Result<(), super::Error> {
    let is_invalid_mutation_scope = case.subject.mutation_scope.is_empty()
        || case.subject.mutation_scope.iter().any(|path| !super::identity::valid_safe_path(path));
    if is_invalid_mutation_scope {
        return Err(super::Error::InvalidSubject);
    }
    let is_missing_frozen_reference =
        case.subject.hidden_material == super::model::HiddenMaterialPosture::Frozen && case.references.is_empty();
    if is_missing_frozen_reference {
        return Err(super::Error::InvalidSubject);
    }
    Ok(())
}

fn validate_environment(case: &super::model::CaseDefinition) -> Result<(), super::Error> {
    if !super::identity::valid_public_label(&case.environment.target_class) {
        return Err(super::Error::InvalidEnvironment);
    }
    match case.environment.network {
        super::model::NetworkPosture::None => {}
        super::model::NetworkPosture::Isolated if case.environment.isolated => {}
        super::model::NetworkPosture::Live if case.environment.isolated && case.live.enabled => {}
        _ => return Err(super::Error::InvalidEnvironment),
    }
    Ok(())
}

fn validate_oracles(case: &super::model::CaseDefinition) -> Result<(), super::Error> {
    debug_assert!(case.gates.len() <= super::model::MAX_ASSETS);
    debug_assert!(case.references.len() <= super::model::MAX_ASSETS);
    if case.gates.is_empty() || case.references.is_empty() {
        return Err(super::Error::InvalidOracle);
    }
    let mut gate_ids = std::collections::BTreeSet::new();
    for gate in &case.gates {
        let is_invalid_gate_label = !super::identity::valid_public_label(&gate.id)
            || !super::identity::valid_public_label(&gate.evidence_schema);
        if is_invalid_gate_label || !gate.objective {
            return Err(super::Error::InvalidOracle);
        }
        super::identity::validate(&gate.implementation_identity)?;
        super::identity::validate(&gate.input_identity)?;
        if !gate_ids.insert(&gate.id) {
            return Err(super::Error::Duplicate);
        }
    }
    if let Some(rubric) = &case.rubric {
        super::identity::validate(&rubric.identity)?;
        super::identity::validate(&rubric.judge_profile_identity)?;
        if !rubric.advisory {
            return Err(super::Error::InvalidOracle);
        }
    }
    for reference in &case.references {
        super::identity::validate(&reference.identity)?;
        super::identity::validate(&reference.source_identity)?;
        let is_stale_required_reference = reference.freshness_required && !reference.fresh;
        let is_invalid_capture = !super::identity::valid_public_label(&reference.capture_class);
        let has_invalid_reference = [
            is_invalid_capture,
            reference.agent_controlled,
            is_stale_required_reference,
        ]
        .into_iter()
        .any(|value| value);
        if has_invalid_reference {
            return Err(super::Error::InvalidOracle);
        }
    }
    debug_assert_eq!(gate_ids.len(), case.gates.len());
    debug_assert!(!case.references.is_empty());
    Ok(())
}

fn validate_artifact_policy(case: &super::model::CaseDefinition) -> Result<(), super::Error> {
    let policy = &case.artifacts;
    let is_missing_allowlist = policy.allowed_roles.is_empty() || policy.allowed_media.is_empty();
    let is_invalid_count = policy.max_count == 0 || policy.max_count > super::model::MAX_ARTIFACTS;
    let is_invalid_bytes = policy.max_bytes == 0 || policy.max_bytes > super::model::MAX_ARTIFACT_BYTES;
    let is_unsafe_destination = !policy.private_destination || !policy.redaction_required;
    let has_invalid_policy = [
        is_missing_allowlist,
        is_invalid_count,
        is_invalid_bytes,
        is_unsafe_destination,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_policy {
        return Err(super::Error::InvalidArtifact);
    }
    Ok(())
}

fn validate_asset(asset: &super::model::AssetFact, expected_identity: &str) -> Result<(), super::Error> {
    let is_invalid_file_type = !asset.regular_file || asset.symlink;
    let is_invalid_bytes = asset.byte_count == 0;
    let is_invalid_path = !super::identity::valid_safe_path(&asset.relative_path);
    let is_identity_mismatch = asset.content_identity != expected_identity;
    let has_invalid_asset = [
        is_invalid_file_type,
        is_invalid_bytes,
        is_invalid_path,
        is_identity_mismatch,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_asset {
        return Err(super::Error::AssetParity);
    }
    super::identity::validate(&asset.content_identity)
}
