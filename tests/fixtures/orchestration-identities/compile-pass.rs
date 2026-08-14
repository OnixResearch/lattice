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
    accept_instance(&InstanceId::try_from_hex(DIGEST).expect("instance"));
    accept_dispatch(&DispatchId::try_from_hex(DIGEST).expect("dispatch"));
    accept_proposal(&ProposalId::try_from_hex(DIGEST).expect("proposal"));
    accept_fact(&FactId::try_from_hex(DIGEST).expect("fact"));
    accept_reservation(&ReservationId::try_from_text(DIGEST).expect("reservation"));
    accept_definition_revision(&DefinitionRevision::try_from_hex(DIGEST).expect("definition revision"));
    accept_workflow_revision(&WorkflowRevision::try_from_hex(DIGEST).expect("workflow revision"));
    accept_policy(&AuthorityPolicyIdentity::try_from_hex(DIGEST).expect("policy"));
    accept_event(&EventId::try_from_hex(DIGEST).expect("event"));
    accept_head(&LedgerHeadId::try_from_hex(DIGEST).expect("head"));
    accept_idempotency(&TriggerIdempotencyKey::try_from_text("request-one").expect("idempotency"));
}
