//! Pure Animus-owned quality-record contracts inspired by Qualifier.
//!
//! Qualifier remains a bounded reference. Runtime core owns safe DTOs,
//! deterministic BLAKE3 identities, lifecycle transitions, span freshness, and
//! guidance inventory logic. Filesystem sidecars, JSONL storage, clocks, Git,
//! discovery, and CLI execution remain host/adapter concerns.

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

pub use core::active_quality_records;
pub use core::blake3_quality_fingerprint;
pub use core::classify_quality_span_freshness;
pub use core::export_quality_sidecar_record;
pub use core::quality_dependency_isolation_report;
pub use core::quality_record_id;
pub use core::report_quality_after_edit;
pub use core::resolve_quality_agents_guidance;
pub use core::roundtrip_unknown_sidecar_record;
pub use core::validate_quality_record;

pub use data::QualityAgentsGuidanceError;
pub use data::QualityAgentsGuidancePage;
pub use data::QualityAnnotationBody;
pub use data::QualityDependencyIsolationReport;
pub use data::QualityDependencyRequirement;
pub use data::QualityDiagnostic;
pub use data::QualityDiagnosticKind;
pub use data::QualityEditReport;
pub use data::QualityFingerprint;
pub use data::QualityFreshness;
pub use data::QualityIssuerClassification;
pub use data::QualityLifecycleReport;
pub use data::QualityMetadataField;
pub use data::QualityRecordBody;
pub use data::QualityRecordEnvelope;
pub use data::QualityRecordKind;
pub use data::QualitySidecarExport;
pub use data::QualitySpan;
pub use data::QualitySpanFreshnessReport;
pub use data::QualityUnknownSidecarRecord;
pub use data::QualityValidationResult;

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
