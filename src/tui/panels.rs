//! Panel rendering: properties sidebar, output log, status bar.

use rat_fuzzy::FuzzyFinder;
use rat_fuzzy::FuzzyStyle;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::view::InteractionMode;
use rat_nodegraph::view::NodeGraphWidget;
use rat_streaming::render_streaming_lines;
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;

use crate::node_kind_provider;
use crate::replay::ReplayCursor;
use crate::replay::ReplayFrame;
use crate::replay::ReplayTimeline;
use crate::store::RunDetailRecord;
use crate::store::RunNodeDetail;
use crate::store::RunNodeDetailStatus;
use crate::tui::app::AddNodeKind;
use crate::tui::app::AddNodeStep;
use crate::tui::app::FocusPanel;
use crate::tui::app::NodeRunStatus;
use crate::tui::app::PropertyEditField;
use crate::tui::app::rendered_latest_node_output_lines;
use crate::workflow_validation::DiagnosticSeverity;

const COMPACT_STATUS_HELP_WIDTH: u16 = 72;
const FULL_EMPTY_STATE_MIN_WIDTH: u16 = 48;
const FULL_EMPTY_STATE_MIN_HEIGHT: u16 = 7;
const CANVAS_IMAGE_PREVIEW_MAX_CHARS: usize = 80;
const CANVAS_PREVIEW_HEIGHT: u16 = 1;
const QUIT_CONFIRM_WIDTH: u16 = 62;
const QUIT_CONFIRM_HEIGHT: u16 = 5;
const QUIT_CONFIRM_TOP_OFFSET: u16 = 3;
const WORKFLOW_BROWSER_WIDTH: u16 = 64;
const WORKFLOW_BROWSER_HEIGHT: u16 = 16;
const WORKFLOW_BROWSER_TOP_OFFSET: u16 = 2;
const WORKFLOW_TEMPLATE_BROWSER_WIDTH: u16 = 78;
const WORKFLOW_TEMPLATE_BROWSER_HEIGHT: u16 = 20;
const WORKFLOW_TEMPLATE_BROWSER_TOP_OFFSET: u16 = 2;
const WORKFLOW_TEMPLATE_METADATA_ROWS: usize = 6;
const ADD_NODE_WIDTH: u16 = 66;
const ADD_NODE_HEIGHT: u16 = 14;
const ADD_NODE_TOP_OFFSET: u16 = 2;
const RUN_HISTORY_WIDTH: u16 = 72;
const RUN_HISTORY_HEIGHT: u16 = 16;
const RUN_HISTORY_TOP_OFFSET: u16 = 2;
const APPROVAL_WIDTH: u16 = 78;
const APPROVAL_HEIGHT: u16 = 18;
const APPROVAL_TOP_OFFSET: u16 = 2;
const DIAGNOSTICS_WIDTH: u16 = 78;
const DIAGNOSTICS_HEIGHT: u16 = 16;
const DIAGNOSTICS_TOP_OFFSET: u16 = 2;
const VERTICAL_SECTION_COUNT: usize = 3;
const HORIZONTAL_SECTION_COUNT: usize = 2;

mod layout;
use layout::canvas::*;
use layout::footer::*;
use layout::sidebar::*;
mod overlays;
use overlays::browsers::*;
use overlays::chooser::*;
use overlays::editor::*;
use overlays::entry::*;
mod summaries;
use summaries::*;
mod timeline;
use timeline::*;

/// Render the full application layout.
pub fn render(app: &mut crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    // Top-level vertical split: main area + output + status bar.
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // canvas + properties
            Constraint::Length(10), // output log
            Constraint::Length(1),  // status bar
        ])
        .split(area);
    debug_assert_eq!(vert.len(), VERTICAL_SECTION_COUNT);

    // Horizontal split for canvas and properties panel.
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // canvas
            Constraint::Percentage(30), // properties
        ])
        .split(vert[0]);
    debug_assert_eq!(horiz.len(), HORIZONTAL_SECTION_COUNT);

    render_surface(app, horiz[0], buf);
    render_properties(app, horiz[1], buf);
    render_output(app, vert[1], buf);
    render_status_bar(app, vert[2], buf);

    // Overlay: fuzzy palette.
    if app.show_palette {
        render_palette(app, area, buf);
    }

    if app.is_diagnostics_open() {
        render_diagnostics(app, area, buf);
    }

    if app.is_run_history_open() {
        render_run_history(app, area, buf);
    }

    if app.is_approval_list_open() {
        render_approvals(app, area, buf);
    }

    if app.is_workflow_template_browser_open() {
        render_workflow_template_browser(app, area, buf);
    }

    if app.is_workflow_browser_open() {
        render_workflow_browser(app, area, buf);
    }

    if app.is_add_node_flow_open() {
        render_add_node_flow(app, area, buf);
    }

    if app.authoring.is_some() {
        render_authoring(app, area, buf);
    }

    if app.is_confirming_quit() {
        render_quit_confirmation(area, buf);
    }
}

#[cfg(test)]
mod tests;
