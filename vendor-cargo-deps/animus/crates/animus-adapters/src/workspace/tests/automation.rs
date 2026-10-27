#![allow(unknown_lints, path_segment_repetition)]

use super::support::*;

const AUTOMATION_ID: &str = "automation:nightly-docs";
const PROMPT: &str = "summarize bounded workspace findings";
const STOP_CRITERION: &str = "stop-after-first-report";
const POLICY_PROFILE: &str = "policy:full-access-reviewed";
const CADENCE_DAILY: &str = "daily";
const CADENCE_INVALID: &str = "every few minutes maybe";
const CADENCE_THREAD_WAKEUP: &str = "thread_wakeup";
const KIND_SCHEDULED: &str = "scheduled_standalone";
const KIND_THREAD: &str = "thread_wakeup";
const SANDBOX_READ_ONLY: &str = "read_only";
const SANDBOX_FULL_ACCESS: &str = "full_access";
const EVENT_WORKSPACE_CLEANUP: &str = "workspace_cleanup";
const EVENT_RESTORE_AVAILABLE: &str = "restore_available";
const MANIFEST_HASH: &str = "blake3:automation-manifest";
const SNAPSHOT_ID: &str = "snap:automation-restore";
const EXPECTED_STOP_COUNT: usize = 1;
const INVALID_AUTOMATION_CASES: usize = 3;

#[test]
fn automation_contract_accepts_schedule_and_thread_wakeup() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.automation_contract]
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.automation_contract.validation]
    let scheduled = crate::WorkspaceAutomationRunContract::new(valid_contract_draft())
        .expect("scheduled standalone automation should validate");
    let thread_wakeup =
        crate::WorkspaceAutomationRunContract::new(crate::WorkspaceAutomationRunContractDraft {
            automation_id: "automation:thread-wakeup",
            kind: KIND_THREAD,
            cadence: CADENCE_THREAD_WAKEUP,
            ..valid_contract_draft()
        })
        .expect("thread-bound wakeup should validate");

    assert_eq!(scheduled.automation_id(), AUTOMATION_ID);
    assert_eq!(
        scheduled.kind(),
        crate::WorkspaceAutomationKind::ScheduledStandalone
    );
    assert_eq!(scheduled.cadence(), CADENCE_DAILY);
    assert_eq!(scheduled.workspace_id(), WORKSPACE_ID);
    assert_eq!(
        scheduled.sandbox_profile(),
        crate::WorkspaceAutomationSandboxProfile::ReadOnly
    );
    assert_eq!(scheduled.stop_criteria().len(), EXPECTED_STOP_COUNT);
    assert_eq!(
        thread_wakeup.kind(),
        crate::WorkspaceAutomationKind::ThreadWakeup
    );
}

#[test]
fn automation_contract_rejects_invalid_cadence_missing_target_and_unadmitted_full_access() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.automation_contract.validation]
    let cases = [
        crate::WorkspaceAutomationRunContractDraft {
            cadence: CADENCE_INVALID,
            ..valid_contract_draft()
        },
        crate::WorkspaceAutomationRunContractDraft {
            workspace_id: "",
            ..valid_contract_draft()
        },
        crate::WorkspaceAutomationRunContractDraft {
            sandbox_profile: SANDBOX_FULL_ACCESS,
            policy_profile: None,
            ..valid_contract_draft()
        },
    ];

    assert_eq!(cases.len(), INVALID_AUTOMATION_CASES);
    for draft in cases {
        let error = crate::WorkspaceAutomationRunContract::new(draft)
            .expect_err("invalid automation must fail before scheduler registration");
        assert!(!alloc::format!("{error}").is_empty());
    }
}

#[test]
fn full_access_automation_requires_explicit_policy_profile() {
    let contract =
        crate::WorkspaceAutomationRunContract::new(crate::WorkspaceAutomationRunContractDraft {
            sandbox_profile: SANDBOX_FULL_ACCESS,
            policy_profile: Some(POLICY_PROFILE),
            ..valid_contract_draft()
        })
        .expect("reviewed full-access automation profile should validate");

    assert_eq!(contract.policy_profile(), Some(POLICY_PROFILE));
}

#[test]
fn cleanup_events_require_restore_evidence() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.cleanup_reporting]
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.cleanup_reporting.restore]
    let cleanup = crate::WorkspaceAutomationEvent::new(crate::WorkspaceAutomationEventDraft {
        kind: EVENT_WORKSPACE_CLEANUP,
        automation_id: AUTOMATION_ID,
        workspace_id: WORKSPACE_ID,
        manifest_hash: MANIFEST_HASH,
        restore_snapshot_id: Some(SNAPSHOT_ID),
    })
    .expect("cleanup event with restore evidence should validate");
    let restore = crate::WorkspaceAutomationEvent::new(crate::WorkspaceAutomationEventDraft {
        kind: EVENT_RESTORE_AVAILABLE,
        automation_id: AUTOMATION_ID,
        workspace_id: WORKSPACE_ID,
        manifest_hash: MANIFEST_HASH,
        restore_snapshot_id: Some(SNAPSHOT_ID),
    })
    .expect("restore-available event should validate");

    assert_eq!(
        cleanup.kind(),
        crate::WorkspaceAutomationEventKind::WorkspaceCleanup
    );
    assert_eq!(cleanup.restore_snapshot_id(), Some(SNAPSHOT_ID));
    assert_eq!(
        restore.kind(),
        crate::WorkspaceAutomationEventKind::RestoreAvailable
    );
}

#[test]
fn cleanup_event_without_restore_evidence_fails_closed() {
    // r[verify agent_harness_runtime.openai_codex_worktree_automations.cleanup_reporting.restore]
    let error = crate::WorkspaceAutomationEvent::new(crate::WorkspaceAutomationEventDraft {
        kind: EVENT_WORKSPACE_CLEANUP,
        automation_id: AUTOMATION_ID,
        workspace_id: WORKSPACE_ID,
        manifest_hash: MANIFEST_HASH,
        restore_snapshot_id: None,
    })
    .expect_err("cleanup without restore evidence must fail");

    assert_eq!(
        error,
        crate::AdapterContractError::EmptyField {
            field: "automation_event_restore_snapshot_id",
        }
    );
}

fn valid_contract_draft() -> crate::WorkspaceAutomationRunContractDraft<'static> {
    crate::WorkspaceAutomationRunContractDraft {
        automation_id: AUTOMATION_ID,
        kind: KIND_SCHEDULED,
        prompt: PROMPT,
        cadence: CADENCE_DAILY,
        workspace_id: WORKSPACE_ID,
        sandbox_profile: SANDBOX_READ_ONLY,
        stop_criteria: &[STOP_CRITERION],
        policy_profile: None,
    }
}
