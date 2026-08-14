#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CairnSnapshot {
    pub schema: String,
    pub active_count: u64,
    pub has_work: bool,
    pub selected: Option<String>,
    pub active_changes: Vec<String>,
}

// r[impl lattice.guarded_agent_templates.cairn]
pub fn make_cairn_snapshot(
    mut active_changes: Vec<String>,
    requested: Option<&str>,
) -> Result<CairnSnapshot, BlockReason> {
    if active_changes.len() > MAX_ACTIVE_CHANGES || active_changes.iter().any(|id| !valid_change_id(id)) {
        return Err(BlockReason::MalformedSnapshot);
    }
    active_changes.sort();
    let mut unique = std::collections::BTreeSet::new();
    if active_changes.iter().any(|id| !unique.insert(id.clone())) {
        return Err(BlockReason::MalformedSnapshot);
    }
    let selected = match requested {
        Some(id) if !valid_change_id(id) => return Err(BlockReason::SelectionMissing),
        Some(id) if active_changes.binary_search_by(|candidate| candidate.as_str().cmp(id)).is_err() => {
            return Err(BlockReason::SelectionMissing);
        }
        Some(id) => Some(id.to_owned()),
        None => active_changes.first().cloned(),
    };
    let active_count = u64::try_from(active_changes.len()).map_err(|_| BlockReason::MalformedSnapshot)?;
    let snapshot = CairnSnapshot {
        schema: "lattice.guarded.cairn-snapshot.v1".to_owned(),
        active_count,
        has_work: active_count > 0,
        selected,
        active_changes,
    };
    debug_assert!(valid_snapshot(&snapshot));
    debug_assert_eq!(snapshot.has_work, snapshot.selected.is_some());
    Ok(snapshot)
}

// r[impl lattice.guarded_agent_templates.cairn]
pub fn decide_cairn_progress(
    policy: ProgressPolicy,
    before: &CairnSnapshot,
    after: &CairnSnapshot,
    validation_passed: bool,
) -> ProgressDecision {
    if !valid_snapshot(before) || !valid_snapshot(after) {
        return blocked(BlockReason::MalformedSnapshot);
    }
    debug_assert_eq!(usize::try_from(before.active_count).ok(), Some(before.active_changes.len()));
    debug_assert_eq!(usize::try_from(after.active_count).ok(), Some(after.active_changes.len()));
    let Some(selected) = before.selected.as_ref() else {
        if before.active_count == 0 && after.active_count == 0 && validation_passed {
            return ProgressDecision::Complete { before: 0, delta: 0 };
        }
        return blocked(BlockReason::SelectionMissing);
    };
    if !before.active_changes.iter().any(|id| id == selected) {
        return blocked(BlockReason::SelectionChanged);
    }
    let is_selected_removed = !after.active_changes.iter().any(|id| id == selected);
    decide_progress(policy, &ProgressObservation {
        metrics: vec![
            MetricFact {
                phase: ObservationPhase::Before,
                value: before.active_count.to_string(),
            },
            MetricFact {
                phase: ObservationPhase::After,
                value: after.active_count.to_string(),
            },
        ],
        prerequisites: ProgressPrerequisites {
            validation_passed,
            restoration_required: false,
            restoration_confirmed: false,
            selected_change_removal_required: true,
            selected_change_removed: is_selected_removed,
        },
    })
}

fn valid_snapshot(snapshot: &CairnSnapshot) -> bool {
    snapshot.schema == "lattice.guarded.cairn-snapshot.v1"
        && snapshot.has_work == (snapshot.active_count > 0)
        && usize::try_from(snapshot.active_count).ok() == Some(snapshot.active_changes.len())
        && snapshot.active_changes.len() <= MAX_ACTIVE_CHANGES
        && snapshot.active_changes.windows(2).all(|pair| pair[0] < pair[1])
        && snapshot.active_changes.iter().all(|id| valid_change_id(id))
}

fn valid_change_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_CHANGE_ID_BYTES
        && id.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OctetSnapshot {
    pub schema: String,
    pub findings: String,
    pub restoration_confirmed: bool,
    pub policy_identity_before: String,
    pub policy_identity_after: String,
}

// r[impl lattice.guarded_agent_templates.octet]
pub fn decide_octet_progress(
    policy: ProgressPolicy,
    before: &OctetSnapshot,
    after: &OctetSnapshot,
    validation_passed: bool,
) -> ProgressDecision {
    if !valid_octet_snapshot(before) || !valid_octet_snapshot(after) {
        return blocked(BlockReason::MalformedSnapshot);
    }
    debug_assert!(valid_identity(&before.policy_identity_before));
    debug_assert!(valid_identity(&after.policy_identity_after));
    let is_restoration_confirmed = before.restoration_confirmed
        && after.restoration_confirmed
        && before.policy_identity_before == before.policy_identity_after
        && after.policy_identity_before == after.policy_identity_after
        && before.policy_identity_before == after.policy_identity_before;
    decide_progress(policy, &ProgressObservation {
        metrics: vec![
            MetricFact {
                phase: ObservationPhase::Before,
                value: before.findings.clone(),
            },
            MetricFact {
                phase: ObservationPhase::After,
                value: after.findings.clone(),
            },
        ],
        prerequisites: ProgressPrerequisites {
            validation_passed,
            restoration_required: true,
            restoration_confirmed: is_restoration_confirmed,
            selected_change_removal_required: false,
            selected_change_removed: false,
        },
    })
}

fn valid_octet_snapshot(snapshot: &OctetSnapshot) -> bool {
    snapshot.schema == "lattice.guarded.octet-snapshot.v1"
        && valid_identity(&snapshot.policy_identity_before)
        && valid_identity(&snapshot.policy_identity_after)
}

fn valid_identity(identity: &str) -> bool {
    const HEX_CHARACTERS_PER_BYTE: usize = 2;
    const DIGEST_HEX_LENGTH: usize = blake3::OUT_LEN.saturating_mul(HEX_CHARACTERS_PER_BYTE);
    identity.len() == DIGEST_HEX_LENGTH
        && identity.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
