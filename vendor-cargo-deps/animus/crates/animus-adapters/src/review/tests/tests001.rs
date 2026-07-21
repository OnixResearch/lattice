const SOURCE: &str = "hermes-review-gate";
const STAGE: &str = "final-review";
const SUBJECT: &str = "animus-runtime";
const SEQUENCE_ONE: &str = "sequence-one";
const SEQUENCE_TWO: &str = "sequence-two";
const SEQUENCE_THREE: &str = "sequence-three";
const SINGLE_FINDING_COUNT: usize = 1;
const TWO_FINDING_COUNT: usize = 2;
const THREE_RECORD_COUNT: usize = 3;
const ZERO_COUNT: usize = 0;

#[derive(Clone, Copy)]
struct RecordInput<'a> {
    markdown: &'a str,
    sequence: &'a str,
}

fn draft(markdown: &str) -> super::ReviewReportDraft<'_> {
    super::ReviewReportDraft {
        markdown,
        source: SOURCE,
        stage: STAGE,
        subject: SUBJECT,
    }
}

fn record(
    input: RecordInput<'_>,
) -> Result<super::ReviewMetricRecord, super::ReviewReportError> {
    let report = super::parse_review_report(&draft(input.markdown))?;
    super::append_review_metric_record(
        &report,
        &super::ReviewMetricRecordDraft {
            sequence: input.sequence,
        },
    )
}

#[test]
fn valid_pass_warn_and_fail_reports_parse_deterministically() {
    // r[verify agent_harness_runtime.hermes_review_reports.report.valid]
    let pass = super::parse_review_report(&draft("VERDICT: PASS\n\n## Findings\n")).unwrap();
    let warn = super::parse_review_report(&draft(
        "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=test promoted=no] Add a negative fixture.\n\n## Gaps\n- human prose\n",
    ))
    .unwrap();
    let fail = super::parse_review_report(&draft(
        "VERDICT: FAIL\n\n## Findings\n- [class=safety scope=receipt route=check promoted=yes] Existing rail covers this.\n\n## Next Steps\n- keep reviewable\n",
    ))
    .unwrap();

    assert_eq!(pass.verdict(), super::ReviewVerdict::Pass);
    assert_eq!(warn.verdict(), super::ReviewVerdict::Warn);
    assert_eq!(fail.verdict(), super::ReviewVerdict::Fail);
    assert_eq!(warn.findings().len(), SINGLE_FINDING_COUNT);
    assert!(warn.diagnostics().is_empty());
    assert_eq!(fail.sections().len(), TWO_FINDING_COUNT);
}

#[test]
fn checklist_bullets_outside_findings_are_ignored() {
    // r[verify agent_harness_runtime.hermes_review_reports.findings_scope.ignore_checklists]
    let report = super::parse_review_report(&draft(
        "VERDICT: WARN\n\n## OpenSpec Check\n- [ ] this is not a metric\n\n## Findings\n- [class=docs scope=lifecycle route=spec promoted=no] Missing closeout note.\n\n## Gaps\n- [class=fake scope=fake route=fake promoted=no] prose only\n",
    ))
    .unwrap();

    assert_eq!(report.findings().len(), SINGLE_FINDING_COUNT);
    assert_eq!(report.findings()[ZERO_COUNT].class(), "docs");
    assert!(report.diagnostics().is_empty());
}

#[test]
fn malformed_verdict_and_finding_tags_are_typed() {
    // r[verify agent_harness_runtime.hermes_review_reports.finding_tags.invalid]
    let verdict_error = super::parse_review_report(&draft("VERDICT: MAYBE\n\n## Findings\n"))
        .expect_err("unsupported verdict should fail");
    assert_eq!(
        verdict_error.code(),
        super::ReviewDiagnosticCode::MalformedVerdict
    );

    let missing_route = super::parse_review_report(&draft(
        "VERDICT: WARN\n\n## Findings\n- [class=docs scope=lifecycle promoted=no] Missing route tag.\n",
    ))
    .unwrap();
    assert!(missing_route.diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == super::ReviewDiagnosticCode::MissingFindingTag
            && diagnostic.field() == "route"
    }));
    assert!(
        super::append_review_metric_record(
            &missing_route,
            &super::ReviewMetricRecordDraft {
                sequence: SEQUENCE_ONE,
            },
        )
        .is_err()
    );

    let duplicate_and_unknown = super::parse_review_report(&draft(
        "VERDICT: WARN\n\n## Findings\n- [class=docs class=lint scope=lifecycle route=test promoted=maybe] Bad tags.\n",
    ))
    .unwrap();
    assert!(
        duplicate_and_unknown
            .diagnostics()
            .iter()
            .any(|diagnostic| {
                diagnostic.code() == super::ReviewDiagnosticCode::DuplicateFindingTag
                    && diagnostic.field() == "class"
            })
    );
    assert!(
        duplicate_and_unknown
            .diagnostics()
            .iter()
            .any(|diagnostic| {
                diagnostic.code() == super::ReviewDiagnosticCode::UnknownPromotedValue
                    && diagnostic.field() == "promoted"
            })
    );
}
