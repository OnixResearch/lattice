use alloc::string::ToString;

#[derive(Clone, Copy)]
pub(super) struct SnapshotIdentity<'a> {
    pub(super) resource_class: &'a str,
    pub(super) resource_identity: &'a str,
    pub(super) snapshot_hash: &'a str,
    pub(super) normalization: super::TextNormalization,
    pub(super) line_count: u32,
    pub(super) tag_generation: &'a str,
    pub(super) tagged_lines: &'a [super::TaggedLine],
}

#[derive(Clone, Copy)]
pub(super) struct ReplacementIdentity<'a> {
    pub(super) snapshot_hash: &'a str,
    pub(super) start_tag: &'a str,
    pub(super) end_tag: &'a str,
    pub(super) replacement: &'a str,
}

#[derive(Clone, Copy)]
pub(super) struct ReceiptIdentity<'a> {
    pub(super) plan_hash: &'a str,
    pub(super) resource_redaction: super::ResourceRedaction,
    pub(super) resource_identity: Option<&'a str>,
    pub(super) after_snapshot_hash: &'a str,
    pub(super) diagnostics: &'a [super::Text],
    pub(super) spans: &'a [super::SpanMetadata],
}

pub(super) fn snapshot_identity(input: SnapshotIdentity<'_>) -> super::Text {
    let line_count = input.line_count.to_string();
    let tagged_hash = tagged_lines(input.tagged_lines);
    super::super::hash_parts(&[
        super::SNAPSHOT_SCHEMA,
        input.resource_class,
        input.resource_identity,
        input.snapshot_hash,
        input.normalization.as_str(),
        line_count.as_str(),
        input.tag_generation,
        tagged_hash.as_str(),
    ])
}

pub(super) fn tagged_lines(lines: &[super::TaggedLine]) -> super::Text {
    let mut normalized = super::Text::from(super::SNAPSHOT_SCHEMA);
    for line in lines {
        normalized.push('|');
        normalized.push_str(line.tag());
        normalized.push('|');
        normalized.push_str(line.line_number().to_string().as_str());
        normalized.push('|');
        normalized.push_str(line.line_hash());
    }
    alloc::format!("{}", blake3::hash(normalized.as_bytes()).to_hex())
}

pub(super) fn replacement(input: ReplacementIdentity<'_>) -> super::Text {
    super::super::hash_parts(&[
        super::PLAN_SCHEMA,
        input.snapshot_hash,
        input.start_tag,
        input.end_tag,
        input.replacement,
    ])
}

pub(super) fn plan(
    snapshot_hash: &str,
    newline_policy: super::NewlinePolicy,
    spans: &[super::Validated],
) -> super::Text {
    let spans_hash = validated_spans(spans);
    super::super::hash_parts(&[
        super::PLAN_SCHEMA,
        snapshot_hash,
        newline_policy.as_str(),
        spans_hash.as_str(),
    ])
}

pub(super) fn validated_spans(spans: &[super::Validated]) -> super::Text {
    let mut normalized = super::Text::from(super::PLAN_SCHEMA);
    for span in spans {
        normalized.push('|');
        normalized.push_str(span.start_tag());
        normalized.push('|');
        normalized.push_str(span.end_tag());
        normalized.push('|');
        normalized.push_str(span.replacement_hash());
    }
    alloc::format!("{}", blake3::hash(normalized.as_bytes()).to_hex())
}

pub(super) fn receipt(input: ReceiptIdentity<'_>) -> super::Text {
    let diagnostics_hash = diagnostics(input.diagnostics);
    let spans_hash = receipt_spans(input.spans);
    super::super::hash_parts(&[
        super::RECEIPT_SCHEMA,
        input.plan_hash,
        input.resource_redaction.as_str(),
        input
            .resource_identity
            .unwrap_or(super::OMITTED_RESOURCE_IDENTITY),
        input.after_snapshot_hash,
        diagnostics_hash.as_str(),
        spans_hash.as_str(),
    ])
}

pub(super) fn diagnostics(values: &[super::Text]) -> super::Text {
    let mut normalized = super::Text::from(super::RECEIPT_SCHEMA);
    for value in values {
        normalized.push('|');
        normalized.push_str(value.as_str());
    }
    alloc::format!("{}", blake3::hash(normalized.as_bytes()).to_hex())
}

pub(super) fn receipt_spans(spans: &[super::SpanMetadata]) -> super::Text {
    let mut normalized = super::Text::from(super::RECEIPT_SCHEMA);
    for span in spans {
        normalized.push('|');
        normalized.push_str(span.start_line().to_string().as_str());
        normalized.push('|');
        normalized.push_str(span.end_line().to_string().as_str());
        normalized.push('|');
        normalized.push_str(span.replacement_chars().to_string().as_str());
        normalized.push('|');
        normalized.push_str(span.replacement_hash());
    }
    alloc::format!("{}", blake3::hash(normalized.as_bytes()).to_hex())
}
