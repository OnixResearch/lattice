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

use super::*;

const CREATED_LABEL: &str = "created.local";
const ISSUER: &str = "urn:animus:tester";
const KIND_CONCERN: &str = "concern";
const KIND_RESOLVE: &str = "resolve";
const KIND_WAIVER: &str = "waiver";
const LINE_ONE: &str = "fn main() {";
const LINE_THREE: &str = "}";
const LINE_TWO: &str = "    println!(\"ok\");";
const SUBJECT: &str = "src/main.rs";
const SUMMARY: &str = "Review the boundary before release.";
const SUGGESTED_FIX: &str = "Add a focused negative test.";
const ZERO_LINE: u32 = 0;

#[test]
fn scope_sidecar_unknown_and_dependency_isolation_stay_bounded() {
    // r[agent_harness_runtime.qualifier_quality.scope.boundary]
    let report = quality_dependency_isolation_report();
    assert_eq!(
        report.qualifier_crate,
        QualityDependencyRequirement::NotRequired
    );
    assert_eq!(
        report.sidecar_discovery,
        QualityDependencyRequirement::NotRequired
    );
    assert_eq!(report.git_diff, QualityDependencyRequirement::NotRequired);
    assert_eq!(
        report.sidecar_compaction,
        QualityDependencyRequirement::NotRequired
    );

    let unknown = roundtrip_unknown_sidecar_record("future opaque sidecar body");
    assert!(unknown.preserves_roundtrip);
    assert!(!unknown.exposes_raw_body_to_core);
    assert_eq!(unknown.raw_hash.algorithm, BLAKE3_ALGORITHM);
}

#[test]
fn canonical_ids_are_deterministic_across_equivalent_body_ordering() {
    // r[agent_harness_runtime.qualifier_quality.envelope.deterministic]
    let first = record(annotation_body(
        vec!["b", "a"],
        vec![metadata("z", "last"), metadata("a", "first")],
        Vec::new(),
        QualityRecordKind::Annotation,
    ));
    let second = record(annotation_body(
        vec!["a", "b"],
        vec![metadata("a", "first"), metadata("z", "last")],
        Vec::new(),
        QualityRecordKind::Annotation,
    ));

    assert_eq!(first.id, second.id);
    assert!(validate_quality_record(&first).is_ok());
    assert!(validate_quality_record(&second).is_ok());
}

#[test]
fn malformed_envelopes_and_unsafe_bodies_fail_closed() {
    // r[agent_harness_runtime.qualifier_quality.envelope.invalid]
    // r[agent_harness_runtime.qualifier_quality.scope.redaction]
    let mut invalid = record(annotation_body(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        QualityRecordKind::Annotation,
    ));
    invalid.version = CURRENT_QUALITY_VERSION + 1;
    invalid.subject.clear();
    invalid.issuer = "not-a-uri".to_string();
    invalid.id = QualityFingerprint::new("SHA256", "abc");
    invalid.body = QualityRecordBody::Annotation(QualityAnnotationBody {
        summary: "RAW_PROMPT provider_payload tool_input: tool_output: model_output: API_KEY /home/user/file".to_string(),
        ..invalid.body.annotation().clone()
    });

    let diagnostics = validate_quality_record(&invalid).expect_err("invalid record");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == QualityDiagnosticKind::UnsupportedVersion })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == QualityDiagnosticKind::InvalidId })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == QualityDiagnosticKind::UnsafeBody })
    );
}

#[test]
fn sidecar_export_preserves_safe_annotation_semantics() {
    // r[agent_harness_runtime.qualifier_quality.sidecar_adapter.export]
    // r[agent_harness_runtime.qualifier_quality.sidecar_adapter.unknown]
    // r[agent_harness_runtime.qualifier_quality.sidecar_adapter.isolated]
    let record = record(annotation_body(
        vec!["safety", "review"],
        vec![metadata("classification", "agent")],
        vec!["prior-id".to_string()],
        QualityRecordKind::Annotation,
    ));
    let export = export_quality_sidecar_record(&record).expect("safe export");

    assert_eq!(export.id, record.id);
    assert_eq!(export.subject, SUBJECT);
    assert_eq!(export.kind, KIND_CONCERN);
    assert_eq!(export.summary, SUMMARY);
    assert_eq!(export.references, vec!["prior-id".to_string()]);
    assert!(export.redacted);
}

#[test]
fn lifecycle_threads_references_supersedes_same_subject_and_never_auto_resolves() {
    // r[agent_harness_runtime.qualifier_quality.lifecycle.references]
    // r[agent_harness_runtime.qualifier_quality.lifecycle.supersession]
    // r[agent_harness_runtime.qualifier_quality.lifecycle.no_auto_resolve]
    let prior = record(annotation_body(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        QualityRecordKind::Annotation,
    ));
    let reference = record(annotation_body(
        Vec::new(),
        Vec::new(),
        vec![prior.id.value_hex.clone()],
        QualityRecordKind::Annotation,
    ));
    let active = active_quality_records(&[prior.clone(), reference.clone()]);
    assert!(active.diagnostics.is_empty());
    assert_eq!(active.active_ids.len(), 2);
    assert_eq!(
        active.referenced_context_ids,
        vec![prior.id.value_hex.clone()]
    );

    let mut resolve = record(annotation_body(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        QualityRecordKind::Resolve,
    ));
    resolve.body = QualityRecordBody::Resolve(QualityAnnotationBody {
        supersedes: vec![prior.id.value_hex.clone()],
        kind: KIND_RESOLVE.to_string(),
        ..resolve.body.annotation().clone()
    });
    resolve.id = quality_record_id(&resolve);
    let active = active_quality_records(&[prior.clone(), resolve.clone()]);
    assert!(!active.active_ids.contains(&prior.id.value_hex));
    assert!(active.active_ids.contains(&resolve.id.value_hex));

    let edit_report = report_quality_after_edit(&[prior.clone(), reference]);
    assert!(!edit_report.emitted_resolve_record);
    assert!(
        edit_report
            .surfaced_record_ids
            .contains(&prior.id.value_hex)
    );

    let mut cross_subject = resolve.clone();
    cross_subject.subject = "other.rs".to_string();
    let lifecycle = active_quality_records(&[prior.clone(), cross_subject]);
    assert!(
        lifecycle
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == QualityDiagnosticKind::InvalidLifecycle })
    );

    let mut cycle_a = prior.clone();
    let mut cycle_b = resolve;
    cycle_a.body = QualityRecordBody::Annotation(QualityAnnotationBody {
        supersedes: vec![cycle_b.id.value_hex.clone()],
        ..cycle_a.body.annotation().clone()
    });
    cycle_b.body = QualityRecordBody::Resolve(QualityAnnotationBody {
        supersedes: vec![cycle_a.id.value_hex.clone()],
        ..cycle_b.body.annotation().clone()
    });
    let lifecycle = active_quality_records(&[cycle_a, cycle_b]);
    assert!(
        lifecycle
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == QualityDiagnosticKind::InvalidLifecycle })
    );
}

#[test]
fn span_freshness_covers_states_and_invalid_spans() {
    // r[agent_harness_runtime.qualifier_quality.span_freshness.states]
    // r[agent_harness_runtime.qualifier_quality.span_freshness.invalid]
    let lines = lines();
    let no_hash_span = span(None);
    let no_hash = classify_quality_span_freshness(&lines, &no_hash_span);
    assert_eq!(no_hash.state, QualityFreshness::NoHash);
    let actual_hash = no_hash.actual_hash.expect("actual hash");

    let fresh = classify_quality_span_freshness(&lines, &span(Some(actual_hash.clone())));
    assert_eq!(fresh.state, QualityFreshness::Fresh);

    let changed = vec![
        LINE_ONE.to_string(),
        "changed".to_string(),
        LINE_THREE.to_string(),
    ];
    let drifted = classify_quality_span_freshness(&changed, &span(Some(actual_hash)));
    assert_eq!(drifted.state, QualityFreshness::Drifted);

    let missing = classify_quality_span_freshness(&[LINE_ONE.to_string()], &span(None));
    assert_eq!(missing.state, QualityFreshness::Missing);

    let invalid = classify_quality_span_freshness(
        &lines,
        &QualitySpan {
            start_line: ZERO_LINE,
            ..span(None)
        },
    );
    assert_eq!(invalid.state, QualityFreshness::Missing);
    assert!(
        invalid
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == QualityDiagnosticKind::InvalidSpan })
    );
}

#[test]
fn agents_guidance_is_deterministic_and_reports_unknown_topics() {
    // r[agent_harness_runtime.qualifier_quality.agents_cli.orientation]
    // r[agent_harness_runtime.qualifier_quality.agents_cli.topics]
    let orientation = resolve_quality_agents_guidance(None).expect("orientation");
    assert_eq!(orientation.topic, "orientation");
    assert!(!orientation.reads_network);
    assert!(orientation.markdown.contains("Animus agents"));

    for topic in orientation.available_topics.clone() {
        let page = resolve_quality_agents_guidance(Some(&topic)).expect("known topic");
        assert_eq!(page.topic, topic);
    }

    let error = resolve_quality_agents_guidance(Some("unknown")).expect_err("unknown topic");
    assert_eq!(error.unknown_topic, "unknown");
    assert_eq!(error.available_topics, orientation.available_topics);
}

fn annotation_body(
    tags: Vec<&str>,
    metadata_fields: Vec<QualityMetadataField>,
    references: Vec<String>,
    kind: QualityRecordKind,
) -> QualityRecordBody {
    let body = QualityAnnotationBody {
        kind: match kind {
            QualityRecordKind::Annotation => KIND_CONCERN,
            QualityRecordKind::Resolve => KIND_RESOLVE,
            QualityRecordKind::Waiver => KIND_WAIVER,
        }
        .to_string(),
        summary: SUMMARY.to_string(),
        span: Some(span(None)),
        suggested_fix: Some(SUGGESTED_FIX.to_string()),
        references,
        supersedes: Vec::new(),
        tags: tags
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
        metadata: metadata_fields,
    };
    match kind {
        QualityRecordKind::Annotation => QualityRecordBody::Annotation(body),
        QualityRecordKind::Resolve => QualityRecordBody::Resolve(body),
        QualityRecordKind::Waiver => QualityRecordBody::Waiver(body),
    }
}

fn lines() -> Vec<String> {
    vec![
        LINE_ONE.to_string(),
        LINE_TWO.to_string(),
        LINE_THREE.to_string(),
    ]
}

fn metadata(key: &str, value: &str) -> QualityMetadataField {
    QualityMetadataField {
        key: key.to_string(),
        value: value.to_string(),
    }
}

fn record(body: QualityRecordBody) -> QualityRecordEnvelope {
    let mut record = QualityRecordEnvelope {
        version: CURRENT_QUALITY_VERSION,
        record_type: body.kind(),
        subject: SUBJECT.to_string(),
        issuer: ISSUER.to_string(),
        issuer_classification: Some(QualityIssuerClassification::Agent),
        created_at_label: CREATED_LABEL.to_string(),
        id: QualityFingerprint::new(BLAKE3_ALGORITHM, "placeholder"),
        body,
    };
    record.id = quality_record_id(&record);
    record
}

fn span(content_hash: Option<QualityFingerprint>) -> QualitySpan {
    QualitySpan {
        start_line: 1,
        end_line: 2,
        start_column: 1,
        end_column: 80,
        content_hash,
    }
}
