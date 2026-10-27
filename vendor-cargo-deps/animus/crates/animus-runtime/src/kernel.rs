//! Minimal runtime kernel boundary.
//!
//! This module is the direct import path for embedders that only need typed
//! runtime input/config, neutral loop orchestration, turn state, runtime events,
//! capability resolution, model/tool dispatch planning, and injected adapter
//! calls. Optional domains such as Steel, headless protocol, provenance,
//! worker-team delegation, batch, compaction, dataset export, research/eval,
//! TUI, std shell, provider clients, concrete storage, and host authority stay
//! outside this kernel import path.

pub use crate::{
    AdapterOutcome, AdapterRequest, AdapterStep, DEFAULT_MODEL_CAPABILITY_LABEL,
    DEFAULT_TOOL_CAPABILITY_LABEL, DecisionStep, LoopDecisionKind, LoopFabric,
    MODEL_PROVIDER_NEUTRAL_CONTRACT_IDENTITY, ModelDispatchCancellationPosture,
    ModelDispatchFallbackPolicy, ModelDispatchPlan, ModelDispatchPlanIssue,
    ModelDispatchPlanRequest, ModelDispatchPlanStatus, ModelDispatchPolicyDraft,
    ModelDispatchReceiptPolicy, ModelDispatchStreamPolicy, ModelDispatchTimeoutPosture, RunReport,
    RuntimeAdapters, RuntimeCapabilityDescriptor, RuntimeCapabilityDescriptorDraft,
    RuntimeCapabilityError, RuntimeCapabilityErrorKind, RuntimeCapabilityFamily,
    RuntimeCapabilityFamilyMapError, RuntimeCapabilityFamilyRecord, RuntimeCapabilityOutcome,
    RuntimeCapabilityRegistry, RuntimeCapabilityResolution, RuntimeCapabilityServiceKind,
    RuntimeConfig, RuntimeError, RuntimeEvent, RuntimeEventKind, RuntimeFailure, RuntimeInput,
    RuntimeLoopContractError, RuntimeLoopDecision, RuntimeLoopDecisionDraft, RuntimeLoopTurn,
    RuntimeObservabilityProjection, RuntimeStatus, TOOL_DISPATCH_NEUTRAL_CONTRACT_IDENTITY,
    ToolDispatchCancellationPosture, ToolDispatchFallbackPolicy, ToolDispatchOutputBoundary,
    ToolDispatchPlan, ToolDispatchPlanIssue, ToolDispatchPlanRequest, ToolDispatchPlanStatus,
    ToolDispatchPolicyDraft, ToolDispatchTimeoutPosture, TurnPlan, TurnState,
    apply_adapter_outcome, apply_decision, complete_with_fabric_error, plan_model_dispatch,
    plan_next_turn, plan_tool_dispatch, project_run_report_observability,
    project_runtime_event_observability, run_agent_loop, runtime_capability_family_map,
    validate_runtime_capability_family_map,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_path_preserves_representative_facade_imports() {
        // r[verify agent_harness_runtime.runtime_kernel_split.kernel]
        // r[verify agent_harness_runtime.runtime_kernel_split.compat_facade]
        assert_eq!(
            core::any::type_name::<RuntimeInput>(),
            core::any::type_name::<crate::RuntimeInput>()
        );
        assert_eq!(
            core::any::type_name::<RuntimeConfig>(),
            core::any::type_name::<crate::RuntimeConfig>()
        );
        assert_eq!(
            core::any::type_name::<ToolDispatchPlan>(),
            core::any::type_name::<crate::ToolDispatchPlan>()
        );
        assert_eq!(
            core::any::type_name::<ModelDispatchPlan>(),
            core::any::type_name::<crate::ModelDispatchPlan>()
        );
        assert_eq!(
            core::any::type_name::<RuntimeCapabilityRegistry>(),
            core::any::type_name::<crate::RuntimeCapabilityRegistry>()
        );
        assert_eq!(
            core::any::type_name::<RuntimeEvent>(),
            core::any::type_name::<crate::RuntimeEvent>()
        );
    }
}
