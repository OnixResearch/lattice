//! Forwarding shim to the dedicated `animus-router` crate.
//!
//! This module re-exports every public item from `animus-router` so that
//! existing dependents inside `animus-adapters` and the rest of the workspace
//! compile without churn. The shim will be removed once direct consumers
//! migrate to `animus-router` directly.

/// Host-owned model-router execution service DTOs and contracts.
pub mod execution;

pub use animus_router::Bounds;
pub use animus_router::BoundsDraft;
pub use animus_router::Budget;
pub use animus_router::BudgetExhausted;
pub use animus_router::BudgetKind;
pub use animus_router::BudgetLedger;
pub use animus_router::CatalogEntry;
pub use animus_router::CatalogEntryDraft;
pub use animus_router::DeadlineClass;
pub use animus_router::DurationBucket;
pub use animus_router::ExecutionPlanError;
pub use animus_router::FallbackChain;
pub use animus_router::FallbackDecision;
pub use animus_router::FallbackEligibility;
pub use animus_router::FallbackPolicy;
pub use animus_router::FallbackRouteReason;
pub use animus_router::FallbackStep;
pub use animus_router::OutputContract;
pub use animus_router::PrivacyBand;
pub use animus_router::QuorumClass;
pub use animus_router::Receipt;
pub use animus_router::ReceiptDraft;
pub use animus_router::ReceiptLeak;
pub use animus_router::Recipe;
pub use animus_router::RecipeDraft;
pub use animus_router::Redaction;
pub use animus_router::RequestMetadata;
pub use animus_router::RequestMetadataDraft;
pub use animus_router::ResultDraft;
pub use animus_router::Role;
pub use animus_router::RoleStep;
pub use animus_router::RouteFamily;
pub use animus_router::RouteResult;
pub use animus_router::RouteResultStatus;
pub use animus_router::RouterExecutionPlan;
pub use animus_router::RuleSignalSource;
pub use animus_router::SelectedPlan;
pub use animus_router::SelectionDenial;
pub use animus_router::SignalBand;
pub use animus_router::SignalDraft;
pub use animus_router::SignalProjectionError;
pub use animus_router::SignalSource;
pub use animus_router::Signals;
pub use animus_router::StepFailure;
pub use animus_router::TaskShape;
pub use animus_router::ValidationError;
pub use animus_router::evaluate_fallback;
pub use animus_router::materialize_execution_plan;
pub use animus_router::project_signals;
pub use animus_router::select_recipe;
pub use animus_router::validate_receipt_text;
