const AUTOMATION_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_id");
const AUTOMATION_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_kind");
const AUTOMATION_PROMPT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_prompt");
const AUTOMATION_CADENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_cadence");
const AUTOMATION_WORKSPACE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_workspace_id");
const AUTOMATION_SANDBOX_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_sandbox_profile");
const AUTOMATION_STOP_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_stop_criterion");
const AUTOMATION_STOPS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_stop_criteria");
const AUTOMATION_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_policy_profile");
const EVENT_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_event_kind");
const EVENT_MANIFEST_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_event_manifest_hash");
const EVENT_RESTORE_SNAPSHOT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("automation_event_restore_snapshot_id");

const KIND_SCHEDULED_STANDALONE: &str = "scheduled_standalone";
const KIND_THREAD_WAKEUP: &str = "thread_wakeup";
const SANDBOX_READ_ONLY: &str = "read_only";
const SANDBOX_WORKSPACE_WRITE: &str = "workspace_write";
const SANDBOX_FULL_ACCESS: &str = "full_access";
const CADENCE_MANUAL: &str = "manual";
const CADENCE_HOURLY: &str = "hourly";
const CADENCE_DAILY: &str = "daily";
const CADENCE_THREAD_WAKEUP: &str = "thread_wakeup";
const CADENCE_CRON_PREFIX: &str = "cron:";
const EVENT_FINDING: &str = "finding";
const EVENT_NO_OP: &str = "no_op";
const EVENT_FAILED_RUN: &str = "failed_run";
const EVENT_WORKSPACE_CLEANUP: &str = "workspace_cleanup";
const EVENT_RESTORE_AVAILABLE: &str = "restore_available";

/// Automation run class for isolated workspaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomationKind {
    /// Standalone run selected by an external scheduler.
    ScheduledStandalone,
    /// Wakeup bound to an existing runtime thread.
    ThreadWakeup,
}

impl AutomationKind {
    /// Parses an automation kind label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(
            AUTOMATION_KIND_FIELD,
            value,
            |normalized| match normalized {
                KIND_SCHEDULED_STANDALONE => Some(Self::ScheduledStandalone),
                KIND_THREAD_WAKEUP => Some(Self::ThreadWakeup),
                _ => None,
            },
        )
    }

    /// Returns the stable automation-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ScheduledStandalone => KIND_SCHEDULED_STANDALONE,
            Self::ThreadWakeup => KIND_THREAD_WAKEUP,
        }
    }
}

/// Sandbox profile requested by an automation run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomationSandboxProfile {
    /// Read-only execution profile.
    ReadOnly,
    /// Workspace-scoped write profile.
    WorkspaceWrite,
    /// Full-access profile requiring an explicit policy profile.
    FullAccess,
}

impl AutomationSandboxProfile {
    /// Parses an automation sandbox profile.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(
            AUTOMATION_SANDBOX_FIELD,
            value,
            |normalized| match normalized {
                SANDBOX_READ_ONLY => Some(Self::ReadOnly),
                SANDBOX_WORKSPACE_WRITE => Some(Self::WorkspaceWrite),
                SANDBOX_FULL_ACCESS => Some(Self::FullAccess),
                _ => None,
            },
        )
    }

    /// Returns the stable sandbox-profile label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => SANDBOX_READ_ONLY,
            Self::WorkspaceWrite => SANDBOX_WORKSPACE_WRITE,
            Self::FullAccess => SANDBOX_FULL_ACCESS,
        }
    }
}

/// Validated automation run contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationRunContract {
    automation_id: alloc::string::String,
    kind: AutomationKind,
    prompt: alloc::string::String,
    cadence: alloc::string::String,
    workspace_id: alloc::string::String,
    sandbox_profile: AutomationSandboxProfile,
    stop_criteria: alloc::vec::Vec<alloc::string::String>,
    policy_profile: Option<alloc::string::String>,
}

impl AutomationRunContract {
    /// Validates an automation run contract before scheduler registration.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when fields are empty, unsupported,
    /// or an unattended full-access sandbox lacks an explicit policy profile.
    pub fn new(draft: AutomationRunContractDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let automation_id =
            crate::validation::normalized_adapter_field(AUTOMATION_ID_FIELD, draft.automation_id)?;
        let kind = AutomationKind::parse(draft.kind)?;
        let prompt =
            crate::validation::normalized_adapter_field(AUTOMATION_PROMPT_FIELD, draft.prompt)?;
        let cadence = validate_cadence(kind, draft.cadence)?;
        let workspace_id = crate::validation::normalized_adapter_field(
            AUTOMATION_WORKSPACE_FIELD,
            draft.workspace_id,
        )?;
        let sandbox_profile = AutomationSandboxProfile::parse(draft.sandbox_profile)?;
        reject_unadmitted_full_access(sandbox_profile, draft.policy_profile)?;
        let stop_criteria = super::text::required_texts(
            AUTOMATION_STOPS_FIELD,
            AUTOMATION_STOP_FIELD,
            draft.stop_criteria,
        )?;
        let policy_profile =
            super::text::optional_field(AUTOMATION_POLICY_FIELD, draft.policy_profile)?;
        Ok(Self {
            automation_id,
            kind,
            prompt,
            cadence,
            workspace_id,
            sandbox_profile,
            stop_criteria,
            policy_profile,
        })
    }

    /// Returns the automation identity.
    #[must_use]
    pub fn automation_id(&self) -> &str {
        self.automation_id.as_str()
    }

    /// Returns the automation kind.
    #[must_use]
    pub const fn kind(&self) -> AutomationKind {
        self.kind
    }

    /// Returns bounded prompt text selected for the run.
    #[must_use]
    pub fn prompt(&self) -> &str {
        self.prompt.as_str()
    }

    /// Returns the normalized cadence label.
    #[must_use]
    pub fn cadence(&self) -> &str {
        self.cadence.as_str()
    }

    /// Returns the target workspace identity.
    #[must_use]
    pub fn workspace_id(&self) -> &str {
        self.workspace_id.as_str()
    }

    /// Returns the sandbox profile.
    #[must_use]
    pub const fn sandbox_profile(&self) -> AutomationSandboxProfile {
        self.sandbox_profile
    }

    /// Returns bounded stop criteria.
    #[must_use]
    pub fn stop_criteria(&self) -> &[alloc::string::String] {
        self.stop_criteria.as_slice()
    }

    /// Returns the policy profile admitted for risky automation, if any.
    #[must_use]
    pub fn policy_profile(&self) -> Option<&str> {
        self.policy_profile.as_deref()
    }
}

/// Borrowed automation run contract input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutomationRunContractDraft<'a> {
    /// Stable automation identity.
    pub automation_id: &'a str,
    /// Automation kind label.
    pub kind: &'a str,
    /// Bounded prompt text or prompt template label.
    pub prompt: &'a str,
    /// Cadence label such as `daily`, `hourly`, `manual`, `thread_wakeup`, or `cron:<expr>`.
    pub cadence: &'a str,
    /// Target isolated workspace identity.
    pub workspace_id: &'a str,
    /// Requested sandbox profile label.
    pub sandbox_profile: &'a str,
    /// Bounded stop criteria labels.
    pub stop_criteria: &'a [&'a str],
    /// Optional admitted policy profile for high-risk profiles.
    pub policy_profile: Option<&'a str>,
}

