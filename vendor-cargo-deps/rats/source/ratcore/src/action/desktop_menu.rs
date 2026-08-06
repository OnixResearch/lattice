use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::*;

const ROOT_MENU_SCOPE: &str = "root";
const DEFAULT_MENU_ORDER: i32 = 0;
const SEPARATOR_LABEL: &str = "separator";

semantic_string_id! {
    /// Identifier for a desktop menu item, distinct from its optional action route.
    pub struct DesktopMenuItemId;
}

/// Desktop menu item kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopMenuItemKind {
    /// Actionable menu item.
    Action,
    /// Submenu container.
    Submenu,
    /// Visual separator.
    Separator,
}

/// Checked state for desktop menu items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopMenuCheckedState {
    /// No checked state.
    None,
    /// Independent checked item.
    Checked(bool),
    /// Radio item in a named group.
    Radio {
        /// Radio group identifier.
        group: String,
        /// Whether this item is selected.
        selected: bool,
    },
}

/// Declarative desktop menu entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMenuEntry {
    /// Stable menu item identifier.
    pub id: DesktopMenuItemId,
    /// Display label. Action entries may leave this empty to use action label.
    pub label: String,
    /// Item kind.
    pub kind: DesktopMenuItemKind,
    /// Optional action identifier.
    pub action_id: Option<ActionId>,
    /// Optional parent menu identifier.
    pub parent_id: Option<DesktopMenuItemId>,
    /// Stable order within parent.
    pub order: i32,
    /// Optional mnemonic.
    pub mnemonic: Option<char>,
    /// Optional accelerator.
    pub accelerator: Option<ActionShortcut>,
    /// Checked or radio state.
    pub checked: DesktopMenuCheckedState,
    /// Whether the menu entry is enabled before registry validation.
    pub enabled: bool,
    /// Optional disabled reason.
    pub disabled_reason: Option<String>,
}

impl DesktopMenuEntry {
    /// Creates an action menu item.
    #[must_use]
    pub fn action(id: impl Into<DesktopMenuItemId>, action_id: impl Into<ActionId>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            kind: DesktopMenuItemKind::Action,
            action_id: Some(action_id.into()),
            parent_id: None,
            order: DEFAULT_MENU_ORDER,
            mnemonic: None,
            accelerator: None,
            checked: DesktopMenuCheckedState::None,
            enabled: true,
            disabled_reason: None,
        }
    }

    /// Creates a submenu container.
    #[must_use]
    pub fn submenu(id: impl Into<DesktopMenuItemId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind: DesktopMenuItemKind::Submenu,
            action_id: None,
            parent_id: None,
            order: DEFAULT_MENU_ORDER,
            mnemonic: None,
            accelerator: None,
            checked: DesktopMenuCheckedState::None,
            enabled: true,
            disabled_reason: None,
        }
    }

    /// Creates a separator.
    #[must_use]
    pub fn separator(id: impl Into<DesktopMenuItemId>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            kind: DesktopMenuItemKind::Separator,
            action_id: None,
            parent_id: None,
            order: DEFAULT_MENU_ORDER,
            mnemonic: None,
            accelerator: None,
            checked: DesktopMenuCheckedState::None,
            enabled: false,
            disabled_reason: None,
        }
    }

    /// Sets display label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets parent menu identifier.
    #[must_use]
    pub fn with_parent(mut self, parent_id: impl Into<DesktopMenuItemId>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Sets order.
    #[must_use]
    pub const fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Sets mnemonic.
    #[must_use]
    pub const fn with_mnemonic(mut self, mnemonic: char) -> Self {
        self.mnemonic = Some(mnemonic);
        self
    }

    /// Sets accelerator.
    #[must_use]
    pub fn with_accelerator(mut self, accelerator: ActionShortcut) -> Self {
        self.accelerator = Some(accelerator);
        self
    }

    /// Sets checked state.
    #[must_use]
    pub fn with_checked(mut self, checked: DesktopMenuCheckedState) -> Self {
        self.checked = checked;
        self
    }

    /// Sets enablement.
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool, disabled_reason: Option<String>) -> Self {
        self.enabled = enabled;
        self.disabled_reason = disabled_reason;
        self
    }
}

/// Desktop menu diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopMenuDiagnosticCode {
    /// Duplicate menu item identifier.
    DuplicateItemId,
    /// Required label is empty.
    EmptyLabel,
    /// Parent item is absent.
    MissingParent,
    /// Duplicate mnemonic in one menu scope.
    DuplicateMnemonic,
    /// Radio group has no selected enabled item.
    InvalidRadioGroup,
    /// Checked item has no action source.
    MissingCheckedStateSource,
    /// Accelerator or shortcut conflicts with a sibling item.
    ShortcutConflict,
    /// Action item references an absent action.
    MissingAction,
}

/// Desktop menu diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMenuDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: DesktopMenuDiagnosticCode,
    /// Menu item associated with the diagnostic.
    pub item_id: Option<DesktopMenuItemId>,
    /// Other conflicting item identifiers.
    pub related_item_ids: Vec<DesktopMenuItemId>,
    /// Human-readable message.
    pub message: String,
}

impl DesktopMenuDiagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(
        code: DesktopMenuDiagnosticCode,
        item_id: Option<DesktopMenuItemId>,
        related_item_ids: Vec<DesktopMenuItemId>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            item_id,
            related_item_ids,
            message: message.into(),
        }
    }
}

/// Renderable desktop menu item projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMenuItemProjection {
    /// Stable item identifier.
    pub id: DesktopMenuItemId,
    /// Display label.
    pub label: String,
    /// Item kind.
    pub kind: DesktopMenuItemKind,
    /// Optional action identifier.
    pub action_id: Option<ActionId>,
    /// Parent item identifier.
    pub parent_id: Option<DesktopMenuItemId>,
    /// Stable order.
    pub order: i32,
    /// Optional accelerator label.
    pub accelerator_label: Option<String>,
    /// Optional mnemonic.
    pub mnemonic: Option<char>,
    /// Checked state.
    pub checked: DesktopMenuCheckedState,
    /// Whether the item can be activated.
    pub enabled: bool,
    /// Disabled reason when present.
    pub disabled_reason: Option<String>,
}

/// Desktop menu projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMenuProjection {
    /// Renderable items sorted by parent, order, and identifier.
    pub items: Vec<DesktopMenuItemProjection>,
    /// Diagnostics.
    pub diagnostics: Vec<DesktopMenuDiagnostic>,
}

/// Focus role assigned to a menu item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopMenuFocusRole {
    /// Menu bar item.
    MenuBarItem,
    /// Submenu item.
    MenuItem,
    /// Submenu container.
    Submenu,
}

/// Focus metadata for a menu item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMenuFocusNode {
    /// Item identifier.
    pub item_id: DesktopMenuItemId,
    /// Focus role.
    pub role: DesktopMenuFocusRole,
    /// Accessible label.
    pub label: String,
    /// Whether the item is disabled.
    pub disabled: bool,
    /// Optional shortcut text.
    pub shortcut_label: Option<String>,
    /// Whether the item can be activated.
    pub can_activate: bool,
}

/// Focus projection for a desktop menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMenuFocusProjection {
    /// Focusable nodes; separators and hidden rows are skipped.
    pub nodes: Vec<DesktopMenuFocusNode>,
    /// First active item identifier when one exists.
    pub active_item_id: Option<DesktopMenuItemId>,
}

/// Renderer consuming menu projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopMenuRenderer {
    /// Meteorite/Dioxus renderer.
    Meteorite,
    /// Subwayrat/ratatui renderer.
    Subwayrat,
}

/// Renderer adapter projection for desktop menus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopMenuRendererProjection {
    /// Renderer consuming the projection.
    pub renderer: DesktopMenuRenderer,
    /// Shared menu projection.
    pub menu: DesktopMenuProjection,
    /// Shared focus projection.
    pub focus: DesktopMenuFocusProjection,
}

/// Builds a desktop menu projection from registry metadata and menu entries.
#[must_use]
pub fn build_desktop_menu_projection(
    snapshot: &ActionRegistrySnapshot,
    entries: Vec<DesktopMenuEntry>,
) -> DesktopMenuProjection {
    let mut diagnostics = Vec::new();
    diagnostics.extend(duplicate_item_diagnostics(&entries));
    diagnostics.extend(missing_parent_diagnostics(&entries));
    diagnostics.extend(duplicate_mnemonic_diagnostics(&entries));
    diagnostics.extend(shortcut_conflict_diagnostics(&entries));
    diagnostics.extend(invalid_radio_group_diagnostics(&entries));
    let mut items: Vec<_> = entries.iter().map(|entry| project_entry(snapshot, entry, &mut diagnostics)).collect();
    items.sort_by(|left, right| {
        parent_scope(&left.parent_id)
            .cmp(&parent_scope(&right.parent_id))
            .then(left.order.cmp(&right.order))
            .then(left.id.cmp(&right.id))
    });
    DesktopMenuProjection { items, diagnostics }
}

/// Projects menu items into focus and accessibility metadata.
#[must_use]
pub fn project_desktop_menu_focus(menu: &DesktopMenuProjection) -> DesktopMenuFocusProjection {
    let nodes: Vec<_> = menu
        .items
        .iter()
        .filter(|item| item.kind != DesktopMenuItemKind::Separator)
        .map(|item| DesktopMenuFocusNode {
            item_id: item.id.clone(),
            role: focus_role(item),
            label: item.label.clone(),
            disabled: !item.enabled,
            shortcut_label: item.accelerator_label.clone(),
            can_activate: item.enabled && item.kind == DesktopMenuItemKind::Action,
        })
        .collect();
    let active_item_id = nodes.iter().find(|node| node.can_activate).map(|node| node.item_id.clone());
    DesktopMenuFocusProjection { nodes, active_item_id }
}

/// Wraps shared menu and focus projections for a renderer.
#[must_use]
pub fn project_desktop_menu_renderer(
    renderer: DesktopMenuRenderer,
    snapshot: &ActionRegistrySnapshot,
    entries: Vec<DesktopMenuEntry>,
) -> DesktopMenuRendererProjection {
    let menu = build_desktop_menu_projection(snapshot, entries);
    let focus = project_desktop_menu_focus(&menu);
    DesktopMenuRendererProjection { renderer, menu, focus }
}

fn project_entry(
    snapshot: &ActionRegistrySnapshot,
    entry: &DesktopMenuEntry,
    diagnostics: &mut Vec<DesktopMenuDiagnostic>,
) -> DesktopMenuItemProjection {
    let action = entry.action_id.as_ref().and_then(|action_id| snapshot.action(action_id));
    if entry.kind == DesktopMenuItemKind::Action && action.is_none() {
        diagnostics.push(DesktopMenuDiagnostic::new(
            DesktopMenuDiagnosticCode::MissingAction,
            Some(entry.id.clone()),
            Vec::new(),
            "menu action references an absent registry action",
        ));
    }
    let label = resolved_label(entry, action, diagnostics);
    let registry_enabled = action.map_or(true, |action| snapshot.is_active(action));
    let registry_disabled_reason = action.and_then(|action| snapshot.inactive_reason(action));
    let missing_checked_source = entry.checked != DesktopMenuCheckedState::None && entry.action_id.is_none();
    if missing_checked_source {
        diagnostics.push(DesktopMenuDiagnostic::new(
            DesktopMenuDiagnosticCode::MissingCheckedStateSource,
            Some(entry.id.clone()),
            Vec::new(),
            "checked menu item has no action source",
        ));
    }
    DesktopMenuItemProjection {
        id: entry.id.clone(),
        label,
        kind: entry.kind,
        action_id: entry.action_id.clone(),
        parent_id: entry.parent_id.clone(),
        order: entry.order,
        accelerator_label: entry.accelerator.as_ref().map(ActionShortcut::display),
        mnemonic: entry.mnemonic,
        checked: entry.checked.clone(),
        enabled: entry.enabled && registry_enabled && !missing_checked_source,
        disabled_reason: entry
            .disabled_reason
            .clone()
            .or(registry_disabled_reason)
            .or_else(|| missing_checked_source.then(|| "missing checked action source".to_string())),
    }
}

fn resolved_label(
    entry: &DesktopMenuEntry,
    action: Option<&ActionDescriptor>,
    diagnostics: &mut Vec<DesktopMenuDiagnostic>,
) -> String {
    if entry.kind == DesktopMenuItemKind::Separator {
        return SEPARATOR_LABEL.to_string();
    }
    let label = if entry.label.trim().is_empty() {
        action.map_or_else(String::new, |action| action.label.clone())
    } else {
        entry.label.clone()
    };
    if label.trim().is_empty() {
        diagnostics.push(DesktopMenuDiagnostic::new(
            DesktopMenuDiagnosticCode::EmptyLabel,
            Some(entry.id.clone()),
            Vec::new(),
            "menu item label is empty",
        ));
    }
    label
}

fn duplicate_item_diagnostics(entries: &[DesktopMenuEntry]) -> Vec<DesktopMenuDiagnostic> {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for entry in entries {
        if !seen.insert(entry.id.clone()) {
            diagnostics.push(DesktopMenuDiagnostic::new(
                DesktopMenuDiagnosticCode::DuplicateItemId,
                Some(entry.id.clone()),
                Vec::new(),
                "duplicate menu item id",
            ));
        }
    }
    diagnostics
}

fn missing_parent_diagnostics(entries: &[DesktopMenuEntry]) -> Vec<DesktopMenuDiagnostic> {
    let ids: BTreeSet<_> = entries.iter().map(|entry| &entry.id).collect();
    entries
        .iter()
        .filter_map(|entry| {
            let parent_id = entry.parent_id.as_ref()?;
            (!ids.contains(parent_id)).then(|| {
                DesktopMenuDiagnostic::new(
                    DesktopMenuDiagnosticCode::MissingParent,
                    Some(entry.id.clone()),
                    vec![parent_id.clone()],
                    "menu parent is absent",
                )
            })
        })
        .collect()
}

fn duplicate_mnemonic_diagnostics(entries: &[DesktopMenuEntry]) -> Vec<DesktopMenuDiagnostic> {
    let mut grouped: BTreeMap<(String, char), Vec<DesktopMenuItemId>> = BTreeMap::new();
    for entry in entries {
        let Some(mnemonic) = entry.mnemonic else {
            continue;
        };
        grouped
            .entry((parent_scope(&entry.parent_id), mnemonic.to_ascii_lowercase()))
            .or_default()
            .push(entry.id.clone());
    }
    grouped
        .into_iter()
        .filter_map(|((_scope, _mnemonic), ids)| {
            let (first, rest) = ids.split_first()?;
            (!rest.is_empty()).then(|| {
                DesktopMenuDiagnostic::new(
                    DesktopMenuDiagnosticCode::DuplicateMnemonic,
                    Some(first.clone()),
                    rest.to_vec(),
                    "duplicate mnemonic in menu scope",
                )
            })
        })
        .collect()
}

fn shortcut_conflict_diagnostics(entries: &[DesktopMenuEntry]) -> Vec<DesktopMenuDiagnostic> {
    let mut grouped: BTreeMap<(String, String), Vec<DesktopMenuItemId>> = BTreeMap::new();
    for entry in entries {
        let Some(accelerator) = &entry.accelerator else {
            continue;
        };
        grouped
            .entry((parent_scope(&entry.parent_id), accelerator.display()))
            .or_default()
            .push(entry.id.clone());
    }
    grouped
        .into_iter()
        .filter_map(|((_scope, _shortcut), ids)| {
            let (first, rest) = ids.split_first()?;
            (!rest.is_empty()).then(|| {
                DesktopMenuDiagnostic::new(
                    DesktopMenuDiagnosticCode::ShortcutConflict,
                    Some(first.clone()),
                    rest.to_vec(),
                    "duplicate accelerator in menu scope",
                )
            })
        })
        .collect()
}

fn invalid_radio_group_diagnostics(entries: &[DesktopMenuEntry]) -> Vec<DesktopMenuDiagnostic> {
    let mut grouped: BTreeMap<(String, String), Vec<&DesktopMenuEntry>> = BTreeMap::new();
    for entry in entries {
        let DesktopMenuCheckedState::Radio { group, .. } = &entry.checked else {
            continue;
        };
        grouped.entry((parent_scope(&entry.parent_id), group.clone())).or_default().push(entry);
    }
    grouped
        .into_iter()
        .filter_map(|((_scope, _group), items)| {
            let selected_enabled = items.iter().any(|item| {
                item.enabled && matches!(item.checked, DesktopMenuCheckedState::Radio { selected: true, .. })
            });
            (!selected_enabled).then(|| {
                DesktopMenuDiagnostic::new(
                    DesktopMenuDiagnosticCode::InvalidRadioGroup,
                    items.first().map(|item| item.id.clone()),
                    items.iter().map(|item| item.id.clone()).collect(),
                    "radio group has no selected enabled item",
                )
            })
        })
        .collect()
}

fn parent_scope(parent_id: &Option<DesktopMenuItemId>) -> String {
    parent_id.as_ref().map_or_else(|| ROOT_MENU_SCOPE.to_string(), ToString::to_string)
}

fn focus_role(item: &DesktopMenuItemProjection) -> DesktopMenuFocusRole {
    if item.parent_id.is_none() && item.kind == DesktopMenuItemKind::Submenu {
        return DesktopMenuFocusRole::MenuBarItem;
    }
    if item.kind == DesktopMenuItemKind::Submenu {
        return DesktopMenuFocusRole::Submenu;
    }
    DesktopMenuFocusRole::MenuItem
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_ORDER: i32 = 1;
    const SAVE_ORDER: i32 = 2;
    const VIEW_ORDER: i32 = 3;

    fn registry() -> ActionRegistrySnapshot {
        build_registry(vec![
            ActionDescriptor::new("file.save", "Save", ActionSource::new("test"))
                .with_shortcut(ActionShortcut::global("s", vec!["Ctrl".to_string()])),
            ActionDescriptor::new("view.sidebar", "Sidebar", ActionSource::new("test")),
        ])
        .snapshot
    }

    #[test]
    fn nested_menu_preserves_action_identity_and_focus_metadata() {
        let entries = vec![
            DesktopMenuEntry::submenu("file", "File").with_order(FILE_ORDER),
            DesktopMenuEntry::action("save", "file.save")
                .with_parent("file")
                .with_mnemonic('S')
                .with_order(SAVE_ORDER),
            DesktopMenuEntry::separator("sep").with_parent("file"),
            DesktopMenuEntry::action("sidebar", "view.sidebar")
                .with_parent("file")
                .with_checked(DesktopMenuCheckedState::Checked(true))
                .with_order(VIEW_ORDER),
        ];

        let menu = build_desktop_menu_projection(&registry(), entries);
        let focus = project_desktop_menu_focus(&menu);

        assert!(menu.diagnostics.is_empty());
        assert!(
            menu.items
                .iter()
                .any(|item| { item.action_id.as_ref().is_some_and(|id| id == "file.save") && item.label == "Save" })
        );
        assert!(focus.nodes.iter().all(|node| node.label != SEPARATOR_LABEL));
        assert_eq!(focus.active_item_id, Some(DesktopMenuItemId::new("save")));
    }

    #[test]
    fn malformed_menu_reports_duplicate_empty_parent_mnemonic_radio_shortcut_and_action_errors() {
        let conflict_shortcut = ActionShortcut::global("k", vec!["Ctrl".to_string()]);
        let entries = vec![
            DesktopMenuEntry::action("dup", "missing.action").with_label(""),
            DesktopMenuEntry::action("dup", "file.save").with_parent("missing-parent"),
            DesktopMenuEntry::action("mnemonic-a", "file.save")
                .with_mnemonic('X')
                .with_accelerator(conflict_shortcut.clone()),
            DesktopMenuEntry::action("other", "file.save")
                .with_mnemonic('x')
                .with_accelerator(conflict_shortcut)
                .with_checked(DesktopMenuCheckedState::Radio {
                    group: "view-mode".to_string(),
                    selected: false,
                }),
            DesktopMenuEntry::submenu("checked", "Checked").with_checked(DesktopMenuCheckedState::Checked(true)),
        ];

        let menu = build_desktop_menu_projection(&registry(), entries);

        assert!(
            menu.diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DesktopMenuDiagnosticCode::DuplicateItemId })
        );
        assert!(
            menu.diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DesktopMenuDiagnosticCode::MissingParent })
        );
        assert!(
            menu.diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DesktopMenuDiagnosticCode::DuplicateMnemonic })
        );
        assert!(
            menu.diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DesktopMenuDiagnosticCode::InvalidRadioGroup })
        );
        assert!(
            menu.diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DesktopMenuDiagnosticCode::ShortcutConflict })
        );
        assert!(
            menu.diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DesktopMenuDiagnosticCode::MissingCheckedStateSource })
        );
        assert!(
            menu.diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DesktopMenuDiagnosticCode::MissingAction })
        );
    }

    #[test]
    fn renderer_menu_projections_share_semantics() {
        let entries = vec![DesktopMenuEntry::action("save", "file.save")];

        let web = project_desktop_menu_renderer(DesktopMenuRenderer::Meteorite, &registry(), entries.clone());
        let tui = project_desktop_menu_renderer(DesktopMenuRenderer::Subwayrat, &registry(), entries);

        assert_eq!(web.menu, tui.menu);
        assert_eq!(web.focus, tui.focus);
    }
}
