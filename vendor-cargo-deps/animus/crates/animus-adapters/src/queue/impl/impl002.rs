/// Borrowed background queue enqueue request input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueEnqueueDraft<'a> {
    /// Safe command identity label, not raw shell text.
    pub command_label: &'a str,
    /// Optional queue group label.
    pub group: Option<&'a str>,
    /// Safe environment policy label.
    pub environment_policy: &'a str,
    /// Safe timeout policy label.
    pub timeout_policy: &'a str,
    /// Model-visible output bounds.
    pub output_bounds: QueueLogBounds,
}

/// Validated background queue enqueue request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueEnqueueRequest {
    command_label: String,
    group: Option<String>,
    environment_policy: String,
    timeout_policy: String,
    output_bounds: QueueLogBounds,
}

impl QueueEnqueueRequest {
    /// Validates an enqueue request for an injected host queue service.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError`] when labels are empty, too long, or unsafe.
    pub fn new(draft: QueueEnqueueDraft<'_>) -> Result<Self, QueueContractError> {
        let command_label =
            crate::validation::normalized_adapter_field(COMMAND_LABEL_FIELD, draft.command_label)?;
        let group = draft
            .group
            .map(|label| crate::validation::normalized_adapter_field(GROUP_FIELD, label))
            .transpose()?;
        let environment_policy = crate::validation::normalized_adapter_field(
            ENVIRONMENT_POLICY_FIELD,
            draft.environment_policy,
        )?;
        let timeout_policy = crate::validation::normalized_adapter_field(
            TIMEOUT_POLICY_FIELD,
            draft.timeout_policy,
        )?;
        Ok(Self {
            command_label,
            group,
            environment_policy,
            timeout_policy,
            output_bounds: draft.output_bounds,
        })
    }

    /// Returns the safe command identity label.
    #[must_use]
    pub fn command_label(&self) -> &str {
        self.command_label.as_str()
    }

    /// Returns the optional group label.
    #[must_use]
    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    /// Returns the environment policy label.
    #[must_use]
    pub fn environment_policy(&self) -> &str {
        self.environment_policy.as_str()
    }

    /// Returns the timeout policy label.
    #[must_use]
    pub fn timeout_policy(&self) -> &str {
        self.timeout_policy.as_str()
    }

    /// Returns output viewport bounds.
    #[must_use]
    pub const fn output_bounds(&self) -> QueueLogBounds {
        self.output_bounds
    }
}

/// Stable task status class independent of one backend's vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueTaskStatus {
    /// Waiting for execution.
    Pending,
    /// Currently running.
    Running,
    /// Paused by host or user policy.
    Paused,
    /// Stashed and not eligible to run yet.
    Stashed,
    /// Finished successfully.
    Completed,
    /// Finished with failure.
    Failed,
    /// Cancelled before successful completion.
    Cancelled,
}

impl QueueTaskStatus {
    /// Returns the stable task status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Stashed => "stashed",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Borrowed task observation input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueTaskObservationDraft<'a> {
    /// Backend or host task ID.
    pub task_id: &'a str,
    /// Queue group label.
    pub group: &'a str,
    /// Stable normalized status.
    pub status: QueueTaskStatus,
    /// Optional backend-specific status label retained as safe metadata.
    pub backend_status_label: Option<&'a str>,
}

/// Validated task observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueTaskObservation {
    task_id: String,
    group: String,
    status: QueueTaskStatus,
    backend_status_label: Option<String>,
}

impl QueueTaskObservation {
    /// Validates a queue task observation.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError`] when labels are not adapter-safe.
    pub fn new(draft: QueueTaskObservationDraft<'_>) -> Result<Self, QueueContractError> {
        let task_id = crate::validation::normalized_adapter_field(TASK_ID_FIELD, draft.task_id)?;
        let group = crate::validation::normalized_adapter_field(GROUP_FIELD, draft.group)?;
        let backend_status_label = draft
            .backend_status_label
            .map(|label| crate::validation::normalized_adapter_field(BACKEND_LABEL_FIELD, label))
            .transpose()?;
        Ok(Self {
            task_id,
            group,
            status: draft.status,
            backend_status_label,
        })
    }

    /// Returns the task ID.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns the group label.
    #[must_use]
    pub fn group(&self) -> &str {
        self.group.as_str()
    }

    /// Returns the stable task status.
    #[must_use]
    pub const fn status(&self) -> QueueTaskStatus {
        self.status
    }

    /// Returns the optional backend status label.
    #[must_use]
    pub fn backend_status_label(&self) -> Option<&str> {
        self.backend_status_label.as_deref()
    }
}

/// Borrowed counts of tasks in stable queue states for one group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueStatusCountDraft {
    /// Pending task count.
    pub pending: u32,
    /// Running task count.
    pub running: u32,
    /// Paused task count.
    pub paused: u32,
    /// Stashed task count.
    pub stashed: u32,
    /// Completed task count.
    pub completed: u32,
}

/// Counts of tasks in stable queue states for one group.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueStatusCounts {
    pending: u32,
    running: u32,
    paused: u32,
    stashed: u32,
    completed: u32,
}

impl QueueStatusCounts {
    /// Builds stable status counts for one group.
    #[must_use]
    pub const fn new(draft: QueueStatusCountDraft) -> Self {
        Self {
            pending: draft.pending,
            running: draft.running,
            paused: draft.paused,
            stashed: draft.stashed,
            completed: draft.completed,
        }
    }

    /// Returns pending task count.
    #[must_use]
    pub const fn pending(self) -> u32 {
        self.pending
    }

    /// Returns running task count.
    #[must_use]
    pub const fn running(self) -> u32 {
        self.running
    }

    /// Returns paused task count.
    #[must_use]
    pub const fn paused(self) -> u32 {
        self.paused
    }

    /// Returns stashed task count.
    #[must_use]
    pub const fn stashed(self) -> u32 {
        self.stashed
    }

    /// Returns completed task count.
    #[must_use]
    pub const fn completed(self) -> u32 {
        self.completed
    }
}
