// Public root reexports intentionally keep the `Worker*` prefix for embedders.
#![allow(unknown_lints)]
#![allow(path_segment_repetition)]

/// Draft scoped worker task supplied before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerTaskDraft {
    /// Stable safe task identity for this delegation.
    pub task_id: String,
    /// Human-reviewable objective for the worker.
    pub objective: String,
    /// Explicit scope for allowed work.
    pub scope: Option<super::WorkerScope>,
    /// Minimized packet of parent context selected for the worker.
    pub context_packet: Option<super::WorkerContextPacket>,
    /// Capabilities available to the worker.
    pub allowed_capabilities: Vec<super::WorkerCapability>,
    /// Turn, tool-call, and context budgets.
    pub budgets: Option<super::WorkerBudgets>,
    /// Expected worker output schema.
    pub output_schema: Option<super::WorkerOutputSchema>,
    /// Parent trace metadata used to tie reports back to the lead run.
    pub parent_trace: Option<super::ParentTrace>,
    /// Authority metadata that keeps parent policy in force.
    pub authority: Option<super::WorkerAuthority>,
}

/// Validated scoped worker task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerTask {
    pub(crate) task_id: String,
    pub(crate) objective: String,
    pub(crate) scope: super::WorkerScope,
    pub(crate) context_packet: super::WorkerContextPacket,
    pub(crate) allowed_capabilities: Vec<super::WorkerCapability>,
    pub(crate) budgets: super::WorkerBudgets,
    pub(crate) output_schema: super::WorkerOutputSchema,
    pub(crate) parent_trace: super::ParentTrace,
    pub(crate) authority: super::WorkerAuthority,
}

impl WorkerTask {
    /// Returns the stable worker task identity.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns the worker objective.
    #[must_use]
    pub fn objective(&self) -> &str {
        self.objective.as_str()
    }

    /// Returns the bounded worker scope.
    #[must_use]
    pub const fn scope(&self) -> &super::WorkerScope {
        &self.scope
    }

    /// Returns the minimized worker context packet.
    #[must_use]
    pub const fn context_packet(&self) -> &super::WorkerContextPacket {
        &self.context_packet
    }

    /// Returns allowed worker capabilities.
    #[must_use]
    pub fn allowed_capabilities(&self) -> &[super::WorkerCapability] {
        self.allowed_capabilities.as_slice()
    }

    /// Returns worker execution budgets.
    #[must_use]
    pub const fn budgets(&self) -> &super::WorkerBudgets {
        &self.budgets
    }

    /// Returns the expected worker output schema.
    #[must_use]
    pub const fn output_schema(&self) -> &super::WorkerOutputSchema {
        &self.output_schema
    }

    /// Returns parent trace metadata.
    #[must_use]
    pub const fn parent_trace(&self) -> &super::ParentTrace {
        &self.parent_trace
    }

    /// Returns authority metadata.
    #[must_use]
    pub const fn authority(&self) -> &super::WorkerAuthority {
        &self.authority
    }
}
