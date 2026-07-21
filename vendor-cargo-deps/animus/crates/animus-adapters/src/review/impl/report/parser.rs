fn flush_section(
    sections: &mut Vec<ReviewSection>,
    current: &mut Option<SectionBuilder>,
) -> Result<(), ReviewReportError> {
    if let Some(builder) = current.take() {
        if sections.len() >= MAX_SECTIONS {
            return Err(ReviewReportError::new(
                ReviewDiagnosticCode::TooManyItems,
                "sections",
            ));
        }
        sections.push(builder.into_section());
    }
    Ok(())
}

fn parse_findings(markdown: &str) -> (Vec<ReviewFinding>, Vec<ReviewDiagnostic>) {
    let mut findings = Vec::new();
    let mut diagnostics = Vec::new();
    let mut in_findings = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix(SECTION_PREFIX) {
            in_findings = title.trim() == FINDINGS_SECTION_TITLE;
            continue;
        }
        if in_findings && trimmed.starts_with(BULLET_PREFIX) {
            parse_finding_bullet(trimmed, &mut findings, &mut diagnostics);
        }
    }
    (findings, diagnostics)
}

fn parse_finding_bullet(
    bullet: &str,
    findings: &mut Vec<ReviewFinding>,
    diagnostics: &mut Vec<ReviewDiagnostic>,
) {
    if findings.len() >= MAX_FINDINGS {
        diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::TooManyItems,
            "findings",
        ));
        return;
    }
    let content = bullet.trim_start_matches(BULLET_PREFIX).trim();
    let Some(after_open) = content.strip_prefix('[') else {
        diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::MalformedFindingBullet,
            "finding",
        ));
        return;
    };
    let Some(close_index) = after_open.find(']') else {
        diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::MalformedFindingBullet,
            "finding",
        ));
        return;
    };
    let tag_block = &after_open[..close_index];
    let summary = after_open[close_index + SINGLE_ITEM_COUNT..].trim();
    let tags = parse_finding_tags(tag_block, diagnostics);
    if let Some(finding) = build_finding(tags, summary, diagnostics) {
        findings.push(finding);
    }
}

fn parse_finding_tags(tag_block: &str, diagnostics: &mut Vec<ReviewDiagnostic>) -> FindingTags {
    let mut tags = FindingTags::default();
    for raw_tag in tag_block.split_whitespace() {
        let mut parts = raw_tag.splitn(TAG_SPLIT_LIMIT, TAG_SEPARATOR);
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();
        if value.is_empty() {
            diagnostics.push(ReviewDiagnostic::new(
                ReviewDiagnosticCode::MalformedFindingBullet,
                UNKNOWN_TAG_FIELD,
            ));
            continue;
        }
        assign_tag(&mut tags, key, value, diagnostics);
    }
    tags
}

fn assign_tag(
    tags: &mut FindingTags,
    key: &str,
    value: &str,
    diagnostics: &mut Vec<ReviewDiagnostic>,
) {
    match key {
        TAG_CLASS => assign_text_tag(&mut tags.class, TAG_CLASS, value, diagnostics),
        TAG_SCOPE => assign_text_tag(&mut tags.scope, TAG_SCOPE, value, diagnostics),
        TAG_ROUTE => assign_text_tag(&mut tags.route, TAG_ROUTE, value, diagnostics),
        TAG_PROMOTED => assign_promoted_tag(tags, value, diagnostics),
        _unknown => diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::UnknownFindingTag,
            UNKNOWN_TAG_FIELD,
        )),
    }
}

fn assign_text_tag(
    slot: &mut Option<String>,
    field: &str,
    value: &str,
    diagnostics: &mut Vec<ReviewDiagnostic>,
) {
    if slot.is_some() {
        diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::DuplicateFindingTag,
            field,
        ));
    } else {
        *slot = Some(value.to_owned());
    }
}

fn assign_promoted_tag(
    tags: &mut FindingTags,
    value: &str,
    diagnostics: &mut Vec<ReviewDiagnostic>,
) {
    if tags.promoted.is_some() {
        diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::DuplicateFindingTag,
            TAG_PROMOTED,
        ));
        return;
    }
    match value {
        PROMOTED_YES => tags.promoted = Some(true),
        PROMOTED_NO => tags.promoted = Some(false),
        _unknown => diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::UnknownPromotedValue,
            TAG_PROMOTED,
        )),
    }
}

fn build_finding(
    tags: FindingTags,
    summary: &str,
    diagnostics: &mut Vec<ReviewDiagnostic>,
) -> Option<ReviewFinding> {
    let class = required_tag(tags.class, TAG_CLASS, diagnostics)?;
    let scope = required_tag(tags.scope, TAG_SCOPE, diagnostics)?;
    let route = required_tag(tags.route, TAG_ROUTE, diagnostics)?;
    let promoted = required_promoted(tags.promoted, diagnostics)?;
    let summary = normalize_summary(summary, diagnostics)?;
    let summary_hash = hash_text(summary.as_str());
    Some(ReviewFinding {
        class,
        scope,
        route,
        promoted,
        summary,
        summary_hash,
    })
}

fn required_tag(
    value: Option<String>,
    field: &str,
    diagnostics: &mut Vec<ReviewDiagnostic>,
) -> Option<String> {
    if let Some(text) = value {
        Some(text)
    } else {
        diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::MissingFindingTag,
            field,
        ));
        None
    }
}

fn required_promoted(value: Option<bool>, diagnostics: &mut Vec<ReviewDiagnostic>) -> Option<bool> {
    if let Some(promoted) = value {
        Some(promoted)
    } else {
        diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::MissingFindingTag,
            TAG_PROMOTED,
        ));
        None
    }
}

fn normalize_summary(summary: &str, diagnostics: &mut Vec<ReviewDiagnostic>) -> Option<String> {
    if summary.is_empty() || summary.chars().count() > MAX_FINDING_SUMMARY_CHARS {
        diagnostics.push(ReviewDiagnostic::new(
            ReviewDiagnosticCode::MalformedFindingBullet,
            "summary",
        ));
        None
    } else {
        Some(summary.to_owned())
    }
}

fn safe_metric_findings(
    findings: &[ReviewFinding],
) -> Result<Vec<ReviewMetricFinding>, ReviewReportError> {
    let mut metrics = Vec::with_capacity(findings.len());
    for finding in findings {
        validate_safe_metadata(TAG_CLASS, finding.class())?;
        validate_safe_metadata(TAG_SCOPE, finding.scope())?;
        validate_safe_metadata(TAG_ROUTE, finding.route())?;
        validate_safe_metadata("summary", finding.summary())?;
        metrics.push(ReviewMetricFinding {
            class: finding.class.clone(),
            scope: finding.scope.clone(),
            route: finding.route.clone(),
            promoted: finding.promoted,
            summary_hash: finding.summary_hash.clone(),
        });
    }
    Ok(metrics)
}

fn validate_safe_metadata(field: &str, value: &str) -> Result<String, ReviewReportError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > MAX_SAFE_FIELD_CHARS
        || contains_unsafe_metadata(trimmed)
    {
        Err(ReviewReportError::new(
            ReviewDiagnosticCode::UnsafeMetadata,
            field,
        ))
    } else {
        Ok(trimmed.to_owned())
    }
}

fn contains_unsafe_metadata(value: &str) -> bool {
    value.starts_with('/')
        || UNSAFE_METADATA_MARKERS
            .iter()
            .any(|marker| value.contains(marker))
}

fn record_hash(report: &ReviewReport, sequence: &str, findings: &[ReviewMetricFinding]) -> String {
    let mut canonical = String::new();
    canonical.push_str(sequence);
    canonical.push_str(HASH_SEPARATOR);
    canonical.push_str(report.report_hash());
    canonical.push_str(HASH_SEPARATOR);
    canonical.push_str(report.verdict().as_str());
    for finding in findings {
        canonical.push_str(HASH_SEPARATOR);
        canonical.push_str(finding.class());
        canonical.push_str(HASH_SEPARATOR);
        canonical.push_str(finding.scope());
        canonical.push_str(HASH_SEPARATOR);
        canonical.push_str(finding.route());
        canonical.push_str(HASH_SEPARATOR);
        canonical.push_str(finding.summary_hash());
    }
    hash_text(canonical.as_str())
}

fn add_candidate_finding(
    groups: &mut Vec<ReviewPromotionCandidate>,
    finding: &ReviewMetricFinding,
) {
    if let Some(existing) = groups.iter_mut().find(|candidate| {
        candidate.class() == finding.class()
            && candidate.scope() == finding.scope()
            && candidate.route() == finding.route()
    }) {
        existing.count += SINGLE_ITEM_COUNT;
        existing.example_hashes.push(finding.summary_hash.clone());
    } else {
        groups.push(ReviewPromotionCandidate {
            class: finding.class.clone(),
            scope: finding.scope.clone(),
            route: finding.route.clone(),
            count: SINGLE_ITEM_COUNT,
            example_hashes: vec![finding.summary_hash.clone()],
        });
    }
}

fn hash_text(text: &str) -> String {
    format!("{}", blake3::hash(text.as_bytes()).to_hex())
}
