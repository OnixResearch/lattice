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
        next: choregraph_core::MachineState,
        bindings: Vec<super::authority::IntentBinding>,
    },
    Denied,
}

#[derive(Debug, Clone)]
pub struct EventRequest {
    pub declaration_key: String,
    pub node_id: String,
    pub source_node_id: String,
    pub action: choregraph_core::Action,
    pub request_ref: String,
    pub payload_ref: String,
}

#[must_use]
pub fn transition(
    machine: &choregraph_core::AdmittedMachine,
    state: &choregraph_core::MachineState,
    event: &choregraph_core::ConcreteEvent,
    context: &TransitionContext,
) -> TransitionResult {
    let Ok(plan) = seaglass_execution_graph::admit_concrete_machine(machine.clone()) else {
        return TransitionResult::Denied;
    };
    match seaglass_execution_graph::step_concrete_machine(&plan, state, event) {
        seaglass_execution_graph::ConcreteMachineDecision::Accept { next, intents } => {
            let mut bindings = Vec::with_capacity(intents.len());
            for intent in &intents {
                let Ok(binding) = bind_handoff(intent, context) else {
                    return TransitionResult::Denied;
                };
                bindings.push(binding);
            }
            TransitionResult::Accepted { next, bindings }
        }
        seaglass_execution_graph::ConcreteMachineDecision::Deny { .. } => TransitionResult::Denied,
    }
}

#[must_use]
pub fn requested_event(request: EventRequest) -> choregraph_core::ConcreteEvent {
    let mut event = choregraph_core::ConcreteEvent::new(request.node_id, request.action);
    let request_source = format!("node/{}/input/request", request.source_node_id);
    let payload_source = format!("node/{}/input/payload", request.source_node_id);
    event.effects.push(choregraph_core::ConcreteEffectRequest {
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

struct BindingReference {
    source: String,
    identity: String,
}

fn slot_binding(slot_id: &str, reference: BindingReference) -> choregraph_core::ConcreteBindingInput {
    choregraph_core::ConcreteBindingInput {
        slot_id: Some(slot_id.to_owned()),
        source_ref: Some(reference.source),
        reference: choregraph_core::ConcreteReferenceSpec {
            profile: super::authority::REQUEST_REFERENCE_PROFILE.to_owned(),
            identity: reference.identity,
        },
    }
}

fn bind_handoff(
    intent: &seaglass_execution_graph::ConcreteIntentHandoff,
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

fn seaglass_state_label(state: seaglass_execution_graph::StepState) -> &'static str {
    match state {
        seaglass_execution_graph::StepState::Pending => "pending",
        seaglass_execution_graph::StepState::Ready => "ready",
        seaglass_execution_graph::StepState::Running => "running",
        seaglass_execution_graph::StepState::Completed => "completed",
        seaglass_execution_graph::StepState::Failed => "failed",
        seaglass_execution_graph::StepState::Cancelled => "cancelled",
    }
}

fn seaglass_action_label(action: seaglass_execution_graph::StepAction) -> &'static str {
    match action {
        seaglass_execution_graph::StepAction::MarkReady => "mark-ready",
        seaglass_execution_graph::StepAction::Start => "start",
        seaglass_execution_graph::StepAction::Complete => "complete",
        seaglass_execution_graph::StepAction::Fail => "fail",
        seaglass_execution_graph::StepAction::Cancel => "cancel",
    }
}
