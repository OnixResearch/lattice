//! Pure bounded same-plan provider retry contracts.
//!
//! Nickel remains the policy source of truth. This module revalidates host-loaded
//! values, classifies safe provider observations, enforces exact invocation and
//! semantic-stream boundaries, plans deterministic delays, gates injected timer
//! observations, and projects safe progress and receipts. It owns no clocks,
//! sleeps, network clients, credentials, provider discovery, or router fallback.

mod core;
mod data;
#[path = "receipt.rs"]
mod evidence;

pub use core::{
    advance_provider_retry_attempt, classify_provider_retry_failure, evaluate_provider_retry_timer,
    observe_provider_retry_model_frame, observe_provider_retry_stream, plan_provider_retry,
    plan_provider_retry_wait, project_provider_retry_decision_event,
    project_provider_retry_timer_event, provider_retry_invocation_content_hash,
    provider_retry_invocation_input_content_hash, provider_retry_policy_content_hash,
    validate_provider_retry_invocation, validate_provider_retry_policy,
    validate_provider_retry_progress_event,
};
pub use data::{
    PROVIDER_RETRY_DECISION_HASH_DOMAIN, PROVIDER_RETRY_INVOCATION_HASH_DOMAIN,
    PROVIDER_RETRY_POLICY_HASH_DOMAIN, PROVIDER_RETRY_POLICY_SCHEMA,
    PROVIDER_RETRY_RECEIPT_HASH_DOMAIN, PROVIDER_RETRY_RECEIPT_SCHEMA, ProviderRetryAttemptState,
    ProviderRetryCancellation, ProviderRetryDecision, ProviderRetryDecisionStatus,
    ProviderRetryDispatchGate, ProviderRetryDispatchGateStatus, ProviderRetryDispatchRequest,
    ProviderRetryError, ProviderRetryFailureClass, ProviderRetryFailureObservation,
    ProviderRetryInvocation, ProviderRetryInvocationInput, ProviderRetryIssue,
    ProviderRetryMapping, ProviderRetryMappingInput, ProviderRetryPlanInput, ProviderRetryPolicy,
    ProviderRetryPolicyInput, ProviderRetryProgressEvent, ProviderRetryProgressStatus,
    ProviderRetryReceipt, ProviderRetryReceiptInput, ProviderRetryReceiptStatus,
    ProviderRetrySemanticFrame, ProviderRetrySensitiveDraft, ProviderRetryStreamObservation,
    ProviderRetryStreamState, ProviderRetryTimerEvaluation, ProviderRetryTimerObservation,
    ProviderRetryTimerOutcome, ProviderRetryTimerRequest, ProviderRetryWaitPlan,
    ProviderRetryWaitState, ProviderRetryWaitStatus,
};
pub use evidence::project_provider_retry_receipt;

#[cfg(test)]
mod tests;
