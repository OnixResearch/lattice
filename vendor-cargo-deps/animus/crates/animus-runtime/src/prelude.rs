/// Runtime loop composition prelude.
pub mod loop_prelude {
    pub use crate::AdapterOutcome;
    pub use crate::AdapterRequest;
    pub use crate::AdapterStep;
    pub use crate::DecisionStep;
    pub use crate::LoopDecisionKind;
    pub use crate::LoopFabric;
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
    pub use crate::RuntimeCapabilityFamily;
    pub use crate::RuntimeCapabilityFamilyMapError;
    pub use crate::RuntimeCapabilityFamilyRecord;
    pub use crate::RuntimeConfig;
    pub use crate::RuntimeError;
    pub use crate::RuntimeFailure;
    pub use crate::RuntimeInput;
    pub use crate::RuntimeLoopContractError;
    pub use crate::RuntimeLoopDecision;
    pub use crate::RuntimeLoopDecisionDraft;
    pub use crate::RuntimeLoopTurn;
    pub use crate::RuntimeObservabilityProjection;
    pub use crate::RuntimeStatus;
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
}

/// Tool and capability composition prelude.
pub mod tool_prelude {
    pub use animus_adapters::AdapterCallFailureClass;
    pub use animus_adapters::AdapterCallFailureMetadata;
    pub use animus_adapters::AdapterCallFailureMetadataDraft;
    pub use animus_adapters::AdapterCallOutcome;
    pub use animus_adapters::CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY;
    pub use animus_adapters::CANONICAL_MODEL_RESPONSE_CONTRACT_IDENTITY;
    pub use animus_adapters::CanonicalMessageContentPart;
    pub use animus_adapters::CanonicalMessageContentPartDraft;
    pub use animus_adapters::CanonicalModelContractError;
    pub use animus_adapters::CanonicalModelFinishReason;
    pub use animus_adapters::CanonicalModelMessage;
    pub use animus_adapters::CanonicalModelMessageDraft;
    pub use animus_adapters::CanonicalModelOutputControls;
    pub use animus_adapters::CanonicalModelOutputControlsDraft;
    pub use animus_adapters::CanonicalModelReasoningEffort;
    pub use animus_adapters::CanonicalModelReasoningRequest;
    pub use animus_adapters::CanonicalModelReasoningRequestDraft;
    pub use animus_adapters::CanonicalModelRequest;
    pub use animus_adapters::CanonicalModelRequestDraft;
    pub use animus_adapters::CanonicalModelResponseEnvelope;
    pub use animus_adapters::CanonicalModelResponseEnvelopeDraft;
    pub use animus_adapters::CanonicalModelResponseFormatIntent;
    pub use animus_adapters::CanonicalModelResponseFormatIntentDraft;
    pub use animus_adapters::CanonicalModelResponseItem;
    pub use animus_adapters::CanonicalModelResponseItemDraft;
    pub use animus_adapters::CanonicalModelRole;
    pub use animus_adapters::CanonicalModelSafeMetadata;
    pub use animus_adapters::CanonicalModelSafeMetadataDraft;
    pub use animus_adapters::CanonicalModelToolDeclaration;
    pub use animus_adapters::CanonicalModelToolDeclarationDraft;
    pub use animus_adapters::CanonicalModelUnsafeTextReason;
    pub use animus_adapters::CanonicalModelUsage;
    pub use animus_adapters::CanonicalModelUsageDraft;
    pub use animus_adapters::ComponentAdapter;
    pub use animus_adapters::ComponentOutcomeAdapter;
    pub use animus_adapters::ModelAdapter;
    pub use animus_adapters::ModelOutcomeAdapter;
    pub use animus_adapters::ModelRequest;
    pub use animus_adapters::ModelResponse;
    pub use animus_adapters::OutcomeKind;
    pub use animus_adapters::ToolAdapter;
    pub use animus_adapters::ToolOutcomeAdapter;
    pub use animus_adapters::ToolRequest;
    pub use animus_adapters::ToolResponse;
    pub use animus_adapters::tool::ApprovalPosture;
    pub use animus_adapters::tool::ArgumentFieldDraft;
    pub use animus_adapters::tool::ArgumentPreparationRequest;
    pub use animus_adapters::tool::OutputPolicy;
    pub use animus_adapters::tool::PreparedArguments;
    pub use animus_adapters::tool::RequestDraft;

    pub use crate::DEFAULT_MODEL_CAPABILITY_LABEL;
    pub use crate::DEFAULT_TOOL_CAPABILITY_LABEL;
    pub use crate::RuntimeAdapters;
    pub use crate::RuntimeCapabilityDescriptor;
    pub use crate::RuntimeCapabilityDescriptorDraft;
    pub use crate::RuntimeCapabilityError;
    pub use crate::RuntimeCapabilityErrorKind;
    pub use crate::RuntimeCapabilityOutcome;
    pub use crate::RuntimeCapabilityRegistry;
    pub use crate::RuntimeCapabilityResolution;
    pub use crate::RuntimeCapabilityServiceKind;
    pub use crate::ToolDispatchCancellationPosture;
    pub use crate::ToolDispatchFallbackPolicy;
    pub use crate::ToolDispatchOutputBoundary;
    pub use crate::ToolDispatchPlan;
    pub use crate::ToolDispatchPlanIssue;
    pub use crate::ToolDispatchPlanRequest;
    pub use crate::ToolDispatchPlanStatus;
    pub use crate::ToolDispatchPolicyDraft;
    pub use crate::ToolDispatchTimeoutPosture;
    pub use crate::plan_tool_dispatch;
}

/// Headless protocol composition prelude.
#[cfg(any(feature = "compat", feature = "headless"))]
pub mod headless_prelude {
    pub use crate::AutomationEnvelope;
    pub use crate::AutomationError;
    pub use crate::AutomationEventDraft;
    pub use crate::AutomationEventKind;
    pub use crate::CompletionReceipt;
    pub use crate::CompletionReceiptDraft;
    pub use crate::CompletionStatus;
    pub use crate::EventPayloadSummary;
    pub use crate::EventProjection;
    pub use crate::EventRedaction;
    pub use crate::EventSequence;
    pub use crate::FinalOutput;
    pub use crate::HeadlessCommand;
    pub use crate::HeadlessCommandDraft;
    pub use crate::HeadlessCommandKind;
    pub use crate::HeadlessProtocolError;
    pub use crate::HeadlessProtocolErrorKind;
    pub use crate::HeadlessProtocolEvent;
    pub use crate::HeadlessProtocolEventClass;
    pub use crate::HeadlessQueueing;
    pub use crate::HeadlessRecordBatch;
    pub use crate::HeadlessResponse;
    pub use crate::HeadlessResponseStatus;
    pub use crate::HeadlessRuntimePhase;
    pub use crate::NoninteractiveBehavior;
    pub use crate::OutputPreference;
    pub use crate::RunIdentity;
    pub use crate::RunRequest;
    pub use crate::RunRequestDraft;
    pub use crate::RuntimeThreadController;
    pub use crate::SandboxProfile;
    pub use crate::StructuredOutputDraft;
    pub use crate::StructuredOutputFailure;
    pub use crate::StructuredOutputReport;
    pub use crate::StructuredOutputStatus;
    pub use crate::StructuredSchemaDraft;
    pub use crate::ThreadResume;
    pub use crate::ThreadResumeDraft;
    pub use crate::ThreadSnapshot;
    pub use crate::ThreadStatus;
    pub use crate::acknowledge_headless_command;
    pub use crate::build_automation_stream;
    pub use crate::map_runtime_events_to_headless;
    pub use crate::parse_headless_command_record;
    pub use crate::project_event;
    pub use crate::prompt_required_completion;
    pub use crate::select_final_output;
    pub use crate::split_lf_records;
    pub use crate::summarize_event_payload;
    pub use crate::validate_structured_output;
    pub use crate::validate_thread_cancellation;
    pub use crate::validate_thread_resume;
}

/// Provenance composition prelude.
#[cfg(any(feature = "compat", feature = "provenance"))]
pub mod provenance_prelude {
    pub use crate::ProvenanceActor;
    pub use crate::ProvenanceArtifact;
    pub use crate::ProvenanceArtifactDraft;
    pub use crate::ProvenanceArtifactKind;
    pub use crate::ProvenanceAuthorizationStatus;
    pub use crate::ProvenanceBackendRequirement;
    pub use crate::ProvenanceBranchReport;
    pub use crate::ProvenanceContentObservation;
    pub use crate::ProvenanceContentObservationDraft;
    pub use crate::ProvenanceDiffHunk;
    pub use crate::ProvenanceError;
    pub use crate::ProvenanceErrorKind;
    pub use crate::ProvenanceFact;
    pub use crate::ProvenanceFactArtifact;
    pub use crate::ProvenanceFactAuthorization;
    pub use crate::ProvenanceFactError;
    pub use crate::ProvenanceFactModelActivity;
    pub use crate::ProvenanceFactObservation;
    pub use crate::ProvenanceFactQuery;
    pub use crate::ProvenanceFactRuntimeBlock;
    pub use crate::ProvenanceFactToolActivity;
    pub use crate::ProvenanceFileIdentity;
    pub use crate::ProvenanceFileIdentityDraft;
    pub use crate::ProvenanceGitAiAuthorshipLog;
    pub use crate::ProvenanceGitAiAuthorshipLogDraft;
    pub use crate::ProvenanceGitAiFileAttestation;
    pub use crate::ProvenanceGitAiFileAttestationDraft;
    pub use crate::ProvenanceGitAiMetadata;
    pub use crate::ProvenanceGitAiMetadataDraft;
    pub use crate::ProvenanceGraphEntry;
    pub use crate::ProvenanceLedgerEntry;
    pub use crate::ProvenanceLineRange;
    pub use crate::ProvenanceMessageEntry;
    pub use crate::ProvenanceMutationKind;
    pub use crate::ProvenanceOpaqueFact;
    pub use crate::ProvenanceRangeTransform;
    pub use crate::ProvenanceReceipt;
    pub use crate::ProvenanceReceiptDraft;
    pub use crate::ProvenanceRedactionClass;
    pub use crate::ProvenanceReplay;
    pub use crate::ProvenanceReplayMessage;
    pub use crate::ProvenanceReplayMetadata;
    pub use crate::ProvenanceReplayRole;
    pub use crate::ProvenanceSafeField;
    pub use crate::ProvenanceSummaryEntry;
    pub use crate::ProvenanceToolCheckpoint;
    pub use crate::ProvenanceToolCheckpointDraft;
    pub use crate::ProvenanceToolCheckpointReceipt;
    pub use crate::ProvenanceToolEffect;
    pub use crate::ProvenanceToolEffectDraft;
    pub use crate::ProvenanceToolEffectStatus;
    pub use crate::ProvenanceToolPolicy;
    pub use crate::ProvenanceToolPolicyDraft;
    pub use crate::ProvenanceUnsupportedEntry;
    pub use crate::ProvenanceUsageEntry;
    pub use crate::ToolpathExportPolicy;
    pub use crate::ToolpathProjection;
    pub use crate::ToolpathStep;
    pub use crate::VersionControlProvenanceBackend;
    pub use crate::classify_provenance_tool_policy;
    pub use crate::compress_line_ranges;
    pub use crate::export_toolpath_projection;
    pub use crate::query_provenance_facts;
    pub use crate::record_provenance_checkpoint;
    pub use crate::remove_line_ranges;
    pub use crate::replay_provenance_ledger;
    pub use crate::transform_line_ranges;
    pub use crate::validate_provenance_graph;
    pub use crate::validate_vcs_neutral_field_names;
}

/// Worker-team delegation composition prelude.
#[cfg(any(feature = "compat", feature = "worker-team"))]
pub mod worker_prelude {
    pub use crate::Claim;
    pub use crate::Conflict;
    pub use crate::ConflictReport;
    pub use crate::ContextAdmissionPolicy;
    pub use crate::DelegationFailure;
    pub use crate::EvidenceReference;
    pub use crate::FailureKind;
    pub use crate::MutationIntent;
    pub use crate::ParentTrace;
    pub use crate::Report;
    pub use crate::ReportMergeOutcome;
    pub use crate::WorkerAuthority;
    pub use crate::WorkerBudgets;
    pub use crate::WorkerCancelOutcome;
    pub use crate::WorkerCapability;
    pub use crate::WorkerContextItem;
    pub use crate::WorkerContextItemKind;
    pub use crate::WorkerContextPacket;
    pub use crate::WorkerContextRedaction;
    pub use crate::WorkerHandle;
    pub use crate::WorkerMergedReport;
    pub use crate::WorkerOutputSchema;
    pub use crate::WorkerScope;
    pub use crate::WorkerService;
    pub use crate::WorkerStatus;
    pub use crate::WorkerTask;
    pub use crate::WorkerTaskDraft;
    pub use crate::collect_report;
    pub use crate::enqueue_task;
    pub use crate::merge_reports;
    pub use crate::request_cancel;
    pub use crate::request_status;
    pub use crate::validate_context_packet;
    pub use crate::validate_report;
    pub use crate::validate_task;
}

/// Policy and middleware composition prelude.
#[cfg(any(feature = "compat", feature = "policy"))]
pub mod policy_prelude {
    pub use crate::HookAction;
    pub use crate::HookBoundary;
    pub use crate::HookDecision;
    pub use crate::LifecycleHookCapability;
    pub use crate::LifecycleHookDefinition;
    pub use crate::LifecycleHookDefinitionDraft;
    pub use crate::LifecycleHookDefinitionSource;
    pub use crate::LifecycleHookEffect;
    pub use crate::LifecycleHookEffectDraft;
    pub use crate::LifecycleHookError;
    pub use crate::LifecycleHookErrorKind;
    pub use crate::LifecycleHookEventKind;
    pub use crate::LifecycleHookFailureClass;
    pub use crate::LifecycleHookInput;
    pub use crate::LifecycleHookInputDraft;
    pub use crate::LifecycleHookInputReportDraft;
    pub use crate::LifecycleHookOutcome;
    pub use crate::LifecycleHookOutcomeClass;
    pub use crate::LifecycleHookOutcomeDraft;
    pub use crate::LifecycleHookOutcomeReportDraft;
    pub use crate::LifecycleHookOutput;
    pub use crate::LifecycleHookOutputDraft;
    pub use crate::LifecycleHookPolicyStatus;
    pub use crate::LifecycleHookRedaction;
    pub use crate::LifecycleHookRequest;
    pub use crate::LifecycleHookRoutePlan;
    pub use crate::LifecycleHookTrustDecision;
    pub use crate::LifecycleHookTrustRecord;
    pub use crate::LifecycleHookTrustRecordDraft;
    pub use crate::LifecycleHookTrustScope;
    pub use crate::LifecycleHookTrustStatus;
    pub use crate::MiddlewareError;
    pub use crate::RuntimeMiddleware;
    pub use crate::RuntimeMiddlewareStack;
    pub use crate::TurnHookEffect;
    pub use crate::TurnHookError;
    pub use crate::TurnHookErrorKind;
    pub use crate::TurnHookEvent;
    pub use crate::TurnHookOutcome;
    pub use crate::TurnHookState;
    pub use crate::TurnHookTransition;
    pub use crate::advance_turn_hooks;
    pub use crate::bind_lifecycle_hook_output_to_policy;
    pub use crate::evaluate_lifecycle_hook_trust;
    pub use crate::route_lifecycle_hook_requests;
}

/// Semantic status-selection composition prelude.
#[cfg(any(feature = "compat", feature = "status"))]
pub mod status_prelude {
    pub use animus_adapters::AgentStatusObservation;
    pub use animus_adapters::AgentStatusObservationDraft;
    pub use animus_adapters::AgentStatusObservationKind;
    pub use animus_adapters::AgentStatusPresentation;
    pub use animus_adapters::AgentStatusPresentationDraft;
    pub use animus_adapters::HostStatusObservationBatch;
    pub use animus_adapters::HostStatusRequest;
    pub use animus_adapters::SemanticAgentState;
    pub use animus_adapters::StatusSequence;
    pub use animus_adapters::StatusTtl;

    pub use crate::AgentStatusSelection;
    pub use crate::RejectedStatusCandidate;
    pub use crate::RejectedStatusReason;
    pub use crate::StatusAuthoritySummary;
    pub use crate::StatusDecisionExplanation;
    pub use crate::StatusEvidenceError;
    pub use crate::StatusEvidenceSummary;
    pub use crate::StatusFallbackReason;
    pub use crate::StatusFreshness;
    pub use crate::select_agent_status;
    pub use crate::validate_status_evidence_summary;
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
        let turn =
            super::loop_prelude::RuntimeLoopTurn::new(input.summary(), None, None).expect("turn should validate");
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
        assert!(registry.contains_service(super::tool_prelude::RuntimeCapabilityServiceKind::Model));
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
        let status_request =
            super::status_prelude::HostStatusRequest::new("agent").expect("status request should validate");

        assert!(matches!(decision, super::loop_prelude::RuntimeLoopContractError::CurrentShapeRequired { .. }));
        assert_eq!(registry.kind(), super::tool_prelude::RuntimeCapabilityErrorKind::DuplicateLabel);
        assert!(hook.is_empty());
        assert_eq!(status_request.target_id(), "agent");
    }

    fn tool_request() -> super::tool_prelude::ToolRequest {
        let fields = [super::tool_prelude::ArgumentFieldDraft {
            name: Some(TOOL_INPUT),
            value: Some(TOOL_ARGUMENT_VALUE),
        }];
        let arguments =
            super::tool_prelude::PreparedArguments::prepare(&super::tool_prelude::ArgumentPreparationRequest {
                current_schema_identity: TOOL_SCHEMA,
                allowed_field_names: &[TOOL_INPUT],
                compatibility_rules: &[],
                fields: &fields,
            })
            .expect("arguments should validate");
        assert_eq!(arguments.schema_identity(), TOOL_SCHEMA);
        assert_eq!(arguments.fields().len(), TOOL_ARGUMENT_COUNT);
        let request = super::tool_prelude::ToolRequest::from_draft(&super::tool_prelude::RequestDraft {
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
