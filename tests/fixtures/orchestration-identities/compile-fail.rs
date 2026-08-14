use lattice::orchestration::AuthorityPolicyIdentity;
use lattice::orchestration::DefinitionRevision;
use lattice::orchestration::DispatchId;
use lattice::orchestration::EventId;
use lattice::orchestration::FactId;
use lattice::orchestration::InstanceId;
use lattice::orchestration::LedgerHeadId;
use lattice::orchestration::ProposalId;
use lattice::orchestration::ReservationId;
use lattice::orchestration::TriggerIdempotencyKey;
use lattice::orchestration::WorkflowRevision;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn accept_instance(_: &InstanceId) {}
fn accept_dispatch(_: &DispatchId) {}
fn accept_proposal(_: &ProposalId) {}
fn accept_fact(_: &FactId) {}
fn accept_reservation(_: &ReservationId) {}
fn accept_definition_revision(_: &DefinitionRevision) {}
fn accept_workflow_revision(_: &WorkflowRevision) {}
fn accept_policy(_: &AuthorityPolicyIdentity) {}
fn accept_event(_: &EventId) {}
fn accept_head(_: &LedgerHeadId) {}
fn accept_idempotency(_: &TriggerIdempotencyKey) {}

fn main() {
    let instance = InstanceId::try_from_hex(DIGEST).expect("instance");
    let dispatch = DispatchId::try_from_hex(DIGEST).expect("dispatch");
    let proposal = ProposalId::try_from_hex(DIGEST).expect("proposal");
    let fact = FactId::try_from_hex(DIGEST).expect("fact");
    let reservation = ReservationId::try_from_text(DIGEST).expect("reservation");
    let definition_revision = DefinitionRevision::try_from_hex(DIGEST).expect("definition revision");
    let workflow_revision = WorkflowRevision::try_from_hex(DIGEST).expect("workflow revision");
    let policy = AuthorityPolicyIdentity::try_from_hex(DIGEST).expect("policy");
    let event = EventId::try_from_hex(DIGEST).expect("event");
    let head = LedgerHeadId::try_from_hex(DIGEST).expect("head");
    let idempotency = TriggerIdempotencyKey::try_from_text("request-one").expect("idempotency");

    accept_instance(&dispatch);
    accept_dispatch(&instance);
    accept_proposal(&fact);
    accept_fact(&proposal);
    accept_reservation(&dispatch);
    accept_definition_revision(&workflow_revision);
    accept_workflow_revision(&definition_revision);
    accept_policy(&event);
    accept_event(&policy);
    accept_head(&event);
    accept_event(&head);
    accept_idempotency(&instance);
    accept_instance(&idempotency);
    let _ = reservation;
}
