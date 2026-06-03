//! Pure scheduling core and scheduler-daemon outcome models.

use std::collections::HashSet;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::headless::HeadlessRunSummary;

const MILLIS_PER_SECOND: i64 = 1_000;
const MIN_INTERVAL_SECS: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleDefinition {
    pub id: String,
    pub workflow_name: String,
    pub recurrence: RecurrenceRule,
    pub enabled: bool,
    pub overlap_policy: OverlapPolicy,
    pub missed_run_policy: MissedRunPolicy,
    pub next_run_epoch_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceRule {
    EverySeconds { interval_secs: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlapPolicy {
    Forbid,
    Allow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissedRunPolicy {
    RunOnce,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleDiagnostic {
    pub schedule_id: String,
    pub code: ScheduleDiagnosticCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleDiagnosticCode {
    EmptyScheduleId,
    EmptyWorkflowName,
    MissingWorkflow,
    InvalidRecurrence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueRunPlan {
    pub schedule_id: String,
    pub workflow_name: String,
    pub due_epoch_ms: i64,
    pub next_run_epoch_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedScheduleRun {
    pub schedule_id: String,
    pub workflow_name: String,
    pub reason: ScheduleSkipReason,
    pub next_run_epoch_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleSkipReason {
    Disabled,
    OverlapPrevented,
    MissedRunSkipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DueRunEvaluation {
    pub due_runs: Vec<DueRunPlan>,
    pub skipped_runs: Vec<SkippedScheduleRun>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleDaemonOutcome {
    Started {
        schedule_id: String,
        workflow_name: String,
        summary: Box<HeadlessRunSummary>,
        next_run_epoch_ms: i64,
    },
    Error {
        schedule_id: String,
        workflow_name: String,
        message: String,
        next_run_epoch_ms: i64,
    },
    Skipped(SkippedScheduleRun),
}

impl fmt::Display for ScheduleDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "schedule {}: {}", self.schedule_id, self.message)
    }
}

// r[impl n8nclone.scheduling.schedule_model]
pub fn validate_schedule(schedule: &ScheduleDefinition, workflow_names: &HashSet<String>) -> Vec<ScheduleDiagnostic> {
    let mut diagnostics = Vec::new();
    if schedule.id.trim().is_empty() {
        diagnostics.push(ScheduleDiagnostic {
            schedule_id: schedule.id.clone(),
            code: ScheduleDiagnosticCode::EmptyScheduleId,
            message: "schedule id cannot be empty".to_string(),
        });
    }
    if schedule.workflow_name.trim().is_empty() {
        diagnostics.push(ScheduleDiagnostic {
            schedule_id: schedule.id.clone(),
            code: ScheduleDiagnosticCode::EmptyWorkflowName,
            message: "workflow name cannot be empty".to_string(),
        });
    } else if !workflow_names.contains(&schedule.workflow_name) {
        diagnostics.push(ScheduleDiagnostic {
            schedule_id: schedule.id.clone(),
            code: ScheduleDiagnosticCode::MissingWorkflow,
            message: format!("workflow '{}' does not exist", schedule.workflow_name),
        });
    }
    if schedule.recurrence.interval_secs() < MIN_INTERVAL_SECS {
        diagnostics.push(ScheduleDiagnostic {
            schedule_id: schedule.id.clone(),
            code: ScheduleDiagnosticCode::InvalidRecurrence,
            message: "schedule interval must be at least one second".to_string(),
        });
    }
    diagnostics
}

// r[impl n8nclone.scheduling.due_evaluator]
pub fn evaluate_due_runs(
    schedules: &[ScheduleDefinition],
    current_epoch_ms: i64,
    running_schedule_ids: &HashSet<String>,
) -> DueRunEvaluation {
    let mut evaluation = DueRunEvaluation::default();
    for schedule in schedules {
        if !schedule.enabled {
            evaluation
                .skipped_runs
                .push(skipped(schedule, ScheduleSkipReason::Disabled, schedule.next_run_epoch_ms));
            continue;
        }
        if schedule.next_run_epoch_ms > current_epoch_ms {
            continue;
        }
        if schedule.overlap_policy == OverlapPolicy::Forbid && running_schedule_ids.contains(&schedule.id) {
            evaluation.skipped_runs.push(skipped(
                schedule,
                ScheduleSkipReason::OverlapPrevented,
                schedule.next_run_epoch_ms,
            ));
            continue;
        }
        if schedule.missed_run_policy == MissedRunPolicy::Skip
            && missed_by_at_least_one_interval(schedule, current_epoch_ms)
        {
            evaluation.skipped_runs.push(skipped(
                schedule,
                ScheduleSkipReason::MissedRunSkipped,
                next_after(schedule, current_epoch_ms),
            ));
            continue;
        }
        evaluation.due_runs.push(DueRunPlan {
            schedule_id: schedule.id.clone(),
            workflow_name: schedule.workflow_name.clone(),
            due_epoch_ms: schedule.next_run_epoch_ms,
            next_run_epoch_ms: next_after(schedule, current_epoch_ms),
        });
    }
    evaluation
}

impl RecurrenceRule {
    pub fn interval_secs(self) -> u64 {
        match self {
            RecurrenceRule::EverySeconds { interval_secs } => interval_secs,
        }
    }

    fn interval_ms(self) -> i64 {
        let seconds = i64::try_from(self.interval_secs()).expect("schedule interval seconds fit in i64");
        seconds.saturating_mul(MILLIS_PER_SECOND)
    }
}

fn skipped(schedule: &ScheduleDefinition, reason: ScheduleSkipReason, next_run_epoch_ms: i64) -> SkippedScheduleRun {
    SkippedScheduleRun {
        schedule_id: schedule.id.clone(),
        workflow_name: schedule.workflow_name.clone(),
        reason,
        next_run_epoch_ms,
    }
}

fn missed_by_at_least_one_interval(schedule: &ScheduleDefinition, current_epoch_ms: i64) -> bool {
    current_epoch_ms.saturating_sub(schedule.next_run_epoch_ms) >= schedule.recurrence.interval_ms()
}

fn next_after(schedule: &ScheduleDefinition, current_epoch_ms: i64) -> i64 {
    let interval_ms = schedule.recurrence.interval_ms();
    let mut next = schedule.next_run_epoch_ms;
    while next <= current_epoch_ms {
        next = next.saturating_add(interval_ms);
    }
    next
}

pub trait ScheduledWorkflowRunner {
    fn run_scheduled_workflow(&mut self, workflow_name: &str) -> Result<HeadlessRunSummary, String>;
}

// r[impl n8nclone.scheduling.daemon.run_due]
pub fn run_due_plans<R: ScheduledWorkflowRunner>(plans: &[DueRunPlan], runner: &mut R) -> Vec<ScheduleDaemonOutcome> {
    plans
        .iter()
        .map(|plan| match runner.run_scheduled_workflow(&plan.workflow_name) {
            Ok(summary) => ScheduleDaemonOutcome::Started {
                schedule_id: plan.schedule_id.clone(),
                workflow_name: plan.workflow_name.clone(),
                summary: Box::new(summary),
                next_run_epoch_ms: plan.next_run_epoch_ms,
            },
            Err(message) => ScheduleDaemonOutcome::Error {
                schedule_id: plan.schedule_id.clone(),
                workflow_name: plan.workflow_name.clone(),
                message,
                next_run_epoch_ms: plan.next_run_epoch_ms,
            },
        })
        .collect()
}

pub fn render_daemon_outcome(outcome: &ScheduleDaemonOutcome) -> String {
    match outcome {
        ScheduleDaemonOutcome::Started {
            schedule_id,
            workflow_name,
            summary,
            ..
        } => format!(
            "started schedule {schedule_id} workflow {workflow_name}: {} nodes, {} failures",
            summary.node_count, summary.failures
        ),
        ScheduleDaemonOutcome::Error {
            schedule_id,
            workflow_name,
            message,
            ..
        } => format!("schedule {schedule_id} workflow {workflow_name} failed: {message}"),
        ScheduleDaemonOutcome::Skipped(skip) => {
            format!("skipped schedule {} workflow {}: {:?}", skip.schedule_id, skip.workflow_name, skip.reason)
        }
    }
}
