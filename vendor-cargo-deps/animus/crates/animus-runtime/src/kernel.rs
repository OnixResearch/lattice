//! Minimal runtime kernel boundary.
//!
//! This module is the direct import path for embedders that only need typed
//! runtime input/config, neutral loop orchestration, turn state, runtime events,
//! capability resolution, model/tool dispatch planning, and injected adapter
//! calls. Optional domains such as Steel, headless protocol, provenance,
//! worker-team delegation, batch, compaction, dataset export, research/eval,
//! TUI, std shell, provider clients, concrete storage, and host authority stay
//! outside this kernel import path.

pub use crate::AdapterOutcome;
pub use crate::AdapterRequest;
pub use crate::AdapterStep;
pub use crate::DEFAULT_MODEL_CAPABILITY_LABEL;
pub use crate::DEFAULT_TOOL_CAPABILITY_LABEL;
pub use crate::DecisionStep;
pub use crate::LoopDecisionKind;
pub use crate::LoopFabric;
pub use crate::MODEL_PROVIDER_NEUTRAL_CONTRACT_IDENTITY;
pub use crate::ModelDispatchCancellationPosture;
pub use crate::ModelDispatchFallbackPolicy;
pub use crate::ModelDispatchPlan;
pub use crate::ModelDispatchPlanIssue;
pub use crate::ModelDispatchPlanRequest;
pub use crate::ModelDispatchPlanStatus;
pub use crate::ModelDispatchPolicyDraft;
pub use crate::ModelDispatchReceiptPolicy;
pub use crate::ModelDispatchStreamPolicy;
pub use crate::ModelDispatchTimeoutPosture;
pub use crate::RunReport;
pub use crate::RuntimeAdapters;
pub use crate::RuntimeCapabilityDescriptor;
pub use crate::RuntimeCapabilityDescriptorDraft;
pub use crate::RuntimeCapabilityError;
pub use crate::RuntimeCapabilityErrorKind;
pub use crate::RuntimeCapabilityFamily;
pub use crate::RuntimeCapabilityFamilyMapError;
pub use crate::RuntimeCapabilityFamilyRecord;
pub use crate::RuntimeCapabilityOutcome;
pub use crate::RuntimeCapabilityRegistry;
pub use crate::RuntimeCapabilityResolution;
pub use crate::RuntimeCapabilityServiceKind;
pub use crate::RuntimeConfig;
pub use crate::RuntimeError;
pub use crate::RuntimeEvent;
pub use crate::RuntimeEventKind;
pub use crate::RuntimeFailure;
pub use crate::RuntimeInput;
pub use crate::RuntimeLoopContractError;
pub use crate::RuntimeLoopDecision;
pub use crate::RuntimeLoopDecisionDraft;
pub use crate::RuntimeLoopTurn;
pub use crate::RuntimeObservabilityProjection;
pub use crate::RuntimeStatus;
pub use crate::TOOL_DISPATCH_NEUTRAL_CONTRACT_IDENTITY;
pub use crate::ToolDispatchCancellationPosture;
pub use crate::ToolDispatchFallbackPolicy;
pub use crate::ToolDispatchOutputBoundary;
pub use crate::ToolDispatchPlan;
pub use crate::ToolDispatchPlanIssue;
pub use crate::ToolDispatchPlanRequest;
pub use crate::ToolDispatchPlanStatus;
pub use crate::ToolDispatchPolicyDraft;
pub use crate::ToolDispatchTimeoutPosture;
pub use crate::TurnPlan;
pub use crate::TurnState;
pub use crate::apply_adapter_outcome;
pub use crate::apply_decision;
pub use crate::complete_with_fabric_error;
pub use crate::plan_model_dispatch;
pub use crate::plan_next_turn;
pub use crate::plan_tool_dispatch;
pub use crate::project_run_report_observability;
pub use crate::project_runtime_event_observability;
pub use crate::run_agent_loop;
pub use crate::runtime_capability_family_map;
pub use crate::validate_runtime_capability_family_map;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_path_preserves_representative_facade_imports() {
        // r[verify agent_harness_runtime.runtime_kernel_split.kernel]
        // r[verify agent_harness_runtime.runtime_kernel_split.compat_facade]
        assert_eq!(core::any::type_name::<RuntimeInput>(), core::any::type_name::<crate::RuntimeInput>());
        assert_eq!(core::any::type_name::<RuntimeConfig>(), core::any::type_name::<crate::RuntimeConfig>());
        assert_eq!(core::any::type_name::<ToolDispatchPlan>(), core::any::type_name::<crate::ToolDispatchPlan>());
        assert_eq!(core::any::type_name::<ModelDispatchPlan>(), core::any::type_name::<crate::ModelDispatchPlan>());
        assert_eq!(
            core::any::type_name::<RuntimeCapabilityRegistry>(),
            core::any::type_name::<crate::RuntimeCapabilityRegistry>()
        );
        assert_eq!(core::any::type_name::<RuntimeEvent>(), core::any::type_name::<crate::RuntimeEvent>());
    }
}
