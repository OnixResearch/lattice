//! Public runtime facade exports.

#![allow(unknown_lints, excessive_file_length)]

pub use crate::admission::{
    MODEL_PROVIDER_NEUTRAL_CONTRACT_IDENTITY, ModelDispatchCancellationPosture,
    ModelDispatchFallbackPolicy, ModelDispatchPlan, ModelDispatchPlanIssue,
    ModelDispatchPlanRequest, ModelDispatchPlanStatus, ModelDispatchPolicyDraft,
    ModelDispatchReceiptPolicy, ModelDispatchStreamPolicy, ModelDispatchTimeoutPosture,
    plan_model_dispatch,
};
pub use crate::background::{
    NamedProcessJobProfile, ProcessJobBackend, ProcessJobEnvironmentVariable,
    ProcessJobMetadataField, ProcessJobNotificationPolicy, ProcessJobProfile,
    ProcessJobProfileError, ProcessJobProfileErrorKind, ProcessJobProfileIdentity,
    ProcessJobProfileManifest, ProcessJobProfilePolicy, ProcessJobProfileResolveRequest,
    ProcessJobProfileSource, ProcessJobProfileSourcePrecedence, ProcessJobResourceLimits,
    ResolvedProcessJobProfile, StartProcessJobRequest, resolve_process_job_profile,
};
pub use crate::batch::{
    BatchConcurrencyPolicy, BatchError, BatchExecutionMode, BatchExecutionService, BatchJob,
    BatchJobDraft, BatchJobOutcome, BatchJobOutcomeDraft, BatchJobReceipt, BatchJobReceiptDraft,
    BatchJobStatus, BatchOutputFormat, BatchOutputLabel, BatchOutputRecord, BatchReceipt,
    BatchReceiptDraft, BatchRedactionPolicy, BatchRenderer, BatchRequestDraft, BatchResumeAction,
    BatchResumeDecision, BatchResumePlan, BatchRunIdentity, ResumeCompletedJobDraft,
    ResumeManifestDraft, ResumeStaleReason, evaluate_resume_manifest, execute_batch_plan,
    render_plan_order_records,
};
pub use crate::boundary::{evaluate_sandbox_escalation, sandbox_escalation_event};
pub use crate::capmap::{
    RuntimeCapabilityFamily, RuntimeCapabilityFamilyMapError, RuntimeCapabilityFamilyRecord,
    runtime_capability_family_map, validate_runtime_capability_family_map,
};
pub use crate::caps::{
    DEFAULT_MODEL_CAPABILITY_LABEL, DEFAULT_TOOL_CAPABILITY_LABEL, RuntimeCapabilityDescriptor,
    RuntimeCapabilityDescriptorDraft, RuntimeCapabilityError, RuntimeCapabilityErrorKind,
    RuntimeCapabilityOutcome, RuntimeCapabilityRegistry, RuntimeCapabilityResolution,
    RuntimeCapabilityServiceKind,
};
pub use crate::checkpoint::{
    VcsSnapshotBackendKind, VcsSnapshotConfirmationStatus, VcsSnapshotDiagnostic,
    VcsSnapshotDiagnosticKind, VcsSnapshotError, VcsSnapshotErrorKind, VcsSnapshotFreshness,
    VcsSnapshotIntent, VcsSnapshotOperation, VcsSnapshotPlan, VcsSnapshotPlanRequest,
    VcsSnapshotPlanRequestDraft, VcsSnapshotPlanStatus, VcsSnapshotPlannerInput,
    VcsSnapshotProtectedAction, VcsSnapshotReceipt, VcsSnapshotRecord, VcsSnapshotRecordDraft,
    VcsSnapshotServiceStatus, plan_vcs_snapshot_operation, validate_vcs_snapshot_receipt_text,
};
pub use crate::composition_conformance::{
    CompositionCaseKind, CompositionDiagnostic, CompositionDiagnosticKind,
    CompositionEvidenceRecord, CompositionRailReport, CompositionScenario, CompositionSurface,
    composition_conformance_matrix, render_composition_evidence, run_composition_conformance,
    validate_composition_evidence_text, validate_composition_matrix,
};
pub use crate::context_layering::{
    ContextLayerAdmissionPolicy, ContextLayerBoundaryEffect, ContextLayerClass,
    ContextLayerDescriptor, ContextLayerDiagnostic, ContextLayerDiagnosticKind,
    ContextLayerDiscoveryBoundary, ContextLayerFreshness, ContextLayerInclusionPolicy,
    ContextLayerPlan, ContextLayerPolicySwitch, ContextLayerReceipt, ContextLayerReceiptEntry,
    ContextLayerRedaction, ContextLayerSourceKind, ContextLayerTrust, ContextLayerValidationResult,
    assemble_context_layers, blake3_context_layer_fingerprint, validate_context_descriptor,
};
pub use crate::contract::{
    DEFAULT_MAX_TURNS, MIN_MAX_TURNS, MaxTurns, RunReport, RuntimeConfig, RuntimeError,
    RuntimeFailure, RuntimeInput, RuntimeStatus,
};
pub use crate::controller::{
    CompletionAcknowledgement, ContinuationAction, ContinuationCandidate,
    ContinuationCompletionRecord, ContinuationDecision, ContinuationDispatch, ContinuationError,
    ContinuationPolicy, ContinuationPriority, ContinuationReason, ContinuationReceipt,
    ContinuationReceiptDraft, ContinuationState, ContinuationStateClass, ContinuationWorkKind,
    DispatchAcknowledgement, DispatchAcknowledgementOutcome, PendingContinuationDispatch,
    PendingContinuationWork, select_next_continuation,
};
pub use crate::dataset_export::{
    DatasetChatMessage, DatasetChatRow, DatasetDiagnostic, DatasetDiagnosticKind,
    DatasetExportPolicy, DatasetExportResult, DatasetFingerprint, DatasetInspectionReport,
    DatasetMetadataProjection, DatasetPreferenceExample, DatasetPreferenceRow, DatasetReplayEntry,
    DatasetReplayRole, DatasetRlStep, DatasetRlStepKind, DatasetRlTrajectory,
    DatasetRlTrajectoryRow, DatasetSessionSource, DatasetSkippedRow, DatasetTarget,
    blake3_dataset_export_fingerprint, export_chat_messages_row, export_preference_row,
    export_rl_trajectory_row, inspect_dataset_exports, render_chat_row_jsonl,
    render_chat_rows_jsonl,
};
pub use crate::engine::{
    ApproachRecord, ApproachState, AuditOutcome, GapStrength, PortfolioAudit,
    PortfolioAuditFinding, PortfolioAuditRisk, PortfolioBlocker, PortfolioBlockerClass,
    PortfolioBudget, PortfolioContract, PortfolioDispatch, PortfolioError, PortfolioErrorKind,
    PortfolioFingerprint, PortfolioPhase, PortfolioReceipt, PortfolioResult, PortfolioState,
    PortfolioUsage, PortfolioValidationReceipt, PortfolioValidationStatus, ReopenNovelty,
    begin_adversarial_audit, begin_independent_exploration, begin_synthesis,
    begin_targeted_redirection, begin_validation, blake3_portfolio_fingerprint, block_approach,
    block_portfolio, exhaust_portfolio, initialize_portfolio, is_portfolio_budget_exhausted,
    portfolio_contract_fingerprint, portfolio_evidence_fingerprint, project_portfolio_receipt,
    record_adversarial_audit, record_portfolio_dispatch, register_approach, reopen_approach,
    underexplored_live_families, validate_approach_record, validate_portfolio_contract,
    validate_portfolio_dispatch, validate_portfolio_receipt, validate_portfolio_result,
};
pub use crate::eval_feedback::{
    EvalFeedbackCandidate, EvalFeedbackCandidateKind, EvalFeedbackDiagnostic,
    EvalFeedbackDiagnosticKind, EvalFeedbackFingerprint, EvalFeedbackFreshnessDecision,
    EvalFeedbackFreshnessPolicy, EvalFeedbackOutputFormat, EvalFeedbackPlan, EvalFeedbackReceipt,
    EvalFeedbackReceiptLeak, EvalFeedbackRedactionState, EvalFeedbackReviewDecision,
    EvalFeedbackReviewOutcome, EvalFeedbackRunStatus, EvalFeedbackScopeRequest,
    EvalFeedbackTaskSet, EvalFeedbackTaskSetKind, EvalFeedbackValidationResult,
    EvalFeedbackVerifierIdentity, blake3_eval_feedback_fingerprint,
    classify_eval_feedback_freshness, evaluate_feedback_review, validate_eval_feedback_plan,
    validate_eval_feedback_receipt,
};
pub use crate::fabric::LoopFabric;
pub use crate::fresh_context_tasks::{
    FreshTaskActionKind, FreshTaskBudget, FreshTaskCancellationReason, FreshTaskDescriptor,
    FreshTaskDiagnostic, FreshTaskDiagnosticKind, FreshTaskDispatchDecision, FreshTaskFingerprint,
    FreshTaskInheritance, FreshTaskIsolation, FreshTaskMode, FreshTaskReceipt, FreshTaskResult,
    FreshTaskResultStatus, FreshTaskSchedulerState, FreshTaskValidationResult,
    blake3_fresh_task_fingerprint, evaluate_fresh_task_dispatch, evaluate_side_question_action,
    validate_fresh_task_descriptor, validate_fresh_task_result,
};
pub use crate::headless::{
    AutomationEnvelope, AutomationError, AutomationEventDraft, AutomationEventKind,
    CompletionReceipt, CompletionReceiptDraft, CompletionStatus, EventPayloadSummary,
    EventProjection, EventRedaction, EventSequence, FinalOutput, HeadlessCommand,
    HeadlessCommandDraft, HeadlessCommandKind, HeadlessProtocolError, HeadlessProtocolErrorKind,
    HeadlessProtocolEvent, HeadlessProtocolEventClass, HeadlessQueueing, HeadlessRecordBatch,
    HeadlessResponse, HeadlessResponseStatus, HeadlessRuntimePhase, NoninteractiveBehavior,
    OutputPreference, RunIdentity, RunRequest, RunRequestDraft, RuntimeThreadController,
    SandboxProfile, StructuredOutputDraft, StructuredOutputFailure, StructuredOutputReport,
    StructuredOutputStatus, StructuredSchemaDraft, ThreadResume, ThreadResumeDraft, ThreadSnapshot,
    ThreadStatus, acknowledge_headless_command, build_automation_stream,
    map_runtime_events_to_headless, parse_headless_command_record, project_event,
    prompt_required_completion, select_final_output, split_lf_records, summarize_event_payload,
    validate_structured_output, validate_thread_cancellation, validate_thread_resume,
};
pub use crate::hooks::{
    HookAction, HookBoundary, HookDecision, LifecycleHookCapability, LifecycleHookDefinition,
    LifecycleHookDefinitionDraft, LifecycleHookDefinitionSource, LifecycleHookEffect,
    LifecycleHookEffectDraft, LifecycleHookError, LifecycleHookErrorKind, LifecycleHookEventKind,
    LifecycleHookFailureClass, LifecycleHookInput, LifecycleHookInputDraft,
    LifecycleHookInputReportDraft, LifecycleHookOutcome, LifecycleHookOutcomeClass,
    LifecycleHookOutcomeDraft, LifecycleHookOutcomeReportDraft, LifecycleHookOutput,
    LifecycleHookOutputDraft, LifecycleHookPolicyStatus, LifecycleHookRedaction,
    LifecycleHookRequest, LifecycleHookRoutePlan, LifecycleHookTrustDecision,
    LifecycleHookTrustRecord, LifecycleHookTrustRecordDraft, LifecycleHookTrustScope,
    LifecycleHookTrustStatus, MiddlewareError, RuntimeMiddleware, RuntimeMiddlewareStack,
    TurnHookEffect, TurnHookError, TurnHookErrorKind, TurnHookEvent, TurnHookOutcome,
    TurnHookState, TurnHookTransition, advance_turn_hooks, bind_lifecycle_hook_output_to_policy,
    evaluate_lifecycle_hook_trust, route_lifecycle_hook_requests,
};
pub use crate::mailbox::{
    ActiveWorkReference, ActiveWorkStatus, CancellationHandleClass, CancellationRequest,
    DispatchAcknowledgementDraft, ImmediateDispatchRequest, InterruptKeyword,
    InterruptQueueAdmission, InterruptSendMode, InterruptSignalDraft, MessageBudgetClass,
    MessageDispatch, MessageDispatchAcknowledgementOutcome, MessageEditDraft,
    MessageItemOperationDraft, MessageQueueError, MessageQueueErrorKind, MessageQueueEvent,
    MessageQueueOperation, MessageQueueOperationKind, MessageQueuePolicy, MessageQueuePolicyDraft,
    MessageQueueReceipt, MessageQueueReceiptDraft, MessageQueueState, MessageQueueTransition,
    MessageRedactionState, MessageReorderDraft, MessageSafetyClass, MessageScopeOperationDraft,
    PauseRequest, QueueItemEvent, QueuedMessage, QueuedMessageDraft, RedirectionPayload,
    RedirectionPayloadDraft, UserControlEvent, apply_message_queue_operation,
    blake3_message_queue_hash, classify_interrupt_signal, validate_message_queue_receipt_text,
};
pub use crate::modelprofiles::{
    MODEL_INVOCATION_PLAN_HASH_DOMAIN, MODEL_INVOCATION_RECEIPT_HASH_DOMAIN,
    MODEL_RUNTIME_PROFILE_HASH_DOMAIN, MODEL_RUNTIME_PROFILE_SCHEMA,
    MODEL_RUNTIME_RESOLUTION_HASH_DOMAIN, ModelDialectTransitionPlan,
    ModelDialectTransitionRequest, ModelInvocationAuthority, ModelInvocationCompositionInput,
    ModelInvocationDiagnostic, ModelInvocationIdentitySnapshot, ModelInvocationOmissionClass,
    ModelInvocationPlan, ModelInvocationReceipt, ModelInvocationReceiptLeak,
    ModelInvocationReceiptSensitiveDraft, ModelInvocationSkillReceiptEntry,
    ModelInvocationToolReceiptEntry, ModelInvocationTransitionDecision,
    ModelInvocationTransitionKind, ModelInvocationTransitionRequest, ModelMemoryAccessMode,
    ModelNamedResourceReference, ModelNamedResourceReferenceDraft, ModelProfileAdmissionStatus,
    ModelProfileEnablementPolicy, ModelProfileSourceScope, ModelProfileSpecificity,
    ModelProfileTrustStatus, ModelPromptSourceReference, ModelPromptSourceReferenceDraft,
    ModelResourceCeiling, ModelRuntimeBinding, ModelRuntimeBindingDraft, ModelRuntimeBudgetPolicy,
    ModelRuntimeBudgetPolicyDraft, ModelRuntimeProfile, ModelRuntimeProfileDraft,
    ModelRuntimeProfileError, ModelRuntimeProfileIssue, ModelRuntimeProfileResolution,
    ModelRuntimeProfileSet, ModelRuntimeProfileSetDraft, ModelRuntimeTarget,
    ModelRuntimeTargetDraft, ModelSkillCompositionResource, ModelSkillPlanEntry,
    ModelSkillReference, ModelSkillReferenceDraft, ModelSkillResourcePosture, ModelToolCallBinding,
    ModelToolCallBindingDraft, ModelToolDeclaration, ModelToolset, ModelToolsetDraft,
    ModelTransportDialectAdmissionDraft, NativeToolCallingMode, ProfileBoundModelDispatchPlan,
    ResolvedModelRuntimeProfile, bind_model_dispatch_plan, blake3_model_runtime_profile_identity,
    compose_model_invocation_plan, evaluate_model_dialect_transition,
    evaluate_model_invocation_transition, project_model_invocation_receipt,
    resolve_model_runtime_profile, select_steel_skill_modules, validate_model_tool_call_binding,
};
pub use crate::orchestration::{
    KanbanBlockRecord, KanbanBoardService, KanbanComment, KanbanCompletion, KanbanContextAssembly,
    KanbanContextHandoff, KanbanContextItem, KanbanError, KanbanErrorKind, KanbanHeartbeat,
    KanbanMutationIntent, KanbanPriority, KanbanPromotionDecision, KanbanPromotionOutcome,
    KanbanQuarantinedHandoff, KanbanTask, KanbanTaskDraft, KanbanTaskGraph, KanbanTaskGraphDraft,
    KanbanTaskState, KanbanTenantPolicy, KanbanWorkspaceAdmission, KanbanWorkspaceKind,
    KanbanWorkspaceScope, OrchestrationAdapterBoardService, append_kanban_handoff,
    assemble_kanban_context, evaluate_kanban_promotion, load_kanban_graph, validate_kanban_graph,
    validate_kanban_workspace_mutation,
};
pub use crate::pi_compaction::{
    PiBranchHandoffPlan, PiBranchHandoffRequest, PiBranchSummaryMode, PiCompactionDecision,
    PiCompactionDiagnostic, PiCompactionDiagnosticKind, PiCompactionFingerprint, PiCompactionPlan,
    PiCompactionPolicy, PiCompactionRedactionBehavior, PiCompactionReplayEntry,
    PiCompactionReplayEntryKind, PiCompactionSummarizerRequest, PiCompactionSummarizerSelection,
    PiCompactionSummarizerState, PiCompactionSummaryEntry, PiCompactionSummaryFormat,
    PiCompactionToolBinding, PiCompactionTrigger, PiCompactionValidationResult,
    blake3_pi_compaction_fingerprint, evaluate_pi_compaction_trigger,
    evaluate_required_pi_summarizer, prepare_pi_branch_handoff_summary, prepare_pi_compaction,
    validate_pi_summary_entry,
};
pub use crate::provenance::{
    ProvenanceActor, ProvenanceArtifact, ProvenanceArtifactDraft, ProvenanceArtifactKind,
    ProvenanceAuthorizationStatus, ProvenanceBackendRequirement, ProvenanceBranchReport,
    ProvenanceContentObservation, ProvenanceContentObservationDraft, ProvenanceDiffHunk,
    ProvenanceError, ProvenanceErrorKind, ProvenanceFact, ProvenanceFactArtifact,
    ProvenanceFactAuthorization, ProvenanceFactError, ProvenanceFactModelActivity,
    ProvenanceFactObservation, ProvenanceFactQuery, ProvenanceFactRuntimeBlock,
    ProvenanceFactToolActivity, ProvenanceFileIdentity, ProvenanceFileIdentityDraft,
    ProvenanceGitAiAuthorshipLog, ProvenanceGitAiAuthorshipLogDraft,
    ProvenanceGitAiFileAttestation, ProvenanceGitAiFileAttestationDraft, ProvenanceGitAiMetadata,
    ProvenanceGitAiMetadataDraft, ProvenanceGraphEntry, ProvenanceLedgerEntry, ProvenanceLineRange,
    ProvenanceMessageEntry, ProvenanceMutationKind, ProvenanceOpaqueFact, ProvenanceRangeTransform,
    ProvenanceReceipt, ProvenanceReceiptDraft, ProvenanceRedactionClass, ProvenanceReplay,
    ProvenanceReplayMessage, ProvenanceReplayMetadata, ProvenanceReplayRole, ProvenanceSafeField,
    ProvenanceSummaryEntry, ProvenanceToolCheckpoint, ProvenanceToolCheckpointDraft,
    ProvenanceToolCheckpointReceipt, ProvenanceToolEffect, ProvenanceToolEffectDraft,
    ProvenanceToolEffectStatus, ProvenanceToolPolicy, ProvenanceToolPolicyDraft,
    ProvenanceUnsupportedEntry, ProvenanceUsageEntry, ToolpathExportPolicy, ToolpathProjection,
    ToolpathStep, VersionControlProvenanceBackend, classify_provenance_tool_policy,
    compress_line_ranges, export_toolpath_projection, query_provenance_facts,
    record_provenance_checkpoint, remove_line_ranges, replay_provenance_ledger,
    transform_line_ranges, validate_provenance_graph, validate_vcs_neutral_field_names,
};
pub use crate::provider_hardening::{
    ModelCapabilityDecisionStatus, ModelCapabilityDowngrade, ModelCapabilityDowngradePolicy,
    ModelCapabilityIssue, ModelCapabilityIssueKind, ModelCapabilityNegotiationDecision,
    ModelCapabilityNegotiationInput, ProviderApiStyle, ProviderApiStylePolicy, ProviderDiagnostic,
    ProviderDiagnosticKind, ProviderEndpointSensitivity, ProviderExtraBodyDescriptor,
    ProviderHeaderDescriptor, ProviderHeaderResolution, ProviderHeaderValueSource,
    ProviderModelCapabilityMatrix, ProviderModelClass, ProviderModelContentPartClass,
    ProviderModelMetadata, ProviderPrivacyPosture, ProviderSecretSourceReference,
    ProviderSelectionReceipt, ProviderSelectionReceiptInput, ProviderStreamBudgetKind,
    ProviderStreamChannel, ProviderStreamParseFailure, ProviderStreamParseFailureKind,
    ProviderStreamParseFailureReport, ProviderStreamParseReport, ProviderStreamParserBudget,
    ProviderStreamParserInput, ProviderStreamParserInventoryEntry, ProviderStreamParserReceipt,
    ProviderStreamParserStatus, ProviderStructuredOutputMode, ProviderValidationResult,
    blake3_provider_hardening_fingerprint, filter_agent_model, negotiate_model_capability,
    parse_provider_stream_chunks, provider_selection_receipt, provider_stream_parser_inventory,
    resolve_extra_body_descriptor, resolve_provider_api_style, validate_model_capability_matrix,
    validate_provider_header, validate_provider_stream_parser_inventory,
    validate_provider_stream_parser_receipt,
};
pub use crate::quality_records::{
    QualityAgentsGuidanceError, QualityAgentsGuidancePage, QualityAnnotationBody,
    QualityDependencyIsolationReport, QualityDependencyRequirement, QualityDiagnostic,
    QualityDiagnosticKind, QualityEditReport, QualityFingerprint, QualityFreshness,
    QualityIssuerClassification, QualityLifecycleReport, QualityMetadataField, QualityRecordBody,
    QualityRecordEnvelope, QualityRecordKind, QualitySidecarExport, QualitySpan,
    QualitySpanFreshnessReport, QualityUnknownSidecarRecord, QualityValidationResult,
    active_quality_records, blake3_quality_fingerprint, classify_quality_span_freshness,
    export_quality_sidecar_record, quality_dependency_isolation_report, quality_record_id,
    report_quality_after_edit, resolve_quality_agents_guidance, roundtrip_unknown_sidecar_record,
    validate_quality_record,
};
pub use crate::researchspec::{
    ResearchSpecArchiveConclusion, ResearchSpecBaselineEvidence, ResearchSpecCheckResult,
    ResearchSpecCheckStatus, ResearchSpecDiagnostic, ResearchSpecDiagnosticKind,
    ResearchSpecFingerprint, ResearchSpecGuardedFingerprint, ResearchSpecHypothesis,
    ResearchSpecHypothesisDecision, ResearchSpecKeepDecision, ResearchSpecKeepOutcome,
    ResearchSpecMeasurementRisk, ResearchSpecMetricDirection, ResearchSpecMetricSample,
    ResearchSpecMetricValue, ResearchSpecPromotionDecision, ResearchSpecPromotionGroup,
    ResearchSpecPromotionOutcome, ResearchSpecProvenance, ResearchSpecRunEvidence,
    ResearchSpecRunStatus, ResearchSpecStudyContract, ResearchSpecThresholdPolicy,
    ResearchSpecValidationResult, ResearchSpecWorkloadIdentity, blake3_researchspec_fingerprint,
    evaluate_research_keep_gate, evaluate_research_promotion_groups, validate_research_archive,
    validate_research_baseline, validate_research_hypothesis, validate_research_run,
    validate_research_study_contract,
};
pub use crate::runtime_dag::{
    RuntimeDagBlockedNode, RuntimeDagBounds, RuntimeDagEdge, RuntimeDagGraph, RuntimeDagIssueKind,
    RuntimeDagNode, RuntimeDagReceipt, RuntimeDagReceiptLeak, RuntimeDagRedactionStatus,
    RuntimeDagTopologicalBatch, RuntimeDagTopologicalPlan, RuntimeDagValidationIssue,
    RuntimeDagValidationReport, blake3_runtime_dag_fingerprint, explain_runtime_dag_blocked_nodes,
    plan_runtime_dag_topology, project_runtime_dag_receipt, validate_runtime_dag_graph,
    validate_runtime_dag_receipt,
};
pub use crate::scaffold::{
    ScaffoldAdmissionDecision, ScaffoldAdmissionEffect, ScaffoldAdmissionOutcome,
    ScaffoldAdmissionRequest, ScaffoldDiagnostic, ScaffoldDiagnosticKind, ScaffoldFingerprint,
    ScaffoldInfluenceReceipt, ScaffoldInfluenceValidationOutcome, ScaffoldProfile,
    ScaffoldProfileProvenance, ScaffoldReceiptSensitiveEvidence, ScaffoldReviewStatus,
    ScaffoldRevision, ScaffoldRevisionProposal, ScaffoldSelectionOutcome, ScaffoldSelectionPlan,
    ScaffoldSelectionRequest, ScaffoldStrategyKnob, ScaffoldValidationResult,
    apply_scaffold_admission, blake3_scaffold_fingerprint, select_scaffold_profile,
    validate_scaffold_influence_receipt, validate_scaffold_profile,
};
pub use crate::shell::{RuntimeAdapters, run_agent_loop};
#[cfg(feature = "steel")]
pub use crate::shell::{SteelPolicyRuntime, run_with_steel_policy, run_with_steel_program};
pub use crate::telemetry::{
    AdapterComponent, LoopDecisionKind, RuntimeEvent, RuntimeEventKind,
    RuntimeObservabilityProjection, RuntimeTerminalKind, project_model_stream_events,
    project_run_report_observability, project_runtime_event_observability,
};
pub use crate::tool_admission::{
    TOOL_DISPATCH_NEUTRAL_CONTRACT_IDENTITY, ToolDispatchCancellationPosture,
    ToolDispatchFallbackPolicy, ToolDispatchOutputBoundary, ToolDispatchPlan,
    ToolDispatchPlanIssue, ToolDispatchPlanRequest, ToolDispatchPlanStatus,
    ToolDispatchPolicyDraft, ToolDispatchTimeoutPosture, plan_tool_dispatch,
};
#[cfg(feature = "steel")]
pub use crate::transition::complete_with_steel_policy_denial;
pub use crate::transition::{
    AdapterOutcome, AdapterRequest, AdapterStep, DecisionStep, TurnPlan, TurnState,
    apply_adapter_outcome, apply_decision, complete_with_fabric_error, complete_with_guard_block,
    complete_with_guard_failure, plan_next_turn,
};
pub use crate::trust_boundary::{
    TrustBoundaryAction, TrustBoundaryActionKind, TrustBoundaryDescriptor, TrustBoundaryDiagnostic,
    TrustBoundaryDiagnosticKind, TrustBoundaryEvidencePolicy, TrustBoundaryFingerprint,
    TrustBoundaryIssueCode, TrustBoundaryJudgeBinding, TrustBoundaryJudgeOutcome,
    TrustBoundaryJudgeVeto, TrustBoundaryMonitorDecision, TrustBoundaryReceiptLeak,
    TrustBoundaryRedactionStatus, TrustBoundaryValidationResult, TrustBoundaryVerifierArtifact,
    TrustBoundaryViolationReceipt, bind_trust_boundary_judge, blake3_trust_boundary_fingerprint,
    monitor_trust_boundary_action, validate_trust_boundary_descriptor,
    validate_trust_boundary_receipt,
};
pub use crate::turns::{
    RuntimeCompletionCandidate, RuntimeLoopContractError, RuntimeLoopDecision,
    RuntimeLoopDecisionDraft, RuntimeLoopTurn,
};
pub use crate::workcmd::{
    WorkflowCommandArgumentType, WorkflowCommandArgumentValue, WorkflowCommandArgumentValueDraft,
    WorkflowCommandCapability, WorkflowCommandDuplicatePolicy, WorkflowCommandError,
    WorkflowCommandErrorKind, WorkflowCommandInputField, WorkflowCommandInputFieldDraft,
    WorkflowCommandManifest, WorkflowCommandManifestDraft, WorkflowCommandOutput,
    WorkflowCommandOutputContract, WorkflowCommandOutputContractDraft, WorkflowCommandOutputDraft,
    WorkflowCommandOutputKind, WorkflowCommandPolicy, WorkflowCommandProvenance,
    WorkflowCommandProvenanceDraft, WorkflowCommandRegistry, WorkflowCommandRegistryDiagnostic,
    WorkflowCommandRegistryDiagnosticKind, WorkflowCommandRegistryPolicy,
    WorkflowCommandSideEffectPosture, WorkflowCommandSourceScope, WorkflowCommandTrustStatus,
    WorkflowInvocationArgument, WorkflowInvocationArgumentDraft, WorkflowInvocationBinding,
    WorkflowInvocationPlan, WorkflowValidationStatus, bind_workflow_command_arguments,
    build_workflow_command_registry, prepare_workflow_command_invocation,
    validate_workflow_command_output,
};
pub use crate::worker_team::{
    Claim, Conflict, ConflictReport, ContextAdmissionPolicy, DelegationFailure, EvidenceReference,
    FailureKind, MutationIntent, ParentTrace, Report, ReportMergeOutcome, WorkerAuthority,
    WorkerBudgets, WorkerCancelOutcome, WorkerCapability, WorkerContextItem, WorkerContextItemKind,
    WorkerContextPacket, WorkerContextRedaction, WorkerHandle, WorkerMergedReport,
    WorkerOutputSchema, WorkerScope, WorkerService, WorkerStatus, WorkerTask, WorkerTaskDraft,
    collect_report, enqueue_task, merge_reports, request_cancel, request_status,
    validate_context_packet, validate_report, validate_task,
};
pub use crate::zerostack_midturn::{
    ZerostackCachedTokenRule, ZerostackInjectedServiceState, ZerostackMidturnDecision,
    ZerostackMidturnDiagnostic, ZerostackMidturnDiagnosticKind, ZerostackMidturnFingerprint,
    ZerostackMidturnPlan, ZerostackMidturnPolicy, ZerostackMidturnServices,
    ZerostackMidturnSummary, ZerostackMidturnSummaryKind, ZerostackMidturnValidationResult,
    ZerostackPromptPressure, ZerostackProviderUsage, ZerostackRouteMetadata,
    ZerostackToolArtifactDescriptor, ZerostackToolArtifactPolicy,
    artifactize_zerostack_tool_output, blake3_zerostack_midturn_fingerprint,
    calculate_zerostack_prompt_pressure, plan_zerostack_midturn_compaction,
    validate_zerostack_midturn_summary,
};
pub use animus_session_core::{
    AgentControlCheckpointDecision, AgentControlCheckpointTransition,
    AgentControlCommandDisposition, AgentControlCommandTransition, AgentControlReductionError,
    AgentControlState, AgentControlTerminalTransition, reduce_agent_control_checkpoint,
    reduce_agent_control_command, reduce_agent_control_terminal,
};
pub use animus_session_core::{
    SessionBranchCheckpointStatus, SessionBranchContinuationTarget, SessionBranchDiagnostic,
    SessionBranchDiagnosticKind, SessionBranchError, SessionBranchErrorKind,
    SessionBranchOperation, SessionBranchPlan, SessionBranchPlanStatus, SessionBranchPlannerInput,
    SessionBranchPolicy, SessionBranchPolicyAdmission, SessionBranchPolicyDraft,
    SessionBranchReceipt, SessionBranchRequest, SessionBranchRequestDraft, SessionCopyDescriptor,
    SessionCopyDescriptorClass, SessionCopyDescriptorDraft, SessionLedgerEntry,
    SessionLedgerEntryDraft, plan_session_branch, validate_session_branch_receipt_text,
};

/// Pure status-selection result.
pub type AgentStatusSelection = crate::status::choose::Selection;
/// Safe summary of a rejected status candidate.
pub type RejectedStatusCandidate = crate::status::types::RejectedCandidate;
/// Reason an observation was not selected.
pub type RejectedStatusReason = crate::status::types::RejectedReason;
/// Safe summary of a selected status authority.
pub type StatusAuthoritySummary = crate::status::types::AuthoritySummary;
/// Bounded explanation for a status selection decision.
pub type StatusDecisionExplanation = crate::status::explain::DecisionExplanation;
/// Evidence-summary validation error.
pub type StatusEvidenceError = crate::status::safe::EvidenceError;
/// Bounded redacted evidence summary for durable explanations.
pub type StatusEvidenceSummary = crate::status::safe::EvidenceSummary;
/// Reason a decision selected fallback or unknown state.
pub type StatusFallbackReason = crate::status::types::FallbackReason;
/// Freshness class assigned to a status decision.
pub type StatusFreshness = crate::status::types::Freshness;

/// Selects one semantic status authority for a target without ambient I/O.
#[must_use]
pub fn select_agent_status(
    request: &animus_adapters::HostStatusRequest,
    current_sequence: animus_adapters::StatusSequence,
    observations: &[animus_adapters::AgentStatusObservation],
) -> AgentStatusSelection {
    crate::status::choose::select(request, current_sequence, observations)
}

/// Validates a durable status evidence summary.
///
/// # Errors
///
/// Returns [`StatusEvidenceError`] when the summary is empty, overlong, or
/// contains raw or unsafe evidence markers.
pub fn validate_status_evidence_summary(text: &str) -> Result<(), StatusEvidenceError> {
    crate::status::safe::validate_summary(text)
}
