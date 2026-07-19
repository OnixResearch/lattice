//! Tests for complex image workflow examples.

use std::collections::HashMap;
use std::path::Path;

use n8nclone::agent_node::AgentAdapter;
use n8nclone::agent_node::AgentError;
use n8nclone::agent_node::AgentOutput;
use n8nclone::agent_node::AgentResponse;
use n8nclone::agent_node::ExecutionContext as AgentExecutionContext;
use n8nclone::credentials::EmptyCredentialProvider;
use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::engine::value::ImageMetadata;
use n8nclone::engine::value::ImageValue;
use n8nclone::file_node::NoopFileSystem;
use n8nclone::http::NoopHttpClient;
use n8nclone::image_node::MemoryImageIo;
use n8nclone::workflow_format::load_workflow_nickel_file;
use pretty_assertions::assert_eq;

const SELFIE_TO_FINE_ART_EXAMPLE: &str = "examples/workflows/selfie-to-fine-art.ncl";
const SELFIE_WORKFLOW_NAME: &str = "selfie-to-fine-art";
const SELFIE_PATH: &str = "/fixtures/selfie.png";
const PORTRAIT_PATH: &str = "/fixtures/inspiration-portrait.jpg";
const LIGHTING_PATH: &str = "/fixtures/inspiration-lighting.png";
const PNG_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\nexample";
const JPEG_FIXTURE: &[u8] = b"\xff\xd8example\xff\xd9";
const GENERATED_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\ngenerated";
const EXPECTED_IMAGE_INPUT_COUNT: usize = 3;
const EXPECTED_COMPLETED_NODE_COUNT: usize = 4;

#[derive(Debug)]
struct ExampleAgentAdapter;

impl AgentAdapter for ExampleAgentAdapter {
    fn execute(
        &self,
        request: &n8nclone::agent_node::AgentRequest,
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

// r[verify n8nclone.workflow_examples.image]
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
