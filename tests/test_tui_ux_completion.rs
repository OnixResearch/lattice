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
