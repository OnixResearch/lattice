/// Compresses sorted or adjacent line ranges.
///
/// # Errors
///
/// Returns [`ProvenanceError`] when ranges are unsorted, overlapping in input,
/// or exceed the bounded line-count policy.
pub fn compress_line_ranges(
    ranges: &[ProvenanceLineRange],
) -> Result<Collection<ProvenanceLineRange>, ProvenanceError> {
    validate_sorted_non_overlapping_ranges(ranges)?;
    validate_range_volume(ranges)?;
    let mut compressed: Collection<ProvenanceLineRange> = Collection::new();
    for range in ranges {
        if let Some(last) = compressed.last_mut() {
            let next_expected = last
                .end
                .checked_add(MIN_LINE_NUMBER)
                .ok_or_else(|| overflow_error("line_range", last.end))?;
            if range.start <= next_expected {
                last.end = range.end;
                continue;
            }
        }
        compressed.push(*range);
    }
    Ok(compressed)
}

/// Removes attributed line ranges from another set of ranges.
///
/// # Errors
///
/// Returns [`ProvenanceError`] when inputs are malformed or expansion would
/// exceed the bounded line-count policy.
pub fn remove_line_ranges(
    ranges: &[ProvenanceLineRange],
    removed: &[ProvenanceLineRange],
) -> Result<Collection<ProvenanceLineRange>, ProvenanceError> {
    let mut lines = expand_ranges_to_set(ranges)?;
    for line in expand_ranges_to_set(removed)? {
        lines.remove(&line);
    }
    compress_lines_from_set(&lines)
}

/// Transforms attributed ranges through diff hunks.
///
/// # Errors
///
/// Returns [`ProvenanceError`] when inputs are malformed, unsorted, overflowing,
/// or exceed the bounded expansion policy.
pub fn transform_line_ranges(
    ranges: &[ProvenanceLineRange],
    hunks: &[ProvenanceDiffHunk],
) -> Result<ProvenanceRangeTransform, ProvenanceError> {
    validate_sorted_hunks(hunks)?;
    let mut transformed = BTreeSet::new();
    for original_line in expand_ranges_to_set(ranges)? {
        if let Some(new_line) = transform_original_line(original_line, hunks)? {
            transformed.insert(new_line);
        }
    }
    let accepted_line_count = u64::try_from(transformed.len()).map_err(|_| {
        ProvenanceError::new(
            ProvenanceErrorKind::ArithmeticOverflow,
            "accepted_line_count",
            UNKNOWN_SAFE_FIELD,
            "accepted line count overflowed",
        )
    })?;
    Ok(ProvenanceRangeTransform {
        preserved_ranges: compress_lines_from_set(&transformed)?,
        accepted_line_count,
    })
}
fn validate_hunk_end(field: &'static str, start: u32, len: u32) -> Result<(), ProvenanceError> {
    if len == ZERO_LENGTH {
        return Ok(());
    }
    start
        .checked_add(len - MIN_LINE_NUMBER)
        .ok_or_else(|| overflow_error(field, start))?;
    Ok(())
}

fn validate_sorted_non_overlapping_ranges(
    ranges: &[ProvenanceLineRange],
) -> Result<(), ProvenanceError> {
    let mut previous: Option<ProvenanceLineRange> = None;
    for range in ranges {
        if let Some(prior) = previous {
            if range.start <= prior.end {
                return Err(ProvenanceError::new(
                    ProvenanceErrorKind::UnsortedRange,
                    "line_range",
                    format_range_subject(range.start, range.end),
                    "ranges must be sorted and non-overlapping before compression",
                ));
            }
        }
        previous = Some(*range);
    }
    Ok(())
}

fn validate_range_volume(ranges: &[ProvenanceLineRange]) -> Result<(), ProvenanceError> {
    let mut total = 0_u64;
    for range in ranges {
        let width = u64::from(range.end - range.start) + u64::from(MIN_LINE_NUMBER);
        total = total
            .checked_add(width)
            .ok_or_else(|| overflow_error("line_range_count", range.end))?;
        if total > MAX_RANGE_CORE_LINES {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::FieldTooLong,
                "line_range_count",
                total.to_string(),
                "line range expansion exceeds bounded core policy",
            ));
        }
    }
    Ok(())
}

fn expand_ranges_to_set(
    ranges: &[ProvenanceLineRange],
) -> Result<BTreeSet<u32>, ProvenanceError> {
    let ranges = compress_line_ranges(ranges)?;
    let mut lines = BTreeSet::new();
    for range in ranges {
        for line in range.start..=range.end {
            lines.insert(line);
        }
    }
    Ok(lines)
}

fn compress_lines_from_set(
    lines: &BTreeSet<u32>,
) -> Result<Collection<ProvenanceLineRange>, ProvenanceError> {
    let mut ranges = Collection::new();
    let mut current_start: Option<u32> = None;
    let mut current_end: Option<u32> = None;
    for line in lines {
        if let (Some(start), Some(end)) = (current_start, current_end) {
            let expected_next = end
                .checked_add(MIN_LINE_NUMBER)
                .ok_or_else(|| overflow_error("line_range", end))?;
            if *line == expected_next {
                current_start = Some(start);
                current_end = Some(*line);
            } else {
                ranges.push(ProvenanceLineRange::new(start, end)?);
                current_start = Some(*line);
                current_end = Some(*line);
            }
        } else {
            current_start = Some(*line);
            current_end = Some(*line);
        }
    }
    if let (Some(start), Some(end)) = (current_start, current_end) {
        ranges.push(ProvenanceLineRange::new(start, end)?);
    }
    Ok(ranges)
}

fn validate_sorted_hunks(hunks: &[ProvenanceDiffHunk]) -> Result<(), ProvenanceError> {
    let mut previous_end: Option<u32> = None;
    for hunk in hunks {
        if let Some(prior_end) = previous_end {
            if hunk.old_start <= prior_end {
                return Err(ProvenanceError::new(
                    ProvenanceErrorKind::UnsortedRange,
                    "diff_hunk",
                    hunk.old_start.to_string(),
                    "hunks must be sorted and non-overlapping",
                ));
            }
        }
        previous_end = Some(hunk.old_end_for_order()?);
    }
    Ok(())
}

fn transform_original_line(
    original_line: u32,
    hunks: &[ProvenanceDiffHunk],
) -> Result<Option<u32>, ProvenanceError> {
    let mut cumulative_delta = 0_i64;
    for hunk in hunks {
        if hunk.removes_original_line(original_line)? {
            return Ok(None);
        }
        if hunk.shifts_original_line(original_line)? {
            cumulative_delta = cumulative_delta
                .checked_add(hunk.delta())
                .ok_or_else(|| overflow_error("hunk_delta", original_line))?;
        }
    }
    apply_delta_to_line(original_line, cumulative_delta).map(Some)
}

fn apply_delta_to_line(line: u32, delta: i64) -> Result<u32, ProvenanceError> {
    let shifted = i64::from(line)
        .checked_add(delta)
        .ok_or_else(|| overflow_error("line_shift", line))?;
    if shifted < i64::from(MIN_LINE_NUMBER) {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::ArithmeticOverflow,
            "line_shift",
            line.to_string(),
            "hunk shift moved line before the first line",
        ));
    }
    u32::try_from(shifted).map_err(|_| overflow_error("line_shift", line))
}
fn format_range_subject(start: u32, end: u32) -> Text {
    let mut subject = start.to_string();
    subject.push(RANGE_SEPARATOR);
    subject.push_str(end.to_string().as_str());
    subject
}

fn overflow_error(field: &'static str, subject: u32) -> ProvenanceError {
    ProvenanceError::new(
        ProvenanceErrorKind::ArithmeticOverflow,
        field,
        subject.to_string(),
        "line arithmetic overflowed",
    )
}
