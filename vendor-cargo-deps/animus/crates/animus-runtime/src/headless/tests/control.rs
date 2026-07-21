use super::super::*;
use super::support::*;

#[test]
fn controller_resume_and_cancellation_check_thread_identity_before_execution() {
    let snapshot = ThreadSnapshot::new(THREAD_ID, LEDGER_HASH, ThreadStatus::Running)
        .expect("snapshot should validate");
    let resume = ThreadResume::validate(&ThreadResumeDraft {
        thread_id: THREAD_ID,
        ledger_hash: LEDGER_HASH,
        expected_ledger_hash: Some(LEDGER_HASH),
    })
    .expect("resume should validate");
    let bad_resume = ThreadResume::validate(&ThreadResumeDraft {
        thread_id: THREAD_ID,
        ledger_hash: OTHER_LEDGER_HASH,
        expected_ledger_hash: Some(OTHER_LEDGER_HASH),
    })
    .expect("bad resume identity is internally consistent before snapshot check");
    let wrong_thread = ThreadResume::validate(&ThreadResumeDraft {
        thread_id: OTHER_THREAD_ID,
        ledger_hash: LEDGER_HASH,
        expected_ledger_hash: Some(LEDGER_HASH),
    })
    .expect("wrong thread identity is internally consistent before snapshot check");
    let completed = ThreadSnapshot::new(THREAD_ID, LEDGER_HASH, ThreadStatus::Completed)
        .expect("completed snapshot should validate");

    assert_eq!(validate_thread_resume(&snapshot, &resume), Ok(()));
    assert_eq!(validate_thread_cancellation(&snapshot, THREAD_ID), Ok(()));
    assert_eq!(
        validate_thread_resume(&snapshot, &bad_resume),
        Err(AutomationError::ResumeMismatch {
            thread_id: Text::from(THREAD_ID),
            expected_ledger_hash: Text::from(LEDGER_HASH),
            actual_ledger_hash: Text::from(OTHER_LEDGER_HASH),
        })
    );
    assert_eq!(
        validate_thread_resume(&snapshot, &wrong_thread),
        Err(AutomationError::ThreadIdentityMismatch {
            expected_thread_id: Text::from(THREAD_ID),
            actual_thread_id: Text::from(OTHER_THREAD_ID),
        })
    );
    assert_eq!(
        validate_thread_cancellation(&completed, THREAD_ID),
        Err(AutomationError::ThreadNotRunning {
            thread_id: Text::from(THREAD_ID),
            status: ThreadStatus::Completed,
        })
    );
}
