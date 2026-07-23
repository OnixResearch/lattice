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
const PTY_TRANSCRIPT_MAX_BYTES: usize = 1_048_576;
const PTY_POLL_ATTEMPTS: usize = 250;
const PTY_POLL_DELAY_MS: u64 = 20;
const TUI_QUIT_INPUT: &[u8] = b"\x11";
const TUI_WORKSPACE_SURFACES: &[&str] = &["Canvas", "Properties", "Output", "untitled"];

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

fn transcript_has_workspace(transcript: &str) -> bool {
    TUI_WORKSPACE_SURFACES.iter().all(|surface| transcript.contains(surface))
}

fn wait_for_workspace(transcript: &Arc<Mutex<Vec<u8>>>) -> String {
    for _ in 0..PTY_POLL_ATTEMPTS {
        let text = transcript_text(transcript);
        if transcript_has_workspace(&text) {
            return text;
        }
        thread::sleep(Duration::from_millis(PTY_POLL_DELAY_MS));
    }
    panic!(
        "TUI did not render the workspace within the bounded PTY startup window: {}",
        transcript_text(transcript)
    );
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

#[test]
fn operator_e2e_tui_process_renders_in_a_pty_and_exits_on_keyboard_input() {
    let temp_dir = tempfile::tempdir().expect("temporary operator E2E directory");
    let config_path = write_cli_config(&temp_dir);
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
    writer.write_all(TUI_QUIT_INPUT).expect("quit input writes to PTY");
    writer.flush().expect("quit input flushes to PTY");
    let status = wait_for_pty_exit(child.as_mut());
    drop(writer);
    drop(pair.master);
    reader_thread.join().expect("PTY transcript reader joins");

    assert!(status.success(), "TUI exit status: {status}; transcript: {rendered}");
    assert!(transcript_has_workspace(&rendered));
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
#[tokio::test]
async fn active_run_keeps_rendering_and_input_live_while_replay_events_stream() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));
    app.add_shell_node("Long", LONG_RUNNING_COMMAND);

    assert!(app.start_workflow_run());
    for _ in 0..ACTIVE_REPLAY_POLL_ATTEMPTS {
        app.poll_run_events();
        if app.run_state.active_replay_timeline.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(ACTIVE_REPLAY_POLL_DELAY_MS)).await;
    }

    assert!(app.run_state.active);
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
        node_id,
        port_id,
        port_label: IMAGE_OUTPUT_LABEL.to_string(),
        text: IMAGE_SUMMARY.to_string(),
    });

    let rendered = rendered_frame_text(&mut app);

    assert!(rendered.contains("Image preview"), "preview missing: {rendered}");
    assert!(rendered.contains("image/png"), "metadata missing: {rendered}");
    assert!(!rendered.contains(RAW_IMAGE_BYTES), "raw bytes rendered: {rendered}");
}
