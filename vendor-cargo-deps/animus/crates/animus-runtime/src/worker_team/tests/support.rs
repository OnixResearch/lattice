pub(super) const TASK_ID: &str = "worker-docs-audit";
pub(super) const OBJECTIVE: &str = "Audit selected delegation docs";
pub(super) const SCOPE_LABEL: &str = "docs-audit";
pub(super) const SCOPE_SUMMARY: &str = "Review selected docs and report findings";
pub(super) const TARGET_DOCS: &str = "docs/agent-team.md";
pub(super) const PACKET_ID: &str = "packet-docs-audit";
pub(super) const CONTEXT_LABEL: &str = "selected-summary";
pub(super) const CONTEXT_BODY: &str = "Bounded summary of the relevant parent turn";
pub(super) const TOOL_LABEL: &str = "read-only-doc-search";
pub(super) const MUTATION_KIND: &str = "patch-proposal";
pub(super) const SCHEMA_NAME: &str = "agent-team-report";
pub(super) const SUMMARY_FIELD: &str = "summary";
pub(super) const RISK_FIELD: &str = "risk";
pub(super) const PARENT_RUN_ID: &str = "parent-run";
pub(super) const TURN_ID: &str = "turn-7";
pub(super) const POLICY_HASH_INPUT: &[u8] = b"worker-team-parent-policy";
pub(super) const GRANT_LABEL: &str = "bounded-worker";
pub(super) const WORKER_ID: &str = "worker-a";
pub(super) const EVIDENCE_ID: &str = "ev-summary";
pub(super) const EVIDENCE_SUMMARY: &str = "bounded fixture evidence";
pub(super) const CLAIM_SUBJECT: &str = "delegation-scope";
pub(super) const CLAIM_VALUE: &str = "bounded";
pub(super) const CONFLICTING_CLAIM_VALUE: &str = "unbounded";
pub(super) const MUTATION_TARGET: &str = "docs-agent-team";
pub(super) const HANDLE_ID: &str = "worker-handle";
pub(super) const EXPECTED_SINGLE_CALL: usize = 1;
pub(super) const INVALID_TASK_CASES: usize = 5;
pub(super) const CONTEXT_LEAK_CASES: usize = 5;
pub(super) const VALID_MAX_TURNS: u16 = 4;
pub(super) const VALID_MAX_CONTEXT_ITEMS: u16 = 3;
pub(super) const VALID_MAX_TOOL_CALLS: u16 = 2;

pub(super) fn valid_task() -> super::super::WorkerTask {
    super::super::validate_task(valid_task_draft()).expect("valid worker task")
}

pub(super) fn valid_task_draft() -> super::super::WorkerTaskDraft {
    super::super::WorkerTaskDraft {
        task_id: String::from(TASK_ID),
        objective: String::from(OBJECTIVE),
        scope: Some(valid_scope()),
        context_packet: Some(valid_context_packet()),
        allowed_capabilities: valid_capabilities(),
        budgets: Some(valid_budgets()),
        output_schema: Some(valid_schema()),
        parent_trace: Some(valid_parent_trace()),
        authority: Some(valid_authority()),
    }
}

pub(super) fn valid_scope() -> super::super::WorkerScope {
    super::super::WorkerScope {
        label: String::from(SCOPE_LABEL),
        summary: String::from(SCOPE_SUMMARY),
        allowed_targets: vec![String::from(TARGET_DOCS)],
    }
}

pub(super) fn valid_context_packet() -> super::super::WorkerContextPacket {
    super::super::WorkerContextPacket {
        packet_id: String::from(PACKET_ID),
        policy: super::super::ContextAdmissionPolicy::default(),
        items: vec![super::super::WorkerContextItem {
            kind: super::super::WorkerContextItemKind::Summary,
            label: String::from(CONTEXT_LABEL),
            body: String::from(CONTEXT_BODY),
            redaction: super::super::WorkerContextRedaction::SafeSummary,
        }],
    }
}

pub(super) fn unsafe_context_packet(
    kind: super::super::WorkerContextItemKind,
    body: &str,
) -> super::super::WorkerContextPacket {
    super::super::WorkerContextPacket {
        packet_id: String::from(PACKET_ID),
        policy: super::super::ContextAdmissionPolicy::default(),
        items: vec![super::super::WorkerContextItem {
            kind,
            label: String::from(CONTEXT_LABEL),
            body: String::from(body),
            redaction: super::super::WorkerContextRedaction::Raw,
        }],
    }
}

pub(super) fn valid_capabilities() -> Vec<super::super::WorkerCapability> {
    vec![
        super::super::WorkerCapability::ReadContext,
        super::super::WorkerCapability::ReportFindings,
        super::super::WorkerCapability::UseTool {
            tool_label: String::from(TOOL_LABEL),
        },
        super::super::WorkerCapability::RequestMutation {
            mutation_kind: String::from(MUTATION_KIND),
        },
    ]
}

pub(super) const fn valid_budgets() -> super::super::WorkerBudgets {
    super::super::WorkerBudgets {
        turns: VALID_MAX_TURNS,
        context_items: VALID_MAX_CONTEXT_ITEMS,
        tool_calls: VALID_MAX_TOOL_CALLS,
    }
}

pub(super) fn valid_schema() -> super::super::WorkerOutputSchema {
    super::super::WorkerOutputSchema {
        name: String::from(SCHEMA_NAME),
        required_fields: vec![String::from(SUMMARY_FIELD), String::from(RISK_FIELD)],
    }
}

pub(super) fn valid_parent_trace() -> super::super::ParentTrace {
    super::super::ParentTrace {
        parent_run_id: String::from(PARENT_RUN_ID),
        turn_id: String::from(TURN_ID),
        policy_hash: blake3::hash(POLICY_HASH_INPUT).to_hex().to_string(),
    }
}

pub(super) fn valid_authority() -> super::super::WorkerAuthority {
    super::super::WorkerAuthority {
        grant_label: String::from(GRANT_LABEL),
        parent_policy_required: true,
    }
}

pub(super) fn valid_report() -> super::super::Report {
    super::super::Report {
        worker_id: String::from(WORKER_ID),
        schema_name: String::from(SCHEMA_NAME),
        fields: valid_report_fields(),
        evidence: vec![super::super::EvidenceReference {
            evidence_id: String::from(EVIDENCE_ID),
            summary: String::from(EVIDENCE_SUMMARY),
        }],
        claims: vec![super::super::Claim {
            subject: String::from(CLAIM_SUBJECT),
            value: String::from(CLAIM_VALUE),
            evidence_ids: vec![String::from(EVIDENCE_ID)],
        }],
        mutation_intents: vec![super::super::MutationIntent {
            target: String::from(MUTATION_TARGET),
            mutation_kind: String::from(MUTATION_KIND),
            evidence_id: String::from(EVIDENCE_ID),
        }],
    }
}

pub(super) fn valid_report_fields() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        (String::from(SUMMARY_FIELD), String::from("safe summary")),
        (String::from(RISK_FIELD), String::from("low")),
    ])
}

#[derive(Default)]
pub(super) struct RecordingService {
    pub(super) enqueued: usize,
    pub(super) status_queries: usize,
    pub(super) cancellations: usize,
    pub(super) collections: usize,
}

impl super::super::WorkerService for RecordingService {
    fn enqueue(
        &mut self,
        _task: &super::super::WorkerTask,
    ) -> Result<super::super::WorkerHandle, super::super::DelegationFailure> {
        self.enqueued += EXPECTED_SINGLE_CALL;
        super::super::WorkerHandle::new(HANDLE_ID)
    }

    fn status(
        &mut self,
        _handle: &super::super::WorkerHandle,
    ) -> Result<super::super::WorkerStatus, super::super::DelegationFailure> {
        self.status_queries += EXPECTED_SINGLE_CALL;
        Ok(super::super::WorkerStatus::Running)
    }

    fn cancel(
        &mut self,
        _handle: &super::super::WorkerHandle,
    ) -> Result<super::super::WorkerCancelOutcome, super::super::DelegationFailure> {
        self.cancellations += EXPECTED_SINGLE_CALL;
        Ok(super::super::WorkerCancelOutcome::Accepted)
    }

    fn collect(
        &mut self,
        _handle: &super::super::WorkerHandle,
    ) -> Result<super::super::Report, super::super::DelegationFailure> {
        self.collections += EXPECTED_SINGLE_CALL;
        Ok(valid_report())
    }
}
