use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::*;

const EMPTY_QUERY: &str = "";

/// Default shortcut assigned to an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingDefault {
    /// Action identifier.
    pub action_id: ActionId,
    /// Default shortcut.
    pub shortcut: ActionShortcut,
}

impl KeybindingDefault {
    /// Creates a default shortcut entry.
    #[must_use]
    pub fn new(action_id: impl Into<ActionId>, shortcut: ActionShortcut) -> Self {
        Self {
            action_id: action_id.into(),
            shortcut,
        }
    }
}

/// User shortcut override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingOverride {
    /// Action identifier.
    pub action_id: ActionId,
    /// Override shortcut; `None` means the action was explicitly cleared.
    pub shortcut: Option<ActionShortcut>,
}

impl KeybindingOverride {
    /// Creates an assigned override.
    #[must_use]
    pub fn assign(action_id: impl Into<ActionId>, shortcut: ActionShortcut) -> Self {
        Self {
            action_id: action_id.into(),
            shortcut: Some(shortcut),
        }
    }

    /// Creates a clear override.
    #[must_use]
    pub fn clear(action_id: impl Into<ActionId>) -> Self {
        Self {
            action_id: action_id.into(),
            shortcut: None,
        }
    }
}

/// Reserved shortcut policy entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedShortcut {
    /// Reserved shortcut.
    pub shortcut: ActionShortcut,
    /// Reason shown to users.
    pub reason: String,
}

impl ReservedShortcut {
    /// Creates a reserved shortcut.
    #[must_use]
    pub fn new(shortcut: ActionShortcut, reason: impl Into<String>) -> Self {
        Self {
            shortcut,
            reason: reason.into(),
        }
    }
}

/// Keybinding editor diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindingEditorDiagnosticCode {
    /// Registry has no actions.
    EmptyRegistry,
    /// Override or command references an unknown action.
    UnknownAction,
    /// Action is disabled and cannot be rebound.
    DisabledAction,
    /// More than one override targets the same action.
    DuplicateOverride,
    /// Shortcut is reserved by policy.
    ReservedShortcut,
    /// Override has malformed shortcut data.
    MalformedOverride,
    /// Shortcut conflicts with another action in the same scope.
    ShortcutConflict,
    /// Shortcut capture was empty.
    EmptyCapture,
    /// Shortcut capture used an unsupported key.
    UnsupportedKey,
    /// Shortcut capture contained malformed modifiers.
    MalformedModifier,
}

/// Keybinding editor diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingEditorDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: KeybindingEditorDiagnosticCode,
    /// Primary action identifier.
    pub action_id: Option<ActionId>,
    /// Conflicting action identifiers.
    pub related_action_ids: Vec<ActionId>,
    /// Human-readable message.
    pub message: String,
}

impl KeybindingEditorDiagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(
        code: KeybindingEditorDiagnosticCode,
        action_id: Option<ActionId>,
        related_action_ids: Vec<ActionId>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            action_id,
            related_action_ids,
            message: message.into(),
        }
    }
}

/// Projected keybinding editor row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingEditorRow {
    /// Action identifier.
    pub action_id: ActionId,
    /// Action label.
    pub label: String,
    /// Action category.
    pub category: String,
    /// Default shortcut label.
    pub default_shortcut_label: Option<String>,
    /// Current shortcut label.
    pub current_shortcut_label: Option<String>,
    /// Whether current binding differs from default.
    pub changed: bool,
    /// Whether the action can be edited.
    pub editable: bool,
    /// Row diagnostics.
    pub diagnostics: Vec<KeybindingEditorDiagnostic>,
}

/// Keybinding editor projection model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingEditorModel {
    /// Rows after search filtering.
    pub rows: Vec<KeybindingEditorRow>,
    /// Current search query.
    pub search_query: String,
    /// Number of changed bindings before filtering.
    pub changed_count: usize,
    /// Global diagnostics.
    pub diagnostics: Vec<KeybindingEditorDiagnostic>,
}

/// Shortcut edit command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutEditCommand {
    /// Assign a captured shortcut.
    Assign {
        /// Action identifier.
        action_id: ActionId,
        /// Shortcut to assign.
        shortcut: ActionShortcut,
    },
    /// Clear the current shortcut.
    Clear {
        /// Action identifier.
        action_id: ActionId,
    },
    /// Reset to default shortcut.
    Reset {
        /// Action identifier.
        action_id: ActionId,
    },
    /// Renderer reported an unsupported key capture.
    UnsupportedKey {
        /// Action identifier.
        action_id: ActionId,
        /// Unsupported key label.
        key: String,
    },
}

/// Shortcut edit reducer outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutEditOutcome {
    /// Accepted overrides after the command.
    pub overrides: Vec<KeybindingOverride>,
    /// Rebuilt editor model.
    pub model: KeybindingEditorModel,
    /// Diagnostic when command failed closed.
    pub diagnostic: Option<KeybindingEditorDiagnostic>,
}

/// Renderer family consuming keybinding editor projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindingEditorRenderer {
    /// Meteorite/Dioxus renderer.
    Meteorite,
    /// Subwayrat/ratatui renderer.
    Subwayrat,
}

/// Renderer adapter projection for the keybinding editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeybindingEditorRendererProjection {
    /// Renderer consuming the projection.
    pub renderer: KeybindingEditorRenderer,
    /// Shared editor model.
    pub model: KeybindingEditorModel,
}

/// Builds a keybinding editor model from registry, defaults, overrides, policy, and search.
#[must_use]
pub fn build_keybinding_editor_model(
    snapshot: &ActionRegistrySnapshot,
    defaults: &[KeybindingDefault],
    overrides: &[KeybindingOverride],
    reserved: &[ReservedShortcut],
    search_query: &str,
) -> KeybindingEditorModel {
    let mut diagnostics = validate_editor_inputs(snapshot, defaults, overrides, reserved);
    let current = current_shortcuts(snapshot, defaults, overrides);
    diagnostics.extend(conflict_diagnostics(snapshot, &current));
    diagnostics.extend(reserved_diagnostics(&current, reserved));
    let all_rows: Vec<_> = snapshot
        .actions()
        .iter()
        .map(|action| editor_row(snapshot, action, defaults, &current, &diagnostics))
        .collect();
    let changed_count = all_rows.iter().filter(|row| row.changed).count();
    let normalized_query = search_query.trim().to_ascii_lowercase();
    let rows = all_rows.into_iter().filter(|row| query_matches(row, &normalized_query)).collect();
    if snapshot.actions().is_empty() {
        diagnostics.push(KeybindingEditorDiagnostic::new(
            KeybindingEditorDiagnosticCode::EmptyRegistry,
            None,
            Vec::new(),
            "action registry is empty",
        ));
    }
    KeybindingEditorModel {
        rows,
        search_query: search_query.to_string(),
        changed_count,
        diagnostics,
    }
}

/// Applies a shortcut edit command and rebuilds the editor model.
#[must_use]
pub fn apply_shortcut_edit(
    snapshot: &ActionRegistrySnapshot,
    defaults: &[KeybindingDefault],
    overrides: &[KeybindingOverride],
    reserved: &[ReservedShortcut],
    command: ShortcutEditCommand,
    search_query: &str,
) -> ShortcutEditOutcome {
    let mut next_overrides = overrides.to_vec();
    let diagnostic = validate_shortcut_command(snapshot, defaults, &next_overrides, reserved, &command);
    if diagnostic.is_none() {
        apply_valid_command(&mut next_overrides, command);
    }
    let model = build_keybinding_editor_model(snapshot, defaults, &next_overrides, reserved, search_query);
    ShortcutEditOutcome {
        overrides: next_overrides,
        model,
        diagnostic,
    }
}

/// Wraps a shared keybinding editor model for a renderer.
#[must_use]
pub fn project_keybinding_editor_renderer(
    renderer: KeybindingEditorRenderer,
    snapshot: &ActionRegistrySnapshot,
    defaults: &[KeybindingDefault],
    overrides: &[KeybindingOverride],
    reserved: &[ReservedShortcut],
    search_query: &str,
) -> KeybindingEditorRendererProjection {
    KeybindingEditorRendererProjection {
        renderer,
        model: build_keybinding_editor_model(snapshot, defaults, overrides, reserved, search_query),
    }
}

fn validate_editor_inputs(
    snapshot: &ActionRegistrySnapshot,
    defaults: &[KeybindingDefault],
    overrides: &[KeybindingOverride],
    reserved: &[ReservedShortcut],
) -> Vec<KeybindingEditorDiagnostic> {
    let mut diagnostics = Vec::new();
    let action_ids: BTreeSet<_> = snapshot.actions().iter().map(|action| &action.id).collect();
    diagnostics.extend(unknown_default_diagnostics(defaults, &action_ids));
    diagnostics.extend(unknown_override_diagnostics(overrides, &action_ids));
    diagnostics.extend(duplicate_override_diagnostics(overrides));
    for action in snapshot.actions() {
        if !snapshot.is_active(action) {
            diagnostics.push(KeybindingEditorDiagnostic::new(
                KeybindingEditorDiagnosticCode::DisabledAction,
                Some(action.id.clone()),
                Vec::new(),
                "disabled action cannot be rebound",
            ));
        }
    }
    for override_entry in overrides {
        if let Some(shortcut) = &override_entry.shortcut {
            diagnostics.extend(validate_shortcut_shape(
                Some(override_entry.action_id.clone()),
                shortcut,
                KeybindingEditorDiagnosticCode::MalformedOverride,
            ));
        }
    }
    for reserved_shortcut in reserved {
        diagnostics.extend(validate_shortcut_shape(
            None,
            &reserved_shortcut.shortcut,
            KeybindingEditorDiagnosticCode::MalformedOverride,
        ));
    }
    diagnostics
}

fn unknown_default_diagnostics(
    defaults: &[KeybindingDefault],
    action_ids: &BTreeSet<&ActionId>,
) -> Vec<KeybindingEditorDiagnostic> {
    defaults
        .iter()
        .filter(|default| !action_ids.contains(&default.action_id))
        .map(|default| {
            KeybindingEditorDiagnostic::new(
                KeybindingEditorDiagnosticCode::UnknownAction,
                Some(default.action_id.clone()),
                Vec::new(),
                "default shortcut references unknown action",
            )
        })
        .collect()
}

fn unknown_override_diagnostics(
    overrides: &[KeybindingOverride],
    action_ids: &BTreeSet<&ActionId>,
) -> Vec<KeybindingEditorDiagnostic> {
    overrides
        .iter()
        .filter(|override_entry| !action_ids.contains(&override_entry.action_id))
        .map(|override_entry| {
            KeybindingEditorDiagnostic::new(
                KeybindingEditorDiagnosticCode::UnknownAction,
                Some(override_entry.action_id.clone()),
                Vec::new(),
                "override references unknown action",
            )
        })
        .collect()
}

fn duplicate_override_diagnostics(overrides: &[KeybindingOverride]) -> Vec<KeybindingEditorDiagnostic> {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for override_entry in overrides {
        if !seen.insert(override_entry.action_id.clone()) {
            diagnostics.push(KeybindingEditorDiagnostic::new(
                KeybindingEditorDiagnosticCode::DuplicateOverride,
                Some(override_entry.action_id.clone()),
                Vec::new(),
                "duplicate shortcut override",
            ));
        }
    }
    diagnostics
}

fn current_shortcuts(
    snapshot: &ActionRegistrySnapshot,
    defaults: &[KeybindingDefault],
    overrides: &[KeybindingOverride],
) -> BTreeMap<ActionId, Option<ActionShortcut>> {
    let mut current = BTreeMap::new();
    for action in snapshot.actions() {
        current.insert(action.id.clone(), action.shortcut.clone());
    }
    for default in defaults {
        current.insert(default.action_id.clone(), Some(default.shortcut.clone()));
    }
    for override_entry in overrides {
        current.insert(override_entry.action_id.clone(), override_entry.shortcut.clone());
    }
    current
}

fn conflict_diagnostics(
    snapshot: &ActionRegistrySnapshot,
    current: &BTreeMap<ActionId, Option<ActionShortcut>>,
) -> Vec<KeybindingEditorDiagnostic> {
    let mut grouped: BTreeMap<String, Vec<ActionId>> = BTreeMap::new();
    for action in snapshot.actions() {
        if !snapshot.is_active(action) {
            continue;
        }
        let Some(Some(shortcut)) = current.get(&action.id) else {
            continue;
        };
        grouped.entry(shortcut_key(shortcut)).or_default().push(action.id.clone());
    }
    grouped
        .into_iter()
        .filter_map(|(_shortcut, ids)| {
            let (first, rest) = ids.split_first()?;
            (!rest.is_empty()).then(|| {
                KeybindingEditorDiagnostic::new(
                    KeybindingEditorDiagnosticCode::ShortcutConflict,
                    Some(first.clone()),
                    rest.to_vec(),
                    "shortcut conflicts with another enabled action in scope",
                )
            })
        })
        .collect()
}

fn reserved_diagnostics(
    current: &BTreeMap<ActionId, Option<ActionShortcut>>,
    reserved: &[ReservedShortcut],
) -> Vec<KeybindingEditorDiagnostic> {
    let reserved_keys: BTreeMap<_, _> = reserved
        .iter()
        .map(|reserved_shortcut| (shortcut_key(&reserved_shortcut.shortcut), reserved_shortcut.reason.clone()))
        .collect();
    current
        .iter()
        .filter_map(|(action_id, shortcut)| {
            let shortcut = shortcut.as_ref()?;
            let reason = reserved_keys.get(&shortcut_key(shortcut))?;
            Some(KeybindingEditorDiagnostic::new(
                KeybindingEditorDiagnosticCode::ReservedShortcut,
                Some(action_id.clone()),
                Vec::new(),
                reason.clone(),
            ))
        })
        .collect()
}

fn editor_row(
    snapshot: &ActionRegistrySnapshot,
    action: &ActionDescriptor,
    defaults: &[KeybindingDefault],
    current: &BTreeMap<ActionId, Option<ActionShortcut>>,
    diagnostics: &[KeybindingEditorDiagnostic],
) -> KeybindingEditorRow {
    let default_shortcut = defaults
        .iter()
        .find(|default| default.action_id == action.id)
        .map(|default| default.shortcut.clone())
        .or_else(|| action.shortcut.clone());
    let current_shortcut = current.get(&action.id).cloned().flatten();
    let row_diagnostics: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.action_id.as_ref() == Some(&action.id))
        .cloned()
        .collect();
    KeybindingEditorRow {
        action_id: action.id.clone(),
        label: action.label.clone(),
        category: action.category.clone(),
        default_shortcut_label: default_shortcut.as_ref().map(ActionShortcut::display),
        current_shortcut_label: current_shortcut.as_ref().map(ActionShortcut::display),
        changed: default_shortcut != current_shortcut,
        editable: snapshot.is_active(action) && row_diagnostics.is_empty(),
        diagnostics: row_diagnostics,
    }
}

fn query_matches(row: &KeybindingEditorRow, normalized_query: &str) -> bool {
    normalized_query == EMPTY_QUERY
        || row.action_id.as_str().to_ascii_lowercase().contains(normalized_query)
        || row.label.to_ascii_lowercase().contains(normalized_query)
        || row.category.to_ascii_lowercase().contains(normalized_query)
}

fn validate_shortcut_command(
    snapshot: &ActionRegistrySnapshot,
    defaults: &[KeybindingDefault],
    overrides: &[KeybindingOverride],
    reserved: &[ReservedShortcut],
    command: &ShortcutEditCommand,
) -> Option<KeybindingEditorDiagnostic> {
    match command {
        ShortcutEditCommand::Assign { action_id, shortcut } => {
            validate_assign_command(snapshot, defaults, overrides, reserved, action_id, shortcut)
        }
        ShortcutEditCommand::Clear { action_id } | ShortcutEditCommand::Reset { action_id } => {
            validate_action_editable(snapshot, action_id)
        }
        ShortcutEditCommand::UnsupportedKey { action_id, key } => Some(KeybindingEditorDiagnostic::new(
            KeybindingEditorDiagnosticCode::UnsupportedKey,
            Some(action_id.clone()),
            Vec::new(),
            format!("unsupported key capture: {key}"),
        )),
    }
}

fn validate_assign_command(
    snapshot: &ActionRegistrySnapshot,
    defaults: &[KeybindingDefault],
    overrides: &[KeybindingOverride],
    reserved: &[ReservedShortcut],
    action_id: &ActionId,
    shortcut: &ActionShortcut,
) -> Option<KeybindingEditorDiagnostic> {
    if let Some(diagnostic) = validate_action_editable(snapshot, action_id) {
        return Some(diagnostic);
    }
    if shortcut.key.trim().is_empty() {
        return Some(KeybindingEditorDiagnostic::new(
            KeybindingEditorDiagnosticCode::EmptyCapture,
            Some(action_id.clone()),
            Vec::new(),
            "shortcut capture is empty",
        ));
    }
    if shortcut.modifiers.iter().any(|modifier| modifier.trim().is_empty()) {
        return Some(KeybindingEditorDiagnostic::new(
            KeybindingEditorDiagnosticCode::MalformedModifier,
            Some(action_id.clone()),
            Vec::new(),
            "shortcut capture contains an empty modifier",
        ));
    }
    if let Some(reserved_shortcut) = reserved
        .iter()
        .find(|reserved_shortcut| shortcut_key(&reserved_shortcut.shortcut) == shortcut_key(shortcut))
    {
        return Some(KeybindingEditorDiagnostic::new(
            KeybindingEditorDiagnosticCode::ReservedShortcut,
            Some(action_id.clone()),
            Vec::new(),
            reserved_shortcut.reason.clone(),
        ));
    }
    let mut next_overrides = overrides.to_vec();
    upsert_override(&mut next_overrides, KeybindingOverride::assign(action_id.clone(), shortcut.clone()));
    let current = current_shortcuts(snapshot, defaults, &next_overrides);
    conflict_diagnostics(snapshot, &current).into_iter().find(|diagnostic| {
        diagnostic.action_id.as_ref() == Some(action_id)
            || diagnostic.related_action_ids.iter().any(|id| id == action_id)
    })
}

fn validate_action_editable(
    snapshot: &ActionRegistrySnapshot,
    action_id: &ActionId,
) -> Option<KeybindingEditorDiagnostic> {
    let Some(action) = snapshot.action(action_id) else {
        return Some(KeybindingEditorDiagnostic::new(
            KeybindingEditorDiagnosticCode::UnknownAction,
            Some(action_id.clone()),
            Vec::new(),
            "shortcut command references unknown action",
        ));
    };
    (!snapshot.is_active(action)).then(|| {
        KeybindingEditorDiagnostic::new(
            KeybindingEditorDiagnosticCode::DisabledAction,
            Some(action_id.clone()),
            Vec::new(),
            "disabled action cannot be rebound",
        )
    })
}

fn validate_shortcut_shape(
    action_id: Option<ActionId>,
    shortcut: &ActionShortcut,
    malformed_code: KeybindingEditorDiagnosticCode,
) -> Vec<KeybindingEditorDiagnostic> {
    let mut diagnostics = Vec::new();
    if shortcut.key.trim().is_empty() {
        diagnostics.push(KeybindingEditorDiagnostic::new(
            malformed_code,
            action_id.clone(),
            Vec::new(),
            "shortcut key is empty",
        ));
    }
    if shortcut.modifiers.iter().any(|modifier| modifier.trim().is_empty()) {
        diagnostics.push(KeybindingEditorDiagnostic::new(
            malformed_code,
            action_id,
            Vec::new(),
            "shortcut modifier is empty",
        ));
    }
    diagnostics
}

fn apply_valid_command(overrides: &mut Vec<KeybindingOverride>, command: ShortcutEditCommand) {
    match command {
        ShortcutEditCommand::Assign { action_id, shortcut } => {
            upsert_override(overrides, KeybindingOverride::assign(action_id, shortcut));
        }
        ShortcutEditCommand::Clear { action_id } => {
            upsert_override(overrides, KeybindingOverride::clear(action_id));
        }
        ShortcutEditCommand::Reset { action_id } => {
            overrides.retain(|override_entry| override_entry.action_id != action_id);
        }
        ShortcutEditCommand::UnsupportedKey { .. } => {}
    }
}

fn upsert_override(overrides: &mut Vec<KeybindingOverride>, next: KeybindingOverride) {
    overrides.retain(|override_entry| override_entry.action_id != next.action_id);
    overrides.push(next);
    overrides.sort_by(|left, right| left.action_id.cmp(&right.action_id));
}

fn shortcut_key(shortcut: &ActionShortcut) -> String {
    format!(
        "{}:{}:{}",
        shortcut.scope.to_ascii_lowercase(),
        shortcut.key.to_ascii_lowercase(),
        shortcut
            .modifiers
            .iter()
            .map(|modifier| modifier.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join("+")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shortcut(key: &str) -> ActionShortcut {
        ActionShortcut::global(key, vec!["Ctrl".to_string()])
    }

    fn registry() -> ActionRegistrySnapshot {
        build_registry(vec![
            ActionDescriptor::new("file.save", "Save", ActionSource::new("test")).with_shortcut(shortcut("s")),
            ActionDescriptor::new("file.open", "Open", ActionSource::new("test")).with_shortcut(shortcut("o")),
            ActionDescriptor::new("file.disabled", "Disabled", ActionSource::new("test")).with_enablement(
                ActionEnablement::Disabled {
                    reason: "not available".to_string(),
                },
            ),
        ])
        .snapshot
    }

    #[test]
    fn editor_model_projects_defaults_overrides_search_and_changed_state() {
        let overrides = vec![KeybindingOverride::assign("file.save", shortcut("k"))];
        let model = build_keybinding_editor_model(&registry(), &[], &overrides, &[], "save");

        assert_eq!(model.rows.len(), 1);
        assert_eq!(model.rows[0].action_id, "file.save");
        assert_eq!(model.rows[0].current_shortcut_label, Some("Ctrl+k".to_string()));
        assert_eq!(model.changed_count, 1);
    }

    #[test]
    fn model_reports_empty_registry_disabled_duplicate_reserved_malformed_and_unknown_inputs() {
        let empty_registry = build_registry(Vec::new()).snapshot;
        let model = build_keybinding_editor_model(
            &empty_registry,
            &[KeybindingDefault::new("missing", shortcut("m"))],
            &[
                KeybindingOverride::assign("missing", ActionShortcut::global("", vec!["Ctrl".to_string()])),
                KeybindingOverride::clear("missing"),
            ],
            &[ReservedShortcut::new(shortcut("r"), "reserved")],
            EMPTY_QUERY,
        );
        let disabled_model = build_keybinding_editor_model(
            &registry(),
            &[],
            &[KeybindingOverride::assign("file.open", shortcut("s"))],
            &[ReservedShortcut::new(shortcut("s"), "reserved")],
            EMPTY_QUERY,
        );

        assert!(
            model
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == KeybindingEditorDiagnosticCode::EmptyRegistry })
        );
        assert!(
            model
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == KeybindingEditorDiagnosticCode::UnknownAction })
        );
        assert!(
            model
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == KeybindingEditorDiagnosticCode::DuplicateOverride })
        );
        assert!(
            model
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == KeybindingEditorDiagnosticCode::MalformedOverride })
        );
        assert!(
            disabled_model
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == KeybindingEditorDiagnosticCode::DisabledAction })
        );
        assert!(
            disabled_model
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == KeybindingEditorDiagnosticCode::ReservedShortcut })
        );
        assert!(
            disabled_model
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == KeybindingEditorDiagnosticCode::ShortcutConflict })
        );
    }

    #[test]
    fn edit_reducer_accepts_assign_clear_and_reset() {
        let assigned = apply_shortcut_edit(
            &registry(),
            &[],
            &[],
            &[],
            ShortcutEditCommand::Assign {
                action_id: ActionId::new("file.save"),
                shortcut: shortcut("k"),
            },
            EMPTY_QUERY,
        );
        let cleared = apply_shortcut_edit(
            &registry(),
            &[],
            &assigned.overrides,
            &[],
            ShortcutEditCommand::Clear {
                action_id: ActionId::new("file.save"),
            },
            EMPTY_QUERY,
        );
        let reset = apply_shortcut_edit(
            &registry(),
            &[],
            &cleared.overrides,
            &[],
            ShortcutEditCommand::Reset {
                action_id: ActionId::new("file.save"),
            },
            EMPTY_QUERY,
        );

        assert!(assigned.diagnostic.is_none());
        assert_eq!(assigned.overrides[0].shortcut, Some(shortcut("k")));
        assert_eq!(cleared.overrides[0].shortcut, None);
        assert!(reset.overrides.is_empty());
    }

    #[test]
    fn edit_reducer_rejects_empty_unsupported_malformed_reserved_conflicting_stale_and_disabled() {
        let reserved = vec![ReservedShortcut::new(shortcut("r"), "reserved")];
        let empty = apply_shortcut_edit(
            &registry(),
            &[],
            &[],
            &[],
            ShortcutEditCommand::Assign {
                action_id: ActionId::new("file.save"),
                shortcut: ActionShortcut::global("", vec!["Ctrl".to_string()]),
            },
            EMPTY_QUERY,
        );
        let malformed = apply_shortcut_edit(
            &registry(),
            &[],
            &[],
            &[],
            ShortcutEditCommand::Assign {
                action_id: ActionId::new("file.save"),
                shortcut: ActionShortcut::global("k", vec!["".to_string()]),
            },
            EMPTY_QUERY,
        );
        let reserved_result = apply_shortcut_edit(
            &registry(),
            &[],
            &[],
            &reserved,
            ShortcutEditCommand::Assign {
                action_id: ActionId::new("file.save"),
                shortcut: shortcut("r"),
            },
            EMPTY_QUERY,
        );
        let conflict = apply_shortcut_edit(
            &registry(),
            &[],
            &[],
            &[],
            ShortcutEditCommand::Assign {
                action_id: ActionId::new("file.open"),
                shortcut: shortcut("s"),
            },
            EMPTY_QUERY,
        );
        let stale = apply_shortcut_edit(
            &registry(),
            &[],
            &[],
            &[],
            ShortcutEditCommand::Clear {
                action_id: ActionId::new("missing"),
            },
            EMPTY_QUERY,
        );
        let disabled = apply_shortcut_edit(
            &registry(),
            &[],
            &[],
            &[],
            ShortcutEditCommand::Reset {
                action_id: ActionId::new("file.disabled"),
            },
            EMPTY_QUERY,
        );
        let unsupported = apply_shortcut_edit(
            &registry(),
            &[],
            &[],
            &[],
            ShortcutEditCommand::UnsupportedKey {
                action_id: ActionId::new("file.save"),
                key: "Hyper".to_string(),
            },
            EMPTY_QUERY,
        );

        assert!(matches!(
            empty.diagnostic,
            Some(KeybindingEditorDiagnostic {
                code: KeybindingEditorDiagnosticCode::EmptyCapture,
                ..
            })
        ));
        assert!(matches!(
            malformed.diagnostic,
            Some(KeybindingEditorDiagnostic {
                code: KeybindingEditorDiagnosticCode::MalformedModifier,
                ..
            })
        ));
        assert!(matches!(
            reserved_result.diagnostic,
            Some(KeybindingEditorDiagnostic {
                code: KeybindingEditorDiagnosticCode::ReservedShortcut,
                ..
            })
        ));
        assert!(matches!(
            conflict.diagnostic,
            Some(KeybindingEditorDiagnostic {
                code: KeybindingEditorDiagnosticCode::ShortcutConflict,
                ..
            })
        ));
        assert!(matches!(
            stale.diagnostic,
            Some(KeybindingEditorDiagnostic {
                code: KeybindingEditorDiagnosticCode::UnknownAction,
                ..
            })
        ));
        assert!(matches!(
            disabled.diagnostic,
            Some(KeybindingEditorDiagnostic {
                code: KeybindingEditorDiagnosticCode::DisabledAction,
                ..
            })
        ));
        assert!(matches!(
            unsupported.diagnostic,
            Some(KeybindingEditorDiagnostic {
                code: KeybindingEditorDiagnosticCode::UnsupportedKey,
                ..
            })
        ));
        assert!(empty.overrides.is_empty());
        assert!(conflict.overrides.is_empty());
    }

    #[test]
    fn renderer_editor_projections_share_validation() {
        let overrides = vec![KeybindingOverride::assign("file.save", shortcut("k"))];

        let web = project_keybinding_editor_renderer(
            KeybindingEditorRenderer::Meteorite,
            &registry(),
            &[],
            &overrides,
            &[],
            EMPTY_QUERY,
        );
        let tui = project_keybinding_editor_renderer(
            KeybindingEditorRenderer::Subwayrat,
            &registry(),
            &[],
            &overrides,
            &[],
            EMPTY_QUERY,
        );

        assert_eq!(web.model, tui.model);
    }
}
