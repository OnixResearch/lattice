//! Tests for image input operation nodes.

use std::collections::HashMap;

use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::engine::value::TypeTag;
use n8nclone::engine::workflow::NodeConfig;
use n8nclone::engine::workflow::Workflow;
use n8nclone::image_node::MemoryImageIo;
use n8nclone::workflow_validation::WorkflowDiagnosticCode;
use n8nclone::workflow_validation::validate_workflow;
use pretty_assertions::assert_eq;

const IMAGE_NODE_LABEL: &str = "selfie";
const PATH_ALLOWED_PNG: &str = "/allowed/selfie.png";
const PATH_ALLOWED_JPEG: &str = "/allowed/selfie.jpg";
const PATH_OTHER: &str = "/other/selfie.png";
const PATH_MISSING: &str = "/allowed/missing.png";
const ROOT_ALLOWED: &str = "/allowed";
const IMAGE_OUTPUT_INDEX: usize = 0;
const METADATA_OUTPUT_INDEX: usize = 1;
const TINY_IMAGE_BYTE_BUDGET: usize = 4;
const PNG_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\nfixture";
const JPEG_FIXTURE: &[u8] = b"\xff\xd8fixture\xff\xd9";
const BAD_IMAGE_FIXTURE: &[u8] = b"not an image";

fn image_io() -> MemoryImageIo {
    MemoryImageIo::new(
        [
            (PATH_ALLOWED_PNG.to_string(), PNG_FIXTURE.to_vec()),
            (PATH_ALLOWED_JPEG.to_string(), JPEG_FIXTURE.to_vec()),
            (PATH_OTHER.to_string(), PNG_FIXTURE.to_vec()),
        ]
        .into(),
    )
}

// r[verify n8nclone.workflow_execution.image_input_nodes.load]
#[test]
fn add_image_input_node_records_config_and_ports() {
    let mut workflow = Workflow::new("image-workflow");
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, PATH_ALLOWED_PNG);
    let node = workflow.graph.node(node_id).expect("image node exists");

    assert!(node.input_ports.is_empty());
    assert_eq!(node.output_ports.len(), METADATA_OUTPUT_INDEX + 1);
    assert_eq!(node.output_ports[IMAGE_OUTPUT_INDEX].type_tag, TypeTag::Image.as_str());
    match workflow.node_config(node_id).expect("image config exists") {
        NodeConfig::ImageInput { path, .. } => assert_eq!(path, PATH_ALLOWED_PNG),
        other => panic!("expected image input config, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.image_input_nodes.load]
#[tokio::test]
async fn image_input_node_loads_png_through_interface() {
    let mut workflow = Workflow::new("image-workflow");
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, PATH_ALLOWED_PNG);
    let executor = Executor::new_with_image_io(Box::new(image_io()));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let node = workflow.graph.node(node_id).expect("image node exists");
            let image = outputs.get(&node.output_ports[IMAGE_OUTPUT_INDEX].id).expect("image output");
            assert_eq!(image.type_tag(), TypeTag::Image);
            assert!(image.as_display_string().contains("image/png"));
            let metadata = outputs.get(&node.output_ports[METADATA_OUTPUT_INDEX].id).expect("metadata output");
            assert!(metadata.as_display_string().contains(PATH_ALLOWED_PNG));
        }
        other => panic!("expected completed image node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.image_input_nodes.load]
#[tokio::test]
async fn image_input_node_loads_jpeg_through_interface() {
    let mut workflow = Workflow::new("image-workflow");
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, PATH_ALLOWED_JPEG);
    let executor = Executor::new_with_image_io(Box::new(image_io()));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let node = workflow.graph.node(node_id).expect("image node exists");
            let image = outputs.get(&node.output_ports[IMAGE_OUTPUT_INDEX].id).expect("image output");
            assert!(image.as_display_string().contains("image/jpeg"));
        }
        other => panic!("expected completed image node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.image_input_nodes.load]
#[tokio::test]
async fn unsupported_image_format_marks_node_failed() {
    let io = MemoryImageIo::new([(PATH_ALLOWED_PNG.to_string(), BAD_IMAGE_FIXTURE.to_vec())].into());
    let mut workflow = Workflow::new("bad-image-workflow");
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, PATH_ALLOWED_PNG);
    let executor = Executor::new_with_image_io(Box::new(io));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("unsupported image format"), "error: {error}"),
        other => panic!("expected failed image node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.image_input_nodes.load]
#[tokio::test]
async fn missing_image_marks_node_failed() {
    let mut workflow = Workflow::new("missing-image-workflow");
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, PATH_MISSING);
    let executor = Executor::new_with_image_io(Box::new(image_io()));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("missing image"), "error: {error}"),
        other => panic!("expected failed image node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.image_input_nodes.load]
#[tokio::test]
async fn oversized_image_marks_node_failed() {
    let mut workflow = Workflow::new("oversized-image-workflow");
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, PATH_ALLOWED_PNG);
    workflow.set_node_config(node_id, NodeConfig::ImageInput {
        path: PATH_ALLOWED_PNG.to_string(),
        allowed_roots: vec![ROOT_ALLOWED.to_string()],
        max_bytes: TINY_IMAGE_BYTE_BUDGET,
    });
    let executor = Executor::new_with_image_io(Box::new(image_io()));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("too large"), "error: {error}"),
        other => panic!("expected failed image node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.image_input_nodes.load]
#[test]
fn path_outside_allowed_roots_is_validation_diagnostic() {
    let mut workflow = Workflow::new("unsafe-image-workflow");
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, PATH_OTHER);
    workflow.set_node_config(node_id, NodeConfig::ImageInput {
        path: PATH_OTHER.to_string(),
        allowed_roots: vec![ROOT_ALLOWED.to_string()],
        max_bytes: n8nclone::image_node::DEFAULT_MAX_IMAGE_BYTES,
    });

    let diagnostics = validate_workflow(&workflow);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::UnsafeImagePath));
}
