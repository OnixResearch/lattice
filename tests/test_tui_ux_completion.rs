mod common;

use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use lattice::config::AppConfig;
use lattice::engine::RunId;
use lattice::engine::Workflow;
use lattice::engine::workflow::NodeConfig;
use lattice::store::RunNodeDetailStatus;
use lattice::store::Store;
use lattice::tui::App;
use lattice::tui::app::RunOutputLine;
use lattice::tui::app::palette_action;
use lattice::tui::input::handle_event;
use lattice::tui::panels;
use portable_pty::CommandBuilder;
use portable_pty::PtySize;
use portable_pty::native_pty_system;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tempfile::TempDir;

const FRAME_WIDTH: u16 = 120;
const FRAME_HEIGHT: u16 = 40;
const ACTIVE_REPLAY_POLL_ATTEMPTS: usize = 50;
const ACTIVE_REPLAY_POLL_DELAY_MS: u64 = 10;
const LONG_RUNNING_COMMAND: &str = "sleep 1";
const IMAGE_SUMMARY: &str = "image/png image 1x1 hash abc123";
const RAW_IMAGE_BYTES: &str = "raw-image-bytes-must-not-render";
const IMAGE_OUTPUT_LABEL: &str = "image";
const IMAGE_PATH: &str = "/tmp/tui-preview.png";
const CLI_CONFIG_FILE_NAME: &str = "config.ncl";
const CLI_DEFAULT_TIMEOUT_SECS: u64 = 60;
const CLI_MAX_PARALLELISM: usize = 1;
const PTY_ROWS: u16 = 40;
const PTY_COLUMNS: u16 = 120;
const PTY_PIXEL_WIDTH: u16 = 0;
const PTY_PIXEL_HEIGHT: u16 = 0;
const PTY_READ_BUFFER_BYTES: usize = 4096;
const PTY_TRANSCRIPT_MAX_BYTES: usize = 8_388_608;
const PTY_POLL_ATTEMPTS: usize = 250;
const PTY_POLL_DELAY_MS: u64 = 20;
const TUI_QUIT_INPUT: &[u8] = b"\x11";
const TUI_OPEN_WORKFLOW_INPUT: &[u8] = b"\x0f";
const TUI_RUN_WORKFLOW_INPUT: &[u8] = b"\x12";
const TUI_OPEN_DIAGNOSTICS_INPUT: &[u8] = b"\x04";
const TUI_OPEN_HISTORY_INPUT: &[u8] = b"\x10history\r";
const TUI_CONFIRM_INPUT: &[u8] = b"\r";
const TUI_ESCAPE_INPUT: &[u8] = b"\x1b";
const TUI_RERUN_INPUT: &[u8] = b"r";
const TUI_WORKSPACE_SURFACES: &[&str] = &["Canvas", "Properties", "Output", "untitled"];
const TUI_WORKFLOW_BROWSER_SURFACES: &[&str] = &["Open Workflow", TUI_WORKFLOW_NAME];
const TUI_DENIED_RUN_SURFACES: &[&str] = &[TUI_NODE_LABEL, "AuthorityUnavailable", "run", "complete"];
const TUI_HISTORY_SURFACES: &[&str] = &["Run History", TUI_WORKFLOW_NAME];
const TUI_WORKFLOW_NAME: &str = "operator-tui-workflow";
const TUI_NODE_LABEL: &str = "Operator Shell";
const TUI_SHELL_COMMAND: &str = "printf tui-process-e2e";
const STORE_FILE_NAME: &str = "lattice.redb";
const TUI_RUN_HISTORY_LIMIT: usize = 10;
const EXPECTED_TUI_RUN_COUNT: usize = 1;
const EXPECTED_RERUN_COUNT: usize = 2;
const FAIL_ONCE_WORKFLOW_NAME: &str = "operator-tui-fail-once";
const FAIL_ONCE_NODE_LABEL: &str = "Fail Once Node";
const FAIL_ONCE_MARKER_FILE_NAME: &str = "tui-fail-once-marker";
const FAIL_ONCE_FAILURE_TOKEN: &str = "tui-first-attempt-timed-out";
const FAIL_ONCE_SUCCESS_TOKEN: &str = "tui-explicit-rerun-succeeded";
const FAIL_ONCE_TIMEOUT_SECS: u64 = 1;
const FAIL_ONCE_SLEEP_SECS: u64 = 2;
const INVALID_WORKFLOW_NAME: &str = "operator-tui-invalid";
const INVALID_NODE_LABEL: &str = "Invalid Empty Shell";
const INVALID_EFFECT_NODE_LABEL: &str = "Forbidden Validation Effect";
const INVALID_EFFECT_FILE_NAME: &str = "tui-invalid-forbidden-effect";
const INVALID_EFFECT_TOKEN: &str = "invalid-workflow-effect";
const CANCEL_WORKFLOW_NAME: &str = "operator-tui-cancel";
const CANCEL_NODE_LABEL: &str = "Cancellable Shell";
const CANCEL_READY_FILE_NAME: &str = "tui-cancel-ready";
const CANCEL_FORBIDDEN_FILE_NAME: &str = "tui-cancel-forbidden";
const CANCEL_FORBIDDEN_TOKEN: &str = "cancel-forbidden-effect";
const CANCEL_EFFECT_DELAY_SECS: u64 = 2;
const RESIZE_WORKFLOW_NAME: &str = "operator-tui-resize";
const RESIZE_NODE_LABEL: &str = "Resize Preserved Node";
const PTY_CONSTRAINED_ROWS: u16 = 6;
const PTY_CONSTRAINED_COLUMNS: u16 = 32;

fn test_config(temp_dir: &TempDir) -> AppConfig {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("create app data directory");
    AppConfig {
        data_dir,
        ..AppConfig::default()
    }
}

fn rendered_frame_text(app: &mut App) -> String {
    let area = Rect::new(0, 0, FRAME_WIDTH, FRAME_HEIGHT);
    let mut buffer = Buffer::empty(area);
    panels::render(app, area, &mut buffer);
    buffer.content().iter().map(|cell| cell.symbol()).collect()
}

fn write_cli_config(temp_dir: &TempDir) -> std::path::PathBuf {
    let config_path = temp_dir.path().join(CLI_CONFIG_FILE_NAME);
    let data_dir = temp_dir.path().join("data");
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {CLI_DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {CLI_MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string(),
        ),
    )
    .expect("operator E2E config writes");
    config_path
}

fn transcript_text(transcript: &Arc<Mutex<Vec<u8>>>) -> String {
    let bytes = transcript.lock().expect("PTY transcript lock remains available");
    String::from_utf8_lossy(&bytes).into_owned()
}

fn transcript_has_surfaces(transcript: &str, surfaces: &[&str]) -> bool {
    surfaces.iter().all(|surface| transcript.contains(surface))
}

fn transcript_has_workspace(transcript: &str) -> bool {
    transcript_has_surfaces(transcript, TUI_WORKSPACE_SURFACES)
}

fn wait_for_surfaces(transcript: &Arc<Mutex<Vec<u8>>>, surfaces: &[&str], phase: &str) -> String {
    assert!(!surfaces.is_empty(), "PTY surface wait must name observable output");
    for _ in 0..PTY_POLL_ATTEMPTS {
        let text = transcript_text(transcript);
        if transcript_has_surfaces(&text, surfaces) {
            return text;
        }
        thread::sleep(Duration::from_millis(PTY_POLL_DELAY_MS));
    }
    panic!("TUI did not render {phase} within the bounded PTY window: {}", transcript_text(transcript));
}

fn wait_for_workspace(transcript: &Arc<Mutex<Vec<u8>>>) -> String {
    wait_for_surfaces(transcript, TUI_WORKSPACE_SURFACES, "the workspace")
}

fn write_pty_input(writer: &mut dyn Write, input: &[u8], description: &str) {
    assert!(!input.is_empty(), "PTY input must contain an action");
    writer.write_all(input).unwrap_or_else(|error| panic!("{description} input writes to PTY: {error}"));
    writer.flush().unwrap_or_else(|error| panic!("{description} input flushes to PTY: {error}"));
}

fn save_tui_workflow(temp_dir: &TempDir, workflow: &Workflow) {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("operator TUI data directory creates");
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("operator TUI store opens");
    store.save_workflow(workflow).expect("operator TUI workflow saves");
}

fn save_operator_tui_workflow(temp_dir: &TempDir) {
    let mut workflow = Workflow::new(TUI_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(TUI_NODE_LABEL, TUI_SHELL_COMMAND);
    save_tui_workflow(temp_dir, &workflow);
}

#[cfg(unix)]
fn save_fail_once_workflow(temp_dir: &TempDir) {
    let marker_path = temp_dir.path().join(FAIL_ONCE_MARKER_FILE_NAME);
    let mut workflow = Workflow::new(FAIL_ONCE_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_shell_node(
        FAIL_ONCE_NODE_LABEL,
        common::timeout_once_then_succeed_command(
            &marker_path,
            FAIL_ONCE_SLEEP_SECS,
            FAIL_ONCE_FAILURE_TOKEN,
            FAIL_ONCE_SUCCESS_TOKEN,
        ),
    );
    workflow.set_node_config(node_id, NodeConfig::Shell {
        command: common::timeout_once_then_succeed_command(
            &marker_path,
            FAIL_ONCE_SLEEP_SECS,
            FAIL_ONCE_FAILURE_TOKEN,
            FAIL_ONCE_SUCCESS_TOKEN,
        ),
        cwd: None,
        env: std::collections::HashMap::new(),
        timeout_secs: FAIL_ONCE_TIMEOUT_SECS,
        pipe_stdin: true,
    });
    save_tui_workflow(temp_dir, &workflow);
}

#[cfg(unix)]
fn save_invalid_workflow(temp_dir: &TempDir) -> std::path::PathBuf {
    let effect_path = temp_dir.path().join(INVALID_EFFECT_FILE_NAME);
    let mut workflow = Workflow::new(INVALID_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(INVALID_NODE_LABEL, "");
    workflow.add_shell_node(
        INVALID_EFFECT_NODE_LABEL,
        common::identified_write_command(&effect_path, INVALID_EFFECT_TOKEN),
    );
    save_tui_workflow(temp_dir, &workflow);
    effect_path
}

#[cfg(unix)]
fn save_cancellable_workflow(temp_dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let ready_path = temp_dir.path().join(CANCEL_READY_FILE_NAME);
    let forbidden_path = temp_dir.path().join(CANCEL_FORBIDDEN_FILE_NAME);
    let mut workflow = Workflow::new(CANCEL_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(
        CANCEL_NODE_LABEL,
        common::ready_then_delayed_effect_command(
            &ready_path,
            &forbidden_path,
            CANCEL_EFFECT_DELAY_SECS,
            CANCEL_FORBIDDEN_TOKEN,
        ),
    );
    save_tui_workflow(temp_dir, &workflow);
    (ready_path, forbidden_path)
}

#[cfg(unix)]
fn save_resize_workflow(temp_dir: &TempDir) {
    let mut workflow = Workflow::new(RESIZE_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(RESIZE_NODE_LABEL, TUI_SHELL_COMMAND);
    save_tui_workflow(temp_dir, &workflow);
}

fn wait_for_pty_exit(child: &mut dyn portable_pty::Child) -> portable_pty::ExitStatus {
    for _ in 0..PTY_POLL_ATTEMPTS {
        if let Some(status) = child.try_wait().expect("PTY child status remains observable") {
            return status;
        }
        thread::sleep(Duration::from_millis(PTY_POLL_DELAY_MS));
    }
    child.kill().expect("timed-out PTY child terminates");
    let status = child.wait().expect("terminated PTY child is reaped");
    panic!("TUI did not exit within the bounded PTY shutdown window: {status}");
}

fn wait_for_process_exit(child: &mut std::process::Child) -> std::process::ExitStatus {
    for _ in 0..PTY_POLL_ATTEMPTS {
        if let Some(status) = child.try_wait().expect("process child status remains observable") {
            return status;
        }
        thread::sleep(Duration::from_millis(PTY_POLL_DELAY_MS));
    }
    child.kill().expect("timed-out process child terminates");
    let status = child.wait().expect("terminated process child is reaped");
    panic!("process did not exit within the bounded shutdown window: {status}");
}

fn bounded_transcript_read_len(current_bytes: usize, incoming_bytes: usize) -> usize {
    PTY_TRANSCRIPT_MAX_BYTES.saturating_sub(current_bytes).min(incoming_bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FreshSurfaceError {
    EmptyExpectation,
    CursorOutOfBounds,
    MissingFreshSurface,
}

fn classify_fresh_surfaces(transcript: &[u8], cursor: usize, surfaces: &[&str]) -> Result<String, FreshSurfaceError> {
    if surfaces.is_empty() {
        return Err(FreshSurfaceError::EmptyExpectation);
    }
    if cursor > transcript.len() {
        return Err(FreshSurfaceError::CursorOutOfBounds);
    }
    let fresh = String::from_utf8_lossy(&transcript[cursor..]).into_owned();
    if !transcript_has_surfaces(&fresh, surfaces) {
        return Err(FreshSurfaceError::MissingFreshSurface);
    }
    Ok(fresh)
}

fn wait_for_fresh_surfaces(transcript: &Arc<Mutex<Vec<u8>>>, cursor: usize, surfaces: &[&str], phase: &str) -> String {
    for _ in 0..PTY_POLL_ATTEMPTS {
        let bytes = transcript.lock().expect("PTY transcript lock remains available");
        if let Ok(fresh) = classify_fresh_surfaces(&bytes, cursor, surfaces) {
            return fresh;
        }
        drop(bytes);
        thread::sleep(Duration::from_millis(PTY_POLL_DELAY_MS));
    }
    panic!(
        "TUI did not render fresh {phase} surfaces within the bounded PTY window: {}",
        transcript_text(transcript)
    );
}

fn pty_size(rows: u16, columns: u16) -> PtySize {
    PtySize {
        rows,
        cols: columns,
        pixel_width: PTY_PIXEL_WIDTH,
        pixel_height: PTY_PIXEL_HEIGHT,
    }
}

struct TuiPtySession {
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    transcript: Arc<Mutex<Vec<u8>>>,
    reader_thread: Option<thread::JoinHandle<()>>,
}

impl TuiPtySession {
    fn start(config_path: &std::path::Path, working_directory: &std::path::Path) -> Self {
        let pair = native_pty_system().openpty(pty_size(PTY_ROWS, PTY_COLUMNS)).expect("operator E2E PTY opens");
        let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_lattice"));
        command.arg("--config");
        command.arg(config_path.as_os_str());
        command.cwd(working_directory);
        let child = pair.slave.spawn_command(command).expect("Lattice TUI starts inside the PTY");
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().expect("PTY reader clones");
        let writer = pair.master.take_writer().expect("PTY writer opens");
        let transcript = Arc::new(Mutex::new(Vec::new()));
        let reader_transcript = Arc::clone(&transcript);
        let reader_thread = thread::spawn(move || {
            let mut buffer = [0_u8; PTY_READ_BUFFER_BYTES];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => return,
                    Ok(read_bytes) => {
                        let mut transcript = reader_transcript.lock().expect("PTY transcript lock remains available");
                        let retained_bytes = bounded_transcript_read_len(transcript.len(), read_bytes);
                        transcript.extend_from_slice(&buffer[..retained_bytes]);
                        if retained_bytes < read_bytes {
                            return;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(_) => return,
                }
            }
        });
        Self {
            master: Some(pair.master),
            writer: Some(writer),
            child: Some(child),
            transcript,
            reader_thread: Some(reader_thread),
        }
    }

    fn cursor(&self) -> usize {
        self.transcript.lock().expect("PTY transcript lock remains available").len()
    }

    fn input(&mut self, input: &[u8], description: &str) {
        write_pty_input(self.writer.as_mut().expect("PTY writer remains owned").as_mut(), input, description);
    }

    fn wait(&self, surfaces: &[&str], phase: &str) -> String {
        wait_for_surfaces(&self.transcript, surfaces, phase)
    }

    fn wait_fresh(&self, cursor: usize, surfaces: &[&str], phase: &str) -> String {
        wait_for_fresh_surfaces(&self.transcript, cursor, surfaces, phase)
    }

    fn resize(&self, rows: u16, columns: u16) {
        let expected = pty_size(rows, columns);
        let master = self.master.as_ref().expect("PTY master remains owned");
        master.resize(expected).expect("PTY resize succeeds");
        let observed = master.get_size().expect("PTY resized dimensions remain observable");
        assert_eq!(observed.rows, expected.rows);
        assert_eq!(observed.cols, expected.cols);
    }

    fn finish(mut self) -> (portable_pty::ExitStatus, String) {
        let status = wait_for_pty_exit(self.child.as_mut().expect("PTY child remains owned").as_mut());
        self.child = None;
        self.writer = None;
        self.master = None;
        self.reader_thread
            .take()
            .expect("PTY reader thread remains owned")
            .join()
            .expect("PTY reader joins");
        let transcript = transcript_text(&self.transcript);
        (status, transcript)
    }
}

impl Drop for TuiPtySession {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.writer = None;
        self.master = None;
        if let Some(reader_thread) = self.reader_thread.take() {
            let _ = reader_thread.join();
        }
    }
}

#[test]
fn fresh_surface_classifier_accepts_new_tokens_and_rejects_stale_or_invalid_phases() {
    let old = b"old failure surface";
    let mut transcript = old.to_vec();
    transcript.extend_from_slice(b" new explicit rerun success");

    assert!(classify_fresh_surfaces(&transcript, old.len(), &["explicit rerun", "success"]).is_ok());
    assert_eq!(
        classify_fresh_surfaces(&transcript, old.len(), &["old failure"]),
        Err(FreshSurfaceError::MissingFreshSurface),
    );
    assert_eq!(
        classify_fresh_surfaces(&transcript, transcript.len().saturating_add(1), &["success"]),
        Err(FreshSurfaceError::CursorOutOfBounds),
    );
    assert_eq!(classify_fresh_surfaces(&transcript, old.len(), &[]), Err(FreshSurfaceError::EmptyExpectation),);
}

#[test]
fn operator_e2e_tui_process_navigates_executes_inspects_history_and_exits() {
    let temp_dir = tempfile::tempdir().expect("temporary operator E2E directory");
    let config_path = write_cli_config(&temp_dir);
    save_operator_tui_workflow(&temp_dir);
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: PTY_ROWS,
            cols: PTY_COLUMNS,
            pixel_width: PTY_PIXEL_WIDTH,
            pixel_height: PTY_PIXEL_HEIGHT,
        })
        .expect("operator E2E PTY opens");
    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_lattice"));
    command.arg("--config");
    command.arg(config_path.as_os_str());
    command.cwd(temp_dir.path());
    let mut child = pair.slave.spawn_command(command).expect("Lattice TUI starts inside the PTY");
    drop(pair.slave);
    let mut reader = pair.master.try_clone_reader().expect("PTY reader clones");
    let mut writer = pair.master.take_writer().expect("PTY writer opens");
    let transcript = Arc::new(Mutex::new(Vec::new()));
    let reader_transcript = Arc::clone(&transcript);
    let reader_thread = thread::spawn(move || {
        let mut buffer = [0_u8; PTY_READ_BUFFER_BYTES];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => return,
                Ok(read_bytes) => {
                    let mut transcript = reader_transcript.lock().expect("PTY transcript lock remains available");
                    let retained_bytes = bounded_transcript_read_len(transcript.len(), read_bytes);
                    transcript.extend_from_slice(&buffer[..retained_bytes]);
                    if retained_bytes < read_bytes {
                        return;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => return,
            }
        }
    });

    let rendered = wait_for_workspace(&transcript);
    write_pty_input(writer.as_mut(), TUI_OPEN_WORKFLOW_INPUT, "open workflow");
    let browser = wait_for_surfaces(&transcript, TUI_WORKFLOW_BROWSER_SURFACES, "the workflow browser");
    write_pty_input(writer.as_mut(), TUI_CONFIRM_INPUT, "select workflow");
    let opened = wait_for_surfaces(&transcript, &[TUI_NODE_LABEL], "the selected workflow");
    write_pty_input(writer.as_mut(), TUI_RUN_WORKFLOW_INPUT, "run workflow");
    let completed = wait_for_surfaces(&transcript, TUI_DENIED_RUN_SURFACES, "the authority-denied workflow run");
    write_pty_input(writer.as_mut(), TUI_OPEN_HISTORY_INPUT, "open run history");
    let history = wait_for_surfaces(&transcript, TUI_HISTORY_SURFACES, "run history");
    write_pty_input(writer.as_mut(), TUI_CONFIRM_INPUT, "inspect run history");
    let details = wait_for_surfaces(&transcript, &["Outputs for", TUI_NODE_LABEL, "failed"], "run details");
    write_pty_input(writer.as_mut(), TUI_QUIT_INPUT, "quit");
    let status = wait_for_pty_exit(child.as_mut());
    drop(writer);
    drop(pair.master);
    reader_thread.join().expect("PTY transcript reader joins");

    assert!(status.success(), "TUI exit status: {status}; transcript: {details}");
    assert!(transcript_has_workspace(&rendered));
    assert!(transcript_has_surfaces(&browser, TUI_WORKFLOW_BROWSER_SURFACES));
    assert!(opened.contains(TUI_NODE_LABEL));
    assert!(transcript_has_surfaces(&completed, TUI_DENIED_RUN_SURFACES));
    assert!(transcript_has_surfaces(&history, TUI_HISTORY_SURFACES));
    assert!(details.contains(&format!("Outputs for {TUI_NODE_LABEL}")));

    let store = Store::new(temp_dir.path().join("data").join(STORE_FILE_NAME)).expect("operator TUI store reopens");
    let runs = store.list_runs(TUI_RUN_HISTORY_LIMIT).expect("operator TUI run history lists");
    assert_eq!(runs.len(), EXPECTED_TUI_RUN_COUNT);
    assert_eq!(runs[0].workflow_name, TUI_WORKFLOW_NAME);
    assert_eq!(runs[0].failures, EXPECTED_TUI_RUN_COUNT);
    let details = store
        .load_run_details(runs[0].run_id)
        .expect("operator TUI run details load")
        .expect("operator TUI run details persist");
    assert_eq!(details.nodes[0].status, RunNodeDetailStatus::Failed);
}

// r[verify lattice.tui_ux.process_failure_recovery]
// r[verify lattice.tui_ux.process_failure_recovery.scenario.rerun]
#[cfg(unix)]
#[test]
fn operator_e2e_tui_process_inspects_failure_and_explicitly_reruns_fresh_identity() {
    let temp_dir = tempfile::tempdir().expect("temporary failure-recovery PTY directory");
    let config_path = write_cli_config(&temp_dir);
    save_fail_once_workflow(&temp_dir);
    let mut session = TuiPtySession::start(&config_path, temp_dir.path());

    session.wait(TUI_WORKSPACE_SURFACES, "the initial failure-recovery workspace");
    let cursor = session.cursor();
    session.input(TUI_OPEN_WORKFLOW_INPUT, "open fail-once workflow");
    session.wait_fresh(cursor, &["Open Workflow", FAIL_ONCE_WORKFLOW_NAME], "the fail-once browser");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "select fail-once workflow");
    session.wait_fresh(cursor, &[FAIL_ONCE_NODE_LABEL], "the selected fail-once workflow");
    let cursor = session.cursor();
    session.input(TUI_RUN_WORKFLOW_INPUT, "run fail-once workflow");
    session.wait_fresh(cursor, &[FAIL_ONCE_NODE_LABEL, "AuthorityUnavailable"], "the denied first attempt");

    let cursor = session.cursor();
    session.input(TUI_OPEN_HISTORY_INPUT, "open failed run history");
    session.wait_fresh(cursor, &["Run History", "run:0"], "failed run history");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "inspect failed run");
    session.wait_fresh(cursor, &["Outputs for", FAIL_ONCE_NODE_LABEL, "failed"], "failed run details");
    let cursor = session.cursor();
    session.input(TUI_RERUN_INPUT, "explicitly rerun failed history entry");
    session.input(TUI_ESCAPE_INPUT, "close history after explicit rerun");
    session.wait_fresh(
        cursor,
        &[FAIL_ONCE_NODE_LABEL, "AuthorityUnavailable", "run:1 complete"],
        "the denied explicit rerun",
    );
    let cursor = session.cursor();
    session.input(TUI_OPEN_HISTORY_INPUT, "reopen rerun history");
    session.wait_fresh(cursor, &["Run History", "run:1"], "rerun history");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "inspect denied rerun");
    session.wait_fresh(cursor, &["Outputs for", FAIL_ONCE_NODE_LABEL, "failed"], "denied rerun details");
    session.input(TUI_QUIT_INPUT, "quit after explicit rerun");
    let (status, transcript) = session.finish();

    assert!(status.success(), "TUI exit status: {status}; transcript: {transcript}");
    assert!(
        !temp_dir.path().join(FAIL_ONCE_MARKER_FILE_NAME).exists(),
        "authority denial must precede the fail-once shell marker"
    );
    let store = Store::new(temp_dir.path().join("data").join(STORE_FILE_NAME)).expect("failure-recovery store opens");
    let runs = store.list_runs(TUI_RUN_HISTORY_LIMIT).expect("failure-recovery runs list");
    assert_eq!(runs.len(), EXPECTED_RERUN_COUNT);
    assert_ne!(runs[0].run_id, runs[1].run_id);
    assert!(runs.iter().all(|run| run.failures == EXPECTED_TUI_RUN_COUNT));
    for run in runs {
        let details = store.load_run_details(run.run_id).expect("denied details load").expect("denied details persist");
        assert_eq!(details.nodes[0].status, RunNodeDetailStatus::Failed);
    }
}

// r[verify lattice.tui_ux.process_failure_recovery.scenario.diagnostics]
#[cfg(unix)]
#[test]
fn operator_e2e_tui_process_rejects_invalid_workflow_before_effect_and_remains_navigable() {
    let temp_dir = tempfile::tempdir().expect("temporary validation-rejection PTY directory");
    let config_path = write_cli_config(&temp_dir);
    let forbidden_effect_path = save_invalid_workflow(&temp_dir);
    let mut session = TuiPtySession::start(&config_path, temp_dir.path());

    session.wait(TUI_WORKSPACE_SURFACES, "the initial validation workspace");
    let cursor = session.cursor();
    session.input(TUI_OPEN_WORKFLOW_INPUT, "open invalid workflow");
    session.wait_fresh(cursor, &["Open Workflow", INVALID_WORKFLOW_NAME], "the invalid workflow browser");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "select invalid workflow");
    session.wait_fresh(cursor, &[INVALID_EFFECT_NODE_LABEL], "the selected invalid workflow");
    let cursor = session.cursor();
    session.input(TUI_RUN_WORKFLOW_INPUT, "request invalid workflow run");
    session.wait_fresh(cursor, &["Workflow Diagnostics", "empty command", "fatal"], "fatal workflow diagnostics");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "close fatal diagnostics");
    session.input(TUI_OPEN_WORKFLOW_INPUT, "open browser after diagnostics");
    session.wait_fresh(cursor, &["Open Workflow", INVALID_WORKFLOW_NAME], "navigation after diagnostics");
    let cursor = session.cursor();
    session.input(TUI_ESCAPE_INPUT, "close browser after diagnostics");
    session.wait_fresh(cursor, &["C-D: diagnostics"], "workspace after browser close");
    session.input(TUI_QUIT_INPUT, "quit after validation rejection");
    let (status, transcript) = session.finish();

    assert!(status.success(), "TUI exit status: {status}; transcript: {transcript}");
    assert!(!forbidden_effect_path.exists(), "invalid workflow must not perform its potential effect");
    let store = Store::new(temp_dir.path().join("data").join(STORE_FILE_NAME)).expect("validation store opens");
    assert!(store.list_runs(TUI_RUN_HISTORY_LIMIT).expect("validation runs list").is_empty());
}

// r[verify lattice.tui_ux.process_failure_recovery.scenario.cancel]
#[cfg(unix)]
#[test]
fn operator_e2e_tui_process_denies_cancellable_effect_without_current_authority() {
    let temp_dir = tempfile::tempdir().expect("temporary cancellation PTY directory");
    let config_path = write_cli_config(&temp_dir);
    let (ready_path, forbidden_effect_path) = save_cancellable_workflow(&temp_dir);
    let mut session = TuiPtySession::start(&config_path, temp_dir.path());

    session.wait(TUI_WORKSPACE_SURFACES, "the initial cancellation workspace");
    let cursor = session.cursor();
    session.input(TUI_OPEN_WORKFLOW_INPUT, "open cancellable workflow");
    session.wait_fresh(cursor, &["Open Workflow", CANCEL_WORKFLOW_NAME], "the cancellation browser");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "select cancellable workflow");
    session.wait_fresh(cursor, &[CANCEL_NODE_LABEL], "the selected cancellable workflow");
    let cursor = session.cursor();
    session.input(TUI_RUN_WORKFLOW_INPUT, "run cancellable workflow");
    session.wait_fresh(cursor, &[CANCEL_NODE_LABEL, "AuthorityUnavailable", "run:0 complete"], "the denied outcome");
    assert!(!ready_path.exists(), "authority denial must precede the cancellation gate");
    assert!(!forbidden_effect_path.exists(), "denied child must not perform its pending effect");

    let cursor = session.cursor();
    session.input(TUI_OPEN_HISTORY_INPUT, "open denied run history");
    session.wait_fresh(cursor, &["Run History", "run:0"], "denied run history");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "inspect denied run");
    session.wait_fresh(cursor, &["Outputs for", CANCEL_NODE_LABEL, "failed"], "denied run details");
    session.input(TUI_QUIT_INPUT, "quit after denial");
    let (status, transcript) = session.finish();

    assert!(status.success(), "TUI exit status: {status}; transcript: {transcript}");
    assert!(!forbidden_effect_path.exists(), "pending effect must remain absent after TUI exit");
    let store = Store::new(temp_dir.path().join("data").join(STORE_FILE_NAME)).expect("denial store opens");
    let runs = store.list_runs(TUI_RUN_HISTORY_LIMIT).expect("denial runs list");
    assert_eq!(runs.len(), EXPECTED_TUI_RUN_COUNT);
    assert_eq!(runs[0].failures, EXPECTED_TUI_RUN_COUNT);
    let details = store
        .load_run_details(runs[0].run_id)
        .expect("denial details load")
        .expect("denial details persist");
    assert_eq!(details.nodes[0].status, RunNodeDetailStatus::Failed);
}

// r[verify lattice.tui_ux.process_failure_recovery.scenario.resize]
// r[verify lattice.tui_ux.process_failure_recovery.scenario.non-claim]
#[cfg(unix)]
#[test]
fn operator_e2e_tui_process_resize_preserves_modal_workflow_and_keyboard_input() {
    let temp_dir = tempfile::tempdir().expect("temporary resize PTY directory");
    let config_path = write_cli_config(&temp_dir);
    save_resize_workflow(&temp_dir);
    let mut session = TuiPtySession::start(&config_path, temp_dir.path());

    session.wait(TUI_WORKSPACE_SURFACES, "the initial resize workspace");
    let cursor = session.cursor();
    session.input(TUI_OPEN_WORKFLOW_INPUT, "open resize workflow");
    session.wait_fresh(cursor, &["Open Workflow", RESIZE_WORKFLOW_NAME], "the resize workflow browser");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "select resize workflow");
    session.wait_fresh(cursor, &[RESIZE_NODE_LABEL], "the selected resize workflow");
    let cursor = session.cursor();
    session.input(TUI_OPEN_DIAGNOSTICS_INPUT, "open resize diagnostics modal");
    session.wait_fresh(cursor, &["Workflow Diagnostics", "Workflow ready to run"], "the resize diagnostics modal");

    let cursor = session.cursor();
    session.resize(PTY_CONSTRAINED_ROWS, PTY_CONSTRAINED_COLUMNS);
    session.wait_fresh(cursor, &["Workflow Diagnostics"], "the constrained diagnostics modal");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "close constrained diagnostics modal");
    session.input(TUI_OPEN_DIAGNOSTICS_INPUT, "reopen constrained diagnostics modal");
    session.wait_fresh(cursor, &["Workflow Diagnostics"], "the reopened constrained modal");
    let cursor = session.cursor();
    session.resize(PTY_ROWS, PTY_COLUMNS);
    session.wait_fresh(cursor, &["Workflow Diagnostics", "Workflow ready to run"], "the restored diagnostics modal");
    let cursor = session.cursor();
    session.input(TUI_CONFIRM_INPUT, "close restored diagnostics modal");
    session.input(TUI_OPEN_WORKFLOW_INPUT, "open browser after resize");
    session.wait_fresh(cursor, &["Open Workflow", RESIZE_WORKFLOW_NAME], "navigation after resize");
    let cursor = session.cursor();
    session.input(TUI_ESCAPE_INPUT, "close browser after resize");
    session.wait_fresh(cursor, &["C-D: diagnostics"], "workspace after resize browser close");
    session.input(TUI_QUIT_INPUT, "quit after resize");
    let (status, transcript) = session.finish();

    assert!(status.success(), "TUI exit status: {status}; transcript: {transcript}");
    let store = Store::new(temp_dir.path().join("data").join(STORE_FILE_NAME)).expect("resize store opens");
    assert_eq!(
        store
            .load_workflow(RESIZE_WORKFLOW_NAME)
            .expect("resize workflow loads")
            .expect("resize workflow persists")
            .meta
            .name,
        RESIZE_WORKFLOW_NAME,
    );
    assert!(store.list_runs(TUI_RUN_HISTORY_LIMIT).expect("resize runs list").is_empty());
}

#[test]
fn operator_e2e_tui_process_rejects_non_terminal_stdio_without_hanging() {
    let temp_dir = tempfile::tempdir().expect("temporary operator E2E directory");
    let config_path = write_cli_config(&temp_dir);

    let mut child = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("non-terminal TUI process starts");
    let status = wait_for_process_exit(&mut child);
    let mut stderr = Vec::new();
    child
        .stderr
        .take()
        .expect("non-terminal TUI stderr is piped")
        .read_to_end(&mut stderr)
        .expect("non-terminal TUI stderr reads");

    assert!(!status.success());
    assert!(!stderr.is_empty());
}

// r[verify n8nclone.tui_ux.keyboard_first]
// r[verify n8nclone.tui_ux.workspace]
// r[verify n8nclone.tui_ux.workspace.starts]
#[test]
fn workspace_render_exposes_all_panels_and_keyboard_authoring_actions() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));

    let rendered = rendered_frame_text(&mut app);

    for surface in ["Canvas", "Properties", "Output", "untitled"] {
        assert!(rendered.contains(surface), "missing {surface}: {rendered}");
    }
    assert!(rendered.contains("Ctrl-P"), "status help missing: {rendered}");
    assert!(app.palette_data.items.iter().all(|candidate| palette_action(&candidate.id).is_some()));

    handle_event(&mut app, &Event::Key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)));
    assert!(app.show_palette);
    let palette = rendered_frame_text(&mut app);
    assert!(palette.contains("Command Palette"), "palette missing: {palette}");
    assert!(palette.contains("Add Shell Node"), "authoring action missing: {palette}");
}

// r[verify n8nclone.tui_ux.background_runs.long_running]
// r[verify n8nclone.tui_ux.background_runs.responsive]
// r[verify n8nclone.tui_ux.replay_debugger.active_run.stream]
#[tokio::test(flavor = "multi_thread")]
async fn active_run_keeps_rendering_and_input_live_while_replay_events_stream() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir))
        .with_authority_provider(Arc::new(common::authority::TestRunContextProvider::new()));
    app.add_shell_node("Long", LONG_RUNNING_COMMAND);

    assert!(app.start_workflow_run());
    for _ in 0..ACTIVE_REPLAY_POLL_ATTEMPTS {
        app.poll_run_events();
        if app.run_state.active_replay_timeline.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(ACTIVE_REPLAY_POLL_DELAY_MS)).await;
    }

    assert!(app.run_state.active, "authorized active run ended early: {:?}", app.run_state.node_statuses);
    let timeline = app.run_state.active_replay_timeline.as_ref().expect("active replay timeline");
    assert!(!timeline.frames.is_empty());
    assert!(timeline.diagnostics.iter().all(|diagnostic| !diagnostic.fatal));

    handle_event(&mut app, &Event::Key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)));
    assert!(app.show_palette);
    let rendered = rendered_frame_text(&mut app);
    assert!(rendered.contains("Command Palette"), "active render missing: {rendered}");
    assert!(app.cancel_active_run());
}

// r[verify n8nclone.tui_ux.image_nodes.preview]
// r[verify n8nclone.tui_ux.image_nodes.preview.canvas]
#[test]
fn canvas_renders_bounded_image_metadata_fallback_without_raw_bytes() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));
    app.add_image_input_node("Preview", IMAGE_PATH);
    let node_id = app.graph_state.focused.expect("image node focused");
    let port_id = app.workflow.graph.node(node_id).expect("image node").output_ports[0].id;
    app.run_output_lines.push(RunOutputLine {
        run_id: RunId(0),
        node_id: lattice::tui::graph::workflow_node_id(node_id),
        port_id,
        port_label: IMAGE_OUTPUT_LABEL.to_string(),
        text: IMAGE_SUMMARY.to_string(),
    });

    let rendered = rendered_frame_text(&mut app);

    assert!(rendered.contains("Image preview"), "preview missing: {rendered}");
    assert!(rendered.contains("image/png"), "metadata missing: {rendered}");
    assert!(!rendered.contains(RAW_IMAGE_BYTES), "raw bytes rendered: {rendered}");
}

// r[verify lattice.tui_ux.portable_authoring_parity.verification]
// r[verify lattice.tui_ux.portable_authoring_parity.discoverability.scenario.help]
#[cfg(unix)]
#[test]
fn operator_e2e_tui_process_authors_http_and_recovers_invalid_constrained_draft() {
    let temp_dir = tempfile::tempdir().expect("authoring PTY directory");
    let config_path = write_cli_config(&temp_dir);
    let mut session = TuiPtySession::start(&config_path, temp_dir.path());
    session.wait(TUI_WORKSPACE_SURFACES, "the initial authoring workspace");
    let cursor = session.cursor();
    session.input(b"\x17", "open typed workflow authoring");
    session.wait_fresh(cursor, &["Typed Authoring", "Workflow name"], "the typed metadata browser");
    let cursor = session.cursor();
    session.input(b"\r\x15\r", "reject an empty workflow name without applying it");
    session.wait_fresh(cursor, &["workflow name cannot be blank"], "recoverable workflow-name validation");
    let cursor = session.cursor();
    session.resize(PTY_CONSTRAINED_ROWS, PTY_CONSTRAINED_COLUMNS);
    session.wait_fresh(cursor, &["Esc back"], "visible constrained-draft recovery");
    session.input(TUI_ESCAPE_INPUT, "cancel the invalid metadata draft");
    let cursor = session.cursor();
    session.resize(PTY_ROWS, PTY_COLUMNS);
    session.wait_fresh(cursor, &["Workflow name", "untitled"], "the unchanged prior name after cancellation");
    let cursor = session.cursor();
    session.input(b"\r\x15operator-authoring-pty\r", "author and validate the workflow name");
    session.wait_fresh(cursor, &["operator-authoring-pty"], "the authored workflow name");
    session.input(TUI_ESCAPE_INPUT, "close typed metadata authoring");
    let cursor = session.cursor();
    session.input(b"\x0eh\r", "choose guided HTTP creation");
    session.wait_fresh(cursor, &["HTTP request (METHOD URL)"], "the typed HTTP request draft");
    let mut request = vec![0x7f; 64];
    request.extend_from_slice(b"GET https://example.invalid/authored\r");
    let cursor = session.cursor();
    session.input(&request, "create an HTTP definition without dispatching a request");
    session.wait_fresh(cursor, &["HTTP 1"], "the authored HTTP node on the canvas");
    let cursor = session.cursor();
    session.input(b"\x13", "persist the authored workflow");
    session.wait_fresh(cursor, &["Saved 'operator-authoring-pty'"], "the actual save result");
    session.input(TUI_QUIT_INPUT, "exit the saved authoring session");
    let (status, transcript) = session.finish();
    assert!(status.success(), "authoring TUI exit: {status}; transcript: {transcript}");
    let store = Store::new(temp_dir.path().join("data").join(STORE_FILE_NAME)).unwrap();
    let workflow = store.load_workflow("operator-authoring-pty").unwrap().unwrap();
    assert_eq!(workflow.meta.name, "operator-authoring-pty");
    let config = workflow.configs.values().next().expect("authored HTTP config");
    let lattice::engine::workflow::NodeConfig::Http { method, url, .. } = config else {
        panic!("persisted config is not HTTP")
    };
    assert_eq!(method, "GET");
    assert_eq!(url, "https://example.invalid/authored");
    assert!(
        store.list_runs(TUI_RUN_HISTORY_LIMIT).unwrap().is_empty(),
        "authoring must not dispatch a workflow run"
    );
    let nickel = temp_dir.path().join("authored.ncl");
    std::fs::write(&nickel, lattice::workflow_format::workflow_to_nickel(&workflow)).unwrap();
    let imported = lattice::workflow_format::load_workflow_nickel_file(&nickel).unwrap();
    assert_eq!(
        lattice::workflow_format::workflow_to_portable(&imported),
        lattice::workflow_format::workflow_to_portable(&workflow)
    );
}
