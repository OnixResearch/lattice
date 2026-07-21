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
use std::collections::{BTreeMap, BTreeSet};

use super::{
    BLAKE3_ALGORITHM, CURRENT_QUALITY_VERSION, FIELD_BODY, FIELD_ID, FIELD_ISSUER, FIELD_LIFECYCLE,
    FIELD_REDACTION, FIELD_SPAN, FIELD_SUBJECT, FIELD_VERSION, FULL_LINE_HASH_DOMAIN,
    HASH_PART_SEPARATOR, QUALITY_HASH_DOMAIN, REQUIRED_COUNT,
    data::{
        QualityAgentsGuidanceError, QualityAgentsGuidancePage, QualityAnnotationBody,
        QualityDependencyIsolationReport, QualityDependencyRequirement, QualityDiagnostic,
        QualityDiagnosticKind, QualityEditReport, QualityFingerprint, QualityFreshness,
        QualityIssuerClassification, QualityLifecycleReport, QualityMetadataField,
        QualityRecordEnvelope, QualitySidecarExport, QualitySpan, QualitySpanFreshnessReport,
        QualityUnknownSidecarRecord, QualityValidationResult,
    },
};

const AGENTS_TOPIC_CONCEPTS: &str = "concepts";
const AGENTS_TOPIC_ORIENTATION: &str = "orientation";
const AGENTS_TOPIC_PITFALLS: &str = "pitfalls";
const AGENTS_TOPIC_WORKFLOWS: &str = "workflows";
const COLUMN_MIN: u32 = 1;
const MAX_BODY_CHARS: usize = 4096;
const MAX_TOPIC_COUNT: usize = 4;
const RAW_CREDENTIAL_MARKER: &str = "API_KEY";
const RAW_HOST_PATH_MARKER: &str = "/home/";
const RAW_MODEL_MARKER: &str = "model_output:";
const RAW_PROMPT_MARKER: &str = "RAW_PROMPT";
const RAW_PROVIDER_MARKER: &str = "provider_payload";
const RAW_TOOL_INPUT_MARKER: &str = "tool_input:";
const RAW_TOOL_OUTPUT_MARKER: &str = "tool_output:";
const URI_SEPARATOR: &str = "://";
const URN_PREFIX: &str = "urn:";
const WINDOWS_PATH_MARKER: &str = ":\\";

/// Computes a domain-separated BLAKE3 fingerprint.
#[must_use]
pub fn blake3_quality_fingerprint(label: &str, bytes: &[u8]) -> QualityFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(QUALITY_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    QualityFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Computes the canonical content-addressed quality record ID.
#[must_use]
pub fn quality_record_id(record: &QualityRecordEnvelope) -> QualityFingerprint {
    let canonical = canonical_record_without_id(record);
    blake3_quality_fingerprint("record", canonical.as_bytes())
}

/// Validates a safe quality record envelope.
///
/// # Errors
///
/// Returns diagnostics for unsupported versions, empty subject or issuer,
/// non-URI issuer, malformed issuer classification, mismatched IDs, missing body
/// fields, unsafe body text, invalid spans, references, and supersession issues.
pub fn validate_quality_record(record: &QualityRecordEnvelope) -> QualityValidationResult {
    let mut diagnostics = Vec::new();
    if record.version != CURRENT_QUALITY_VERSION {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::UnsupportedVersion,
            FIELD_VERSION,
            "quality record version is unsupported",
        ));
    }
    require_text(&record.subject, FIELD_SUBJECT, &mut diagnostics);
    require_issuer(&record.issuer, &mut diagnostics);
    require_text(&record.created_at_label, FIELD_BODY, &mut diagnostics);
    if record.record_type != record.body.kind() {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::InvalidField,
            FIELD_BODY,
            "record type must match typed body kind",
        ));
    }
    require_hash(&record.id, FIELD_ID, &mut diagnostics);
    let expected = quality_record_id(record);
    if record.id != expected {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::InvalidId,
            FIELD_ID,
            "quality record id does not match canonical body",
        ));
    }
    validate_body(record.body.annotation(), &mut diagnostics);
    if matches!(
        record.issuer_classification,
        Some(QualityIssuerClassification::Unknown)
    ) {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::InvalidField,
            FIELD_ISSUER,
            "issuer classification must be specific when present",
        ));
    }
    to_validation_result(diagnostics)
}

/// Exports a safe record to a Qualifier-compatible sidecar projection.
///
/// # Errors
///
/// Returns validation diagnostics when the record is not safe for projection.
pub fn export_quality_sidecar_record(
    record: &QualityRecordEnvelope,
) -> Result<QualitySidecarExport, Vec<QualityDiagnostic>> {
    validate_quality_record(record)?;
    let body = record.body.annotation();
    Ok(QualitySidecarExport {
        id: record.id.clone(),
        subject: record.subject.clone(),
        kind: body.kind.clone(),
        summary: body.summary.clone(),
        redacted: true,
        references: sorted(body.references.clone()),
        supersedes: sorted(body.supersedes.clone()),
        tags: sorted(body.tags.clone()),
    })
}

/// Creates an opaque passthrough handle for unknown sidecar records.
#[must_use]
pub fn roundtrip_unknown_sidecar_record(raw_line: &str) -> QualityUnknownSidecarRecord {
    QualityUnknownSidecarRecord {
        raw_hash: blake3_quality_fingerprint("unknown-sidecar", raw_line.as_bytes()),
        preserves_roundtrip: true,
        exposes_raw_body_to_core: false,
    }
}

/// Reports default dependency isolation for reusable core contracts.
#[must_use]
pub const fn quality_dependency_isolation_report() -> QualityDependencyIsolationReport {
    QualityDependencyIsolationReport {
        qualifier_crate: QualityDependencyRequirement::NotRequired,
        sidecar_discovery: QualityDependencyRequirement::NotRequired,
        git_diff: QualityDependencyRequirement::NotRequired,
        sidecar_compaction: QualityDependencyRequirement::NotRequired,
    }
}

/// Computes active quality records using pure lifecycle semantics.
#[must_use]
pub fn active_quality_records(records: &[QualityRecordEnvelope]) -> QualityLifecycleReport {
    let mut diagnostics = Vec::new();
    let mut by_id = BTreeMap::new();
    for record in records {
        if by_id.insert(record.id.value_hex.clone(), record).is_some() {
            diagnostics.push(QualityDiagnostic::new(
                QualityDiagnosticKind::InvalidLifecycle,
                FIELD_LIFECYCLE,
                "duplicate quality record id",
            ));
        }
    }
    let mut superseded = BTreeSet::new();
    let mut referenced_context_ids = BTreeSet::new();
    for record in records {
        let body = record.body.annotation();
        for reference in &body.references {
            if reference == &record.id.value_hex {
                diagnostics.push(QualityDiagnostic::new(
                    QualityDiagnosticKind::InvalidLifecycle,
                    FIELD_LIFECYCLE,
                    "quality record cannot reference itself",
                ));
            }
            referenced_context_ids.insert(reference.clone());
        }
        for superseded_id in &body.supersedes {
            if superseded_id == &record.id.value_hex {
                diagnostics.push(QualityDiagnostic::new(
                    QualityDiagnosticKind::InvalidLifecycle,
                    FIELD_LIFECYCLE,
                    "quality record cannot supersede itself",
                ));
            }
            if let Some(prior) = by_id.get(superseded_id) {
                if prior.subject != record.subject {
                    diagnostics.push(QualityDiagnostic::new(
                        QualityDiagnosticKind::InvalidLifecycle,
                        FIELD_LIFECYCLE,
                        "quality record cannot supersede a different subject",
                    ));
                }
            }
            superseded.insert(superseded_id.clone());
        }
    }
    detect_supersession_cycles(records, &mut diagnostics);
    let mut active_ids: Vec<String> = records
        .iter()
        .map(|record| record.id.value_hex.clone())
        .filter(|id| !superseded.contains(id))
        .collect();
    active_ids.sort();
    QualityLifecycleReport {
        active_ids,
        referenced_context_ids: referenced_context_ids.into_iter().collect(),
        diagnostics,
    }
}

/// Surfaces records after edits without auto-emitting resolve records.
#[must_use]
pub fn report_quality_after_edit(records: &[QualityRecordEnvelope]) -> QualityEditReport {
    let lifecycle = active_quality_records(records);
    QualityEditReport {
        surfaced_record_ids: lifecycle.active_ids,
        emitted_resolve_record: false,
    }
}

/// Classifies span freshness over caller-supplied lines.
#[must_use]
pub fn classify_quality_span_freshness(
    lines: &[String],
    span: &QualitySpan,
) -> QualitySpanFreshnessReport {
    let diagnostics = validate_span(span);
    if !diagnostics.is_empty() {
        return QualitySpanFreshnessReport {
            state: QualityFreshness::Missing,
            expected_hash: span.content_hash.clone(),
            actual_hash: None,
            diagnostics,
        };
    }
    let start = (span.start_line - REQUIRED_COUNT) as usize;
    let end = span.end_line as usize;
    if end > lines.len() || start >= lines.len() {
        return QualitySpanFreshnessReport {
            state: QualityFreshness::Missing,
            expected_hash: span.content_hash.clone(),
            actual_hash: None,
            diagnostics: Vec::new(),
        };
    }
    let actual_hash = hash_span_lines(&lines[start..end]);
    let state = match &span.content_hash {
        None => QualityFreshness::NoHash,
        Some(expected) if expected == &actual_hash => QualityFreshness::Fresh,
        Some(_) => QualityFreshness::Drifted,
    };
    QualitySpanFreshnessReport {
        state,
        expected_hash: span.content_hash.clone(),
        actual_hash: Some(actual_hash),
        diagnostics: Vec::new(),
    }
}

/// Resolves compact AGENTS-CLI-inspired guidance.
///
/// # Errors
///
/// Returns a structured error for unknown topics with the available topic list.
pub fn resolve_quality_agents_guidance(
    topic: Option<&str>,
) -> Result<QualityAgentsGuidancePage, QualityAgentsGuidanceError> {
    let available_topics = guidance_topics();
    let selected = topic.unwrap_or(AGENTS_TOPIC_ORIENTATION);
    let markdown = match selected {
        AGENTS_TOPIC_ORIENTATION => {
            "# Animus agents\nUse Animus for typed agent-runtime contracts. Prefer safe DTOs, explicit evidence, and host-owned side effects."
        }
        AGENTS_TOPIC_CONCEPTS => {
            "# Concepts\nQuality records are safe annotations with deterministic IDs, redaction, lifecycle state, and VCS-neutral spans."
        }
        AGENTS_TOPIC_WORKFLOWS => {
            "# Workflows\nValidate records, classify span freshness, then archive evidence only after focused positive and negative checks pass."
        }
        AGENTS_TOPIC_PITFALLS => {
            "# Pitfalls\nDo not store raw prompts, provider bodies, host paths, unbounded tool output, or Git-only assumptions in core records."
        }
        _ => {
            return Err(QualityAgentsGuidanceError {
                unknown_topic: selected.to_string(),
                available_topics,
            });
        }
    };
    Ok(QualityAgentsGuidancePage {
        topic: selected.to_string(),
        markdown: markdown.to_string(),
        available_topics,
        reads_network: false,
    })
}

fn canonical_body(body: &QualityAnnotationBody) -> String {
    let mut parts = Vec::new();
    parts.push(format!("kind={}", body.kind));
    parts.push(format!("summary={}", body.summary));
    if let Some(span) = &body.span {
        parts.push(format!(
            "span={}:{}:{}:{}:{}",
            span.start_line,
            span.end_line,
            span.start_column,
            span.end_column,
            span.content_hash
                .as_ref()
                .map_or("", |hash| hash.value_hex.as_str())
        ));
    }
    if let Some(suggested_fix) = &body.suggested_fix {
        parts.push(format!("suggested_fix={suggested_fix}"));
    }
    parts.push(format!(
        "references={}",
        sorted(body.references.clone()).join(",")
    ));
    parts.push(format!(
        "supersedes={}",
        sorted(body.supersedes.clone()).join(",")
    ));
    parts.push(format!("tags={}", sorted(body.tags.clone()).join(",")));
    parts.push(format!("metadata={}", canonical_metadata(&body.metadata)));
    parts.join(";")
}

fn canonical_metadata(metadata: &[QualityMetadataField]) -> String {
    let mut pairs: Vec<String> = metadata
        .iter()
        .map(|field| format!("{}={}", field.key, field.value))
        .collect();
    pairs.sort();
    pairs.join(",")
}

fn canonical_record_without_id(record: &QualityRecordEnvelope) -> String {
    let class = record
        .issuer_classification
        .map_or("", QualityIssuerClassification::as_str);
    format!(
        "version={};type={};subject={};issuer={};class={};created={};body={}",
        record.version,
        record.record_type.as_str(),
        record.subject,
        record.issuer,
        class,
        record.created_at_label,
        canonical_body(record.body.annotation())
    )
}

fn contains_unsafe_marker(text: &str) -> bool {
    text.contains(RAW_PROMPT_MARKER)
        || text.contains(RAW_PROVIDER_MARKER)
        || text.contains(RAW_TOOL_INPUT_MARKER)
        || text.contains(RAW_TOOL_OUTPUT_MARKER)
        || text.contains(RAW_MODEL_MARKER)
        || text.contains(RAW_CREDENTIAL_MARKER)
        || text.contains(RAW_HOST_PATH_MARKER)
        || text.contains(WINDOWS_PATH_MARKER)
}

fn detect_supersession_cycles(
    records: &[QualityRecordEnvelope],
    diagnostics: &mut Vec<QualityDiagnostic>,
) {
    let graph: BTreeMap<String, Vec<String>> = records
        .iter()
        .map(|record| {
            (
                record.id.value_hex.clone(),
                record.body.annotation().supersedes.clone(),
            )
        })
        .collect();
    for record in records {
        let mut seen = BTreeSet::new();
        let mut stack = record.body.annotation().supersedes.clone();
        while let Some(next) = stack.pop() {
            if next == record.id.value_hex || !seen.insert(next.clone()) {
                diagnostics.push(QualityDiagnostic::new(
                    QualityDiagnosticKind::InvalidLifecycle,
                    FIELD_LIFECYCLE,
                    "supersession cycle detected",
                ));
                break;
            }
            if let Some(parents) = graph.get(&next) {
                stack.extend(parents.clone());
            }
        }
    }
}

fn guidance_topics() -> Vec<String> {
    let mut topics = vec![
        AGENTS_TOPIC_ORIENTATION.to_string(),
        AGENTS_TOPIC_CONCEPTS.to_string(),
        AGENTS_TOPIC_WORKFLOWS.to_string(),
        AGENTS_TOPIC_PITFALLS.to_string(),
    ];
    assert_eq!(topics.len(), MAX_TOPIC_COUNT);
    topics.sort();
    topics
}

fn hash_span_lines(lines: &[String]) -> QualityFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FULL_LINE_HASH_DOMAIN.as_bytes());
    for line in lines {
        hasher.update(HASH_PART_SEPARATOR.as_bytes());
        hasher.update(line.as_bytes());
    }
    QualityFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

fn require_hash(
    fingerprint: &QualityFingerprint,
    field: &str,
    diagnostics: &mut Vec<QualityDiagnostic>,
) {
    if fingerprint.algorithm != BLAKE3_ALGORITHM || fingerprint.value_hex.trim().is_empty() {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::InvalidId,
            field,
            "quality fingerprints must use non-empty BLAKE3 values",
        ));
    }
}

fn require_issuer(issuer: &str, diagnostics: &mut Vec<QualityDiagnostic>) {
    require_text(issuer, FIELD_ISSUER, diagnostics);
    if !issuer.contains(URI_SEPARATOR) && !issuer.starts_with(URN_PREFIX) {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::InvalidField,
            FIELD_ISSUER,
            "quality record issuer must be a URI",
        ));
    }
}

fn require_text(text: &str, field: &str, diagnostics: &mut Vec<QualityDiagnostic>) {
    if text.trim().is_empty() {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::InvalidField,
            field,
            "required quality field is empty",
        ));
    }
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn to_validation_result(diagnostics: Vec<QualityDiagnostic>) -> QualityValidationResult {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_body(body: &QualityAnnotationBody, diagnostics: &mut Vec<QualityDiagnostic>) {
    require_text(&body.kind, FIELD_BODY, diagnostics);
    require_text(&body.summary, FIELD_BODY, diagnostics);
    validate_text(&body.kind, FIELD_BODY, diagnostics);
    validate_text(&body.summary, FIELD_REDACTION, diagnostics);
    if let Some(suggested_fix) = &body.suggested_fix {
        validate_text(suggested_fix, FIELD_REDACTION, diagnostics);
    }
    for text in body
        .references
        .iter()
        .chain(body.supersedes.iter())
        .chain(body.tags.iter())
    {
        validate_text(text, FIELD_BODY, diagnostics);
    }
    for field in &body.metadata {
        require_text(&field.key, FIELD_BODY, diagnostics);
        require_text(&field.value, FIELD_BODY, diagnostics);
        validate_text(&field.key, FIELD_REDACTION, diagnostics);
        validate_text(&field.value, FIELD_REDACTION, diagnostics);
    }
    if let Some(span) = &body.span {
        diagnostics.extend(validate_span(span));
    }
}

fn validate_span(span: &QualitySpan) -> Vec<QualityDiagnostic> {
    let mut diagnostics = Vec::new();
    if span.start_line < REQUIRED_COUNT
        || span.end_line < REQUIRED_COUNT
        || span.start_line > span.end_line
        || span.start_column < COLUMN_MIN
        || span.end_column < COLUMN_MIN
        || (span.start_line == span.end_line && span.start_column > span.end_column)
    {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::InvalidSpan,
            FIELD_SPAN,
            "quality span must use valid one-indexed ranges",
        ));
    }
    diagnostics
}

fn validate_text(text: &str, field: &str, diagnostics: &mut Vec<QualityDiagnostic>) {
    if text.chars().count() > MAX_BODY_CHARS || contains_unsafe_marker(text) {
        diagnostics.push(QualityDiagnostic::new(
            QualityDiagnosticKind::UnsafeBody,
            field,
            "quality record body contains unsafe or unbounded material",
        ));
    }
}
