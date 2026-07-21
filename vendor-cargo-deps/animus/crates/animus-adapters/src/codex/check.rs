#[derive(Clone, Copy)]
pub(super) struct CollectionPolicy {
    pub(super) required: bool,
    pub(super) max_items: usize,
}

#[derive(Clone, Copy)]
pub(super) struct CollectionSize {
    pub(super) field: crate::validation::FieldName,
    pub(super) actual_items: usize,
    pub(super) max_items: usize,
}

#[derive(Clone, Copy)]
pub(super) struct LineRange {
    pub(super) start_line: u32,
    pub(super) end_line: u32,
}

#[derive(Clone, Copy)]
pub(super) struct ColumnRange {
    pub(super) lines: LineRange,
    pub(super) start_column: Option<u32>,
    pub(super) end_column: Option<u32>,
}

#[derive(Clone, Copy)]
pub(super) struct EvidenceBounds<'a> {
    pub(super) line_count: u32,
    pub(super) byte_count: u64,
    pub(super) snippet: &'a str,
}

#[derive(Clone, Copy)]
pub(super) struct ScoreBounds {
    pub(super) score: u8,
    pub(super) max_score: u8,
}

pub(super) fn text_collection(
    field: crate::validation::FieldName,
    values: &[&str],
    policy: CollectionPolicy,
) -> Result<super::Collection<super::Text>, super::DiffReviewError> {
    if policy.required && values.is_empty() {
        return Err(super::DiffReviewError::EmptyCollection {
            field: field.as_str(),
        });
    }
    collection_size(CollectionSize {
        field,
        actual_items: values.len(),
        max_items: policy.max_items,
    })?;
    let mut normalized = super::Collection::with_capacity(values.len());
    for value in values {
        let text = required_safe_text(field, value)?;
        reject_duplicate(field, &mut normalized, text.as_str())?;
        normalized.push(text);
    }
    Ok(normalized)
}

pub(super) fn collection_size(limit: CollectionSize) -> Result<(), super::DiffReviewError> {
    if limit.actual_items > limit.max_items {
        return Err(super::DiffReviewError::CollectionTooLarge {
            field: limit.field.as_str(),
            max_items: limit.max_items,
            actual_items: limit.actual_items,
        });
    }
    Ok(())
}

pub(super) fn reject_duplicate(
    field: crate::validation::FieldName,
    values: &mut super::Collection<super::Text>,
    value: &str,
) -> Result<(), super::DiffReviewError> {
    if contains_text(values.as_slice(), value) {
        return Err(super::DiffReviewError::DuplicateValue {
            field: field.as_str(),
            value: super::Text::from(value),
        });
    }
    Ok(())
}

pub(super) fn required_text(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<super::Text, super::DiffReviewError> {
    crate::validation::normalized_adapter_field(field, value).map_err(super::DiffReviewError::from)
}

pub(super) fn required_safe_text(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<super::Text, super::DiffReviewError> {
    let text = required_text(field, value)?;
    reject_raw_material(field, text.as_str())?;
    Ok(text)
}

pub(super) fn optional_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<super::Text>, super::DiffReviewError> {
    value.map(|text| required_text(field, text)).transpose()
}

pub(super) fn span_lines(lines: LineRange) -> Result<(), super::DiffReviewError> {
    if lines.start_line < super::MIN_REVIEW_LINE || lines.end_line < super::MIN_REVIEW_LINE {
        return Err(super::DiffReviewError::InvalidSpan {
            reason: "line numbers are one-based",
        });
    }
    if lines.start_line > lines.end_line {
        return Err(super::DiffReviewError::InvalidSpan {
            reason: "start line must not exceed end line",
        });
    }
    Ok(())
}

pub(super) fn span_columns(columns: ColumnRange) -> Result<(), super::DiffReviewError> {
    assert!(columns.lines.start_line >= super::MIN_REVIEW_LINE);
    assert!(columns.lines.end_line >= super::MIN_REVIEW_LINE);
    if columns
        .start_column
        .is_some_and(|column| column < super::MIN_REVIEW_COLUMN)
        || columns
            .end_column
            .is_some_and(|column| column < super::MIN_REVIEW_COLUMN)
    {
        return Err(super::DiffReviewError::InvalidSpan {
            reason: "columns are one-based",
        });
    }
    if columns.lines.start_line == columns.lines.end_line
        && matches!((columns.start_column, columns.end_column), (Some(start), Some(end)) if start > end)
    {
        return Err(super::DiffReviewError::InvalidSpan {
            reason: "start column must not exceed end column on one line",
        });
    }
    Ok(())
}

pub(super) fn evidence_kind(
    kind: super::issue::DiffReviewEvidenceKind,
) -> Result<(), super::DiffReviewError> {
    match kind {
        super::issue::DiffReviewEvidenceKind::FileSpan
        | super::issue::DiffReviewEvidenceKind::PatchHunk
        | super::issue::DiffReviewEvidenceKind::ExternalInput => Ok(()),
        super::issue::DiffReviewEvidenceKind::FullFile
        | super::issue::DiffReviewEvidenceKind::UnboundedLog
        | super::issue::DiffReviewEvidenceKind::UnrelatedContext => {
            Err(super::DiffReviewError::UnsupportedEvidenceKind { kind })
        }
    }
}

pub(super) fn evidence_bounds(bounds: EvidenceBounds<'_>) -> Result<(), super::DiffReviewError> {
    if bounds.line_count < super::MIN_REVIEW_LINE {
        return unbounded_evidence(u64::from(bounds.line_count));
    }
    if bounds.line_count > super::MAX_EVIDENCE_LINES {
        return unbounded_evidence(u64::from(bounds.line_count));
    }
    if bounds.byte_count > super::MAX_EVIDENCE_BYTES {
        return Err(super::DiffReviewError::UnboundedEvidence {
            field: super::EVIDENCE_SNIPPET_FIELD.as_str(),
            max_value: super::MAX_EVIDENCE_BYTES,
            actual_value: bounds.byte_count,
        });
    }
    assert!(
        bounds.line_count >= super::MIN_REVIEW_LINE,
        "minimum line count bound is enforced"
    );
    assert!(
        bounds.line_count <= super::MAX_EVIDENCE_LINES,
        "maximum line count bound is enforced"
    );
    assert!(
        bounds.byte_count <= super::MAX_EVIDENCE_BYTES,
        "byte count bounds are enforced"
    );
    let snippet_bytes = match u64::try_from(bounds.snippet.len()) {
        Ok(value) => value,
        Err(_overflow) => return snippet_length_overflow(),
    };
    if snippet_bytes > super::MAX_EVIDENCE_BYTES {
        return Err(super::DiffReviewError::UnboundedEvidence {
            field: super::EVIDENCE_SNIPPET_FIELD.as_str(),
            max_value: super::MAX_EVIDENCE_BYTES,
            actual_value: snippet_bytes,
        });
    }
    Ok(())
}

pub(super) fn confidence(bounds: ScoreBounds) -> Result<(), super::DiffReviewError> {
    if bounds.score > bounds.max_score {
        return Err(super::DiffReviewError::InvalidConfidence {
            max_score: bounds.max_score,
            actual_score: bounds.score,
        });
    }
    Ok(())
}

pub(super) fn contains_text(values: &[super::Text], needle: &str) -> bool {
    values.iter().any(|value| value == needle)
}

fn reject_raw_material(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<(), super::DiffReviewError> {
    if let Some(marker) = super::RAW_MATERIAL_MARKERS
        .iter()
        .copied()
        .find(|marker| value.contains(marker))
    {
        return Err(super::DiffReviewError::UnsafeEvidence {
            field: field.as_str(),
            marker,
        });
    }
    Ok(())
}

fn unbounded_evidence(actual_value: u64) -> Result<(), super::DiffReviewError> {
    Err(super::DiffReviewError::UnboundedEvidence {
        field: super::EVIDENCE_SNIPPET_FIELD.as_str(),
        max_value: u64::from(super::MAX_EVIDENCE_LINES),
        actual_value,
    })
}

fn snippet_length_overflow() -> Result<(), super::DiffReviewError> {
    Err(super::DiffReviewError::UnboundedEvidence {
        field: super::EVIDENCE_SNIPPET_FIELD.as_str(),
        max_value: super::MAX_EVIDENCE_BYTES,
        actual_value: super::MAX_EVIDENCE_BYTES.saturating_add(1),
    })
}
