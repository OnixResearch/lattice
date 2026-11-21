//! Fence records and the per-work lock, driven through the step functions.

// r[verify lattice.implementation_attempt.consumer.scenario.fence_lock]
use super::super::super::refusal::Reason;
use super::Decision;
use super::Error;
use super::Refused;
use super::decide;

/// Set in the child process that holds the lock until its stdin closes.
const HOLDER_STATE: &str = "LATTICE_FENCE_LOCK_HOLDER_STATE";
const HOLDER_WORK: &str = "work-held-by-a-child";

struct Root {
    _root: tempfile::TempDir,
    state: std::path::PathBuf,
}

impl Root {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("state");
        let state = root.path().canonicalize().expect("canonical state");
        Self { _root: root, state }
    }

    /// An envelope of one work at `fence` and its attempt directory.
    fn attempt(&self, attempt_id: &str, fence: u64) -> (super::Envelope, std::path::PathBuf) {
        let mut envelope = super::super::super::tests::host_local().envelope;
        envelope.attempt_id = attempt_id.into();
        envelope.fence = fence;
        let directory = self.state.join(format!("{attempt_id}-{fence}"));
        super::super::super::artifacts::private_directory(&directory).expect("attempt directory");
        (envelope, directory)
    }

    fn lock_handle(&self, work_id: &str) -> std::fs::File {
        std::fs::File::open(super::work_directory(&self.state, work_id)).expect("work directory")
    }
}

fn stale(dispatched: bool) -> Result<(), Error> {
    Err(Error::Refused(Refused {
        reason: Reason::StaleFence,
        dispatched,
    }))
}

#[test]
// r[verify lattice.implementation_attempt.consumer.scenario.refusal]
fn fences_admit_first_and_exact_replay_but_refuse_stale_and_crossed() {
    assert_eq!(decide(None, None, 7, "a"), Ok(Decision::Record));
    assert_eq!(decide(Some(6), None, 7, "a"), Ok(Decision::Record));
    assert_eq!(decide(Some(7), Some("a"), 7, "a"), Ok(Decision::Replay));
    assert_eq!(decide(Some(8), None, 7, "a"), Err(Reason::StaleFence));
    assert_eq!(decide(Some(8), Some("a"), 7, "a"), Err(Reason::StaleFence));
    assert_eq!(decide(Some(7), Some("b"), 7, "a"), Err(Reason::CrossedAttempt));
}

#[test]
fn records_are_bounded_named_and_checked_in_a_state_root() {
    let root = Root::new();
    let (mut envelope, directory) = root.attempt("attempt-host-local", 7);
    drop(super::admit(&root.state, &envelope, &directory).expect("first record"));
    let guard = super::admit(&root.state, &envelope, &directory).expect("exact replay");
    guard.check_crossed(&envelope, &directory).expect("recovery of the recorded attempt");
    drop(guard);
    let (crossed, crossed_directory) = root.attempt("attempt-crossed", 7);
    assert_eq!(
        super::admit(&root.state, &crossed, &crossed_directory).map(drop),
        Err(Reason::CrossedAttempt.message().to_owned())
    );
    let guard = super::Guard::acquire(&root.state, &crossed.work_id).expect("lock");
    let refused = Refused {
        reason: Reason::CrossedAttempt,
        dispatched: false,
    };
    assert_eq!(guard.check_crossed(&crossed, &crossed_directory), Err(Error::Refused(refused)));
    drop(guard);
    envelope.fence += 1;
    envelope.attempt_id = "attempt-successor".into();
    drop(super::admit(&root.state, &envelope, &directory).expect("successor fence"));
    let (late, late_directory) = root.attempt("attempt-late", 7);
    assert_eq!(
        super::admit(&root.state, &late, &late_directory).map(drop),
        Err(Reason::StaleFence.message().to_owned())
    );
    let work = super::work_directory(&root.state, &envelope.work_id);
    std::fs::write(work.join("not-a-fence"), b"x").expect("foreign entry");
    assert!(super::admit(&root.state, &envelope, &directory).is_err(), "unexpected entries fail closed");
}

#[test]
fn a_run_that_passed_its_check_creates_no_intent_once_a_higher_fence_is_recorded() {
    let root = Root::new();
    let (seven, seven_directory) = root.attempt("attempt-seven", 7);
    let (eight, eight_directory) = root.attempt("attempt-eight", 8);
    // The fence-7 run records its fence and pauses before its intent.
    drop(super::admit(&root.state, &seven, &seven_directory).expect("fence 7 passes its check"));
    drop(super::admit(&root.state, &eight, &eight_directory).expect("fence 8 is recorded"));
    let guard = super::Guard::acquire(&root.state, &seven.work_id).expect("lock");
    assert!(!guard.is_current(&seven).expect("fence records"), "a repair step would stop here");
    assert_eq!(guard.create_intent(&seven, &seven_directory, b"intent").map(drop), stale(false));
    assert!(!seven_directory.join(super::super::intent::FILE).exists(), "no intent, so no worker");
    assert!(guard.is_current(&eight).expect("fence records"));
    assert!(guard.create_intent(&eight, &eight_directory, b"intent").expect("the current fence"));
}

#[test]
fn a_stale_refusal_reports_whether_its_attempt_holds_an_intent() {
    let root = Root::new();
    let (seven, seven_directory) = root.attempt("attempt-seven", 7);
    let (eight, eight_directory) = root.attempt("attempt-eight", 8);
    let guard = super::admit(&root.state, &seven, &seven_directory).expect("fence 7");
    assert!(guard.create_intent(&seven, &seven_directory, b"intent").expect("fence 7 intent"));
    drop(guard);
    drop(super::admit(&root.state, &eight, &eight_directory).expect("fence 8 is recorded"));
    let guard = super::Guard::acquire(&root.state, &seven.work_id).expect("lock");
    assert_eq!(guard.record(&seven, &seven_directory), stale(true), "the fence-7 intent may have executed");
    let printed = super::super::super::refusal::record_with(&seven, Reason::StaleFence, true);
    assert_eq!((printed["reason"].clone(), printed["dispatched"].clone()), ("stale_fence".into(), true.into()));
    drop(guard);

    let (other, other_directory) = root.attempt("attempt-other", 7);
    let guard = super::Guard::acquire(&root.state, &other.work_id).expect("lock");
    assert_eq!(guard.record(&other, &other_directory), stale(false), "no fence-7 intent of this attempt");
    // Afterwards no fence-7 run of this attempt can create the intent.
    assert_eq!(guard.create_intent(&other, &other_directory, b"intent").map(drop), stale(false));
    drop(guard);
    assert_eq!(
        super::admit(&root.state, &other, &other_directory).map(drop),
        Err(Reason::StaleFence.message().to_owned())
    );
    assert!(!other_directory.join(super::super::intent::FILE).exists());

    // Recovery's closure marker is not an execution intent.
    let marker = super::super::super::recovery::closed_marker(&"1".repeat(64));
    std::fs::write(other_directory.join(super::super::intent::FILE), marker).expect("closure marker");
    let guard = super::Guard::acquire(&root.state, &other.work_id).expect("lock");
    assert_eq!(guard.record(&other, &other_directory), stale(false));
}

#[test]
fn the_lock_excludes_other_handles_until_its_guard_drops() {
    let root = Root::new();
    let guard = super::Guard::acquire(&root.state, "work-6").expect("lock");
    let other = root.lock_handle("work-6");
    assert!(matches!(other.try_lock(), Err(std::fs::TryLockError::WouldBlock)), "held");
    drop(guard);
    // Blocking, not `try_lock`: a process another test forks may briefly
    // share the dropped descriptor until its `exec` closes it.
    other.lock().expect("released when the holder drops its handle");
}

#[test]
fn the_lock_is_released_when_its_holder_process_exits() {
    let root = Root::new();
    let test = module_path!().split_once("::").map_or("", |(_, path)| path);
    let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            &format!("{test}::lock_holder_child"),
            "--nocapture",
            "--test-threads=1",
        ])
        .env(HOLDER_STATE, &root.state)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("child holder");
    let mut stdout = std::io::BufReader::new(child.stdout.take().expect("child stdout"));
    let mut line = String::new();
    while !line.contains("lock-held") {
        line.clear();
        let read = std::io::BufRead::read_line(&mut stdout, &mut line).expect("child output");
        assert_ne!(read, 0, "the child exited before holding the lock");
    }
    let other = root.lock_handle(HOLDER_WORK);
    assert!(matches!(other.try_lock(), Err(std::fs::TryLockError::WouldBlock)), "held by the child");
    drop(child.stdin.take());
    assert!(child.wait().expect("child exits").success());
    other.try_lock().expect("released when the holder process exits");
}

/// Child half of the test above; a no-op unless the parent names a state.
#[test]
fn lock_holder_child() {
    let Some(state) = std::env::var_os(HOLDER_STATE) else {
        return;
    };
    let guard = super::Guard::acquire(std::path::Path::new(&state), HOLDER_WORK).expect("child lock");
    println!("lock-held");
    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input).expect("parent stdin");
    // Exit with the guard alive: only process exit can release it.
    std::mem::forget(guard);
    std::process::exit(0);
}
