#[test]
fn summary_is_read_only_and_record_append_is_explicit() {
    // r[verify agent_harness_runtime.hermes_review_reports.metrics_boundary.summary_readonly]
    // r[verify agent_harness_runtime.hermes_review_reports.metrics_boundary.record_append]
    let first = record(RecordInput {
        markdown: "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=test promoted=no] Add parser fixture.\n",
        sequence: SEQUENCE_ONE,
    })
    .unwrap();
    let second = record(RecordInput {
        markdown: "VERDICT: PASS\n\n## Findings\n- [class=lint scope=adapter route=test promoted=yes] Existing parser fixture.\n",
        sequence: SEQUENCE_TWO,
    })
    .unwrap();
    let records = alloc::vec![first.clone(), second];
    let before_hash = alloc::string::String::from(first.record_hash());
    let summary = super::summarize_review_metrics(records.as_slice());

    assert_eq!(summary.record_count(), TWO_FINDING_COUNT);
    assert_eq!(summary.finding_count(), TWO_FINDING_COUNT);
    assert_eq!(summary.warn_count(), SINGLE_FINDING_COUNT);
    assert_eq!(summary.pass_count(), SINGLE_FINDING_COUNT);
    assert_eq!(records[ZERO_COUNT].record_hash(), before_hash.as_str());
    assert_eq!(
        first.findings()[ZERO_COUNT].summary_hash().len(),
        crate::SAFE_HASH_HEX_CHARS
    );
}

#[test]
fn record_append_rejects_unsafe_metadata() {
    // r[verify agent_harness_runtime.hermes_review_reports.metrics_boundary.record_append]
    let report = super::parse_review_report(&draft(
        "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=test promoted=no] raw prompt: secret instructions\n",
    ))
    .unwrap();
    let error = super::append_review_metric_record(
        &report,
        &super::ReviewMetricRecordDraft {
            sequence: SEQUENCE_ONE,
        },
    )
    .expect_err("raw prompt marker should be rejected");
    assert_eq!(error.code(), super::ReviewDiagnosticCode::UnsafeMetadata);

    let path_draft = super::ReviewReportDraft {
        markdown: "VERDICT: PASS\n\n## Findings\n",
        source: "/home/example/private",
        stage: STAGE,
        subject: SUBJECT,
    };
    let path_error =
        super::parse_review_report(&path_draft).expect_err("absolute path should fail");
    assert_eq!(
        path_error.code(),
        super::ReviewDiagnosticCode::UnsafeMetadata
    );
}

#[test]
fn repeated_unpromoted_findings_form_promotion_candidates() {
    // r[verify agent_harness_runtime.hermes_review_reports.promotions.candidate]
    let first = record(RecordInput {
        markdown: "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=test promoted=no] Add parser fixture.\n",
        sequence: SEQUENCE_ONE,
    })
    .unwrap();
    let second = record(RecordInput {
        markdown: "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=test promoted=no] Add another parser fixture.\n",
        sequence: SEQUENCE_TWO,
    })
    .unwrap();
    let promoted = record(RecordInput {
        markdown: "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=test promoted=yes] Already covered.\n",
        sequence: SEQUENCE_THREE,
    })
    .unwrap();
    let records = alloc::vec![first, second, promoted];
    let candidates = super::review_promotion_candidates(records.as_slice(), TWO_FINDING_COUNT);

    assert_eq!(
        super::summarize_review_metrics(records.as_slice()).record_count(),
        THREE_RECORD_COUNT
    );
    assert_eq!(candidates.len(), SINGLE_FINDING_COUNT);
    assert_eq!(candidates[ZERO_COUNT].class(), "lint");
    assert_eq!(candidates[ZERO_COUNT].scope(), "adapter");
    assert_eq!(candidates[ZERO_COUNT].route(), "test");
    assert_eq!(candidates[ZERO_COUNT].count(), TWO_FINDING_COUNT);
    assert_eq!(
        candidates[ZERO_COUNT].example_hashes().len(),
        TWO_FINDING_COUNT
    );
}
