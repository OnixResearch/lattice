//! Implementation-neutral authority-provider contracts for guarded host actions.

mod evaluate;
mod input;
mod port;
mod proof;

#[cfg(test)]
mod tests;

pub use evaluate::{
    EffectiveAuthorityDecision, HostAuthorityDecision, HostAuthorityDecisionDraft,
    HostAuthorityFreshnessClass, HostAuthorityIssue, HostAuthorityReplayClass, HostAuthorityStatus,
    RequiredHostGates, Status, build_host_authority_decision, intersect_host_authority,
};
pub use input::{
    AgentAuthorityBinding, AgentAuthorityBindingDraft, BindingFreshness, HostAuthorityBounds,
    HostAuthorityRequest, HostAuthorityRequestDraft, admit_agent_authority_binding,
    admit_host_authority_request, validate_agent_authority_binding_set,
};
pub use port::HostAuthorityService;
pub use proof::{
    HOST_AUTHORITY_NON_CLAIMS, HostAuthorityReceipt, HostAuthorityReceiptDraft,
    HostAuthorityReceiptLeak, build_host_authority_receipt, validate_host_authority_receipt_text,
};
