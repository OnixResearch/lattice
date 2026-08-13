//! Maintained saved-child execution, output routing, and correlation evidence.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use lattice::control_flow::SubworkflowInputMapping;
use lattice::control_flow::SubworkflowNodeConfig;
use lattice::control_flow::SubworkflowOutputMapping;
use lattice::engine::Executor;
use lattice::engine::NodeConfig;
use lattice::engine::RepositorySubworkflowRunnerShell;
use lattice::engine::RunId;
use lattice::engine::SubworkflowEvent;
use lattice::engine::SubworkflowEventKind;
use lattice::engine::SubworkflowEventMetadata;
use lattice::engine::SubworkflowExecution;
use lattice::engine::SubworkflowExecutionError;
use lattice::engine::SubworkflowInvocation;
use lattice::engine::SubworkflowRunnerShell;
use lattice::engine::Value;
use lattice::engine::Workflow;
use lattice::engine::executor::NodeStatus;
use lattice::store::Store;
use pretty_assertions::assert_eq;

const PARENT_WORKFLOW: &str = "parent-workflow";
const CHILD_WORKFLOW: &str = "saved-child-workflow";
const PARENT_INPUT: &str = "parent-input";
const CHILD_INPUT: &str = "stdin";
const CHILD_OUTPUT: &str = "stdout";
const PARENT_OUTPUT: &str = "result";
const CHILD_RESULT: &str = "child-result";
const CHILD_RUN_ID: RunId = RunId(41);
const HEADLESS_RUN_ID: RunId = RunId(52);
const DRIFTED_PARENT_RUN_ID: RunId = RunId(77);
const EXPECTED_STEP_COUNT: usize = 1;
const EXPECTED_OUTPUT_COUNT: usize = 1;
const EXPECTED_EVENT_COUNT: usize = 2;

fn subworkflow_config(child_output: &str) -> SubworkflowNodeConfig {
    SubworkflowNodeConfig {
        workflow_name: CHILD_WORKFLOW.to_string(),
        input_mappings: vec![SubworkflowInputMapping {
            parent_input: PARENT_INPUT.to_string(),
            child_input: CHILD_INPUT.to_string(),
        }],
        output_mappings: vec![SubworkflowOutputMapping {
            child_output: child_output.to_string(),
            parent_output: PARENT_OUTPUT.to_string(),
        }],
    }
}

fn parent_workflow(child_output: &str) -> (Workflow, lattice::engine::workflow::NodeId) {
    let mut workflow = Workflow::new(PARENT_WORKFLOW, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_subworkflow_node("Saved child", subworkflow_config(child_output));
    (workflow, node_id)
}

fn named_parent_inputs() -> BTreeMap<String, Value> {
    BTreeMap::from([(PARENT_INPUT.to_string(), Value::from_string("parent-value".to_string()))])
}

fn completed_outputs(step_status: &NodeStatus) -> &HashMap<lattice::engine::workflow::PortId, Value> {
    match step_status {
        NodeStatus::Completed { outputs, .. } => outputs,
        other => panic!("expected completed subworkflow step, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.subworkflows.invocation.outputs]
#[tokio::test]
async fn saved_child_executes_and_routes_declared_output_to_the_real_parent_port() {
    let temporary = tempfile::tempdir().expect("temporary store directory");
    let store = Arc::new(Store::new(temporary.path().join("lattice.redb")).expect("open store"));
    let mut child = Workflow::new(CHILD_WORKFLOW, lattice::engine::workflow::WorkflowTimestamp::epoch());
    child.add_shell_node("Child result", format!("printf {CHILD_RESULT}"));
    store.save_workflow(&child).expect("save child workflow");

    let (parent, subworkflow_node_id) = parent_workflow(CHILD_OUTPUT);
    let parent_output_port = parent
        .graph
        .node(subworkflow_node_id)
        .expect("subworkflow node")
        .output_ports
        .iter()
        .find(|port| port.label == PARENT_OUTPUT)
        .expect("declared parent output")
        .id;
    let shell = RepositorySubworkflowRunnerShell::new(Arc::clone(&store));
    let mut executor = Executor::new_with_subworkflow_runner_shell(Box::new(shell));

    let (_parent_run_id, steps) = executor
        .run_with_named_inputs(&parent, &HashMap::new(), &named_parent_inputs())
        .await
        .expect("bind parent inputs");

    assert_eq!(steps.len(), EXPECTED_STEP_COUNT);
    let outputs = completed_outputs(&steps[0].status);
    assert_eq!(outputs.len(), EXPECTED_OUTPUT_COUNT);
    assert_eq!(outputs.get(&parent_output_port).expect("real mapped output").as_display_string(), CHILD_RESULT);
}

#[tokio::test]
async fn headless_production_shell_denies_effectful_child_without_child_authority() {
    let temporary = tempfile::tempdir().expect("temporary store directory");
    let store = Store::new(temporary.path().join("lattice.redb")).expect("open store");
    let mut child = Workflow::new(CHILD_WORKFLOW, lattice::engine::workflow::WorkflowTimestamp::epoch());
    child.add_shell_node("Child result", format!("printf {CHILD_RESULT}"));
    store.save_workflow(&child).expect("save child workflow");
    let mut parent = Workflow::new(PARENT_WORKFLOW, lattice::engine::workflow::WorkflowTimestamp::epoch());
    parent.add_subworkflow_node("Saved child", SubworkflowNodeConfig {
        workflow_name: CHILD_WORKFLOW.to_string(),
        input_mappings: Vec::new(),
        output_mappings: vec![SubworkflowOutputMapping {
            child_output: CHILD_OUTPUT.to_string(),
            parent_output: PARENT_OUTPUT.to_string(),
        }],
    });
    store.save_workflow(&parent).expect("save parent workflow");

    let summary = lattice::headless::run_saved_workflow(&store, PARENT_WORKFLOW, HEADLESS_RUN_ID)
        .await
        .expect("headless child execution");

    assert_eq!(summary.run_id, HEADLESS_RUN_ID);
    assert_eq!(summary.failures, EXPECTED_STEP_COUNT);
    assert_eq!(summary.completed, 0);
}

#[derive(Default)]
struct RecordingCorrelatedShell {
    events: Arc<Mutex<Vec<SubworkflowEvent>>>,
}

#[async_trait]
impl SubworkflowRunnerShell for RecordingCorrelatedShell {
    async fn execute(
        &self,
        invocation: &SubworkflowInvocation,
    ) -> Result<SubworkflowExecution, SubworkflowExecutionError> {
        let metadata = SubworkflowEventMetadata {
            parent_run_id: invocation.correlation.parent_run_id,
            subworkflow_node_id: invocation.correlation.subworkflow_node_id,
            child_run_id: CHILD_RUN_ID,
            workflow_name: invocation.workflow_name.clone(),
        };
        let events = vec![
            SubworkflowEvent {
                kind: SubworkflowEventKind::Started,
                metadata: metadata.clone(),
            },
            SubworkflowEvent {
                kind: SubworkflowEventKind::Completed,
                metadata,
            },
        ];
        self.events.lock().expect("event recorder lock").extend(events.clone());
        Ok(SubworkflowExecution {
            workflow_name: invocation.workflow_name.clone(),
            child_run_id: CHILD_RUN_ID,
            outputs: BTreeMap::from([(CHILD_OUTPUT.to_string(), Value::from_string(CHILD_RESULT.to_string()))]),
            events,
        })
    }
}

// r[verify n8nclone.workflow_execution.subworkflows.events.correlation]
#[tokio::test]
async fn child_events_bind_parent_run_child_run_and_subworkflow_node_identity() {
    let (parent, subworkflow_node_id) = parent_workflow(CHILD_OUTPUT);
    let shell = RecordingCorrelatedShell::default();
    let recorded_events = Arc::clone(&shell.events);
    let mut executor = Executor::new_with_subworkflow_runner_shell(Box::new(shell));

    let (parent_run_id, steps) = executor
        .run_with_named_inputs(&parent, &HashMap::new(), &named_parent_inputs())
        .await
        .expect("bind parent inputs");

    assert!(matches!(steps[0].status, NodeStatus::Completed { .. }));
    let events = recorded_events.lock().expect("event recorder lock");
    assert_eq!(events.len(), EXPECTED_EVENT_COUNT);
    assert_eq!(events[0].kind, SubworkflowEventKind::Started);
    assert_eq!(events[1].kind, SubworkflowEventKind::Completed);
    assert!(events.iter().all(|event| event.metadata.parent_run_id == parent_run_id));
    assert!(events.iter().all(|event| event.metadata.subworkflow_node_id == subworkflow_node_id));
    assert!(events.iter().all(|event| event.metadata.child_run_id == CHILD_RUN_ID));
    assert!(events.iter().all(|event| event.metadata.workflow_name == CHILD_WORKFLOW));
}

struct DriftedCorrelationShell;

#[async_trait]
impl SubworkflowRunnerShell for DriftedCorrelationShell {
    async fn execute(
        &self,
        invocation: &SubworkflowInvocation,
    ) -> Result<SubworkflowExecution, SubworkflowExecutionError> {
        let metadata = SubworkflowEventMetadata {
            parent_run_id: DRIFTED_PARENT_RUN_ID,
            subworkflow_node_id: invocation.correlation.subworkflow_node_id,
            child_run_id: CHILD_RUN_ID,
            workflow_name: invocation.workflow_name.clone(),
        };
        Ok(SubworkflowExecution {
            workflow_name: invocation.workflow_name.clone(),
            child_run_id: CHILD_RUN_ID,
            outputs: BTreeMap::from([(CHILD_OUTPUT.to_string(), Value::from_string(CHILD_RESULT.to_string()))]),
            events: vec![
                SubworkflowEvent {
                    kind: SubworkflowEventKind::Started,
                    metadata: metadata.clone(),
                },
                SubworkflowEvent {
                    kind: SubworkflowEventKind::Completed,
                    metadata,
                },
            ],
        })
    }
}

#[tokio::test]
async fn missing_runner_missing_child_output_and_correlation_drift_fail_without_outputs() {
    let (parent, _) = parent_workflow(CHILD_OUTPUT);
    let mut unavailable = Executor::new();
    let (_, unavailable_steps) = unavailable
        .run_with_named_inputs(&parent, &HashMap::new(), &named_parent_inputs())
        .await
        .expect("bind unavailable inputs");
    assert!(matches!(unavailable_steps[0].status, NodeStatus::Failed { .. }));

    let temporary = tempfile::tempdir().expect("temporary store directory");
    let store = Arc::new(Store::new(temporary.path().join("lattice.redb")).expect("open store"));
    let mut child = Workflow::new(CHILD_WORKFLOW, lattice::engine::workflow::WorkflowTimestamp::epoch());
    child.add_shell_node("Child result", format!("printf {CHILD_RESULT}"));
    store.save_workflow(&child).expect("save child workflow");
    let (missing_output_parent, _) = parent_workflow("missing-output");
    let mut missing_output =
        Executor::new_with_subworkflow_runner_shell(Box::new(RepositorySubworkflowRunnerShell::new(store)));
    let (_, missing_output_steps) = missing_output
        .run_with_named_inputs(&missing_output_parent, &HashMap::new(), &named_parent_inputs())
        .await
        .expect("bind missing-output inputs");
    assert!(matches!(missing_output_steps[0].status, NodeStatus::Failed { .. }));

    let mut drifted = Executor::new_with_subworkflow_runner_shell(Box::new(DriftedCorrelationShell));
    let (_, drifted_steps) = drifted
        .run_with_named_inputs(&parent, &HashMap::new(), &named_parent_inputs())
        .await
        .expect("bind drifted inputs");
    assert!(matches!(drifted_steps[0].status, NodeStatus::Failed { .. }));
}

#[tokio::test]
async fn recursive_child_reference_is_rejected_before_runner_dispatch() {
    let mut workflow = Workflow::new(PARENT_WORKFLOW, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = workflow.add_subworkflow_node("Recursive child", SubworkflowNodeConfig {
        workflow_name: PARENT_WORKFLOW.to_string(),
        input_mappings: Vec::new(),
        output_mappings: Vec::new(),
    });
    assert!(matches!(workflow.node_config(node_id), Some(NodeConfig::Subworkflow(_))));

    let mut executor = Executor::new();
    let (_, steps) = executor.run(&workflow, &HashMap::new()).await;
    let NodeStatus::Failed { error } = &steps[0].status else {
        panic!("recursive subworkflow must fail");
    };
    assert!(error.contains("recursive subworkflow reference"));
}
