use super::super::*;
use super::support::*;

#[test]
fn tracking_distinguishes_open_fixed_dismissed_and_new_findings() {
    let prior_findings = [
        finding(FindingInput {
            finding_id: FINDING_ID,
            source_label: TARGET_LABEL,
        }),
        finding(FindingInput {
            finding_id: "dismissed",
            source_label: TARGET_LABEL,
        }),
    ];
    let prior = report(RUN_ID, &prior_findings).expect("prior report should validate");
    let current_findings = [
        finding(FindingInput {
            finding_id: NEXT_FINDING_ID,
            source_label: TARGET_LABEL,
        }),
        finding(FindingInput {
            finding_id: FINDING_ID,
            source_label: TARGET_LABEL,
        }),
    ];
    let current = report(NEXT_RUN_ID, &current_findings).expect("current report should validate");
    let records =
        compare_diff_review_rerun(&prior, &current, &["dismissed"]).expect("rerun comparison should validate");

    assert_eq!(records.len(), EXPECTED_RERUN_RECORDS);
    assert_eq!(records[OPEN_RECORD_INDEX].finding_id(), FINDING_ID);
    assert_eq!(records[OPEN_RECORD_INDEX].state(), DiffReviewFindingState::Open);
    assert_eq!(records[DISMISSED_RECORD_INDEX].finding_id(), "dismissed");
    assert_eq!(records[DISMISSED_RECORD_INDEX].state(), DiffReviewFindingState::Dismissed);
    assert_eq!(records[NEW_RECORD_INDEX].finding_id(), NEXT_FINDING_ID);
    assert_eq!(records[NEW_RECORD_INDEX].state(), DiffReviewFindingState::NewlyIntroduced);

    let fixed_prior_findings = [finding(FindingInput {
        finding_id: FINDING_ID,
        source_label: TARGET_LABEL,
    })];
    let fixed_prior = report(RUN_ID, &fixed_prior_findings).expect("fixed prior report should validate");
    let fixed_current = report(NEXT_RUN_ID, &[]).expect("empty current report should validate");
    let fixed_records =
        compare_diff_review_rerun(&fixed_prior, &fixed_current, &[]).expect("fixed comparison should validate");
    assert_eq!(fixed_records[FIXED_RECORD_INDEX].state(), DiffReviewFindingState::Fixed);
}
