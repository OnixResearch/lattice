#[test]
fn range_core_compresses_removes_and_shifts_attribution() {
    // r[verify agent_harness_runtime.provenance_git_ai.range_core]
    // r[verify agent_harness_runtime.provenance_git_ai.range_core.positive]
    let adjacent = [
        line_range(FIRST_RANGE_START, FIRST_RANGE_END),
        line_range(FIRST_RANGE_END + MIN_LINE_NUMBER, SECOND_RANGE_END),
    ];
    let compressed = compress_line_ranges(&adjacent).expect("adjacent ranges should compress");
    assert_eq!(compressed, [line_range(COMPRESSED_RANGE_START, COMPRESSED_RANGE_END)]);

    let removed = remove_line_ranges(
        &[line_range(FIRST_RANGE_START, SECOND_RANGE_END)],
        &[line_range(FIRST_RANGE_END, FIRST_RANGE_END)],
    )
    .expect("range removal should succeed");
    assert_eq!(
        removed,
        [
            line_range(FIRST_RANGE_START, FIRST_RANGE_END - MIN_LINE_NUMBER),
            line_range(SECOND_RANGE_START - MIN_LINE_NUMBER, SECOND_RANGE_END),
        ]
    );

    let transform = transform_line_ranges(
        &[
            line_range(FIRST_RANGE_START, FIRST_RANGE_END),
            line_range(SECOND_RANGE_START, SECOND_RANGE_END),
        ],
        &[
            ProvenanceDiffHunk::new(
                DELETE_HUNK_OLD_START,
                DELETE_HUNK_OLD_LEN,
                DELETE_HUNK_NEW_START,
                ZERO_LENGTH,
            )
            .expect("delete hunk should validate"),
            ProvenanceDiffHunk::new(
                INSERT_HUNK_OLD_START,
                ZERO_LENGTH,
                INSERT_HUNK_NEW_START,
                INSERT_HUNK_NEW_LEN,
            )
            .expect("insert hunk should validate"),
        ],
    )
    .expect("range transform should succeed");

    assert_eq!(transform.accepted_line_count, EXPECTED_ACCEPTED_LINES);
    assert_eq!(
        transform.preserved_ranges,
        [
            line_range(EXPECTED_SHIFTED_RANGE_START, EXPECTED_SHIFTED_RANGE_END),
            line_range(EXPECTED_SHIFTED_SECOND_START, EXPECTED_SHIFTED_SECOND_END),
        ]
    );
}

#[test]
fn malformed_ranges_fail_without_panic() {
    // r[verify agent_harness_runtime.provenance_git_ai.range_core.negative]
    let zero = ProvenanceLineRange::new(ZERO_LENGTH, FIRST_RANGE_END)
        .expect_err("zero line should fail");
    let reversed = ProvenanceLineRange::new(FIRST_RANGE_END, FIRST_RANGE_START)
        .expect_err("reversed range should fail");
    let unsorted = compress_line_ranges(&[
        line_range(SECOND_RANGE_START, SECOND_RANGE_END),
        line_range(FIRST_RANGE_START, FIRST_RANGE_END),
    ])
    .expect_err("unsorted ranges should fail");
    let overlapping_hunks = transform_line_ranges(
        &[line_range(FIRST_RANGE_START, SECOND_RANGE_END)],
        &[
            ProvenanceDiffHunk::new(FIRST_RANGE_START, FIRST_RANGE_END, FIRST_RANGE_START, FIRST_RANGE_END)
                .expect("first hunk should validate"),
            ProvenanceDiffHunk::new(FIRST_RANGE_END, FIRST_RANGE_END, FIRST_RANGE_END, FIRST_RANGE_END)
                .expect("second hunk should validate"),
        ],
    )
    .expect_err("overlapping hunks should fail");

    assert_eq!(zero.kind(), ProvenanceErrorKind::InvalidRange);
    assert_eq!(reversed.kind(), ProvenanceErrorKind::InvalidRange);
    assert_eq!(unsorted.kind(), ProvenanceErrorKind::UnsortedRange);
    assert_eq!(overlapping_hunks.kind(), ProvenanceErrorKind::UnsortedRange);
}
