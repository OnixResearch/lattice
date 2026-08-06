use super::*;

const ACTION_OPEN: &str = "file.open";
const ACTION_SAVE: &str = "file.save";

#[test]
fn default_shell_starts_with_stable_regions() {
    let model = build_ide_shell(&IdeShellConfig::default(), &IdeAdapterSet::all_available(), vec![default_action((
        ACTION_OPEN,
        "Open",
    ))]);

    assert!(model.is_valid());
    assert_eq!(model.regions.navigation.width, NAVIGATION_WIDTH);
    assert_eq!(model.regions.auxiliary.width, AUXILIARY_WIDTH);
    assert!(model.regions.main.width >= MIN_MAIN_WIDTH);
    assert_eq!(model.regions.status.height, STATUS_HEIGHT);
}

#[test]
fn one_action_projects_to_all_command_surfaces() {
    let model = build_ide_shell(&IdeShellConfig::default(), &IdeAdapterSet::all_available(), vec![default_action((
        ACTION_SAVE,
        "Save",
    ))]);

    assert_eq!(model.commands.keymap[0].action_id, ACTION_SAVE);
    assert_eq!(model.commands.leader[0].action_id, ACTION_SAVE);
    assert_eq!(model.commands.palette[0].action_id, ACTION_SAVE);
    assert_eq!(model.commands.menu[0].action_id, ACTION_SAVE);
    assert_eq!(model.commands.help.get(ACTION_SAVE), Some(&"Ctrl+s".to_string()));
}

#[test]
fn missing_adapters_and_bad_layout_report_diagnostics() {
    let config = IdeShellConfig {
        width: 20,
        height: 6,
        persisted_layout_version: Some(99),
        ..IdeShellConfig::default()
    };
    let adapters = IdeAdapterSet {
        project_tree: false,
        buffers: false,
        diagnostics: true,
        preview: true,
        tasks: true,
    };

    let model = build_ide_shell(&config, &adapters, Vec::new());

    assert_issue(&model, IdeIssueCode::MissingAdapter);
    assert_issue(&model, IdeIssueCode::LayoutRestoreFailed);
    assert_issue(&model, IdeIssueCode::LayoutTooSmall);
}

#[test]
fn shortcut_conflicts_project_as_action_issues() {
    let actions = vec![
        default_action((ACTION_OPEN, "Open")),
        default_action((ACTION_SAVE, "Save")),
    ];

    let model = build_ide_shell(&IdeShellConfig::default(), &IdeAdapterSet::all_available(), actions);

    assert_issue(&model, IdeIssueCode::ActionRegistryIssue);
}

fn default_action(action: (&str, &str)) -> crate::action_registry::ActionDescriptor {
    let (id, label) = action;
    crate::action_registry::ActionDescriptor::new(id, label, crate::action_registry::ActionSource::new("builtin"))
        .with_category("file")
        .with_shortcut(crate::action_registry::ActionShortcut::global("s", vec!["Ctrl".to_string()]))
}

fn assert_issue(model: &IdeShellModel, code: IdeIssueCode) {
    assert!(model.issues.iter().any(|issue| issue.code == code));
}
