mod common;

use std::collections::HashMap;
use std::sync::Mutex;

use lattice::engine::protocol::ActionRequest;
use lattice::engine::protocol::AttemptState;
use lattice::engine::protocol::Error;
use lattice::engine::protocol::Identity;
use lattice::engine::protocol::Issue;
use lattice::engine::protocol::LabelId;
use lattice::engine::protocol::MessageEnvelope;
use lattice::engine::protocol::RolePlacement;
use lattice::engine::protocol::SessionEvent;
use lattice::engine::protocol::SessionPlan;
use lattice::engine::protocol::SessionRecord;
use lattice::engine::protocol::plan_session;
use lattice::engine::protocol::shell::AdapterDispatch;
use lattice::engine::protocol::shell::AdapterProfile;
use lattice::engine::protocol::shell::Adapters;
use lattice::engine::protocol::shell::ProtocolAdapter;
use lattice::engine::protocol::shell::ProtocolAuthority;
use lattice::engine::protocol::shell::Runtime;
use lattice::engine::protocol::shell::SessionRepository;
use lattice::engine::protocol::shell::StoreError;

#[derive(Default)]
struct MemoryRepository {
    session: Mutex<Option<SessionRecord>>,
    commits: Mutex<Vec<AttemptState>>,
    fail_create: bool,
}

impl SessionRepository for MemoryRepository {
    fn create(&self, session: &SessionRecord) -> Result<(), StoreError> {
        if self.fail_create {
            return Err(StoreError::Unavailable);
        }
        let mut slot = self.session.lock().map_err(|_| StoreError::Unavailable)?;
        if slot.is_some() {
            return Err(StoreError::Conflict);
        }
        *slot = Some(session.clone());
        Ok(())
    }

    fn load(&self, identity: &Identity) -> Result<Option<SessionRecord>, StoreError> {
        let slot = self.session.lock().map_err(|_| StoreError::Unavailable)?;
        Ok(slot.as_ref().filter(|session| session.plan.session == *identity).cloned())
    }

    fn commit(&self, expected_version: u64, session: &SessionRecord, _event: &SessionEvent) -> Result<(), StoreError> {
        let mut slot = self.session.lock().map_err(|_| StoreError::Unavailable)?;
        let current = slot.as_ref().ok_or(StoreError::Conflict)?;
        if current.version != expected_version || session.version != expected_version.saturating_add(1) {
            return Err(StoreError::Conflict);
        }
        if let Some(attempt) = session.attempts.last() {
            self.commits.lock().map_err(|_| StoreError::Unavailable)?.push(attempt.state);
        }
        *slot = Some(session.clone());
        Ok(())
    }
}

struct StaticAuthority {
    result: Result<Identity, Issue>,
}

impl ProtocolAuthority for StaticAuthority {
    fn authorize(&self, _session: &SessionRecord, _envelope: &MessageEnvelope) -> Result<Identity, Issue> {
        self.result.clone()
    }
}

struct RecordingAdapter {
    profile: AdapterProfile,
    result: AdapterDispatch,
    observed_states: Mutex<Vec<AttemptState>>,
}

impl ProtocolAdapter for RecordingAdapter {
    fn profile(&self) -> &AdapterProfile {
        &self.profile
    }

    fn dispatch(
        &self,
        _envelope: &MessageEnvelope,
        _limits: &lattice::engine::protocol::AdapterLimits,
    ) -> AdapterDispatch {
        self.observed_states
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(AttemptState::DispatchedUnknown);
        self.result
    }
}

fn plan() -> (SessionPlan, Vec<RolePlacement>) {
    let cohort = common::protocol::cohort(common::protocol::native_source(common::protocol::transfer(
        common::protocol::ROLE_A,
        common::protocol::ROLE_B,
        common::protocol::LABEL_A,
        common::protocol::end(),
    )));
    let placements = common::protocol::native_placements(&cohort);
    let plan = match plan_session(cohort, placements.clone(), common::protocol::identity("shell-session")) {
        Ok(plan) => plan,
        Err(error) => panic!("plan failed: {error:?}"),
    };
    (plan, placements)
}

fn adapters(placements: &[RolePlacement], result: AdapterDispatch) -> Adapters {
    let mut native: HashMap<String, Box<dyn ProtocolAdapter>> = HashMap::new();
    for placement in placements {
        if let lattice::engine::protocol::PlacementKind::Native { runtime_route } = &placement.kind {
            native.insert(
                runtime_route.as_str().to_owned(),
                Box::new(RecordingAdapter {
                    profile: AdapterProfile::Native {
                        cohort: placement.adapter_cohort.clone(),
                        runtime_route: runtime_route.clone(),
                    },
                    result,
                    observed_states: Mutex::new(Vec::new()),
                }),
            );
        }
    }
    Adapters::new(native, HashMap::new())
}

fn request() -> ActionRequest {
    ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("shell-payload"),
    }
}

// r[verify lattice.role_protocol.persistence.failed]
#[test]
fn failed_initial_persistence_starts_no_authority_or_adapter_work() {
    let (plan, placements) = plan();
    let repository = MemoryRepository {
        fail_create: true,
        ..MemoryRepository::default()
    };
    let authority = StaticAuthority {
        result: Ok(common::protocol::identity("authority")),
    };
    let adapters = adapters(&placements, AdapterDispatch::Started);
    let shell = Runtime::new(&repository, &authority, &adapters);
    assert_eq!(shell.create(plan), Err(Error::Issue(Issue::PersistenceFailed)));
    assert!(repository.commits.lock().is_ok_and(|commits| commits.is_empty()));
}

// r[verify lattice.role_protocol.authorization.replayed]
// r[verify lattice.role_protocol.external_roles.denied]
#[test]
fn missing_current_authority_persists_failure_before_adapter_dispatch() {
    let (plan, placements) = plan();
    let repository = MemoryRepository::default();
    let authority = StaticAuthority {
        result: Err(Issue::AuthorityDenied),
    };
    let adapters = adapters(&placements, AdapterDispatch::Started);
    let shell = Runtime::new(&repository, &authority, &adapters);
    let session = match shell.create(plan) {
        Ok(session) => session,
        Err(error) => panic!("session creation failed: {error:?}"),
    };
    assert_eq!(shell.dispatch(&session, request()), Err(Error::Issue(Issue::AuthorityDenied)));
    let loaded = match repository.load(&session.plan.session) {
        Ok(Some(session)) => session,
        other => panic!("failed load: {other:?}"),
    };
    assert_eq!(loaded.attempts[0].state, AttemptState::Failed);
}

// r[verify lattice.role_protocol.uncertainty.crash]
#[test]
fn dispatch_persists_prepared_authorized_and_unknown_before_adapter_start() {
    let (plan, placements) = plan();
    let repository = MemoryRepository::default();
    let authority = StaticAuthority {
        result: Ok(common::protocol::identity("authority-attempt")),
    };
    let adapters = adapters(&placements, AdapterDispatch::Started);
    let shell = Runtime::new(&repository, &authority, &adapters);
    let session = match shell.create(plan) {
        Ok(session) => session,
        Err(error) => panic!("session creation failed: {error:?}"),
    };
    let dispatched = match shell.dispatch(&session, request()) {
        Ok(dispatched) => dispatched,
        Err(error) => panic!("dispatch failed: {error:?}"),
    };
    assert_eq!(dispatched.dispatched_messages, 1);
    assert_eq!(dispatched.session.attempts[0].state, AttemptState::DispatchedUnknown);
    let commits = match repository.commits.lock() {
        Ok(commits) => commits.clone(),
        Err(error) => panic!("commit log lock failed: {error}"),
    };
    assert_eq!(commits, vec![
        AttemptState::Prepared,
        AttemptState::Authorized,
        AttemptState::DispatchedUnknown
    ]);
}

// r[verify lattice.role_protocol.transitions.transfer]
#[test]
fn accepted_receiver_observation_commits_after_unknown_dispatch() {
    let (plan, placements) = plan();
    let repository = MemoryRepository::default();
    let authority = StaticAuthority {
        result: Ok(common::protocol::identity("authority-attempt")),
    };
    let adapters = adapters(&placements, AdapterDispatch::Started);
    let shell = Runtime::new(&repository, &authority, &adapters);
    let session = match shell.create(plan) {
        Ok(session) => session,
        Err(error) => panic!("session creation failed: {error:?}"),
    };
    let dispatched = match shell.dispatch(&session, request()) {
        Ok(dispatched) => dispatched,
        Err(error) => panic!("dispatch failed: {error:?}"),
    };
    let envelope = dispatched.session.attempts[0].messages[0].clone();
    let observed = match shell.observe(
        &dispatched.session,
        &envelope,
        true,
        common::protocol::identity("receiver-evidence"),
        None,
    ) {
        Ok(observed) => observed,
        Err(error) => panic!("observation failed: {error:?}"),
    };
    let settled = match shell.settle(&observed, &dispatched.attempt) {
        Ok(settled) => settled,
        Err(error) => panic!("settle failed: {error:?}"),
    };
    assert_eq!(settled.attempts[0].state, AttemptState::Committed);
    assert_eq!(settled.owners[0].owner, common::protocol::lattice_role(common::protocol::ROLE_B));
}

// r[verify lattice.role_protocol.replay.send]
#[test]
fn shell_replay_reads_records_without_authority_or_adapter_dispatch() {
    let (plan, placements) = plan();
    let repository = MemoryRepository::default();
    let authority = StaticAuthority {
        result: Ok(common::protocol::identity("authority-attempt")),
    };
    let adapters = adapters(&placements, AdapterDispatch::Started);
    let shell = Runtime::new(&repository, &authority, &adapters);
    let session = match shell.create(plan) {
        Ok(session) => session,
        Err(error) => panic!("session creation failed: {error:?}"),
    };
    let replayed = match shell.replay(&session.plan.session) {
        Ok(replayed) => replayed,
        Err(error) => panic!("replay failed: {error:?}"),
    };
    assert_eq!(replayed, session);
    assert!(repository.commits.lock().is_ok_and(|commits| commits.is_empty()));
}
