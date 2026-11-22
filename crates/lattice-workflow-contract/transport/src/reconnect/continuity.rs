//! Observation continuity check `lattice.workflow-observation-continuity.v1`.
//!
//! A pure comparison of the last retained observation with a newly received one,
//! typically across a reconnect, an endpoint restart, or a vsock reset. It
//! rejects observations that cannot belong to the same execution history and
//! names every log stream whose position is no longer complete. Consumers
//! implement the same check from `fixtures/continuity-fixtures-v1.json`.

#![allow(
    unknown_lints,
    non_trait_imports,
    allocation_in_loop,
    unbounded_collection_growth,
    reason = "inputs are admitted observations whose log references are bounded by the producer contract limits"
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice_workflow_contract::ExecutionObservation;
use lattice_workflow_contract::Label;
use lattice_workflow_contract::LogStream;
use serde::Deserialize;
use serde::Serialize;

pub const OBSERVATION_CONTINUITY_SCHEMA: &str = "lattice.workflow-observation-continuity.v1";

/// One log stream of one node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogStreamKey {
    pub node_id: Label,
    pub stream: LogStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuityRejection {
    RequestChanged,
    ExecutionChanged,
    WorkflowChanged,
    SequenceDecreased,
    TerminalReverted,
    CursorRegressed,
}

/// Result of comparing two observations.
///
/// `Gap.nodes` lists each node with at least one incomplete stream and
/// `Gap.streams` lists each incomplete (node, stream) pair; both are sorted and
/// free of duplicates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub enum Continuity {
    Continuous,
    Gap {
        nodes: Vec<Label>,
        streams: Vec<LogStreamKey>,
    },
    Reject {
        reason: ContinuityRejection,
    },
}

/// C7 continuity check.
///
/// Rejects a changed request, execution or workflow identity, a decreased
/// sequence, a terminal state that changed, and a log cursor that moved
/// backwards for any (node, stream) pair. Otherwise reports a gap for every log
/// reference in `next` marked `gap_before`, not `retained`, or `truncated`.
// r[impl lattice.workflow_interchange.observation_continuity]
#[must_use]
pub fn continuity(previous: Option<&ExecutionObservation>, next: &ExecutionObservation) -> Continuity {
    if let Some(previous) = previous
        && let Some(reason) = rejection(previous, next)
    {
        return Continuity::Reject { reason };
    }
    gaps(next)
}

fn rejection(previous: &ExecutionObservation, next: &ExecutionObservation) -> Option<ContinuityRejection> {
    debug_assert!(!previous.execution_identity.as_str().is_empty());
    debug_assert!(!next.execution_identity.as_str().is_empty());
    if previous.request_identity != next.request_identity {
        return Some(ContinuityRejection::RequestChanged);
    }
    if previous.execution_identity != next.execution_identity {
        return Some(ContinuityRejection::ExecutionChanged);
    }
    if previous.workflow_identity != next.workflow_identity {
        return Some(ContinuityRejection::WorkflowChanged);
    }
    if next.sequence < previous.sequence {
        return Some(ContinuityRejection::SequenceDecreased);
    }
    let was_terminal = previous.terminal || previous.state.is_terminal();
    if was_terminal && (next.state != previous.state || !next.terminal) {
        return Some(ContinuityRejection::TerminalReverted);
    }
    let previous_cursors = latest_cursors(previous);
    let has_regressed = latest_cursors(next)
        .iter()
        .any(|(key, cursor)| previous_cursors.get(key).is_some_and(|previous_cursor| cursor < previous_cursor));
    if has_regressed {
        return Some(ContinuityRejection::CursorRegressed);
    }
    None
}

fn latest_cursors(observation: &ExecutionObservation) -> BTreeMap<(&Label, LogStream), u64> {
    let mut cursors = BTreeMap::new();
    for reference in &observation.logs {
        let latest = cursors.entry((&reference.node_id, reference.stream)).or_insert(reference.cursor);
        *latest = (*latest).max(reference.cursor);
    }
    cursors
}

fn gaps(next: &ExecutionObservation) -> Continuity {
    let streams = next
        .logs
        .iter()
        .filter(|reference| reference.gap_before || !reference.retained || reference.truncated)
        .map(|reference| LogStreamKey {
            node_id: reference.node_id.clone(),
            stream: reference.stream,
        })
        .collect::<BTreeSet<_>>();
    if streams.is_empty() {
        return Continuity::Continuous;
    }
    let nodes = streams.iter().map(|key| key.node_id.clone()).collect::<BTreeSet<_>>();
    Continuity::Gap {
        nodes: nodes.into_iter().collect(),
        streams: streams.into_iter().collect(),
    }
}
