//! Pure provider registry hardening contracts.

// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
mod core;
mod data;
mod stream;

pub use core::{
    blake3_provider_hardening_fingerprint, filter_agent_model, negotiate_model_capability,
    provider_selection_receipt, resolve_extra_body_descriptor, resolve_provider_api_style,
    validate_model_capability_matrix, validate_provider_header,
};
pub use data::{
    ModelCapabilityDecisionStatus, ModelCapabilityDowngrade, ModelCapabilityDowngradePolicy,
    ModelCapabilityIssue, ModelCapabilityIssueKind, ModelCapabilityNegotiationDecision,
    ModelCapabilityNegotiationInput, ProviderApiStyle, ProviderApiStylePolicy, ProviderDiagnostic,
    ProviderDiagnosticKind, ProviderEndpointSensitivity, ProviderExtraBodyDescriptor,
    ProviderHeaderDescriptor, ProviderHeaderResolution, ProviderHeaderValueSource,
    ProviderModelCapabilityMatrix, ProviderModelClass, ProviderModelContentPartClass,
    ProviderModelMetadata, ProviderPrivacyPosture, ProviderSecretSourceReference,
    ProviderSelectionReceipt, ProviderSelectionReceiptInput, ProviderStreamChannel,
    ProviderStructuredOutputMode, ProviderValidationResult,
};
pub use stream::{
    ProviderStreamBudgetKind, ProviderStreamParseFailure, ProviderStreamParseFailureKind,
    ProviderStreamParseFailureReport, ProviderStreamParseReport, ProviderStreamParserBudget,
    ProviderStreamParserInput, ProviderStreamParserInventoryEntry, ProviderStreamParserReceipt,
    ProviderStreamParserStatus, parse_provider_stream_chunks, provider_stream_parser_inventory,
    validate_provider_stream_parser_inventory, validate_provider_stream_parser_receipt,
};

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const PROVIDER_HARDENING_HASH_DOMAIN: &str = "animus.provider-hardening.v1";
pub(crate) const FIELD_API_STYLE: &str = "api_style";
pub(crate) const FIELD_EXTRA_BODY: &str = "extra_body";
pub(crate) const FIELD_HEADER: &str = "header";
pub(crate) const FIELD_MODEL: &str = "model";

#[cfg(test)]
mod tests;
