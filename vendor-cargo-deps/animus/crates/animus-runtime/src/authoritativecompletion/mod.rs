//! Pure receipt-authoritative agent completion contracts.
//!
//! This module validates already-loaded registry, change, execution, candidate,
//! and Cairn summary values. It performs no filesystem, VCS, process, network,
//! clock, environment, provider, terminal, persistence, or retry effects.

mod completion;
mod model;
mod selection;
mod validation;

pub use completion::{
    admit_completion_candidate, admit_execution_results, evaluate_authoritative_completion,
};
pub use model::{
    AgentCheckArtifactDraft, AgentCheckDiagnostic, AgentCheckDiagnosticCode,
    AgentCheckDiagnosticDraft, AgentCheckExecutionDraft, AgentCheckRegistry,
    AgentCheckRegistryDraft, AgentCheckRow, AgentCheckRowDraft, AgentCheckSelection,
    AuthoritativeCompletionCandidate, AuthoritativeCompletionCandidateDraft,
    AuthoritativeCompletionOutcome, AuthoritativeCompletionReceipt, CairnCompletionSummaryDraft,
    CairnRequiredCheck, ChangedArtifactDraft, CheckCapability, CheckStatus, CheckTier,
    CompletionCandidateSource, ExecutionEvidenceReport, SourceSpanDraft,
};
pub use selection::{
    AgentCheckSelectionRequest, cairn_command_hash, cairn_selection_hash,
    select_required_agent_checks,
};
pub use validation::validate_agent_check_registry;

/// Animus-authored registry schema consumed by the pure runtime.
pub const AGENT_CHECK_REGISTRY_SCHEMA: &str = "animus.agent-check-registry.v1";
/// Exact authoritative completion schema owned by the pinned Cairn provider.
pub const CAIRN_COMPLETION_SCHEMA: &str = "cairn.agent-completion.v2";
/// Cairn generated-policy identity pinned by the current Animus registry.
pub const PINNED_CAIRN_POLICY_HASH: &str =
    "302bc1f8bcb27920c37a6bdc21865e2ca1fbb5a38c268570b3fc326734e59942";
/// Product behaviors that this reusable core is forbidden to perform.
pub const PROHIBITED_AUTHORITATIVE_COMPLETION_EFFECTS: &[&str] = &[
    "filesystem read",
    "VCS query",
    "process execution",
    "clock read",
    "network access",
    "environment inspection",
    "provider call",
    "terminal access",
    "artifact persistence",
    "implicit retry",
    "workspace mutation",
];

#[cfg(test)]
mod tests;
