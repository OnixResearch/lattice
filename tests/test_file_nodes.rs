//! Tests for file operation nodes.

use std::collections::HashMap;

use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::engine::value::Value;
use n8nclone::engine::workflow::NodeConfig;
use n8nclone::engine::workflow::Workflow;
use n8nclone::file_node::FileError;
use n8nclone::file_node::FileOperationKind;
use n8nclone::file_node::FileOperationPlan;
use n8nclone::file_node::FileOperationResult;
use n8nclone::file_node::FileSystem;
use n8nclone::file_node::MemoryFileSystem;
use n8nclone::file_node::validate_path_allowed;
use n8nclone::workflow_validation::WorkflowDiagnosticCode;
use n8nclone::workflow_validation::validate_workflow;
use pretty_assertions::assert_eq;

const READ_NODE_LABEL: &str = "read-file";
const WRITE_NODE_LABEL: &str = "write-file";
const APPEND_NODE_LABEL: &str = "append-file";
const PATH_ALLOWED: &str = "/allowed/data.txt";
const PATH_OTHER: &str = "/other/data.txt";
const ROOT_ALLOWED: &str = "/allowed";
const INITIAL_CONTENT: &str = "initial";
const WRITE_CONTENT: &str = "written";
const APPEND_CONTENT: &str = " appended";
const LOCAL_FILE_NAME: &str = "local-data.txt";
const CONTENT_INPUT_INDEX: usize = 0;
const CONTENT_OUTPUT_INDEX: usize = 0;
const BYTES_OUTPUT_INDEX: usize = 2;

fn memory_fs() -> MemoryFileSystem {
    MemoryFileSystem::new([(PATH_ALLOWED.to_string(), INITIAL_CONTENT.to_string())].into())
}

// r[verify n8nclone.workflow_execution.file_nodes.config]
#[test]
fn add_file_node_records_config_and_ports() {
    let mut workflow = Workflow::new("file-workflow");
    let node_id = workflow.add_file_node(READ_NODE_LABEL, FileOperationKind::Read, PATH_ALLOWED);
    let node = workflow.graph.node(node_id).expect("file node exists");

    assert_eq!(node.input_ports.len(), 1);
    assert_eq!(node.output_ports.len(), BYTES_OUTPUT_INDEX + 1);
    match workflow.node_config(node_id).expect("file config exists") {
        NodeConfig::File { operation, path, .. } => {
            assert_eq!(*operation, FileOperationKind::Read);
            assert_eq!(path, PATH_ALLOWED);
        }
        other => panic!("expected file config, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.file_nodes.interface]
// r[verify n8nclone.workflow_execution.file_nodes.read]
#[tokio::test]
async fn read_file_node_reads_content_through_interface() {
    let mut workflow = Workflow::new("read-workflow");
    let node_id = workflow.add_file_node(READ_NODE_LABEL, FileOperationKind::Read, PATH_ALLOWED);
    let executor = Executor::new_with_file_system(Box::new(memory_fs()));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let node = workflow.graph.node(node_id).expect("file node exists");
            let content = outputs
                .get(&node.output_ports[CONTENT_OUTPUT_INDEX].id)
                .expect("content output")
                .as_display_string();
            assert_eq!(content, INITIAL_CONTENT);
        }
        other => panic!("expected completed file node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.file_nodes.read]
#[tokio::test]
async fn missing_file_marks_node_failed() {
    let mut workflow = Workflow::new("missing-read");
    let node_id = workflow.add_file_node(READ_NODE_LABEL, FileOperationKind::Read, PATH_OTHER);
    let executor = Executor::new_with_file_system(Box::new(memory_fs()));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("missing file"), "error: {error}"),
        other => panic!("expected failed file node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.file_nodes.write_append]
#[tokio::test]
async fn write_file_node_writes_content_and_reports_bytes() {
    let fs = MemoryFileSystem::new(HashMap::new());
    let mut workflow = Workflow::new("write-workflow");
    let node_id = workflow.add_file_node(WRITE_NODE_LABEL, FileOperationKind::Write, PATH_ALLOWED);
    workflow.set_node_config(node_id, NodeConfig::File {
        operation: FileOperationKind::Write,
        path: PATH_ALLOWED.to_string(),
        content: Some(WRITE_CONTENT.to_string()),
        overwrite: false,
        allowed_roots: vec![ROOT_ALLOWED.to_string()],
    });
    let executor = Executor::new_with_file_system(Box::new(fs.clone()));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            assert_eq!(fs.read_text(PATH_ALLOWED).as_deref(), Some(WRITE_CONTENT));
            let node = workflow.graph.node(node_id).expect("file node exists");
            let bytes =
                outputs.get(&node.output_ports[BYTES_OUTPUT_INDEX].id).expect("bytes output").as_display_string();
            assert_eq!(bytes, WRITE_CONTENT.len().to_string());
        }
        other => panic!("expected completed file node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.file_nodes.write_append]
#[tokio::test]
async fn append_file_node_appends_input_content() {
    let fs = memory_fs();
    let mut workflow = Workflow::new("append-workflow");
    let node_id = workflow.add_file_node(APPEND_NODE_LABEL, FileOperationKind::Append, PATH_ALLOWED);
    let node = workflow.graph.node(node_id).expect("file node exists");
    let content_input = node.input_ports[CONTENT_INPUT_INDEX].id;
    let inputs = [(content_input, Value::from_string(APPEND_CONTENT.to_string()))].into();
    let executor = Executor::new_with_file_system(Box::new(fs.clone()));

    let result = executor.execute_node(&workflow, node_id, &inputs).await;

    assert!(matches!(result.status, NodeStatus::Completed { .. }));
    assert_eq!(fs.read_text(PATH_ALLOWED).as_deref(), Some("initial appended"));
}

// r[verify n8nclone.workflow_execution.file_nodes.safety]
#[test]
fn path_outside_allowed_roots_is_validation_diagnostic() {
    let mut workflow = Workflow::new("unsafe-file");
    let node_id = workflow.add_file_node(READ_NODE_LABEL, FileOperationKind::Read, PATH_OTHER);
    workflow.set_node_config(node_id, NodeConfig::File {
        operation: FileOperationKind::Read,
        path: PATH_OTHER.to_string(),
        content: None,
        overwrite: false,
        allowed_roots: vec![ROOT_ALLOWED.to_string()],
    });

    let diagnostics = validate_workflow(&workflow);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::UnsafeFilePath));
}

// r[verify n8nclone.workflow_execution.file_nodes.safety]
#[tokio::test]
async fn write_without_overwrite_fails_without_modifying_existing_file() {
    let fs = memory_fs();
    let mut workflow = Workflow::new("overwrite-file");
    let node_id = workflow.add_file_node(WRITE_NODE_LABEL, FileOperationKind::Write, PATH_ALLOWED);
    workflow.set_node_config(node_id, NodeConfig::File {
        operation: FileOperationKind::Write,
        path: PATH_ALLOWED.to_string(),
        content: Some(WRITE_CONTENT.to_string()),
        overwrite: false,
        allowed_roots: vec![ROOT_ALLOWED.to_string()],
    });
    let executor = Executor::new_with_file_system(Box::new(fs.clone()));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("already exists"), "error: {error}"),
        other => panic!("expected failed file node, got {other:?}"),
    }
    assert_eq!(fs.read_text(PATH_ALLOWED).as_deref(), Some(INITIAL_CONTENT));
}

// r[verify n8nclone.workflow_execution.llm_operating_graph.scenario.production-file-effects]
#[tokio::test]
async fn default_executor_reads_local_file_within_allowed_root() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join(LOCAL_FILE_NAME);
    std::fs::write(&path, INITIAL_CONTENT).expect("write local fixture");
    let path = path.to_string_lossy().into_owned();
    let allowed_root = directory.path().to_string_lossy().into_owned();
    let mut workflow = Workflow::new("local-read");
    let node_id = workflow.add_file_node(READ_NODE_LABEL, FileOperationKind::Read, &path);
    workflow.set_node_config(node_id, NodeConfig::File {
        operation: FileOperationKind::Read,
        path,
        content: None,
        overwrite: false,
        allowed_roots: vec![allowed_root],
    });
    let executor = Executor::new();

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    assert!(matches!(result.status, NodeStatus::Completed { .. }));
}

// r[verify n8nclone.workflow_execution.llm_operating_graph.scenario.production-file-effects]
#[test]
fn local_filesystem_writes_and_appends_within_allowed_root() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join(LOCAL_FILE_NAME).to_string_lossy().into_owned();
    let allowed_roots = vec![directory.path().to_string_lossy().into_owned()];
    let file_system = n8nclone::file_node::LocalFs;
    let write = FileOperationPlan {
        kind: FileOperationKind::Write,
        path: path.clone(),
        content: Some(WRITE_CONTENT.to_string()),
        overwrite: false,
        allowed_roots: allowed_roots.clone(),
    };
    let append = FileOperationPlan {
        kind: FileOperationKind::Append,
        path: path.clone(),
        content: Some(APPEND_CONTENT.to_string()),
        overwrite: false,
        allowed_roots,
    };

    assert_eq!(
        file_system.execute(&write),
        Ok(FileOperationResult::Written {
            bytes: WRITE_CONTENT.len()
        })
    );
    assert_eq!(
        file_system.execute(&append),
        Ok(FileOperationResult::Written {
            bytes: APPEND_CONTENT.len()
        })
    );
    assert_eq!(std::fs::read_to_string(path).expect("read local result"), "written appended");
}

// r[verify n8nclone.workflow_execution.file_nodes.safety]
#[test]
fn path_prefix_sibling_is_not_inside_allowed_root() {
    let roots = vec![ROOT_ALLOWED.to_string()];

    assert_eq!(
        validate_path_allowed("/allowed-other/data.txt", &roots),
        Err(FileError::UnsafePath("/allowed-other/data.txt".to_string()))
    );
}

// r[verify n8nclone.workflow_execution.llm_operating_graph.scenario.production-file-effects]
#[cfg(unix)]
#[test]
fn local_filesystem_rejects_symlink_escape() {
    let allowed = tempfile::tempdir().expect("allowed temporary directory");
    let outside = tempfile::tempdir().expect("outside temporary directory");
    let outside_path = outside.path().join(LOCAL_FILE_NAME);
    std::fs::write(&outside_path, INITIAL_CONTENT).expect("write outside fixture");
    let link_path = allowed.path().join(LOCAL_FILE_NAME);
    std::os::unix::fs::symlink(&outside_path, &link_path).expect("create escaping symlink");
    let plan = FileOperationPlan {
        kind: FileOperationKind::Read,
        path: link_path.to_string_lossy().into_owned(),
        content: None,
        overwrite: false,
        allowed_roots: vec![allowed.path().to_string_lossy().into_owned()],
    };

    assert!(matches!(n8nclone::file_node::LocalFs.execute(&plan), Err(FileError::UnsafePath(_))));
}
