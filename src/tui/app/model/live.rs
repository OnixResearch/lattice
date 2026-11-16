//! Live execution observations and task ownership.

use super::*;

/// Command palette data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunState {
    pub active: bool,
    pub node_statuses: HashMap<u64, NodeRunStatus>,
    pub node_started: HashMap<u64, std::time::Instant>,
    pub node_start_times: HashMap<u64, String>,
    pub node_durations_millis: HashMap<u64, u64>,
    pub node_tokens: HashMap<u64, u64>,
    pub node_cost_microunits: HashMap<u64, u64>,
    pub node_attempts: HashMap<u64, u32>,
    pub active_replay_timeline: Option<ReplayTimeline>,
}

impl RunState {
    pub(in crate::tui::app) fn idle() -> Self {
        Self {
            active: false,
            node_statuses: HashMap::new(),
            node_started: HashMap::new(),
            node_start_times: HashMap::new(),
            node_durations_millis: HashMap::new(),
            node_tokens: HashMap::new(),
            node_cost_microunits: HashMap::new(),
            node_attempts: HashMap::new(),
            active_replay_timeline: None,
        }
    }
}

pub(in crate::tui::app) struct ActiveRun {
    pub(in crate::tui::app) run_id: RunId,
    pub(in crate::tui::app) workflow: Workflow,
    pub(in crate::tui::app) receiver: mpsc::Receiver<RunEvent>,
    pub(in crate::tui::app) handle: JoinHandle<()>,
    pub(in crate::tui::app) cancellation: Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Debug)]
pub(in crate::tui::app) enum RunEvent {
    ReplayStarted {
        run_id: RunId,
        events: Vec<crate::replay::ReplayEventRecord>,
    },
    NodeProgress(crate::engine::executor::NodeProgress),
    ObservationIncomplete,
    Completed {
        run_id: RunId,
        workflow: Box<Workflow>,
        results: Vec<StepResult>,
        observed_completion_order: Vec<crate::engine::workflow::NodeId>,
    },
}
