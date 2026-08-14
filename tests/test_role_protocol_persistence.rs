mod common;

use lattice::engine::protocol::ActionRequest;
use lattice::engine::protocol::Error;
use lattice::engine::protocol::Issue;
use lattice::engine::protocol::LabelId;
use lattice::engine::protocol::create_session;
use lattice::engine::protocol::plan_action;
use lattice::engine::protocol::plan_session;
use lattice::engine::protocol::prepare_attempt;
use lattice::engine::protocol::replay;
use lattice::engine::protocol::shell::SessionRepository;
use lattice::store::Store;

fn session() -> lattice::engine::protocol::SessionRecord {
    let cohort = common::protocol::cohort(common::protocol::native_source(common::protocol::transfer(
        common::protocol::ROLE_A,
        common::protocol::ROLE_B,
        common::protocol::LABEL_A,
        common::protocol::end(),
    )));
    let plan = match plan_session(
        cohort.clone(),
        common::protocol::native_placements(&cohort),
        common::protocol::identity("durable-session"),
    ) {
        Ok(plan) => plan,
        Err(error) => panic!("plan failed: {error:?}"),
    };
    match create_session(plan) {
        Ok(session) => session,
        Err(error) => panic!("session failed: {error:?}"),
    }
}

fn store() -> (tempfile::TempDir, Store) {
    let directory = match tempfile::tempdir() {
        Ok(directory) => directory,
        Err(error) => panic!("temporary directory failed: {error}"),
    };
    let store = match Store::new(directory.path().join("lattice.redb")) {
        Ok(store) => store,
        Err(error) => panic!("store failed: {error}"),
    };
    (directory, store)
}

// r[verify lattice.role_protocol.persistence.failed]
// r[verify lattice.role_protocol.replay.send]
#[test]
fn redb_persists_exact_events_and_replay_matches_without_dispatch() {
    let (_directory, store) = store();
    let initial = session();
    assert!(store.create(&initial).is_ok());
    let action = match plan_action(&initial, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("persisted-payload"),
    }) {
        Ok(action) => action,
        Err(error) => panic!("action failed: {error:?}"),
    };
    let prepared = match prepare_attempt(&initial, action) {
        Ok(prepared) => prepared,
        Err(error) => panic!("prepare failed: {error:?}"),
    };
    let event = match prepared.events.last() {
        Some(event) => event,
        None => panic!("prepared state has no event"),
    };
    assert!(store.commit(initial.version, &prepared, event).is_ok());
    let loaded = match store.load(&prepared.plan.session) {
        Ok(Some(session)) => session,
        Ok(None) => panic!("persisted session missing"),
        Err(error) => panic!("load failed: {error:?}"),
    };
    assert_eq!(loaded, prepared);
    assert_eq!(replay(loaded.plan.clone(), &loaded.events), Ok(loaded));
}

// r[verify lattice.role_protocol.persistence.failed]
#[test]
fn stale_version_and_duplicate_session_are_rejected_without_state_change() {
    let (_directory, store) = store();
    let initial = session();
    assert!(store.create(&initial).is_ok());
    assert_eq!(store.create(&initial), Err(lattice::engine::protocol::shell::StoreError::Conflict));
    let action = match plan_action(&initial, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("stale-payload"),
    }) {
        Ok(action) => action,
        Err(error) => panic!("action failed: {error:?}"),
    };
    let prepared = match prepare_attempt(&initial, action) {
        Ok(prepared) => prepared,
        Err(error) => panic!("prepare failed: {error:?}"),
    };
    let event = match prepared.events.last() {
        Some(event) => event,
        None => panic!("prepared state has no event"),
    };
    assert_eq!(
        store.commit(initial.version.saturating_add(1), &prepared, event),
        Err(lattice::engine::protocol::shell::StoreError::Conflict)
    );
    assert_eq!(store.load(&initial.plan.session), Ok(Some(initial)));
}

#[test]
fn forged_action_identity_is_rejected_before_persistence() {
    let initial = session();
    let mut action = match plan_action(&initial, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("forged-payload"),
    }) {
        Ok(action) => action,
        Err(error) => panic!("action failed: {error:?}"),
    };
    action.identity = common::protocol::identity("forged-action");
    assert_eq!(prepare_attempt(&initial, action), Err(Error::Issue(Issue::IdentityMismatch)));
    assert!(initial.attempts.is_empty());
}
