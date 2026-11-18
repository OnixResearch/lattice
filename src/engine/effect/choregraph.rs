//! Choregraph V2 transition and exact intent-binding adapter.

const INTENT_IDENTITY_DOMAIN: &[u8] = b"lattice.choregraph-concrete-intent.v1";
const TRANSITION_SEPARATOR: &str = "->";
const DISPATCH_REQUEST_SLOT: &str = "dispatch-request";
const RESOLVED_PAYLOAD_SLOT: &str = "resolved-payload";

#[derive(Debug, Clone)]
pub struct TransitionContext {
    pub machine: super::authority::MachineIdentity,
    pub workflow_revision: super::authority::WorkflowRevisionIdentity,
    pub run: super::authority::RunIdentity,
    pub authority: super::authority::AuthorityIdentity,
    pub scope: super::authority::CollaborationScopeBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionResult {
    Accepted {
        next: choregraph_core::model::automaton::MachineState,
        bindings: Vec<super::authority::IntentBinding>,
    },
    Denied,
}

#[derive(Debug, Clone)]
pub struct EventRequest {
    pub declaration_key: String,
    pub node_id: String,
    pub source_node_id: String,
    pub action: choregraph_core::model::semantics::Action,
    pub request_ref: String,
    pub payload_ref: String,
}

#[must_use]
pub fn transition(
    machine: &choregraph_core::model::automaton::AdmittedMachine,
    state: &choregraph_core::model::automaton::MachineState,
    event: &choregraph_core::concrete::event::ConcreteEvent,
    context: &TransitionContext,
) -> TransitionResult {
    let Ok(plan) = seaglass_execution_graph::v2::admit_concrete_machine(machine.clone()) else {
        return TransitionResult::Denied;
    };
    match seaglass_execution_graph::v2::step_concrete_machine(&plan, state, event) {
        Ok(seaglass_execution_graph::v2::ConcreteMachineDecision::Accept { next, intents }) => {
            let mut bindings = Vec::with_capacity(intents.len());
            for intent in &intents {
                let Ok(binding) = bind_handoff(intent, context) else {
                    return TransitionResult::Denied;
                };
                bindings.push(binding);
            }
            TransitionResult::Accepted { next, bindings }
        }
        Ok(seaglass_execution_graph::v2::ConcreteMachineDecision::Deny { .. }) | Err(_) => TransitionResult::Denied,
    }
}

#[must_use]
pub fn requested_event(request: EventRequest) -> choregraph_core::concrete::event::ConcreteEvent {
    let mut event = choregraph_core::concrete::event::ConcreteEvent::new(request.node_id, request.action);
    let request_source = format!("node/{}/input/request", request.source_node_id);
    let payload_source = format!("node/{}/input/payload", request.source_node_id);
    event.effects.push(choregraph_core::concrete::effect::ConcreteEffectRequest {
        declaration_key: request.declaration_key,
        request: slot_binding(DISPATCH_REQUEST_SLOT, BindingReference {
            source: request_source,
            identity: request.request_ref,
        }),
        payload: slot_binding(RESOLVED_PAYLOAD_SLOT, BindingReference {
            source: payload_source,
            identity: request.payload_ref,
        }),
    });
    event
}

/// Host-generated identities consumed by one native effect invocation.
#[derive(Debug, Clone)]
pub struct InvocationReferences {
    pub request_ref: String,
    pub payload_ref: String,
}

/// Only the owning node's native invocation roles may consume host-generated
/// dispatch and resolved-payload identities. Exact content refs are not accepted.
#[must_use]
pub fn requested_construction_event(
    effect: &choregraph_nickel::construction::model::projection::ExpandedEffect,
    references: InvocationReferences,
) -> Option<choregraph_core::concrete::event::ConcreteEvent> {
    let request = native_invocation_slot(&effect.request_binding, effect.node_id, DISPATCH_REQUEST_SLOT)?;
    let payload = native_invocation_slot(&effect.payload_binding, effect.node_id, RESOLVED_PAYLOAD_SLOT)?;
    if effect.from_state != "ready" || effect.action != "start" {
        return None;
    }
    let mut event = choregraph_core::concrete::event::ConcreteEvent::new(
        format!("lattice-node/{}", effect.node_id),
        choregraph_core::model::semantics::Action::Start,
    );
    event.effects.push(choregraph_core::concrete::effect::ConcreteEffectRequest {
        declaration_key: effect.declaration_key.clone(),
        request: slot_binding(request.slot_id, request.reference(references.request_ref)),
        payload: slot_binding(payload.slot_id, payload.reference(references.payload_ref)),
    });
    Some(event)
}

/// Slot binding sourced from the owning node's native invocation role.
struct InvocationSlot<'effect> {
    slot_id: &'effect str,
    source_ref: &'effect str,
}

impl InvocationSlot<'_> {
    fn reference(&self, identity: String) -> BindingReference {
        BindingReference {
            source: self.source_ref.to_owned(),
            identity,
        }
    }
}

fn native_invocation_slot<'effect>(
    binding: &'effect choregraph_nickel::construction::model::projection::ExpandedBinding,
    node_id: u64,
    role: &str,
) -> Option<InvocationSlot<'effect>> {
    let choregraph_nickel::construction::model::projection::ExpandedBinding::Slot { slot_id, source_ref } = binding
    else {
        return None;
    };
    let is_owned_source = *source_ref == format!("node/{node_id}/invocation/{role}");
    let is_role_slot = slot_id.ends_with(&format!("/{role}"));
    (is_owned_source && is_role_slot).then_some(InvocationSlot { slot_id, source_ref })
}

struct BindingReference {
    source: String,
    identity: String,
}

fn slot_binding(
    slot_id: &str,
    reference: BindingReference,
) -> choregraph_core::concrete::reference::ConcreteBindingInput {
    choregraph_core::concrete::reference::ConcreteBindingInput {
        slot_id: Some(slot_id.to_owned()),
        source_ref: Some(reference.source),
        reference: choregraph_core::concrete::reference::ConcreteReferenceSpec {
            profile: super::authority::REQUEST_REFERENCE_PROFILE.to_owned(),
            identity: reference.identity,
        },
    }
}

fn bind_handoff(
    intent: &seaglass_execution_graph::v2::ConcreteIntentHandoff,
    context: &TransitionContext,
) -> Result<super::authority::IntentBinding, super::authority::BindingError> {
    debug_assert!(!intent.node_id.is_empty());
    debug_assert!(!intent.operation.is_empty());
    let transition = format!(
        "{}{}{}",
        seaglass_state_label(intent.from_state),
        TRANSITION_SEPARATOR,
        seaglass_action_label(intent.action)
    );
    let identity_fields = [
        context.machine.as_str(),
        context.workflow_revision.as_str(),
        context.run.as_str(),
        intent.node_id.as_str(),
        transition.as_str(),
        intent.operation.as_str(),
        intent.resource.as_str(),
        intent.ability.as_str(),
        intent.request.identity.as_str(),
        intent.payload.identity.as_str(),
        super::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF,
        context.authority.as_str(),
    ];
    let intent_identity = super::authority::blake3_reference(INTENT_IDENTITY_DOMAIN, &identity_fields);
    Ok(super::authority::IntentBinding {
        schema: super::authority::CHOREGRAPH_EFFECT_AUTHORITY_BINDING_SCHEMA.to_owned(),
        machine: context.machine.clone(),
        workflow_revision: context.workflow_revision.clone(),
        run: context.run.clone(),
        node: super::authority::NodeIdentity::new(&intent.node_id)?,
        transition: super::authority::TransitionIdentity::new(transition)?,
        operation: super::authority::OperationIdentity::new(&intent.operation)?,
        resource: super::authority::ResourceIdentity::new(&intent.resource)?,
        ability: super::authority::AbilityIdentity::new(&intent.ability)?,
        request: super::authority::RequestIdentity::new(&intent.request.identity)?,
        payload: super::authority::PayloadIdentity::new(&intent.payload.identity)?,
        policy: super::authority::PolicyIdentity::new(super::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF)?,
        authority: context.authority.clone(),
        intent: super::authority::IntentIdentity::new(intent_identity)?,
        scope: context.scope.clone(),
    })
}

fn seaglass_state_label(state: seaglass_execution_graph::lifecycle::StepState) -> &'static str {
    match state {
        seaglass_execution_graph::lifecycle::StepState::Pending => "pending",
        seaglass_execution_graph::lifecycle::StepState::Ready => "ready",
        seaglass_execution_graph::lifecycle::StepState::Running => "running",
        seaglass_execution_graph::lifecycle::StepState::Completed => "completed",
        seaglass_execution_graph::lifecycle::StepState::Failed => "failed",
        seaglass_execution_graph::lifecycle::StepState::Cancelled => "cancelled",
    }
}

fn seaglass_action_label(action: seaglass_execution_graph::lifecycle::StepAction) -> &'static str {
    match action {
        seaglass_execution_graph::lifecycle::StepAction::MarkReady => "mark-ready",
        seaglass_execution_graph::lifecycle::StepAction::Start => "start",
        seaglass_execution_graph::lifecycle::StepAction::Complete => "complete",
        seaglass_execution_graph::lifecycle::StepAction::Fail => "fail",
        seaglass_execution_graph::lifecycle::StepAction::Cancel => "cancel",
    }
}
