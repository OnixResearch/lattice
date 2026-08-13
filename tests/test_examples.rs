//! Tests for repository example templates and workflows.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use lattice::config::NodeTemplate;
use lattice::config::TemplateParameterDefault;
use lattice::config::load_node_templates_with_diagnostics;
use lattice::engine::executor::RunId;
use lattice::engine::executor::node_execution_cache_key;
use lattice::engine::value::Value;
use lattice::engine::value::ValueData;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::PortSpec;
use lattice::engine::workflow::Workflow;
use lattice::headless::HeadlessRunStatus;
use lattice::headless::run_saved_workflow;
use lattice::store::Store;
use lattice::workflow_format::load_workflow_nickel_file;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

const EXAMPLE_TEMPLATES_DIR: &str = "examples/templates";
const EXAMPLE_WORKFLOWS_DIR: &str = "examples/workflows";
const NICKEL_EXTENSION: &str = "ncl";
const TEMPLATE_WORKFLOW_NAME_PREFIX: &str = "example-template-";
const TEMPLATE_SOURCE_NODE_LABEL: &str = "example stdin";
const TEMPLATE_SOURCE_COMMAND: &str = "printf 'needle beta\\nalpha one\\nneedle alpha\\nneedle alpha\\n'";
const TEMPLATE_STDIN_TEXT: &str = "needle beta\nalpha one\nneedle alpha\nneedle alpha\n";
const TEMPLATE_STDIN_PORT_LABEL: &str = "stdin";
const TEMPLATE_STDOUT_PORT_LABEL: &str = "stdout";
const TEMPLATE_EXIT_PORT_LABEL: &str = "exit";
const COMPLEX_SORT_TEMPLATE_NAME: &str = "complex-sort-lines";
const COMPLEX_UNIQUE_TEMPLATE_NAME: &str = "complex-unique-lines";
const COMPLEX_CUT_FIELD_TEMPLATE_NAME: &str = "complex-cut-field";
const PARAM_DELIMITER: &str = "delimiter";
const PARAM_FIELD: &str = "field";
const DELIMITER_SPACE_VALUE: &str = " ";
const FIELD_TWO_VALUE: &str = "2";
const SORT_EXPECTED_STDOUT: &str = "alpha one\nneedle alpha\nneedle alpha\nneedle beta\n";
const UNIQUE_EXPECTED_STDOUT: &str = "needle beta\nalpha one\nneedle alpha\n";
const CUT_FIELD_EXPECTED_STDOUT: &str = "beta\none\nalpha\nalpha\n";
const COMPLEX_TEXT_NORMALIZATION_WORKFLOW_NAME: &str = "complex-text-normalization";
const DAILY_DIGEST_WORKFLOW_NAME: &str = "daily-digest";
const CSV_CLEANUP_WORKFLOW_NAME: &str = "csv-cleanup-report";
const LOG_TRIAGE_WORKFLOW_NAME: &str = "log-triage";
const COMPLEX_TEXT_NORMALIZATION_FINAL_LABEL: &str = "count lines";
const DAILY_DIGEST_FINAL_LABEL: &str = "render digest";
const CSV_CLEANUP_FINAL_LABEL: &str = "render csv report";
const LOG_TRIAGE_FINAL_LABEL: &str = "render triage";
const COMPLEX_TEXT_NORMALIZATION_FINAL_STDIN: &str = "thread alpha\nthread beta\n";
const DAILY_DIGEST_FINAL_STDIN: &str = "2026-06-03|security|critical|rotate token\n2026-06-03|ops|warning|disk high\n";
const CSV_CLEANUP_FINAL_STDIN: &str = "Data,Cara\nOps,Alice\n";
const LOG_TRIAGE_FINAL_STDIN: &str = "2026-06-03T10:01:00Z WARN latency 1200ms /api/search\n2026-06-03T10:02:00Z ERROR timeout /api/payments\n2026-06-03T10:03:00Z WARN latency 980ms /api/reports\n";
const COMPLEX_TEXT_NORMALIZATION_EXPECTED_TOKEN: &str = "2";
const DAILY_DIGEST_EXPECTED_STDOUT: &str =
    "# Daily digest\n\n- [critical] security: rotate token\n- [warning] ops: disk high\n";
const CSV_CLEANUP_EXPECTED_STDOUT: &str = "# Active users by team\n\n- Data: Cara\n- Ops: Alice\n";
const LOG_TRIAGE_EXPECTED_STDOUT: &str = "# Log triage\n\nWARN: 2\nERROR: 1\n\n- 2026-06-03T10:01:00Z WARN latency 1200ms /api/search\n- 2026-06-03T10:02:00Z ERROR timeout /api/payments\n- 2026-06-03T10:03:00Z WARN latency 980ms /api/reports\n";
const SUCCESS_EXIT_CODE: i32 = 0;
const TEMPLATE_RUN_ID_START: u64 = 300;
const WORKFLOW_RUN_ID_START: u64 = 400;
const TEMPLATE_WORKFLOW_NODE_COUNT: usize = 2;
const TEMPLATE_WORKFLOW_STEP_COUNT: usize = 2;
const NO_FAILURES: usize = 0;
const SINGLE_DIAGNOSTIC_COUNT: usize = 1;
const INVALID_TEMPLATE_FILE_NAME: &str = "invalid-template.ncl";
const INVALID_WORKFLOW_FILE_NAME: &str = "invalid-workflow.ncl";
const INVALID_WORKFLOW_TEXT: &str = "{}\n";
const INVALID_TEMPLATE_TEXT: &str = r#"{
  name = "invalid-complex-template",
  description = "Invalid example template",
  kind = "shell",
  command = "echo ${count}",
  parameters = [
    { name = "count", type = "number", default = "ten", required = false },
  ],
  input_ports = [],
  output_ports = [],
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExampleTemplateCase {
    name: &'static str,
    values: &'static [(&'static str, &'static str)],
    expected_stdout: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExampleWorkflowCase {
    name: &'static str,
    final_node_label: &'static str,
    final_stdin: &'static str,
    expected_stdout: ExpectedStdout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedStdout {
    Exact(&'static str),
    ContainsToken(&'static str),
}

const DEFAULT_TEMPLATE_VALUES: &[(&str, &str)] = &[];
const CUT_FIELD_TEMPLATE_VALUES: &[(&str, &str)] =
    &[(PARAM_DELIMITER, DELIMITER_SPACE_VALUE), (PARAM_FIELD, FIELD_TWO_VALUE)];
const EXAMPLE_TEMPLATE_CASES: &[ExampleTemplateCase] = &[
    ExampleTemplateCase {
        name: COMPLEX_SORT_TEMPLATE_NAME,
        values: DEFAULT_TEMPLATE_VALUES,
        expected_stdout: SORT_EXPECTED_STDOUT,
    },
    ExampleTemplateCase {
        name: COMPLEX_UNIQUE_TEMPLATE_NAME,
        values: DEFAULT_TEMPLATE_VALUES,
        expected_stdout: UNIQUE_EXPECTED_STDOUT,
    },
    ExampleTemplateCase {
        name: COMPLEX_CUT_FIELD_TEMPLATE_NAME,
        values: CUT_FIELD_TEMPLATE_VALUES,
        expected_stdout: CUT_FIELD_EXPECTED_STDOUT,
    },
];
const EXAMPLE_WORKFLOW_CASES: &[ExampleWorkflowCase] = &[
    ExampleWorkflowCase {
        name: COMPLEX_TEXT_NORMALIZATION_WORKFLOW_NAME,
        final_node_label: COMPLEX_TEXT_NORMALIZATION_FINAL_LABEL,
        final_stdin: COMPLEX_TEXT_NORMALIZATION_FINAL_STDIN,
        expected_stdout: ExpectedStdout::ContainsToken(COMPLEX_TEXT_NORMALIZATION_EXPECTED_TOKEN),
    },
    ExampleWorkflowCase {
        name: DAILY_DIGEST_WORKFLOW_NAME,
        final_node_label: DAILY_DIGEST_FINAL_LABEL,
        final_stdin: DAILY_DIGEST_FINAL_STDIN,
        expected_stdout: ExpectedStdout::Exact(DAILY_DIGEST_EXPECTED_STDOUT),
    },
    ExampleWorkflowCase {
        name: CSV_CLEANUP_WORKFLOW_NAME,
        final_node_label: CSV_CLEANUP_FINAL_LABEL,
        final_stdin: CSV_CLEANUP_FINAL_STDIN,
        expected_stdout: ExpectedStdout::Exact(CSV_CLEANUP_EXPECTED_STDOUT),
    },
    ExampleWorkflowCase {
        name: LOG_TRIAGE_WORKFLOW_NAME,
        final_node_label: LOG_TRIAGE_FINAL_LABEL,
        final_stdin: LOG_TRIAGE_FINAL_STDIN,
        expected_stdout: ExpectedStdout::Exact(LOG_TRIAGE_EXPECTED_STDOUT),
    },
];

// r[verify n8nclone.node_templates.examples.complex]
// r[verify n8nclone.node_templates.examples.headless]
#[tokio::test]
async fn complex_example_templates_render_and_run_headlessly() {
    let (store, _temp_dir) = temp_store();
    let load_result = load_node_templates_with_diagnostics(Path::new(EXAMPLE_TEMPLATES_DIR));

    assert!(load_result.diagnostics.is_empty());
    assert_eq!(sorted_template_names(&load_result.templates), sorted_case_names(EXAMPLE_TEMPLATE_CASES));
    assert_complex_template_defaults(&load_result.templates);

    for (case_index, case) in EXAMPLE_TEMPLATE_CASES.iter().enumerate() {
        let template = template_by_name(&load_result.templates, case.name);
        let (workflow, template_node_id, template_stdin, template_stdout, template_exit) =
            template_workflow(template, case);
        let workflow_name = workflow.meta.name.clone();
        let run_id = RunId(TEMPLATE_RUN_ID_START + u64::try_from(case_index).expect("case index fits in u64"));
        let template_input_hash = template_input_cache_key(&workflow, template_node_id, template_stdin);
        store.save_workflow(&workflow).expect("template workflow saves");

        let summary = run_saved_workflow(&store, &workflow_name, run_id)
            .await
            .expect("example template workflow runs headlessly");

        assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
        assert_eq!(summary.node_count, TEMPLATE_WORKFLOW_NODE_COUNT);
        assert_eq!(summary.steps_run, TEMPLATE_WORKFLOW_STEP_COUNT);
        assert_eq!(summary.failures, NO_FAILURES);
        let outputs = store
            .get_cache(&template_input_hash)
            .expect("template cache lookup succeeds")
            .expect("template outputs are cached");
        assert_cached_string(&outputs, template_stdout, case.expected_stdout);
        assert_cached_exit_code(&outputs, template_exit);
    }
}

// r[verify n8nclone.workflow_examples.complex]
// r[verify n8nclone.workflow_examples.headless]
#[tokio::test]
async fn complex_example_workflows_import_and_run_headlessly() {
    let (store, _temp_dir) = temp_store();
    let workflow_paths = collect_nickel_files(Path::new(EXAMPLE_WORKFLOWS_DIR))
        .expect("example workflow paths collect")
        .into_iter()
        .filter(|path| path.file_stem().and_then(|stem| stem.to_str()).is_some_and(is_text_workflow_case))
        .collect::<Vec<_>>();

    assert_eq!(workflow_paths.len(), EXAMPLE_WORKFLOW_CASES.len());
    for (workflow_index, workflow_path) in workflow_paths.iter().enumerate() {
        let workflow = load_workflow_nickel_file(workflow_path).expect("example workflow imports");
        let workflow_name = workflow.meta.name.clone();
        let case = workflow_case_by_name(&workflow_name);
        let final_node_id = node_id_by_label(&workflow, case.final_node_label);
        let final_stdin = input_port_id(&workflow, final_node_id, TEMPLATE_STDIN_PORT_LABEL);
        let final_stdout = output_port_id(&workflow, final_node_id, TEMPLATE_STDOUT_PORT_LABEL);
        let final_exit = output_port_id(&workflow, final_node_id, TEMPLATE_EXIT_PORT_LABEL);
        let final_input_hash = workflow_input_cache_key(&workflow, final_node_id, final_stdin, case.final_stdin);
        let run_id = RunId(WORKFLOW_RUN_ID_START + u64::try_from(workflow_index).expect("workflow index fits in u64"));
        store.save_workflow(&workflow).expect("example workflow saves");

        let summary =
            run_saved_workflow(&store, &workflow_name, run_id).await.expect("example workflow runs headlessly");

        assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
        assert_eq!(summary.node_count, workflow.graph.node_count());
        assert_eq!(summary.steps_run, workflow.graph.node_count());
        assert_eq!(summary.failures, NO_FAILURES);
        let outputs = store
            .get_cache(&final_input_hash)
            .expect("workflow cache lookup succeeds")
            .expect("workflow final outputs are cached");
        assert_cached_stdout(&outputs, final_stdout, case.expected_stdout);
        assert_cached_exit_code(&outputs, final_exit);
    }
}

#[test]
fn invalid_example_template_reports_diagnostic() {
    let temp_dir = tempfile::tempdir().expect("temporary invalid template directory");
    std::fs::write(temp_dir.path().join(INVALID_TEMPLATE_FILE_NAME), INVALID_TEMPLATE_TEXT)
        .expect("invalid template written");

    let result = load_node_templates_with_diagnostics(temp_dir.path());

    assert!(result.templates.is_empty());
    assert_eq!(result.diagnostics.len(), SINGLE_DIAGNOSTIC_COUNT);
    assert!(result.diagnostics[0].message.contains("default value type"));
}

#[test]
fn invalid_example_workflow_is_rejected() {
    let temp_dir = tempfile::tempdir().expect("temporary invalid workflow directory");
    let path = temp_dir.path().join(INVALID_WORKFLOW_FILE_NAME);
    std::fs::write(&path, INVALID_WORKFLOW_TEXT).expect("invalid workflow written");

    let error = load_workflow_nickel_file(&path).expect_err("invalid workflow rejected");

    assert!(error.to_string().contains("workflow JSON decode failed"));
}

fn temp_store() -> (Store, TempDir) {
    let temp_dir = tempfile::tempdir().expect("temporary store directory");
    let store = Store::new(temp_dir.path().join("test.redb")).expect("store opens");
    (store, temp_dir)
}

fn collect_nickel_files(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut paths = Vec::new();
    collect_nickel_files_into(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_nickel_files_into(path: &Path, paths: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if path.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) == Some(NICKEL_EXTENSION) {
            paths.push(path.to_path_buf());
        }
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        collect_nickel_files_into(&entry.path(), paths)?;
    }
    Ok(())
}

fn sorted_template_names(templates: &[NodeTemplate]) -> Vec<&str> {
    let mut names = templates.iter().map(|template| template.name.as_str()).collect::<Vec<_>>();
    names.sort();
    names
}

fn sorted_case_names(cases: &[ExampleTemplateCase]) -> Vec<&str> {
    let mut names = cases.iter().map(|case| case.name).collect::<Vec<_>>();
    names.sort();
    names
}

fn template_by_name<'a>(templates: &'a [NodeTemplate], name: &str) -> &'a NodeTemplate {
    templates.iter().find(|template| template.name == name).expect("example template should load")
}

fn is_text_workflow_case(name: &str) -> bool {
    EXAMPLE_WORKFLOW_CASES.iter().any(|case| case.name == name)
}

fn workflow_case_by_name(name: &str) -> &'static ExampleWorkflowCase {
    EXAMPLE_WORKFLOW_CASES
        .iter()
        .find(|case| case.name == name)
        .expect("example workflow should have an expected output case")
}

fn node_id_by_label(workflow: &Workflow, label: &str) -> NodeId {
    workflow
        .graph
        .nodes()
        .find(|node| node.label == label)
        .map(|node| node.id)
        .expect("example final node exists")
}

fn assert_complex_template_defaults(templates: &[NodeTemplate]) {
    let sort_template = template_by_name(templates, COMPLEX_SORT_TEMPLATE_NAME);
    let unique_template = template_by_name(templates, COMPLEX_UNIQUE_TEMPLATE_NAME);
    let cut_template = template_by_name(templates, COMPLEX_CUT_FIELD_TEMPLATE_NAME);

    assert_eq!(sort_template.parameters.len(), DEFAULT_TEMPLATE_VALUES.len() + 1);
    assert_eq!(unique_template.parameters.len(), DEFAULT_TEMPLATE_VALUES.len() + 1);
    assert!(matches!(cut_template.parameters[0].default, Some(TemplateParameterDefault::String(_))));
    assert!(matches!(cut_template.parameters[1].default, Some(TemplateParameterDefault::Number(_))));
}

fn raw_template_values(case: &ExampleTemplateCase) -> HashMap<String, String> {
    case.values.iter().map(|(name, value)| ((*name).to_string(), (*value).to_string())).collect()
}

fn template_workflow(
    template: &NodeTemplate,
    case: &ExampleTemplateCase,
) -> (Workflow, NodeId, PortId, PortId, PortId) {
    let mut workflow = Workflow::new(
        format!("{TEMPLATE_WORKFLOW_NAME_PREFIX}{}", template.name),
        lattice::engine::workflow::WorkflowTimestamp::epoch(),
    );
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
    let command = lattice::config::render_template_command(template, &raw_template_values(case))
        .expect("template command renders");
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

fn template_input_cache_key(workflow: &Workflow, node_id: NodeId, stdin_port: PortId) -> lattice::engine::ContentHash {
    workflow_input_cache_key(workflow, node_id, stdin_port, TEMPLATE_STDIN_TEXT)
}

fn workflow_input_cache_key(
    workflow: &Workflow,
    node_id: NodeId,
    stdin_port: PortId,
    stdin_text: &str,
) -> lattice::engine::ContentHash {
    let inputs = HashMap::from([(stdin_port, Value::from_string(stdin_text.to_string()))]);
    node_execution_cache_key(node_id, workflow.node_config(node_id).expect("node config exists"), &inputs)
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

fn assert_cached_string(outputs: &HashMap<PortId, Value>, port: PortId, expected: &str) {
    assert_eq!(string_output(outputs, port), expected);
}

fn string_output(outputs: &HashMap<PortId, Value>, port: PortId) -> &str {
    match &outputs.get(&port).expect("string output cached").data {
        ValueData::String(value) => value,
        other => panic!("expected string output, got {other:?}"),
    }
}

fn assert_cached_exit_code(outputs: &HashMap<PortId, Value>, exit_port: PortId) {
    match &outputs.get(&exit_port).expect("exit output cached").data {
        ValueData::ExitCode(code) => assert_eq!(*code, SUCCESS_EXIT_CODE),
        other => panic!("expected exit code output, got {other:?}"),
    }
}
