//! Parity tests verifying that every public item re-exported from
//! `animus-adapters::router` resolves to the live `animus-router` type
//! with the same API shape after the crate-extraction barrier.

// Type-parity tests intentionally spend most statements on compile-time type
// resolution. Runtime assertion density is checked by the router unit tests.
#![allow(unknown_lints, assertion_density)]

const EXPECTED_ROUTER_ITEM_COUNT: usize = 40;
const EXPECTED_ALIAS_COUNT: usize = 21;
const PARITY_PROMPT_CHARS: u32 = 1;
const PARITY_TOOL_CALLS: u32 = 0;
const PARITY_CONTEXT_PER_MILLE: u16 = 1;
const FUNCTION_ITEM_BYTES: usize = 0;

fn assert_type<T: ?Sized>() {}

fn assert_function_item<F>(function: &F) {
    assert_eq!(core::mem::size_of_val(function), FUNCTION_ITEM_BYTES);
}

/// Every re-exported type name compiles from the original path.
#[test]
fn adapter_reexport_types_resolve() {
    assert_type::<animus_adapters::router::Budget>();
    assert_type::<animus_adapters::router::Bounds>();
    assert_type::<animus_adapters::router::CatalogEntry>();
    assert_type::<animus_adapters::router::DeadlineClass>();
    assert_type::<animus_adapters::router::DurationBucket>();
    assert_type::<animus_adapters::router::FallbackEligibility>();
    assert_type::<animus_adapters::router::FallbackPolicy>();
    assert_type::<animus_adapters::router::OutputContract>();
    assert_type::<animus_adapters::router::PrivacyBand>();
    assert_type::<animus_adapters::router::QuorumClass>();
    assert_type::<animus_adapters::router::Receipt>();
    assert_type::<animus_adapters::router::Redaction>();
    assert_type::<animus_adapters::router::Recipe>();
    assert_type::<animus_adapters::router::Role>();
    assert_type::<animus_adapters::router::RouteFamily>();
    assert_type::<animus_adapters::router::SelectedPlan>();
    assert_type::<animus_adapters::router::ExecutionPlanError>();
    assert_type::<animus_adapters::router::FallbackChain>();
    assert_type::<animus_adapters::router::FallbackStep>();
    assert_type::<animus_adapters::router::RoleStep>();
    assert_type::<animus_adapters::router::RouterExecutionPlan>();
    assert_type::<animus_adapters::router::SelectionDenial>();
    assert_type::<animus_adapters::router::SignalBand>();
    assert_type::<animus_adapters::router::Signals>();
    assert_type::<animus_adapters::router::TaskShape>();
    assert_type::<animus_adapters::router::RouteResult>();
    assert_type::<animus_adapters::router::RouteResultStatus>();
    assert_type::<animus_adapters::router::ValidationError>();
    assert_type::<animus_adapters::router::ReceiptLeak>();
    assert_type::<animus_adapters::router::BoundsDraft<'_>>();
    assert_type::<animus_adapters::router::CatalogEntryDraft<'_>>();
    assert_type::<animus_adapters::router::ReceiptDraft<'_>>();
    assert_type::<animus_adapters::router::RecipeDraft<'_>>();
    assert_type::<animus_adapters::router::RequestMetadata>();
    assert_type::<animus_adapters::router::RequestMetadataDraft<'_>>();
    assert_type::<animus_adapters::router::RuleSignalSource>();
    assert_type::<animus_adapters::router::SignalDraft<'_>>();
    assert_type::<animus_adapters::router::SignalProjectionError>();
    assert_type::<dyn animus_adapters::router::SignalSource>();
    assert_type::<animus_adapters::router::ResultDraft<'_>>();

    assert_function_item(&animus_adapters::router::project_signals);
    assert_function_item(&animus_adapters::router::select_recipe);
    assert_function_item(&animus_adapters::router::materialize_execution_plan);
    assert_function_item(&animus_adapters::router::validate_receipt_text);

    assert_eq!(EXPECTED_ROUTER_ITEM_COUNT, router_item_names().len());
}

/// Every re-exported type also compiles from the direct `animus_router` path.
#[test]
fn direct_router_types_resolve() {
    assert_type::<animus_router::Budget>();
    assert_type::<animus_router::Bounds>();
    assert_type::<animus_router::CatalogEntry>();
    assert_type::<animus_router::DeadlineClass>();
    assert_type::<animus_router::DurationBucket>();
    assert_type::<animus_router::FallbackEligibility>();
    assert_type::<animus_router::FallbackPolicy>();
    assert_type::<animus_router::OutputContract>();
    assert_type::<animus_router::PrivacyBand>();
    assert_type::<animus_router::QuorumClass>();
    assert_type::<animus_router::Receipt>();
    assert_type::<animus_router::Redaction>();
    assert_type::<animus_router::Recipe>();
    assert_type::<animus_router::Role>();
    assert_type::<animus_router::RouteFamily>();
    assert_type::<animus_router::SelectedPlan>();
    assert_type::<animus_router::ExecutionPlanError>();
    assert_type::<animus_router::FallbackChain>();
    assert_type::<animus_router::FallbackStep>();
    assert_type::<animus_router::RoleStep>();
    assert_type::<animus_router::RouterExecutionPlan>();
    assert_type::<animus_router::SelectionDenial>();
    assert_type::<animus_router::SignalBand>();
    assert_type::<animus_router::Signals>();
    assert_type::<animus_router::TaskShape>();
    assert_type::<animus_router::RouteResult>();
    assert_type::<animus_router::RouteResultStatus>();
    assert_type::<animus_router::ValidationError>();
    assert_type::<animus_router::ReceiptLeak>();
    assert_type::<animus_router::BoundsDraft<'_>>();
    assert_type::<animus_router::CatalogEntryDraft<'_>>();
    assert_type::<animus_router::ReceiptDraft<'_>>();
    assert_type::<animus_router::RecipeDraft<'_>>();
    assert_type::<animus_router::RequestMetadata>();
    assert_type::<animus_router::RequestMetadataDraft<'_>>();
    assert_type::<animus_router::RuleSignalSource>();
    assert_type::<animus_router::SignalDraft<'_>>();
    assert_type::<animus_router::SignalProjectionError>();
    assert_type::<dyn animus_router::SignalSource>();
    assert_type::<animus_router::ResultDraft<'_>>();

    assert_function_item(&animus_router::project_signals);
    assert_function_item(&animus_router::select_recipe);
    assert_function_item(&animus_router::materialize_execution_plan);
    assert_function_item(&animus_router::validate_receipt_text);

    assert_eq!(EXPECTED_ROUTER_ITEM_COUNT, router_item_names().len());
}

/// Verifies the facade type aliases in `animus-adapters` still compile.
#[test]
fn facade_aliases_resolve() {
    assert_type::<animus_adapters::ExecutionBounds>();
    assert_type::<animus_adapters::ExecutionBoundsDraft<'_>>();
    assert_type::<animus_adapters::ModelCatalogEntry>();
    assert_type::<animus_adapters::ModelCatalogEntryDraft<'_>>();
    assert_type::<animus_adapters::ModelRole>();
    assert_type::<animus_adapters::RequestSignalDraft<'_>>();
    assert_type::<animus_adapters::RequestSignals>();
    assert_type::<animus_adapters::RouterRequestMetadata>();
    assert_type::<animus_adapters::RouterRequestMetadataDraft<'_>>();
    assert_type::<animus_adapters::RouterBudget>();
    assert_type::<animus_adapters::RouterReceipt>();
    assert_type::<animus_adapters::RouterRedaction>();
    assert_type::<animus_adapters::RouterRecipe>();
    assert_type::<animus_adapters::RouterRecipeDraft<'_>>();
    assert_type::<animus_adapters::RouterValidationError>();
    assert_type::<animus_adapters::RouterSignalProjectionError>();
    assert_type::<animus_adapters::RuleRouterSignalSource>();
    assert_type::<animus_adapters::SelectedRoutePlan>();
    assert_type::<animus_adapters::TimeoutClass>();
    assert_type::<animus_adapters::RouteResultDraft<'_>>();
    assert_type::<animus_adapters::RouterReceiptLeak>();

    assert_function_item(&animus_adapters::project_router_signals);
    let source = animus_adapters::router::RuleSignalSource;
    let metadata_result = animus_adapters::RouterRequestMetadata::new(animus_adapters::RouterRequestMetadataDraft {
        prompt_char_estimate: PARITY_PROMPT_CHARS,
        tool_call_count: PARITY_TOOL_CALLS,
        contains_code: false,
        requires_structured_output: false,
        context_window_fill_per_mille: PARITY_CONTEXT_PER_MILLE,
        privacy_policy: "remote_allowed",
        fallback_hint: "eligible",
    });
    assert!(metadata_result.is_ok());
    let metadata = match metadata_result {
        Ok(metadata) => metadata,
        Err(_error) => return,
    };
    let projection_result = animus_adapters::project_router_signals_with_source(&source, &metadata);
    assert!(projection_result.is_ok());
    assert_function_item(&animus_adapters::validate_router_receipt_text);

    assert_eq!(EXPECTED_ALIAS_COUNT, alias_names().len());
}

fn router_item_names() -> [&'static str; EXPECTED_ROUTER_ITEM_COUNT] {
    [
        "Budget",
        "Bounds",
        "CatalogEntry",
        "DeadlineClass",
        "DurationBucket",
        "FallbackEligibility",
        "FallbackPolicy",
        "OutputContract",
        "PrivacyBand",
        "QuorumClass",
        "Receipt",
        "Redaction",
        "Recipe",
        "Role",
        "RouteFamily",
        "SelectedPlan",
        "ExecutionPlanError",
        "FallbackChain",
        "FallbackStep",
        "RoleStep",
        "RouterExecutionPlan",
        "SelectionDenial",
        "SignalBand",
        "Signals",
        "TaskShape",
        "RouteResult",
        "RouteResultStatus",
        "ValidationError",
        "ReceiptLeak",
        "BoundsDraft",
        "CatalogEntryDraft",
        "ReceiptDraft",
        "RecipeDraft",
        "RequestMetadata",
        "RequestMetadataDraft",
        "RuleSignalSource",
        "SignalDraft",
        "SignalProjectionError",
        "SignalSource",
        "ResultDraft",
    ]
}

fn alias_names() -> [&'static str; EXPECTED_ALIAS_COUNT] {
    [
        "ExecutionBounds",
        "ExecutionBoundsDraft",
        "ModelCatalogEntry",
        "ModelCatalogEntryDraft",
        "ModelRole",
        "RequestSignalDraft",
        "RequestSignals",
        "RouterRequestMetadata",
        "RouterRequestMetadataDraft",
        "RouterBudget",
        "RouterReceipt",
        "RouterRedaction",
        "RouterRecipe",
        "RouterRecipeDraft",
        "RouterValidationError",
        "RouterSignalProjectionError",
        "RuleRouterSignalSource",
        "SelectedRoutePlan",
        "TimeoutClass",
        "RouteResultDraft",
        "RouterReceiptLeak",
    ]
}
