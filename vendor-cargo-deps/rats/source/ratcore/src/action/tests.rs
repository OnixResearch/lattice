use super::*;

const SAVE_ID: &str = "file.save";
const OPEN_ID: &str = "file.open";
const QUIT_ID: &str = "app.quit";
const SOURCE_NAME: &str = "builtin";
const FILE_CATEGORY: &str = "file";
const APP_CATEGORY: &str = "app";
const SAVE_KEY: &str = "s";
const OPEN_KEY: &str = "o";
const CTRL_MODIFIER: &str = "Ctrl";
const DISABLED_REASON: &str = "workspace is read-only";

fn source() -> ActionSource {
    ActionSource::new(SOURCE_NAME).with_kind("core")
}

fn save_action() -> ActionDescriptor {
    ActionDescriptor::new(SAVE_ID, "Save", source())
        .with_description("Save the current file")
        .with_category(FILE_CATEGORY)
        .with_icon("save")
        .with_shortcut(ActionShortcut::global(SAVE_KEY, vec![CTRL_MODIFIER.to_string()]))
}

fn open_action() -> ActionDescriptor {
    ActionDescriptor::new(OPEN_ID, "Open", source())
        .with_category(FILE_CATEGORY)
        .with_shortcut(ActionShortcut::global(OPEN_KEY, vec![CTRL_MODIFIER.to_string()]))
}

fn quit_action() -> ActionDescriptor {
    ActionDescriptor::new(QUIT_ID, "Quit", source()).with_category(APP_CATEGORY).with_enablement(
        ActionEnablement::Disabled {
            reason: DISABLED_REASON.to_string(),
        },
    )
}

#[test]
fn snapshot_preserves_valid_metadata() {
    let build = build_registry(vec![save_action(), quit_action()]);
    assert!(build.is_valid(), "issues: {:?}", build.issues);
    let save = build.snapshot.action_by_raw_id(SAVE_ID).unwrap();
    assert_eq!(save.label, "Save");
    assert_eq!(save.description.as_deref(), Some("Save the current file"));
    assert_eq!(save.icon.as_deref(), Some("save"));
    assert_eq!(save.category, FILE_CATEGORY);
    assert_eq!(save.source.name, SOURCE_NAME);
    assert_eq!(save.source.kind.as_deref(), Some("core"));
    assert_eq!(save.shortcut.as_ref().map(ActionShortcut::display), Some("Ctrl+s".to_string()));
    let quit = build.snapshot.action_by_raw_id(QUIT_ID).unwrap();
    assert_eq!(quit.enablement.disabled_reason(), Some(DISABLED_REASON));
}

#[test]
fn projections_keep_order_and_enablement() {
    let build = build_registry(vec![save_action(), quit_action(), open_action()]);
    assert!(build.is_valid(), "issues: {:?}", build.issues);
    let palette_items = build.snapshot.command_palette_items();
    assert_eq!(palette_items[0].action_id, QUIT_ID);
    assert!(!palette_items[0].enabled);
    assert_eq!(palette_items[0].disabled_reason.as_deref(), Some(DISABLED_REASON));
    assert_eq!(palette_items[1].action_id, OPEN_ID);
    assert!(palette_items[1].enabled);
    assert_eq!(palette_items[2].action_id, SAVE_ID);
    let bindings = build.snapshot.keymap_bindings();
    assert_eq!(bindings.len(), 2);
    assert!(bindings.iter().any(|binding| binding.action_id == SAVE_ID));
    assert!(bindings.iter().all(|binding| binding.action_id != QUIT_ID));
    let help = build.snapshot.shortcut_help_sections();
    assert_eq!(help.len(), 1);
    assert_eq!(help[0].category, FILE_CATEGORY);
    assert_eq!(help[0].entries.len(), 2);
}

#[test]
fn duplicate_ids_are_reported_and_omitted() {
    let duplicate = ActionDescriptor::new(SAVE_ID, "Save Copy", source());
    let build = build_registry(vec![save_action(), duplicate]);
    assert!(!build.is_valid());
    assert!(build.issues.iter().any(|issue| issue.code == ActionRegistryIssueCode::DuplicateId));
    assert!(build.snapshot.action_by_raw_id(SAVE_ID).is_none());
}

#[test]
fn malformed_descriptor_reports_field_diagnostics() {
    let malformed = ActionDescriptor::new("", "", ActionSource::new(""))
        .with_category("")
        .with_shortcut(ActionShortcut::new("", "", vec![String::new()]));
    let build = build_registry(vec![malformed]);
    assert!(!build.is_valid());
    let codes = build.issues.iter().map(|issue| issue.code).collect::<std::collections::BTreeSet<_>>();
    assert!(codes.contains(&ActionRegistryIssueCode::EmptyId));
    assert!(codes.contains(&ActionRegistryIssueCode::EmptyLabel));
    assert!(codes.contains(&ActionRegistryIssueCode::EmptyCategory));
    assert!(codes.contains(&ActionRegistryIssueCode::EmptySource));
    assert!(codes.contains(&ActionRegistryIssueCode::EmptyShortcutScope));
    assert!(codes.contains(&ActionRegistryIssueCode::EmptyShortcutKey));
    assert!(codes.contains(&ActionRegistryIssueCode::EmptyShortcutModifier));
    assert!(build.snapshot.actions().is_empty());
}

#[test]
fn conflicting_shortcuts_fail_closed() {
    let open_conflict = ActionDescriptor::new(OPEN_ID, "Open", source())
        .with_category(FILE_CATEGORY)
        .with_shortcut(ActionShortcut::global(SAVE_KEY, vec![CTRL_MODIFIER.to_string()]));
    let build = build_registry(vec![save_action(), open_conflict]);
    assert!(!build.is_valid());
    assert!(build.issues.iter().any(|issue| issue.code == ActionRegistryIssueCode::ShortcutConflict));
    assert!(build.snapshot.keymap_bindings().is_empty());
    let menu_items = build.snapshot.menu_items();
    assert!(menu_items.iter().all(|item| !item.enabled));
    assert!(menu_items.iter().all(|item| item.disabled_reason.as_deref() == Some(BLOCKED_CONFLICT_REASON)));
}

#[test]
fn disabled_action_does_not_conflict_with_enabled_shortcut() {
    let disabled_conflict = ActionDescriptor::new(OPEN_ID, "Open", source())
        .with_category(FILE_CATEGORY)
        .with_shortcut(ActionShortcut::global(SAVE_KEY, vec![CTRL_MODIFIER.to_string()]))
        .with_enablement(ActionEnablement::Disabled {
            reason: DISABLED_REASON.to_string(),
        });
    let build = build_registry(vec![save_action(), disabled_conflict]);
    assert!(build.is_valid(), "issues: {:?}", build.issues);
    let bindings = build.snapshot.keymap_bindings();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].action_id, SAVE_ID);
    let disabled_item =
        build.snapshot.command_palette_items().into_iter().find(|item| item.action_id == OPEN_ID).unwrap();
    assert!(!disabled_item.enabled);
    assert_eq!(disabled_item.disabled_reason.as_deref(), Some(DISABLED_REASON));
}
