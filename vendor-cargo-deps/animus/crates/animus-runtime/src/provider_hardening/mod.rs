//! Pure provider registry hardening contracts.

// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while
// drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
mod core;
mod data;
mod stream;

pub use core::blake3_provider_hardening_fingerprint;
pub use core::filter_agent_model;
pub use core::negotiate_model_capability;
pub use core::provider_selection_receipt;
pub use core::resolve_extra_body_descriptor;
pub use core::resolve_provider_api_style;
pub use core::validate_model_capability_matrix;
pub use core::validate_provider_header;

pub use data::ModelCapabilityDecisionStatus;
pub use data::ModelCapabilityDowngrade;
pub use data::ModelCapabilityDowngradePolicy;
pub use data::ModelCapabilityIssue;
pub use data::ModelCapabilityIssueKind;
pub use data::ModelCapabilityNegotiationDecision;
pub use data::ModelCapabilityNegotiationInput;
pub use data::ProviderApiStyle;
pub use data::ProviderApiStylePolicy;
pub use data::ProviderDiagnostic;
pub use data::ProviderDiagnosticKind;
pub use data::ProviderEndpointSensitivity;
pub use data::ProviderExtraBodyDescriptor;
pub use data::ProviderHeaderDescriptor;
pub use data::ProviderHeaderResolution;
pub use data::ProviderHeaderValueSource;
pub use data::ProviderModelCapabilityMatrix;
pub use data::ProviderModelClass;
pub use data::ProviderModelContentPartClass;
pub use data::ProviderModelMetadata;
pub use data::ProviderPrivacyPosture;
pub use data::ProviderSecretSourceReference;
pub use data::ProviderSelectionReceipt;
pub use data::ProviderSelectionReceiptInput;
pub use data::ProviderStreamChannel;
pub use data::ProviderStructuredOutputMode;
pub use data::ProviderValidationResult;
pub use stream::ProviderStreamBudgetKind;
pub use stream::ProviderStreamParseFailure;
pub use stream::ProviderStreamParseFailureKind;
pub use stream::ProviderStreamParseFailureReport;
pub use stream::ProviderStreamParseReport;
pub use stream::ProviderStreamParserBudget;
pub use stream::ProviderStreamParserInput;
pub use stream::ProviderStreamParserInventoryEntry;
pub use stream::ProviderStreamParserReceipt;
pub use stream::ProviderStreamParserStatus;
pub use stream::parse_provider_stream_chunks;
pub use stream::provider_stream_parser_inventory;
pub use stream::validate_provider_stream_parser_inventory;
pub use stream::validate_provider_stream_parser_receipt;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const PROVIDER_HARDENING_HASH_DOMAIN: &str = "animus.provider-hardening.v1";
pub(crate) const FIELD_API_STYLE: &str = "api_style";
pub(crate) const FIELD_EXTRA_BODY: &str = "extra_body";
pub(crate) const FIELD_HEADER: &str = "header";
pub(crate) const FIELD_MODEL: &str = "model";

#[cfg(test)]
mod tests;
