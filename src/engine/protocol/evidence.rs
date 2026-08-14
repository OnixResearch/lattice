//! Safe inspection and bounded protocol evidence.
//!
//! r[impl lattice.role_protocol.diagnostics]
//! r[impl lattice.role_protocol.evidence]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the evidence projector uses the protocol model vocabulary without changing domain ownership"
)]

use super::model::AttemptState;
use super::model::Error;
use super::model::Identity;
use super::model::Issue;
use super::model::SessionRecord;

const MAX_DIAGNOSTIC_TEXT_BYTES: usize = 512;
const FORBIDDEN_DIAGNOSTIC_FRAGMENTS: &[&str] = &[
    "/home/",
    "/etc/",
    "authorization:",
    "bearer ",
    "credential",
    "password",
    "secret",
    "token=",
    "http://",
    "https://",
];

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptInspection {
    pub identity: Identity,
    pub sequence: u64,
    pub state: AttemptState,
    pub message_count: usize,
    pub observation_count: usize,
    pub outcome: Option<Identity>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionInspection {
    pub schema: String,
    pub session: Identity,
    pub protocol: Identity,
    pub complete_local_set: Identity,
    pub projection_receipt: Identity,
    pub placement: Identity,
    pub session_record: Identity,
    pub event_chain: Identity,
    pub version: u64,
    pub status: super::model::SessionStatus,
    pub role_count: usize,
    pub attempts: Vec<AttemptInspection>,
    pub issue_codes: Vec<String>,
    pub claim_boundary: String,
}

pub fn inspect(session: &SessionRecord, issues: &[Issue]) -> Result<SessionInspection, Error> {
    Ok(SessionInspection {
        schema: super::model::EVIDENCE_SCHEMA.to_owned(),
        session: session.plan.session.clone(),
        protocol: session.plan.cohort.global_artifact.clone(),
        complete_local_set: session.plan.cohort.complete_local_set.clone(),
        projection_receipt: session.plan.cohort.projection_receipt.clone(),
        placement: super::identity::serialized_reference(super::identity::EVIDENCE_DOMAIN, &session.plan.roles)?,
        session_record: super::identity::serialized_reference(super::identity::EVIDENCE_DOMAIN, session)?,
        event_chain: super::identity::serialized_reference(super::identity::EVIDENCE_DOMAIN, &session.events)?,
        version: session.version,
        status: session.status,
        role_count: session.plan.roles.len(),
        attempts: session
            .attempts
            .iter()
            .map(|attempt| AttemptInspection {
                identity: attempt.identity.clone(),
                sequence: attempt.sequence,
                state: attempt.state,
                message_count: attempt.messages.len(),
                observation_count: attempt.observations.len(),
                outcome: attempt.outcome.clone(),
            })
            .collect(),
        issue_codes: issues.iter().map(|issue| issue.code().to_owned()).collect(),
        claim_boundary: super::model::CLAIM_BOUNDARY.to_owned(),
    })
}

pub fn identity(report: &SessionInspection) -> Result<Identity, Error> {
    super::identity::serialized_reference(super::identity::EVIDENCE_DOMAIN, report)
}

#[must_use]
pub fn safe_diagnostic_text(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= MAX_DIAGNOSTIC_TEXT_BYTES
        && !FORBIDDEN_DIAGNOSTIC_FRAGMENTS.iter().any(|fragment| text.to_ascii_lowercase().contains(fragment))
}
