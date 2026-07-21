//! `#![no_std]` model-router decision core for `animus`.
//!
//! This crate owns recipe/role/output-contract DTOs, signal projection, recipe
//! selection, execution-plan materialization, route-result aggregation, and
//! safe receipt assembly. It defines its own validation helpers locally to avoid a circular
//! dependency on `animus-adapters` and depends on the BLAKE3 workspace crate for receipt
//! identity.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![allow(unknown_lints, module_file_count)]
#![forbid(unsafe_code)]

extern crate alloc;

// ── Sub-modules ──────────────────────────────────────────────────────────────

mod boundary;
mod choose;
mod fairness;
mod input;
mod limit;
pub(crate) mod outcome;
mod policy;
mod projection;
mod schema;

/// Evidence module (edition-2024 file-style entry point).
mod evidence;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod test_support;

// ── Public re-exports ────────────────────────────────────────────────────────

pub use boundary::AdapterContractError;
pub use choose::{
    Budget, CatalogEntry, CatalogEntryDraft, RotatingSelectionRequest, SelectedPlan,
    SelectionDenial, select_recipe, select_recipe_with_rotation,
};
pub use evidence::{
    DurationBucket, ExecutionPlanError, FallbackChain, FallbackStep, QuorumClass, Receipt,
    ReceiptDraft, ReceiptLeak, Redaction, RoleStep, RouterExecutionPlan,
    materialize_execution_plan, materialize_execution_plan_with_rotation, validate_receipt_text,
};
pub use fairness::{
    RouterRotationCursor, RouterRotationCursorClass, RouterRotationCursorDraft,
    RouterRotationDecision, RouterRotationError, RouterRotationPolicy, RouterRotationReceipt,
    RouterRotationReceiptDraft, RouterRotationSnapshot, RouterRotationSnapshotDraft,
    RouterRotationStaleCursorPolicy, rotate_eligible_candidates,
};
pub use input::{
    RequestMetadata, RequestMetadataDraft, RuleSignalSource, SignalProjectionError, SignalSource,
    project_signals,
};
pub use limit::{Bounds, BoundsDraft, BudgetLedger, DeadlineClass, FallbackPolicy};
pub use outcome::{
    BudgetExhausted, BudgetKind, FallbackDecision, FallbackRouteReason, ResultDraft, RouteResult,
    RouteResultStatus, StepFailure, evaluate_fallback,
};
pub use policy::ROUTER_RECIPE_POLICY_JSON;
pub use projection::{
    FallbackEligibility, PrivacyBand, SignalBand, SignalDraft, Signals, TaskShape,
};
pub use schema::{OutputContract, Recipe, RecipeDraft, Role, RouteFamily};

pub(crate) type FieldName = boundary::FieldName;
pub(crate) type SafeHash = boundary::SafeHash;

pub(crate) fn normalized_adapter_field(
    field_name: FieldName,
    field_text: &str,
) -> Result<alloc::string::String, AdapterContractError> {
    assert!(
        field_name.as_str().chars().count() >= boundary::MIN_ADAPTER_FIELD_CHARS,
        "diagnostic field name must be non-empty"
    );
    boundary::normalized_adapter_field(field_name, field_text)
}

// ── Shared helpers ───────────────────────────────────────────────────────────

const HASH_SEPARATOR: &str = "|";

/// Router validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    /// Generic adapter field contract error.
    Contract(AdapterContractError),
    /// Required role list was empty.
    EmptyRoles,
    /// A role appeared more than once.
    DuplicateRole {
        /// Duplicated role.
        role: Role,
    },
    /// Fan-out or parallelism was unbounded or zero.
    UnboundedFanOut,
    /// Quorum was zero.
    ZeroQuorum,
    /// Quorum required more successes than possible candidates.
    ImpossibleQuorum,
    /// Repair attempts were omitted and therefore unlimited.
    UnlimitedRepair,
    /// Fallback shape was disallowed by bounds.
    FallbackDisallowed,
}

impl From<AdapterContractError> for ValidationError {
    fn from(error: AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

pub(crate) fn parse_value<T>(
    field_name: FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, AdapterContractError> {
    let normalized = normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or_else(|| AdapterContractError::UnsupportedRouterValue {
        field: field_name.as_str(),
        value: normalized,
    })
}

pub(crate) fn hash_parts(parts: &[&str]) -> alloc::string::String {
    let mut text = alloc::string::String::new();
    let part_count = parts.len();
    assert!(part_count > 0, "hash input must name at least one part");
    assert!(HASH_SEPARATOR.len() == 1, "hash separator must stay stable");
    text.reserve(part_count.saturating_mul(boundary::MIN_ADAPTER_FIELD_CHARS));
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    alloc::format!("{}", blake3::hash(text.as_bytes()).to_hex())
}
