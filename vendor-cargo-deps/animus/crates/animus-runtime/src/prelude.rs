/// Runtime loop composition prelude.
pub mod loop_prelude {
    pub use crate::{
        AdapterOutcome, AdapterRequest, AdapterStep, DecisionStep, LoopDecisionKind, LoopFabric,
        ModelDispatchCancellationPosture, ModelDispatchFallbackPolicy, ModelDispatchPlan,
        ModelDispatchPlanIssue, ModelDispatchPlanRequest, ModelDispatchPlanStatus,
        ModelDispatchPolicyDraft, ModelDispatchReceiptPolicy, ModelDispatchStreamPolicy,
        ModelDispatchTimeoutPosture, RunReport, RuntimeCapabilityFamily,
        RuntimeCapabilityFamilyMapError, RuntimeCapabilityFamilyRecord, RuntimeConfig,
        RuntimeError, RuntimeFailure, RuntimeInput, RuntimeLoopContractError, RuntimeLoopDecision,
        RuntimeLoopDecisionDraft, RuntimeLoopTurn, RuntimeObservabilityProjection, RuntimeStatus,
        ToolDispatchCancellationPosture, ToolDispatchFallbackPolicy, ToolDispatchOutputBoundary,
        ToolDispatchPlan, ToolDispatchPlanIssue, ToolDispatchPlanRequest, ToolDispatchPlanStatus,
        ToolDispatchPolicyDraft, ToolDispatchTimeoutPosture, TurnPlan, TurnState,
        apply_adapter_outcome, apply_decision, complete_with_fabric_error, plan_model_dispatch,
        plan_next_turn, plan_tool_dispatch, project_run_report_observability,
        project_runtime_event_observability, run_agent_loop, runtime_capability_family_map,
        validate_runtime_capability_family_map,
    };
}

/// Tool and capability composition prelude.
pub mod tool_prelude {
    pub use crate::{
        DEFAULT_MODEL_CAPABILITY_LABEL, DEFAULT_TOOL_CAPABILITY_LABEL, RuntimeAdapters,
        RuntimeCapabilityDescriptor, RuntimeCapabilityDescriptorDraft, RuntimeCapabilityError,
        RuntimeCapabilityErrorKind, RuntimeCapabilityOutcome, RuntimeCapabilityRegistry,
        RuntimeCapabilityResolution, RuntimeCapabilityServiceKind, ToolDispatchCancellationPosture,
        ToolDispatchFallbackPolicy, ToolDispatchOutputBoundary, ToolDispatchPlan,
        ToolDispatchPlanIssue, ToolDispatchPlanRequest, ToolDispatchPlanStatus,
        ToolDispatchPolicyDraft, ToolDispatchTimeoutPosture, plan_tool_dispatch,
    };
    pub use animus_adapters::tool::{
        ApprovalPosture, ArgumentFieldDraft, ArgumentPreparationRequest, OutputPolicy,
        PreparedArguments, RequestDraft,
    };
    pub use animus_adapters::{
        AdapterCallFailureClass, AdapterCallFailureMetadata, AdapterCallFailureMetadataDraft,
        AdapterCallOutcome, CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
        CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY, CanonicalMessageContentPart,
        CanonicalMessageContentPartDraft, CanonicalModelContractError, CanonicalModelFinishReason,
        CanonicalModelMessage, CanonicalModelMessageDraft, CanonicalModelOutputControls,
        CanonicalModelOutputControlsDraft, CanonicalModelReasoningEffort,
        CanonicalModelReasoningRequest, CanonicalModelReasoningRequestDraft, CanonicalModelRequest,
        CanonicalModelRequestDraft, CanonicalModelResponseEnvelope,
        CanonicalModelResponseEnvelopeDraft, CanonicalModelResponseFormatIntent,
        CanonicalModelResponseFormatIntentDraft, CanonicalModelResponseItem,
        CanonicalModelResponseItemDraft, CanonicalModelRole, CanonicalModelSafeMetadata,
        CanonicalModelSafeMetadataDraft, CanonicalModelToolDeclaration,
        CanonicalModelToolDeclarationDraft, CanonicalModelUnsafeTextReason, CanonicalModelUsage,
        CanonicalModelUsageDraft, ComponentAdapter, ComponentOutcomeAdapter, ModelAdapter,
        ModelOutcomeAdapter, ModelRequest, ModelResponse, OutcomeKind, ToolAdapter,
        ToolOutcomeAdapter, ToolRequest, ToolResponse,
    };
}

/// Headless protocol composition prelude.
#[cfg(any(feature = "compat", feature = "headless"))]
pub mod headless_prelude {
    pub use crate::{
        AutomationEnvelope, AutomationError, AutomationEventDraft, AutomationEventKind,
        CompletionReceipt, CompletionReceiptDraft, CompletionStatus, EventPayloadSummary,
        EventProjection, EventRedaction, EventSequence, FinalOutput, HeadlessCommand,
        HeadlessCommandDraft, HeadlessCommandKind, HeadlessProtocolError,
        HeadlessProtocolErrorKind, HeadlessProtocolEvent, HeadlessProtocolEventClass,
        HeadlessQueueing, HeadlessRecordBatch, HeadlessResponse, HeadlessResponseStatus,
        HeadlessRuntimePhase, NoninteractiveBehavior, OutputPreference, RunIdentity, RunRequest,
        RunRequestDraft, RuntimeThreadController, SandboxProfile, StructuredOutputDraft,
        StructuredOutputFailure, StructuredOutputReport, StructuredOutputStatus,
        StructuredSchemaDraft, ThreadResume, ThreadResumeDraft, ThreadSnapshot, ThreadStatus,
        acknowledge_headless_command, build_automation_stream, map_runtime_events_to_headless,
        parse_headless_command_record, project_event, prompt_required_completion,
        select_final_output, split_lf_records, summarize_event_payload, validate_structured_output,
        validate_thread_cancellation, validate_thread_resume,
    };
}

/// Provenance composition prelude.
#[cfg(any(feature = "compat", feature = "provenance"))]
pub mod provenance_prelude {
    pub use crate::{
        ProvenanceActor, ProvenanceArtifact, ProvenanceArtifactDraft, ProvenanceArtifactKind,
        ProvenanceAuthorizationStatus, ProvenanceBackendRequirement, ProvenanceBranchReport,
        ProvenanceContentObservation, ProvenanceContentObservationDraft, ProvenanceDiffHunk,
        ProvenanceError, ProvenanceErrorKind, ProvenanceFact, ProvenanceFactArtifact,
        ProvenanceFactAuthorization, ProvenanceFactError, ProvenanceFactModelActivity,
        ProvenanceFactObservation, ProvenanceFactQuery, ProvenanceFactRuntimeBlock,
        ProvenanceFactToolActivity, ProvenanceFileIdentity, ProvenanceFileIdentityDraft,
        ProvenanceGitAiAuthorshipLog, ProvenanceGitAiAuthorshipLogDraft,
        ProvenanceGitAiFileAttestation, ProvenanceGitAiFileAttestationDraft,
        ProvenanceGitAiMetadata, ProvenanceGitAiMetadataDraft, ProvenanceGraphEntry,
        ProvenanceLedgerEntry, ProvenanceLineRange, ProvenanceMessageEntry, ProvenanceMutationKind,
        ProvenanceOpaqueFact, ProvenanceRangeTransform, ProvenanceReceipt, ProvenanceReceiptDraft,
        ProvenanceRedactionClass, ProvenanceReplay, ProvenanceReplayMessage,
        ProvenanceReplayMetadata, ProvenanceReplayRole, ProvenanceSafeField,
        ProvenanceSummaryEntry, ProvenanceToolCheckpoint, ProvenanceToolCheckpointDraft,
        ProvenanceToolCheckpointReceipt, ProvenanceToolEffect, ProvenanceToolEffectDraft,
        ProvenanceToolEffectStatus, ProvenanceToolPolicy, ProvenanceToolPolicyDraft,
        ProvenanceUnsupportedEntry, ProvenanceUsageEntry, ToolpathExportPolicy, ToolpathProjection,
        ToolpathStep, VersionControlProvenanceBackend, classify_provenance_tool_policy,
        compress_line_ranges, export_toolpath_projection, query_provenance_facts,
        record_provenance_checkpoint, remove_line_ranges, replay_provenance_ledger,
        transform_line_ranges, validate_provenance_graph, validate_vcs_neutral_field_names,
    };
}

/// Worker-team delegation composition prelude.
#[cfg(any(feature = "compat", feature = "worker-team"))]
pub mod worker_prelude {
    pub use crate::{
        Claim, Conflict, ConflictReport, ContextAdmissionPolicy, DelegationFailure,
        EvidenceReference, FailureKind, MutationIntent, ParentTrace, Report, ReportMergeOutcome,
        WorkerAuthority, WorkerBudgets, WorkerCancelOutcome, WorkerCapability, WorkerContextItem,
        WorkerContextItemKind, WorkerContextPacket, WorkerContextRedaction, WorkerHandle,
        WorkerMergedReport, WorkerOutputSchema, WorkerScope, WorkerService, WorkerStatus,
        WorkerTask, WorkerTaskDraft, collect_report, enqueue_task, merge_reports, request_cancel,
        request_status, validate_context_packet, validate_report, validate_task,
    };
}

/// Policy and middleware composition prelude.
#[cfg(any(feature = "compat", feature = "policy"))]
pub mod policy_prelude {
    pub use crate::{
        HookAction, HookBoundary, HookDecision, LifecycleHookCapability, LifecycleHookDefinition,
        LifecycleHookDefinitionDraft, LifecycleHookDefinitionSource, LifecycleHookEffect,
        LifecycleHookEffectDraft, LifecycleHookError, LifecycleHookErrorKind,
        LifecycleHookEventKind, LifecycleHookFailureClass, LifecycleHookInput,
        LifecycleHookInputDraft, LifecycleHookInputReportDraft, LifecycleHookOutcome,
        LifecycleHookOutcomeClass, LifecycleHookOutcomeDraft, LifecycleHookOutcomeReportDraft,
        LifecycleHookOutput, LifecycleHookOutputDraft, LifecycleHookPolicyStatus,
        LifecycleHookRedaction, LifecycleHookRequest, LifecycleHookRoutePlan,
        LifecycleHookTrustDecision, LifecycleHookTrustRecord, LifecycleHookTrustRecordDraft,
        LifecycleHookTrustScope, LifecycleHookTrustStatus, MiddlewareError, RuntimeMiddleware,
        RuntimeMiddlewareStack, TurnHookEffect, TurnHookError, TurnHookErrorKind, TurnHookEvent,
        TurnHookOutcome, TurnHookState, TurnHookTransition, advance_turn_hooks,
        bind_lifecycle_hook_output_to_policy, evaluate_lifecycle_hook_trust,
        route_lifecycle_hook_requests,
    };
}

/// Semantic status-selection composition prelude.
#[cfg(any(feature = "compat", feature = "status"))]
pub mod status_prelude {
    pub use crate::{
        AgentStatusSelection, RejectedStatusCandidate, RejectedStatusReason,
        StatusAuthoritySummary, StatusDecisionExplanation, StatusEvidenceError,
        StatusEvidenceSummary, StatusFallbackReason, StatusFreshness, select_agent_status,
        validate_status_evidence_summary,
    };
    pub use animus_adapters::{
        AgentStatusObservation, AgentStatusObservationDraft, AgentStatusObservationKind,
        AgentStatusPresentation, AgentStatusPresentationDraft, HostStatusObservationBatch,
        HostStatusRequest, SemanticAgentState, StatusSequence, StatusTtl,
    };
}

#[cfg(all(test, feature = "compat"))]
mod tests {
    const SUMMARY: &str = "intent: compose runtime";
    const TOOL_LABEL: &str = "search";
    const TOOL_INPUT: &str = "query";
    const TOOL_SCHEMA: &str = "schema.search.v1";
    const TOOL_ARGUMENT_VALUE: &str = "adapter contracts";
    const TOOL_CALL_ID: &str = "tool-call-1";
    const TOOL_VISIBLE_BUDGET: u32 = 64;
    const TOOL_ARGUMENT_COUNT: usize = 1;
    const TOOL_MUTATION_TARGET: &str = "resource:file:README";
    const TOOL_CAPABILITY: &str = "tool.read";

    #[test]
    fn domain_preludes_construct_representative_runtime_dtos() {
        let input = super::loop_prelude::RuntimeInput::new(SUMMARY).expect("input should validate");
        let turn = super::loop_prelude::RuntimeLoopTurn::new(input.summary(), None, None)
            .expect("turn should validate");
        let decision = super::loop_prelude::RuntimeLoopDecision::from_draft(
            super::loop_prelude::RuntimeLoopDecisionDraft::respond(SUMMARY),
        )
        .expect("decision should validate");
        let tool = tool_request();
        let registry = super::tool_prelude::RuntimeCapabilityRegistry::compatibility_model_tool()
            .expect("compat registry should validate");

        assert_eq!(turn.summary(), SUMMARY);
        assert_eq!(decision.kind(), "respond");
        assert_eq!(tool.name(), TOOL_LABEL);
        assert_eq!(tool.schema_identity(), TOOL_SCHEMA);
        assert!(
            registry.contains_service(super::tool_prelude::RuntimeCapabilityServiceKind::Model)
        );
    }

    #[test]
    fn domain_preludes_reject_invalid_boundary_values() {
        let decision = super::loop_prelude::RuntimeLoopDecision::from_draft(
            super::loop_prelude::RuntimeLoopDecisionDraft::tool(TOOL_LABEL, TOOL_INPUT),
        )
        .expect_err("legacy raw tool should fail");
        let registry = super::tool_prelude::RuntimeCapabilityRegistry::from_drafts(&[
            super::tool_prelude::RuntimeCapabilityDescriptorDraft::default_model(),
            super::tool_prelude::RuntimeCapabilityDescriptorDraft::default_model(),
        ])
        .expect_err("duplicate model capability should fail");
        let hook = super::policy_prelude::RuntimeMiddlewareStack::empty();
        let status_request = super::status_prelude::HostStatusRequest::new("agent")
            .expect("status request should validate");

        assert!(matches!(
            decision,
            super::loop_prelude::RuntimeLoopContractError::CurrentShapeRequired { .. }
        ));
        assert_eq!(
            registry.kind(),
            super::tool_prelude::RuntimeCapabilityErrorKind::DuplicateLabel
        );
        assert!(hook.is_empty());
        assert_eq!(status_request.target_id(), "agent");
    }

    fn tool_request() -> super::tool_prelude::ToolRequest {
        let fields = [super::tool_prelude::ArgumentFieldDraft {
            name: Some(TOOL_INPUT),
            value: Some(TOOL_ARGUMENT_VALUE),
        }];
        let arguments = super::tool_prelude::PreparedArguments::prepare(
            &super::tool_prelude::ArgumentPreparationRequest {
                current_schema_identity: TOOL_SCHEMA,
                allowed_field_names: &[TOOL_INPUT],
                compatibility_rules: &[],
                fields: &fields,
            },
        )
        .expect("arguments should validate");
        assert_eq!(arguments.schema_identity(), TOOL_SCHEMA);
        assert_eq!(arguments.fields().len(), TOOL_ARGUMENT_COUNT);
        let request =
            super::tool_prelude::ToolRequest::from_draft(&super::tool_prelude::RequestDraft {
                name: Some(TOOL_LABEL),
                arguments: Some(&arguments),
                correlation_id: Some(TOOL_CALL_ID),
                output_policy: super::tool_prelude::OutputPolicy {
                    visible_char_budget: TOOL_VISIBLE_BUDGET,
                },
                mutation_targets: &[TOOL_MUTATION_TARGET],
                required_capability_labels: &[TOOL_CAPABILITY],
                approval_posture: super::tool_prelude::ApprovalPosture::PreApproved.as_str(),
            })
            .expect("tool request should validate");
        assert_eq!(request.name(), TOOL_LABEL);
        assert_eq!(request.correlation_id(), TOOL_CALL_ID);
        request
    }
}
