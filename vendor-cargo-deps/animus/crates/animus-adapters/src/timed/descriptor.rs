use super::CALENDAR_FIELD;
use super::HASH_FIELD;
use super::HashInput;
use super::ID_FIELD;
use super::MIN_RETENTION_COUNT;
use super::PARAMETER_HASH_FIELD;
use super::RECIPE_FIELD;
use super::ScheduleCadence;
use super::ScheduleConcurrencyPolicy;
use super::ScheduleError;
use super::ScheduleExecutionMode;
use super::Text;
use super::WORKSPACE_FIELD;
use super::required_text;
use super::validate_hash;
use super::validate_hashes;

/// Untrusted scheduled workflow descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduledWorkflowDraft<'a> {
    /// Schedule identity.
    pub schedule_id: &'a str,
    /// Recipe identity.
    pub recipe_id: &'a str,
    /// Recipe version hash.
    pub recipe_version_hash: &'a str,
    /// Parameter value hashes.
    pub parameter_hashes: &'a [&'a str],
    /// Cadence label.
    pub cadence: &'a str,
    /// Timezone or calendar policy label.
    pub calendar_policy: &'a str,
    /// Execution mode label.
    pub execution_mode: &'a str,
    /// Concurrency policy label.
    pub concurrency_policy: &'a str,
    /// Workspace scope label.
    pub workspace_scope: &'a str,
    /// Retry limit.
    pub retry_limit: u32,
    /// Retention count.
    pub retention_count: u32,
    /// Provenance hash.
    pub provenance_hash: &'a str,
    /// Recipe trust receipt.
    pub recipe_trust: Option<&'a crate::RecipeTrustReceipt>,
}

/// Validated scheduled workflow descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledWorkflow {
    schedule_id: Text,
    recipe_id: Text,
    cadence: ScheduleCadence,
    execution_mode: ScheduleExecutionMode,
    concurrency_policy: ScheduleConcurrencyPolicy,
    workspace_scope: Text,
    retry_limit: u32,
    retention_count: u32,
}

impl ScheduledWorkflow {
    /// Validates a scheduled workflow descriptor without starting a timer or queue task.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError`] when fields are malformed, hashes are invalid,
    /// recipe trust is missing, cadence is unsupported, or bounds are invalid.
    pub fn validate(draft: &ScheduledWorkflowDraft<'_>) -> Result<Self, ScheduleError> {
        let receipt = draft.recipe_trust.ok_or(ScheduleError::UntrustedRecipe)?;
        if receipt.status() != crate::RecipeTrustStatus::Admitted {
            return Err(ScheduleError::UntrustedRecipe);
        }
        let schedule_id = required_text(ID_FIELD, draft.schedule_id)?;
        let recipe_id = required_text(RECIPE_FIELD, draft.recipe_id)?;
        validate_hash(HashInput {
            field: HASH_FIELD.as_str(),
            value: draft.recipe_version_hash,
        })?;
        validate_hashes(PARAMETER_HASH_FIELD, draft.parameter_hashes, true)?;
        let cadence = ScheduleCadence::parse(draft.cadence)?;
        required_text(CALENDAR_FIELD, draft.calendar_policy)?;
        let execution_mode = ScheduleExecutionMode::parse(draft.execution_mode)?;
        let concurrency_policy = ScheduleConcurrencyPolicy::parse(draft.concurrency_policy)?;
        let workspace_scope = required_text(WORKSPACE_FIELD, draft.workspace_scope)?;
        if draft.retention_count < MIN_RETENTION_COUNT {
            return Err(ScheduleError::InvalidBounds);
        }
        validate_hash(HashInput {
            field: HASH_FIELD.as_str(),
            value: draft.provenance_hash,
        })?;
        Ok(Self {
            schedule_id,
            recipe_id,
            cadence,
            execution_mode,
            concurrency_policy,
            workspace_scope,
            retry_limit: draft.retry_limit,
            retention_count: draft.retention_count,
        })
    }

    /// Returns schedule identity.
    #[must_use]
    pub fn schedule_id(&self) -> &str {
        self.schedule_id.as_str()
    }

    /// Returns recipe identity.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        self.recipe_id.as_str()
    }

    /// Returns cadence.
    #[must_use]
    pub const fn cadence(&self) -> ScheduleCadence {
        self.cadence
    }

    /// Returns execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> ScheduleExecutionMode {
        self.execution_mode
    }

    /// Returns concurrency policy.
    #[must_use]
    pub const fn concurrency_policy(&self) -> ScheduleConcurrencyPolicy {
        self.concurrency_policy
    }

    /// Returns workspace scope label.
    #[must_use]
    pub fn workspace_scope(&self) -> &str {
        self.workspace_scope.as_str()
    }

    /// Returns retry limit.
    #[must_use]
    pub const fn retry_limit(&self) -> u32 {
        self.retry_limit
    }

    /// Returns retention count.
    #[must_use]
    pub const fn retention_count(&self) -> u32 {
        self.retention_count
    }
}
