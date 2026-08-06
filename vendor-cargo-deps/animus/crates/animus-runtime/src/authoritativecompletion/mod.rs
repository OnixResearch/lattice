//! Pure receipt-authoritative agent completion contracts.
//!
//! This module validates already-loaded registry, change, execution, candidate,
//! and Cairn summary values. It performs no filesystem, VCS, process, network,
//! clock, environment, provider, terminal, persistence, or retry effects.

mod completion;
mod model;
mod selection;
mod validation;

pub use completion::admit_completion_candidate;
pub use completion::admit_execution_results;
pub use completion::evaluate_authoritative_completion;
pub use model::AgentCheckArtifactDraft;
pub use model::AgentCheckDiagnostic;
pub use model::AgentCheckDiagnosticCode;
pub use model::AgentCheckDiagnosticDraft;
pub use model::AgentCheckExecutionDraft;
pub use model::AgentCheckRegistry;
pub use model::AgentCheckRegistryDraft;
pub use model::AgentCheckRow;
pub use model::AgentCheckRowDraft;
pub use model::AgentCheckSelection;
pub use model::AuthoritativeCompletionCandidate;
pub use model::AuthoritativeCompletionCandidateDraft;
pub use model::AuthoritativeCompletionOutcome;
pub use model::AuthoritativeCompletionReceipt;
pub use model::CairnCompletionSummaryDraft;
pub use model::CairnRequiredCheck;
pub use model::ChangedArtifactDraft;
pub use model::CheckCapability;
pub use model::CheckStatus;
pub use model::CheckTier;
pub use model::CompletionCandidateSource;
pub use model::ExecutionEvidenceReport;
pub use model::SourceSpanDraft;
pub use selection::AgentCheckSelectionRequest;
pub use selection::cairn_command_hash;
pub use selection::cairn_selection_hash;
pub use selection::select_required_agent_checks;
pub use validation::validate_agent_check_registry;

/// Animus-authored registry schema consumed by the pure runtime.
pub const AGENT_CHECK_REGISTRY_SCHEMA: &str = "animus.agent-check-registry.v1";
/// Exact authoritative completion schema owned by the pinned Cairn provider.
pub const CAIRN_COMPLETION_SCHEMA: &str = "cairn.agent-completion.v2";
/// Cairn generated-policy identity pinned by the current Animus registry.
pub const PINNED_CAIRN_POLICY_HASH: &str = "b8f81a80716eb7427dacdcd4dac9c01c1599d2bee79cdeb0a6e444525377da59";
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
