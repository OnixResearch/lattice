//! Tests for headless workflow runs.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use n8nclone::config::DEFAULT_NODE_TEMPLATE_FILES;
use n8nclone::config::NodeTemplate;
use n8nclone::config::load_node_templates_with_diagnostics;
use n8nclone::config::render_template_command;
use n8nclone::engine::executor::RunId;
use n8nclone::engine::executor::node_execution_cache_key;
use n8nclone::engine::value::TypeTag;
use n8nclone::engine::value::Value;
use n8nclone::engine::value::ValueData;
use n8nclone::engine::workflow::NodeConfig;
use n8nclone::engine::workflow::Workflow;
use n8nclone::headless::HeadlessProcessResult;
use n8nclone::headless::HeadlessRunError;
use n8nclone::headless::HeadlessRunStatus;
use n8nclone::headless::process_result_for_summary;
use n8nclone::headless::render_run_summary_text;
use n8nclone::headless::run_saved_workflow;
use n8nclone::store::Store;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use rat_nodegraph::model::PortSpec;

const SUCCESS_WORKFLOW_NAME: &str = "headless-success";
const MISSING_WORKFLOW_NAME: &str = "missing-workflow";
const BROKEN_WORKFLOW_NAME: &str = "headless-broken";
const IMAGE_WORKFLOW_NAME: &str = "headless-image";
const SUCCESS_NODE_LABEL: &str = "say hello";
const SUCCESS_COMMAND: &str = "printf hello";
const BROKEN_NODE_LABEL: &str = "broken";
const IMAGE_NODE_LABEL: &str = "load image";
const IMAGE_OUTPUT_PORT_LABEL: &str = "image";
const HEADLESS_IMAGE_FILE_NAME: &str = "headless-selfie.png";
const MISSING_CWD: &str = "/definitely/missing/n8nclone/headless-test";
const SUCCESS_RUN_ID: u64 = 7;
const MISSING_RUN_ID: u64 = 11;
const FAILED_RUN_ID: u64 = 13;
const IMAGE_RUN_ID: u64 = 17;
const RECENT_RUN_LIMIT: usize = 8;
const SINGLE_NODE_COUNT: usize = 1;
const NO_FAILURES: usize = 0;
const ONE_FAILURE: usize = 1;
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 4;
const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "n8nclone.redb";
const TEMPLATES_DIR_NAME: &str = "templates";
const TEMPLATE_WORKFLOW_NAME_PREFIX: &str = "headless-template-";
const TEMPLATE_SOURCE_NODE_LABEL: &str = "template stdin";
const TEMPLATE_SOURCE_COMMAND: &str = "printf 'alpha\\nneedle\\nneedle beta\\n'";
const TEMPLATE_STDIN_TEXT: &str = "alpha\nneedle\nneedle beta\n";
const TEMPLATE_STDIN_PORT_LABEL: &str = "stdin";
const TEMPLATE_STDOUT_PORT_LABEL: &str = "stdout";
const TEMPLATE_EXIT_PORT_LABEL: &str = "exit";
const TEMPLATE_GREP_NAME: &str = "grep";
const TEMPLATE_HEAD_NAME: &str = "head";
const TEMPLATE_REPLACE_NAME: &str = "replace";
const TEMPLATE_WORD_COUNT_NAME: &str = "word-count";
const PARAM_PATTERN: &str = "pattern";
const PARAM_REPLACEMENT: &str = "replacement";
const PARAM_NEEDLE_VALUE: &str = "needle";
const PARAM_THREAD_VALUE: &str = "thread";
const GREP_EXPECTED_STDOUT: &str = "needle\nneedle beta\n";
const REPLACE_EXPECTED_STDOUT: &str = "alpha\nthread\nthread beta\n";
const WORD_COUNT_EXPECTED_TOKEN: &str = "3";
const SUCCESS_EXIT_CODE: i32 = 0;
const TEMPLATE_RUN_ID_START: u64 = 100;
const TEMPLATE_WORKFLOW_NODE_COUNT: usize = 2;
const TEMPLATE_WORKFLOW_STEP_COUNT: usize = 2;
const PNG_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\nfixture";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TemplateHeadlessCase {
    name: &'static str,
    values: &'static [(&'static str, &'static str)],
    expected_stdout: ExpectedStdout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedStdout {
    Exact(&'static str),
    ContainsToken(&'static str),
}

const GREP_TEMPLATE_VALUES: &[(&str, &str)] = &[(PARAM_PATTERN, PARAM_NEEDLE_VALUE)];
const REPLACE_TEMPLATE_VALUES: &[(&str, &str)] = &[
    (PARAM_PATTERN, PARAM_NEEDLE_VALUE),
    (PARAM_REPLACEMENT, PARAM_THREAD_VALUE),
];
const DEFAULTED_TEMPLATE_VALUES: &[(&str, &str)] = &[];
const DEFAULT_TEMPLATE_HEADLESS_CASES: &[TemplateHeadlessCase] = &[
    TemplateHeadlessCase {
        name: TEMPLATE_GREP_NAME,
        values: GREP_TEMPLATE_VALUES,
        expected_stdout: ExpectedStdout::Exact(GREP_EXPECTED_STDOUT),
    },
    TemplateHeadlessCase {
        name: TEMPLATE_HEAD_NAME,
        values: DEFAULTED_TEMPLATE_VALUES,
        expected_stdout: ExpectedStdout::Exact(TEMPLATE_STDIN_TEXT),
    },
    TemplateHeadlessCase {
        name: TEMPLATE_REPLACE_NAME,
        values: REPLACE_TEMPLATE_VALUES,
        expected_stdout: ExpectedStdout::Exact(REPLACE_EXPECTED_STDOUT),
    },
    TemplateHeadlessCase {
        name: TEMPLATE_WORD_COUNT_NAME,
        values: DEFAULTED_TEMPLATE_VALUES,
        expected_stdout: ExpectedStdout::ContainsToken(WORD_COUNT_EXPECTED_TOKEN),
    },
];

fn temp_store() -> (Store, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("temporary store directory");
    let store = Store::new(temp_dir.path().join("test.redb")).expect("store opens");
    (store, temp_dir)
}

fn write_test_config(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data directory created");
    let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
    let data_dir_string = data_dir.display().to_string();
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir_string,
        ),
    )
    .expect("config written");
    (config_path, data_dir)
}

fn write_default_template_catalog(dir: &Path) {
    std::fs::create_dir_all(dir).expect("template directory created");
    for template_file in DEFAULT_NODE_TEMPLATE_FILES {
        std::fs::write(dir.join(template_file.file_name), template_file.contents).expect("default template written");
    }
}

fn template_by_name<'a>(templates: &'a [NodeTemplate], name: &str) -> &'a NodeTemplate {
    templates.iter().find(|template| template.name == name).expect("default template should load")
}

fn sorted_template_names(templates: &[NodeTemplate]) -> Vec<&str> {
    let mut names = templates.iter().map(|template| template.name.as_str()).collect::<Vec<_>>();
    names.sort();
    names
}

fn sorted_headless_case_names(cases: &[TemplateHeadlessCase]) -> Vec<&str> {
    let mut names = cases.iter().map(|case| case.name).collect::<Vec<_>>();
    names.sort();
    names
}

fn raw_template_values(case: &TemplateHeadlessCase) -> HashMap<String, String> {
    case.values.iter().map(|(name, value)| ((*name).to_string(), (*value).to_string())).collect()
}

fn template_workflow(
    template: &NodeTemplate,
    case: &TemplateHeadlessCase,
) -> (Workflow, NodeId, PortId, PortId, PortId) {
    let mut workflow = Workflow::new(format!("{TEMPLATE_WORKFLOW_NAME_PREFIX}{}", template.name));
    let source_id = workflow.add_shell_node(TEMPLATE_SOURCE_NODE_LABEL, TEMPLATE_SOURCE_COMMAND);
    let input_specs = template
        .input_ports
        .iter()
        .map(|port| PortSpec::new(port.label.clone(), port.type_tag.clone()))
        .collect::<Vec<_>>();
    let output_specs = template
        .output_ports
        .iter()
        .map(|port| PortSpec::new(port.label.clone(), port.type_tag.clone()))
        .collect::<Vec<_>>();
    let template_id = workflow.graph.add_node(template.name.clone(), &input_specs, &output_specs);
    let command = render_template_command(template, &raw_template_values(case)).expect("template command renders");
    workflow.set_node_config(template_id, NodeConfig::shell(command));

    let source_stdout = output_port_id(&workflow, source_id, TEMPLATE_STDOUT_PORT_LABEL);
    let template_stdin = input_port_id(&workflow, template_id, TEMPLATE_STDIN_PORT_LABEL);
    workflow.graph.add_edge(source_stdout, template_stdin).expect("template stdin wired");

    let template_stdout = output_port_id(&workflow, template_id, TEMPLATE_STDOUT_PORT_LABEL);
    let template_exit = output_port_id(&workflow, template_id, TEMPLATE_EXIT_PORT_LABEL);
    (workflow, template_id, template_stdin, template_stdout, template_exit)
}

fn input_port_id(workflow: &Workflow, node_id: NodeId, label: &str) -> PortId {
    workflow
        .graph
        .node(node_id)
        .expect("node exists")
        .input_ports
        .iter()
        .find(|port| port.label == label)
        .map(|port| port.id)
        .expect("input port exists")
}

fn output_port_id(workflow: &Workflow, node_id: NodeId, label: &str) -> PortId {
    workflow
        .graph
        .node(node_id)
        .expect("node exists")
        .output_ports
        .iter()
        .find(|port| port.label == label)
        .map(|port| port.id)
        .expect("output port exists")
}

fn template_input_cache_key(workflow: &Workflow, node_id: NodeId, stdin_port: PortId) -> n8nclone::engine::ContentHash {
    let inputs = HashMap::from([(stdin_port, Value::from_string(TEMPLATE_STDIN_TEXT.to_string()))]);
    node_execution_cache_key(node_id, workflow.node_config(node_id).expect("template config exists"), &inputs)
}

fn assert_cached_stdout(outputs: &HashMap<PortId, Value>, stdout_port: PortId, expected: ExpectedStdout) {
    let stdout = string_output(outputs, stdout_port);
    match expected {
        ExpectedStdout::Exact(expected_stdout) => assert_eq!(stdout, expected_stdout),
        ExpectedStdout::ContainsToken(expected_token) => {
            assert!(stdout.split_whitespace().any(|token| token == expected_token), "stdout was {stdout:?}");
        }
    }
}

fn assert_cached_exit_code(outputs: &HashMap<PortId, Value>, exit_port: PortId) {
    match &outputs.get(&exit_port).expect("exit output cached").data {
        ValueData::ExitCode(code) => assert_eq!(*code, SUCCESS_EXIT_CODE),
        other => panic!("expected exit code output, got {other:?}"),
    }
}

fn string_output(outputs: &HashMap<PortId, Value>, port: PortId) -> &str {
    match &outputs.get(&port).expect("string output cached").data {
        ValueData::String(value) => value,
        other => panic!("expected string output, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.headless.run]
// r[verify n8nclone.workflow_execution.headless.summary]
// r[verify n8nclone.workflow_execution.headless.exit_status]
// r[verify n8nclone.workflow_execution.headless.history]
#[tokio::test]
async fn headless_run_saved_workflow_succeeds_and_persists_history() {
    let (store, _temp_dir) = temp_store();
    let mut workflow = Workflow::new(SUCCESS_WORKFLOW_NAME);
    workflow.add_shell_node(SUCCESS_NODE_LABEL, SUCCESS_COMMAND);
    store.save_workflow(&workflow).expect("workflow saves");

    let summary = run_saved_workflow(&store, SUCCESS_WORKFLOW_NAME, RunId(SUCCESS_RUN_ID))
        .await
        .expect("headless run succeeds");

    assert_eq!(summary.run_id, RunId(SUCCESS_RUN_ID));
    assert_eq!(summary.workflow_name, SUCCESS_WORKFLOW_NAME);
    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(summary.node_count, SINGLE_NODE_COUNT);
    assert_eq!(summary.failures, NO_FAILURES);
    assert_eq!(process_result_for_summary(&summary), HeadlessProcessResult::Success);
    assert!(render_run_summary_text(&summary).contains("succeeded"));

    let records = store.list_runs(RECENT_RUN_LIMIT).expect("run history lists");
    assert_eq!(records.len(), SINGLE_NODE_COUNT);
    assert_eq!(records[0].run_id, RunId(SUCCESS_RUN_ID));
    assert_eq!(records[0].failures, NO_FAILURES);
}

// r[verify n8nclone.workflow_execution.image_input_nodes.production_io.headless]
#[tokio::test]
async fn headless_run_loads_local_image_with_default_executor() {
    let (store, temp_dir) = temp_store();
    let image_path = temp_dir.path().join(HEADLESS_IMAGE_FILE_NAME);
    std::fs::write(&image_path, PNG_FIXTURE).expect("headless image fixture written");
    let mut workflow = Workflow::new(IMAGE_WORKFLOW_NAME);
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, image_path.display().to_string());
    let image_port = output_port_id(&workflow, node_id, IMAGE_OUTPUT_PORT_LABEL);
    let input_hash = node_execution_cache_key(
        node_id,
        workflow.node_config(node_id).expect("image node config exists"),
        &HashMap::new(),
    );
    store.save_workflow(&workflow).expect("workflow saves");

    let summary = run_saved_workflow(&store, IMAGE_WORKFLOW_NAME, RunId(IMAGE_RUN_ID))
        .await
        .expect("headless image workflow runs");

    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(summary.failures, NO_FAILURES);
    let outputs = store.get_cache(&input_hash).expect("image cache lookup succeeds").expect("image outputs cached");
    let image = outputs.get(&image_port).expect("image output cached");
    assert_eq!(image.type_tag(), TypeTag::Image);
    assert!(image.as_display_string().contains("image/png"));
}

// r[verify n8nclone.node_templates.catalog]
// r[verify n8nclone.node_templates.catalog.headless]
// r[verify n8nclone.workflow_execution.headless.run]
#[tokio::test]
async fn default_template_catalog_runs_every_template_headlessly() {
    let (store, temp_dir) = temp_store();
    let templates_dir = temp_dir.path().join(TEMPLATES_DIR_NAME);
    write_default_template_catalog(&templates_dir);
    let load_result = load_node_templates_with_diagnostics(&templates_dir);

    assert!(load_result.diagnostics.is_empty());
    assert_eq!(
        sorted_template_names(&load_result.templates),
        sorted_headless_case_names(DEFAULT_TEMPLATE_HEADLESS_CASES)
    );

    for (case_index, case) in DEFAULT_TEMPLATE_HEADLESS_CASES.iter().enumerate() {
        let template = template_by_name(&load_result.templates, case.name);
        let (workflow, template_node_id, template_stdin, template_stdout, template_exit) =
            template_workflow(template, case);
        let workflow_name = workflow.meta.name.clone();
        let run_id = RunId(TEMPLATE_RUN_ID_START + u64::try_from(case_index).expect("case index fits in u64"));
        let template_input_hash = template_input_cache_key(&workflow, template_node_id, template_stdin);
        store.save_workflow(&workflow).expect("template workflow saves");

        let summary =
            run_saved_workflow(&store, &workflow_name, run_id).await.expect("template workflow runs headlessly");

        assert_eq!(summary.workflow_name, workflow_name);
        assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
        assert_eq!(summary.node_count, TEMPLATE_WORKFLOW_NODE_COUNT);
        assert_eq!(summary.steps_run, TEMPLATE_WORKFLOW_STEP_COUNT);
        assert_eq!(summary.failures, NO_FAILURES);
        let outputs = store
            .get_cache(&template_input_hash)
            .expect("template cache lookup succeeds")
            .expect("template outputs are cached");
        assert_cached_stdout(&outputs, template_stdout, case.expected_stdout);
        assert_cached_exit_code(&outputs, template_exit);
    }

    let records = store.list_runs(DEFAULT_NODE_TEMPLATE_FILES.len()).expect("template run history lists");
    assert_eq!(records.len(), DEFAULT_NODE_TEMPLATE_FILES.len());
}

// r[verify n8nclone.workflow_execution.headless.run]
// r[verify n8nclone.workflow_execution.headless.summary]
// r[verify n8nclone.workflow_execution.headless.exit_status]
#[test]
fn cli_run_saved_workflow_emits_json_summary() {
    let temp_dir = tempfile::tempdir().expect("temporary cli directory");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    let mut workflow = Workflow::new(SUCCESS_WORKFLOW_NAME);
    workflow.add_shell_node(SUCCESS_NODE_LABEL, SUCCESS_COMMAND);
    store.save_workflow(&workflow).expect("workflow saves");
    drop(store);

    let output = Command::new(env!("CARGO_BIN_EXE_n8nclone"))
        .arg("--config")
        .arg(&config_path)
        .arg("run")
        .arg(SUCCESS_WORKFLOW_NAME)
        .arg("--output")
        .arg("json")
        .output()
        .expect("cli run executes");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let summary: n8nclone::headless::HeadlessRunSummary =
        serde_json::from_slice(&output.stdout).expect("json summary parses");
    assert_eq!(summary.workflow_name, SUCCESS_WORKFLOW_NAME);
    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(summary.failures, NO_FAILURES);
}

// r[verify n8nclone.workflow_execution.headless.errors]
#[tokio::test]
async fn headless_run_missing_workflow_fails_without_history_record() {
    let (store, _temp_dir) = temp_store();

    let error = run_saved_workflow(&store, MISSING_WORKFLOW_NAME, RunId(MISSING_RUN_ID))
        .await
        .expect_err("missing workflow fails");

    assert_eq!(error, HeadlessRunError::WorkflowNotFound(MISSING_WORKFLOW_NAME.to_string()));
    let records = store.list_runs(RECENT_RUN_LIMIT).expect("run history lists");
    assert!(records.is_empty());
}

// r[verify n8nclone.workflow_execution.headless.exit_status]
// r[verify n8nclone.workflow_execution.headless.errors]
#[tokio::test]
async fn headless_run_failed_node_returns_failure_summary() {
    let (store, _temp_dir) = temp_store();
    let mut workflow = Workflow::new(BROKEN_WORKFLOW_NAME);
    let node_id = workflow.add_shell_node(BROKEN_NODE_LABEL, SUCCESS_COMMAND);
    workflow.set_node_config(node_id, NodeConfig::Shell {
        command: SUCCESS_COMMAND.to_string(),
        cwd: Some(MISSING_CWD.to_string()),
        env: std::collections::HashMap::new(),
        timeout_secs: DEFAULT_TIMEOUT_SECS,
        pipe_stdin: true,
    });
    store.save_workflow(&workflow).expect("workflow saves");

    let summary = run_saved_workflow(&store, BROKEN_WORKFLOW_NAME, RunId(FAILED_RUN_ID))
        .await
        .expect("headless run produces summary");

    assert_eq!(summary.status, HeadlessRunStatus::Failed);
    assert_eq!(summary.node_count, SINGLE_NODE_COUNT);
    assert_eq!(summary.failures, ONE_FAILURE);
    assert_eq!(process_result_for_summary(&summary), HeadlessProcessResult::Failure);

    let records = store.list_runs(RECENT_RUN_LIMIT).expect("run history lists");
    assert_eq!(records.len(), SINGLE_NODE_COUNT);
    assert_eq!(records[0].run_id, RunId(FAILED_RUN_ID));
    assert_eq!(records[0].failures, ONE_FAILURE);
}
