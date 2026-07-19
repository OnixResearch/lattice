use super::*;

const EDITOR: &str = "editor";
const SIDEBAR: &str = "sidebar";
const TERMINAL: &str = "terminal";
const SEARCH: &str = "search";
const MISSING: &str = "missing";
const VIEWPORT_WIDTH: u16 = 120;
const VIEWPORT_HEIGHT: u16 = 40;
const HALF_NUMERATOR: u16 = 1;
const HALF_DENOMINATOR: u16 = 2;
const THIRD_NUMERATOR: u16 = 1;
const THIRD_DENOMINATOR: u16 = 3;
const UNSUPPORTED_VERSION: u16 = 99;

fn half() -> SplitRatio {
    SplitRatio::new(HALF_NUMERATOR, HALF_DENOMINATOR).unwrap()
}

fn third() -> SplitRatio {
    SplitRatio::new(THIRD_NUMERATOR, THIRD_DENOMINATOR).unwrap()
}

#[test]
fn layout_tree_represents_splits_tabs_floating_and_focus() {
    let layout = DockLayout::new(EDITOR)
        .split_focused(SIDEBAR, DockAxis::Horizontal, third())
        .unwrap()
        .tab_focused(TERMINAL)
        .unwrap()
        .float_panel(FloatingPanel::new(SEARCH))
        .unwrap()
        .focus_panel(EDITOR)
        .unwrap();
    assert_eq!(layout.focus.as_deref(), Some(EDITOR));
    assert!(layout.root.contains_panel(SIDEBAR));
    assert!(layout.root.contains_panel(TERMINAL));
    assert!(
        layout
            .floating
            .iter()
            .any(|floating| floating.panel == SEARCH)
    );
    let regions = layout.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT));
    assert!(regions.contains_key(EDITOR));
    assert!(regions.contains_key(TERMINAL));
    assert!(regions.contains_key(SEARCH));
}

#[test]
fn invalid_operations_are_rejected_without_mutating_previous_layout() {
    let layout = DockLayout::new(EDITOR);
    assert_eq!(
        layout.clone().focus_panel(MISSING).unwrap_err(),
        DockingError::UnknownPanel(MISSING.to_string())
    );
    assert_eq!(
        layout
            .clone()
            .float_panel(FloatingPanel::new(EDITOR))
            .unwrap_err(),
        DockingError::DuplicatePanel(EDITOR.to_string())
    );
    assert_eq!(layout.panel_ids(), vec![EDITOR.to_string()]);
    assert_eq!(
        SplitRatio::new(HALF_DENOMINATOR, HALF_DENOMINATOR),
        Err(DockingError::InvalidRatio)
    );
}

#[test]
fn resize_root_and_project_regions_are_deterministic() {
    let layout = DockLayout::new(EDITOR)
        .split_focused(SIDEBAR, DockAxis::Horizontal, half())
        .unwrap()
        .resize_root_split(third())
        .unwrap();
    let regions = layout.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT));
    assert_eq!(
        regions.get(EDITOR).unwrap().width,
        VIEWPORT_WIDTH / THIRD_DENOMINATOR
    );
    assert_eq!(regions.get(SIDEBAR).unwrap().height, VIEWPORT_HEIGHT);
}

#[test]
fn snapshot_restore_validates_version_duplicates_focus_and_constraints() {
    let layout = DockLayout::new(EDITOR)
        .split_focused(SIDEBAR, DockAxis::Horizontal, half())
        .unwrap();
    let snapshot = layout.snapshot();
    assert_eq!(DockLayout::restore(snapshot.clone()).unwrap(), layout);
    let mut bad_version = snapshot.clone();
    bad_version.version = UNSUPPORTED_VERSION;
    assert_eq!(
        DockLayout::restore(bad_version).unwrap_err(),
        DockingError::UnsupportedVersion(UNSUPPORTED_VERSION)
    );
    let mut bad_focus = snapshot.clone();
    bad_focus.focus = Some(MISSING.to_string());
    assert_eq!(
        DockLayout::restore(bad_focus).unwrap_err(),
        DockingError::UnknownPanel(MISSING.to_string())
    );
    let mut bad_tabs = snapshot;
    bad_tabs.root = DockNode::Tabs {
        panels: vec![EDITOR.to_string()],
        active: MISSING.to_string(),
    };
    assert_eq!(
        DockLayout::restore(bad_tabs).unwrap_err(),
        DockingError::InvalidTabs(MISSING.to_string())
    );
}

#[test]
fn tui_and_web_intents_share_semantics() {
    let start = DockLayout::new(EDITOR);
    let tui = apply_intent(
        start.clone(),
        DockIntent::new(
            DockRenderer::Tui,
            DockOperation::SplitFocused {
                panel: SIDEBAR.to_string(),
                axis: DockAxis::Horizontal,
                ratio: half(),
            },
        ),
    )
    .unwrap();
    let web = apply_intent(
        start,
        DockIntent::new(
            DockRenderer::Web,
            DockOperation::SplitFocused {
                panel: SIDEBAR.to_string(),
                axis: DockAxis::Horizontal,
                ratio: half(),
            },
        ),
    )
    .unwrap();
    assert_eq!(tui.root, web.root);
    assert_eq!(tui.focus, web.focus);
    assert_eq!(
        tui.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)),
        web.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT))
    );
}
