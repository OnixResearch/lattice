use super::*;

const FIRST_NODE: u64 = 11;
const SECOND_NODE: u64 = 22;
const THIRD_NODE: u64 = 33;
const PROCESS_LANE: &str = "process";

fn policy(total_slots: u32, lane_slots: u32) -> CapacityPolicy {
    CapacityPolicy {
        total_slots,
        lane_slots: std::collections::BTreeMap::from([(PROCESS_LANE.to_string(), lane_slots)]),
    }
}

fn candidate(node_id: u64, conflict: ConflictState) -> Candidate {
    Candidate {
        node_id,
        lane: PROCESS_LANE.to_string(),
        conflict,
        backend_ready: true,
        authority_ready: true,
        credential_ready: true,
        attempt_ready: true,
        suspended: false,
        cancelled: false,
        terminal: false,
    }
}

// r[verify lattice.workflow_execution.ready_frontier.planner]
// r[verify lattice.workflow_execution.ready_frontier.planner.scenario.all_fit]
#[test]
fn valid_policy_and_full_capacity_select_every_node() {
    let policy = policy(2, 2);
    validate_policy(&policy).expect("positive policy");
    let frontier = [
        candidate(FIRST_NODE, ConflictState::Pure),
        candidate(SECOND_NODE, ConflictState::Pure),
    ];
    let planned = plan(&policy, &ActiveReservations::default(), &frontier).expect("frontier plans");

    assert_eq!(planned.selected_nodes, vec![FIRST_NODE, SECOND_NODE]);
    assert!(planned.blocked_nodes.is_empty());
}

// r[verify lattice.workflow_execution.ready_frontier.planner.scenario.bounded_prefix]
#[test]
fn stable_capacity_prefix_has_typed_blockers() {
    let policy = policy(1, 1);
    let frontier = [
        candidate(FIRST_NODE, ConflictState::Pure),
        candidate(SECOND_NODE, ConflictState::Pure),
        candidate(THIRD_NODE, ConflictState::Pure),
    ];
    let planned = plan(&policy, &ActiveReservations::default(), &frontier).expect("frontier plans");

    assert_eq!(planned.selected_nodes, vec![FIRST_NODE]);
    assert_eq!(planned.blocked_nodes.len(), 2);
    assert!(planned.blocked_nodes.iter().all(|blocked| blocked.reason == Blocker::TotalCapacity));
}

#[test]
fn active_capacity_and_effect_conflicts_are_reserved() {
    let policy = policy(2, 2);
    let conflict_key = "file:root/path".to_string();
    let active = ActiveReservations {
        total: 1,
        lanes: std::collections::BTreeMap::from([(PROCESS_LANE.to_string(), 1)]),
        conflict_keys: std::collections::BTreeSet::from([conflict_key.clone()]),
        unresolved_conflict: false,
    };
    let frontier = [
        candidate(FIRST_NODE, ConflictState::Resolved(conflict_key)),
        candidate(SECOND_NODE, ConflictState::Pure),
    ];
    let planned = plan(&policy, &active, &frontier).expect("frontier plans");

    assert_eq!(planned.selected_nodes, vec![SECOND_NODE]);
    assert_eq!(planned.blocked_nodes[0].reason, Blocker::EffectConflict);
}

// r[verify lattice.workflow_execution.ready_frontier.conflicts.scenario.unresolved_alias]
#[test]
fn unresolved_effect_runs_alone_while_pure_work_can_overlap() {
    let policy = policy(3, 3);
    let frontier = [
        candidate(FIRST_NODE, ConflictState::Unresolved),
        candidate(SECOND_NODE, ConflictState::Resolved("file:other".to_string())),
        candidate(THIRD_NODE, ConflictState::Pure),
    ];
    let planned = plan(&policy, &ActiveReservations::default(), &frontier).expect("frontier plans");

    assert_eq!(planned.selected_nodes, vec![FIRST_NODE, THIRD_NODE]);
    assert_eq!(planned.blocked_nodes[0].reason, Blocker::UnresolvedConflict);
}

// r[verify lattice.workflow_execution.ready_frontier.dispatch.scenario.missing_authority]
#[test]
fn invalid_policy_and_missing_preparation_are_rejected() {
    assert_eq!(validate_policy(&policy(0, 1)), Err(PolicyError::ZeroTotal));
    let valid = policy(1, 1);
    let mut denied = candidate(FIRST_NODE, ConflictState::Pure);
    denied.authority_ready = false;
    let planned = plan(&valid, &ActiveReservations::default(), &[denied]).expect("frontier plans");

    assert!(planned.selected_nodes.is_empty());
    assert_eq!(planned.blocked_nodes[0].reason, Blocker::MissingAuthority);
}

#[test]
fn every_preparation_and_lifecycle_blocker_is_typed() {
    let valid = policy(1, 1);
    let cases: &[(Blocker, fn(&mut Candidate))] = &[
        (Blocker::MissingBackend, |candidate: &mut Candidate| candidate.backend_ready = false),
        (Blocker::MissingAuthority, |candidate: &mut Candidate| candidate.authority_ready = false),
        (Blocker::MissingCredential, |candidate: &mut Candidate| candidate.credential_ready = false),
        (Blocker::MissingAttempt, |candidate: &mut Candidate| candidate.attempt_ready = false),
        (Blocker::Suspension, |candidate: &mut Candidate| candidate.suspended = true),
        (Blocker::Cancellation, |candidate: &mut Candidate| candidate.cancelled = true),
        (Blocker::TerminalState, |candidate: &mut Candidate| candidate.terminal = true),
    ];
    for (expected, mutate) in cases {
        let mut blocked_candidate = candidate(FIRST_NODE, ConflictState::Pure);
        mutate(&mut blocked_candidate);
        let planned = plan(&valid, &ActiveReservations::default(), &[blocked_candidate]).expect("blocked plan");
        assert_eq!(planned.blocked_nodes[0].reason, *expected);
    }
}

#[test]
fn lane_capacity_is_independent_from_total_capacity() {
    let policy = CapacityPolicy {
        total_slots: 2,
        lane_slots: std::collections::BTreeMap::from([(PROCESS_LANE.to_string(), 1), ("pure".to_string(), 2)]),
    };
    let frontier = [
        candidate(FIRST_NODE, ConflictState::Pure),
        candidate(SECOND_NODE, ConflictState::Pure),
    ];
    let planned = plan(&policy, &ActiveReservations::default(), &frontier).expect("lane plan");
    assert_eq!(planned.selected_nodes, vec![FIRST_NODE]);
    assert_eq!(planned.blocked_nodes[0].reason, Blocker::ResourceCapacity);
}

#[test]
fn invalid_active_reservation_fails_closed() {
    let valid = policy(1, 1);
    let active = ActiveReservations {
        total: 2,
        ..ActiveReservations::default()
    };
    assert_eq!(plan(&valid, &active, &[]), Err(PolicyError::ActiveTotalExceedsPolicy));
}
