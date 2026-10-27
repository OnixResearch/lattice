//! Pure composition with the accepted Valence build-service profile.
//!
//! Producer request/attempt/authority records are supplied observations, not
//! invented by Lattice. Only Lattice orchestration and an acknowledged GitHub
//! status are constructed here. The companion receipt links native event and
//! result identities because the accepted profile has no event-identity field;
//! it never fabricates transition timestamps, build outputs, CAS observations,
//! machine selection, billing, or export truth.
// r[impl lattice.github_ci.evidence_boundary]

use serde::Serialize;
use valence_core::build_service::BuildServiceBundle;
use valence_core::build_service::BuildServiceRole;
use valence_core::build_service::{self as valence};

use super::core::Identity;
use super::core::Operation;
use super::core::Phase;
use super::core::Policy;
use super::session::ExchangeObservation;
use super::session::Record;

pub const EVIDENCE_BYTES_MAX: usize = 1_048_576;
pub const EVENTS_MAX: usize = 256;
pub const EXCHANGES_MAX: usize = 64;
const PROFILE_RECORDS_MAX: usize = 64;
const LINKAGE_SCHEMA: &str = "lattice.github-ci.valence-linkage.v1";
const LINKAGE_NON_CLAIM: &str = "Lattice records supplied public observations and canonical links only; this does not verify producer signatures, holder possession, current authority, external status truth, build correctness, hermeticity, host isolation, or release eligibility";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EvidenceError {
    #[error("evidence exceeds the admitted bounds")]
    Bounds,
    #[error("orchestration evidence does not bind the exact durable session")]
    Session,
    #[error("a current acknowledged Check Run observation is unavailable")]
    Unobserved,
    #[error("producer evidence policy or canonical record linkage differs")]
    Producer,
    #[error("authority evidence does not match the persisted exchange observations")]
    Exchange,
    #[error("the actual Valence build-service validator rejected the composed bundle")]
    Profile,
}

/// A per-event recorded observation. Result is the native result digest retained
/// by Mantle's verified successful event, not a fabricated Valence BuildResult.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EventLink {
    pub sequence: u64,
    pub event: Identity,
    pub phase: Phase,
    pub result: Option<Identity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExchangeLink {
    pub observation: ExchangeObservation,
    pub authority_record: Identity,
    /// The producer record's declared operation, not an inferred signature verdict.
    pub declared_operation: Operation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LinkageObservations {
    pub schema: &'static str,
    pub role: &'static str,
    pub workflow: Identity,
    pub matrix_job: Identity,
    pub planned_subject: Identity,
    pub workflow_generation: u32,
    pub producer_attempt_generation: u32,
    pub native_request: Identity,
    pub semantic_request: Identity,
    pub attempt: Identity,
    pub attempt_fence: u64,
    pub producer: Identity,
    pub service: Identity,
    pub project: Identity,
    pub owner: Identity,
    pub producer_policy: Identity,
    pub request_record: Identity,
    pub attempt_record: Identity,
    pub orchestration_record: Identity,
    pub external_status_record: Identity,
    pub check_run: u64,
    pub check_update: Identity,
    pub state_version: u64,
    pub exchanges: Vec<ExchangeLink>,
    pub events: Vec<EventLink>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LinkageReceipt {
    pub identity: Identity,
    pub observations: LinkageObservations,
}

/// Validated linkage only. The report is the real producer profile's report;
/// neither it nor this companion is live authority or signature evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Projection {
    pub bundle: BuildServiceBundle,
    pub report: valence::BuildServiceReport,
    pub linkage: LinkageReceipt,
}

/// Compose supplied producer records with an actual acknowledged session state.
/// `accepted_producer_policy` compares to the profile's **bare lowercase 64-hex**
/// identity. Basalt policy identities in authority records use the same explicit
/// bare spelling; verifier receipt and subject references retain `blake3:`.
/// There is no silent digest-spelling alias or whole-build trust promotion.
///
/// The producer bundle may rely on the appended real external-status observation
/// to satisfy its external-status class. All other claimed classes still need
/// their supplied actual records; the final real validator must accept all of it.
/// Workflow generation and producer attempt generation remain distinct fields.
pub fn project(
    record: &Record,
    producer: &BuildServiceBundle,
    accepted_producer_policy: Identity,
    policy: &Policy,
) -> Result<Projection, EvidenceError> {
    bound_record_counts(producer)?;
    bound_serialized(producer)?;
    if !bare_matches(&producer.profile.producer_policy, accepted_producer_policy) {
        return Err(EvidenceError::Producer);
    }
    let native = record.attempt.as_ref().ok_or(EvidenceError::Unobserved)?;
    let state = record.state.as_ref().ok_or(EvidenceError::Unobserved)?;
    if record.exchanges.is_empty()
        || record.exchanges.len() > policy.bounds.evidence.min(EXCHANGES_MAX)
        || state.history.len() > policy.bounds.event_history.min(EVENTS_MAX)
    {
        return Err(EvidenceError::Bounds);
    }
    state.validate(policy).map_err(|_| EvidenceError::Session)?;
    record.check.target.validate().map_err(|_| EvidenceError::Session)?;
    if state.workflow != record.workflow
        || state.job != record.job
        || state.generation != record.generation
        || state.idempotency != record.idempotency
        || state.binding != native.binding
        || native.native_request_hash != record.native_request_hash
        || state.binding.attempt != Identity(*blake3::hash(native.attempt.as_bytes()).as_bytes())
        || !mantle_build_contract::async_api::safe_segment(&native.attempt)
        || native.fence == 0
        || native.fence != state.binding.fence
        || state.binding.producer != record.producer
        || record.check.target.external_id != record.job
        || state.check_id != record.check.check_id
    {
        return Err(EvidenceError::Session);
    }
    let check_id = record.check.check_id.filter(|id| *id != 0).ok_or(EvidenceError::Unobserved)?;
    let applied = record.check.applied.as_ref().ok_or(EvidenceError::Unobserved)?;
    if record.check.pending.is_some() || applied.version != state.version || applied.terminal != state.phase.terminal()
    {
        return Err(EvidenceError::Unobserved);
    }
    let request = unique(producer.service_requests.iter().filter(|request| {
        bare_matches(&request.concrete_request_hash, record.native_request_hash)
            && bare_matches(&request.idempotency_key, record.idempotency)
    }))?;
    let request_identity = valence::service_request_identity(request);
    let attempt = unique(
        producer
            .attempts
            .iter()
            .filter(|attempt| attempt.attempt_id == native.attempt && attempt.request_identity == request_identity),
    )?;
    if !observational_role(request.role) || !observational_role(attempt.role) {
        return Err(EvidenceError::Producer);
    }
    let attempt_identity = valence::attempt_identity(attempt);
    let mut exchange_links = Vec::with_capacity(record.exchanges.len());
    let mut previous_receipt = None;
    let mut submitted = false;
    for observation in &record.exchanges {
        if observation.requested != observation.granted || previous_receipt == Some(observation.receipt) {
            return Err(EvidenceError::Exchange);
        }
        previous_receipt = Some(observation.receipt);
        let mut decisions = producer.authority_decisions.iter().filter(|decision| {
            decision.attempt_identity == attempt_identity
                && decision
                    .verifier_evidence_ref
                    .as_deref()
                    .is_some_and(|reference| framed_matches(reference, observation.receipt))
        });
        let decision = decisions.next().ok_or(EvidenceError::Exchange)?;
        if decisions.next().is_some()
            || !bare_matches(&decision.policy_identity, observation.policy)
            || !framed_matches(&decision.subject_projection, observation.subject)
            || decision.decision != "allow"
            || !observational_role(decision.role)
        {
            return Err(EvidenceError::Exchange);
        }
        let declared_operation = operation(&decision.operation)?;
        if declared_operation == Operation::Submit {
            if observation.subject != record.subject {
                return Err(EvidenceError::Exchange);
            }
            submitted = true;
        }
        exchange_links.push(ExchangeLink {
            observation: observation.clone(),
            authority_record: identity_from_bare(&valence::authority_decision_identity(decision))?,
            declared_operation,
        });
    }
    if !submitted {
        return Err(EvidenceError::Exchange);
    }
    let non_claims = vec![valence::BUILD_SERVICE_REQUIRED_NON_CLAIM.to_owned()];
    let job = valence::OrchestrationRecord {
        job_id: record.job.hex(),
        workflow_identity: record.workflow.hex(),
        matrix_job_identity: record.job.hex(),
        attempt_identity: attempt_identity.clone(),
        role: BuildServiceRole::Linked,
        non_claims: non_claims.clone(),
    };
    let job_identity = valence::orchestration_identity(&job);
    let status = valence::ExternalStatusRecord {
        status_id: format!(
            "github:{}/{}/check-run/{check_id}",
            record.check.target.owner, record.check.target.repository
        ),
        provider: "github-check-runs".to_owned(),
        job_identity: job_identity.clone(),
        attempt_identity: attempt_identity.clone(),
        observed_state: observed_state(state.phase).to_owned(),
        role: BuildServiceRole::RecordedOnly,
        non_claims,
    };
    let status_identity = valence::external_status_identity(&status);
    let mut bundle = producer.clone();
    if let Some(existing) = bundle.orchestration_jobs.iter().find(|existing| existing.job_id == job.job_id) {
        if existing != &job {
            return Err(EvidenceError::Producer);
        }
    } else {
        if bundle.orchestration_jobs.len() >= PROFILE_RECORDS_MAX {
            return Err(EvidenceError::Bounds);
        }
        bundle.orchestration_jobs.push(job);
    }
    if let Some(existing) = bundle.external_statuses.iter().find(|existing| existing.status_id == status.status_id) {
        if existing != &status {
            return Err(EvidenceError::Producer);
        }
    } else {
        if bundle.external_statuses.len() >= PROFILE_RECORDS_MAX {
            return Err(EvidenceError::Bounds);
        }
        bundle.external_statuses.push(status);
    }
    bound_serialized(&bundle)?;
    let report = valence::validate_build_service_bundle(&bundle);
    if !report.valid {
        return Err(EvidenceError::Profile);
    }
    let mut events = Vec::with_capacity(state.history.len());
    for event in state.history.values() {
        if (event.phase == Phase::Succeeded && event.evidence.len() != 1)
            || (event.phase != Phase::Succeeded && !event.evidence.is_empty())
        {
            return Err(EvidenceError::Session);
        }
        events.push(EventLink {
            sequence: event.sequence,
            event: event.event,
            phase: event.phase,
            result: event.evidence.iter().next().copied(),
        });
    }
    let observations = LinkageObservations {
        schema: LINKAGE_SCHEMA,
        role: "recorded_only",
        workflow: record.workflow,
        matrix_job: record.job,
        planned_subject: record.subject,
        workflow_generation: record.generation,
        producer_attempt_generation: attempt.generation,
        native_request: record.native_request_hash,
        semantic_request: state.binding.request,
        attempt: state.binding.attempt,
        attempt_fence: native.fence,
        producer: record.producer,
        service: record.service,
        project: record.project,
        owner: record.owner,
        producer_policy: accepted_producer_policy,
        request_record: identity_from_bare(&request_identity)?,
        attempt_record: identity_from_bare(&attempt_identity)?,
        orchestration_record: identity_from_bare(&job_identity)?,
        external_status_record: identity_from_bare(&status_identity)?,
        check_run: check_id,
        check_update: applied.update_id,
        state_version: state.version,
        exchanges: exchange_links,
        events,
        non_claims: vec![
            valence::BUILD_SERVICE_REQUIRED_NON_CLAIM.to_owned(),
            LINKAGE_NON_CLAIM.to_owned(),
        ],
    };
    bound_serialized(&observations)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(LINKAGE_SCHEMA.as_bytes());
    serde_json::to_writer(HashingWriter(&mut hasher), &observations).map_err(|_| EvidenceError::Bounds)?;
    Ok(Projection {
        bundle,
        report,
        linkage: LinkageReceipt {
            identity: Identity(*hasher.finalize().as_bytes()),
            observations,
        },
    })
}

fn unique<'a, T: 'a>(mut values: impl Iterator<Item = &'a T>) -> Result<&'a T, EvidenceError> {
    let value = values.next().ok_or(EvidenceError::Producer)?;
    if values.next().is_some() {
        return Err(EvidenceError::Producer);
    }
    Ok(value)
}

fn observational_role(role: BuildServiceRole) -> bool {
    matches!(role, BuildServiceRole::Linked | BuildServiceRole::RecordedOnly)
}

fn bare_matches(text: &str, identity: Identity) -> bool {
    text == blake3::Hash::from_bytes(identity.0).to_hex().as_str()
}

fn framed_matches(text: &str, identity: Identity) -> bool {
    text.strip_prefix("blake3:").is_some_and(|text| bare_matches(text, identity))
}

fn identity_from_bare(text: &str) -> Result<Identity, EvidenceError> {
    if text.len() != 64 || !text.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(EvidenceError::Producer);
    }
    let mut bytes = [0_u8; 32];
    hex::decode_to_slice(text, &mut bytes).map_err(|_| EvidenceError::Producer)?;
    Ok(Identity(bytes))
}

fn operation(ability: &str) -> Result<Operation, EvidenceError> {
    match ability {
        "mantle/build/submit" => Ok(Operation::Submit),
        "mantle/status/read" => Ok(Operation::Status),
        "mantle/event/read" => Ok(Operation::Events),
        "mantle/log/read" => Ok(Operation::Log),
        "mantle/result/read" => Ok(Operation::Result),
        "mantle/attempt/cancel-owned" => Ok(Operation::OwnedCancel),
        _ => Err(EvidenceError::Exchange),
    }
}

fn observed_state(phase: Phase) -> &'static str {
    match phase {
        Phase::Admitted | Phase::Queued => "queued",
        Phase::Running => "in_progress",
        Phase::Succeeded => "completed:success",
        Phase::Failed => "completed:failure",
        Phase::Cancelled => "completed:cancelled",
        Phase::Infrastructure => "completed:action_required",
    }
}

fn bound_record_counts(bundle: &BuildServiceBundle) -> Result<(), EvidenceError> {
    if [
        bundle.service_requests.len(),
        bundle.attempts.len(),
        bundle.transitions.len(),
        bundle.authority_decisions.len(),
        bundle.machine_classes.len(),
        bundle.resource_decisions.len(),
        bundle.resource_observations.len(),
        bundle.retry_edges.len(),
        bundle.usage_records.len(),
        bundle.build_results.len(),
        bundle.reuse_edges.len(),
        bundle.orchestration_jobs.len(),
        bundle.external_statuses.len(),
    ]
    .into_iter()
    .any(|length| length > PROFILE_RECORDS_MAX)
    {
        return Err(EvidenceError::Bounds);
    }
    Ok(())
}

// Pure memory sinks: count/hash serialized bytes without allocating an unbounded
// intermediate document, reading files, or performing any host I/O.
struct ByteCounter(usize);
impl std::io::Write for ByteCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|length| *length <= EVIDENCE_BYTES_MAX)
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::OutOfMemory))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct HashingWriter<'a>(&'a mut blake3::Hasher);
impl std::io::Write for HashingWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn bound_serialized(value: &impl Serialize) -> Result<(), EvidenceError> {
    serde_json::to_writer(ByteCounter(0), value).map_err(|_| EvidenceError::Bounds)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use super::super::core;
    use super::super::github;
    use super::super::mantle;
    use super::*;

    fn id(byte: u8) -> Identity {
        Identity([byte; 32])
    }
    fn fixture() -> (Record, BuildServiceBundle, Policy) {
        let policy = Policy {
            bounds: core::Bounds {
                matrix_axes: 4,
                matrix_values_per_axis: 4,
                matrix_jobs: 8,
                public_label_bytes: 128,
                event_history: 32,
                evidence: 8,
                annotations: 4,
                annotation_bytes: 256,
                title_bytes: 128,
                summary_bytes: 1024,
                text_bytes: 2048,
                url_bytes: 512,
                reconciliations: 4,
            },
            retry: core::RetryPolicy {
                max_retries: 3,
                initial_delay_ms: 100,
                max_delay_ms: 800,
                max_elapsed_ms: 2000,
            },
            capability_lifetime_ms: 10_000,
            public_urls: core::PublicUrlPolicy {
                allowed: BTreeSet::new(),
            },
        };
        let native_attempt = "attempt_7";
        let binding = core::AttemptBinding {
            request: id(3),
            attempt: mantle::public_identity(native_attempt.as_bytes()),
            fence: 9,
            producer: id(4),
        };
        let result = id(24);
        let state = core::DurableState {
            schema_version: core::STATE_SCHEMA_VERSION,
            workflow: id(1),
            job: id(2),
            generation: 7,
            idempotency: id(5),
            binding,
            check_id: Some(42),
            cursor: 3,
            version: 3,
            phase: Phase::Succeeded,
            history: BTreeMap::from([
                (1, core::EventFacts {
                    binding,
                    event: id(21),
                    sequence: 1,
                    phase: Phase::Queued,
                    evidence: BTreeSet::new(),
                }),
                (2, core::EventFacts {
                    binding,
                    event: id(22),
                    sequence: 2,
                    phase: Phase::Running,
                    evidence: BTreeSet::new(),
                }),
                (3, core::EventFacts {
                    binding,
                    event: id(23),
                    sequence: 3,
                    phase: Phase::Succeeded,
                    evidence: BTreeSet::from([result]),
                }),
            ]),
            evidence: BTreeSet::from([result]),
            reconciliations: 0,
            awaiting_reconciliation: false,
            cancellation_requested: false,
        };
        let observation = ExchangeObservation {
            receipt: id(10),
            policy: id(11),
            subject: id(12),
            requested: id(13),
            granted: id(13),
        };
        let record = Record {
            revision: 8,
            workflow: state.workflow,
            job: state.job,
            generation: state.generation,
            subject: observation.subject,
            idempotency: state.idempotency,
            native_request_hash: id(0xab),
            native_submission: id(0xac),
            service: id(7),
            project: id(8),
            owner: id(9),
            producer: binding.producer,
            attempt: Some(mantle::NativeAttempt {
                attempt: native_attempt.to_owned(),
                fence: binding.fence,
                native_request_hash: id(0xab),
                binding,
            }),
            state: Some(state),
            event_cursor: None,
            check: github::CheckRunBinding {
                target: github::CheckRunTarget {
                    owner: "OnixResearch".to_owned(),
                    repository: "private-ci".to_owned(),
                    head_sha: "a".repeat(40),
                    name: "Build".to_owned(),
                    external_id: id(2),
                },
                check_id: Some(42),
                applied: Some(github::AppliedProjection {
                    version: 3,
                    update_id: id(14),
                    terminal: true,
                }),
                pending: None,
            },
            exchanges: BTreeSet::from([observation.clone()]),
        };
        let non_claims = vec![valence::BUILD_SERVICE_REQUIRED_NON_CLAIM.to_owned()];
        let request = valence::ServiceRequestRecord {
            request_id: "native-request".to_owned(),
            concrete_request_hash: record.native_request_hash.hex(),
            idempotency_key: record.idempotency.hex(),
            role: BuildServiceRole::Linked,
            non_claims: non_claims.clone(),
        };
        let attempt = valence::AttemptRecord {
            attempt_id: native_attempt.to_owned(),
            request_identity: valence::service_request_identity(&request),
            generation: 1,
            parent_attempt_identity: None,
            service_policy: id(30).hex(),
            role: BuildServiceRole::RecordedOnly,
            non_claims: non_claims.clone(),
        };
        let authority = valence::AuthorityDecisionRecord {
            decision_id: "exchange-one".to_owned(),
            attempt_identity: valence::attempt_identity(&attempt),
            subject_projection: format!("blake3:{}", observation.subject.hex()),
            policy_identity: observation.policy.hex(),
            operation: Operation::Submit.name().to_owned(),
            decision: "allow".to_owned(),
            verifier_evidence_ref: Some(format!("blake3:{}", observation.receipt.hex())),
            role: BuildServiceRole::RecordedOnly,
            non_claims: non_claims.clone(),
        };
        let bundle = BuildServiceBundle {
            profile: valence::BuildServiceEvidenceProfile {
                profile_id: valence::BUILD_SERVICE_PROFILE_ID.to_owned(),
                profile_version: 1,
                canonicalization: valence::BUILD_SERVICE_CANONICALIZATION.to_owned(),
                producer_policy: id(30).hex(),
                claimed_observation_classes: vec!["external-status".to_owned()],
                non_claims,
            },
            service_requests: vec![request],
            attempts: vec![attempt],
            transitions: vec![],
            authority_decisions: vec![authority],
            machine_classes: vec![],
            resource_decisions: vec![],
            resource_observations: vec![],
            retry_edges: vec![],
            usage_records: vec![],
            build_results: vec![],
            reuse_edges: vec![],
            orchestration_jobs: vec![],
            external_statuses: vec![],
        };
        (record, bundle, policy)
    }

    // r[verify lattice.github_ci.evidence_boundary]
    #[test]
    fn actual_profile_links_acknowledged_status_and_exact_native_observations() {
        let (record, producer, policy) = fixture();
        let projected = project(&record, &producer, id(30), &policy).unwrap();
        assert!(valence::validate_build_service_bundle(&projected.bundle).valid);
        let links = &projected.linkage.observations;
        assert_eq!(links.workflow_generation, 7);
        assert_eq!(links.producer_attempt_generation, 1);
        assert_eq!(links.events.last().unwrap().result, Some(id(24)));
        assert_eq!(links.native_request, id(0xab));
        assert_eq!(links.semantic_request, id(3));
        assert_eq!(
            links.exchanges[0].authority_record.hex(),
            valence::authority_decision_identity(&producer.authority_decisions[0])
        );
        assert_eq!(projected.bundle.external_statuses[0].observed_state, "completed:success");
        assert_eq!(projected.bundle.external_statuses[0].role, BuildServiceRole::RecordedOnly);
        assert_eq!(projected.bundle.orchestration_jobs[0].attempt_identity, links.attempt_record.hex());
    }

    // r[verify lattice.github_ci.evidence_boundary]
    #[test]
    fn foreign_request_attempt_policy_and_digest_spellings_are_rejected() {
        let cases: &[fn(&mut Record, &mut BuildServiceBundle)] = &[
            |r, _| r.workflow = id(99),
            |r, _| r.attempt.as_mut().unwrap().attempt = "foreign-attempt".to_owned(),
            |_, b| b.profile.producer_policy = id(99).hex(),
            |_, b| b.service_requests[0].concrete_request_hash = id(99).hex(),
            |_, b| b.service_requests[0].idempotency_key = id(99).hex(),
            |_, b| b.attempts[0].request_identity = id(99).hex(),
            |_, b| b.profile.producer_policy = format!("blake3:{}", id(30).hex()),
            |_, b| b.service_requests[0].concrete_request_hash.make_ascii_uppercase(),
        ];
        for change in cases {
            let (mut record, mut bundle, policy) = fixture();
            change(&mut record, &mut bundle);
            assert!(matches!(
                project(&record, &bundle, id(30), &policy),
                Err(EvidenceError::Session | EvidenceError::Producer)
            ));
        }
    }

    // r[verify lattice.github_ci.evidence_boundary]
    #[test]
    fn every_exchange_requires_exact_receipt_subject_policy_and_observational_role() {
        let cases: &[fn(&mut BuildServiceBundle)] = &[
            |b| b.authority_decisions.clear(),
            |b| b.authority_decisions[0].verifier_evidence_ref = Some(format!("blake3:{}", id(99).hex())),
            |b| b.authority_decisions[0].subject_projection = format!("blake3:{}", id(99).hex()),
            |b| b.authority_decisions[0].policy_identity = id(99).hex(),
            |b| b.authority_decisions[0].operation = "mantle/*".to_owned(),
            |b| b.authority_decisions[0].operation = Operation::Status.name().to_owned(),
            |b| b.authority_decisions[0].decision = "deny".to_owned(),
            |b| b.authority_decisions[0].role = BuildServiceRole::Verified,
            |b| b.authority_decisions.push(b.authority_decisions[0].clone()),
        ];
        for change in cases {
            let (record, mut bundle, policy) = fixture();
            change(&mut bundle);
            assert_eq!(project(&record, &bundle, id(30), &policy), Err(EvidenceError::Exchange));
        }
        let (mut record, bundle, policy) = fixture();
        let mut observation = record.exchanges.pop_first().unwrap();
        observation.granted = id(99);
        record.exchanges.insert(observation);
        assert_eq!(project(&record, &bundle, id(30), &policy), Err(EvidenceError::Exchange));
        let (mut record, bundle, policy) = fixture();
        record.subject = id(99);
        assert_eq!(project(&record, &bundle, id(30), &policy), Err(EvidenceError::Exchange));
        let (mut record, bundle, policy) = fixture();
        let mut contradictory = record.exchanges.first().unwrap().clone();
        contradictory.requested = id(99);
        contradictory.granted = id(99);
        record.exchanges.insert(contradictory);
        assert_eq!(project(&record, &bundle, id(30), &policy), Err(EvidenceError::Exchange));
    }

    // r[verify lattice.github_ci.evidence_boundary]
    #[test]
    fn uncertain_or_stale_check_mutations_do_not_become_external_status_truth() {
        let cases: &[fn(&mut Record)] = &[
            |r| r.check.applied = None,
            |r| r.check.applied.as_mut().unwrap().version -= 1,
            |r| r.check.applied.as_mut().unwrap().terminal = false,
            |r| {
                r.check.pending = Some(github::CheckRunIntent {
                    version: 3,
                    update_id: id(14),
                    terminal: true,
                })
            },
            |r| {
                r.check.check_id = None;
                r.state.as_mut().unwrap().check_id = None;
            },
        ];
        for change in cases {
            let (mut record, bundle, policy) = fixture();
            change(&mut record);
            assert_eq!(project(&record, &bundle, id(30), &policy), Err(EvidenceError::Unobserved));
        }
    }

    // r[verify lattice.github_ci.evidence_boundary]
    #[test]
    fn real_validator_rejects_privacy_and_unsupported_build_claims_without_fabrication() {
        let (record, mut producer, policy) = fixture();
        producer.profile.claimed_observation_classes.push("build".to_owned());
        assert_eq!(project(&record, &producer, id(30), &policy), Err(EvidenceError::Profile));
        let (record, mut producer, policy) = fixture();
        producer.authority_decisions[0].decision_id = "Bearer do-not-persist".to_owned();
        assert_eq!(project(&record, &producer, id(30), &policy), Err(EvidenceError::Profile));
    }

    // r[verify lattice.github_ci.evidence_boundary]
    #[test]
    fn oversized_producer_documents_fail_closed() {
        let (record, mut producer, policy) = fixture();
        producer.service_requests[0].request_id = "x".repeat(EVIDENCE_BYTES_MAX);
        assert_eq!(project(&record, &producer, id(30), &policy), Err(EvidenceError::Bounds));
        let (record, mut producer, policy) = fixture();
        producer.service_requests = vec![producer.service_requests[0].clone(); PROFILE_RECORDS_MAX + 1];
        assert_eq!(project(&record, &producer, id(30), &policy), Err(EvidenceError::Bounds));
    }
}
