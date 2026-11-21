//! What one worker step is asked, reports and is charged.

// r[impl lattice.implementation_attempt.lineage.scenario.usage]

/// The usage phase a charge is attributed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Preparation,
    Edit,
    Check,
    Repair,
}

/// What the worker port observed of one step. The exit status never
/// decides a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum WorkerOutcome {
    Exited {
        code: i32,
    },
    Cancelled,
    /// The step's effects are uncertain, for example after unknown teardown.
    Unknown,
}

/// One worker step as the worker port reports it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub outcome: WorkerOutcome,
    /// Worker-reported model requests; `None` is unknown, never zero.
    pub model_requests: Option<u64>,
    /// Worker-reported tool invocations; `None` is unknown, never zero.
    pub tool_invocations: Option<u64>,
    /// A continuation the worker proposes; only Kiln may admit it.
    pub proposal: Option<super::KilnOwned>,
}

/// What the shell asks the worker port to do next.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub binding: super::StepBinding,
    pub iteration: u32,
    pub phase: Phase,
    /// The candidate revision this step repairs; `None` for the first step.
    pub parent: Option<String>,
    /// The admitted failure classes the step repairs. No transcript, check
    /// output or verifier-private material is carried.
    pub failures: Vec<String>,
}

/// One attributed charge. Reported counts are kept as reported; the
/// charge is the conservative amount drawn from every budget.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub phase: Phase,
    pub iteration: u32,
    pub model_requests: Option<u64>,
    pub tool_invocations: Option<u64>,
    pub provenance: crate::orchestration::UsageProvenance,
    pub charged: super::super::lineage::Draws,
}
