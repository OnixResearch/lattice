/// Parses an in-memory review report.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when the verdict, report size, or safe host
/// metadata is invalid.
pub fn parse_review_report(
    draft: &ReviewReportDraft<'_>,
) -> Result<ReviewReport, ReviewReportError> {
    validate_report_size(draft.markdown)?;
    let source = validate_safe_metadata("source", draft.source)?;
    let stage = validate_safe_metadata("stage", draft.stage)?;
    let subject = validate_safe_metadata("subject", draft.subject)?;
    let verdict = parse_verdict(draft.markdown)?;
    let sections = parse_sections(draft.markdown)?;
    let (findings, diagnostics) = parse_findings(draft.markdown);
    Ok(ReviewReport {
        verdict,
        source,
        stage,
        subject,
        report_hash: hash_text(draft.markdown),
        sections,
        findings,
        diagnostics,
    })
}

/// Builds exactly one safe append-only metric record from a parsed report.
///
/// # Errors
///
/// Returns [`ReviewReportError`] when the parsed report contains diagnostics or
/// when the host sequence or finding metadata is unsafe.
pub fn append_review_metric_record(
    report: &ReviewReport,
    draft: &ReviewMetricRecordDraft<'_>,
) -> Result<ReviewMetricRecord, ReviewReportError> {
    if !report.diagnostics().is_empty() {
        return Err(ReviewReportError::new(
            ReviewDiagnosticCode::ReportHasDiagnostics,
            "diagnostics",
        ));
    }
    let sequence = validate_safe_metadata("sequence", draft.sequence)?;
    let findings = safe_metric_findings(report.findings())?;
    let record_hash = record_hash(report, sequence.as_str(), findings.as_slice());
    Ok(ReviewMetricRecord {
        record_hash,
        sequence,
        source: report.source.clone(),
        stage: report.stage.clone(),
        subject: report.subject.clone(),
        report_hash: report.report_hash.clone(),
        verdict: report.verdict,
        findings,
    })
}

/// Produces a read-only metrics summary over supplied records.
#[must_use]
pub fn summarize_review_metrics(records: &[ReviewMetricRecord]) -> ReviewMetricsSummary {
    let mut summary = ReviewMetricsSummary {
        records: records.len(),
        findings: ZERO_RECORD_COUNT,
        pass_verdicts: ZERO_RECORD_COUNT,
        warn_verdicts: ZERO_RECORD_COUNT,
        fail_verdicts: ZERO_RECORD_COUNT,
    };
    for record in records {
        summary.findings += record.findings().len();
        match record.verdict() {
            ReviewVerdict::Pass => summary.pass_verdicts += SINGLE_ITEM_COUNT,
            ReviewVerdict::Warn => summary.warn_verdicts += SINGLE_ITEM_COUNT,
            ReviewVerdict::Fail => summary.fail_verdicts += SINGLE_ITEM_COUNT,
        }
    }
    summary
}

/// Groups repeated unpromoted findings without mutating records or lifecycle state.
#[must_use]
pub fn review_promotion_candidates(
    records: &[ReviewMetricRecord],
    minimum_count: usize,
) -> Vec<ReviewPromotionCandidate> {
    let threshold = minimum_count.max(MIN_PROMOTION_COUNT);
    let mut groups: Vec<ReviewPromotionCandidate> = Vec::new();
    for record in records {
        for finding in record.findings() {
            if finding.promoted() {
                continue;
            }
            add_candidate_finding(&mut groups, finding);
        }
    }
    groups
        .into_iter()
        .filter(|candidate| candidate.count() >= threshold)
        .collect()
}

fn validate_report_size(markdown: &str) -> Result<(), ReviewReportError> {
    if markdown.len() > MAX_REPORT_BYTES {
        Err(ReviewReportError::new(
            ReviewDiagnosticCode::ReportTooLarge,
            "markdown",
        ))
    } else {
        Ok(())
    }
}

fn parse_verdict(markdown: &str) -> Result<ReviewVerdict, ReviewReportError> {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(VERDICT_PREFIX) {
            return verdict_from_label(value.trim());
        }
    }
    Err(ReviewReportError::new(
        ReviewDiagnosticCode::MissingVerdict,
        "verdict",
    ))
}

fn verdict_from_label(label: &str) -> Result<ReviewVerdict, ReviewReportError> {
    match label {
        VERDICT_PASS => Ok(ReviewVerdict::Pass),
        VERDICT_WARN => Ok(ReviewVerdict::Warn),
        VERDICT_FAIL => Ok(ReviewVerdict::Fail),
        _other => Err(ReviewReportError::new(
            ReviewDiagnosticCode::MalformedVerdict,
            "verdict",
        )),
    }
}

fn parse_sections(markdown: &str) -> Result<Vec<ReviewSection>, ReviewReportError> {
    let mut builders = Vec::new();
    let mut current = None;
    for line in markdown.lines() {
        if let Some(title) = line.trim().strip_prefix(SECTION_PREFIX) {
            flush_section(&mut builders, &mut current)?;
            current = Some(SectionBuilder {
                title: title.trim().to_owned(),
                lines: Vec::new(),
            });
        } else if let Some(builder) = current.as_mut() {
            builder.lines.push(line.to_owned());
        }
    }
    flush_section(&mut builders, &mut current)?;
    Ok(builders)
}
