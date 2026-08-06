use super::super::super::*;
use super::super::support::*;

#[test]
fn hashed_edit_rejects_stale_snapshot_before_plan() {
    let snapshot = hashed_snapshot();
    let spans = [span_draft(SpanFixture {
        start_tag: TAG_FIRST,
        end_tag: TAG_FIRST,
        replacement: REPLACEMENT_TEXT,
    })];
    let request = hashed::RequestDraft {
        current_snapshot_hash: HASH_B,
        ..hashed_request(&spans)
    };
    let error = hashed::validate_plan(&snapshot, &request).expect_err("stale snapshot should fail closed");

    assert_eq!(request.current_snapshot_hash, HASH_B);
    assert_eq!(error, hashed::Error::StaleSnapshot {
        expected_hash: Text::from(HASH_A),
        current_hash: Text::from(HASH_B),
    });
}

#[test]
fn hashed_edit_rejects_duplicate_and_unknown_tags() {
    let mut lines = hashed_lines();
    lines[SECOND_INDEX].tag = TAG_FIRST;
    let duplicate = hashed::Snapshot::new(&hashed::SnapshotDraft {
        resource_class: RESOURCE_CLASS_FILE,
        resource_identity: RESOURCE_ID_HASH,
        snapshot_hash: HASH_A,
        normalization: NORMALIZATION_LABEL,
        line_count: SNAPSHOT_LINE_COUNT,
        tag_generation: TAG_GENERATION,
        tagged_lines: &lines,
    })
    .expect_err("duplicate tag should fail");
    assert_eq!(duplicate, hashed::Error::DuplicateTag {
        tag: Text::from(TAG_FIRST),
    });

    let snapshot = hashed_snapshot();
    let spans = [span_draft(SpanFixture {
        start_tag: TAG_UNKNOWN,
        end_tag: TAG_UNKNOWN,
        replacement: REPLACEMENT_TEXT,
    })];
    let request = hashed_request(&spans);
    let unknown = hashed::validate_plan(&snapshot, &request).expect_err("unknown tag should fail");
    assert_eq!(unknown, hashed::Error::UnknownTag {
        tag: Text::from(TAG_UNKNOWN),
    });
}

#[test]
fn hashed_edit_rejects_invalid_overlap_and_order() {
    let snapshot = hashed_snapshot();
    let invalid_spans = [span_draft(SpanFixture {
        start_tag: TAG_THIRD,
        end_tag: TAG_SECOND,
        replacement: REPLACEMENT_TEXT,
    })];
    let invalid_request = hashed_request(&invalid_spans);
    let invalid = hashed::validate_plan(&snapshot, &invalid_request).expect_err("reverse span should fail");
    assert_eq!(invalid, hashed::Error::InvalidSpan {
        start_line: THIRD_LINE_NUMBER,
        end_line: SECOND_LINE_NUMBER,
    });

    let overlapping_spans = [
        span_draft(SpanFixture {
            start_tag: TAG_FIRST,
            end_tag: TAG_SECOND,
            replacement: REPLACEMENT_TEXT,
        }),
        span_draft(SpanFixture {
            start_tag: TAG_SECOND,
            end_tag: TAG_THIRD,
            replacement: DELETE_TEXT,
        }),
    ];
    let overlapping_request = hashed_request(&overlapping_spans);
    let overlapping = hashed::validate_plan(&snapshot, &overlapping_request).expect_err("overlap should fail");
    assert_eq!(overlapping, hashed::Error::OverlappingSpan {
        previous_end_line: SECOND_LINE_NUMBER,
        current_start_line: SECOND_LINE_NUMBER,
    });

    let out_of_order_spans = [
        span_draft(SpanFixture {
            start_tag: TAG_THIRD,
            end_tag: TAG_THIRD,
            replacement: DELETE_TEXT,
        }),
        span_draft(SpanFixture {
            start_tag: TAG_FIRST,
            end_tag: TAG_FIRST,
            replacement: REPLACEMENT_TEXT,
        }),
    ];
    let out_of_order_request = hashed_request(&out_of_order_spans);
    let out_of_order =
        hashed::validate_plan(&snapshot, &out_of_order_request).expect_err("out-of-order edits should fail");
    assert_eq!(out_of_order, hashed::Error::OutOfOrderSpan {
        previous_start_line: THIRD_LINE_NUMBER,
        current_start_line: FIRST_LINE_NUMBER,
    });
}

#[test]
fn hashed_edit_rejects_overlong_replacement_and_private_receipt_path() {
    let snapshot = hashed_snapshot();
    let replacement_body = "x".repeat(OVERSIZED_REPLACEMENT_CHARS);
    let bad_span_set = [span_draft(SpanFixture {
        start_tag: TAG_FIRST,
        end_tag: TAG_FIRST,
        replacement: replacement_body.as_str(),
    })];
    let bad_request = hashed_request(&bad_span_set);
    let bad_error = hashed::validate_plan(&snapshot, &bad_request).expect_err("oversized replacement should fail");
    assert_eq!(bad_error, hashed::Error::ReplacementTooLong {
        max_chars: super::super::super::usize_to_u64(crate::MAX_ADAPTER_FIELD_CHARS),
        actual_chars: super::super::super::usize_to_u64(OVERSIZED_REPLACEMENT_CHARS),
    });

    let valid_spans = [span_draft(SpanFixture {
        start_tag: TAG_FIRST,
        end_tag: TAG_FIRST,
        replacement: REPLACEMENT_TEXT,
    })];
    let valid_request = hashed_request(&valid_spans);
    let plan =
        hashed::validate_plan(&snapshot, &valid_request).expect("valid plan should support receipt negative case");
    let private_path = hashed::Receipt::new(&hashed::ReceiptDraft {
        plan: &plan,
        after_snapshot_hash: HASH_D,
        resource_redaction: RESOURCE_REDACTION_RELATIVE_LABEL,
        resource_identity: Some(PRIVATE_RESOURCE_ID),
        diagnostics: &[DIAGNOSTIC_APPLIED],
    })
    .expect_err("private path should fail receipt validation");
    assert_eq!(private_path, hashed::Error::UnsafeResourceIdentity {
        resource_identity: Text::from(PRIVATE_RESOURCE_ID),
    });
}
