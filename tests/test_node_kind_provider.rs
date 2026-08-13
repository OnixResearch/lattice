//! Tests for built-in node-kind provider metadata and validation.

use lattice::config::AppConfig;
use lattice::engine::workflow::NodeConfig;
use lattice::file_node::FileOperationKind;
use lattice::node_kind_provider::NodeKindValidationCode;
use lattice::node_kind_provider::property_lines_for_config;
use lattice::node_kind_provider::provider_for_config;
use lattice::node_kind_provider::provider_for_portable_kind;
use lattice::node_kind_provider::validate_config_for_kind;
use lattice::tui::App;
use lattice::tui::app::PropertyEditField;

const NODE_LABEL: &str = "node";
const SHELL_KIND: &str = "shell";
const UNSUPPORTED_KIND: &str = "unsupported";
const SHELL_COMMAND: &str = "printf ok";
const EMPTY_COMMAND: &str = "   ";
const COMMAND_LABEL: &str = "Cmd";
const TYPE_LABEL: &str = "Type";
const SHELL_NODE_LABEL: &str = "Shell";
const PRODUCT_ROOT_ID: &str = "operator-root";
const PRODUCT_RELATIVE_PATH: &str = "nested/item.txt";

fn test_config(temp_dir: &tempfile::TempDir) -> AppConfig {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("create app data directory");
    AppConfig {
        data_dir,
        ..AppConfig::default()
    }
}

// r[verify n8nclone.tui_ux.node_kind_provider.property_metadata]
// r[verify n8nclone.tui_ux.node_kind_provider]
#[test]
fn provider_property_metadata_lists_shell_type_and_command() {
    let config = NodeConfig::shell(SHELL_COMMAND);

    let lines = property_lines_for_config(&config);

    assert!(lines.iter().any(|line| line.label == TYPE_LABEL && line.value == "Shell"));
    assert!(lines.iter().any(|line| line.label == COMMAND_LABEL && line.value == SHELL_COMMAND));
}

// r[verify n8nclone.workflow_validation.node_kind_provider.config_validation]
#[test]
fn provider_validation_accepts_valid_shell_and_rejects_empty_command() {
    let valid = NodeConfig::shell(SHELL_COMMAND);
    let invalid = NodeConfig::shell(EMPTY_COMMAND);

    let valid_issues = validate_config_for_kind(NODE_LABEL, &valid);
    let invalid_issues = validate_config_for_kind(NODE_LABEL, &invalid);

    assert!(valid_issues.is_empty());
    assert!(invalid_issues.iter().any(|issue| issue.code == NodeKindValidationCode::EmptyShellCommand));
}

#[test]
fn provider_validation_accepts_product_root_ids_and_rejects_unsafe_relative_paths() {
    let valid = NodeConfig::File {
        operation: FileOperationKind::Read,
        path: PRODUCT_RELATIVE_PATH.to_owned(),
        content: None,
        overwrite: false,
        allowed_roots: vec![PRODUCT_ROOT_ID.to_owned()],
    };
    let invalid = NodeConfig::File {
        operation: FileOperationKind::Read,
        path: "../escape.txt".to_owned(),
        content: None,
        overwrite: false,
        allowed_roots: vec![PRODUCT_ROOT_ID.to_owned()],
    };

    assert!(validate_config_for_kind(NODE_LABEL, &valid).is_empty());
    assert!(
        validate_config_for_kind(NODE_LABEL, &invalid)
            .iter()
            .any(|issue| issue.code == NodeKindValidationCode::UnsafeFilePath)
    );
}

// r[verify n8nclone.workflow_format.node_kind_provider]
// r[verify n8nclone.workflow_format.node_kind_provider.export]
// r[verify n8nclone.workflow_format.node_kind_provider.import]
#[test]
fn provider_registry_resolves_supported_and_rejects_unknown_kinds() {
    let config = NodeConfig::shell(SHELL_COMMAND);

    let provider = provider_for_config(&config);
    let imported = provider_for_portable_kind(SHELL_KIND);
    let unsupported = provider_for_portable_kind(UNSUPPORTED_KIND);

    assert_eq!(provider.portable_kind(), SHELL_KIND);
    assert!(imported.is_some());
    assert!(unsupported.is_none());
}

// r[verify n8nclone.tui_ux.node_kind_provider.unsupported_edit]
#[test]
fn unsupported_property_edit_does_not_mutate_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));
    app.add_shell_node(SHELL_NODE_LABEL, SHELL_COMMAND);
    app.clear_workflow_dirty();

    let started = app.start_property_edit(PropertyEditField::ImagePath);

    assert!(!started);
    assert!(app.property_edit.is_none());
    assert!(!app.has_unsaved_changes());
}
