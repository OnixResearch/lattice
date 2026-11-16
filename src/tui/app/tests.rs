use rat_streaming::streaming_output::DisplayLine;
use tempfile::TempDir;

use super::*;
use crate::agent_node::Adapter;
use crate::agent_node::Error;
use crate::agent_node::ExecutionContext as AgentExecutionContext;
use crate::agent_node::Request;
use crate::agent_node::Response;
use crate::config::PortTemplate;
use crate::config::TemplateParameter;
use crate::config::TemplateParameterDefault;
use crate::config::TemplateParameterType;
use crate::engine::value::ContentHash;
use crate::engine::value::ImageMetadata;
use crate::engine::value::ImageValue;
use crate::image_node::MemoryImageIo;

const DEFAULT_SHELL_TIMEOUT_SECS: u64 = 60;

const RUN_EVENT_POLL_LIMIT: usize = 50;

const RUN_EVENT_POLL_DELAY_MS: u64 = 10;

const SHELL_STDOUT_OUTPUT_INDEX: usize = 0;

const SHELL_STDERR_OUTPUT_INDEX: usize = 1;

const SHELL_EXIT_OUTPUT_INDEX: usize = 2;

const AGENT_IMAGE_OUTPUT_INDEX: usize = 2;

const EXIT_CODE_SUCCESS: i32 = 0;

const EXIT_CODE_GREP_NO_MATCH: i32 = 1;

const SNAPSHOT_RUN_ID: u64 = 0;

const SNAPSHOT_WORKFLOW_NAME: &str = "snapshot";

const SNAPSHOT_NODE_LABEL: &str = "Snapshot label";

const MUTATED_NODE_LABEL: &str = "Mutated label";

const SNAPSHOT_COMMAND: &str = "echo snapshot";

const SNAPSHOT_STDOUT_TEXT: &str = "snapshot output";

const SNAPSHOT_STDOUT: &str = "snapshot output\n";

const SNAPSHOT_INPUT_HASH_BYTES: &[u8] = b"snapshot-log";

const SNAPSHOT_OUTPUT_RECORD_COUNT: usize = 2;

const SNAPSHOT_SECOND_RUN_ID: u64 = 1;

const OLDER_STDOUT_TEXT: &str = "older output";

const OTHER_NODE_TEXT: &str = "other node output";

const TEMPLATE_PARAMETER_COUNT: usize = 1;

const TEMPLATE_FIRST_PARAMETER_INDEX: usize = 0;

const TEMPLATE_NUMBER_DEFAULT: f64 = 10.0;

const TEMPLATE_NUMBER_MIN: f64 = 1.0;

const TEMPLATE_NUMBER_MAX: f64 = 10.0;

const IMAGE_PREVIEW_BYTES: &[u8] = b"raw-image-bytes";

const TUI_LOCAL_IMAGE_FILE_NAME: &str = "tui-selfie.png";

const TUI_LOCAL_PNG_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\ntui";

const TUI_MISSING_IMAGE_PATH: &str = "/tui/missing.png";

const TUI_BAD_IMAGE_PATH: &str = "/tui/bad.png";

const TUI_BAD_IMAGE_BYTES: &[u8] = b"not an image";

const RUN_DETAIL_STORAGE_ERROR: &str = "storage offline";

#[derive(Debug)]
struct TuiErrorAgentAdapter {
    error: Error,
}

impl Adapter for TuiErrorAgentAdapter {
    fn execution_identity(&self, _request: &Request) -> Result<String, Error> {
        Ok("test/tui-error-agent/v1".to_owned())
    }
    fn execute(&self, _request: &Request, _context: &AgentExecutionContext) -> Result<Response, Error> {
        Err(self.error.clone())
    }
}

fn test_config(temp_dir: &TempDir) -> AppConfig {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("create app data directory");
    AppConfig {
        data_dir,
        ..AppConfig::default()
    }
}

/// Select exact local-operator authority for one test workflow without
/// weakening the production fail-closed default.
fn operator_authority(
    workflow: &Workflow,
    root: &std::path::Path,
) -> crate::engine::effect::standalone::OperatorRunContextSource {
    const EXPIRY_UNIX_MS: u64 = 4_102_444_800_000;
    let inspection =
        crate::engine::effect::standalone::inspect_workflow(workflow).expect("workflow requirements inspect");
    let profile = crate::engine::effect::standalone::core::Profile {
        schema: inspection.schema,
        workflow_revision: inspection.workflow_revision,
        machine_identity: inspection.machine_identity,
        policy_ref: inspection.policy_ref,
        expires_at_unix_ms: EXPIRY_UNIX_MS,
        grants: inspection.grants,
        roots: std::collections::BTreeMap::new(),
    };
    let profile_path = root.join("authority.ncl");
    std::fs::write(root.join("authority.json"), serde_json::to_vec(&profile).expect("authority profile serializes"))
        .expect("authority profile writes");
    std::fs::write(&profile_path, "import \"authority.json\"\n").expect("authority source writes");
    crate::engine::effect::standalone::OperatorRunContextSource::new(profile_path)
}

fn output_text_lines(app: &State) -> Vec<String> {
    (0..app.output.display_line_count())
        .filter_map(|line_index| match app.output.get_display_line(line_index) {
            DisplayLine::Text(line) => Some(line),
            DisplayLine::Omitted(_) => None,
        })
        .collect()
}

fn image_executor_factory(files: HashMap<String, Vec<u8>>) -> Arc<dyn Fn() -> Executor + Send + Sync> {
    Arc::new(move || Executor::new_with_image_io(Box::new(MemoryImageIo::new(files.clone()))))
}

fn agent_error_executor_factory(error: Error) -> Arc<dyn Fn() -> Executor + Send + Sync> {
    Arc::new(move || Executor::new_with_agent_adapter(Box::new(TuiErrorAgentAdapter { error: error.clone() })))
}

async fn drain_run(app: &mut State) {
    for _ in 0..RUN_EVENT_POLL_LIMIT {
        app.poll_run_events();
        if !app.run_state.active {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(RUN_EVENT_POLL_DELAY_MS)).await;
    }
}

fn parameterized_grep_template() -> NodeTemplate {
    NodeTemplate {
        name: "grep".to_string(),
        description: "Filter text".to_string(),
        kind: "shell".to_string(),
        command: Some("grep -E '${pattern}'".to_string()),
        parameters: vec![TemplateParameter {
            name: "pattern".to_string(),
            type_tag: TemplateParameterType::String,
            description: "Search pattern".to_string(),
            default: None,
            required: true,
            min: None,
            max: None,
        }],
        input_ports: vec![PortTemplate {
            label: "stdin".to_string(),
            type_tag: "string".to_string(),
        }],
        output_ports: vec![PortTemplate {
            label: "stdout".to_string(),
            type_tag: "string".to_string(),
        }],
    }
}

fn rendered_output_records(records: Vec<RunOutputLine>) -> Vec<String> {
    records.into_iter().map(|record| record.rendered()).collect()
}

mod lifecycle;

mod property_transactions;

mod guided_nodes;

mod observed_outputs;

mod background_runs;

mod durable_history;
