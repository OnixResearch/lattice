//! Pure text and JSON formatting for safe reports.

pub fn render_json(result: &super::model::Report) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(result).map(|json| format!("{json}\n"))
}

pub fn render_text(result: &super::model::Report) -> String {
    let mut output = String::new();
    output.push_str(&format!("run {} valid={}\n", result.run_id, result.valid));
    output.push_str(&format!("workflow_identity {}\n", result.workflow_identity));
    output.push_str(&format!(
        "lifecycle terminal={} started_epoch_ms={} finished_epoch_ms={} nodes={} failures={} cache_hits={}\n",
        result.lifecycle.terminal,
        result.lifecycle.started_epoch_ms,
        optional_i64(result.lifecycle.finished_epoch_ms),
        result.lifecycle.node_count,
        result.lifecycle.failures,
        result.lifecycle.cache_hits,
    ));
    output.push_str(&owner_text(&result.owner));
    output.push_str(&format!(
        "details state={} nodes={} values=omitted errors=omitted\n",
        evidence_label(result.details.state),
        result.details.nodes.len(),
    ));
    for node in &result.details.nodes {
        output.push_str(&format!(
            "node {} status={} cached={} outputs={} error_present={}\n",
            node.node_id,
            node_status_label(node.status),
            node.cached,
            node.output_count,
            node.error_present,
        ));
    }
    output.push_str(&format!(
        "run_replay state={} events={} diagnostics={} summaries=omitted\n",
        evidence_label(result.replay.state),
        result.replay.event_count,
        result.replay.diagnostic_codes.len(),
    ));
    if let Some(orchestration) = &result.orchestration {
        output.push_str(&orchestration_text(orchestration));
    }
    output.push_str(&format!("diagnostics {}\n", result.diagnostic_codes.len()));
    for code in &result.diagnostic_codes {
        output.push_str(&format!("diagnostic {}\n", diagnostic_label(*code)));
    }
    debug_assert!(!output.is_empty());
    debug_assert!(output.ends_with('\n'));
    output
}

fn owner_text(owner: &super::model::SafeOwner) -> String {
    match owner {
        super::model::SafeOwner::Unscoped => "owner unscoped\n".to_string(),
        super::model::SafeOwner::Orchestration {
            instance_id,
            dispatch_id,
            binding,
        } => format!(
            "owner orchestration instance={} dispatch={} binding={}\n",
            instance_id,
            dispatch_id,
            owner_binding_label(*binding),
        ),
    }
}

fn orchestration_text(result: &super::model::SafeOrchestrationEvidence) -> String {
    format!(
        concat!(
            "orchestration status={:?} event_chain={} dispatch={} recovery={} ",
            "cancellation_requested={} run_succeeded={}\n",
            "budget settlement={} planned={} observed={} ledger_breached={}\n"
        ),
        result.status,
        evidence_label(result.event_chain),
        dispatch_label(result.dispatch),
        recovery_label(result.recovery),
        result.cancellation_requested,
        optional_bool(result.run_succeeded),
        budget_settlement_label(result.budget.settlement),
        charge_plan_label(result.budget.planned_upper_bound),
        charge_label(result.budget.observed),
        result.budget.orchestration_ledger_breached,
    )
}

fn charge_plan_label(plan: Option<crate::orchestration::BudgetChargePlan>) -> String {
    plan.map_or_else(
        || "-".to_string(),
        |charge| {
            format!(
                "rounds:{};agent_invocations:{};tokens:{};monetary_microunits:{};duration_millis:{}",
                optional_u64(charge.rounds),
                optional_u64(charge.agent_invocations),
                optional_u64(charge.tokens),
                optional_u64(charge.monetary_microunits),
                optional_u64(charge.duration_millis),
            )
        },
    )
}

fn charge_label(charge: Option<crate::orchestration::BudgetCharge>) -> String {
    charge.map_or_else(
        || "-".to_string(),
        |charge| {
            format!(
                "rounds:{};agent_invocations:{};tokens:{};monetary_microunits:{};duration_millis:{}",
                charge.rounds,
                charge.agent_invocations,
                charge.tokens,
                charge.monetary_microunits,
                charge.duration_millis,
            )
        },
    )
}

fn optional_i64(value: Option<i64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "-",
    }
}

fn evidence_label(state: super::model::EvidenceState) -> &'static str {
    match state {
        super::model::EvidenceState::NotRecorded => "not_recorded",
        super::model::EvidenceState::Valid => "valid",
        super::model::EvidenceState::Invalid => "invalid",
    }
}

fn owner_binding_label(state: super::model::OwnerBindingState) -> &'static str {
    match state {
        super::model::OwnerBindingState::NotApplicable => "not_applicable",
        super::model::OwnerBindingState::Exact => "exact",
        super::model::OwnerBindingState::Invalid => "invalid",
    }
}

fn node_status_label(status: super::model::SafeNodeStatus) -> &'static str {
    match status {
        super::model::SafeNodeStatus::Pending => "pending",
        super::model::SafeNodeStatus::Running => "running",
        super::model::SafeNodeStatus::Completed => "completed",
        super::model::SafeNodeStatus::Failed => "failed",
        super::model::SafeNodeStatus::Skipped => "skipped",
    }
}

fn dispatch_label(state: super::model::DispatchEvidenceState) -> &'static str {
    match state {
        super::model::DispatchEvidenceState::NotApplicable => "not_applicable",
        super::model::DispatchEvidenceState::Reserved => "reserved",
        super::model::DispatchEvidenceState::Acknowledged => "acknowledged",
        super::model::DispatchEvidenceState::Finished => "finished",
        super::model::DispatchEvidenceState::AbandonedPending => "abandoned_pending",
        super::model::DispatchEvidenceState::AbandonedAcknowledged => "abandoned_acknowledged",
        super::model::DispatchEvidenceState::Invalid => "invalid",
    }
}

fn recovery_label(state: super::model::RecoveryEvidenceState) -> &'static str {
    match state {
        super::model::RecoveryEvidenceState::NotApplicable => "not_applicable",
        super::model::RecoveryEvidenceState::RunObservedPendingReconciliation => "run_observed_pending_reconciliation",
        super::model::RecoveryEvidenceState::Closed => "closed",
        super::model::RecoveryEvidenceState::Invalid => "invalid",
    }
}

fn budget_settlement_label(state: super::model::BudgetSettlementState) -> &'static str {
    match state {
        super::model::BudgetSettlementState::NotApplicable => "not_applicable",
        super::model::BudgetSettlementState::Reserved => "reserved",
        super::model::BudgetSettlementState::Settled => "settled",
        super::model::BudgetSettlementState::ConservativePendingAbandonment => "conservative_pending_abandonment",
        super::model::BudgetSettlementState::ConservativeAcknowledgedAbandonment => {
            "conservative_acknowledged_abandonment"
        }
        super::model::BudgetSettlementState::Invalid => "invalid",
    }
}

fn diagnostic_label(code: super::model::DiagnosticCode) -> &'static str {
    match code {
        super::model::DiagnosticCode::RunLifecycleInvalid => "run_lifecycle_invalid",
        super::model::DiagnosticCode::DetailRunIdentityMismatch => "detail_run_identity_mismatch",
        super::model::DiagnosticCode::DetailNodeCountMismatch => "detail_node_count_mismatch",
        super::model::DiagnosticCode::DetailNodeIdentityDuplicate => "detail_node_identity_duplicate",
        super::model::DiagnosticCode::ReplayRunIdentityMismatch => "replay_run_identity_mismatch",
        super::model::DiagnosticCode::ReplayInvalid => "replay_invalid",
        super::model::DiagnosticCode::OwnerIdentityMalformed => "owner_identity_malformed",
        super::model::DiagnosticCode::OrchestrationMissing => "orchestration_missing",
        super::model::DiagnosticCode::OrchestrationIdentityMismatch => "orchestration_identity_mismatch",
        super::model::DiagnosticCode::OrchestrationReplayInvalid => "orchestration_replay_invalid",
        super::model::DiagnosticCode::DispatchBindingMissing => "dispatch_binding_missing",
        super::model::DiagnosticCode::DispatchBindingDuplicate => "dispatch_binding_duplicate",
        super::model::DiagnosticCode::DispatchFactMismatch => "dispatch_fact_mismatch",
        super::model::DiagnosticCode::DispatchTerminalConflict => "dispatch_terminal_conflict",
    }
}
