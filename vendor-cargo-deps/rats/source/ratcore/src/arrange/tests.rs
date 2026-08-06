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

fn panel(id: &str) -> PanelId {
    PanelId::new(id)
}

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
        .focus_panel(&panel(EDITOR))
        .unwrap();
    assert_eq!(layout.focus.as_ref().map(PanelId::as_str), Some(EDITOR));
    assert!(layout.root.contains_panel(&panel(SIDEBAR)));
    assert!(layout.root.contains_panel(&panel(TERMINAL)));
    assert!(layout.floating.iter().any(|floating| floating.panel == SEARCH));
    let regions = layout.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT));
    assert!(regions.contains_key(EDITOR));
    assert!(regions.contains_key(TERMINAL));
    assert!(regions.contains_key(SEARCH));
}

#[test]
fn invalid_operations_are_rejected_without_mutating_previous_layout() {
    let layout = DockLayout::new(EDITOR);
    assert_eq!(layout.clone().focus_panel(&panel(MISSING)).unwrap_err(), DockingError::UnknownPanel(panel(MISSING)));
    assert_eq!(
        layout.clone().float_panel(FloatingPanel::new(EDITOR)).unwrap_err(),
        DockingError::DuplicatePanel(panel(EDITOR))
    );
    assert_eq!(layout.panel_ids(), vec![panel(EDITOR)]);
    assert_eq!(SplitRatio::new(HALF_DENOMINATOR, HALF_DENOMINATOR), Err(DockingError::InvalidRatio));
}

#[test]
fn canonical_ratio_rejects_legacy_edges_and_preserves_geometry() {
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 0.0, 1.0, 1.1] {
        assert_eq!(SplitRatio::try_from_legacy_f32(invalid), Err(DockingError::InvalidRatio));
    }

    let ratio = SplitRatio::try_from_legacy_f32(0.35).unwrap();
    assert_eq!(SplitRatio::new(2_u16, 4_u16).unwrap(), half());
    assert!((ratio.as_legacy_f32() - 0.35).abs() < f32::EPSILON * 2.0);
    assert_eq!(ratio.first_len(VIEWPORT_WIDTH), 42);
    assert_eq!(ratio.complement().first_len(VIEWPORT_WIDTH), 78);
}

#[test]
fn insertion_positions_check_host_bounds_and_apply_explicit_policy() {
    let position = DockInsertionIndex::new(8);
    assert_eq!(position.checked_for_len(7), None);
    assert_eq!(position.checked_for_len(8), Some(8));
    assert_eq!(position.clamped_for_len(3), 3);

    let unrepresentable_input = u128::from(u64::MAX).saturating_add(1);
    assert_eq!(
        DockInsertionIndex::try_from(unrepresentable_input),
        Err(DockingError::InvalidInsertionIndex(unrepresentable_input))
    );
}

#[test]
fn identity_namespaces_are_stable_independent_map_keys() {
    let panel_id = PanelId::new(EDITOR);
    let legacy_name = LegacyZoneName::new(EDITOR);
    let node_id = DockNodeId::from_legacy_name(&legacy_name);
    let panel_map = std::collections::HashMap::from([(panel_id.clone(), "panel")]);
    let node_map = std::collections::HashMap::from([(node_id.clone(), "node")]);

    assert_eq!(panel_id.to_string(), EDITOR);
    assert_eq!(node_id.to_string(), EDITOR);
    assert_eq!(panel_map.get(&panel_id), Some(&"panel"));
    assert_eq!(node_map.get(&node_id), Some(&"node"));
}

#[test]
fn resize_root_and_project_regions_are_deterministic() {
    let layout = DockLayout::new(EDITOR)
        .split_focused(SIDEBAR, DockAxis::Horizontal, half())
        .unwrap()
        .resize_root_split(third())
        .unwrap();
    let regions = layout.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT));
    assert_eq!(regions.get(EDITOR).unwrap().width, VIEWPORT_WIDTH / THIRD_DENOMINATOR);
    assert_eq!(regions.get(SIDEBAR).unwrap().height, VIEWPORT_HEIGHT);
}

#[test]
fn snapshot_restore_validates_version_duplicates_focus_and_constraints() {
    let layout = DockLayout::new(EDITOR).split_focused(SIDEBAR, DockAxis::Horizontal, half()).unwrap();
    let snapshot = layout.snapshot();
    assert_eq!(DockLayout::restore(snapshot.clone()).unwrap(), layout);
    let mut bad_version = snapshot.clone();
    bad_version.version = UNSUPPORTED_VERSION;
    assert_eq!(DockLayout::restore(bad_version).unwrap_err(), DockingError::UnsupportedVersion(UNSUPPORTED_VERSION));
    let mut bad_focus = snapshot.clone();
    bad_focus.focus = Some(panel(MISSING));
    assert_eq!(DockLayout::restore(bad_focus).unwrap_err(), DockingError::UnknownPanel(panel(MISSING)));
    let mut bad_tabs = snapshot;
    bad_tabs.root = DockNode::Tabs {
        panels: vec![panel(EDITOR)],
        active: panel(MISSING),
    };
    assert_eq!(DockLayout::restore(bad_tabs).unwrap_err(), DockingError::InvalidTabs(panel(MISSING)));
}

#[test]
fn tui_and_web_intents_share_semantics() {
    let start = DockLayout::new(EDITOR);
    let tui = apply_intent(
        start.clone(),
        DockIntent::new(DockRenderer::Tui, DockOperation::SplitFocused {
            panel: panel(SIDEBAR),
            axis: DockAxis::Horizontal,
            ratio: half(),
        }),
    )
    .unwrap();
    let web = apply_intent(
        start,
        DockIntent::new(DockRenderer::Web, DockOperation::SplitFocused {
            panel: panel(SIDEBAR),
            axis: DockAxis::Horizontal,
            ratio: half(),
        }),
    )
    .unwrap();
    assert_eq!(tui.root, web.root);
    assert_eq!(tui.focus, web.focus);
    assert_eq!(
        tui.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)),
        web.project_regions(DockRect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT))
    );
}
