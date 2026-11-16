//! Run inspection data and persisted receipt projections.

use super::*;

pub(in crate::tui::app) struct RunCounts {
    pub(in crate::tui::app) cache_hits: usize,
    pub(in crate::tui::app) failures: usize,
    pub(in crate::tui::app) nodes: usize,
}

pub fn format_run_record(record: &RunRecord) -> String {
    format!(
        "{} │ {} nodes │ {} failures │ {} cache hits",
        record.run_id, record.node_count, record.failures, record.cache_hits,
    )
}

pub fn trigger_status_lines_from_history(
    triggers: &[crate::triggers::TriggerDefinition],
    records: &[crate::triggers::TriggerRunRecord],
) -> Vec<String> {
    let mut latest_by_trigger = HashMap::new();
    for record in records {
        latest_by_trigger.entry(record.trigger_id.as_str()).or_insert(record.status);
    }
    let lines = triggers
        .iter()
        .map(|trigger| {
            let status = latest_by_trigger
                .get(trigger.id.as_str())
                .map(|status| match status {
                    crate::triggers::TriggerRunStatus::Started => "started",
                    crate::triggers::TriggerRunStatus::Rejected => "rejected",
                    crate::triggers::TriggerRunStatus::Failed => "failed",
                })
                .unwrap_or("none");
            format!(
                "{} │ {:?} │ {} │ last={status}",
                trigger.id,
                trigger.kind,
                if trigger.enabled { "enabled" } else { "disabled" },
            )
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(lines.len(), triggers.len());
    lines
}

// r[impl n8nclone.tui_ux.run_history.details.persist]
// r[impl lattice.workflow_execution.headless_run_details]
pub fn run_detail_record_from_results(run_id: RunId, workflow: &Workflow, results: &[StepResult]) -> RunDetailRecord {
    crate::run::history::detail_record_from_results(run_id, workflow, results)
}

pub(in crate::tui::app) fn canceled_run_detail_record(
    run_id: RunId,
    workflow: &Workflow,
    statuses: &HashMap<u64, NodeRunStatus>,
) -> RunDetailRecord {
    let nodes = workflow
        .graph
        .nodes()
        .map(|node| {
            let status = statuses.get(&node.id.0).copied().unwrap_or(NodeRunStatus::Skipped);
            RunNodeDetail {
                node_id: node.id,
                node_label: node.label.clone(),
                status: run_detail_status_from_node_run_status(status),
                cached: false,
                outputs: Vec::new(),
                error: None,
            }
        })
        .collect::<Vec<_>>();
    let observed_completion_order = nodes.iter().map(|node| node.node_id).collect::<Vec<_>>();
    debug_assert_eq!(nodes.len(), workflow.graph.node_count());
    debug_assert_eq!(observed_completion_order.len(), nodes.len());
    RunDetailRecord {
        run_id,
        nodes,
        observed_completion_order,
    }
}

pub(in crate::tui::app) fn run_detail_status_from_node_run_status(status: NodeRunStatus) -> RunNodeDetailStatus {
    match status {
        NodeRunStatus::Pending => RunNodeDetailStatus::Pending,
        NodeRunStatus::Running => RunNodeDetailStatus::Running,
        NodeRunStatus::Completed => RunNodeDetailStatus::Completed,
        NodeRunStatus::Failed => RunNodeDetailStatus::Failed,
        NodeRunStatus::Skipped => RunNodeDetailStatus::Skipped,
    }
}

#[derive(Debug, Clone)]
pub struct RunHistoryState {
    pub open: bool,
    pub records: Vec<RunRecord>,
    pub selected: usize,
    pub detail: Option<String>,
    pub detail_record: Option<RunDetailRecord>,
    pub selected_detail_node: usize,
    pub replay_timeline: Option<ReplayTimeline>,
    pub replay_cursor: ReplayCursor,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApprovalState {
    pub open: bool,
    pub records: Vec<crate::workflow::gate::shell::SafeApprovalView>,
    pub selected: usize,
    pub detail: Option<String>,
    pub message: Option<String>,
}

impl ApprovalState {
    pub(in crate::tui::app) fn closed() -> Self {
        Self {
            open: false,
            records: Vec::new(),
            selected: 0,
            detail: None,
            message: None,
        }
    }

    pub(in crate::tui::app) fn open(
        records: Vec<crate::workflow::gate::shell::SafeApprovalView>,
        message: Option<String>,
    ) -> Self {
        Self {
            open: true,
            records,
            selected: 0,
            detail: None,
            message,
        }
    }
}

impl RunHistoryState {
    pub(in crate::tui::app) fn closed() -> Self {
        Self {
            open: false,
            records: Vec::new(),
            selected: 0,
            detail: None,
            detail_record: None,
            selected_detail_node: 0,
            replay_timeline: None,
            replay_cursor: crate::replay::replay_cursor(0),
            message: None,
        }
    }

    pub(in crate::tui::app) fn open(records: Vec<RunRecord>, message: Option<String>) -> Self {
        Self {
            open: true,
            records,
            selected: 0,
            detail: None,
            detail_record: None,
            selected_detail_node: 0,
            replay_timeline: None,
            replay_cursor: crate::replay::replay_cursor(0),
            message,
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::tui::app) enum RunDetailLoad {
    Loaded(RunDetailRecord),
    Missing,
    Failed(String),
}

pub(in crate::tui::app) fn classify_run_detail_load(result: Result<Option<RunDetailRecord>, String>) -> RunDetailLoad {
    match result {
        Ok(Some(record)) => RunDetailLoad::Loaded(record),
        Ok(None) => RunDetailLoad::Missing,
        Err(error) => RunDetailLoad::Failed(error),
    }
}
