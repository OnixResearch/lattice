//! Pure capability-bound semantic agent roles.
//!
//! Hosts load Nickel exports, authorize scoped role requests, provide current
//! identity observations, and execute providers, tools, checks, persistence, and
//! receipts outside this module. The reusable core only validates in-memory data,
//! computes exact intersections, binds immutable identities, and projects bounded evidence.

mod assignment;
mod output;
mod receipt;
mod separation;
mod types;
mod validation;

pub use assignment::{
    AGENT_ROLE_ACTIVATE_ABILITY, AGENT_ROLE_ASSIGN_ABILITY, AGENT_ROLE_CAPABILITY_CONTRACT,
    AGENT_ROLE_HANDOFF_ABILITY, AgentRoleActivationResourceInput, AgentRoleAssignment,
    AgentRoleAssignmentRequest, AgentRoleBoundModelPlan, AgentRoleCapabilityCeilings,
    AgentRoleCapabilityDenial, AgentRoleCapabilityResolution, AgentRoleCapabilityResolutionInput,
    AgentRoleCapabilitySource, AgentRoleHandoff, AgentRoleHandoffDraft,
    AgentRoleHandoffResourceInput, AgentRoleHandoffSensitiveDraft, AgentRoleModelCompositionInput,
    AgentRolePlan, AgentRolePlanInput, AgentRolePlanSnapshot, AgentRoleToolActionBinding,
    AgentRoleToolActionInput, AgentRoleTransitionDecision, AgentRoleTransitionKind,
    AgentRoleTransitionRequest, admit_agent_role_assignment, admit_agent_role_capability_use,
    agent_role_activation_resource, agent_role_handoff_resource, bind_agent_role_tool_action,
    build_agent_role_plan, compose_agent_role_model_plan, evaluate_agent_role_transition,
    resolve_agent_role_capabilities, validate_agent_role_handoff,
};
pub use output::{
    AGENT_ROLE_EMIT_CANDIDATE_ABILITY, AGENT_ROLE_EMIT_COORDINATION_ABILITY,
    AGENT_ROLE_EMIT_JUDGMENT_ABILITY, AGENT_ROLE_EMIT_OBSERVATION_ABILITY,
    AGENT_ROLE_EMIT_REVIEW_ABILITY, AGENT_ROLE_EMIT_VERIFICATION_ABILITY,
    AGENT_ROLE_SELECT_ABILITY, AGENT_ROLE_VETO_ABILITY, AgentRoleAdmissionState,
    AgentRoleDeterministicPolicyState, AgentRoleOutput, AgentRoleOutputAdmission,
    AgentRoleOutputClaim, AgentRoleOutputDraft, AgentRoleOutputEffect, AgentRoleOutputLeak,
    AgentRoleOutputResourceInput, AgentRoleOutputStatus, admit_agent_role_output,
    agent_role_output_resource,
};
pub use receipt::{
    AgentRoleReceipt, AgentRoleReceiptInput, AgentRoleReceiptKind, AgentRoleReceiptSensitiveDraft,
    project_agent_role_receipt,
};
pub use separation::{
    AgentRoleSeparationDecision, AgentRoleSeparationObservation, evaluate_agent_role_separation,
};
pub use types::{
    AGENT_ROLE_ASSIGNMENT_HASH_DOMAIN, AGENT_ROLE_CAPABILITY_HASH_DOMAIN,
    AGENT_ROLE_CATALOG_SCHEMA, AGENT_ROLE_HANDOFF_HASH_DOMAIN, AGENT_ROLE_MODEL_PLAN_HASH_DOMAIN,
    AGENT_ROLE_OUTPUT_HASH_DOMAIN, AGENT_ROLE_PLAN_HASH_DOMAIN, AGENT_ROLE_PROFILE_HASH_DOMAIN,
    AGENT_ROLE_RECEIPT_HASH_DOMAIN, AGENT_ROLE_SEPARATION_HASH_DOMAIN, AgentRoleBudgetPolicy,
    AgentRoleBudgetPolicyDraft, AgentRoleCatalog, AgentRoleCatalogDraft, AgentRoleClass,
    AgentRoleContextPosture, AgentRoleError, AgentRoleEvidencePosture, AgentRoleIssue,
    AgentRoleMutationPosture, AgentRoleOutputKind, AgentRoleProfile, AgentRoleProfileDraft,
    AgentRoleSeparationDimension, AgentRoleSeparationPolicy, AgentRoleSeparationPolicyDraft,
    AgentRoleToolset, AgentRoleToolsetDraft,
};
pub use validation::blake3_agent_role_profile_identity;

#[cfg(test)]
mod tests;
