//! Pure guarded-round policy and deterministic observations.

pub const DEFAULT_MINIMUM_DELTA: u64 = 1;
pub const MAX_ACTIVE_CHANGES: usize = 64;
pub const MAX_CHANGE_ID_BYTES: usize = 128;
pub const MAX_METRIC_DIGITS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    Decrease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProgressPolicy {
    pub direction: MetricDirection,
    pub minimum_delta: u64,
}

impl ProgressPolicy {
    pub fn decreasing(minimum_delta: u64) -> Result<Self, BlockReason> {
        if minimum_delta == 0 {
            return Err(BlockReason::InvalidPolicy);
        }
        Ok(Self {
            direction: MetricDirection::Decrease,
            minimum_delta,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationPhase {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MetricFact {
    pub phase: ObservationPhase,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProgressPrerequisites {
    pub validation_passed: bool,
    pub restoration_required: bool,
    pub restoration_confirmed: bool,
    pub selected_change_removal_required: bool,
    pub selected_change_removed: bool,
}

impl ProgressPrerequisites {
    pub const fn standard() -> Self {
        Self {
            validation_passed: true,
            restoration_required: false,
            restoration_confirmed: false,
            selected_change_removal_required: false,
            selected_change_removed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProgressObservation {
    pub metrics: Vec<MetricFact>,
    pub prerequisites: ProgressPrerequisites,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockReason {
    InvalidPolicy,
    MissingBeforeMetric,
    MissingAfterMetric,
    DuplicateBeforeMetric,
    DuplicateAfterMetric,
    MalformedMetric,
    MetricRegression,
    InsufficientDelta,
    ValidationFailed,
    RestorationUnconfirmed,
    SelectedChangeRetained,
    MalformedSnapshot,
    SelectionMissing,
    SelectionChanged,
}

impl BlockReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPolicy => "invalid_policy",
            Self::MissingBeforeMetric => "missing_before_metric",
            Self::MissingAfterMetric => "missing_after_metric",
            Self::DuplicateBeforeMetric => "duplicate_before_metric",
            Self::DuplicateAfterMetric => "duplicate_after_metric",
            Self::MalformedMetric => "malformed_metric",
            Self::MetricRegression => "metric_regression",
            Self::InsufficientDelta => "insufficient_delta",
            Self::ValidationFailed => "validation_failed",
            Self::RestorationUnconfirmed => "restoration_unconfirmed",
            Self::SelectedChangeRetained => "selected_change_retained",
            Self::MalformedSnapshot => "malformed_snapshot",
            Self::SelectionMissing => "selection_missing",
            Self::SelectionChanged => "selection_changed",
        }
    }
}

// r[impl lattice.guarded_agent_templates.evidence]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SuccessorProposal {
    pub before: u64,
    pub after: u64,
    pub delta: u64,
    pub requires_operator_approval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ProgressDecision {
    Improved { successor: SuccessorProposal },
    Complete { before: u64, delta: u64 },
    Blocked { reason: BlockReason },
}

// r[impl lattice.guarded_agent_templates.progress]
pub fn decide_progress(policy: ProgressPolicy, observation: &ProgressObservation) -> ProgressDecision {
    if policy.minimum_delta == 0 {
        return blocked(BlockReason::InvalidPolicy);
    }
    if !observation.prerequisites.validation_passed {
        return blocked(BlockReason::ValidationFailed);
    }
    if observation.prerequisites.restoration_required && !observation.prerequisites.restoration_confirmed {
        return blocked(BlockReason::RestorationUnconfirmed);
    }
    if observation.prerequisites.selected_change_removal_required && !observation.prerequisites.selected_change_removed
    {
        return blocked(BlockReason::SelectedChangeRetained);
    }
    debug_assert!(policy.minimum_delta > 0);
    debug_assert!(observation.prerequisites.validation_passed);
    debug_assert!(!observation.prerequisites.restoration_required || observation.prerequisites.restoration_confirmed);

    let before = match one_metric(&observation.metrics, ObservationPhase::Before) {
        Ok(value) => value,
        Err(reason) => return blocked(reason),
    };
    let after = match one_metric(&observation.metrics, ObservationPhase::After) {
        Ok(value) => value,
        Err(reason) => return blocked(reason),
    };
    let delta = match policy.direction {
        MetricDirection::Decrease => match before.checked_sub(after) {
            Some(value) if value > 0 => value,
            Some(_) => return blocked(BlockReason::InsufficientDelta),
            None => return blocked(BlockReason::MetricRegression),
        },
    };
    if delta < policy.minimum_delta {
        return blocked(BlockReason::InsufficientDelta);
    }
    if after == 0 {
        return ProgressDecision::Complete { before, delta };
    }
    ProgressDecision::Improved {
        successor: SuccessorProposal {
            before,
            after,
            delta,
            requires_operator_approval: true,
        },
    }
}

fn one_metric(metrics: &[MetricFact], phase: ObservationPhase) -> Result<u64, BlockReason> {
    let mut matching = metrics.iter().filter(|fact| fact.phase == phase);
    let first = matching.next().ok_or(match phase {
        ObservationPhase::Before => BlockReason::MissingBeforeMetric,
        ObservationPhase::After => BlockReason::MissingAfterMetric,
    })?;
    if matching.next().is_some() {
        return Err(match phase {
            ObservationPhase::Before => BlockReason::DuplicateBeforeMetric,
            ObservationPhase::After => BlockReason::DuplicateAfterMetric,
        });
    }
    parse_metric(&first.value)
}

fn parse_metric(value: &str) -> Result<u64, BlockReason> {
    if value.is_empty() || value.len() > MAX_METRIC_DIGITS || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(BlockReason::MalformedMetric);
    }
    value.parse::<u64>().map_err(|_| BlockReason::MalformedMetric)
}

const fn blocked(reason: BlockReason) -> ProgressDecision {
    ProgressDecision::Blocked { reason }
}

include!("core/snapshots.rs");
