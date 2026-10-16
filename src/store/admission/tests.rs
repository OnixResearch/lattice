use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static CRASH_ARMED: AtomicBool = AtomicBool::new(false);
const CHILD_PATH: &str = "LATTICE_COMMIT_CRASH_STORE";
const CHILD_MARKER: &str = "LATTICE_COMMIT_CRASH_MARKER";
const CRASH_EXIT: i32 = 91;

pub(super) fn arm_commit_crash() {
    CRASH_ARMED.store(true, Ordering::SeqCst);
}

#[derive(Debug)]
struct CrashBackend(redb::backends::FileBackend);

impl redb::StorageBackend for CrashBackend {
    fn len(&self) -> std::io::Result<u64> {
        self.0.len()
    }
    fn read(&self, offset: u64, out: &mut [u8]) -> std::io::Result<()> {
        self.0.read(offset, out)
    }
    fn set_len(&self, len: u64) -> std::io::Result<()> {
        self.0.set_len(len)
    }
    fn write(&self, offset: u64, data: &[u8]) -> std::io::Result<()> {
        self.0.write(offset, data)
    }
    fn close(&self) -> std::io::Result<()> {
        self.0.close()
    }
    fn sync_data(&self) -> std::io::Result<()> {
        self.0.sync_data()?;
        if CRASH_ARMED.swap(false, Ordering::SeqCst) {
            let marker = std::env::var_os(CHILD_MARKER).expect("child marker path");
            std::fs::write(marker, b"publication commit sync reached")?;
            // No Rust destructors run: the database dies inside the real commit.
            std::process::exit(CRASH_EXIT);
        }
        Ok(())
    }
}

fn request() -> crate::scheduler::ScheduleOccurrenceClaimRequest {
    crate::scheduler::plan_schedule_occurrence(
        &crate::scheduler::DueRunPlan {
            schedule_id: "commit-crash".to_string(),
            workflow_name: "commit-crash-workflow".to_string(),
            due_epoch_ms: 1_000,
            next_run_epoch_ms: 2_000,
        },
        "workflow-revision",
    )
    .expect("exact claim request")
}

#[test]
fn commit_crash_child() {
    let Some(path) = std::env::var_os(CHILD_PATH) else {
        return;
    };
    let file = std::fs::OpenOptions::new().read(true).write(true).open(path).expect("existing store");
    let backend = CrashBackend(redb::backends::FileBackend::new(file).expect("exclusive file backend"));
    let db = redb::Builder::new().create_with_backend(backend).expect("backend opens");
    let store = super::super::Store {
        db: std::sync::Arc::new(db),
        quarantine: std::sync::Arc::new(std::sync::Mutex::new(None)),
        occurrence_admission: std::sync::Arc::new(std::sync::Mutex::new(())),
    };
    let _ = store.claim_schedule_occurrence_with_fault(&request(), super::super::CommitFault::ArmCommitCrash);
    panic!("child must terminate inside publication commit");
}

#[test]
fn process_death_inside_publication_commit_keeps_restart_dispatch_fenced() {
    let directory = tempfile::tempdir().expect("temporary store");
    let path = directory.path().join("commit-crash.redb");
    let marker = directory.path().join("commit-sync-reached");
    let store = super::super::Store::new(&path).expect("store opens");
    store
        .save_schedule(&crate::scheduler::ScheduleDefinition {
            id: "commit-crash".to_string(),
            workflow_name: "commit-crash-workflow".to_string(),
            recurrence: crate::scheduler::RecurrenceRule::EverySeconds { interval_secs: 1 },
            enabled: true,
            overlap_policy: crate::scheduler::OverlapPolicy::Forbid,
            missed_run_policy: crate::scheduler::MissedRunPolicy::RunOnce,
            next_run_epoch_ms: 1_000,
        })
        .expect("schedule persists");
    drop(store);

    let result = std::process::Command::new(std::env::current_exe().expect("unit-test executable"))
        .args(["--exact", "store::admission::tests::commit_crash_child", "--nocapture"])
        .env(CHILD_PATH, &path)
        .env(CHILD_MARKER, &marker)
        .output()
        .expect("commit child executes");
    assert_eq!(result.status.code(), Some(CRASH_EXIT), "{:?}", result);
    assert!(marker.exists());
    assert_eq!(
        crate::run::shell::repair_path(&path).expect("redb structure repairs"),
        crate::run::shell::RepairOutcome::Repaired
    );
    let store = super::super::Store::new(&path).expect("repaired store opens");
    let occurrence = store
        .load_schedule_occurrence_for_recovery(&request().identity)
        .expect("recovery evidence remains readable")
        .expect("durable reservation or journal");
    assert!(store.is_quarantined().expect("durable quarantine"));
    // r[verify lattice.persistence_uncertainty.validation.scenario.commit_crash_fixture]
    assert!(matches!(
        store.admit_schedule_occurrence_dispatch(&occurrence),
        Err(super::super::StoreError::Quarantined)
    ));
    assert!(store.load_run(occurrence.active_run_id).expect("run inspection").is_none());
    assert_eq!(
        store.reconcile_schedule_occurrence_publication().expect("bounded reconciliation"),
        transactional_reconciliation_core::PersistenceState::PublicationUnknown
    );
    assert!(store.abandon_schedule_occurrence(&occurrence.identity, false).is_err());
    let closed = store.abandon_schedule_occurrence(&occurrence.identity, true).expect("acknowledged closure");
    assert_eq!(closed.status, crate::scheduler::ScheduleOccurrenceStatus::AbandonedUnknown);
    drop(store);
    let store = super::super::Store::new(&path).expect("closed store reopens");
    assert!(!store.is_quarantined().expect("closure persisted"));
    assert!(store.load_run(occurrence.active_run_id).expect("no invented effect result").is_none());
}

#[test]
fn repaired_missing_or_conflicting_rows_cannot_remove_dispatch_fence() {
    for remove_row in [false, true] {
        let directory = tempfile::tempdir().expect("temporary store");
        let path = directory.path().join("readback.redb");
        let store = super::super::Store::new(&path).expect("store");
        store
            .save_schedule(&crate::scheduler::ScheduleDefinition {
                id: "commit-crash".to_string(),
                workflow_name: "commit-crash-workflow".to_string(),
                recurrence: crate::scheduler::RecurrenceRule::EverySeconds { interval_secs: 1 },
                enabled: true,
                overlap_policy: crate::scheduler::OverlapPolicy::Forbid,
                missed_run_policy: crate::scheduler::MissedRunPolicy::RunOnce,
                next_run_epoch_ms: 1_000,
            })
            .expect("schedule");
        let crate::scheduler::ScheduleOccurrenceClaimResolution::Inserted(expected) =
            store.claim_schedule_occurrence(&request()).expect("claim")
        else {
            panic!("fresh claim must insert")
        };
        drop(store);
        let store = super::super::Store::new(&path).expect("reopened history is fenced");
        let txn = store.db.begin_write().expect("controlled corruption transaction");
        {
            let mut rows = txn.open_table(super::OCCURRENCES).expect("occurrence table");
            if remove_row {
                rows.remove(expected.identity.as_str()).expect("missing readback fixture");
            } else {
                let mut foreign = (*expected).clone();
                foreign.workflow_revision = "foreign-revision".to_string();
                let bytes = super::super::serialize(&foreign).expect("foreign row encoding");
                rows.insert(expected.identity.as_str(), bytes.as_slice()).expect("conflicting readback fixture");
            }
        }
        txn.commit().expect("fixture persists");
        drop(store);
        let store = super::super::Store::new(&path).expect("fence survives another reopen");
        assert!(matches!(
            store.admit_schedule_occurrence_dispatch(&expected),
            Err(super::super::StoreError::Quarantined)
        ));
        assert_eq!(
            store.reconcile_schedule_occurrence_publication().expect("bounded readback"),
            transactional_reconciliation_core::PersistenceState::PublicationUnknown
        );
        assert!(store.is_quarantined().expect("fence remains"));
        assert!(store.load_run(expected.active_run_id).expect("run inspection").is_none());
        if !remove_row {
            assert!(store.retry_schedule_occurrence(&expected.identity, &expected.workflow_revision, true).is_err());
            assert!(store.abandon_schedule_occurrence(&expected.identity, true).is_err());
            assert!(store.is_quarantined().expect("acknowledgement cannot discard conflict"));
        }
    }
}

#[test]
fn oversized_uncertainty_record_cannot_publish_or_advance_schedule() {
    let directory = tempfile::tempdir().expect("temporary store");
    let store = super::super::Store::new(directory.path().join("bounded-fence.redb")).expect("store");
    store
        .save_schedule(&crate::scheduler::ScheduleDefinition {
            id: "commit-crash".to_string(),
            workflow_name: "commit-crash-workflow".to_string(),
            recurrence: crate::scheduler::RecurrenceRule::EverySeconds { interval_secs: 1 },
            enabled: true,
            overlap_policy: crate::scheduler::OverlapPolicy::Forbid,
            missed_run_policy: crate::scheduler::MissedRunPolicy::RunOnce,
            next_run_epoch_ms: 1_000,
        })
        .expect("schedule");
    let mut oversized = request();
    oversized.workflow_revision = "x".repeat(65_536);
    assert!(matches!(store.claim_schedule_occurrence(&oversized), Err(super::super::StoreError::Conflict(_))));
    assert!(store.list_schedule_occurrences(1).expect("no admitted reservation").is_empty());
    assert_eq!(
        store.load_schedule("commit-crash").expect("schedule").expect("saved schedule").next_run_epoch_ms,
        1_000
    );
    assert!(!store.is_quarantined().expect("rejected draft did not create an uncertain commit"));
}
