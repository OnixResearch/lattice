/// Validated retrieval budget descriptor.
pub type RetrievalBudget = crate::resource::retrieval::contract::Budget;
/// Borrowed retrieval budget fields before validation.
pub type RetrievalBudgetDraft<'a> = crate::resource::retrieval::contract::Draft<'a>;
/// Retrieval-budget validation error.
pub type RetrievalBudgetError = crate::resource::retrieval::failure::Error;
/// Whether host policy requires budgets for retrieval-capable resources.
pub type RetrievalBudgetRequirement = crate::resource::retrieval::contract::Requirement;
/// Retrieval-capable resource class used by budget policy.
pub type RetrievalCapability = crate::resource::retrieval::contract::Capability;
/// Retrieval planning decision.
pub type RetrievalDecision = crate::resource::retrieval::facts::Decision;
/// Bounded retrieval evidence metadata safe for receipts and reports.
pub type RetrievalEvidenceMetadata = crate::resource::retrieval::facts::Metadata;
/// Safe retrieval receipt/report metadata draft.
pub type RetrievalEvidenceMetadataDraft<'a> = crate::resource::retrieval::facts::MetadataDraft<'a>;
/// Evidence status after one retrieval attempt.
pub type RetrievalEvidenceStatus = crate::resource::retrieval::facts::EvidenceStatus;
/// Validated evidence summary used for repeat/stop planning.
pub type RetrievalEvidenceSummary = crate::resource::retrieval::facts::EvidenceSummary;
/// Borrowed evidence summary used for repeat/stop planning.
pub type RetrievalEvidenceSummaryDraft<'a> = crate::resource::retrieval::facts::EvidenceSummaryDraft<'a>;
/// Admission result for a retrieval-capable resource.
pub type RetrievalResourceAdmission = crate::resource::retrieval::contract::Admission;
/// Host retrieval policy applied to one resource descriptor.
pub type RetrievalResourcePolicy = crate::resource::retrieval::contract::Policy;
/// Truncation status for retrieved content metadata.
pub type RetrievalTruncation = crate::resource::retrieval::facts::Truncation;
/// Applies retrieval policy to a resource and optional budget descriptor.
///
/// # Errors
///
/// Returns [`RetrievalBudgetError`] when the capability, budget, or policy
/// combination is not admissible.
pub fn admit_retrieval_resource(
    capability: RetrievalCapability,
    budget: Option<&RetrievalBudget>,
    policy: RetrievalResourcePolicy,
) -> Result<RetrievalResourceAdmission, RetrievalBudgetError> {
    crate::resource::retrieval::contract::admit(capability, budget, policy)
}

/// Plans whether retrieval should stop, repeat, or request missing input.
#[must_use]
pub fn plan_retrieval(budget: &RetrievalBudget, evidence: &RetrievalEvidenceSummary) -> RetrievalDecision {
    crate::resource::retrieval::facts::plan(budget, evidence)
}
/// MCP App adoption decision under the public facade naming scheme.
pub type McpAppAdoptionDecision = crate::mcpapp::AdoptionDecision;
/// MCP App descriptor under the public facade naming scheme.
pub type McpAppDescriptor = crate::mcpapp::Descriptor;
/// Borrowed MCP App descriptor draft under the public facade naming scheme.
pub type McpAppDescriptorDraft<'a> = crate::mcpapp::DescriptorDraft<'a>;
/// MCP App evaluation report under the public facade naming scheme.
pub type McpAppEvaluation = crate::mcpapp::Evaluation;
/// MCP App evaluation axis under the public facade naming scheme.
pub type McpAppEvaluationAxis = crate::mcpapp::EvaluationAxis;
/// MCP App evaluation error under the public facade naming scheme.
pub type McpAppEvaluationError = crate::mcpapp::EvaluationError;
/// MCP App lifecycle policy under the public facade naming scheme.
pub type McpAppLifecyclePolicy = crate::mcpapp::LifecyclePolicy;
/// MCP App policy admission under the public facade naming scheme.
pub type McpAppPolicyAdmission = crate::mcpapp::PolicyAdmission;
/// Borrowed MCP App policy admission draft under the public facade naming scheme.
pub type McpAppPolicyAdmissionDraft = crate::mcpapp::PolicyAdmissionDraft;
/// MCP App risk reason under the public facade naming scheme.
pub type McpAppRiskReason = crate::mcpapp::RiskReason;
/// MCP App sizing policy under the public facade naming scheme.
pub type McpAppSizePolicy = crate::mcpapp::SizePolicy;
/// Security review action class under the public facade naming scheme.
pub type SecurityActionClass = crate::security::ActionClass;
/// Security advisory outcome under the public facade naming scheme.
pub type SecurityAdvisoryOutcome = crate::security::AdvisoryOutcome;
/// Security advisory review under the public facade naming scheme.
pub type SecurityAdvisoryReview = crate::security::AdvisoryReview;
/// Borrowed security advisory review draft under the public facade naming scheme.
pub type SecurityAdvisoryReviewDraft<'a> = crate::security::AdvisoryReviewDraft<'a>;
/// Security fail behavior under the public facade naming scheme.
pub type SecurityFailBehavior = crate::security::FailBehavior;
/// Security fail policy under the public facade naming scheme.
pub type SecurityFailPolicy = crate::security::FailPolicy;
/// Security finding under the public facade naming scheme.
pub type SecurityFinding = crate::security::Finding;
/// Security finding category under the public facade naming scheme.
pub type SecurityFindingCategory = crate::security::Category;
/// Security finding confidence under the public facade naming scheme.
pub type SecurityFindingConfidence = crate::security::Confidence;
/// Security finding disposition under the public facade naming scheme.
pub type SecurityFindingDisposition = crate::security::Disposition;
/// Security permission gate under the public facade naming scheme.
pub type SecurityPermissionGate = crate::security::Gate;
/// Security permission snapshot under the public facade naming scheme.
pub type SecurityPermissionSnapshot = crate::security::Snapshot;
/// Security receipt leak class under the public facade naming scheme.
pub type SecurityReceiptLeak = crate::security::Leak;
/// Security review input under the public facade naming scheme.
pub type SecurityReviewInput = crate::security::Review;
/// Borrowed security review input draft under the public facade naming scheme.
pub type SecurityReviewInputDraft<'a> = crate::security::Draft<'a>;
/// Security review receipt under the public facade naming scheme.
pub type SecurityReviewReceipt = crate::security::Receipt;
/// Security risk class under the public facade naming scheme.
pub type SecurityRiskClass = crate::security::RiskClass;
/// Security scan report under the public facade naming scheme.
pub type SecurityScanReport = crate::security::Report;
/// Security scanner status under the public facade naming scheme.
pub type SecurityScannerStatus = crate::security::ScannerStatus;
/// Security decision under the public facade naming scheme.
pub type SecurityDecision = crate::security::Outcome;
/// Security decision issue under the public facade naming scheme.
pub type SecurityDecisionIssue = crate::security::Issue;
/// Security decision status under the public facade naming scheme.
pub type SecurityDecisionStatus = crate::security::Status;
/// Security review binding under the public facade naming scheme.
pub type SecurityReviewBinding<'a> = crate::security::Binding<'a>;
/// Security review error under the public facade naming scheme.
pub type SecurityReviewError = crate::security::Error;
/// Security tool-call descriptor under the public facade naming scheme.
pub type SecurityToolCallDescriptor = crate::security::ToolCallDescriptor;
/// Borrowed security tool-call descriptor draft under the public facade naming scheme.
pub type SecurityToolCallDescriptorDraft<'a> = crate::security::ToolCallDescriptorDraft<'a>;
/// Binds scanner findings and advisory reviews into permission decisions.
#[must_use]
pub fn bind_security_review(binding: &SecurityReviewBinding<'_>) -> SecurityDecision {
    crate::security::bind(binding)
}

/// Runs deterministic security scanning over safe review labels.
#[must_use]
pub fn scan_review_input(input: &SecurityReviewInput) -> SecurityScanReport {
    crate::security::scan_review_input(input)
}

/// Validates rendered security review receipt text for raw-material leaks.
///
/// # Errors
///
/// Returns [`SecurityReceiptLeak`] when known leak sentinels appear.
pub fn validate_security_review_receipt_text(text: &str) -> Result<(), SecurityReceiptLeak> {
    crate::security::validate_text(text)
}
/// Validated speculative workspace descriptor under the public facade naming scheme.
pub type SpeculativeWorkspaceDescriptor = crate::workspace::intent::Descriptor;
/// Borrowed speculative workspace descriptor input under the public facade naming scheme.
pub type SpeculativeWorkspaceDescriptorDraft<'a> = crate::workspace::intent::DescriptorDraft<'a>;
/// Abort behavior requested for a workspace under the public facade naming scheme.
pub type WorkspaceAbortPolicy = crate::workspace::rules::AbortPolicy;
/// Validated workspace adapter capability advertisement under the public facade naming scheme.
pub type WorkspaceAdapterCapabilities = crate::workspace::adapter::Capabilities;
/// Borrowed workspace adapter capability input under the public facade naming scheme.
pub type WorkspaceAdapterCapabilitiesDraft<'a> = crate::workspace::adapter::CapabilitiesDraft<'a>;
/// Backend kind advertised by a workspace adapter under the public facade naming scheme.
pub type WorkspaceAdapterKind = crate::workspace::adapter::Kind;
/// Pure workspace plan safe to inspect before host adapter execution.
pub type WorkspacePlan = crate::workspace::planning::Plan;
/// First pure workspace planning issue under the public facade naming scheme.
pub type WorkspacePlanIssue = crate::workspace::planning::Issue;
/// Pure workspace planning request under the public facade naming scheme.
pub type WorkspacePlanRequest<'a> = crate::workspace::planning::Request<'a>;
/// Pure workspace planning status under the public facade naming scheme.
pub type WorkspacePlanStatus = crate::workspace::planning::Status;
/// Safe workspace receipt containing labels, counts, redactions, and BLAKE3 hashes only.
pub type WorkspaceReceipt = crate::workspace::receipt::Record;
/// Borrowed fields used to construct a safe workspace receipt.
pub type WorkspaceReceiptDraft<'a> = crate::workspace::receipt::Draft<'a>;
/// Workspace receipt leak class under the public facade naming scheme.
pub type WorkspaceReceiptLeak = crate::workspace::receipt::Leak;
/// Typed workspace adapter outcome under the public facade naming scheme.
pub type WorkspaceAdapterOutcome = crate::workspace::result::Outcome;
/// Borrowed workspace adapter outcome input under the public facade naming scheme.
pub type WorkspaceAdapterOutcomeDraft<'a> = crate::workspace::result::OutcomeDraft<'a>;
/// Workspace adapter outcome kind under the public facade naming scheme.
pub type WorkspaceAdapterOutcomeKind = crate::workspace::result::Kind;
/// Availability state advertised by a workspace adapter under the public facade naming scheme.
pub type WorkspaceAdapterState = crate::workspace::adapter::State;
/// Capability class for speculative workspace adapters under the public facade naming scheme.
pub type WorkspaceCapability = crate::workspace::adapter::Capability;
/// Cleanup behavior required after workspace operations under the public facade naming scheme.
pub type WorkspaceCleanupPolicy = crate::workspace::rules::CleanupPolicy;
/// Target for a successful workspace commit under the public facade naming scheme.
pub type WorkspaceCommitTarget = crate::workspace::rules::CommitTarget;
/// Conflict behavior for workspace commit planning under the public facade naming scheme.
pub type WorkspaceConflictPolicy = crate::workspace::rules::ConflictPolicy;
/// Policy for ignored or local files in speculative workspaces under the public facade naming
/// scheme.
pub type WorkspaceFileInclusionPolicy = crate::workspace::rules::FileInclusionPolicy;
/// Requested workspace operation under the public facade naming scheme.
pub type WorkspaceOperation = crate::workspace::rules::Operation;
/// Restore metadata under the public facade naming scheme.
pub type WorkspaceRestoreMetadata = crate::workspace::isolation::RestoreMetadata;
/// Borrowed restore metadata input under the public facade naming scheme.
pub type WorkspaceRestoreMetadataDraft<'a> = crate::workspace::isolation::RestoreMetadataDraft<'a>;
/// Validated isolated task workspace under the public facade naming scheme.
pub type IsolatedTaskWorkspace = crate::workspace::isolation::IsolatedTaskWorkspace;
/// Borrowed isolated task workspace input under the public facade naming scheme.
pub type IsolatedTaskWorkspaceDraft<'a> = crate::workspace::isolation::IsolatedTaskWorkspaceDraft<'a>;
/// Automation run kind under the public facade naming scheme.
pub type WorkspaceAutomationKind = crate::workspace::automation::AutomationKind;
/// Automation sandbox profile under the public facade naming scheme.
pub type WorkspaceAutomationSandboxProfile = crate::workspace::automation::AutomationSandboxProfile;
/// Validated automation run contract under the public facade naming scheme.
pub type WorkspaceAutomationRunContract = crate::workspace::automation::AutomationRunContract;
/// Borrowed automation run contract input under the public facade naming scheme.
pub type WorkspaceAutomationRunContractDraft<'a> = crate::workspace::automation::AutomationRunContractDraft<'a>;
/// Safe automation event kind under the public facade naming scheme.
pub type WorkspaceAutomationEventKind = crate::workspace::automation::AutomationEventKind;
/// Safe automation event under the public facade naming scheme.
pub type WorkspaceAutomationEvent = crate::workspace::automation::AutomationEvent;
/// Borrowed automation event input under the public facade naming scheme.
pub type WorkspaceAutomationEventDraft<'a> = crate::workspace::automation::AutomationEventDraft<'a>;
/// Handoff direction under the public facade naming scheme.
pub type WorkspaceHandoffDirection = crate::workspace::handoff::HandoffDirection;
/// Handoff conflict status under the public facade naming scheme.
pub type WorkspaceHandoffConflictStatus = crate::workspace::handoff::HandoffConflictStatus;
/// Snapshot handoff record under the public facade naming scheme.
pub type WorkspaceSnapshotHandoffRecord = crate::workspace::handoff::SnapshotHandoffRecord;
/// Borrowed snapshot handoff input under the public facade naming scheme.
pub type WorkspaceSnapshotHandoffRecordDraft<'a> = crate::workspace::handoff::SnapshotHandoffRecordDraft<'a>;
/// Ignored-file redaction class under the public facade naming scheme.
pub type WorkspaceIgnoredFileRedactionClass = crate::workspace::inclusion::IgnoredFileRedactionClass;
/// Ignored-file redaction approval under the public facade naming scheme.
pub type WorkspaceIgnoredFileRedactionApproval = crate::workspace::inclusion::IgnoredFileRedactionApproval;
/// Ignored-file allowlist entry under the public facade naming scheme.
pub type WorkspaceIgnoredFileAllowlistEntry = crate::workspace::inclusion::IgnoredFileAllowlistEntry;
/// Borrowed ignored-file allowlist input under the public facade naming scheme.
pub type WorkspaceIgnoredFileAllowlistEntryDraft<'a> = crate::workspace::inclusion::IgnoredFileAllowlistEntryDraft<'a>;
/// Ignored-file materialization request under the public facade naming scheme.
pub type WorkspaceIgnoredFileMaterializationRequest<'a> =
    crate::workspace::inclusion::IgnoredFileMaterializationRequest<'a>;
/// Ignored-file materialization decision under the public facade naming scheme.
pub type WorkspaceIgnoredFileMaterializationDecision = crate::workspace::inclusion::IgnoredFileMaterializationDecision;
/// Ignored-file materialization status under the public facade naming scheme.
pub type WorkspaceIgnoredFileMaterializationStatus = crate::workspace::inclusion::IgnoredFileMaterializationStatus;
/// Ignored-file materialization denial reason under the public facade naming scheme.
pub type WorkspaceIgnoredFileDenialReason = crate::workspace::inclusion::IgnoredFileDenialReason;

/// Builds a pure workspace plan and fails closed before host execution on invalid input.
#[must_use]
pub fn evaluate_workspace_plan(request: WorkspacePlanRequest<'_>) -> WorkspacePlan {
    crate::workspace::planning::evaluate(request)
}

/// Plans ignored-file materialization without copying data.
///
/// # Errors
///
/// Returns [`crate::AdapterContractError`] when the request file label is empty or malformed.
pub fn plan_workspace_ignored_file_materialization(
    request: WorkspaceIgnoredFileMaterializationRequest<'_>,
) -> Result<WorkspaceIgnoredFileMaterializationDecision, crate::AdapterContractError> {
    crate::workspace::inclusion::plan_ignored_file_materialization(request)
}

/// Validates candidate workspace receipt evidence text for obvious sensitive material.
///
/// # Errors
///
/// Returns [`WorkspaceReceiptLeak`] when known leak sentinels appear.
pub fn validate_workspace_receipt_text(text: &str) -> Result<(), WorkspaceReceiptLeak> {
    crate::workspace::receipt::validate_text(text)
}
