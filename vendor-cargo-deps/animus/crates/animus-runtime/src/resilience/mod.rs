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

pub use core::advance_provider_retry_attempt;
pub use core::classify_provider_retry_failure;
pub use core::evaluate_provider_retry_timer;
pub use core::observe_provider_retry_model_frame;
pub use core::observe_provider_retry_stream;
pub use core::plan_provider_retry;
pub use core::plan_provider_retry_wait;
pub use core::project_provider_retry_decision_event;
pub use core::project_provider_retry_timer_event;
pub use core::provider_retry_invocation_content_hash;
pub use core::provider_retry_invocation_input_content_hash;
pub use core::provider_retry_policy_content_hash;
pub use core::validate_provider_retry_invocation;
pub use core::validate_provider_retry_policy;
pub use core::validate_provider_retry_progress_event;

pub use data::PROVIDER_RETRY_DECISION_HASH_DOMAIN;
pub use data::PROVIDER_RETRY_INVOCATION_HASH_DOMAIN;
pub use data::PROVIDER_RETRY_POLICY_HASH_DOMAIN;
pub use data::PROVIDER_RETRY_POLICY_SCHEMA;
pub use data::PROVIDER_RETRY_RECEIPT_HASH_DOMAIN;
pub use data::PROVIDER_RETRY_RECEIPT_SCHEMA;
pub use data::ProviderRetryAttemptState;
pub use data::ProviderRetryCancellation;
pub use data::ProviderRetryDecision;
pub use data::ProviderRetryDecisionStatus;
pub use data::ProviderRetryDispatchGate;
pub use data::ProviderRetryDispatchGateStatus;
pub use data::ProviderRetryDispatchRequest;
pub use data::ProviderRetryError;
pub use data::ProviderRetryFailureClass;
pub use data::ProviderRetryFailureObservation;
pub use data::ProviderRetryInvocation;
pub use data::ProviderRetryInvocationInput;
pub use data::ProviderRetryIssue;
pub use data::ProviderRetryMapping;
pub use data::ProviderRetryMappingInput;
pub use data::ProviderRetryPlanInput;
pub use data::ProviderRetryPolicy;
pub use data::ProviderRetryPolicyInput;
pub use data::ProviderRetryProgressEvent;
pub use data::ProviderRetryProgressStatus;
pub use data::ProviderRetryReceipt;
pub use data::ProviderRetryReceiptInput;
pub use data::ProviderRetryReceiptStatus;
pub use data::ProviderRetrySemanticFrame;
pub use data::ProviderRetrySensitiveDraft;
pub use data::ProviderRetryStreamObservation;
pub use data::ProviderRetryStreamState;
pub use data::ProviderRetryTimerEvaluation;
pub use data::ProviderRetryTimerObservation;
pub use data::ProviderRetryTimerOutcome;
pub use data::ProviderRetryTimerRequest;
pub use data::ProviderRetryWaitPlan;
pub use data::ProviderRetryWaitState;
pub use data::ProviderRetryWaitStatus;
pub use evidence::project_provider_retry_receipt;

#[cfg(test)]
mod tests;
