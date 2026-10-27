use alloc::{string::String, vec::Vec};

const TASK_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("queue_task_id");
const GROUP_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("queue_group");
const COMMAND_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("queue_command_label");
const ENVIRONMENT_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("queue_environment_policy");
const TIMEOUT_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("queue_timeout_policy");
const BACKEND_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("queue_backend_label");
const SAFE_REFERENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("queue_safe_reference");
const LOG_TEXT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("queue_visible_log_text");
const PARALLELISM_FIELD: &str = "queue_parallelism_limit";
const MAX_VISIBLE_LINES_FIELD: &str = "queue_max_visible_lines";
const MAX_VISIBLE_BYTES_FIELD: &str = "queue_max_visible_bytes";
const VISIBLE_LINES_FIELD: &str = "queue_visible_line_count";
const VISIBLE_BYTES_FIELD: &str = "queue_visible_byte_count";
const TOTAL_LINES_FIELD: &str = "queue_total_line_count";
const TOTAL_BYTES_FIELD: &str = "queue_total_byte_count";
const MIN_POSITIVE_COUNT: u32 = 1;

/// Errors raised while validating background queue adapter DTOs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueueContractError {
    /// A shared adapter field failed validation.
    Adapter(crate::AdapterContractError),
    /// A destructive queue action lacked an explicit target selector.
    AmbiguousTarget {
        /// Destructive operation that required the selector.
        operation: QueueOperationKind,
    },
    /// A parallelism value was supplied for the wrong operation.
    ParallelismMismatch {
        /// Operation whose parallelism field was invalid.
        operation: QueueOperationKind,
    },
    /// A count or bound that must be positive was zero.
    NonPositiveCount {
        /// Field whose count was zero.
        field: &'static str,
    },
    /// A visible log window exceeded configured model-visible bounds.
    LogWindowTooLarge {
        /// Field whose value exceeded the configured bound.
        field: &'static str,
        /// Maximum allowed value.
        max: u32,
        /// Actual supplied value.
        actual: u32,
    },
    /// Log counts were internally inconsistent.
    InconsistentLogCounts {
        /// Visible-count field that exceeded the total.
        visible_field: &'static str,
        /// Total-count field that was smaller than the visible count.
        total_field: &'static str,
    },
    /// A partial log response did not carry an explicit truncation marker.
    MissingTruncationMarker,
    /// A requested optional queue capability was not advertised by the host.
    UnsupportedCapability {
        /// Capability that was required but not advertised.
        capability: QueueCapabilityKind,
    },
    /// Send-input requests must carry only a safe input hash, never raw input.
    MissingInputHash,
}

impl From<crate::AdapterContractError> for QueueContractError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Adapter(error)
    }
}

/// Stable operation classes for a host-owned background queue service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueOperationKind {
    /// Enqueue a new background task.
    Enqueue,
    /// Read task or group status.
    Status,
    /// Wait for a task using an explicit timeout policy.
    Wait,
    /// Read a bounded log viewport.
    Logs,
    /// Send bounded input through a backend-supported channel.
    SendInput,
    /// Pause queued or running work.
    Pause,
    /// Resume paused work.
    Resume,
    /// Restart work.
    Restart,
    /// Cancel or kill work.
    Cancel,
    /// Clean completed task records.
    Clean,
    /// Change group-level parallelism.
    SetGroupParallelism,
}

impl QueueOperationKind {
    /// Returns true when the operation can alter or destroy queued work.
    #[must_use]
    pub const fn requires_explicit_target(self) -> bool {
        matches!(
            self,
            Self::Pause
                | Self::Resume
                | Self::Restart
                | Self::Cancel
                | Self::Clean
                | Self::SetGroupParallelism
        )
    }

    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enqueue => "enqueue",
            Self::Status => "status",
            Self::Wait => "wait",
            Self::Logs => "logs",
            Self::SendInput => "send_input",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Restart => "restart",
            Self::Cancel => "cancel",
            Self::Clean => "clean",
            Self::SetGroupParallelism => "set_group_parallelism",
        }
    }
}

/// Explicit selector kind for queue control operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueTargetKind {
    /// Select one task ID.
    Task,
    /// Select one named group.
    Group,
    /// Select the host-defined default group explicitly.
    DefaultGroup,
    /// Select every group explicitly.
    AllGroups,
}

/// Borrowed target selector input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueTargetDraft<'a> {
    /// Selector kind.
    pub kind: QueueTargetKind,
    /// Task ID or group name when the selector kind requires one.
    pub label: Option<&'a str>,
}

/// Validated queue target selector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueTarget {
    kind: QueueTargetKind,
    label: Option<String>,
}

impl QueueTarget {
    /// Validates an explicit queue target selector.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError`] when a task or group selector omits its
    /// required label or when the label is not adapter-safe.
    pub fn new(draft: QueueTargetDraft<'_>) -> Result<Self, QueueContractError> {
        let label = match draft.kind {
            QueueTargetKind::Task => Some(normalize_required(TASK_ID_FIELD, draft.label)?),
            QueueTargetKind::Group => Some(normalize_required(GROUP_FIELD, draft.label)?),
            QueueTargetKind::DefaultGroup | QueueTargetKind::AllGroups => None,
        };
        Ok(Self {
            kind: draft.kind,
            label,
        })
    }

    /// Returns the selector kind.
    #[must_use]
    pub const fn kind(&self) -> QueueTargetKind {
        self.kind
    }

    /// Returns the optional selector label.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

/// Borrowed queue control request input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueControlDraft<'a> {
    /// Operation requested by the caller.
    pub operation: QueueOperationKind,
    /// Explicit target selector, required for destructive controls.
    pub target: Option<QueueTargetDraft<'a>>,
    /// Positive group parallelism for `SetGroupParallelism` only.
    pub parallelism: Option<u32>,
}

/// Validated queue control request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueControlRequest {
    operation: QueueOperationKind,
    target: Option<QueueTarget>,
    parallelism: Option<u32>,
}

impl QueueControlRequest {
    /// Validates a queue control request.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError::AmbiguousTarget`] when a destructive
    /// operation omits an explicit target selector.
    pub fn new(draft: QueueControlDraft<'_>) -> Result<Self, QueueContractError> {
        if draft.operation.requires_explicit_target() && draft.target.is_none() {
            return Err(QueueContractError::AmbiguousTarget {
                operation: draft.operation,
            });
        }
        let parallelism = match (draft.operation, draft.parallelism) {
            (QueueOperationKind::SetGroupParallelism, Some(value))
                if value >= MIN_POSITIVE_COUNT =>
            {
                Some(value)
            }
            (QueueOperationKind::SetGroupParallelism, _) => {
                return Err(QueueContractError::NonPositiveCount {
                    field: PARALLELISM_FIELD,
                });
            }
            (_, None) => None,
            (operation, Some(_)) => {
                return Err(QueueContractError::ParallelismMismatch { operation });
            }
        };
        let target = draft.target.map(QueueTarget::new).transpose()?;
        if draft.operation == QueueOperationKind::SetGroupParallelism
            && !target.as_ref().is_some_and(|target| {
                matches!(
                    target.kind(),
                    QueueTargetKind::Group | QueueTargetKind::DefaultGroup
                )
            })
        {
            return Err(QueueContractError::ParallelismMismatch {
                operation: draft.operation,
            });
        }
        Ok(Self {
            operation: draft.operation,
            target,
            parallelism,
        })
    }

    /// Returns the operation kind.
    #[must_use]
    pub const fn operation(&self) -> QueueOperationKind {
        self.operation
    }

    /// Returns the explicit target selector, if present.
    #[must_use]
    pub const fn target(&self) -> Option<&QueueTarget> {
        self.target.as_ref()
    }

    /// Returns the positive group parallelism, when requested.
    #[must_use]
    pub const fn parallelism(&self) -> Option<u32> {
        self.parallelism
    }
}

/// Model-visible log bounds chosen by the caller or host policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueLogBounds {
    max_visible_lines: u32,
    max_visible_bytes: u32,
}

impl QueueLogBounds {
    /// Builds non-zero log viewport bounds.
    ///
    /// # Errors
    ///
    /// Returns [`QueueContractError::NonPositiveCount`] when either bound is zero.
    pub const fn new(
        max_visible_lines: u32,
        max_visible_bytes: u32,
    ) -> Result<Self, QueueContractError> {
        if max_visible_lines < MIN_POSITIVE_COUNT {
            return Err(QueueContractError::NonPositiveCount {
                field: MAX_VISIBLE_LINES_FIELD,
            });
        }
        if max_visible_bytes < MIN_POSITIVE_COUNT {
            return Err(QueueContractError::NonPositiveCount {
                field: MAX_VISIBLE_BYTES_FIELD,
            });
        }
        Ok(Self {
            max_visible_lines,
            max_visible_bytes,
        })
    }

    /// Returns the maximum visible log lines.
    #[must_use]
    pub const fn max_visible_lines(self) -> u32 {
        self.max_visible_lines
    }

    /// Returns the maximum visible log bytes.
    #[must_use]
    pub const fn max_visible_bytes(self) -> u32 {
        self.max_visible_bytes
    }
}
