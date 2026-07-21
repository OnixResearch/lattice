pub use crate::router::{
    DurationBucket, FallbackEligibility, FallbackPolicy, OutputContract, PrivacyBand, QuorumClass,
    RouteFamily, RouteResult, RouteResultStatus, SelectionDenial, SignalBand, SignalSource,
    TaskShape, project_signals, select_recipe,
};

/// Hard model-router execution bounds.
pub type ExecutionBounds = crate::router::Bounds;
/// Borrowed fields used to validate model-router execution bounds.
pub type ExecutionBoundsDraft<'a> = crate::router::BoundsDraft<'a>;
/// Safe model catalog projection for router selection.
pub type ModelCatalogEntry = crate::router::CatalogEntry;
/// Borrowed safe model catalog projection fields.
pub type ModelCatalogEntryDraft<'a> = crate::router::CatalogEntryDraft<'a>;
/// Model role used inside a router recipe.
pub type ModelRole = crate::router::Role;
/// Bounded router request metadata.
pub type RouterRequestMetadata = crate::router::RequestMetadata;
/// Borrowed bounded router request metadata fields.
pub type RouterRequestMetadataDraft<'a> = crate::router::RequestMetadataDraft<'a>;
/// Borrowed request-signal projection fields.
pub type RequestSignalDraft<'a> = crate::router::SignalDraft<'a>;
/// Bounded request signals for router selection.
pub type RequestSignals = crate::router::Signals;
/// Router budget state.
pub type RouterBudget = crate::router::Budget;
/// Safe router-recipe receipt.
pub type RouterReceipt = crate::router::Receipt;
/// Borrowed fields used to build a safe router receipt.
pub type RouterReceiptDraft<'a> = crate::router::ReceiptDraft<'a>;
/// Router receipt leak class.
pub type RouterReceiptLeak = crate::router::ReceiptLeak;
/// Router recipe contract checked before provider dispatch.
pub type RouterRecipe = crate::router::Recipe;
/// Borrowed fields used to validate a router recipe.
pub type RouterRecipeDraft<'a> = crate::router::RecipeDraft<'a>;
/// Router receipt redaction marker.
pub type RouterRedaction = crate::router::Redaction;
/// Router validation error.
pub type RouterValidationError = crate::router::ValidationError;
/// Router request-signal projection error.
pub type RouterSignalProjectionError = crate::router::SignalProjectionError;
/// Rule-based default router signal source.
pub type RuleRouterSignalSource = crate::router::RuleSignalSource;
/// Selected router plan.
pub type SelectedRoutePlan = crate::router::SelectedPlan;
/// Timeout/deadline class for bounded host execution.
pub type TimeoutClass = crate::router::DeadlineClass;
/// Borrowed final-output validation facts.
pub type RouteResultDraft<'a> = crate::router::ResultDraft<'a>;

/// Validates safe router receipt text for known leak sentinels.
///
/// # Errors
///
/// Returns [`RouterReceiptLeak`] when unsafe receipt text is detected.
pub fn validate_router_receipt_text(text: &str) -> Result<(), RouterReceiptLeak> {
    crate::router::validate_receipt_text(text)
}

/// Projects bounded request metadata with the default router rules.
#[must_use]
pub fn project_router_signals(
    metadata: &RouterRequestMetadata,
) -> crate::router::SignalDraft<'static> {
    project_signals(metadata)
}

/// Projects bounded request metadata through an injected router signal source.
///
/// # Errors
///
/// Returns [`RouterSignalProjectionError`] when the injected source cannot
/// produce safe bounded signal labels.
pub fn project_router_signals_with_source(
    source: &impl SignalSource,
    metadata: &RouterRequestMetadata,
) -> Result<crate::router::SignalDraft<'static>, RouterSignalProjectionError> {
    source.project(metadata)
}
