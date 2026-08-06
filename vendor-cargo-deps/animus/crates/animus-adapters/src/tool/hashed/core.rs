#[derive(Clone, Copy)]
struct HashMatch<'a> {
    snapshot_hash: &'a str,
    expected_hash: &'a str,
}

#[derive(Clone, Copy)]
struct CurrentHash<'a> {
    expected_hash: &'a str,
    current_hash: &'a str,
}

#[derive(Clone, Copy)]
struct SpanLines {
    start_line: u32,
    end_line: u32,
}

#[derive(Clone, Copy)]
struct SpanSelector<'a> {
    start_tag: &'a str,
    end_tag: &'a str,
}

/// Borrowed span edit request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpanDraft<'a> {
    /// First tagged line selected by the edit span.
    pub start_tag: &'a str,
    /// Last tagged line selected by the edit span.
    pub end_tag: &'a str,
    /// Replacement text. Empty text is a deletion.
    pub replacement: &'a str,
}

/// Borrowed edit-plan validation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestDraft<'a> {
    /// Snapshot hash that the caller expects to edit.
    pub expected_snapshot_hash: &'a str,
    /// Current snapshot hash reported by the host immediately before planning.
    pub current_snapshot_hash: &'a str,
    /// Newline policy for replacement text.
    pub newline_policy: &'a str,
    /// Ordered span edits to validate.
    pub spans: &'a [SpanDraft<'a>],
}

/// Validates a hashed edit plan without reading or mutating the host filesystem.
///
/// # Errors
///
/// Returns [`super::Error`] when the request is stale, mismatched with the
/// supplied snapshot, references unknown or ambiguous tags, contains invalid or
/// overlapping spans, uses unsupported newline policy, or exceeds replacement
/// bounds.
///
/// # Panics
///
/// Panics if the shared hash validator stops returning BLAKE3-sized hashes.
pub fn validate_plan(snapshot: &super::Snapshot, request: &RequestDraft<'_>) -> Result<super::Plan, super::Error> {
    let expected_hash = super::validate_hash(super::SNAPSHOT_HASH_FIELD, request.expected_snapshot_hash)?;
    let current_hash = super::validate_hash(super::CURRENT_SNAPSHOT_HASH_FIELD, request.current_snapshot_hash)?;
    assert_eq!(expected_hash.len(), crate::BLAKE3_HASH_HEX_CHARS);
    assert_eq!(current_hash.len(), crate::BLAKE3_HASH_HEX_CHARS);
    validate_snapshot_hash_match(HashMatch {
        snapshot_hash: snapshot.snapshot_hash(),
        expected_hash: expected_hash.as_str(),
    })?;
    validate_current_hash(CurrentHash {
        expected_hash: expected_hash.as_str(),
        current_hash: current_hash.as_str(),
    })?;
    let newline_policy = super::NewlinePolicy::parse(request.newline_policy)?;
    let spans = collect_validated_spans(snapshot, request.spans)?;
    let plan_hash = super::hash::plan(snapshot.snapshot_hash(), newline_policy, spans.as_slice());
    Ok(super::Plan {
        resource_class: super::Text::from(snapshot.resource_class()),
        snapshot_hash: expected_hash,
        newline_policy,
        spans,
        hash: plan_hash,
    })
}

fn validate_snapshot_hash_match(input: HashMatch<'_>) -> Result<(), super::Error> {
    if input.snapshot_hash != input.expected_hash {
        return Err(super::Error::SnapshotMismatch {
            request_hash: super::Text::from(input.expected_hash),
            snapshot_hash: super::Text::from(input.snapshot_hash),
        });
    }
    Ok(())
}

fn validate_current_hash(input: CurrentHash<'_>) -> Result<(), super::Error> {
    if input.expected_hash != input.current_hash {
        return Err(super::Error::StaleSnapshot {
            expected_hash: super::Text::from(input.expected_hash),
            current_hash: super::Text::from(input.current_hash),
        });
    }
    Ok(())
}

fn collect_validated_spans(
    snapshot: &super::Snapshot,
    drafts: &[SpanDraft<'_>],
) -> Result<super::Collection<super::Validated>, super::Error> {
    super::validate_non_empty_collection(super::EDITS_FIELD, drafts.len())?;
    super::validate_collection_count(super::CountLimit {
        field: super::EDITS_FIELD,
        actual_items: drafts.len(),
        max_items: super::MAX_OPERATIONS,
    })?;
    assert!(!drafts.is_empty());
    assert!(drafts.len() <= super::MAX_OPERATIONS);
    let mut spans = super::Collection::with_capacity(drafts.len());
    let mut previous_start_line = None;
    let mut previous_end_line = None;
    for draft in drafts {
        let span = validate_span(snapshot, draft)?;
        validate_unique_span(spans.as_slice(), SpanSelector {
            start_tag: span.start_tag(),
            end_tag: span.end_tag(),
        })?;
        validate_span_order(previous_start_line, span.start_line())?;
        validate_span_overlap(previous_end_line, span.start_line())?;
        previous_start_line = Some(span.start_line());
        previous_end_line = Some(span.end_line());
        spans.push(span);
    }
    Ok(spans)
}

fn validate_span(snapshot: &super::Snapshot, draft: &SpanDraft<'_>) -> Result<super::Validated, super::Error> {
    debug_assert!(line_count_covers_tags(snapshot), "line count covers tags");
    let start_tag = super::required_label(super::LINE_TAG_FIELD, draft.start_tag)?;
    let end_tag = super::required_label(super::LINE_TAG_FIELD, draft.end_tag)?;
    let start_line = snapshot.line_for_tag(start_tag.as_str())?.line_number();
    let end_line = snapshot.line_for_tag(end_tag.as_str())?.line_number();
    validate_span_lines(SpanLines { start_line, end_line })?;
    let replacement_chars = validate_replacement(draft.replacement)?;
    let replacement_hash = super::hash::replacement(super::hash::ReplacementIdentity {
        snapshot_hash: snapshot.snapshot_hash(),
        start_tag: start_tag.as_str(),
        end_tag: end_tag.as_str(),
        replacement: draft.replacement,
    });
    Ok(super::Validated {
        start_tag,
        end_tag,
        start_line,
        end_line,
        replacement: super::Text::from(draft.replacement),
        replacement_chars,
        replacement_hash,
    })
}

fn line_count_covers_tags(snapshot: &super::Snapshot) -> bool {
    let Ok(tagged_line_count) = u32::try_from(snapshot.tagged_lines().len()) else {
        return false;
    };
    snapshot.line_count() >= tagged_line_count
}

fn validate_span_lines(span: SpanLines) -> Result<(), super::Error> {
    if span.start_line > span.end_line {
        return Err(super::Error::InvalidSpan {
            start_line: span.start_line,
            end_line: span.end_line,
        });
    }
    Ok(())
}

fn validate_unique_span(spans: &[super::Validated], selector: SpanSelector<'_>) -> Result<(), super::Error> {
    if spans
        .iter()
        .any(|span| span.start_tag() == selector.start_tag && span.end_tag() == selector.end_tag)
    {
        return Err(super::Error::DuplicateSpan {
            start_tag: super::Text::from(selector.start_tag),
            end_tag: super::Text::from(selector.end_tag),
        });
    }
    Ok(())
}

fn validate_span_order(previous_start_line: Option<u32>, current_start_line: u32) -> Result<(), super::Error> {
    if let Some(previous) = previous_start_line {
        if current_start_line < previous {
            return Err(super::Error::OutOfOrderSpan {
                previous_start_line: previous,
                current_start_line,
            });
        }
    }
    Ok(())
}

fn validate_span_overlap(previous_end_line: Option<u32>, current_start_line: u32) -> Result<(), super::Error> {
    if let Some(previous) = previous_end_line {
        if current_start_line <= previous {
            return Err(super::Error::OverlappingSpan {
                previous_end_line: previous,
                current_start_line,
            });
        }
    }
    Ok(())
}

fn validate_replacement(replacement: &str) -> Result<u64, super::Error> {
    let actual_chars = replacement.chars().count();
    if actual_chars > super::MAX_REPLACEMENT_CHARS {
        return Err(super::Error::ReplacementTooLong {
            max_chars: super::usize_to_u64(super::MAX_REPLACEMENT_CHARS),
            actual_chars: super::usize_to_u64(actual_chars),
        });
    }
    crate::validation::validate_optional_adapter_field(super::REPLACEMENT_FIELD, replacement)?;
    Ok(super::usize_to_u64(actual_chars))
}
