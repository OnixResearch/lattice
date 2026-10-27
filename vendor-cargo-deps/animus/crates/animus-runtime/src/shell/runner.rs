const AGENT_CONTROL_CANCELLED: &str = "agent_control_cancelled";
const AGENT_CONTROL_WAIT_FAILED: &str = "agent_control_wait_failed";

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Step {
    Continue { state: crate::TurnState },
    Complete { report: crate::RunReport },
}

pub(super) fn run_planned_step<Fabric, Model, Tool>(
    config: crate::RuntimeConfig,
    state: crate::TurnState,
    fabric: &mut Fabric,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> Step
where
    Fabric: crate::LoopFabric,
    Model: animus_adapters::ModelAdapter,
    Model::Error: core::fmt::Display,
    Tool: animus_adapters::ToolAdapter,
    Tool::Error: core::fmt::Display,
{
    match crate::plan_next_turn(config, state) {
        crate::TurnPlan::Evaluate { state: ready, turn } => {
            run_ready_turn(ready, &turn, fabric, adapters)
        }
        crate::TurnPlan::Complete { report } => Step::Complete { report },
    }
}

fn run_ready_turn<Fabric, Model, Tool>(
    state: crate::TurnState,
    turn: &crate::RuntimeLoopTurn,
    fabric: &mut Fabric,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> Step
where
    Fabric: crate::LoopFabric,
    Model: animus_adapters::ModelAdapter,
    Model::Error: core::fmt::Display,
    Tool: animus_adapters::ToolAdapter,
    Tool::Error: core::fmt::Display,
{
    let state = match wait_for_agent_control(
        state,
        animus_adapters::AgentControlCheckpointKind::BeforeFabric,
        adapters,
    ) {
        Ok(ready) => ready,
        Err(report) => return Step::Complete { report },
    };
    let decision = match fabric.decide(turn) {
        Ok(decision) => decision,
        Err(error) => {
            return Step::Complete {
                report: crate::complete_with_fabric_error(state, error.to_string()),
            };
        }
    };

    apply_decision_step(crate::apply_decision(state, decision), adapters)
}

fn apply_decision_step<Model, Tool>(
    step: crate::DecisionStep,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> Step
where
    Model: animus_adapters::ModelAdapter,
    Model::Error: core::fmt::Display,
    Tool: animus_adapters::ToolAdapter,
    Tool::Error: core::fmt::Display,
{
    match step {
        crate::DecisionStep::Dispatch { state, request } => {
            apply_dispatch_step(prepare_dispatch(state, request, adapters), adapters)
        }
        crate::DecisionStep::Complete { report } => Step::Complete { report },
    }
}

enum DispatchStep {
    Dispatch {
        state: crate::TurnState,
        request: crate::AdapterRequest,
    },
    Complete {
        report: crate::RunReport,
    },
}

fn apply_dispatch_step<Model, Tool>(
    step: DispatchStep,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> Step
where
    Model: animus_adapters::ModelAdapter,
    Model::Error: core::fmt::Display,
    Tool: animus_adapters::ToolAdapter,
    Tool::Error: core::fmt::Display,
{
    match step {
        DispatchStep::Dispatch { state, request } => {
            let checkpoint = dispatch_checkpoint(&request);
            match wait_for_agent_control(state, checkpoint, adapters) {
                Ok(ready) => apply_adapter_step(crate::apply_adapter_outcome(
                    ready,
                    dispatch_adapter(request, adapters),
                )),
                Err(report) => Step::Complete { report },
            }
        }
        DispatchStep::Complete { report } => Step::Complete { report },
    }
}

fn dispatch_checkpoint(
    request: &crate::AdapterRequest,
) -> animus_adapters::AgentControlCheckpointKind {
    match request {
        crate::AdapterRequest::Model { .. } => {
            animus_adapters::AgentControlCheckpointKind::BeforeModel
        }
        crate::AdapterRequest::Tool { .. } => {
            animus_adapters::AgentControlCheckpointKind::BeforeTool
        }
    }
}

fn wait_for_agent_control<Model, Tool>(
    state: crate::TurnState,
    checkpoint: animus_adapters::AgentControlCheckpointKind,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> Result<crate::TurnState, crate::RunReport> {
    match adapters.wait_for_agent_control(checkpoint) {
        Ok(animus_adapters::AgentControlPermit::Continue) => Ok(state),
        Ok(animus_adapters::AgentControlPermit::Cancel) => Err(
            crate::transition::complete_with_agent_control_stop(state, AGENT_CONTROL_CANCELLED),
        ),
        Err(_error) => Err(crate::transition::complete_with_agent_control_stop(
            state,
            AGENT_CONTROL_WAIT_FAILED,
        )),
    }
}

fn prepare_dispatch<Model, Tool>(
    state: crate::TurnState,
    request: crate::AdapterRequest,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> DispatchStep {
    match request {
        crate::AdapterRequest::Model { request } => {
            prepare_model_dispatch(state, request, &mut adapters.middleware)
        }
        crate::AdapterRequest::Tool { request } => {
            prepare_tool_dispatch(state, request, &mut adapters.middleware)
        }
    }
}

fn prepare_model_dispatch(
    state: crate::TurnState,
    request: animus_adapters::ModelRequest,
    middleware: &mut crate::RuntimeMiddlewareStack<'_>,
) -> DispatchStep {
    match middleware.prepare_model_dispatch(request) {
        crate::hooks::chain::MiddlewareOutcome::Accepted(request) => DispatchStep::Dispatch {
            state,
            request: crate::AdapterRequest::Model { request },
        },
        crate::hooks::chain::MiddlewareOutcome::Blocked {
            value,
            hook,
            reason,
        } => DispatchStep::Complete {
            report: crate::complete_with_guard_block(
                state,
                crate::AdapterRequest::Model { request: value },
                hook,
                reason,
            ),
        },
        crate::hooks::chain::MiddlewareOutcome::Failed { value, error } => DispatchStep::Complete {
            report: crate::complete_with_guard_failure(
                state,
                crate::AdapterRequest::Model { request: value },
                &error,
            ),
        },
    }
}

fn prepare_tool_dispatch(
    state: crate::TurnState,
    request: animus_adapters::ToolRequest,
    middleware: &mut crate::RuntimeMiddlewareStack<'_>,
) -> DispatchStep {
    match middleware.prepare_tool_dispatch(request) {
        crate::hooks::chain::MiddlewareOutcome::Accepted(request) => DispatchStep::Dispatch {
            state,
            request: crate::AdapterRequest::Tool { request },
        },
        crate::hooks::chain::MiddlewareOutcome::Blocked {
            value,
            hook,
            reason,
        } => DispatchStep::Complete {
            report: crate::complete_with_guard_block(
                state,
                crate::AdapterRequest::Tool { request: value },
                hook,
                reason,
            ),
        },
        crate::hooks::chain::MiddlewareOutcome::Failed { value, error } => DispatchStep::Complete {
            report: crate::complete_with_guard_failure(
                state,
                crate::AdapterRequest::Tool { request: value },
                &error,
            ),
        },
    }
}

fn apply_adapter_step(step: crate::AdapterStep) -> Step {
    match step {
        crate::AdapterStep::Continue { state } => Step::Continue { state },
        crate::AdapterStep::Complete { report } => Step::Complete { report },
    }
}

fn dispatch_adapter<Model, Tool>(
    request: crate::AdapterRequest,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> crate::AdapterOutcome
where
    Model: animus_adapters::ModelAdapter,
    Model::Error: core::fmt::Display,
    Tool: animus_adapters::ToolAdapter,
    Tool::Error: core::fmt::Display,
{
    match request {
        crate::AdapterRequest::Model { request } => dispatch_model_adapter(&request, adapters),
        crate::AdapterRequest::Tool { request } => dispatch_tool_adapter(&request, adapters),
    }
}

fn dispatch_model_adapter<Model, Tool>(
    request: &animus_adapters::ModelRequest,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> crate::AdapterOutcome
where
    Model: animus_adapters::ModelAdapter,
    Model::Error: core::fmt::Display,
    Tool: animus_adapters::ToolAdapter,
    Tool::Error: core::fmt::Display,
{
    // r[impl agent_harness_runtime.model_dispatch.runtime_gate]
    // r[impl agent_harness_runtime.scoped_capabilities.exact_resolution]
    debug_assert!(!request.prompt().is_empty());
    let plan = crate::plan_model_dispatch(crate::ModelDispatchPlanRequest {
        request,
        required_capability: crate::RuntimeCapabilityDescriptorDraft::default_model(),
        registry: &adapters.capabilities,
        policy: crate::ModelDispatchPolicyDraft::compatibility_defaults(),
    });
    debug_assert_eq!(
        plan.request_contract_identity(),
        animus_adapters::MODEL_REQUEST_CONTRACT_IDENTITY
    );
    if !plan.is_admitted() {
        return crate::AdapterOutcome::ModelCall {
            outcome: failed_outcome(
                model_plan_failure_class(plan.issue()),
                model_plan_failure_message(plan.issue()),
            ),
        };
    }
    match adapters.model.as_deref_mut() {
        Some(model) => model.call(request).map_or_else(
            |error| crate::AdapterOutcome::ModelCall {
                outcome: failed_outcome(
                    animus_adapters::AdapterCallFailureClass::ExecutionFailure,
                    error.to_string().as_str(),
                ),
            },
            |response| crate::AdapterOutcome::ModelCall {
                outcome: animus_adapters::AdapterCallOutcome::completed(response),
            },
        ),
        None => missing_service(crate::AdapterComponent::Model),
    }
}

fn dispatch_tool_adapter<Model, Tool>(
    request: &animus_adapters::ToolRequest,
    adapters: &mut super::RuntimeAdapters<'_, Model, Tool>,
) -> crate::AdapterOutcome
where
    Model: animus_adapters::ModelAdapter,
    Model::Error: core::fmt::Display,
    Tool: animus_adapters::ToolAdapter,
    Tool::Error: core::fmt::Display,
{
    // r[impl agent_harness_runtime.tool_calling_breaking.dispatch_plan.fail_closed]
    // r[impl agent_harness_runtime.tool_calling_breaking.catalog_admission.exact_capability]
    debug_assert!(!request.name().is_empty());
    let plan = crate::plan_tool_dispatch(crate::ToolDispatchPlanRequest {
        request,
        definitions: adapters.tool_definitions(),
        registry: &adapters.capabilities,
        executor_present: adapters.tool.is_some(),
        policy: crate::ToolDispatchPolicyDraft::current_defaults(),
    });
    debug_assert_eq!(
        plan.request_contract_identity(),
        animus_adapters::TOOL_REQUEST_CONTRACT_IDENTITY
    );
    if !plan.is_admitted() {
        return crate::AdapterOutcome::ToolCall {
            outcome: failed_outcome(
                tool_plan_failure_class(plan.issue()),
                tool_plan_failure_message(plan.issue()),
            ),
        };
    }
    match adapters.tool.as_deref_mut() {
        Some(tool) => tool.call(request).map_or_else(
            |error| crate::AdapterOutcome::ToolCall {
                outcome: failed_outcome(
                    animus_adapters::AdapterCallFailureClass::ExecutionFailure,
                    error.to_string().as_str(),
                ),
            },
            |response| crate::AdapterOutcome::ToolCall {
                outcome: animus_adapters::AdapterCallOutcome::completed(response),
            },
        ),
        None => missing_service(crate::AdapterComponent::Tool),
    }
}

fn missing_service(component: crate::AdapterComponent) -> crate::AdapterOutcome {
    match component {
        crate::AdapterComponent::Model => crate::AdapterOutcome::ModelCall {
            outcome: failed_outcome(animus_adapters::AdapterCallFailureClass::Absent, "absent"),
        },
        crate::AdapterComponent::Tool => crate::AdapterOutcome::ToolCall {
            outcome: failed_outcome(animus_adapters::AdapterCallFailureClass::Absent, "absent"),
        },
    }
}

fn model_plan_failure_message(issue: crate::ModelDispatchPlanIssue) -> &'static str {
    match issue {
        crate::ModelDispatchPlanIssue::MissingProvider => "absent",
        other => other.as_str(),
    }
}

fn tool_plan_failure_message(issue: crate::ToolDispatchPlanIssue) -> &'static str {
    match issue {
        crate::ToolDispatchPlanIssue::MissingExecutor => "absent",
        other => other.as_str(),
    }
}

fn model_plan_failure_class(
    issue: crate::ModelDispatchPlanIssue,
) -> animus_adapters::AdapterCallFailureClass {
    match issue {
        crate::ModelDispatchPlanIssue::MissingProvider => {
            animus_adapters::AdapterCallFailureClass::Absent
        }
        crate::ModelDispatchPlanIssue::DeniedCapability
        | crate::ModelDispatchPlanIssue::BudgetOverflow
        | crate::ModelDispatchPlanIssue::FallbackDenied => {
            animus_adapters::AdapterCallFailureClass::Denied
        }
        crate::ModelDispatchPlanIssue::Cancelled => {
            animus_adapters::AdapterCallFailureClass::Cancelled
        }
        crate::ModelDispatchPlanIssue::TimedOut => {
            animus_adapters::AdapterCallFailureClass::TimedOut
        }
        crate::ModelDispatchPlanIssue::None
        | crate::ModelDispatchPlanIssue::InvalidCapability
        | crate::ModelDispatchPlanIssue::MalformedRequestShape
        | crate::ModelDispatchPlanIssue::UnsupportedStreamPolicy
        | crate::ModelDispatchPlanIssue::MissingReceiptPolicy => {
            animus_adapters::AdapterCallFailureClass::MalformedOutput
        }
    }
}

fn tool_plan_failure_class(
    issue: crate::ToolDispatchPlanIssue,
) -> animus_adapters::AdapterCallFailureClass {
    match issue {
        crate::ToolDispatchPlanIssue::UnknownTool
        | crate::ToolDispatchPlanIssue::MissingExecutor => {
            animus_adapters::AdapterCallFailureClass::MissingExecutor
        }
        crate::ToolDispatchPlanIssue::UnavailableExecutor => {
            animus_adapters::AdapterCallFailureClass::Unavailable
        }
        crate::ToolDispatchPlanIssue::DeniedCapability => {
            animus_adapters::AdapterCallFailureClass::DeniedCapability
        }
        crate::ToolDispatchPlanIssue::ApprovalDenied => {
            animus_adapters::AdapterCallFailureClass::ApprovalDenied
        }
        crate::ToolDispatchPlanIssue::Cancelled => {
            animus_adapters::AdapterCallFailureClass::Cancelled
        }
        crate::ToolDispatchPlanIssue::TimedOut => {
            animus_adapters::AdapterCallFailureClass::TimedOut
        }
        crate::ToolDispatchPlanIssue::UnsafeReference => {
            animus_adapters::AdapterCallFailureClass::UnsafeOutputReference
        }
        crate::ToolDispatchPlanIssue::UnboundedOutput => {
            animus_adapters::AdapterCallFailureClass::UnboundedOversizedOutput
        }
        crate::ToolDispatchPlanIssue::MutationConflict => {
            animus_adapters::AdapterCallFailureClass::MutationConflict
        }
        crate::ToolDispatchPlanIssue::FallbackDenied => {
            animus_adapters::AdapterCallFailureClass::FallbackDenied
        }
        crate::ToolDispatchPlanIssue::None
        | crate::ToolDispatchPlanIssue::InvalidRequestShape
        | crate::ToolDispatchPlanIssue::InactiveTool
        | crate::ToolDispatchPlanIssue::UntrustedTool
        | crate::ToolDispatchPlanIssue::SchemaIdentityMismatch
        | crate::ToolDispatchPlanIssue::MalformedOutputPolicy => {
            animus_adapters::AdapterCallFailureClass::InvalidInput
        }
    }
}

fn failed_outcome<Response>(
    class: animus_adapters::AdapterCallFailureClass,
    message: &str,
) -> animus_adapters::AdapterCallOutcome<Response> {
    animus_adapters::AdapterCallOutcome::failed(
        class,
        animus_adapters::AdapterCallFailureMetadataDraft {
            message,
            correlation_id: None,
            receipt_reference: None,
        },
    )
    .unwrap_or_else(|_error| fallback_failed_outcome(class))
}

fn fallback_failed_outcome<Response>(
    class: animus_adapters::AdapterCallFailureClass,
) -> animus_adapters::AdapterCallOutcome<Response> {
    animus_adapters::AdapterCallOutcome::fallback_failed(class)
}
