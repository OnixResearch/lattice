use super::super::*;
use super::support::*;

#[test]
fn return_modes_bound_parent_detail() {
    let summary_only = build_subagent_handoff(&SubagentHandoffDraft {
        return_mode: RETURN_SUMMARY_ONLY,
        status: OUTCOME_SUCCESS,
        summary: "bounded summary",
        detail_ref: Some("raw-log-ref"),
        detail_chars: EXCESSIVE_DETAIL_CHARS,
        max_detail_chars: MAX_DETAIL_CHARS,
    })
    .expect("summary-only ignores detail import");
    let full_detail = build_subagent_handoff(&SubagentHandoffDraft {
        return_mode: RETURN_FULL_DETAIL_BOUNDED,
        status: OUTCOME_SUCCESS,
        summary: "bounded summary",
        detail_ref: Some("detail-ref"),
        detail_chars: DETAIL_CHARS,
        max_detail_chars: MAX_DETAIL_CHARS,
    })
    .expect("bounded detail should import");

    assert_eq!(summary_only.detail_ref(), None);
    assert_eq!(full_detail.detail_ref(), Some("detail-ref"));
    assert_eq!(
        build_subagent_handoff(&SubagentHandoffDraft {
            return_mode: RETURN_FULL_DETAIL_BOUNDED,
            status: OUTCOME_SUCCESS,
            summary: "bounded summary",
            detail_ref: Some("detail-ref"),
            detail_chars: EXCESSIVE_DETAIL_CHARS,
            max_detail_chars: MAX_DETAIL_CHARS,
        }),
        Err(SubagentError::DetailBoundsExceeded)
    );
}
