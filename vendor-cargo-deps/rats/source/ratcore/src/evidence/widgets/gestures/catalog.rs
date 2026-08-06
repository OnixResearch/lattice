use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub(super) const ACTION_CLICK_CHROME: &str = "mouse.click.chrome";
pub(super) const ACTION_DOUBLE_COMMAND: &str = "mouse.double-click.command-palette";
pub(super) const ACTION_CONTEXT_TABLE: &str = "mouse.context.table";
pub(super) const ACTION_WHEEL_TREE: &str = "mouse.wheel.tree";
pub(super) const ACTION_DRAG_START_DOCKING: &str = "mouse.drag-start.docking";
pub(super) const ACTION_DRAG_UPDATE_DOCKING: &str = "mouse.drag-update.docking";
pub(super) const ACTION_DRAG_RELEASE_DOCKING: &str = "mouse.drag-release.docking";
pub(super) const ACTION_CANCEL_OVERLAY: &str = "mouse.cancel.overlay";
pub(super) const ACTION_SHIFT_SELECT_SPREADSHEET: &str = "mouse.shift-select.spreadsheet";
pub(super) const ACTION_DISABLED_REGION: &str = "mouse.disabled.region";
pub(super) const ACTION_MISSING_DRAG_START: &str = "mouse.drag-update.missing-start";
pub(super) const ACTION_STALE_CAPTURE: &str = "mouse.drag-release.stale-capture";
pub(super) const ACTION_UNSUPPORTED_BACKEND: &str = "mouse.unsupported.backend";

const CLASS_CLICK: &str = "click";
const CLASS_DOUBLE_CLICK: &str = "double-click";
const CLASS_CONTEXT: &str = "context";
const CLASS_WHEEL: &str = "wheel";
const CLASS_DRAG_START: &str = "drag-start";
const CLASS_DRAG_UPDATE: &str = "drag-update";
const CLASS_DRAG_RELEASE: &str = "drag-release";
const CLASS_CANCEL: &str = "cancel";
const CLASS_SHIFT_SELECT: &str = "shift-select";
const CLASS_UNSUPPORTED: &str = "unsupported";
const SURFACE_CHROME: &str = "chrome";
const SURFACE_COMMAND_PALETTE: &str = "command-palette";
const SURFACE_TABLE: &str = "table";
const SURFACE_TREE: &str = "tree";
const SURFACE_DOCKING: &str = "docking";
const SURFACE_OVERLAY: &str = "overlay";
const SURFACE_SPREADSHEET: &str = "spreadsheet";
const SURFACE_BACKEND: &str = "backend";
const ROUTE_CHROME_PRIMARY: &str = "chrome.primary-action";
const ROUTE_COMMAND_OPEN: &str = "command.open-active";
const ROUTE_TABLE_CONTEXT: &str = "table.context-menu";
const ROUTE_TREE_SCROLL: &str = "tree.scroll";
const ROUTE_DOCKING_SPLITTER: &str = "docking.splitter";
const ROUTE_OVERLAY_CANCEL: &str = "overlay.cancel";
const ROUTE_SPREADSHEET_RANGE: &str = "spreadsheet.range-select";
const ROUTE_BACKEND_UNSUPPORTED: &str = "backend.unsupported-wheel";
const CAPTURE_DOCKING_SPLITTER: &str = "capture.docking.splitter";
const CAPTURE_STALE: &str = "capture.stale";
const DIAGNOSTIC_BACKEND_UNSUPPORTED: &str = "backend.unsupported-wheel";

pub(super) fn baseline_actions() -> BTreeMap<String, super::core::PointerMouseAction> {
    BTreeMap::from([
        routed(ActionSpec {
            action_id: ACTION_CLICK_CHROME,
            event_class: CLASS_CLICK,
            surface: SURFACE_CHROME,
            route_id: ROUTE_CHROME_PRIMARY,
        }),
        routed(ActionSpec {
            action_id: ACTION_DOUBLE_COMMAND,
            event_class: CLASS_DOUBLE_CLICK,
            surface: SURFACE_COMMAND_PALETTE,
            route_id: ROUTE_COMMAND_OPEN,
        }),
        routed(ActionSpec {
            action_id: ACTION_CONTEXT_TABLE,
            event_class: CLASS_CONTEXT,
            surface: SURFACE_TABLE,
            route_id: ROUTE_TABLE_CONTEXT,
        }),
        routed(ActionSpec {
            action_id: ACTION_WHEEL_TREE,
            event_class: CLASS_WHEEL,
            surface: SURFACE_TREE,
            route_id: ROUTE_TREE_SCROLL,
        }),
        cancel_action(),
        routed(ActionSpec {
            action_id: ACTION_SHIFT_SELECT_SPREADSHEET,
            event_class: CLASS_SHIFT_SELECT,
            surface: SURFACE_SPREADSHEET,
            route_id: ROUTE_SPREADSHEET_RANGE,
        }),
        drag_start_action(),
        drag_update_action(ACTION_DRAG_UPDATE_DOCKING),
        drag_release_action(ACTION_DRAG_RELEASE_DOCKING),
        drag_update_action(ACTION_MISSING_DRAG_START),
        stale_capture_action(),
        unsupported_action(),
    ])
}

pub(super) fn disabled_actions() -> BTreeSet<String> {
    [ACTION_DISABLED_REGION.to_string()].into_iter().collect()
}

struct ActionSpec<'a> {
    action_id: &'a str,
    event_class: &'a str,
    surface: &'a str,
    route_id: &'a str,
}

fn routed(spec: ActionSpec<'_>) -> (String, super::core::PointerMouseAction) {
    (
        spec.action_id.to_string(),
        super::core::PointerMouseAction::route(spec.event_class, spec.surface, spec.route_id),
    )
}

fn drag_start_action() -> (String, super::core::PointerMouseAction) {
    (
        ACTION_DRAG_START_DOCKING.to_string(),
        super::core::PointerMouseAction::route(CLASS_DRAG_START, SURFACE_DOCKING, ROUTE_DOCKING_SPLITTER)
            .with_behavior(super::core::PointerMouseBehavior::DragStart {
                capture: CAPTURE_DOCKING_SPLITTER.to_string(),
            }),
    )
}

fn drag_update_action(action_id: &str) -> (String, super::core::PointerMouseAction) {
    (
        action_id.to_string(),
        super::core::PointerMouseAction::route(CLASS_DRAG_UPDATE, SURFACE_DOCKING, ROUTE_DOCKING_SPLITTER)
            .with_behavior(super::core::PointerMouseBehavior::DragUpdate {
                capture: CAPTURE_DOCKING_SPLITTER.to_string(),
            }),
    )
}

fn drag_release_action(action_id: &str) -> (String, super::core::PointerMouseAction) {
    (
        action_id.to_string(),
        super::core::PointerMouseAction::route(CLASS_DRAG_RELEASE, SURFACE_DOCKING, ROUTE_DOCKING_SPLITTER)
            .with_behavior(super::core::PointerMouseBehavior::DragRelease {
                capture: CAPTURE_DOCKING_SPLITTER.to_string(),
            }),
    )
}

fn cancel_action() -> (String, super::core::PointerMouseAction) {
    (
        ACTION_CANCEL_OVERLAY.to_string(),
        super::core::PointerMouseAction::route(CLASS_CANCEL, SURFACE_OVERLAY, ROUTE_OVERLAY_CANCEL)
            .with_behavior(super::core::PointerMouseBehavior::DragCancel),
    )
}

fn stale_capture_action() -> (String, super::core::PointerMouseAction) {
    (
        ACTION_STALE_CAPTURE.to_string(),
        super::core::PointerMouseAction::route(CLASS_DRAG_RELEASE, SURFACE_DOCKING, ROUTE_DOCKING_SPLITTER)
            .with_behavior(super::core::PointerMouseBehavior::StaleCapture {
                expected: CAPTURE_DOCKING_SPLITTER.to_string(),
                actual: CAPTURE_STALE.to_string(),
            }),
    )
}

fn unsupported_action() -> (String, super::core::PointerMouseAction) {
    (
        ACTION_UNSUPPORTED_BACKEND.to_string(),
        super::core::PointerMouseAction::route(CLASS_UNSUPPORTED, SURFACE_BACKEND, ROUTE_BACKEND_UNSUPPORTED)
            .with_behavior(super::core::PointerMouseBehavior::Unsupported {
                diagnostic: DIAGNOSTIC_BACKEND_UNSUPPORTED.to_string(),
            }),
    )
}
