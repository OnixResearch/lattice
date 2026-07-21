//! Pure Animus-owned quality-record contracts inspired by Qualifier.
//!
//! Qualifier remains a bounded reference. Runtime core owns safe DTOs,
//! deterministic BLAKE3 identities, lifecycle transitions, span freshness, and
//! guidance inventory logic. Filesystem sidecars, JSONL storage, clocks, Git,
//! discovery, and CLI execution remain host/adapter concerns.

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
    active_quality_records, blake3_quality_fingerprint, classify_quality_span_freshness,
    export_quality_sidecar_record, quality_dependency_isolation_report, quality_record_id,
    report_quality_after_edit, resolve_quality_agents_guidance, roundtrip_unknown_sidecar_record,
    validate_quality_record,
};
pub use data::{
    QualityAgentsGuidanceError, QualityAgentsGuidancePage, QualityAnnotationBody,
    QualityDependencyIsolationReport, QualityDependencyRequirement, QualityDiagnostic,
    QualityDiagnosticKind, QualityEditReport, QualityFingerprint, QualityFreshness,
    QualityIssuerClassification, QualityLifecycleReport, QualityMetadataField, QualityRecordBody,
    QualityRecordEnvelope, QualityRecordKind, QualitySidecarExport, QualitySpan,
    QualitySpanFreshnessReport, QualityUnknownSidecarRecord, QualityValidationResult,
};

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const CURRENT_QUALITY_VERSION: u32 = 1;
pub(crate) const FIELD_BODY: &str = "body";
pub(crate) const FIELD_ID: &str = "id";
pub(crate) const FIELD_ISSUER: &str = "issuer";
pub(crate) const FIELD_LIFECYCLE: &str = "lifecycle";
pub(crate) const FIELD_REDACTION: &str = "redaction";
pub(crate) const FIELD_SPAN: &str = "span";
pub(crate) const FIELD_SUBJECT: &str = "subject";
pub(crate) const FIELD_VERSION: &str = "version";
pub(crate) const FULL_LINE_HASH_DOMAIN: &str = "animus.quality.span-lines.v1";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const QUALITY_HASH_DOMAIN: &str = "animus.quality-record.v1";
pub(crate) const REQUIRED_COUNT: u32 = 1;

#[cfg(test)]
mod tests;
