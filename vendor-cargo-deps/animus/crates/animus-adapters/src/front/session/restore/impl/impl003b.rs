
/// Borrowed restore target before planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreTargetDraft<'a> {
    /// Safe target label.
    pub target_label: &'a str,
    /// Requested restore mode.
    pub requested_mode: RestoreMode,
    /// Optional native session reference.
    pub native_reference: Option<&'a NativeSessionReference>,
    /// Optional history replay metadata.
    pub history_replay: Option<&'a HistoryReplay>,
}

/// Borrowed restore planning input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestorePlanningInput<'a> {
    /// Host-declared capabilities.
    pub host: RestoreHostCapabilities,
    /// Restore policy.
    pub policy: RestorePolicy,
    /// Restore targets.
    pub targets: &'a [RestoreTargetDraft<'a>],
}

/// Ambient effect ledger for pure planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreAmbientEffectLedger {
    bits: u8,
}

impl RestoreAmbientEffectLedger {
    const fn pure() -> Self {
        Self { bits: EMPTY_BITS }
    }

    /// Returns true when planning spawned a process.
    #[must_use]
    pub const fn spawned_process(self) -> bool {
        has_bit(self.bits, EFFECT_SPAWNED_PROCESS)
    }

    /// Returns true when planning read files.
    #[must_use]
    pub const fn read_files(self) -> bool {
        has_bit(self.bits, EFFECT_READ_FILES)
    }

    /// Returns true when planning opened a PTY.
    #[must_use]
    pub const fn opened_pty(self) -> bool {
        has_bit(self.bits, EFFECT_OPENED_PTY)
    }

    /// Returns true when planning constructed shell commands.
    #[must_use]
    pub const fn constructed_shell_command(self) -> bool {
        has_bit(self.bits, EFFECT_CONSTRUCTED_SHELL_COMMAND)
    }

    /// Returns true when planning inspected environment state.
    #[must_use]
    pub const fn inspected_environment(self) -> bool {
        has_bit(self.bits, EFFECT_INSPECTED_ENVIRONMENT)
    }
}

/// Planned target restore outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreTargetPlan {
    target_label: String,
    requested_mode: RestoreMode,
    status: RestoreOutcomeStatus,
    degraded_reason: Option<RestoreDegradedReason>,
    claims: RestoreCapabilityClaims,
    native_reference_hash: Option<crate::artifact::Hash>,
}

impl RestoreTargetPlan {
    /// Returns target label.
    #[must_use]
    pub fn target_label(&self) -> &str {
        self.target_label.as_str()
    }

    /// Returns requested mode.
    #[must_use]
    pub const fn requested_mode(&self) -> RestoreMode {
        self.requested_mode
    }

    /// Returns outcome status.
    #[must_use]
    pub const fn status(&self) -> RestoreOutcomeStatus {
        self.status
    }

    /// Returns degraded reason.
    #[must_use]
    pub const fn degraded_reason(&self) -> Option<RestoreDegradedReason> {
        self.degraded_reason
    }

    /// Returns capability claims.
    #[must_use]
    pub const fn claims(&self) -> RestoreCapabilityClaims {
        self.claims
    }

    /// Returns native reference hash when present.
    #[must_use]
    pub const fn native_reference_hash(&self) -> Option<&crate::artifact::Hash> {
        self.native_reference_hash.as_ref()
    }
}

/// Pure restore plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestorePlan {
    targets: Vec<RestoreTargetPlan>,
    effects: RestoreAmbientEffectLedger,
}

impl RestorePlan {
    /// Returns target plans.
    #[must_use]
    pub fn targets(&self) -> &[RestoreTargetPlan] {
        self.targets.as_slice()
    }

    /// Returns ambient effect ledger.
    #[must_use]
    pub const fn effects(&self) -> RestoreAmbientEffectLedger {
        self.effects
    }
}

/// Plans restore outcomes from typed in-memory snapshots and host capabilities.
///
/// # Errors
///
/// Returns [`SessionRestoreError`] when target bounds or labels are invalid.
// r[impl agent_harness_runtime.session_restore.restore_matrix]
// r[impl agent_harness_runtime.session_restore.host_owned_execution]
// r[impl agent_harness_runtime.session_restore.degraded_outcomes]
pub fn plan_restore(input: RestorePlanningInput<'_>) -> Result<RestorePlan, SessionRestoreError> {
    validate_item_count(input.targets.len())?;
    let mut plans = Vec::with_capacity(input.targets.len());
    let mut seen_native_hashes: Vec<crate::artifact::Hash> = Vec::new();
    let mut context = RestorePlannerContext {
        host: input.host,
        policy: input.policy,
        seen_native_hashes: &mut seen_native_hashes,
    };
    for target in input.targets {
        let label = normalize(RESTORE_TARGET_FIELD, target.target_label)?;
        let plan = plan_target(label, target, &mut context);
        plans.push(plan);
    }
    Ok(RestorePlan {
        targets: plans,
        effects: RestoreAmbientEffectLedger::pure(),
    })
}

struct RestorePlannerContext<'a> {
    host: RestoreHostCapabilities,
    policy: RestorePolicy,
    seen_native_hashes: &'a mut Vec<crate::artifact::Hash>,
}
