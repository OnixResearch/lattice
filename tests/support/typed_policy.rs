//! Shared log-triage routing fixture for the typed policy operation (Lattice #23).
//!
//! The checked-in `examples/workflows/log-triage.ncl` workflow is loaded unchanged and extended in
//! memory with a feature node, one declared `route incident` Condition node and one continuation
//! per declared branch.

#![allow(dead_code, reason = "shared by the focused test target and the measurement example")]

use std::path::Path;

use lattice::control_flow::ConditionBranch;
use lattice::control_flow::ConditionNodeConfig;
use lattice::control_flow::ConditionPredicate;
use lattice::control_flow::MergeNodeConfig;
use lattice::control_flow::MergePolicy;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::Workflow;

pub const LOG_TRIAGE_PATH: &str = "examples/workflows/log-triage.ncl";
pub const PROFILE_PATH: &str = "tests/fixtures/typed-policy/profile-log-triage-route-v1.json";
pub const ARTIFACT_PATH: &str = "tests/fixtures/typed-policy/policy-log-triage-tree-v1.json";
pub const CASES_PATH: &str = "tests/fixtures/typed-policy/cases.json";
pub const WORKFLOW_NAME: &str = "log-triage";
pub const SOURCE_LABEL: &str = "log source";
pub const FILTER_LABEL: &str = "filter incidents";
pub const FEATURES_LABEL: &str = "extract features";
pub const ROUTE_LABEL: &str = "route incident";
pub const LABELS: [&str; 3] = ["page", "ticket", "ignore"];
pub const CONTINUATIONS: [(&str, &str); 3] = [
    ("page", "page on-call"),
    ("ticket", "open ticket"),
    ("ignore", "archive incident"),
];
const STDIN: &str = "stdin";
const STDOUT: &str = "stdout";
const CONDITION_INPUT: &str = "value";
const MERGED: &str = "merged";
/// Counts WARN/ERROR lines, the largest `latency <n>ms`, ERROR lines on payments/auth, and the
/// deterministic rule hint: `page` on any ERROR, else `ticket` on any WARN, else `ignore`. The
/// command avoids `{{`/`}}`, which are Lattice expression delimiters.
pub const FEATURES_COMMAND: &str = r#"awk 'BEGIN { e = 0; w = 0; m = 0; c = 0 } / ERROR / { e++; if ($0 ~ /\/api\/(payments|auth)/) c++ } / WARN / { w++ } { for (i = 1; i < NF; i++) if ($i == "latency" && $(i + 1) ~ /^[0-9]+ms$/) { v = $(i + 1); sub(/ms$/, "", v); if (v + 0 > m) m = v + 0 } } END { r = "ignore"; if (e > 0) r = "page"; else if (w > 0) r = "ticket"; printf "{\"features\":{\"error_count\":%d,\"warn_count\":%d,\"max_latency_ms\":%d,\"critical_errors\":%d},\"rule\":{\"%s\":true} }\n", e, w, m, c, r }'"#;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Case {
    pub id: String,
    pub family: String,
    pub expected: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Cases {
    pub schema: String,
    pub labels: Vec<String>,
    pub authoring_families: Vec<String>,
    pub held_out_families: Vec<String>,
    pub cases: Vec<Case>,
}

#[derive(Debug, Clone, Copy)]
pub struct RoutingNodes {
    pub source: NodeId,
    pub features: NodeId,
    pub route: NodeId,
    pub continuations: [NodeId; 3],
}

pub fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(Path::new(path)).expect("typed policy fixture reads")
}

pub fn cases() -> Cases {
    serde_json::from_slice(&read_fixture(CASES_PATH)).expect("typed policy cases parse")
}

pub fn artifact() -> Vec<u8> {
    read_fixture(ARTIFACT_PATH)
}

pub fn profile_json() -> serde_json::Value {
    serde_json::from_slice(&read_fixture(PROFILE_PATH)).expect("typed policy profile parses")
}

pub fn admit(profile: &serde_json::Value) -> lattice::workflow::decision::admission::Admitted {
    let bytes = serde_json::to_vec(profile).expect("profile encodes");
    lattice::workflow::decision::admission::admit(&bytes).expect("typed policy profile admits")
}

pub fn runtime(
    profile: &serde_json::Value,
    adapter: std::sync::Arc<dyn lattice::workflow::decision::Adapter>,
) -> lattice::workflow::decision::Runtime {
    lattice::workflow::decision::Runtime {
        profiles: vec![admit(profile)],
        adapter,
    }
}

pub fn leaf_adapter() -> std::sync::Arc<dyn lattice::workflow::decision::Adapter> {
    std::sync::Arc::new(lattice::animus_agent::frozen::LeafAdapter::new(artifact()))
}

/// A shell `printf` emitting `lines`, one per line.
pub fn printf_command(lines: &[String]) -> String {
    let quoted = lines.iter().map(|line| format!("'{line}'")).collect::<Vec<_>>().join(" ");
    format!("printf '%s\\n' {quoted}")
}

/// The log-triage fixture extended with the declared routing Condition and its continuations.
pub fn routing_workflow(lines: &[String]) -> (Workflow, RoutingNodes) {
    let mut workflow = lattice::workflow_format::load_workflow_nickel_file(Path::new(LOG_TRIAGE_PATH))
        .expect("log-triage fixture loads");
    assert_eq!(workflow.meta.name, WORKFLOW_NAME);
    let source = node_by_label(&workflow, SOURCE_LABEL);
    let filter = node_by_label(&workflow, FILTER_LABEL);
    set_shell_command(&mut workflow, source, &printf_command(lines));
    let features = workflow.add_shell_node(FEATURES_LABEL, FEATURES_COMMAND);
    let route = workflow.add_condition_node(ROUTE_LABEL, ConditionNodeConfig {
        branches: LABELS
            .iter()
            .map(|label| ConditionBranch {
                label: (*label).to_owned(),
                predicate: ConditionPredicate::StringContains(format!("\"rule\":{{\"{label}\":true}}")),
            })
            .collect(),
    });
    connect(&mut workflow, (filter, STDOUT), (features, STDIN));
    connect(&mut workflow, (features, STDOUT), (route, CONDITION_INPUT));
    // A Condition branch port that emits nothing does not by itself skip its consumers; a
    // required-input Merge gate per branch is the existing construct that skips them.
    let continuations = CONTINUATIONS.map(|(label, node_label)| {
        let gate = workflow.add_merge_node(format!("{label} gate"), MergeNodeConfig {
            policy: MergePolicy::First,
            required_inputs: vec![label.to_owned()],
        });
        let node = workflow.add_shell_node(node_label, format!("cat > /dev/null; echo {label}"));
        connect(&mut workflow, (route, label), (gate, label));
        connect(&mut workflow, (gate, MERGED), (node, STDIN));
        node
    });
    (workflow, RoutingNodes {
        source,
        features,
        route,
        continuations,
    })
}

pub fn node_by_label(workflow: &Workflow, label: &str) -> NodeId {
    workflow
        .graph
        .nodes()
        .find(|node| node.label == label)
        .map(|node| node.id)
        .unwrap_or_else(|| panic!("node `{label}` exists"))
}

pub fn set_shell_command(workflow: &mut Workflow, node: NodeId, command: &str) {
    let Some(NodeConfig::Shell {
        cwd,
        env,
        timeout_secs,
        pipe_stdin,
        ..
    }) = workflow.node_config(node).cloned()
    else {
        panic!("node is a shell node");
    };
    workflow.set_node_config(node, NodeConfig::Shell {
        command: command.to_owned(),
        cwd,
        env,
        timeout_secs,
        pipe_stdin,
    });
}

pub fn port(workflow: &Workflow, node: NodeId, label: &str, is_output: bool) -> PortId {
    let found = workflow.graph.node(node).and_then(|node| {
        let ports = if is_output {
            &node.output_ports
        } else {
            &node.input_ports
        };
        ports.iter().find(|port| port.label == label).map(|port| port.id)
    });
    found.unwrap_or_else(|| panic!("port `{label}` exists"))
}

pub fn connect(workflow: &mut Workflow, source: (NodeId, &str), target: (NodeId, &str)) {
    let from = port(workflow, source.0, source.1, true);
    let to = port(workflow, target.0, target.1, false);
    workflow.graph.add_edge(from, to).expect("fixture edge admits");
}

/// The label whose continuation completed, if exactly one did.
pub fn routed_label(
    results: &[lattice::engine::executor::StepResult],
    nodes: &RoutingNodes,
) -> Result<Option<String>, String> {
    let completed = CONTINUATIONS
        .iter()
        .zip(nodes.continuations)
        .filter(|(_, node)| {
            results.iter().any(|step| {
                step.node_id == *node && matches!(step.status, lattice::engine::executor::NodeStatus::Completed { .. })
            })
        })
        .map(|((label, _), _)| (*label).to_owned())
        .collect::<Vec<_>>();
    match completed.as_slice() {
        [] => Ok(None),
        [label] => Ok(Some(label.clone())),
        _ => Err(format!("more than one continuation ran: {completed:?}")),
    }
}
