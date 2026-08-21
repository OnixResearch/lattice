//! Product scheduling policy and immutable ready-frontier facts.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the scheduling policy module keeps its small domain vocabulary visible at the composition boundary"
)]

pub(super) mod runtime;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice_core::scheduling::ActiveReservations;
use lattice_core::scheduling::Candidate;
use lattice_core::scheduling::CapacityPolicy;
use lattice_core::scheduling::ConflictState;
use lattice_core::scheduling::PolicyError;

use super::workflow::NodeConfig;

pub const PURE_LANE: &str = "pure";
pub const PROCESS_LANE: &str = "process";
pub const NETWORK_LANE: &str = "network";
pub const FILESYSTEM_LANE: &str = "filesystem";
pub const IMAGE_LANE: &str = "image";
pub const AGENT_LANE: &str = "agent";
pub const OPERATION_LANE: &str = "operation";
pub const SUBWORKFLOW_LANE: &str = "subworkflow";
pub const APPROVAL_LANE: &str = "approval";

const POLICY_IDENTITY_DOMAIN: &[u8] = b"lattice.ready-frontier-policy.v1\0";
const OPERATION_RESOURCE_ALIAS: &str = "resource_alias";
const RESOURCE_LANE_COUNT: usize = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reservation {
    pub lane: String,
    pub conflict: ConflictState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductFacts {
    pub is_suspended: bool,
    pub is_cancelled: bool,
}

// r[impl lattice.workflow_execution.ready_frontier.capacity]
pub fn capacity_policy(
    total_slots: u32,
    lane_overrides: &BTreeMap<String, u32>,
) -> Result<CapacityPolicy, PolicyError> {
    let uniform_policy = CapacityPolicy {
        total_slots,
        lane_slots: uniform_lane_limits(total_slots),
    };
    lattice_core::scheduling::validate_policy(&uniform_policy)?;
    debug_assert_eq!(uniform_policy.lane_slots.len(), RESOURCE_LANE_COUNT);
    debug_assert!(uniform_policy.lane_slots.values().all(|slots| *slots == total_slots));
    if lane_overrides.is_empty() {
        return (total_slots == 1).then_some(uniform_policy).ok_or(PolicyError::MissingLane);
    }
    if lane_overrides.keys().any(|lane| !all_lanes().contains(&lane.as_str())) {
        return Err(PolicyError::UnknownLaneOverride);
    }
    if lane_overrides.len() != RESOURCE_LANE_COUNT {
        return Err(PolicyError::MissingLane);
    }
    let policy = CapacityPolicy {
        total_slots,
        lane_slots: lane_overrides.clone(),
    };
    lattice_core::scheduling::validate_policy(&policy)?;
    Ok(policy)
}

pub fn uniform_lane_limits(total_slots: u32) -> BTreeMap<String, u32> {
    all_lanes().into_iter().map(|lane| (lane.to_string(), total_slots)).collect()
}

pub fn serial_capacity_policy() -> CapacityPolicy {
    let total_slots = 1;
    CapacityPolicy {
        total_slots,
        lane_slots: uniform_lane_limits(total_slots),
    }
}

pub fn capacity_policy_identity(policy: &CapacityPolicy) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(policy)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(POLICY_IDENTITY_DOMAIN);
    hasher.update(&bytes);
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

pub fn candidate(node_id: u64, config: Option<&NodeConfig>, facts: ProductFacts) -> Candidate {
    Candidate {
        node_id,
        lane: lane(config).to_string(),
        conflict: conflict(config),
        backend_ready: config.is_some(),
        authority_ready: true,
        credential_ready: true,
        attempt_ready: true,
        suspended: facts.is_suspended,
        cancelled: facts.is_cancelled,
        terminal: false,
    }
}

pub fn reservation(config: Option<&NodeConfig>) -> Reservation {
    Reservation {
        lane: lane(config).to_string(),
        conflict: conflict(config),
    }
}

pub fn active_reservations(running: &BTreeMap<u64, Reservation>) -> ActiveReservations {
    let mut total = 0_u32;
    let mut lanes = all_lanes().into_iter().map(|lane| (lane.to_string(), 0_u32)).collect::<BTreeMap<_, _>>();
    for reservation in running.values() {
        total = total.saturating_add(1);
        if let Some(count) = lanes.get_mut(&reservation.lane) {
            *count = count.saturating_add(1);
        }
    }
    let conflict_keys = running
        .values()
        .filter_map(|reservation| match &reservation.conflict {
            ConflictState::Resolved(key) => Some(key.clone()),
            ConflictState::Pure | ConflictState::Unresolved => None,
        })
        .collect::<BTreeSet<_>>();
    let has_unresolved_conflict = running.values().any(|reservation| reservation.conflict == ConflictState::Unresolved);
    debug_assert!(u64::try_from(running.len()).is_ok_and(|count| count == u64::from(total)));
    debug_assert_eq!(u64::from(total), lanes.values().map(|count| u64::from(*count)).sum::<u64>());
    ActiveReservations {
        total,
        lanes,
        conflict_keys,
        unresolved_conflict: has_unresolved_conflict,
    }
}

fn lane(config: Option<&NodeConfig>) -> &'static str {
    match config {
        Some(NodeConfig::Shell { .. }) => PROCESS_LANE,
        Some(NodeConfig::Http { .. }) => NETWORK_LANE,
        Some(NodeConfig::File { .. }) => FILESYSTEM_LANE,
        Some(NodeConfig::ImageInput { .. }) => IMAGE_LANE,
        Some(NodeConfig::Agent(_)) => AGENT_LANE,
        Some(NodeConfig::Operation(_)) => OPERATION_LANE,
        Some(NodeConfig::Condition(_) | NodeConfig::Merge(_)) => PURE_LANE,
        Some(NodeConfig::Subworkflow(_)) => SUBWORKFLOW_LANE,
        Some(NodeConfig::Approval(_)) => APPROVAL_LANE,
        None => PURE_LANE,
    }
}

// r[impl lattice.workflow_execution.ready_frontier.conflicts]
fn conflict(config: Option<&NodeConfig>) -> ConflictState {
    match config {
        Some(NodeConfig::Condition(_) | NodeConfig::Merge(_) | NodeConfig::ImageInput { .. }) => ConflictState::Pure,
        Some(NodeConfig::File {
            operation: crate::file_node::FileOperationKind::Read,
            ..
        }) => ConflictState::Pure,
        Some(NodeConfig::Operation(plan)) => operation_conflict(plan),
        Some(_) | None => ConflictState::Unresolved,
    }
}

fn operation_conflict(plan: &crate::operation::InvocationPlan) -> ConflictState {
    let Some(alias) = plan.target.get(OPERATION_RESOURCE_ALIAS).and_then(serde_json::Value::as_str) else {
        return ConflictState::Unresolved;
    };
    if alias.trim().is_empty() {
        return ConflictState::Unresolved;
    }
    ConflictState::Resolved(format!("operation:{alias}"))
}

const fn all_lanes() -> [&'static str; RESOURCE_LANE_COUNT] {
    [
        PURE_LANE,
        PROCESS_LANE,
        NETWORK_LANE,
        FILESYSTEM_LANE,
        IMAGE_LANE,
        AGENT_LANE,
        OPERATION_LANE,
        SUBWORKFLOW_LANE,
        APPROVAL_LANE,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUNDED_TOTAL: u32 = 3;
    const BOUNDED_PROCESS: u32 = 2;

    #[test]
    fn omission_is_serial_and_policy_identity_is_deterministic() {
        let policy = serial_capacity_policy();
        assert_eq!(policy.total_slots, 1);
        assert!(policy.lane_slots.values().all(|slots| *slots == 1));
        let first = capacity_policy_identity(&policy).expect("policy identity");
        let second = capacity_policy_identity(&policy).expect("policy identity");
        assert_eq!(first, second);
    }

    #[test]
    fn named_lane_override_is_bounded_by_total() {
        let mut lane_limits = uniform_lane_limits(BOUNDED_TOTAL);
        lane_limits.insert(PROCESS_LANE.to_string(), BOUNDED_PROCESS);
        let policy = capacity_policy(BOUNDED_TOTAL, &lane_limits).expect("bounded lane policy");
        assert_eq!(policy.total_slots, BOUNDED_TOTAL);
        assert_eq!(policy.lane_slots[PROCESS_LANE], BOUNDED_PROCESS);
        assert_eq!(policy.lane_slots[PURE_LANE], BOUNDED_TOTAL);
    }

    #[test]
    fn unknown_lane_and_zero_total_fail_closed() {
        let unknown = BTreeMap::from([("unknown".to_string(), 1)]);
        assert_eq!(capacity_policy(BOUNDED_TOTAL, &unknown), Err(PolicyError::UnknownLaneOverride));
        assert_eq!(capacity_policy(0, &BTreeMap::new()), Err(PolicyError::ZeroTotal));
        assert_eq!(capacity_policy(BOUNDED_TOTAL, &BTreeMap::new()), Err(PolicyError::MissingLane));
    }

    #[test]
    fn unresolved_effect_reservation_is_explicit() {
        let config = NodeConfig::shell("printf test");
        let reservation = reservation(Some(&config));
        assert_eq!(reservation.lane, PROCESS_LANE);
        assert_eq!(reservation.conflict, ConflictState::Unresolved);
    }
}
