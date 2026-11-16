//! Pure durable agent-orchestration model and decision core.
//!
//! This module owns deterministic validation, lifecycle, budget, authority,
//! event, replay, and trigger-admission decisions. It performs no I/O.

const ORCHESTRATION_HASH_DOMAIN: &[u8] = b"lattice-orchestration:v1";
const DEFINITION_HASH_LABEL: &[u8] = b"definition";
const WORKFLOW_HASH_LABEL: &[u8] = b"workflow";
const INSTANCE_HASH_LABEL: &[u8] = b"instance";
const PROPOSAL_HASH_LABEL: &[u8] = b"proposal";
const APPROVAL_HASH_LABEL: &[u8] = b"approval";
const AUTHORITY_HASH_LABEL: &[u8] = b"authority";
const EVENT_HASH_LABEL: &[u8] = b"event";
const STATE_HASH_LABEL: &[u8] = b"state";
const FACT_HASH_LABEL: &[u8] = b"fact";
const TRIGGER_HASH_LABEL: &[u8] = b"trigger";
const CURRENT_SCHEMA_VERSION: u32 = 1;
const HEX_CHARS_PER_BYTE: usize = 2;
const FIRST_EVENT_SEQUENCE: u64 = 0;
const EVENT_SEQUENCE_INCREMENT: u64 = 1;
const INSTANCE_VERSION_INCREMENT: u64 = 1;
const EPOCH_INCREMENT: u64 = 1;
const ROUND_INCREMENT: u64 = 1;
const MAX_REPLAY_DIAGNOSTICS: usize = 1;
const RETRY_INCREMENT: u64 = 1;
const RETRY_PREFLIGHT_RESERVATION_ID: &str = "retry-preflight";
pub const ABANDONED_ACKNOWLEDGED_SUMMARY: &str =
    "local dispatch abandoned after acknowledgement; remote outcome unknown";
pub const ABANDONED_PENDING_SUMMARY: &str =
    "local dispatch abandoned before acknowledgement; workflow shell was not invoked";
const LOCAL_CHAIN_EVIDENCE_SCOPE: &str = "local-sequence-consistency-only";
const REMOTE_EFFECT_EVIDENCE_SCOPE: &str = "remote-effects-may-be-at-least-once";
mod ledger;
mod lifecycle;
mod model;

pub use budget_ledger_core::BudgetCharge;
pub use budget_ledger_core::BudgetChargePlan;
pub use budget_ledger_core::BudgetDimensions;
pub use budget_ledger_core::BudgetLimits;
pub use budget_ledger_core::BudgetPolicy;
pub use budget_ledger_core::BudgetSettlement;
pub use ledger::balances::reserve_budget;
pub use ledger::balances::settle_budget;
pub use ledger::history::Event;
pub use ledger::history::EventKind;
pub use ledger::history::ReplayReport;
pub use ledger::history::Transition;
pub use ledger::history::replay_events;
pub use lifecycle::protocol::Action;
pub use lifecycle::review::authorize_definition_actor;
pub use lifecycle::review::make_approval;
pub use lifecycle::review::make_proposal;
pub use lifecycle::routing::dispatch_identity;
pub use lifecycle::routing::preflight_retry;
pub use lifecycle::signals::TriggerAdmission;
pub use lifecycle::signals::TriggerContext;
pub use lifecycle::signals::TriggerPlan;
pub use lifecycle::signals::TriggerStartRequest;
pub use lifecycle::signals::admit_trigger_start;
pub use model::admission::AuthorityPolicy;
pub use model::admission::CompletionPolicy;
pub use model::admission::Definition;
pub use model::admission::RedactionPolicy;
pub use model::admission::RetryPolicy;
pub use model::admission::TriggerBinding;
pub use model::admission::TriggerKind;
pub use model::admission::WorkflowRevisionRef;
pub use model::admission::current_schema_version;
pub use model::admission::validate_definition;
pub use model::canonical::approval_identity;
pub use model::canonical::authority_policy_hash;
pub use model::canonical::definition_revision;
pub use model::canonical::fact_identity;
pub use model::canonical::proposal_identity;
pub use model::canonical::scoped_trigger_key;
pub use model::canonical::workflow_revision;
pub use model::failures::Diagnostic;
pub use model::failures::DiagnosticCode;
pub use model::state::ApprovalDecision;
pub use model::state::BudgetUsageBasis;
pub use model::state::DispatchIntent;
pub use model::state::DispatchStatus;
pub use model::state::Fact;
pub use model::state::Instance;
pub use model::state::RetryLedger;
pub use model::state::RetryableFailure;
pub use model::state::RevisionProposal;
pub use model::state::Status;
pub use model::state::is_terminal_status;
pub use model::state::new_instance;
pub use model::state::render_status_text;

pub type BudgetLedger = budget_ledger_core::BudgetLedger<super::identity::ReservationId>;
pub type BudgetReservation = budget_ledger_core::BudgetReservation<super::identity::ReservationId>;

// r[impl lattice.agent_orchestration.core]
// r[impl lattice.agent_orchestration.nominal_identities.transitions]
// r[impl lattice.agent_orchestration.nominal_identities.failure_matrix]
pub fn transition(instance: &Instance, fact: Fact) -> Result<Transition, Diagnostic> {
    if instance.accepted_fact_ids.contains(&fact.id) {
        return Ok(Transition {
            instance: instance.clone(),
            event: None,
            duplicate: true,
        });
    }
    let is_failed_exhaustion =
        instance.status == Status::Failed && matches!(fact.action, Action::BudgetExhausted { .. });
    if !matches!(fact.action, Action::RetryRound { .. }) && !is_failed_exhaustion {
        reject_terminal_mutation(instance)?;
    }
    let mut next = instance.clone();
    let kind = lifecycle::protocol::apply_action(&mut next, &fact.action)?;
    let is_inserted = next.accepted_fact_ids.insert(fact.id.clone());
    debug_assert!(is_inserted);
    next.version = next.version.saturating_add(INSTANCE_VERSION_INCREMENT);
    let event = ledger::history::build_event(instance, &next, kind, fact)?;
    next.last_event_sequence = Some(event.sequence);
    next.event_head = Some(super::identity::LedgerHeadId::from_event(&event.event_hash));
    Ok(Transition {
        instance: next,
        event: Some(event),
        duplicate: false,
    })
}

fn reject_terminal_mutation(instance: &Instance) -> Result<(), Diagnostic> {
    if is_terminal_status(instance.status) {
        return Err(model::failures::diagnostic(
            DiagnosticCode::TerminalStateMutation,
            format!("orchestration instance is terminal in state {:?}", instance.status),
        ));
    }
    Ok(())
}
