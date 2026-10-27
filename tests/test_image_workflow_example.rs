//! Tests for complex image workflow examples.

use std::collections::HashMap;
use std::path::Path;

use lattice::agent_node::AgentAdapter;
use lattice::agent_node::AgentError;
use lattice::agent_node::AgentOutput;
use lattice::agent_node::AgentResponse;
use lattice::agent_node::ExecutionContext as AgentExecutionContext;
use lattice::credentials::EmptyCredentialProvider;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::value::ImageMetadata;
use lattice::engine::value::ImageValue;
use lattice::file_node::NoopFileSystem;
use lattice::http::NoopHttpClient;
use lattice::image_node::MemoryImageIo;
use lattice::workflow_format::load_workflow_nickel_file;
use pretty_assertions::assert_eq;

const SELFIE_TO_FINE_ART_EXAMPLE: &str = "examples/workflows/selfie-to-fine-art.ncl";
const INVALID_IMAGE_EXAMPLE: &str = "examples/workflows/invalid-unsupported-image-format.ncl";
const SELFIE_WORKFLOW_NAME: &str = "selfie-to-fine-art";
const INVALID_WORKFLOW_NAME: &str = "invalid-unsupported-image-format";
const SELFIE_PATH: &str = "/fixtures/selfie.png";
const PORTRAIT_PATH: &str = "/fixtures/inspiration-portrait.jpg";
const LIGHTING_PATH: &str = "/fixtures/inspiration-lighting.png";
const INVALID_SELFIE_PATH: &str = "/fixtures/unsupported-selfie.gif";
const INVALID_PORTRAIT_PATH: &str = "/fixtures/unsupported-portrait.gif";
const INVALID_LIGHTING_PATH: &str = "/fixtures/unsupported-lighting.gif";
const PNG_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\nexample";
const JPEG_FIXTURE: &[u8] = b"\xff\xd8example\xff\xd9";
const GENERATED_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\ngenerated";
const UNSUPPORTED_GIF_FIXTURE: &[u8] = b"GIF89a-invalid-example";
const EXPECTED_IMAGE_INPUT_COUNT: usize = 3;
const EXPECTED_COMPLETED_NODE_COUNT: usize = 4;

#[derive(Debug)]
struct ExampleAgentAdapter;

impl AgentAdapter for ExampleAgentAdapter {
    fn execution_identity(&self, _request: &lattice::agent_node::AgentRequest) -> Result<String, AgentError> {
        Ok("test/image-agent/v1".to_owned())
    }
    fn execute(
        &self,
        request: &lattice::agent_node::AgentRequest,
        _context: &AgentExecutionContext,
    ) -> Result<AgentResponse, AgentError> {
        assert_eq!(request.image_inputs.len(), EXPECTED_IMAGE_INPUT_COUNT);
        Ok(AgentResponse {
            output: AgentOutput::Image(ImageValue::new(
                GENERATED_FIXTURE.to_vec(),
                "image/png".to_string(),
                ImageMetadata::default(),
            )),
            metadata: serde_json::json!({"adapter": "fake-clankers"}),
        })
    }
}

fn example_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SELFIE_TO_FINE_ART_EXAMPLE)
}

fn invalid_example_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(INVALID_IMAGE_EXAMPLE)
}

fn image_io() -> MemoryImageIo {
    MemoryImageIo::new(
        [
            (SELFIE_PATH.to_string(), PNG_FIXTURE.to_vec()),
            (PORTRAIT_PATH.to_string(), JPEG_FIXTURE.to_vec()),
            (LIGHTING_PATH.to_string(), PNG_FIXTURE.to_vec()),
        ]
        .into(),
    )
}

fn invalid_image_io() -> MemoryImageIo {
    MemoryImageIo::new(
        [
            (INVALID_SELFIE_PATH.to_string(), UNSUPPORTED_GIF_FIXTURE.to_vec()),
            (INVALID_PORTRAIT_PATH.to_string(), UNSUPPORTED_GIF_FIXTURE.to_vec()),
            (INVALID_LIGHTING_PATH.to_string(), UNSUPPORTED_GIF_FIXTURE.to_vec()),
        ]
        .into(),
    )
}

// r[verify n8nclone.workflow_examples.image]
// r[verify n8nclone.workflow_examples.image.selfie_to_fine_art]
#[tokio::test]
async fn selfie_to_fine_art_example_imports_and_runs_with_fake_clankers_adapter() {
    let workflow = load_workflow_nickel_file(&example_path()).expect("example imports");
    assert_eq!(workflow.meta.name, SELFIE_WORKFLOW_NAME);
    let mut executor = Executor::new_with_runtime_clients(
        Box::<EmptyCredentialProvider>::default(),
        Box::<NoopHttpClient>::default(),
        Box::<NoopFileSystem>::default(),
        Box::new(image_io()),
        Box::new(ExampleAgentAdapter),
    );

    let (_run_id, results) = executor.run(&workflow, &HashMap::new()).await;

    assert_eq!(results.len(), EXPECTED_COMPLETED_NODE_COUNT);
    assert!(results.iter().all(|step| matches!(step.status, NodeStatus::Completed { .. })));
    let generated = results
        .iter()
        .flat_map(|step| match &step.status {
            NodeStatus::Completed { outputs, .. } => outputs.values().collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .any(|value| value.as_display_string().contains("image/png"));
    assert!(generated);
}

// r[verify n8nclone.workflow_examples.image.invalid]
#[tokio::test]
async fn invalid_image_example_reports_format_diagnostics_without_partial_completion() {
    let workflow = load_workflow_nickel_file(&invalid_example_path()).expect("negative example imports");
    assert_eq!(workflow.meta.name, INVALID_WORKFLOW_NAME);
    let mut executor = Executor::new_with_runtime_clients(
        Box::<EmptyCredentialProvider>::default(),
        Box::<NoopHttpClient>::default(),
        Box::<NoopFileSystem>::default(),
        Box::new(invalid_image_io()),
        Box::new(ExampleAgentAdapter),
    );

    let (_run_id, results) = executor.run(&workflow, &HashMap::new()).await;

    assert_eq!(results.len(), EXPECTED_COMPLETED_NODE_COUNT);
    assert!(results.iter().all(|step| matches!(step.status, NodeStatus::Failed { .. } | NodeStatus::Skipped)));
    assert!(results.iter().any(|step| {
        matches!(&step.status, NodeStatus::Failed { error } if error.contains("unsupported image format"))
    }));
    assert!(!results.iter().any(|step| matches!(step.status, NodeStatus::Completed { .. })));
}
