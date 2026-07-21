/// Parsed structured review report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewReport {
    verdict: ReviewVerdict,
    source: String,
    stage: String,
    subject: String,
    report_hash: String,
    sections: Vec<ReviewSection>,
    findings: Vec<ReviewFinding>,
    diagnostics: Vec<ReviewDiagnostic>,
}

impl ReviewReport {
    /// Returns the review verdict.
    #[must_use]
    pub const fn verdict(&self) -> ReviewVerdict {
        self.verdict
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the safe stage label.
    #[must_use]
    pub fn stage(&self) -> &str {
        self.stage.as_str()
    }

    /// Returns the safe subject label.
    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Returns the BLAKE3 hash of the report text.
    #[must_use]
    pub fn report_hash(&self) -> &str {
        self.report_hash.as_str()
    }

    /// Returns parsed section metadata.
    #[must_use]
    pub fn sections(&self) -> &[ReviewSection] {
        self.sections.as_slice()
    }

    /// Returns parsed metric findings.
    #[must_use]
    pub fn findings(&self) -> &[ReviewFinding] {
        self.findings.as_slice()
    }

    /// Returns parser diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[ReviewDiagnostic] {
        self.diagnostics.as_slice()
    }
}

/// Safe metric finding copied into an append-only record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewMetricFinding {
    class: String,
    scope: String,
    route: String,
    promoted: bool,
    summary_hash: String,
}

impl ReviewMetricFinding {
    /// Returns the finding class tag.
    #[must_use]
    pub fn class(&self) -> &str {
        self.class.as_str()
    }

    /// Returns the finding scope tag.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.scope.as_str()
    }

    /// Returns the finding prevention route tag.
    #[must_use]
    pub fn route(&self) -> &str {
        self.route.as_str()
    }

    /// Returns whether the finding is already promoted.
    #[must_use]
    pub const fn promoted(&self) -> bool {
        self.promoted
    }

    /// Returns the BLAKE3 summary hash.
    #[must_use]
    pub fn summary_hash(&self) -> &str {
        self.summary_hash.as_str()
    }
}

/// Append-only safe metrics record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewMetricRecord {
    record_hash: String,
    sequence: String,
    source: String,
    stage: String,
    subject: String,
    report_hash: String,
    verdict: ReviewVerdict,
    findings: Vec<ReviewMetricFinding>,
}

impl ReviewMetricRecord {
    /// Returns the BLAKE3 record hash.
    #[must_use]
    pub fn record_hash(&self) -> &str {
        self.record_hash.as_str()
    }

    /// Returns the host-supplied sequence label.
    #[must_use]
    pub fn sequence(&self) -> &str {
        self.sequence.as_str()
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the safe stage label.
    #[must_use]
    pub fn stage(&self) -> &str {
        self.stage.as_str()
    }

    /// Returns the safe subject label.
    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Returns the source report hash.
    #[must_use]
    pub fn report_hash(&self) -> &str {
        self.report_hash.as_str()
    }

    /// Returns the review verdict.
    #[must_use]
    pub const fn verdict(&self) -> ReviewVerdict {
        self.verdict
    }

    /// Returns safe metric findings.
    #[must_use]
    pub fn findings(&self) -> &[ReviewMetricFinding] {
        self.findings.as_slice()
    }
}

/// Read-only aggregate over supplied review metric records.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReviewMetricsSummary {
    records: usize,
    findings: usize,
    pass_verdicts: usize,
    warn_verdicts: usize,
    fail_verdicts: usize,
}

impl ReviewMetricsSummary {
    /// Returns number of supplied records.
    #[must_use]
    pub const fn record_count(&self) -> usize {
        self.records
    }

    /// Returns total finding count across supplied records.
    #[must_use]
    pub const fn finding_count(&self) -> usize {
        self.findings
    }

    /// Returns PASS record count.
    #[must_use]
    pub const fn pass_count(&self) -> usize {
        self.pass_verdicts
    }

    /// Returns WARN record count.
    #[must_use]
    pub const fn warn_count(&self) -> usize {
        self.warn_verdicts
    }

    /// Returns FAIL record count.
    #[must_use]
    pub const fn fail_count(&self) -> usize {
        self.fail_verdicts
    }
}

/// Read-only repeated-finding promotion candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPromotionCandidate {
    class: String,
    scope: String,
    route: String,
    count: usize,
    example_hashes: Vec<String>,
}

impl ReviewPromotionCandidate {
    /// Returns the repeated finding class.
    #[must_use]
    pub fn class(&self) -> &str {
        self.class.as_str()
    }

    /// Returns the repeated finding scope.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.scope.as_str()
    }

    /// Returns the suggested prevention route.
    #[must_use]
    pub fn route(&self) -> &str {
        self.route.as_str()
    }

    /// Returns the number of matching unpromoted findings.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Returns representative summary hashes, not raw examples.
    #[must_use]
    pub fn example_hashes(&self) -> &[String] {
        self.example_hashes.as_slice()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SectionBuilder {
    title: String,
    lines: Vec<String>,
}

impl SectionBuilder {
    fn into_section(self) -> ReviewSection {
        let body = self.lines.join(SECTION_BODY_JOINER);
        ReviewSection {
            title: self.title,
            line_count: self.lines.len(),
            body_hash: hash_text(body.as_str()),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct FindingTags {
    class: Option<String>,
    scope: Option<String>,
    route: Option<String>,
    promoted: Option<bool>,
}
