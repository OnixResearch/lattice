/// Automation outcome and cleanup event kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutomationEventKind {
    /// Automation emitted a bounded finding.
    Finding,
    /// Automation completed with no changes.
    NoOp,
    /// Automation failed before a successful result.
    FailedRun,
    /// Isolated workspace cleanup occurred.
    WorkspaceCleanup,
    /// Restore evidence is available after cleanup or failure.
    RestoreAvailable,
}

impl AutomationEventKind {
    /// Parses an automation event kind.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(EVENT_KIND_FIELD, value, |normalized| match normalized {
            EVENT_FINDING => Some(Self::Finding),
            EVENT_NO_OP => Some(Self::NoOp),
            EVENT_FAILED_RUN => Some(Self::FailedRun),
            EVENT_WORKSPACE_CLEANUP => Some(Self::WorkspaceCleanup),
            EVENT_RESTORE_AVAILABLE => Some(Self::RestoreAvailable),
            _ => None,
        })
    }

    /// Returns the stable event-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Finding => EVENT_FINDING,
            Self::NoOp => EVENT_NO_OP,
            Self::FailedRun => EVENT_FAILED_RUN,
            Self::WorkspaceCleanup => EVENT_WORKSPACE_CLEANUP,
            Self::RestoreAvailable => EVENT_RESTORE_AVAILABLE,
        }
    }
}

/// Safe automation event with bounded manifest and restore evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationEvent {
    kind: AutomationEventKind,
    automation_id: alloc::string::String,
    workspace_id: alloc::string::String,
    manifest_hash: alloc::string::String,
    restore_snapshot_id: Option<alloc::string::String>,
}

impl AutomationEvent {
    /// Validates a safe automation event.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when fields are empty, unsupported,
    /// or cleanup/restore events omit restore evidence.
    pub fn new(draft: AutomationEventDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let kind = AutomationEventKind::parse(draft.kind)?;
        let automation_id =
            crate::validation::normalized_adapter_field(AUTOMATION_ID_FIELD, draft.automation_id)?;
        let workspace_id = crate::validation::normalized_adapter_field(
            AUTOMATION_WORKSPACE_FIELD,
            draft.workspace_id,
        )?;
        let manifest_hash = crate::validation::normalized_adapter_field(
            EVENT_MANIFEST_HASH_FIELD,
            draft.manifest_hash,
        )?;
        let restore_snapshot_id =
            super::text::optional_field(EVENT_RESTORE_SNAPSHOT_FIELD, draft.restore_snapshot_id)?;
        reject_missing_restore_evidence(kind, restore_snapshot_id.as_deref())?;
        Ok(Self {
            kind,
            automation_id,
            workspace_id,
            manifest_hash,
            restore_snapshot_id,
        })
    }

    /// Returns the event kind.
    #[must_use]
    pub const fn kind(&self) -> AutomationEventKind {
        self.kind
    }

    /// Returns the automation identity.
    #[must_use]
    pub fn automation_id(&self) -> &str {
        self.automation_id.as_str()
    }

    /// Returns the workspace identity.
    #[must_use]
    pub fn workspace_id(&self) -> &str {
        self.workspace_id.as_str()
    }

    /// Returns the safe manifest hash.
    #[must_use]
    pub fn manifest_hash(&self) -> &str {
        self.manifest_hash.as_str()
    }

    /// Returns the restore snapshot identity, if present.
    #[must_use]
    pub fn restore_snapshot_id(&self) -> Option<&str> {
        self.restore_snapshot_id.as_deref()
    }
}

/// Borrowed automation event input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutomationEventDraft<'a> {
    /// Event kind label.
    pub kind: &'a str,
    /// Stable automation identity.
    pub automation_id: &'a str,
    /// Workspace identity.
    pub workspace_id: &'a str,
    /// Safe manifest hash or manifest identity.
    pub manifest_hash: &'a str,
    /// Optional restore snapshot identity.
    pub restore_snapshot_id: Option<&'a str>,
}

fn validate_cadence(
    kind: AutomationKind,
    cadence: &str,
) -> Result<alloc::string::String, crate::AdapterContractError> {
    let normalized =
        crate::validation::normalized_adapter_field(AUTOMATION_CADENCE_FIELD, cadence)?;
    if cadence_allowed(kind, normalized.as_str()) {
        return Ok(normalized);
    }
    Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
        field: AUTOMATION_CADENCE_FIELD.as_str(),
        value: normalized,
    })
}

fn cadence_allowed(kind: AutomationKind, cadence: &str) -> bool {
    match kind {
        AutomationKind::ScheduledStandalone => {
            matches!(cadence, CADENCE_MANUAL | CADENCE_HOURLY | CADENCE_DAILY)
                || cadence
                    .strip_prefix(CADENCE_CRON_PREFIX)
                    .is_some_and(|cron_body| !cron_body.trim().is_empty())
        }
        AutomationKind::ThreadWakeup => cadence == CADENCE_THREAD_WAKEUP,
    }
}

fn reject_unadmitted_full_access(
    sandbox_profile: AutomationSandboxProfile,
    policy_profile: Option<&str>,
) -> Result<(), crate::AdapterContractError> {
    if !matches!(sandbox_profile, AutomationSandboxProfile::FullAccess) {
        return Ok(());
    }
    if policy_profile.is_some_and(|policy| !policy.trim().is_empty()) {
        return Ok(());
    }
    Err(crate::AdapterContractError::UnsupportedWorkspaceValue {
        field: AUTOMATION_SANDBOX_FIELD.as_str(),
        value: alloc::string::String::from(SANDBOX_FULL_ACCESS),
    })
}

fn reject_missing_restore_evidence(
    kind: AutomationEventKind,
    restore_snapshot_id: Option<&str>,
) -> Result<(), crate::AdapterContractError> {
    if !matches!(
        kind,
        AutomationEventKind::WorkspaceCleanup | AutomationEventKind::RestoreAvailable
    ) {
        return Ok(());
    }
    if restore_snapshot_id.is_some() {
        return Ok(());
    }
    Err(crate::AdapterContractError::EmptyField {
        field: EVENT_RESTORE_SNAPSHOT_FIELD.as_str(),
    })
}
