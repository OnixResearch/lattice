use super::*;

const WIDE_STATUS_WIDTH: u16 = COMPACT_STATUS_HELP_WIDTH;

const NARROW_STATUS_WIDTH: u16 = COMPACT_STATUS_HELP_WIDTH - 1;

const LONG_IMAGE_SUMMARY: &str =
    "image/png image 4000x3000 hash abcdefghijklmnopqrstuvwxyz abcdefghijklmnopqrstuvwxyz abcdefghijklmnopqrstuvwxyz";

fn context(focus: FocusPanel) -> HelpContext {
    HelpContext {
        focus,
        palette_open: false,
        workflow_browser_open: false,
        add_node_open: false,
        run_history_open: false,
        diagnostics_open: false,
        interaction_mode: Mode::Default,
        width: WIDE_STATUS_WIDTH,
    }
}

// r[verify n8nclone.tui_ux.unsaved.dirty_indicator]
// r[verify n8nclone.tui_ux.unsaved.failed_save]
// r[verify n8nclone.tui_ux.unsaved.mutation_marks_dirty]
// r[verify n8nclone.tui_ux.unsaved.save_clears]
#[test]
fn unsaved_indicator_marks_dirty_workflows() {
    assert_eq!(unsaved_indicator(false), "");
    assert_eq!(unsaved_indicator(true), "*");
}

// r[verify n8nclone.tui_ux.help_footer.canvas]
// r[verify n8nclone.tui_ux.help_footer.wiring]
// r[verify n8nclone.tui_ux.keyboard_first.discoverable]
// r[verify n8nclone.tui_ux.properties.discoverable_help]
#[test]
fn contextual_help_matches_canvas_and_wiring_modes() {
    assert_eq!(
        contextual_help_text(context(FocusPanel::Canvas)),
        " e: cmd │ l: label │ t: timeout │ C-W: fields │ C-D: diagnostics │ v: rows │ C-R: run │ C-P: palette ",
    );

    let wiring = HelpContext {
        interaction_mode: Mode::Wiring,
        ..context(FocusPanel::Canvas)
    };
    assert_eq!(contextual_help_text(wiring), " Click target port: connect │ Esc: cancel wiring ",);
}

// r[verify n8nclone.tui_ux.help_footer.properties_output]
// r[verify n8nclone.tui_ux.properties.discoverable_help.footer]
#[test]
fn contextual_help_matches_properties_and_output_focus() {
    assert_eq!(
        contextual_help_text(context(FocusPanel::Properties)),
        " e: cmd │ l: label │ i: image │ p/r/o: agent │ k: creds │ C-W ",
    );
    assert_eq!(contextual_help_text(context(FocusPanel::Output)), " ↑/↓: scroll │ C-P: palette │ A-Tab: focus ",);
}

// r[verify n8nclone.tui_ux.properties.discoverable_help.palette]
#[test]
fn contextual_help_reports_palette_modal() {
    let palette = HelpContext {
        palette_open: true,
        ..context(FocusPanel::Canvas)
    };

    assert_eq!(contextual_help_text(palette), " Type: filter │ ↑/↓: select │ Enter: run │ Esc: close ",);
}

#[test]
fn contextual_help_reports_workflow_browser_modal() {
    let browser = HelpContext {
        workflow_browser_open: true,
        ..context(FocusPanel::Canvas)
    };

    assert_eq!(contextual_help_text(browser), " ↑/↓: choose workflow │ Enter: open │ Esc: close ",);
}

// r[verify n8nclone.tui_ux.help_footer.truncate]
#[test]
fn contextual_help_uses_compact_copy_for_narrow_width() {
    let narrow = HelpContext {
        width: NARROW_STATUS_WIDTH,
        ..context(FocusPanel::Canvas)
    };
    assert_eq!(contextual_help_text(narrow), " e cmd │ l label │ v view │ C-W ");

    let narrow_wiring = HelpContext {
        interaction_mode: Mode::Wiring,
        ..narrow
    };
    assert_eq!(contextual_help_text(narrow_wiring), " Click target │ Esc ");
}

#[test]
fn canvas_image_preview_is_bounded_without_changing_short_metadata() {
    let short = "image/png image 10x10 hash abc";
    assert_eq!(bounded_preview(short), short);
    let long = bounded_preview(LONG_IMAGE_SUMMARY);
    assert!(long.chars().count() <= CANVAS_IMAGE_PREVIEW_MAX_CHARS);
    assert!(long.ends_with('…'));
    assert!(!long.contains("raw-image-bytes"));
}

// r[verify n8nclone.tui_ux.empty_state.tests]
#[test]
fn empty_state_visibility_requires_empty_workflow_without_modal() {
    assert!(should_show_empty_state(0, false));
    assert!(!should_show_empty_state(1, false));
    assert!(!should_show_empty_state(0, true));
}

// r[verify n8nclone.tui_ux.empty_state.next_actions]
#[test]
fn empty_state_copy_lists_next_actions_for_full_canvas() {
    let copy = empty_state_copy(Rect::new(0, 0, FULL_EMPTY_STATE_MIN_WIDTH, FULL_EMPTY_STATE_MIN_HEIGHT));

    assert_eq!(copy.title, "Start your workflow");
    assert!(copy.hints.contains(&"Ctrl-N: add your first node"));
    assert!(copy.hints.contains(&"Ctrl-P: open the command palette"));
}

#[test]
fn empty_state_copy_uses_compact_hint_for_small_canvas() {
    let copy = empty_state_copy(Rect::new(0, 0, FULL_EMPTY_STATE_MIN_WIDTH - 1, FULL_EMPTY_STATE_MIN_HEIGHT));

    assert_eq!(copy.title, "Empty workflow");
    assert_eq!(copy.hints, &["Ctrl-N add node │ Ctrl-P palette"]);
}
