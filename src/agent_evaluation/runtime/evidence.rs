pub const ANIMUS_FACT_SCHEMA: &str = "animus.native-evaluation-adapter.v1";
const MAX_ANIMUS_ITEMS: u32 = 256;
const MAX_ANIMUS_BYTES: u64 = 67_108_864;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimusFact {
    pub schema: String,
    pub task_identity: String,
    pub run_identity: String,
    pub artifact_manifest_identity: String,
    pub outcome_class: String,
    pub item_count: u32,
    pub byte_count: u64,
    pub contains_private_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Receipt {
    pub schema: String,
    pub catalog_identity: String,
    pub case_id: String,
    pub case_revision_identity: String,
    pub trial_identity: String,
    pub subject_identity: String,
    pub pre_state_identity: String,
    pub post_state_identity: String,
    pub agent_profile_identity: String,
    pub environment_identity: String,
    pub policy_identity: String,
    pub gate_set_identity: String,
    pub rubric_identity: Option<String>,
    pub reference_set_identity: String,
    pub artifact_manifest_identity: String,
    pub deployment_identity: Option<String>,
    pub status: super::model::TrialStatus,
    pub passed_gates: u32,
    pub failed_gates: u32,
    pub artifact_count: u32,
    pub artifact_bytes: u64,
    pub advisory_score_basis_points: Option<u16>,
    pub redaction: String,
    pub non_claims: Vec<String>,
    pub animus_fact_identity: Option<String>,
    pub identity: String,
}

pub struct ReceiptInput<'a> {
    pub catalog_identity: &'a str,
    pub case: &'a super::model::AdmittedCase,
    pub trial: &'a super::planning::TrialPlan,
    pub pre_state_identity: &'a str,
    pub post_state_identity: &'a str,
    pub policy_identity: &'a str,
    pub oracles: &'a super::oracles::OracleDecision,
    pub artifacts: &'a super::artifacts::ArtifactManifest,
    pub deployment_identity: Option<&'a str>,
    pub status: super::model::TrialStatus,
    pub animus_fact: Option<&'a AnimusFact>,
}

// r[impl lattice.agent_evaluation.evidence]
pub fn build_receipt(input: &ReceiptInput<'_>) -> Result<Receipt, super::Error> {
    debug_assert_eq!(input.trial.case_id, input.case.definition.id);
    debug_assert_eq!(input.trial.case_revision_identity, input.case.revision_identity);
    validate_receipt_inputs(input)?;
    let reference_set_identity =
        super::identity::hash_value("lattice.agent-evaluation.reference-set.v1", &input.case.definition.references)?;
    let animus_fact_identity = input.animus_fact.map(validate_animus_fact).transpose()?;
    let artifact_count = u32::try_from(input.artifacts.entries.len()).map_err(|_| super::Error::InvalidArtifact)?;
    let mut receipt = Receipt {
        schema: super::model::RECEIPT_SCHEMA.to_string(),
        catalog_identity: input.catalog_identity.to_string(),
        case_id: input.case.definition.id.clone(),
        case_revision_identity: input.case.revision_identity.clone(),
        trial_identity: input.trial.trial_identity.clone(),
        subject_identity: input.trial.subject_identity.clone(),
        pre_state_identity: input.pre_state_identity.to_string(),
        post_state_identity: input.post_state_identity.to_string(),
        agent_profile_identity: input.trial.agent_profile_identity.clone(),
        environment_identity: input.trial.environment_identity.clone(),
        policy_identity: input.policy_identity.to_string(),
        gate_set_identity: input.oracles.gate_set_identity.clone(),
        rubric_identity: input.case.definition.rubric.as_ref().map(|rubric| rubric.identity.clone()),
        reference_set_identity,
        artifact_manifest_identity: input.artifacts.identity.clone(),
        deployment_identity: input.deployment_identity.map(str::to_string),
        status: input.status,
        passed_gates: input.oracles.passed_gates,
        failed_gates: input.oracles.failed_gates,
        artifact_count,
        artifact_bytes: input.artifacts.total_bytes,
        advisory_score_basis_points: input.oracles.advisory_score_basis_points,
        redaction: "metadata_only".to_string(),
        non_claims: input.case.definition.non_claims.clone(),
        animus_fact_identity,
        identity: String::new(),
    };
    receipt.identity = super::identity::hash_value("lattice.agent-evaluation.receipt.v1", &receipt)?;
    debug_assert_eq!(receipt.artifact_count, artifact_count);
    debug_assert_eq!(receipt.redaction, "metadata_only");
    Ok(receipt)
}

pub fn validate_receipt(receipt: &Receipt) -> Result<(), super::Error> {
    debug_assert!(!super::model::RECEIPT_SCHEMA.is_empty());
    let is_invalid_schema = receipt.schema != super::model::RECEIPT_SCHEMA;
    let is_invalid_redaction = receipt.redaction != "metadata_only";
    let is_nonterminal =
        matches!(receipt.status, super::model::TrialStatus::Planned | super::model::TrialStatus::Started);
    let is_overbound_non_claims = receipt.non_claims.len() > super::model::MAX_NON_CLAIMS;
    let has_invalid_shape = [
        is_invalid_schema,
        is_invalid_redaction,
        is_nonterminal,
        is_overbound_non_claims,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_shape {
        return Err(super::Error::InvalidReceipt);
    }
    debug_assert!(receipt.non_claims.len() <= super::model::MAX_NON_CLAIMS);
    for identity in [
        receipt.catalog_identity.as_str(),
        receipt.case_revision_identity.as_str(),
        receipt.trial_identity.as_str(),
        receipt.subject_identity.as_str(),
        receipt.pre_state_identity.as_str(),
        receipt.post_state_identity.as_str(),
        receipt.agent_profile_identity.as_str(),
        receipt.environment_identity.as_str(),
        receipt.policy_identity.as_str(),
        receipt.gate_set_identity.as_str(),
        receipt.reference_set_identity.as_str(),
        receipt.artifact_manifest_identity.as_str(),
        receipt.identity.as_str(),
    ] {
        super::identity::validate(identity)?;
    }
    let mut material = receipt.clone();
    material.identity.clear();
    let expected = super::identity::hash_value("lattice.agent-evaluation.receipt.v1", &material)?;
    if expected != receipt.identity {
        return Err(super::Error::InvalidReceipt);
    }
    Ok(())
}

fn validate_receipt_inputs(input: &ReceiptInput<'_>) -> Result<(), super::Error> {
    debug_assert!(!input.catalog_identity.is_empty());
    debug_assert!(!input.policy_identity.is_empty());
    for identity in [
        input.catalog_identity,
        input.pre_state_identity,
        input.post_state_identity,
        input.policy_identity,
        input.oracles.gate_set_identity.as_str(),
        input.artifacts.identity.as_str(),
    ] {
        super::identity::validate(identity)?;
    }
    let is_case_mismatch = input.trial.case_revision_identity != input.case.revision_identity
        || input.trial.case_id != input.case.definition.id;
    let is_nonterminal_status =
        matches!(input.status, super::model::TrialStatus::Planned | super::model::TrialStatus::Started);
    let is_contradictory_pass = input.oracles.failed_gates > 0 && input.status == super::model::TrialStatus::Passed;
    let has_invalid_input =
        [is_case_mismatch, is_nonterminal_status, is_contradictory_pass].into_iter().any(|value| value);
    if has_invalid_input {
        return Err(super::Error::InvalidReceipt);
    }
    if let Some(deployment_identity) = input.deployment_identity {
        super::identity::validate(deployment_identity)?;
    }
    debug_assert_eq!(input.trial.policy_identity, input.policy_identity);
    debug_assert!(input.status != super::model::TrialStatus::Planned);
    Ok(())
}

fn validate_animus_fact(fact: &AnimusFact) -> Result<String, super::Error> {
    let is_invalid_schema = fact.schema != ANIMUS_FACT_SCHEMA;
    let is_invalid_bounds = fact.item_count > MAX_ANIMUS_ITEMS || fact.byte_count > MAX_ANIMUS_BYTES;
    let is_invalid_outcome = !super::identity::valid_public_label(&fact.outcome_class);
    let has_invalid_fact = [
        is_invalid_schema,
        is_invalid_bounds,
        is_invalid_outcome,
        fact.contains_private_content,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_fact {
        return Err(super::Error::InvalidAnimusFact);
    }
    debug_assert!(fact.item_count <= MAX_ANIMUS_ITEMS);
    debug_assert!(fact.byte_count <= MAX_ANIMUS_BYTES);
    for identity in [
        fact.task_identity.as_str(),
        fact.run_identity.as_str(),
        fact.artifact_manifest_identity.as_str(),
    ] {
        super::identity::validate(identity)?;
    }
    super::identity::hash_value("lattice.agent-evaluation.animus-fact.v1", fact)
}
