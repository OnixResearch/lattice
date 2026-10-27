use super::super::*;
use super::support::*;

#[test]
fn lifecycle_reporting_covers_start_failure_continuation_and_aggregation() {
    let start = SubagentEvent::validate(&SubagentEventDraft {
        child_id: CHILD_ID,
        parent_id: PARENT_ID,
        kind: SubagentEventKind::Start.as_str(),
        status: EVENT_STARTED,
        evidence_ref: None,
    })
    .expect("start event should validate");
    let failure = SubagentEvent::validate(&SubagentEventDraft {
        child_id: CHILD_ID,
        parent_id: PARENT_ID,
        kind: SubagentEventKind::Failure.as_str(),
        status: OUTCOME_FAILURE,
        evidence_ref: Some(FAILURE_EVIDENCE_REF),
    })
    .expect("failure event should validate");
    let continuation = SubagentEvent::validate(&SubagentEventDraft {
        child_id: CHILD_ID,
        parent_id: PARENT_ID,
        kind: SubagentEventKind::Continuation.as_str(),
        status: EVENT_AWAITING_PARENT,
        evidence_ref: None,
    })
    .expect("continuation event should validate");
    let aggregation = SubagentEvent::validate(&SubagentEventDraft {
        child_id: CHILD_ID,
        parent_id: PARENT_ID,
        kind: SubagentEventKind::Aggregation.as_str(),
        status: OUTCOME_PARTIAL_PARALLEL_SUCCESS,
        evidence_ref: Some(AGGREGATE_SUMMARY_REF),
    })
    .expect("aggregation event should validate");
    let bad_kind = SubagentEvent::validate(&SubagentEventDraft {
        child_id: CHILD_ID,
        parent_id: PARENT_ID,
        kind: BAD_EVENT_KIND,
        status: OUTCOME_FAILURE,
        evidence_ref: None,
    })
    .expect_err("unknown event kind should fail");

    assert_eq!(start.child_id(), CHILD_ID);
    assert_eq!(start.parent_id(), PARENT_ID);
    assert_eq!(start.kind(), SubagentEventKind::Start);
    assert_eq!(failure.kind(), SubagentEventKind::Failure);
    assert_eq!(failure.status(), OUTCOME_FAILURE);
    assert_eq!(failure.evidence_ref(), Some(FAILURE_EVIDENCE_REF));
    assert_eq!(continuation.kind(), SubagentEventKind::Continuation);
    assert_eq!(aggregation.kind(), SubagentEventKind::Aggregation);
    assert_eq!(
        bad_kind,
        SubagentError::UnsupportedValue {
            field: EVENT_KIND_FIELD,
            value: Text::from(BAD_EVENT_KIND),
        }
    );
}

#[test]
fn aggregation_keeps_failures_and_write_conflicts_visible() {
    let partial = aggregate_subagent_outcomes(&[
        SubagentOutcomeDraft {
            child_id: CHILD_OK,
            kind: OUTCOME_SUCCESS,
            summary_valid: true,
            write_conflict: false,
        },
        SubagentOutcomeDraft {
            child_id: CHILD_TIMEOUT,
            kind: OUTCOME_TIMEOUT,
            summary_valid: false,
            write_conflict: false,
        },
    ])
    .expect("outcomes should aggregate");
    let conflict = aggregate_subagent_outcomes(&[SubagentOutcomeDraft {
        child_id: CHILD_CONFLICT,
        kind: OUTCOME_WRITE_CONFLICT,
        summary_valid: false,
        write_conflict: true,
    }])
    .expect("conflict outcome should aggregate");

    assert_eq!(
        partial.status(),
        SubagentOutcomeKind::PartialParallelSuccess
    );
    assert_eq!(partial.success_count(), MIN_TURN_COUNT);
    assert_eq!(partial.failure_count(), MIN_TURN_COUNT);
    assert_eq!(conflict.status(), SubagentOutcomeKind::WriteConflict);
    assert!(conflict.write_conflict());
}
