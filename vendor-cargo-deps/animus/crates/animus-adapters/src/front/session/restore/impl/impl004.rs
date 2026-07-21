
fn plan_target(
    label: String,
    target: &RestoreTargetDraft<'_>,
    context: &mut RestorePlannerContext<'_>,
) -> RestoreTargetPlan {
    match target.requested_mode {
        RestoreMode::LiveProcess => capability_plan(
            label,
            target.requested_mode,
            context.host.supports(RestoreMode::LiveProcess),
            live_claims(),
        ),
        RestoreMode::SnapshotLayout => capability_plan(
            label,
            target.requested_mode,
            context.host.supports(RestoreMode::SnapshotLayout),
            layout_claims(),
        ),
        RestoreMode::HistoryReplay => history_plan(label, target, context.host, context.policy),
        RestoreMode::NativeSessionResume => {
            native_plan(label, target, context.host, context.seen_native_hashes)
        }
        RestoreMode::HandoffTransfer => handoff_plan(
            label,
            target.requested_mode,
            context.host.supports(RestoreMode::HandoffTransfer),
        ),
        RestoreMode::Fallback => degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::SafeFallback,
            None,
        ),
    }
}

fn capability_plan(
    label: String,
    mode: RestoreMode,
    supported: bool,
    claims: RestoreCapabilityClaims,
) -> RestoreTargetPlan {
    if supported {
        return restored_plan(label, mode, claims, None);
    }
    degraded_plan(
        label,
        mode,
        RestoreDegradedReason::UnsupportedCapability,
        None,
    )
}

fn live_claims() -> RestoreCapabilityClaims {
    RestoreCapabilityClaims::from_bits(CLAIM_PROCESS_SURVIVED)
}

fn layout_claims() -> RestoreCapabilityClaims {
    RestoreCapabilityClaims::from_bits(CLAIM_LAYOUT_RECONSTRUCTED)
}

fn history_claims(model_context_admitted: bool) -> RestoreCapabilityClaims {
    let model_context_bit = if model_context_admitted {
        CLAIM_MODEL_CONTEXT_ADMITTED
    } else {
        EMPTY_BITS
    };
    RestoreCapabilityClaims::from_bits(CLAIM_DISPLAY_REPLAYED | model_context_bit)
}

fn history_plan(
    label: String,
    target: &RestoreTargetDraft<'_>,
    host: RestoreHostCapabilities,
    policy: RestorePolicy,
) -> RestoreTargetPlan {
    if !policy.history_replay_enabled {
        return degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::HistoryReplayDisabled,
            None,
        );
    }
    let Some(history) = target.history_replay else {
        return degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::UnsafeHistoryReplay,
            None,
        );
    };
    if !host.supports(RestoreMode::HistoryReplay) || !history.enabled() {
        return degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::UnsupportedCapability,
            None,
        );
    }
    restored_plan(
        label,
        target.requested_mode,
        history_claims(history.model_visible() && policy.admit_replay_as_context),
        None,
    )
}

fn native_plan(
    label: String,
    target: &RestoreTargetDraft<'_>,
    host: RestoreHostCapabilities,
    seen_native_hashes: &mut Vec<crate::artifact::Hash>,
) -> RestoreTargetPlan {
    if !host.supports(RestoreMode::NativeSessionResume) {
        return degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::UnsupportedCapability,
            None,
        );
    }
    let Some(reference) = target.native_reference else {
        return degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::MissingNativeReference,
            None,
        );
    };
    if reference.freshness() == NativeSessionFreshness::Stale {
        return degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::StaleNativeReference,
            Some(reference.value_hash().clone()),
        );
    }
    if reference.resume_capability() == NativeResumeCapability::Unsupported {
        return degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::UnsupportedNativeReference,
            Some(reference.value_hash().clone()),
        );
    }
    if seen_native_hashes
        .iter()
        .any(|candidate| candidate == reference.value_hash())
    {
        return degraded_plan(
            label,
            target.requested_mode,
            RestoreDegradedReason::DuplicatedNativeReference,
            Some(reference.value_hash().clone()),
        );
    }
    seen_native_hashes.push(reference.value_hash().clone());
    restored_plan(
        label,
        target.requested_mode,
        RestoreCapabilityClaims::from_bits(CLAIM_NATIVE_RESUME_REQUESTED),
        Some(reference.value_hash().clone()),
    )
}

fn handoff_plan(label: String, mode: RestoreMode, supported: bool) -> RestoreTargetPlan {
    if supported {
        return restored_plan(
            label,
            mode,
            RestoreCapabilityClaims::from_bits(CLAIM_HANDOFF_REQUESTED),
            None,
        );
    }
    degraded_plan(label, mode, RestoreDegradedReason::FailedHandoffClaim, None)
}

fn restored_plan(
    label: String,
    mode: RestoreMode,
    claims: RestoreCapabilityClaims,
    native_reference_hash: Option<crate::artifact::Hash>,
) -> RestoreTargetPlan {
    RestoreTargetPlan {
        target_label: label,
        requested_mode: mode,
        status: RestoreOutcomeStatus::Restored,
        degraded_reason: None,
        claims,
        native_reference_hash,
    }
}

fn degraded_plan(
    label: String,
    mode: RestoreMode,
    reason: RestoreDegradedReason,
    native_reference_hash: Option<crate::artifact::Hash>,
) -> RestoreTargetPlan {
    RestoreTargetPlan {
        target_label: label,
        requested_mode: mode,
        status: RestoreOutcomeStatus::Degraded,
        degraded_reason: Some(reason),
        claims: RestoreCapabilityClaims::fallback(),
        native_reference_hash,
    }
}

/// Borrowed restore receipt fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreReceiptDraft<'a> {
    /// Restore plan.
    pub plan: &'a RestorePlan,
    /// Raw replayed history, forbidden.
    pub raw_history: Option<&'a str>,
    /// Credential-looking value, forbidden.
    pub credential: Option<&'a str>,
    /// Shell command, forbidden.
    pub shell_command: Option<&'a str>,
    /// Full path, forbidden.
    pub full_path: Option<&'a str>,
    /// Transcript body, forbidden.
    pub transcript: Option<&'a str>,
}

/// Safe restore receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreReceipt {
    plan: RestorePlan,
    receipt_hash: crate::artifact::Hash,
}

