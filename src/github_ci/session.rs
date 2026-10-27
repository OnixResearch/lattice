//! Durable native orchestration. Recovery reacquires credentials and observes
//! producer truth; it never reconstructs a bearer token from a stored digest.
use core::Identity;
use core::Policy;
use std::collections::BTreeSet;

use basalt::ucan::TokenSigner;
use mantle_build_contract::async_api as wire;
use redb::ReadableDatabase as _;
use redb::ReadableTable as _;
use redb::ReadableTableMetadata as _;
use serde::Deserialize;
use serde::Serialize;

use super::core;
use super::exchange;
use super::github;
use super::mantle;

const RECORDS: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("github_mantle_sessions_v1");
pub const RECORD_BYTES_MAX: usize = 1_048_576;
pub const CURSOR_BYTES_MAX: usize = 4_096;
pub const EVENT_PAGES_MAX: usize = 4;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("orchestration policy rejected")]
    Policy,
    #[error("orchestration state conflicts")]
    Conflict,
    #[error("durable orchestration publication uncertain; read back before effects")]
    Storage,
    #[error("orchestration requires signed status reconciliation")]
    Reconcile,
    #[error(transparent)]
    Core(#[from] core::Error),
    #[error(transparent)]
    Mantle(#[from] mantle::Error),
    #[error(transparent)]
    Exchange(#[from] exchange::ExchangeError),
    #[error(transparent)]
    GitHub(#[from] github::Error),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExchangeObservation {
    pub receipt: Identity,
    pub policy: Identity,
    pub subject: Identity,
    pub requested: Identity,
    pub granted: Identity,
}

impl ExchangeObservation {
    fn observed(token: &exchange::ExchangedUcan) -> Result<Self, Error> {
        fn public_digest(text: &str) -> Result<Identity, Error> {
            let value = text.strip_prefix("blake3:").ok_or(Error::Policy)?;
            if value.len() != 64 || !value.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
                return Err(Error::Policy);
            }
            let mut bytes = [0; 32];
            hex::decode_to_slice(value, &mut bytes).map_err(|_| Error::Policy)?;
            Ok(Identity(bytes))
        }
        let receipt = token.receipt();
        if receipt.schema != basalt::oidc_exchange::EXCHANGE_RECEIPT_SCHEMA
            || receipt.outcome != basalt::oidc_exchange::ExchangeOutcome::Issued
        {
            return Err(Error::Policy);
        }
        Ok(Self {
            receipt: public_digest(&receipt.receipt_identity)?,
            policy: public_digest(&receipt.policy_identity)?,
            subject: public_digest(receipt.subject_identity.as_deref().ok_or(Error::Policy)?)?,
            requested: public_digest(&receipt.requested_capability_identity)?,
            granted: public_digest(receipt.granted_capability_identity.as_deref().ok_or(Error::Policy)?)?,
        })
    }
}

/// Contains only explicit public bindings, sequence state and a public signed
/// cursor. No OIDC/UCAN/request token, credential value or token fingerprint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub revision: u64,
    pub workflow: Identity,
    pub job: Identity,
    pub generation: u32,
    pub idempotency: Identity,
    pub subject: Identity,
    pub native_request_hash: Identity,
    pub native_submission: Identity,
    pub service: Identity,
    pub project: Identity,
    pub owner: Identity,
    pub producer: Identity,
    pub attempt: Option<mantle::NativeAttempt>,
    pub state: Option<core::DurableState>,
    pub event_cursor: Option<String>,
    pub check: github::CheckRunBinding,
    pub exchanges: BTreeSet<ExchangeObservation>,
}

impl Record {
    pub fn validate_for(
        &self,
        plan: &mantle::PlannedBuild,
        endpoint: &mantle::Endpoint,
        policy: &Policy,
    ) -> Result<(), Error> {
        self.check.target.validate()?;
        if self.exchanges.len() > policy.bounds.evidence {
            return Err(Error::Policy);
        }
        if !self.exchanges.is_empty() && !self.exchanges.iter().any(|exchange| exchange.subject == self.subject) {
            return Err(Error::Conflict);
        }
        if self.workflow != plan.job.workflow
            || self.job != plan.job.job
            || self.generation != plan.job.generation
            || self.idempotency != plan.job.idempotency
            || self.subject != plan.job.subject
            || self.native_request_hash != plan.native_request_hash
            || self.native_submission != mantle::public_identity(plan.resource().as_bytes())
            || self.service != mantle::public_identity(endpoint.service.as_bytes())
            || self.project != mantle::public_identity(endpoint.project.as_bytes())
            || self.owner != mantle::public_identity(endpoint.owner.as_bytes())
            || self.producer != endpoint.producer
            || self.check.target.external_id != self.job
            || self.attempt.is_some() != self.state.is_some()
        {
            return Err(Error::Conflict);
        }
        match (&self.attempt, &self.state) {
            (Some(attempt), Some(state)) => {
                if !wire::safe_segment(&attempt.attempt) || attempt.fence == 0 {
                    return Err(Error::Conflict);
                }
                state.validate_for(&plan.job, policy)?;
                if attempt.binding != state.binding
                    || attempt.native_request_hash != self.native_request_hash
                    || attempt.binding.producer != endpoint.producer
                    || attempt.fence != state.binding.fence
                    || mantle::public_identity(attempt.attempt.as_bytes()) != state.binding.attempt
                    || state.check_id != self.check.check_id
                {
                    return Err(Error::Conflict);
                }
                if let Some(cursor) = &self.event_cursor {
                    validate_cursor(cursor, endpoint, attempt)?;
                }
            }
            (None, None)
                if self.event_cursor.is_none()
                    && self.check.check_id.is_none()
                    && self.check.pending.is_none()
                    && self.check.applied.is_none() => {}
            _ => return Err(Error::Conflict),
        }
        Ok(())
    }
}

/// Reuses the repository's redb durability convention. The host opens the exact
/// owner-private database; no workflow-derived path is accepted here.
pub struct RecoveryStore<'a> {
    db: &'a redb::Database,
    maximum_records: u64,
}
impl<'a> RecoveryStore<'a> {
    pub fn new(db: &'a redb::Database, maximum_records: u64) -> Result<Self, Error> {
        if maximum_records == 0 {
            return Err(Error::Policy);
        }
        let tx = db.begin_write().map_err(|_| Error::Storage)?;
        {
            let _ = tx.open_table(RECORDS).map_err(|_| Error::Storage)?;
        }
        tx.commit().map_err(|_| Error::Storage)?;
        Ok(Self { db, maximum_records })
    }
    pub fn load(&self, job: Identity) -> Result<Option<Record>, Error> {
        let tx = self.db.begin_read().map_err(|_| Error::Storage)?;
        let table = tx.open_table(RECORDS).map_err(|_| Error::Storage)?;
        let row = table.get(job.hex().as_str()).map_err(|_| Error::Storage)?;
        row.map(|row| decode(row.value())).transpose()
    }
    fn commit(&self, previous: Option<&Record>, next: &Record) -> Result<(), Error> {
        if next.revision != previous.map_or(Some(0), |old| old.revision.checked_add(1)).ok_or(Error::Conflict)? {
            return Err(Error::Conflict);
        }
        let bytes = serde_json::to_vec(next).map_err(|_| Error::Storage)?;
        if bytes.len() > RECORD_BYTES_MAX {
            return Err(Error::Policy);
        }
        let mut tx = self.db.begin_write().map_err(|_| Error::Storage)?;
        tx.set_durability(redb::Durability::Immediate).map_err(|_| Error::Storage)?;
        {
            let mut table = tx.open_table(RECORDS).map_err(|_| Error::Storage)?;
            let key = next.job.hex();
            let current =
                table.get(key.as_str()).map_err(|_| Error::Storage)?.map(|row| decode(row.value())).transpose()?;
            if current.as_ref() != previous {
                return Err(Error::Conflict);
            }
            if current.is_none() && table.len().map_err(|_| Error::Storage)? >= self.maximum_records {
                return Err(Error::Policy);
            }
            table.insert(key.as_str(), bytes.as_slice()).map_err(|_| Error::Storage)?;
        }
        tx.commit().map_err(|_| Error::Storage)
    }
}
fn decode(bytes: &[u8]) -> Result<Record, Error> {
    if bytes.len() > RECORD_BYTES_MAX {
        return Err(Error::Policy);
    }
    serde_json::from_slice(bytes).map_err(|_| Error::Conflict)
}

pub struct Rollout<'a> {
    pub stage: core::Stage,
    pub candidate: Identity,
    pub required_producers: &'a BTreeSet<Identity>,
    pub evidence: &'a core::RolloutEvidence,
}
impl Rollout<'_> {
    fn admit(&self) -> Result<(), Error> {
        core::admit_stage(self.stage, self.candidate, self.required_producers, self.evidence)?;
        if self.stage == core::Stage::Offline {
            return Err(Error::Policy);
        }
        Ok(())
    }
}

/// Real Actions acquisition followed immediately by the actual Basalt issuer.
/// The host ports acquire keys through the credential registry, not this module.
pub struct InProcessAuthority<'a> {
    pub oidc: github::ActionsOidcClient<'a>,
    pub basalt: &'a mut dyn exchange::BasaltIssuerHost,
    pub oidc_audience: &'a str,
}
impl InProcessAuthority<'_> {
    fn grant(
        &mut self,
        plan: &core::CapabilityPlan,
        endpoint: &mantle::Endpoint,
        request: &wire::Request,
        now_ms: u64,
    ) -> Result<exchange::ExchangedUcan, Error> {
        let token = self.oidc.acquire_from_environment(self.oidc_audience)?;
        let resource = mantle::canonical_resource(endpoint, request)?;
        Ok(exchange::exchange(self.basalt, token, plan, &endpoint.project, &resource, now_ms)?)
    }
    fn holder(&mut self) -> &dyn TokenSigner {
        self.basalt.ports().holder_signer
    }
}

pub struct Session<'a> {
    pub store: RecoveryStore<'a>,
    pub client: mantle::MantleClient,
    pub policy: &'a Policy,
    pub rollout: Rollout<'a>,
}
impl Session<'_> {
    fn admit(&self, plan: &mantle::PlannedBuild) -> Result<(), Error> {
        self.policy.validate()?;
        if plan.job.request.input_artifacts.len() > self.policy.bounds.evidence {
            return Err(Error::Policy);
        }
        self.rollout.admit()?;
        if plan.job.workflow != self.rollout.candidate
            || !self.rollout.required_producers.contains(&self.client.endpoint().producer)
        {
            return Err(Error::Policy);
        }
        Ok(())
    }
    fn save(&self, old: &Record, mut next: Record, plan: &mantle::PlannedBuild) -> Result<Record, Error> {
        next.revision = old.revision.checked_add(1).ok_or(Error::Conflict)?;
        next.validate_for(plan, self.client.endpoint(), self.policy)?;
        self.store.commit(Some(old), &next)?;
        Ok(next)
    }
    fn admit_record(&self, plan: &mantle::PlannedBuild, record: &Record) -> Result<(), Error> {
        record.validate_for(plan, self.client.endpoint(), self.policy)?;
        if self.store.load(plan.job.job)?.as_ref() != Some(record) {
            return Err(Error::Conflict);
        }
        Ok(())
    }
    fn record_exchange(
        &self,
        record: &Record,
        token: &exchange::ExchangedUcan,
        plan: &mantle::PlannedBuild,
    ) -> Result<Record, Error> {
        let observation = ExchangeObservation::observed(token)?;
        if token.facts().operations.contains(&core::Operation::Submit) && observation.subject != plan.job.subject {
            return Err(Error::Policy);
        }
        if record.exchanges.contains(&observation) {
            return Ok(record.clone());
        }
        let mut next = record.clone();
        next.exchanges.insert(observation);
        self.save(record, next, plan)
    }
    fn capability(
        &self,
        plan: &mantle::PlannedBuild,
        attempt: Option<&mantle::NativeAttempt>,
        operation: core::Operation,
        now_ms: u64,
    ) -> Result<core::CapabilityPlan, Error> {
        // UCAN signed time is in seconds; do not forge millisecond facts.
        let boundary_ms = now_ms - now_ms % 1_000;
        let mut capability = if let Some(attempt) = attempt {
            core::attempt_capability(
                &plan.job.submission,
                attempt.binding.attempt,
                operation,
                self.policy,
                boundary_ms,
            )?
        } else {
            plan.job.submission.clone()
        };
        capability.not_before_ms = boundary_ms;
        capability.expires_ms = boundary_ms.checked_add(self.policy.capability_lifetime_ms).ok_or(Error::Policy)?;
        Ok(capability)
    }

    // r[impl lattice.github_ci.durable_async_submission]
    pub async fn submit(
        &self,
        plan: &mantle::PlannedBuild,
        target: github::CheckRunTarget,
        authority: &mut InProcessAuthority<'_>,
        now_ms: u64,
    ) -> Result<Record, Error> {
        self.admit(plan)?;
        let record = if let Some(record) = self.store.load(plan.job.job)? {
            record.validate_for(plan, self.client.endpoint(), self.policy)?;
            if record.check.target != target {
                return Err(Error::Conflict);
            }
            record
        } else {
            let endpoint = self.client.endpoint();
            let record = Record {
                revision: 0,
                workflow: plan.job.workflow,
                job: plan.job.job,
                generation: plan.job.generation,
                idempotency: plan.job.idempotency,
                subject: plan.job.subject,
                native_request_hash: plan.native_request_hash,
                native_submission: mantle::public_identity(plan.resource().as_bytes()),
                service: mantle::public_identity(endpoint.service.as_bytes()),
                project: mantle::public_identity(endpoint.project.as_bytes()),
                owner: mantle::public_identity(endpoint.owner.as_bytes()),
                producer: endpoint.producer,
                attempt: None,
                state: None,
                event_cursor: None,
                exchanges: BTreeSet::new(),
                check: github::CheckRunBinding::new(target)?,
            };
            record.validate_for(plan, endpoint, self.policy)?;
            self.store.commit(None, &record)?;
            record
        };
        if record.attempt.is_some() {
            return self.recover(plan, &record, authority, now_ms).await;
        }
        let capability = self.capability(plan, None, core::Operation::Submit, now_ms)?;
        let token = authority.grant(&capability, self.client.endpoint(), plan.request(), now_ms)?;
        let record = self.record_exchange(&record, &token, plan)?;
        let response = self.client.execute(plan.request(), &token, authority.holder(), rand::random(), now_ms).await?;
        let (attempt, event) = self.client.bind_submission(plan, &response, now_ms)?;
        let initial = core::DurableState::new(&plan.job, attempt.binding)?;
        let state = core::reconcile(&initial, &core::VerifiedStatus::after_verification(event), self.policy)?.state;
        let mut next = record.clone();
        next.attempt = Some(attempt);
        next.state = Some(state);
        self.save(&record, next, plan)
    }

    // r[impl lattice.github_ci.idempotent_event_reducer]
    pub async fn recover(
        &self,
        plan: &mantle::PlannedBuild,
        record: &Record,
        authority: &mut InProcessAuthority<'_>,
        now_ms: u64,
    ) -> Result<Record, Error> {
        self.admit(plan)?;
        self.admit_record(plan, record)?;
        let state = record.state.as_ref().ok_or(Error::Conflict)?;
        if matches!(core::restart(state, self.policy)?, core::RestartDecision::TerminalProjection) {
            return Ok(record.clone());
        }
        let attempt = record.attempt.as_ref().ok_or(Error::Conflict)?;
        let request = mantle::attempt_request(self.client.endpoint(), wire::Request::Status {
            attempt_id: attempt.attempt.clone(),
        })?;
        let capability = self.capability(plan, Some(attempt), core::Operation::Status, now_ms)?;
        let token = authority.grant(&capability, self.client.endpoint(), &request, now_ms)?;
        let current = self.record_exchange(record, &token, plan)?;
        let response = self.client.execute(&request, &token, authority.holder(), rand::random(), now_ms).await?;
        let facts = self.client.signed_status(&response, attempt, now_ms)?;
        let mut next = current.clone();
        next.state = Some(core::reconcile(state, &core::VerifiedStatus::after_verification(facts), self.policy)?.state);
        // Producer cursors expire. Reacquire one from the bounded event origin;
        // signed sequence admission makes replayed observations duplicates.
        next.event_cursor = None;
        self.save(&current, next, plan)
    }

    pub async fn poll(
        &self,
        plan: &mantle::PlannedBuild,
        record: &Record,
        authority: &mut InProcessAuthority<'_>,
        now_ms: u64,
    ) -> Result<Record, Error> {
        self.admit(plan)?;
        self.admit_record(plan, record)?;
        let attempt = record.attempt.as_ref().ok_or(Error::Conflict)?;
        let mut current = record.clone();
        for _ in 0..EVENT_PAGES_MAX {
            if current.state.as_ref().ok_or(Error::Conflict)?.phase.terminal() {
                return Ok(current);
            }
            let request = mantle::attempt_request(self.client.endpoint(), wire::Request::Events {
                attempt_id: attempt.attempt.clone(),
                cursor: current.event_cursor.clone(),
                items: wire::EVENT_PAGE_MAX,
            })?;
            let capability = self.capability(plan, Some(attempt), core::Operation::Events, now_ms)?;
            let token = authority.grant(&capability, self.client.endpoint(), &request, now_ms)?;
            current = self.record_exchange(&current, &token, plan)?;
            let response = self.client.execute(&request, &token, authority.holder(), rand::random(), now_ms).await?;
            let wire::Response::Events(page) = response.response() else {
                return Err(Error::Conflict);
            };
            let events = &page.values;
            let next_cursor = page.next_cursor.clone();
            let complete = events
                .last()
                .is_none_or(|event| response.status().is_some_and(|status| event.sequence == status.sequence));
            if events.len() > wire::EVENT_PAGE_MAX as usize {
                return Err(Error::Policy);
            }
            validate_cursor(&next_cursor, self.client.endpoint(), attempt)?;
            if !complete && (events.is_empty() || current.event_cursor.as_ref() == Some(&next_cursor)) {
                return Err(Error::Conflict);
            }
            let mut next = current.clone();
            for event in events {
                let facts = self.client.verify_event(&response, event, attempt)?;
                let reduction = core::reduce(
                    next.state.as_ref().ok_or(Error::Conflict)?,
                    &core::VerifiedEvent::after_verification(facts),
                    self.policy,
                )?;
                next.state = Some(reduction.state);
                if reduction.disposition == core::Disposition::Reconcile {
                    current = self.save(&current, next, plan)?;
                    return self.recover(plan, &current, authority, now_ms).await;
                }
            }
            next.event_cursor = Some(next_cursor);
            current = self.save(&current, next, plan)?;
            if complete {
                return Ok(current);
            }
        }
        Err(Error::Reconcile)
    }

    pub async fn cancel(
        &self,
        plan: &mantle::PlannedBuild,
        record: &Record,
        reason: core::CancelReason,
        authority: &mut InProcessAuthority<'_>,
        now_ms: u64,
    ) -> Result<Record, Error> {
        self.admit(plan)?;
        self.admit_record(plan, record)?;
        let attempt = record.attempt.as_ref().ok_or(Error::Conflict)?;
        let state = record.state.as_ref().ok_or(Error::Conflict)?;
        if state.phase.terminal() {
            return Ok(record.clone());
        }
        let request = mantle::attempt_request(self.client.endpoint(), wire::Request::Cancel {
            attempt_id: attempt.attempt.clone(),
            fence: attempt.fence,
        })?;
        let capability = self.capability(plan, Some(attempt), core::Operation::OwnedCancel, now_ms)?;
        let token = authority.grant(&capability, self.client.endpoint(), &request, now_ms)?;
        let granted = self.record_exchange(record, &token, plan)?;
        let cancellation = core::cancel(
            state,
            &core::CancelIntent {
                binding: state.binding,
                expected_version: state.version,
                reason,
            },
            &capability,
            token.facts(),
            self.policy,
            now_ms,
        )?;
        let mut next = granted.clone();
        next.state = Some(cancellation.state);
        let current = self.save(&granted, next, plan)?;
        // An already persisted uncertain cancellation is safely re-presented for
        // this same fenced attempt; a successful response is still observed truth.
        let response = self.client.execute(&request, &token, authority.holder(), rand::random(), now_ms).await?;
        let facts = self.client.signed_status(&response, attempt, now_ms)?;
        let mut next = current.clone();
        next.state = Some(
            core::reconcile(
                current.state.as_ref().ok_or(Error::Conflict)?,
                &core::VerifiedStatus::after_verification(facts),
                self.policy,
            )?
            .state,
        );
        self.save(&current, next, plan)
    }

    pub async fn discover_result(
        &self,
        plan: &mantle::PlannedBuild,
        record: &Record,
        authority: &mut InProcessAuthority<'_>,
        now_ms: u64,
    ) -> Result<(Record, Identity), Error> {
        self.admit(plan)?;
        self.admit_record(plan, record)?;
        let state = record.state.as_ref().ok_or(Error::Conflict)?;
        if state.phase != core::Phase::Succeeded {
            return Err(Error::Conflict);
        }
        let event = state.history.get(&state.cursor).ok_or(Error::Conflict)?;
        if event.evidence.len() != 1 {
            return Err(Error::Conflict);
        }
        let digest = *event.evidence.first().ok_or(Error::Conflict)?;
        let attempt = record.attempt.as_ref().ok_or(Error::Conflict)?;
        let request = mantle::attempt_request(self.client.endpoint(), wire::Request::Result {
            attempt_id: attempt.attempt.clone(),
        })?;
        let capability = self.capability(plan, Some(attempt), core::Operation::Result, now_ms)?;
        let token = authority.grant(&capability, self.client.endpoint(), &request, now_ms)?;
        let current = self.record_exchange(record, &token, plan)?;
        let response = self.client.execute(&request, &token, authority.holder(), rand::random(), now_ms).await?;
        let observed = self.client.verify_result(&response, attempt, digest)?;
        Ok((current, observed))
    }

    /// Log bytes remain transient and bounded; neither recovery records nor
    /// Check Runs receive these bytes automatically.
    pub async fn logs(
        &self,
        plan: &mantle::PlannedBuild,
        record: &Record,
        cursor: Option<String>,
        limit: u32,
        authority: &mut InProcessAuthority<'_>,
        now_ms: u64,
    ) -> Result<(Record, Vec<u8>, String, bool), Error> {
        self.admit(plan)?;
        self.admit_record(plan, record)?;
        let attempt = record.attempt.as_ref().ok_or(Error::Conflict)?;
        if let Some(cursor) = &cursor {
            validate_cursor_kind(cursor, self.client.endpoint(), attempt, "logs")?;
        }
        let request = mantle::attempt_request(self.client.endpoint(), wire::Request::Logs {
            attempt_id: attempt.attempt.clone(),
            cursor,
            bytes: limit,
        })?;
        let capability = self.capability(plan, Some(attempt), core::Operation::Log, now_ms)?;
        let token = authority.grant(&capability, self.client.endpoint(), &request, now_ms)?;
        let current = self.record_exchange(record, &token, plan)?;
        let response = self.client.execute(&request, &token, authority.holder(), rand::random(), now_ms).await?;
        let (bytes, next_cursor) = self.client.log_window(response, attempt)?;
        let complete = bytes.len() < limit as usize;
        if bytes.len() > limit as usize {
            return Err(Error::Policy);
        }
        validate_cursor_kind(&next_cursor, self.client.endpoint(), attempt, "logs")?;
        Ok((current, bytes, next_cursor, complete))
    }

    /// Public projection text/links must already pass the pure public-output
    /// policy. Persist reservation before effect; unknown outcomes remain pending.
    pub fn update_check(
        &self,
        plan: &mantle::PlannedBuild,
        record: &Record,
        projection: &core::CheckProjection,
        github: &github::GitHubChecksClient<'_>,
        context: &crate::credentials::CredentialResolutionContext,
    ) -> Result<Record, Error> {
        self.admit(plan)?;
        self.admit_record(plan, record)?;
        let state = record.state.as_ref().ok_or(Error::Conflict)?;
        if projection.state_version != state.version {
            return Err(Error::Conflict);
        }
        let expected = core::project_check(
            state,
            &projection.title,
            &projection.summary,
            &projection.text,
            &projection.annotations,
            &projection.links,
            self.policy,
        )?;
        if &expected != projection {
            return Err(Error::Conflict);
        }
        let intent = github::prepare_check(&record.check, projection)?;
        let mut reserved = record.clone();
        reserved.check = intent.reserved_binding().clone();
        let reserved = self.save(record, reserved, plan)?;
        let acknowledged = match github.execute(intent, projection, context) {
            Ok(acknowledged) => acknowledged,
            Err(error) => {
                if let Ok(rejected) = reserved.check.after_rejection(&error) {
                    let mut next = reserved.clone();
                    next.check = rejected;
                    self.save(&reserved, next, plan)?;
                }
                return Err(Error::GitHub(error));
            }
        };
        let mut next = reserved.clone();
        next.state.as_mut().ok_or(Error::Conflict)?.check_id = acknowledged.check_id;
        next.check = acknowledged;
        self.save(&reserved, next, plan)
    }
}

fn validate_cursor(text: &str, endpoint: &mantle::Endpoint, attempt: &mantle::NativeAttempt) -> Result<(), Error> {
    validate_cursor_kind(text, endpoint, attempt, "events")
}

fn validate_cursor_kind(
    text: &str,
    _endpoint: &mantle::Endpoint,
    _attempt: &mantle::NativeAttempt,
    kind: &str,
) -> Result<(), Error> {
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    if text.len() > CURSOR_BYTES_MAX {
        return Err(Error::Policy);
    }
    let bytes = URL_SAFE_NO_PAD.decode(text).map_err(|_| Error::Conflict)?;
    let expected_kind = match kind {
        "events" => b'e',
        "logs" => b'l',
        _ => return Err(Error::Policy),
    };
    // The producer MAC covers owner/project/audience/attempt and is verified by
    // the receiver. The signed response authenticates issuance to this exact
    // request; consumers cannot reconstruct the private MAC key.
    if bytes.len() != 82
        || bytes[0] != 1
        || bytes[1] != expected_kind
        || u64::from_le_bytes(bytes[10..18].try_into().map_err(|_| Error::Conflict)?) == 0
    {
        return Err(Error::Conflict);
    }
    Ok(())
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
