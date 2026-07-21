use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use lattice::engine::RunId;
use lattice::run::inspection::Report;
use lattice::store::RunOwner;
use lattice::store::RunRecord;
use lattice::store::Store;

const CHILD_TEST_NAME: &str = "run_store_child_entrypoint";
const CHILD_PHASE_ENV: &str = "LATTICE_RUN_STORE_CHILD_PHASE";
const CHILD_STORE_ENV: &str = "LATTICE_RUN_STORE_CHILD_PATH";
const CHILD_READY_ENV: &str = "LATTICE_RUN_STORE_CHILD_READY";
const CHILD_RELEASE_ENV: &str = "LATTICE_RUN_STORE_CHILD_RELEASE";
const PHASE_CLEAN: &str = "clean";
const PHASE_PANIC: &str = "panic";
const PHASE_ABORT: &str = "abort";
const PHASE_HOLD: &str = "hold";
const PHASE_INVALID: &str = "invalid";
const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "lattice.redb";
const PRIVATE_WORKFLOW_NAME: &str = "PRIVATE_POST_FAILURE_WORKFLOW_DO_NOT_RENDER";
const PRIVATE_CORRUPTION: &str = "PRIVATE_CORRUPTION_DO_NOT_RENDER";
const RUN_ID_VALUE: u64 = 0;
const STARTED_EPOCH_MS: i64 = 1_000;
const FINISHED_EPOCH_MS: i64 = 2_000;
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 1;
const EMPTY_COUNT: usize = 0;
const ONE_RECORD: usize = 1;
const HOLD_POLL_MILLIS: u64 = 10;
const HOLD_POLL_LIMIT: usize = 1_000;
const REPAIR_SUCCESS: &str = "Run evidence store recovery completed.\n";
const ALREADY_CLEAN: &str = "Run evidence store is already clean.\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildPhase {
    Clean,
    Panic,
    Abort,
    Hold,
}

fn parse_child_phase(value: &str) -> Result<ChildPhase, &'static str> {
    match value {
        PHASE_CLEAN => Ok(ChildPhase::Clean),
        PHASE_PANIC => Ok(ChildPhase::Panic),
        PHASE_ABORT => Ok(ChildPhase::Abort),
        PHASE_HOLD => Ok(ChildPhase::Hold),
        _ => Err("unknown run store child phase"),
    }
}

fn record() -> RunRecord {
    RunRecord {
        run_id: RunId(RUN_ID_VALUE),
        owner: RunOwner::Unscoped,
        workflow_name: PRIVATE_WORKFLOW_NAME.to_string(),
        started_epoch_ms: STARTED_EPOCH_MS,
        finished_epoch_ms: Some(FINISHED_EPOCH_MS),
        node_count: EMPTY_COUNT,
        cache_hits: EMPTY_COUNT,
        failures: EMPTY_COUNT,
    }
}

fn assert_seed_record(actual: &RunRecord) {
    assert_eq!(actual.run_id, RunId(RUN_ID_VALUE));
    assert!(matches!(actual.owner, RunOwner::Unscoped));
    assert_eq!(actual.workflow_name, PRIVATE_WORKFLOW_NAME);
    assert_eq!(actual.started_epoch_ms, STARTED_EPOCH_MS);
    assert_eq!(actual.finished_epoch_ms, Some(FINISHED_EPOCH_MS));
    assert_eq!(actual.node_count, EMPTY_COUNT);
    assert_eq!(actual.cache_hits, EMPTY_COUNT);
    assert_eq!(actual.failures, EMPTY_COUNT);
}

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let directory = tempfile::tempdir().expect("temporary directory must exist");
    let data_dir = directory.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data directory must exist");
    let config_path = directory.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string()
        ),
    )
    .expect("config must persist");
    let store_path = data_dir.join(STORE_FILE_NAME);
    (directory, config_path, store_path)
}

fn child_command(phase: &str, store_path: &Path, ready_path: &Path, release_path: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test executable must resolve"));
    command
        .arg("--exact")
        .arg(CHILD_TEST_NAME)
        .env(CHILD_PHASE_ENV, phase)
        .env(CHILD_STORE_ENV, store_path)
        .env(CHILD_READY_ENV, ready_path)
        .env(CHILD_RELEASE_ENV, release_path);
    command
}

fn invoke_child(phase: &str, store_path: &Path, ready_path: &Path, release_path: &Path) -> std::process::Output {
    child_command(phase, store_path, ready_path, release_path)
        .output()
        .expect("child process must execute")
}

fn run_show(config_path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(config_path)
        .arg("runs")
        .arg("show")
        .arg(RUN_ID_VALUE.to_string())
        .arg("--output")
        .arg("json")
        .output()
        .expect("runs show must execute")
}

fn run_repair(config_path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(config_path)
        .arg("runs")
        .arg("repair")
        .output()
        .expect("runs repair must execute")
}

fn file_identity(path: &Path) -> blake3::Hash {
    blake3::hash(&std::fs::read(path).expect("store bytes must load"))
}

fn wait_for_marker(path: &Path) {
    for _ in 0..HOLD_POLL_LIMIT {
        if path.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(HOLD_POLL_MILLIS));
    }
    panic!("child marker was not created within the bounded wait");
}

fn assert_fixed_error(output: &std::process::Output, expected: &str, forbidden_path: &Path) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(expected), "stderr: {stderr}");
    assert!(!stderr.contains(PRIVATE_WORKFLOW_NAME));
    assert!(!stderr.contains(PRIVATE_CORRUPTION));
    assert!(!stderr.contains("Database repair aborted"));
    assert!(!stderr.contains("Cannot acquire lock"));
    assert!(!stderr.contains(&forbidden_path.display().to_string()));
}

// r[verify lattice.workflow_execution.run_store_recovery]
#[test]
fn run_store_child_entrypoint() {
    let Ok(phase_value) = std::env::var(CHILD_PHASE_ENV) else {
        return;
    };
    let phase = parse_child_phase(&phase_value).expect("child phase must be recognized before store open");
    let store_path = std::env::var_os(CHILD_STORE_ENV).expect("child store path must exist");
    let ready_path = std::env::var_os(CHILD_READY_ENV).expect("child ready path must exist");
    let release_path = std::env::var_os(CHILD_RELEASE_ENV).expect("child release path must exist");
    let store = Store::new(Path::new(&store_path)).expect("child store must open");
    store.save_run(&record()).expect("child run must persist");
    match phase {
        ChildPhase::Clean => drop(store),
        ChildPhase::Panic => panic!("intentional writer panic after durable commit"),
        ChildPhase::Abort => std::process::abort(),
        ChildPhase::Hold => {
            std::fs::write(&ready_path, "ready").expect("child ready marker must persist");
            wait_for_marker(Path::new(&release_path));
            drop(store);
        }
    }
}

// r[verify lattice.workflow_execution.safe_run_inspection.scenario.recovery-required]
// r[verify lattice.workflow_execution.run_store_recovery.scenario.success]
#[test]
fn panic_and_abort_require_explicit_repair_before_safe_inspection() {
    for phase in [PHASE_PANIC, PHASE_ABORT] {
        let (_directory, config_path, store_path) = fixture();
        let ready_path = store_path.with_extension("ready");
        let release_path = store_path.with_extension("release");
        let child = invoke_child(phase, &store_path, &ready_path, &release_path);
        assert!(!child.status.success());
        let identity_before = file_identity(&store_path);

        let blocked = run_show(&config_path);
        assert_fixed_error(&blocked, "requires recovery", &store_path);
        assert!(String::from_utf8_lossy(&blocked.stderr).contains("runs repair"));
        assert_eq!(file_identity(&store_path), identity_before);

        let repaired = run_repair(&config_path);
        assert!(repaired.status.success(), "stderr: {}", String::from_utf8_lossy(&repaired.stderr));
        assert_eq!(String::from_utf8_lossy(&repaired.stdout), REPAIR_SUCCESS);
        assert!(repaired.stderr.is_empty());
        let inspected = run_show(&config_path);
        assert!(inspected.status.success(), "stderr: {}", String::from_utf8_lossy(&inspected.stderr));
        let report: Report = serde_json::from_slice(&inspected.stdout).expect("safe report must decode");
        assert!(report.valid);
        assert_eq!(report.run_id, RUN_ID_VALUE);
        let store = Store::new(&store_path).expect("repaired store must reopen normally");
        let runs = store.list_runs(ONE_RECORD).expect("run list must load");
        assert_eq!(runs.len(), ONE_RECORD);
        assert_seed_record(&runs[0]);
    }
}

// r[verify lattice.workflow_execution.safe_run_inspection.scenario.active-owner]
// r[verify lattice.workflow_execution.run_store_recovery.scenario.refusal]
#[test]
fn active_owner_blocks_inspection_and_repair_without_backend_details() {
    let (_directory, config_path, store_path) = fixture();
    let ready_path = store_path.with_extension("ready");
    let release_path = store_path.with_extension("release");
    let mut child = child_command(PHASE_HOLD, &store_path, &ready_path, &release_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("holding child must start");
    wait_for_marker(&ready_path);
    let identity_before = file_identity(&store_path);

    let inspect = run_show(&config_path);
    let repair = run_repair(&config_path);
    assert_fixed_error(&inspect, "owned by another process", &store_path);
    assert_fixed_error(&repair, "owned by another process", &store_path);
    assert_eq!(file_identity(&store_path), identity_before);

    std::fs::write(&release_path, "release").expect("release marker must persist");
    assert!(child.wait().expect("holding child must exit").success());
    let inspected = run_show(&config_path);
    assert!(inspected.status.success(), "stderr: {}", String::from_utf8_lossy(&inspected.stderr));
}

// r[verify lattice.workflow_execution.run_store_recovery.scenario.refusal]
#[test]
fn repair_refuses_missing_corrupt_and_invalid_phase_without_creating_state() {
    let missing_directory = tempfile::tempdir().expect("missing fixture directory must exist");
    let missing_data = missing_directory.path().join("missing-data");
    let missing_config = missing_directory.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &missing_config,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            missing_data.display().to_string()
        ),
    )
    .expect("missing fixture config must persist");
    let missing = run_repair(&missing_config);
    assert_fixed_error(&missing, "store is unavailable", &missing_data);
    assert!(!missing_data.exists());

    let (_corrupt_directory, corrupt_config, corrupt_store) = fixture();
    std::fs::write(&corrupt_store, PRIVATE_CORRUPTION).expect("corrupt fixture must persist");
    let corrupt_identity = file_identity(&corrupt_store);
    let corrupt = run_repair(&corrupt_config);
    assert_fixed_error(&corrupt, "store is unavailable", &corrupt_store);
    assert_eq!(file_identity(&corrupt_store), corrupt_identity);

    let (_invalid_directory, _invalid_config, invalid_store) = fixture();
    let invalid_ready = invalid_store.with_extension("ready");
    let invalid_release = invalid_store.with_extension("release");
    let invalid = invoke_child(PHASE_INVALID, &invalid_store, &invalid_ready, &invalid_release);
    assert!(!invalid.status.success());
    assert!(!invalid_store.exists());
    assert!(!invalid_ready.exists());
    assert!(!invalid_release.exists());
}

// r[verify lattice.workflow_execution.safe_run_inspection]
#[test]
fn clean_writer_exit_remains_immediately_inspectable() {
    let (_directory, config_path, store_path) = fixture();
    let ready_path = store_path.with_extension("ready");
    let release_path = store_path.with_extension("release");
    let child = invoke_child(PHASE_CLEAN, &store_path, &ready_path, &release_path);
    assert!(child.status.success(), "stderr: {}", String::from_utf8_lossy(&child.stderr));
    let inspected = run_show(&config_path);
    assert!(inspected.status.success(), "stderr: {}", String::from_utf8_lossy(&inspected.stderr));
    let report: Report = serde_json::from_slice(&inspected.stdout).expect("safe report must decode");
    assert!(report.valid);
    let repair = run_repair(&config_path);
    assert!(repair.status.success(), "stderr: {}", String::from_utf8_lossy(&repair.stderr));
    assert_eq!(String::from_utf8_lossy(&repair.stdout), ALREADY_CLEAN);
    assert!(repair.stderr.is_empty());
    let inspected_again = run_show(&config_path);
    assert!(inspected_again.status.success(), "stderr: {}", String::from_utf8_lossy(&inspected_again.stderr));
}
