//! Forwarding shim to the dedicated `animus-router` crate.
//!
//! This module re-exports every public item from `animus-router` so that
//! existing dependents inside `animus-adapters` and the rest of the workspace
//! compile without churn. The shim will be removed once direct consumers
//! migrate to `animus-router` directly.

/// Host-owned model-router execution service DTOs and contracts.
pub mod execution;

pub use animus_router::{
    Bounds, BoundsDraft, Budget, BudgetExhausted, BudgetKind, BudgetLedger, CatalogEntry,
    CatalogEntryDraft, DeadlineClass, DurationBucket, ExecutionPlanError, FallbackChain,
    FallbackDecision, FallbackEligibility, FallbackPolicy, FallbackRouteReason, FallbackStep,
    OutputContract, PrivacyBand, QuorumClass, Receipt, ReceiptDraft, ReceiptLeak, Recipe,
    RecipeDraft, Redaction, RequestMetadata, RequestMetadataDraft, ResultDraft, Role, RoleStep,
    RouteFamily, RouteResult, RouteResultStatus, RouterExecutionPlan, RuleSignalSource,
    SelectedPlan, SelectionDenial, SignalBand, SignalDraft, SignalProjectionError, SignalSource,
    Signals, StepFailure, TaskShape, ValidationError, evaluate_fallback,
    materialize_execution_plan, project_signals, select_recipe, validate_receipt_text,
};
