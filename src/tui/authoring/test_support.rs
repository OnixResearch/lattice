//! Shared behavioral driver: owner regression tables use the actual TUI input
//! path, then assert typed outcomes in their own domain-specific tests.
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use crate::engine::Workflow;
use crate::tui::App;
use crate::tui::input;

pub(crate) fn edit(workflow: &mut Workflow, node_id: Option<u64>, id: &str, value: &str) -> Result<(), String> {
    let root = tempfile::tempdir().map_err(|error| error.to_string())?;
    let mut app = App::new(crate::config::AppConfig {
        data_dir: root.path().join("state"),
        ..Default::default()
    });
    app.workflow = workflow.clone();
    if let Some(node_id) = node_id {
        app.graph_state
            .selected
            .insert(crate::tui::graph::subwayrat_node_id(crate::engine::workflow::NodeId(node_id)));
    }
    app.open_authoring("all");
    let state = app.authoring.as_mut().ok_or("Authoring modal did not open")?;
    state.selected = state
        .fields
        .iter()
        .position(|field| field.id == id)
        .ok_or_else(|| format!("No typed TUI authoring action for {id}"))?;
    send(&mut app, KeyModifiers::NONE, KeyCode::Enter);
    send(&mut app, KeyModifiers::CONTROL, KeyCode::Char('u'));
    for character in value.chars() {
        if character == '\n' {
            send(&mut app, KeyModifiers::CONTROL, KeyCode::Char('j'));
        } else {
            send(&mut app, KeyModifiers::NONE, KeyCode::Char(character));
        }
    }
    send(&mut app, KeyModifiers::NONE, KeyCode::Enter);
    let state = app.authoring.as_ref().ok_or("Authoring modal unexpectedly closed")?;
    if let Some(error) = &state.error {
        return Err(error.clone());
    }
    if state.draft.is_some() {
        return Err("Typed TUI draft was not committed".into());
    }
    *workflow = app.workflow;
    Ok(())
}

pub(crate) fn send(app: &mut App, modifiers: KeyModifiers, code: KeyCode) {
    let action = input::handle_event(app, &Event::Key(KeyEvent::new(code, modifiers)));
    assert!(action.is_none(), "modal input must not escape to an effectful application action: {action:?}");
}
