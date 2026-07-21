//! Pure construction and validation of content-omitting inspection reports.

const WORKFLOW_IDENTITY_DOMAIN: &str = "lattice-run-workflow-name-v1";
const IDENTITY_SEPARATOR: &str = "\0";
const MAX_INSPECTION_DIAGNOSTICS: usize = 16;
const MAX_REPLAY_DIAGNOSTIC_CODES: usize = 16;
const BLAKE3_IDENTITY_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const INVALID_IDENTITY: &str = "[invalid]";

struct Diagnostics {
    codes: Vec<super::model::DiagnosticCode>,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            codes: Vec::with_capacity(MAX_INSPECTION_DIAGNOSTICS),
        }
    }

    fn push(&mut self, code: super::model::DiagnosticCode) {
        self.codes.push(code);
    }

    fn is_empty(&self) -> bool {
        self.codes.is_empty()
    }

    fn into_codes(self) -> Vec<super::model::DiagnosticCode> {
        self.codes
    }
}

#[derive(Clone, Copy)]
struct OwnerTuple<'a> {
    instance_id: &'a str,
    dispatch_id: &'a str,
}

// r[impl lattice.workflow_execution.safe_run_inspection]
pub fn inspect(input: super::model::Input<'_>) -> super::model::Report {
    let mut diagnostics = Diagnostics::new();
    validate_lifecycle(input.record, &mut diagnostics);
    let details = inspect_details(input.record, input.details, &mut diagnostics);
    let replay = inspect_replay(input.record, input.replay_events, &mut diagnostics);
    let (owner, orchestration) = inspect_owner(input.record, input.orchestration, &mut diagnostics);
    let is_valid = diagnostics.is_empty();
    let result = super::model::Report {
        schema_version: super::model::SCHEMA_VERSION,
        valid: is_valid,
        run_id: input.record.run_id.0,
        workflow_identity: workflow_identity(&input.record.workflow_name),
        lifecycle: super::model::SafeLifecycle {
            started_epoch_ms: input.record.started_epoch_ms,
            finished_epoch_ms: input.record.finished_epoch_ms,
            node_count: input.record.node_count,
            cache_hits: input.record.cache_hits,
            failures: input.record.failures,
            terminal: input.record.finished_epoch_ms.is_some(),
        },
        owner,
        details,
        replay,
        orchestration,
        diagnostic_codes: diagnostics.into_codes(),
    };
    debug_assert_eq!(result.valid, result.diagnostic_codes.is_empty());
    debug_assert!(!result.workflow_identity.is_empty());
    result
}

fn validate_lifecycle(record: &crate::store::RunRecord, diagnostics: &mut Diagnostics) {
    let has_invalid_counts = record.cache_hits > record.node_count || record.failures > record.node_count;
    let has_invalid_time = record.finished_epoch_ms.is_some_and(|finished| finished < record.started_epoch_ms);
    if has_invalid_counts || has_invalid_time {
        diagnostics.push(super::model::DiagnosticCode::RunLifecycleInvalid);
    }
}

fn inspect_details(
    record: &crate::store::RunRecord,
    details: Option<&crate::store::RunDetailRecord>,
    diagnostics: &mut Diagnostics,
) -> super::model::SafeDetails {
    let Some(details) = details else {
        return super::model::SafeDetails {
            state: super::model::EvidenceState::NotRecorded,
            nodes: Vec::new(),
        };
    };
    if details.run_id != record.run_id {
        diagnostics.push(super::model::DiagnosticCode::DetailRunIdentityMismatch);
        return super::model::SafeDetails {
            state: super::model::EvidenceState::Invalid,
            nodes: Vec::new(),
        };
    }

    let mut node_ids = std::collections::BTreeSet::new();
    let has_duplicate = details.nodes.iter().any(|node| !node_ids.insert(node.node_id.0));
    if details.nodes.len() != record.node_count {
        diagnostics.push(super::model::DiagnosticCode::DetailNodeCountMismatch);
    }
    if has_duplicate {
        diagnostics.push(super::model::DiagnosticCode::DetailNodeIdentityDuplicate);
    }
    let is_valid = details.nodes.len() == record.node_count && !has_duplicate;
    let nodes = details.nodes.iter().map(safe_node).collect::<Vec<_>>();
    debug_assert_eq!(nodes.len(), details.nodes.len());
    super::model::SafeDetails {
        state: if is_valid {
            super::model::EvidenceState::Valid
        } else {
            super::model::EvidenceState::Invalid
        },
        nodes,
    }
}

fn safe_node(node: &crate::store::RunNodeDetail) -> super::model::SafeNode {
    super::model::SafeNode {
        node_id: node.node_id.0,
        status: safe_node_status(node.status),
        cached: node.cached,
        output_count: node.outputs.len(),
        error_present: node.error.is_some(),
    }
}

fn safe_node_status(status: crate::store::RunNodeDetailStatus) -> super::model::SafeNodeStatus {
    match status {
        crate::store::RunNodeDetailStatus::Pending => super::model::SafeNodeStatus::Pending,
        crate::store::RunNodeDetailStatus::Running => super::model::SafeNodeStatus::Running,
        crate::store::RunNodeDetailStatus::Completed => super::model::SafeNodeStatus::Completed,
        crate::store::RunNodeDetailStatus::Failed => super::model::SafeNodeStatus::Failed,
        crate::store::RunNodeDetailStatus::Skipped => super::model::SafeNodeStatus::Skipped,
    }
}

fn inspect_replay(
    record: &crate::store::RunRecord,
    events: Option<&[crate::replay::ReplayEventRecord]>,
    diagnostics: &mut Diagnostics,
) -> super::model::SafeReplay {
    let Some(events) = events else {
        return super::model::SafeReplay {
            state: super::model::EvidenceState::NotRecorded,
            event_count: 0,
            diagnostic_codes: Vec::new(),
        };
    };
    let has_wrong_run = events.iter().any(|event| event.run_id != record.run_id);
    let replay_diagnostics = crate::replay::validate_replay_events(events, None);
    let has_fatal = replay_diagnostics.iter().any(|diagnostic| diagnostic.fatal);
    if has_wrong_run {
        diagnostics.push(super::model::DiagnosticCode::ReplayRunIdentityMismatch);
    }
    if has_fatal {
        diagnostics.push(super::model::DiagnosticCode::ReplayInvalid);
    }
    let diagnostic_codes = replay_diagnostics
        .iter()
        .take(MAX_REPLAY_DIAGNOSTIC_CODES)
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    let result = super::model::SafeReplay {
        state: if has_wrong_run || has_fatal {
            super::model::EvidenceState::Invalid
        } else {
            super::model::EvidenceState::Valid
        },
        event_count: events.len(),
        diagnostic_codes,
    };
    debug_assert_eq!(result.event_count, events.len());
    debug_assert_eq!(result.state == super::model::EvidenceState::Invalid, has_wrong_run || has_fatal);
    result
}

fn inspect_owner(
    record: &crate::store::RunRecord,
    orchestration: Option<super::model::OrchestrationInput<'_>>,
    diagnostics: &mut Diagnostics,
) -> (super::model::SafeOwner, Option<super::model::SafeOrchestrationEvidence>) {
    match &record.owner {
        crate::store::RunOwner::Unscoped => (super::model::SafeOwner::Unscoped, None),
        crate::store::RunOwner::Orchestration {
            instance_id,
            dispatch_id,
        } => inspect_orchestration_owner(
            record,
            OwnerTuple {
                instance_id,
                dispatch_id,
            },
            orchestration,
            diagnostics,
        ),
    }
}

fn inspect_orchestration_owner(
    record: &crate::store::RunRecord,
    owner: OwnerTuple<'_>,
    orchestration: Option<super::model::OrchestrationInput<'_>>,
    diagnostics: &mut Diagnostics,
) -> (super::model::SafeOwner, Option<super::model::SafeOrchestrationEvidence>) {
    let has_valid_owner_identities = is_blake3_identity(owner.instance_id) && is_blake3_identity(owner.dispatch_id);
    if !has_valid_owner_identities {
        diagnostics.push(super::model::DiagnosticCode::OwnerIdentityMalformed);
    }
    let Some(orchestration) = orchestration else {
        diagnostics.push(super::model::DiagnosticCode::OrchestrationMissing);
        return (safe_owner(owner, super::model::OwnerBindingState::Invalid), None);
    };
    if orchestration.instance.id != owner.instance_id {
        diagnostics.push(super::model::DiagnosticCode::OrchestrationIdentityMismatch);
    }
    let event_chain = super::evidence::orchestration_event_chain(orchestration.instance, orchestration.events);
    if event_chain == super::model::EvidenceState::Invalid {
        diagnostics.push(super::model::DiagnosticCode::OrchestrationReplayInvalid);
    }
    let facts = super::evidence::collect_dispatch_facts(record.run_id, owner.dispatch_id, orchestration.events);
    for code in super::evidence::dispatch_diagnostics(&facts) {
        diagnostics.push(code);
    }
    let is_exact = has_valid_owner_identities
        && orchestration.instance.id == owner.instance_id
        && event_chain == super::model::EvidenceState::Valid
        && facts.has_exact_binding();
    let binding = if is_exact {
        super::model::OwnerBindingState::Exact
    } else {
        super::model::OwnerBindingState::Invalid
    };
    let evidence = super::evidence::summarize_orchestration(orchestration.instance, event_chain, &facts);
    debug_assert_eq!(binding == super::model::OwnerBindingState::Exact, is_exact);
    debug_assert_eq!(evidence.event_chain, event_chain);
    (safe_owner(owner, binding), Some(evidence))
}

fn workflow_identity(workflow_name: &str) -> String {
    let material = format!("{WORKFLOW_IDENTITY_DOMAIN}{IDENTITY_SEPARATOR}{workflow_name}");
    format!("b3:{}", blake3::hash(material.as_bytes()).to_hex())
}

fn safe_owner(owner: OwnerTuple<'_>, binding: super::model::OwnerBindingState) -> super::model::SafeOwner {
    super::model::SafeOwner::Orchestration {
        instance_id: safe_identity(owner.instance_id),
        dispatch_id: safe_identity(owner.dispatch_id),
        binding,
    }
}

fn safe_identity(identity: &str) -> String {
    if is_blake3_identity(identity) {
        identity.to_string()
    } else {
        INVALID_IDENTITY.to_string()
    }
}

fn is_blake3_identity(identity: &str) -> bool {
    let hex = identity.strip_prefix(BLAKE3_IDENTITY_PREFIX).unwrap_or(identity);
    hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}
