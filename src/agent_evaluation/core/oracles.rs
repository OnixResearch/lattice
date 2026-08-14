#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateEvidence {
    pub gate_id: String,
    pub implementation_identity: String,
    pub input_identity: String,
    pub evidence_schema: String,
    pub status: super::model::GateStatus,
    pub agent_controlled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JudgeEvidence {
    pub rubric_identity: String,
    pub judge_profile_identity: String,
    pub score_basis_points: u16,
    pub advisory: bool,
    pub contains_private_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OracleDecision {
    pub status: super::model::TrialStatus,
    pub gate_set_identity: String,
    pub passed_gates: u32,
    pub failed_gates: u32,
    pub advisory_score_basis_points: Option<u16>,
}

// r[impl lattice.agent_evaluation.oracles]
pub fn reduce(
    case: &super::model::AdmittedCase,
    gate_evidence: &[GateEvidence],
    judge: Option<&JudgeEvidence>,
) -> Result<OracleDecision, super::Error> {
    debug_assert!(!case.definition.gates.is_empty());
    debug_assert!(case.definition.gates.len() <= super::model::MAX_ASSETS);
    if gate_evidence.len() != case.definition.gates.len() {
        return Err(super::Error::InvalidOracleEvidence);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut passed = 0u32;
    let mut failed = 0u32;
    for definition in &case.definition.gates {
        let evidence = gate_evidence
            .iter()
            .find(|evidence| evidence.gate_id == definition.id)
            .ok_or(super::Error::InvalidOracleEvidence)?;
        validate_gate_evidence(definition, evidence, &mut seen)?;
        match evidence.status {
            super::model::GateStatus::Passed => {
                passed = passed.checked_add(1).ok_or(super::Error::InvalidOracleEvidence)?;
            }
            super::model::GateStatus::Failed
            | super::model::GateStatus::Missing
            | super::model::GateStatus::Stale
            | super::model::GateStatus::TimedOut
            | super::model::GateStatus::Malformed
            | super::model::GateStatus::Contradictory => {
                failed = failed.checked_add(1).ok_or(super::Error::InvalidOracleEvidence)?;
            }
        }
    }
    let advisory_score_basis_points = validate_judge(case, judge)?;
    let gate_set_identity =
        super::identity::hash_value("lattice.agent-evaluation.gate-evidence-set.v1", gate_evidence)?;
    let gate_count = u32::try_from(gate_evidence.len()).map_err(|_| super::Error::InvalidOracleEvidence)?;
    debug_assert_eq!(seen.len(), gate_evidence.len());
    debug_assert_eq!(passed.checked_add(failed), Some(gate_count));
    Ok(OracleDecision {
        status: if failed == 0 {
            super::model::TrialStatus::Passed
        } else {
            super::model::TrialStatus::Failed
        },
        gate_set_identity,
        passed_gates: passed,
        failed_gates: failed,
        advisory_score_basis_points,
    })
}

fn validate_gate_evidence<'a>(
    definition: &'a super::model::GateDefinition,
    evidence: &'a GateEvidence,
    seen: &mut std::collections::BTreeSet<&'a String>,
) -> Result<(), super::Error> {
    let is_duplicate = !seen.insert(&evidence.gate_id);
    let is_implementation_mismatch = evidence.implementation_identity != definition.implementation_identity;
    let is_input_mismatch = evidence.input_identity != definition.input_identity;
    let is_schema_mismatch = evidence.evidence_schema != definition.evidence_schema;
    let has_invalid_evidence = [
        is_duplicate,
        is_implementation_mismatch,
        is_input_mismatch,
        is_schema_mismatch,
        evidence.agent_controlled,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_evidence {
        return Err(super::Error::InvalidOracleEvidence);
    }
    Ok(())
}

fn validate_judge(
    case: &super::model::AdmittedCase,
    judge: Option<&JudgeEvidence>,
) -> Result<Option<u16>, super::Error> {
    match (&case.definition.rubric, judge) {
        (None, None) | (Some(_), None) => Ok(None),
        (None, Some(_)) => Err(super::Error::InvalidJudgeEvidence),
        (Some(rubric), Some(evidence)) => {
            super::identity::validate(&evidence.rubric_identity)?;
            super::identity::validate(&evidence.judge_profile_identity)?;
            let is_rubric_mismatch = evidence.rubric_identity != rubric.identity;
            let is_judge_mismatch = evidence.judge_profile_identity != rubric.judge_profile_identity;
            let is_invalid_score = evidence.score_basis_points > super::model::MAX_RUBRIC_SCORE;
            let is_unsafe_evidence = !evidence.advisory || evidence.contains_private_content;
            let has_invalid_judgment = [
                is_rubric_mismatch,
                is_judge_mismatch,
                is_invalid_score,
                is_unsafe_evidence,
            ]
            .into_iter()
            .any(|value| value);
            if has_invalid_judgment {
                return Err(super::Error::InvalidJudgeEvidence);
            }
            Ok(Some(evidence.score_basis_points))
        }
    }
}
