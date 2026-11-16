//! Imperative protocol persistence, authorization, and adapter shell.
//!
//! r[impl lattice.role_protocol.persistence]
//! r[impl lattice.role_protocol.uncertainty]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "one imperative transaction shell and its local DTO vocabulary keep dispatch order reviewable"
)]

pub mod authority;
pub mod operator;
mod routing;
pub mod transport;

use std::collections::HashMap;

use super::model::ActionRequest;
use super::model::AttemptState;
use super::model::Error;
use super::model::Identity;
use super::model::Issue;
use super::model::MessageEnvelope;
use super::model::SessionEvent;
use super::model::SessionPlan;
use super::model::SessionRecord;

pub trait SessionRepository: Send + Sync {
    fn create(&self, session: &SessionRecord) -> Result<(), StoreError>;
    fn load(&self, identity: &Identity) -> Result<Option<SessionRecord>, StoreError>;
    fn commit(&self, expected_version: u64, session: &SessionRecord, event: &SessionEvent) -> Result<(), StoreError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreError {
    Unavailable,
    Conflict,
    InvalidRecord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterDispatch {
    Started,
    /// A guarantee that this call started no receiver work.
    RejectedBeforeStart,
    /// Failure-shaped response without an admitted receiver observation; still unknown.
    ObservedFailure,
    UnknownAfterStart,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterProfile {
    Native {
        cohort: Identity,
        runtime_route: Identity,
    },
    External {
        cohort: Identity,
        contract: Identity,
        destination_policy: Identity,
        credential_reference: Identity,
    },
}

pub trait ProtocolAdapter: Send + Sync {
    fn profile(&self) -> &AdapterProfile;
    fn dispatch(&self, envelope: &MessageEnvelope, limits: &super::model::AdapterLimits) -> AdapterDispatch;
}

pub trait ProtocolAuthority: Send + Sync {
    fn authorize(&self, session: &SessionRecord, envelope: &MessageEnvelope) -> Result<Identity, Issue>;
}

pub struct Adapters {
    native: HashMap<String, Box<dyn ProtocolAdapter>>,
    external: HashMap<String, Box<dyn ProtocolAdapter>>,
}

impl Adapters {
    #[must_use]
    pub fn new(
        native: HashMap<String, Box<dyn ProtocolAdapter>>,
        external: HashMap<String, Box<dyn ProtocolAdapter>>,
    ) -> Self {
        Self { native, external }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchResult {
    pub session: SessionRecord,
    pub attempt: Identity,
    pub dispatched_messages: usize,
}

pub struct Runtime<'a> {
    repository: &'a dyn SessionRepository,
    authority: &'a dyn ProtocolAuthority,
    adapters: &'a Adapters,
}

impl<'a> Runtime<'a> {
    #[must_use]
    pub const fn new(
        repository: &'a dyn SessionRepository,
        authority: &'a dyn ProtocolAuthority,
        adapters: &'a Adapters,
    ) -> Self {
        Self {
            repository,
            authority,
            adapters,
        }
    }

    #[must_use]
    pub const fn repository(&self) -> &dyn SessionRepository {
        self.repository
    }

    pub fn create(&self, plan: SessionPlan) -> Result<SessionRecord, Error> {
        let session = super::core::create_session(plan)?;
        self.repository.create(&session).map_err(map_store_error)?;
        Ok(session)
    }

    pub fn dispatch(&self, session: &SessionRecord, request: ActionRequest) -> Result<DispatchResult, Error> {
        let action = super::core::plan_action(session, request)?;
        let prepared = super::core::prepare_attempt(session, action)?;
        let attempt = prepared.attempts.last().ok_or(Issue::PersistenceFailed)?;
        let prepared_event = prepared.events.last().ok_or(Issue::PersistenceFailed)?;
        self.repository.commit(session.version, &prepared, prepared_event).map_err(map_store_error)?;

        let authorization_references = match attempt
            .messages
            .iter()
            .map(|envelope| self.authority.authorize(&prepared, envelope))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(references) => references,
            Err(issue) => {
                self.persist_failed_before_dispatch(&prepared, &attempt.identity, issue)?;
                return Err(issue.into());
            }
        };
        let authorized = super::core::record_authorization(&prepared, &attempt.identity, authorization_references)?;
        let authorized_event = authorized.events.last().ok_or(Issue::PersistenceFailed)?;
        self.repository.commit(prepared.version, &authorized, authorized_event).map_err(map_store_error)?;
        // Resolve every deterministic route/profile before the first possible effect.
        // Retain these exact adapter references throughout fanout; never select again
        // after a recipient might have acted.
        let selected = match attempt
            .messages
            .iter()
            .map(|envelope| self.adapters.select(&authorized, envelope))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(selected) => selected,
            Err(issue) => {
                self.persist_failed_before_dispatch(&authorized, &attempt.identity, issue)?;
                return Err(issue.into());
            }
        };
        let unknown = super::core::mark_dispatched_unknown(&authorized, &attempt.identity)?;
        let unknown_event = unknown.events.last().ok_or(Issue::PersistenceFailed)?;
        self.repository.commit(authorized.version, &unknown, unknown_event).map_err(map_store_error)?;

        let mut dispatched_messages = 0_usize;
        for (envelope, adapter) in attempt.messages.iter().zip(selected) {
            let placement = unknown
                .plan
                .roles
                .iter()
                .find(|role| role.role == envelope.destination_role)
                .ok_or(Issue::MissingRole)?;
            match adapter.dispatch(envelope, &placement.placement.limits) {
                AdapterDispatch::Started => {
                    dispatched_messages = dispatched_messages.saturating_add(1);
                }
                AdapterDispatch::RejectedBeforeStart => {
                    if dispatched_messages == 0 {
                        self.persist_rejected_before_start(&unknown, &attempt.identity, Issue::AdapterFailed)?;
                    }
                    return Err(Issue::AdapterFailed.into());
                }
                AdapterDispatch::ObservedFailure | AdapterDispatch::UnknownAfterStart => {
                    return Err(Issue::AdapterFailed.into());
                }
            }
        }
        Ok(DispatchResult {
            session: unknown,
            attempt: attempt.identity.clone(),
            dispatched_messages,
        })
    }

    fn persist_failed_before_dispatch(
        &self,
        session: &SessionRecord,
        attempt: &Identity,
        issue: Issue,
    ) -> Result<(), Error> {
        let failed = super::core::fail_before_dispatch(session, attempt, issue)?;
        let event = failed.events.last().ok_or(Issue::PersistenceFailed)?;
        self.repository.commit(session.version, &failed, event).map_err(map_store_error)
    }

    fn persist_rejected_before_start(
        &self,
        session: &SessionRecord,
        attempt: &Identity,
        issue: Issue,
    ) -> Result<(), Error> {
        // Only the first adapter's explicit no-start guarantee reaches this path.
        // This is not a failed-before-dispatch inference from an unknown attempt.
        let outcome = super::identity::reference(super::identity::OUTCOME_DOMAIN, &[
            attempt.as_str(),
            "rejected-before-start",
            issue.code(),
        ])?;
        let failed = super::core::apply_event(session, &SessionEvent::Failed {
            attempt: attempt.clone(),
            outcome,
        })?;
        let event = failed.events.last().ok_or(Issue::PersistenceFailed)?;
        self.repository.commit(session.version, &failed, event).map_err(map_store_error)
    }

    pub fn observe(
        &self,
        session: &SessionRecord,
        envelope: &MessageEnvelope,
        accepted: bool,
        evidence_reference: Identity,
        issue: Option<Issue>,
    ) -> Result<SessionRecord, Error> {
        let observed = super::core::admit_observation(session, envelope, accepted, evidence_reference, issue)?;
        let event = observed.events.last().ok_or(Issue::PersistenceFailed)?;
        self.repository.commit(session.version, &observed, event).map_err(map_store_error)?;
        Ok(observed)
    }

    pub fn settle(&self, session: &SessionRecord, attempt: &Identity) -> Result<SessionRecord, Error> {
        let settled = super::core::settle_attempt(session, attempt)?;
        let event = settled.events.last().ok_or(Issue::PersistenceFailed)?;
        self.repository.commit(session.version, &settled, event).map_err(map_store_error)?;
        Ok(settled)
    }

    pub fn abandon(
        &self,
        session: &SessionRecord,
        attempt: &Identity,
        acknowledge_unknown: bool,
    ) -> Result<SessionRecord, Error> {
        let abandoned = super::core::abandon_unknown(session, attempt, acknowledge_unknown)?;
        let event = abandoned.events.last().ok_or(Issue::PersistenceFailed)?;
        self.repository.commit(session.version, &abandoned, event).map_err(map_store_error)?;
        Ok(abandoned)
    }

    pub fn replay(&self, identity: &Identity) -> Result<SessionRecord, Error> {
        let persisted = self.repository.load(identity).map_err(map_store_error)?.ok_or(Issue::MissingRole)?;
        super::core::replay(persisted.plan.clone(), &persisted.events)
    }
}

fn map_store_error(error: StoreError) -> Error {
    match error {
        StoreError::Conflict => Issue::ReorderedMessage.into(),
        StoreError::Unavailable | StoreError::InvalidRecord => Issue::PersistenceFailed.into(),
    }
}

#[must_use]
pub fn is_unknown(session: &SessionRecord, attempt: &Identity) -> bool {
    session
        .attempts
        .iter()
        .find(|record| record.identity == *attempt)
        .is_some_and(|record| record.state == AttemptState::DispatchedUnknown)
}
