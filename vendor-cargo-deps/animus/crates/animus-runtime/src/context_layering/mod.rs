//! Pure Zerostack-style context file layering contracts.
//!
//! Hosts own discovery, directory walking, file reads, ignore handling, path
//! canonicalization, clocks, and persistence. Runtime core only evaluates
//! already-loaded descriptors and emits safe prompt sections/receipts.

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

pub use core::{
    assemble_context_layers, blake3_context_layer_fingerprint, validate_context_descriptor,
};
pub use data::{
    ContextLayerAdmissionPolicy, ContextLayerBoundaryEffect, ContextLayerClass,
    ContextLayerDescriptor, ContextLayerDiagnostic, ContextLayerDiagnosticKind,
    ContextLayerDiscoveryBoundary, ContextLayerFreshness, ContextLayerInclusionPolicy,
    ContextLayerPlan, ContextLayerPolicySwitch, ContextLayerReceipt, ContextLayerReceiptEntry,
    ContextLayerRedaction, ContextLayerSourceKind, ContextLayerTrust, ContextLayerValidationResult,
};

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const CONTEXT_LAYER_HASH_DOMAIN: &str = "animus.context-layer.v1";
pub(crate) const EMPTY_COUNT: usize = 0;
pub(crate) const FIELD_BODY: &str = "body";
pub(crate) const FIELD_CLASS: &str = "class";
pub(crate) const FIELD_DISCOVERY: &str = "discovery";
pub(crate) const FIELD_PRECEDENCE: &str = "precedence";
pub(crate) const FIELD_SOURCE: &str = "source";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";

#[cfg(test)]
mod tests;
