//! Pure provider-neutral private model-request snapshot contracts.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::too_many_parameters,
        tigerstyle::usize_in_public_api,
        reason = "published snapshot compatibility contracts retain their typed stable signatures and collection limits"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::compound_condition,
        tigerstyle::function_length,
        reason = "snapshot codecs return typed validation errors for private bytes instead of panicking assertions"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::borrowed_argument_types,
        reason = "private codec append helpers require mutable Vec ownership and capacity, not a slice"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        reason = "the no_std snapshot component uses explicit sibling DTO imports to keep codec ownership visible"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::bool_naming,
        tigerstyle::numeric_units,
        reason = "published snapshot vocabulary uses provider-neutral protocol field names fixed by its versioned contract"
    )
)]

#[path = "canonical.rs"]
mod codec;
mod core;
mod data;
#[path = "receipt.rs"]
mod evidence;

pub use codec::{
    decode_model_request_provider_terminal_private_payload,
    decode_model_request_snapshot_private_payload, model_request_budget_identity,
    model_request_canonical_identity, model_request_output_control_identity,
    model_request_provider_terminal_private_payload, model_request_route_identity,
    model_request_tool_schema_set_identity,
};
pub use core::{
    classify_model_request_snapshot_recovery, observe_model_request_provider_terminal,
    observe_model_request_snapshot_commit, plan_initial_model_request_dispatch,
    plan_model_request_snapshot_commit, plan_same_model_request_retry,
    validate_model_request_provider_terminal_observation, validate_model_request_snapshot,
};
pub use data::*;
pub use evidence::{
    project_model_request_snapshot_receipt, validate_model_request_snapshot_receipt,
};

#[cfg(test)]
mod tests;
