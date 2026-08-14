#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Admission {
    pub case_revision_identity: String,
    pub actor: String,
    pub target_namespace: String,
    pub credential_handle_count: u32,
    pub maximum_deployments: u32,
    pub identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationClass {
    Deployed,
    Validated,
    PartiallyObserved,
    Failed,
    Cancelled,
    CleanupRequired,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Observation {
    pub admission_identity: String,
    pub class: ObservationClass,
    pub deployment_identity: Option<String>,
    pub cleanup_confirmed: bool,
}

// r[impl lattice.agent_evaluation.live]
pub fn admit(
    case: &super::model::AdmittedCase,
    fact: &super::planning::LiveAuthorityFact,
) -> Result<Admission, super::Error> {
    debug_assert!(case.definition.budget.max_deployments <= super::model::MAX_DEPLOYMENTS);
    debug_assert!(!case.revision_identity.is_empty());
    let posture = &case.definition.live;
    let is_actor_mismatch = posture.operator_actor.as_deref() != Some(fact.actor.as_str());
    let is_target_mismatch = posture.target_namespace.as_deref() != Some(fact.target_namespace.as_str());
    let is_credential_mismatch = posture.credential_handles != fact.credential_handles;
    let is_invalid_deployment_limit =
        fact.max_deployments == 0 || fact.max_deployments > case.definition.budget.max_deployments;
    let is_invalid_actor = !super::identity::valid_public_label(&fact.actor);
    let is_unsafe_target =
        !super::identity::valid_public_label(&fact.target_namespace) || fact.target_namespace.contains("prod");
    let is_missing_credentials = fact.credential_handles.is_empty();
    let has_invalid_admission = [
        !posture.enabled,
        is_actor_mismatch,
        is_target_mismatch,
        is_credential_mismatch,
        is_invalid_deployment_limit,
        is_invalid_actor,
        is_unsafe_target,
        is_missing_credentials,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_admission {
        return Err(super::Error::AuthorityDenied);
    }
    let credential_handle_count =
        u32::try_from(fact.credential_handles.len()).map_err(|_| super::Error::AuthorityDenied)?;
    let identity = super::identity::hash_value(
        "lattice.agent-evaluation.live-admission.v1",
        &(
            case.revision_identity.as_str(),
            fact.actor.as_str(),
            fact.target_namespace.as_str(),
            &fact.credential_handles,
            fact.max_deployments,
        ),
    )?;
    debug_assert!(credential_handle_count > 0);
    debug_assert!(fact.max_deployments > 0);
    Ok(Admission {
        case_revision_identity: case.revision_identity.clone(),
        actor: fact.actor.clone(),
        target_namespace: fact.target_namespace.clone(),
        credential_handle_count,
        maximum_deployments: fact.max_deployments,
        identity,
    })
}

pub fn reduce(admission: &Admission, observation: &Observation) -> Result<super::model::TrialStatus, super::Error> {
    debug_assert!(!admission.identity.is_empty());
    debug_assert!(admission.maximum_deployments > 0);
    super::identity::validate(&observation.admission_identity)?;
    if observation.admission_identity != admission.identity {
        return Err(super::Error::LiveOutcomeUnknown);
    }
    if let Some(identity) = &observation.deployment_identity {
        super::identity::validate(identity)?;
    }
    let status = match observation.class {
        ObservationClass::Validated if observation.deployment_identity.is_some() && observation.cleanup_confirmed => {
            super::model::TrialStatus::Passed
        }
        ObservationClass::Deployed | ObservationClass::PartiallyObserved => super::model::TrialStatus::Partial,
        ObservationClass::Failed => super::model::TrialStatus::Failed,
        ObservationClass::Cancelled => super::model::TrialStatus::Cancelled,
        ObservationClass::CleanupRequired => super::model::TrialStatus::CleanupRequired,
        ObservationClass::Unknown => super::model::TrialStatus::Unknown,
        ObservationClass::Validated => super::model::TrialStatus::CleanupRequired,
    };
    debug_assert!(status != super::model::TrialStatus::Planned);
    debug_assert!(status != super::model::TrialStatus::Started);
    Ok(status)
}
