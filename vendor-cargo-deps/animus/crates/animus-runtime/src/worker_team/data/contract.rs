// Public root reexports intentionally keep the `Worker*` prefix for embedders.
#![allow(unknown_lints)]
#![allow(path_segment_repetition)]

/// Bounded worker scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerScope {
    /// Stable scope label.
    pub label: String,
    /// Safe human-readable scope summary.
    pub summary: String,
    /// Safe target labels the worker may inspect or report on.
    pub allowed_targets: Vec<String>,
}

/// Worker execution budgets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkerBudgets {
    /// Worker turns allowed by host policy.
    pub turns: u16,
    /// Selected context items allowed by host policy.
    pub context_items: u16,
    /// Worker tool calls allowed by host policy.
    pub tool_calls: u16,
}

/// Expected worker output schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerOutputSchema {
    /// Stable schema name.
    pub name: String,
    /// Required output field names.
    pub required_fields: Vec<String>,
}

/// Parent trace metadata copied into worker delegation records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParentTrace {
    /// Safe parent run label.
    pub parent_run_id: String,
    /// Safe parent turn label.
    pub turn_id: String,
    /// BLAKE3 hash for the parent policy snapshot.
    pub policy_hash: String,
}

/// Authority metadata for a worker task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerAuthority {
    /// Safe authority grant label.
    pub grant_label: String,
    /// Whether normal parent policy gates still apply.
    pub parent_policy_required: bool,
}

/// Capability available to a worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkerCapability {
    /// Worker may read its selected context packet.
    ReadContext,
    /// Worker may emit findings and evidence references.
    ReportFindings,
    /// Worker may request a named host tool through normal policy.
    UseTool {
        /// Safe tool label.
        tool_label: String,
    },
    /// Worker may propose a mutation intent for later arbitration.
    RequestMutation {
        /// Safe mutation kind label.
        mutation_kind: String,
    },
    /// Invalid capability used to prove bypass requests fail closed.
    BypassParentPolicy,
}
