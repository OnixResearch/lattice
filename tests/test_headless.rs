//! Tests for headless workflow runs.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use lattice::agent_node::AgentCommandRuntimeConfig;
use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentRuntimeConfig;
use lattice::agent_node::AgentTarget;
use lattice::config::DEFAULT_NODE_TEMPLATE_FILES;
use lattice::config::NodeTemplate;
use lattice::config::load_node_templates_with_diagnostics;
use lattice::config::render_template_command;
use lattice::engine::effect::authority;
use lattice::engine::effect::dispatch;
use lattice::engine::effect::execution;
use lattice::engine::executor::RunId;
use lattice::engine::executor::node_execution_cache_key;
use lattice::engine::runner::RunnerEventKind;
use lattice::engine::value::TypeTag;
use lattice::engine::value::Value;
use lattice::engine::value::ValueData;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::PortSpec;
use lattice::engine::workflow::Workflow;
use lattice::headless::HeadlessProcessResult;
use lattice::headless::HeadlessRunError;
use lattice::headless::HeadlessRunStatus;
use lattice::headless::process_result_for_summary;
use lattice::headless::render_run_summary_text;
use lattice::headless::run_saved_workflow;
use lattice::headless::run_saved_workflow_with_agent_runtime_config;
use lattice::headless::run_saved_workflow_with_authority_provider;
use lattice::store::RunNodeDetailStatus;
use lattice::store::Store;

const SUCCESS_WORKFLOW_NAME: &str = "headless-success";
const MISSING_WORKFLOW_NAME: &str = "missing-workflow";
const BROKEN_WORKFLOW_NAME: &str = "headless-broken";
const IMAGE_WORKFLOW_NAME: &str = "headless-image";
const AGENT_WORKFLOW_NAME: &str = "headless-agent";
const SUCCESS_NODE_LABEL: &str = "say hello";
const SUCCESS_COMMAND: &str = "printf hello";
const SUCCESS_OUTPUT: &str = "hello";
const BROKEN_NODE_LABEL: &str = "broken";
const IMAGE_NODE_LABEL: &str = "load image";
const PI_RUNTIME_KEY: &str = "pi";
const PROMPT_ARG_TEMPLATE: &str = "{prompt}";
const AGENT_NODE_LABEL: &str = "summarize";
const IMAGE_OUTPUT_PORT_LABEL: &str = "image";
const AGENT_TEXT_OUTPUT_PORT_LABEL: &str = "text";
const HEADLESS_IMAGE_FILE_NAME: &str = "headless-selfie.png";
const HEADLESS_AGENT_IMAGE_FILE_NAME: &str = "headless-agent-output.png";
const MISSING_CWD: &str = "/definitely/missing/lattice/headless-test";
const SUCCESS_RUN_ID: u64 = 7;
const MISSING_RUN_ID: u64 = 11;
const FAILED_RUN_ID: u64 = 13;
const IMAGE_RUN_ID: u64 = 17;
const AGENT_RUN_ID: u64 = 19;
const AGENT_IMAGE_RUN_ID: u64 = 23;
const RECENT_RUN_LIMIT: usize = 8;
const SINGLE_NODE_COUNT: usize = 1;
const HEADLESS_REPLAY_EVENT_COUNT: usize = 4;
const NO_FAILURES: usize = 0;
const ONE_FAILURE: usize = 1;
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 4;
const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "lattice.redb";
const TEMPLATES_DIR_NAME: &str = "templates";
const TEMPLATE_WORKFLOW_NAME_PREFIX: &str = "headless-template-";
const TEMPLATE_SOURCE_NODE_LABEL: &str = "template stdin";
const TEMPLATE_SOURCE_COMMAND: &str = "printf 'alpha\\nneedle\\nneedle beta\\n'";
const TEMPLATE_STDIN_TEXT: &str = "alpha\nneedle\nneedle beta\n";
const TEMPLATE_STDIN_PORT_LABEL: &str = "stdin";
const TEMPLATE_STDOUT_PORT_LABEL: &str = "stdout";
const TEMPLATE_EXIT_PORT_LABEL: &str = "exit";
const TEMPLATE_COMPLEX_CUT_FIELD_NAME: &str = "complex-cut-field";
const TEMPLATE_COMPLEX_SORT_LINES_NAME: &str = "complex-sort-lines";
const TEMPLATE_COMPLEX_UNIQUE_LINES_NAME: &str = "complex-unique-lines";
const TEMPLATE_GREP_NAME: &str = "grep";
const TEMPLATE_HEAD_NAME: &str = "head";
const TEMPLATE_REPLACE_NAME: &str = "replace";
const TEMPLATE_WORD_COUNT_NAME: &str = "word-count";
const PARAM_PATTERN: &str = "pattern";
const PARAM_REPLACEMENT: &str = "replacement";
const PARAM_NEEDLE_VALUE: &str = "needle";
const PARAM_THREAD_VALUE: &str = "thread";
const CUT_FIELD_EXPECTED_STDOUT: &str = "alpha\nneedle\nneedle\n";
const GREP_EXPECTED_STDOUT: &str = "needle\nneedle beta\n";
const REPLACE_EXPECTED_STDOUT: &str = "alpha\nthread\nthread beta\n";
const WORD_COUNT_EXPECTED_TOKEN: &str = "3";
const SUCCESS_EXIT_CODE: i32 = 0;
const TEMPLATE_RUN_ID_START: u64 = 100;
const TEMPLATE_WORKFLOW_NODE_COUNT: usize = 2;
const TEMPLATE_WORKFLOW_STEP_COUNT: usize = 2;
const PNG_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\nfixture";
const TEST_GRANT_REF: &str = "blake3:3333333333333333333333333333333333333333333333333333333333333333";
const TEST_RECEIPT_REF: &str = "blake3:4444444444444444444444444444444444444444444444444444444444444444";
const TEST_AUTHORITY_REF: &str = "test-current-authority";
const PROCESS_CONTRACT: &str = "lattice-process";

struct CurrentAuthorityFacts;

impl dispatch::AuthorityFactsProvider for CurrentAuthorityFacts {
    fn load(&self, binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, dispatch::Error> {
        let resource = binding.resource.as_str().to_owned();
        let ability = binding.ability.as_str().to_owned();
        let request_ref = binding.request.as_str().to_owned();
        Ok(authority::AuthorizationFacts {
            binding: binding.clone(),
            basalt_input: basalt::authority_core::AuthorityInput {
                schema: basalt::authority_core::AUTHORITY_INPUT_SCHEMA.to_owned(),
                request: basalt::authority_core::AuthorityRequest {
                    contract_id: PROCESS_CONTRACT.to_owned(),
                    resource: resource.clone(),
                    ability: ability.clone(),
                    request_ref: request_ref.clone(),
                },
                verified_grants: vec![basalt::authority_core::VerifiedGrantRef {
                    grant_ref: TEST_GRANT_REF.to_owned(),
                    verification_receipt_ref: TEST_RECEIPT_REF.to_owned(),
                    resource,
                    ability,
                    kind: basalt::authority_core::VerifiedGrantKind::Direct,
                }],
                verification_receipts: vec![basalt::authority_core::UcanVerificationReceiptRef {
                    receipt_ref: TEST_RECEIPT_REF.to_owned(),
                    request_ref,
                    verified_grant_refs: vec![TEST_GRANT_REF.to_owned()],
                    disposition: basalt::authority_core::VerificationRefDisposition::Current,
                }],
                policy_refs: vec![dispatch::trusted_policy_ref(PROCESS_CONTRACT)],
                caveat: basalt::authority_core::AuthorityCaveatDisposition::Satisfied,
                replay: basalt::authority_core::AuthorityReplayDisposition::Admitted,
                claims: vec![basalt::authority_core::AuthorityClaim::ExactRequest],
                non_claims: basalt::authority_core::REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_owned).to_vec(),
            },
            trusted_policy_ref: dispatch::trusted_policy_ref(PROCESS_CONTRACT),
            scope_requirement: authority::ScopeRequirement::UnscopedPermitted,
            scope_snapshot: None,
            revocation: authority::RevocationDisposition::Current,
        })
    }
}

struct NoScope;

impl dispatch::ScopeSnapshotProvider for NoScope {
    fn load_current(
        &self,
        _binding: &authority::IntentBinding,
    ) -> Result<Option<authority::ScopeSnapshotFacts>, dispatch::Error> {
        Ok(None)
    }
}

struct NoRoots;

impl dispatch::ConfiguredRootRegistry for NoRoots {
    fn contains_root(&self, _root_id: &str) -> bool {
        false
    }

    fn with_root(
        &self,
        _root_id: &str,
        _operation: &mut dyn FnMut(&cap_std::fs::Dir) -> Result<(), dispatch::Error>,
    ) -> Result<(), dispatch::Error> {
        Err(dispatch::Error::UnknownRoot)
    }
}

struct CurrentAuthorityProvider;

impl execution::RunContextProvider for CurrentAuthorityProvider {
    fn load(&self, run_id: RunId, store: Option<Store>) -> Result<execution::RunContext, execution::Error> {
        let attempts: Box<dyn dispatch::EffectAttemptStore> = match store {
            Some(store) => Box::new(execution::StoreAttempts::new(store)),
            None => Box::new(execution::UnavailableAttempts),
        };
        let shell =
            dispatch::Shell::new(Box::new(CurrentAuthorityFacts), Box::new(NoScope), attempts, Box::new(NoRoots));
        Ok(execution::RunContext::new(
            authority::RunIdentity::new(format!("test-run/{}", run_id.0))?,
            authority::AuthorityIdentity::new(TEST_AUTHORITY_REF)?,
            authority::CollaborationScopeBinding::Unscoped,
            std::sync::Arc::new(shell),
        ))
    }
}
const HEADLESS_AGENT_FILE_NAME: &str = "headless-agent.sh";
const HEADLESS_AGENT_OUTPUT: &str = "headless agent text";
const HEADLESS_AGENT_IMAGE_PROMPT: &str = "create image";
const EXECUTABLE_MODE: u32 = 0o755;

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
        name: TEMPLATE_COMPLEX_CUT_FIELD_NAME,
        values: DEFAULTED_TEMPLATE_VALUES,
        expected_stdout: ExpectedStdout::Exact(CUT_FIELD_EXPECTED_STDOUT),
    },
    TemplateHeadlessCase {
        name: TEMPLATE_COMPLEX_SORT_LINES_NAME,
        values: DEFAULTED_TEMPLATE_VALUES,
        expected_stdout: ExpectedStdout::Exact(TEMPLATE_STDIN_TEXT),
    },
    TemplateHeadlessCase {
        name: TEMPLATE_COMPLEX_UNIQUE_LINES_NAME,
        values: DEFAULTED_TEMPLATE_VALUES,
        expected_stdout: ExpectedStdout::Exact(TEMPLATE_STDIN_TEXT),
    },
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

#[cfg(unix)]
fn write_agent_script(temp_dir: &tempfile::TempDir) -> String {
    write_agent_script_with_body(temp_dir, &format!("#!/bin/sh\nprintf '{HEADLESS_AGENT_OUTPUT}'\n"))
}

#[cfg(unix)]
fn write_agent_script_with_body(temp_dir: &tempfile::TempDir, body: &str) -> String {
    let path = temp_dir.path().join(HEADLESS_AGENT_FILE_NAME);
    fs::write(&path, body).expect("agent script written");
    let mut permissions = fs::metadata(&path).expect("agent script metadata").permissions();
    permissions.set_mode(EXECUTABLE_MODE);
    fs::set_permissions(&path, permissions).expect("agent script made executable");
    path.display().to_string()
}

fn pi_agent_runtime(command: String) -> AgentRuntimeConfig {
    let mut runtimes = BTreeMap::new();
    runtimes.insert(PI_RUNTIME_KEY.to_string(), AgentCommandRuntimeConfig {
        command,
        args: vec!["-p".to_string(), PROMPT_ARG_TEMPLATE.to_string()],
    });
    AgentRuntimeConfig {
        runtimes,
        openai_compatible: BTreeMap::new(),
        structured_input_projection: Default::default(),
        max_output_bytes: AgentRuntimeConfig::default().max_output_bytes,
        local_process: Default::default(),
    }
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

fn template_input_cache_key(workflow: &Workflow, node_id: NodeId, stdin_port: PortId) -> lattice::engine::ContentHash {
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

// r[verify n8nclone.workflow_execution.headless.exit_status.success]
// r[verify n8nclone.workflow_execution.headless.history.persist]
// r[verify n8nclone.workflow_execution.headless.run.saved]
// r[verify n8nclone.workflow_execution.headless.summary.text]
// r[verify n8nclone.workflow_execution.headless.run]
// r[verify n8nclone.workflow_execution.headless.summary]
// r[verify n8nclone.workflow_execution.headless.exit_status]
// r[verify n8nclone.workflow_execution.headless.history]
// r[verify n8nclone.workflow_execution.replay.event_store]
// r[verify lattice.workflow_execution.headless_run_details]
// r[verify lattice.workflow_execution.headless_run_details.scenario.success]
#[tokio::test]
async fn headless_run_saved_workflow_succeeds_and_persists_history() {
    let (store, _temp_dir) = temp_store();
    let mut workflow = Workflow::new(SUCCESS_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(SUCCESS_NODE_LABEL, SUCCESS_COMMAND);
    store.save_workflow(&workflow).expect("workflow saves");

    let summary = run_saved_workflow_with_authority_provider(
        &store,
        SUCCESS_WORKFLOW_NAME,
        RunId(SUCCESS_RUN_ID),
        &CurrentAuthorityProvider,
    )
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
    let details = store
        .load_run_details(RunId(SUCCESS_RUN_ID))
        .expect("run details load")
        .expect("run details persist");
    assert_eq!(details.nodes.len(), SINGLE_NODE_COUNT);
    assert_eq!(details.nodes[0].status, RunNodeDetailStatus::Completed);
    assert!(details.nodes[0].outputs.iter().any(|output| output.text == SUCCESS_OUTPUT));

    let replay_events = store
        .load_run_replay_events(RunId(SUCCESS_RUN_ID))
        .expect("replay events load")
        .expect("replay events saved");
    assert_eq!(replay_events.len(), HEADLESS_REPLAY_EVENT_COUNT);
    assert_eq!(replay_events[0].kind, RunnerEventKind::RunStarted);
    assert_eq!(replay_events[HEADLESS_REPLAY_EVENT_COUNT - 1].kind, RunnerEventKind::RunCompleted);
}

// r[verify n8nclone.workflow_execution.image_input_nodes.production_io.headless]
#[tokio::test]
async fn headless_run_loads_local_image_with_default_executor() {
    let (store, temp_dir) = temp_store();
    let image_path = temp_dir.path().join(HEADLESS_IMAGE_FILE_NAME);
    std::fs::write(&image_path, PNG_FIXTURE).expect("headless image fixture written");
    let mut workflow = Workflow::new(IMAGE_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let image_path_label = image_path.display().to_string();
    let node_id = workflow.add_image_input_node(IMAGE_NODE_LABEL, &image_path_label);
    workflow.set_node_config(node_id, NodeConfig::ImageInput {
        path: image_path_label,
        allowed_roots: vec![temp_dir.path().to_string_lossy().into_owned()],
        max_bytes: lattice::image_node::DEFAULT_MAX_IMAGE_BYTES,
    });
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

// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.default_executor]
// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.command]
#[cfg(unix)]
#[tokio::test]
async fn headless_run_uses_configured_production_agent_runtime() {
    let (store, temp_dir) = temp_store();
    let command = write_agent_script(&temp_dir);
    let mut workflow = Workflow::new(AGENT_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, "summarize this", AgentOutputMode::Text);
    let text_port = output_port_id(&workflow, node_id, AGENT_TEXT_OUTPUT_PORT_LABEL);
    let input_hash = node_execution_cache_key(
        node_id,
        workflow.node_config(node_id).expect("agent node config exists"),
        &HashMap::new(),
    );
    store.save_workflow(&workflow).expect("workflow saves");
    let agent_runtime = pi_agent_runtime(command);

    let summary =
        run_saved_workflow_with_agent_runtime_config(&store, AGENT_WORKFLOW_NAME, RunId(AGENT_RUN_ID), agent_runtime)
            .await
            .expect("headless agent workflow runs");

    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(summary.failures, NO_FAILURES);
    let outputs = store.get_cache(&input_hash).expect("agent cache lookup succeeds").expect("agent outputs cached");
    let text = string_output(&outputs, text_port);
    assert_eq!(text, HEADLESS_AGENT_OUTPUT);
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.default_runtime]
// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.image]
#[cfg(unix)]
#[tokio::test]
async fn headless_run_maps_agent_image_handoff_to_image_output_port() {
    let (store, temp_dir) = temp_store();
    fs::write(temp_dir.path().join(HEADLESS_AGENT_IMAGE_FILE_NAME), PNG_FIXTURE).expect("agent image written");
    let envelope = serde_json::json!({"path": HEADLESS_AGENT_IMAGE_FILE_NAME, "media_type": "image/png"});
    let command = write_agent_script_with_body(&temp_dir, &format!("#!/bin/sh\nprintf '{envelope}'\n"));
    let mut workflow = Workflow::new(AGENT_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_id =
        workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, HEADLESS_AGENT_IMAGE_PROMPT, AgentOutputMode::Image);
    if let Some(NodeConfig::Agent(settings)) = workflow.configs.get_mut(&node_id.0) {
        settings.cwd = Some(temp_dir.path().display().to_string());
    }
    let image_port = output_port_id(&workflow, node_id, IMAGE_OUTPUT_PORT_LABEL);
    let input_hash = node_execution_cache_key(
        node_id,
        workflow.node_config(node_id).expect("agent node config exists"),
        &HashMap::new(),
    );
    store.save_workflow(&workflow).expect("workflow saves");
    let agent_runtime = pi_agent_runtime(command);

    let summary = run_saved_workflow_with_agent_runtime_config(
        &store,
        AGENT_WORKFLOW_NAME,
        RunId(AGENT_IMAGE_RUN_ID),
        agent_runtime,
    )
    .await
    .expect("headless image handoff workflow runs");

    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(summary.failures, NO_FAILURES);
    let outputs = store.get_cache(&input_hash).expect("agent cache lookup succeeds").expect("agent outputs cached");
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
            run_saved_workflow_with_authority_provider(&store, &workflow_name, run_id, &CurrentAuthorityProvider)
                .await
                .expect("template workflow runs headlessly");

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

// r[verify n8nclone.workflow_execution.headless.summary.json]
// r[verify n8nclone.workflow_execution.headless.run]
// r[verify n8nclone.workflow_execution.headless.summary]
// r[verify n8nclone.workflow_execution.headless.exit_status]
#[test]
fn cli_run_saved_workflow_emits_json_summary() {
    let temp_dir = tempfile::tempdir().expect("temporary cli directory");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    let mut workflow = Workflow::new(SUCCESS_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node(SUCCESS_NODE_LABEL, SUCCESS_COMMAND);
    store.save_workflow(&workflow).expect("workflow saves");
    drop(store);

    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .arg("run")
        .arg(SUCCESS_WORKFLOW_NAME)
        .arg("--output")
        .arg("json")
        .output()
        .expect("cli run executes");

    assert!(!output.status.success(), "default product execution must fail closed");
    let summary: lattice::headless::HeadlessRunSummary =
        serde_json::from_slice(&output.stdout).expect("json summary parses");
    assert_eq!(summary.workflow_name, SUCCESS_WORKFLOW_NAME);
    assert_eq!(summary.status, HeadlessRunStatus::Failed);
    assert_eq!(summary.failures, ONE_FAILURE);
}

// r[verify n8nclone.workflow_execution.headless.errors.load_failure]
// r[verify n8nclone.workflow_execution.headless.run.missing]
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
    assert!(store.load_run_details(RunId(MISSING_RUN_ID)).expect("run details lookup succeeds").is_none());
}

// r[verify n8nclone.workflow_execution.headless.exit_status.failure]
// r[verify n8nclone.workflow_execution.headless.exit_status]
// r[verify n8nclone.workflow_execution.headless.errors]
// r[verify lattice.workflow_execution.headless_run_details]
// r[verify lattice.workflow_execution.headless_run_details.scenario.failure]
#[tokio::test]
async fn headless_run_failed_node_returns_failure_summary() {
    let (store, _temp_dir) = temp_store();
    let mut workflow = Workflow::new(BROKEN_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
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
    let details = store
        .load_run_details(RunId(FAILED_RUN_ID))
        .expect("failed details load")
        .expect("failed details persist");
    assert_eq!(details.nodes.len(), SINGLE_NODE_COUNT);
    assert_eq!(details.nodes[0].status, RunNodeDetailStatus::Failed);
    assert!(details.nodes[0].outputs.is_empty());
    assert!(details.nodes[0].error.is_some());
}
