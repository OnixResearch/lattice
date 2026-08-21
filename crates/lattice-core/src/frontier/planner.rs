//! Pure bounded ready-frontier selection over explicit product facts.

pub const MAX_FRONTIER_NODE_COUNT: u32 = 4_096;
pub const MAX_RESOURCE_LANE_COUNT: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CapacityPolicy {
    pub total_slots: u32,
    pub lane_slots: std::collections::BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    ZeroTotal,
    TooManyTotalSlots,
    NoLanes,
    MissingLane,
    TooManyLanes,
    EmptyLane,
    UnknownLaneOverride,
    ZeroLane,
    LaneExceedsTotal,
    ActiveTotalExceedsPolicy,
    UnknownActiveLane,
    ActiveLaneExceedsTotal,
    ActiveLaneExceedsPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictState {
    Pure,
    Resolved(String),
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActiveReservations {
    pub total: u32,
    pub lanes: std::collections::BTreeMap<String, u32>,
    pub conflict_keys: std::collections::BTreeSet<String>,
    pub unresolved_conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub node_id: u64,
    pub lane: String,
    pub conflict: ConflictState,
    pub backend_ready: bool,
    pub authority_ready: bool,
    pub credential_ready: bool,
    pub attempt_ready: bool,
    pub suspended: bool,
    pub cancelled: bool,
    pub terminal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Blocker {
    TotalCapacity,
    ResourceCapacity,
    UnknownLane,
    UnresolvedConflict,
    EffectConflict,
    MissingBackend,
    MissingAuthority,
    MissingCredential,
    MissingAttempt,
    Suspension,
    Cancellation,
    TerminalState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedNode {
    pub node_id: u64,
    pub reason: Blocker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontierPlan {
    pub selected_nodes: Vec<u64>,
    pub blocked_nodes: Vec<BlockedNode>,
}

pub fn validate_policy(policy: &CapacityPolicy) -> Result<(), PolicyError> {
    if policy.total_slots == 0 {
        return Err(PolicyError::ZeroTotal);
    }
    if policy.total_slots > MAX_FRONTIER_NODE_COUNT {
        return Err(PolicyError::TooManyTotalSlots);
    }
    if policy.lane_slots.is_empty() {
        return Err(PolicyError::NoLanes);
    }
    if u32::try_from(policy.lane_slots.len()).map_or(true, |count| count > MAX_RESOURCE_LANE_COUNT) {
        return Err(PolicyError::TooManyLanes);
    }
    for (lane, slots) in &policy.lane_slots {
        if lane.trim().is_empty() {
            return Err(PolicyError::EmptyLane);
        }
        if *slots == 0 {
            return Err(PolicyError::ZeroLane);
        }
        if *slots > policy.total_slots {
            return Err(PolicyError::LaneExceedsTotal);
        }
    }
    debug_assert!(policy.total_slots <= MAX_FRONTIER_NODE_COUNT);
    debug_assert!(policy.lane_slots.values().all(|slots| *slots > 0));
    Ok(())
}

// r[impl lattice.workflow_execution.ready_frontier.planner]
pub fn plan(
    policy: &CapacityPolicy,
    active: &ActiveReservations,
    ordered_frontier: &[Candidate],
) -> Result<FrontierPlan, PolicyError> {
    validate_policy(policy)?;
    validate_active_reservations(policy, active)?;
    if u32::try_from(ordered_frontier.len()).map_or(true, |count| count > MAX_FRONTIER_NODE_COUNT) {
        return Err(PolicyError::TooManyTotalSlots);
    }

    let mut remaining_total_slots = policy.total_slots.saturating_sub(active.total);
    let mut remaining_lane_slots = policy.lane_slots.clone();
    for (lane, reserved) in &active.lanes {
        let remaining = remaining_lane_slots.get_mut(lane).ok_or(PolicyError::UnknownActiveLane)?;
        *remaining = remaining.saturating_sub(*reserved);
    }
    let mut selected_conflicts = active.conflict_keys.clone();
    let mut has_selected_unresolved = active.unresolved_conflict;
    let mut selected_nodes = Vec::with_capacity(ordered_frontier.len());
    let mut blocked_nodes = Vec::with_capacity(ordered_frontier.len());
    let mut observed_nodes = std::collections::BTreeSet::new();

    for candidate in ordered_frontier {
        if !observed_nodes.insert(candidate.node_id) {
            blocked_nodes.push(blocked(candidate.node_id, Blocker::TerminalState));
            continue;
        }
        let blocker = product_blocker(candidate)
            .or_else(|| capacity_blocker(candidate, remaining_total_slots, &remaining_lane_slots))
            .or_else(|| conflict_blocker(candidate, &selected_conflicts, has_selected_unresolved));
        if let Some(reason) = blocker {
            blocked_nodes.push(blocked(candidate.node_id, reason));
            continue;
        }

        remaining_total_slots = remaining_total_slots.saturating_sub(1);
        let lane_slots = remaining_lane_slots.get_mut(&candidate.lane).ok_or(PolicyError::EmptyLane)?;
        *lane_slots = lane_slots.saturating_sub(1);
        match &candidate.conflict {
            ConflictState::Pure => {}
            ConflictState::Resolved(key) => {
                selected_conflicts.insert(key.clone());
            }
            ConflictState::Unresolved => has_selected_unresolved = true,
        }
        selected_nodes.push(candidate.node_id);
    }

    debug_assert!(selected_nodes.len() <= ordered_frontier.len());
    debug_assert_eq!(selected_nodes.len().saturating_add(blocked_nodes.len()), ordered_frontier.len());
    debug_assert!(active.total <= policy.total_slots);
    Ok(FrontierPlan {
        selected_nodes,
        blocked_nodes,
    })
}

fn validate_active_reservations(policy: &CapacityPolicy, active: &ActiveReservations) -> Result<(), PolicyError> {
    if active.total > policy.total_slots {
        return Err(PolicyError::ActiveTotalExceedsPolicy);
    }
    for (lane, reserved) in &active.lanes {
        validate_active_lane(policy, ActiveLaneReservation {
            lane,
            reserved_slots: *reserved,
            active_total_slots: active.total,
        })?;
    }
    debug_assert!(active.lanes.len() <= policy.lane_slots.len());
    debug_assert!(active.lanes.values().all(|reserved| *reserved <= active.total));
    Ok(())
}

struct ActiveLaneReservation<'a> {
    lane: &'a str,
    reserved_slots: u32,
    active_total_slots: u32,
}

fn validate_active_lane(policy: &CapacityPolicy, reservation: ActiveLaneReservation<'_>) -> Result<(), PolicyError> {
    let policy_slots = policy.lane_slots.get(reservation.lane).ok_or(PolicyError::UnknownActiveLane)?;
    if reservation.reserved_slots > reservation.active_total_slots {
        return Err(PolicyError::ActiveLaneExceedsTotal);
    }
    if reservation.reserved_slots > *policy_slots {
        return Err(PolicyError::ActiveLaneExceedsPolicy);
    }
    Ok(())
}

fn product_blocker(candidate: &Candidate) -> Option<Blocker> {
    let blocker = [
        (candidate.terminal, Blocker::TerminalState),
        (candidate.cancelled, Blocker::Cancellation),
        (candidate.suspended, Blocker::Suspension),
        (!candidate.backend_ready, Blocker::MissingBackend),
        (!candidate.authority_ready, Blocker::MissingAuthority),
        (!candidate.credential_ready, Blocker::MissingCredential),
        (!candidate.attempt_ready, Blocker::MissingAttempt),
    ]
    .into_iter()
    .find_map(|(is_blocked, reason)| is_blocked.then_some(reason));
    debug_assert!(!matches!(blocker, Some(Blocker::MissingAuthority)) || !candidate.authority_ready);
    debug_assert!(blocker.is_some() || candidate.backend_ready);
    blocker
}

fn capacity_blocker(
    candidate: &Candidate,
    remaining_total_slots: u32,
    remaining_lane_slots: &std::collections::BTreeMap<String, u32>,
) -> Option<Blocker> {
    if remaining_total_slots == 0 {
        return Some(Blocker::TotalCapacity);
    }
    match remaining_lane_slots.get(&candidate.lane) {
        None => Some(Blocker::UnknownLane),
        Some(0) => Some(Blocker::ResourceCapacity),
        Some(_) => None,
    }
}

fn conflict_blocker(
    candidate: &Candidate,
    selected_conflicts: &std::collections::BTreeSet<String>,
    has_selected_unresolved: bool,
) -> Option<Blocker> {
    match &candidate.conflict {
        ConflictState::Pure => None,
        ConflictState::Unresolved if has_selected_unresolved || !selected_conflicts.is_empty() => {
            Some(Blocker::UnresolvedConflict)
        }
        ConflictState::Unresolved => None,
        ConflictState::Resolved(_) if has_selected_unresolved => Some(Blocker::UnresolvedConflict),
        ConflictState::Resolved(key) if selected_conflicts.contains(key) => Some(Blocker::EffectConflict),
        ConflictState::Resolved(_) => None,
    }
}

const fn blocked(node_id: u64, reason: Blocker) -> BlockedNode {
    BlockedNode { node_id, reason }
}

#[cfg(test)]
#[path = "planner/tests.rs"]
mod tests;
