#![no_std]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;

const EMPTY_DEPENDENCY_COUNT: usize = 0;
const ROOT_LEVEL: u32 = 0;
const NEXT_LEVEL_INCREMENT: u32 = 1;
const MIN_PARALLEL_NODE_COUNT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionNode {
    pub id: String,
}

impl ExecutionNode {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct ExecutionEdge {
    pub source: String,
    pub target: String,
}

impl ExecutionEdge {
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionGraph {
    pub nodes: Vec<ExecutionNode>,
    pub edges: Vec<ExecutionEdge>,
}

impl ExecutionGraph {
    pub fn new(nodes: Vec<ExecutionNode>, edges: Vec<ExecutionEdge>) -> Self {
        Self { nodes, edges }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum GraphDiagnosticCode {
    EmptyNodeId,
    DuplicateNodeId,
    DuplicateEdge,
    MissingSourceNode,
    MissingTargetNode,
    SelfLoop,
    CycleDetected,
    GraphTooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphDiagnostic {
    pub code: GraphDiagnosticCode,
    pub node_id: Option<String>,
    pub related_node_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    graph: ExecutionGraph,
    pub topological_order: Vec<String>,
}

impl ExecutionPlan {
    pub fn graph(&self) -> &ExecutionGraph {
        &self.graph
    }

    pub fn dependencies(&self, node_id: &str) -> Vec<String> {
        direct_dependencies(&self.graph, node_id)
    }

    pub fn dependents(&self, node_id: &str) -> Vec<String> {
        direct_dependents(&self.graph, node_id)
    }

    pub fn parallel_groups(&self) -> Vec<ExecutionGroup> {
        parallel_groups(self)
    }

    pub fn critical_path(&self) -> Vec<String> {
        critical_path(self)
    }

    pub fn bottlenecks(&self) -> Vec<Bottleneck> {
        bottlenecks(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionGroup {
    pub node_ids: Vec<String>,
    pub level: u32,
    pub can_parallelize: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bottleneck {
    pub node_id: String,
    pub dependent_count: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepAction {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepTransitionError {
    InvalidTransition { from: StepState, action: StepAction },
    TerminalState { from: StepState, action: StepAction },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeDecision {
    Execute,
    AwaitDependencies,
    SkipCompleted,
    RejectTerminalMutation,
    BlockedByFailedDependency,
}

pub fn admit_graph(graph: &ExecutionGraph) -> Result<ExecutionPlan, Vec<GraphDiagnostic>> {
    let diagnostics = validate_static_graph(graph);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let order = topological_order_unchecked(graph);
    if order.len() != graph.nodes.len() {
        return Err(alloc::vec![diagnostic(
            GraphDiagnosticCode::CycleDetected,
            None,
            None,
            "execution graph contains a cycle",
        )]);
    }

    Ok(ExecutionPlan {
        graph: graph.clone(),
        topological_order: order,
    })
}

pub fn validate_graph(graph: &ExecutionGraph) -> Vec<GraphDiagnostic> {
    match admit_graph(graph) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics,
    }
}

fn validate_static_graph(graph: &ExecutionGraph) -> Vec<GraphDiagnostic> {
    let mut diagnostics = Vec::new();
    if u32::try_from(graph.nodes.len()).is_err() {
        diagnostics.push(diagnostic(
            GraphDiagnosticCode::GraphTooLarge,
            None,
            None,
            "execution graph has more nodes than the supported level and analytics range",
        ));
    }

    let mut node_ids = BTreeSet::new();
    for node in &graph.nodes {
        if node.id.trim().is_empty() {
            diagnostics.push(diagnostic(
                GraphDiagnosticCode::EmptyNodeId,
                Some(node.id.clone()),
                None,
                "execution graph node id cannot be empty",
            ));
            continue;
        }
        if !node_ids.insert(node.id.clone()) {
            diagnostics.push(diagnostic(
                GraphDiagnosticCode::DuplicateNodeId,
                Some(node.id.clone()),
                None,
                "execution graph node id must be unique",
            ));
        }
    }

    let mut edges = BTreeSet::new();
    for edge in &graph.edges {
        if !node_ids.contains(&edge.source) {
            diagnostics.push(diagnostic(
                GraphDiagnosticCode::MissingSourceNode,
                Some(edge.source.clone()),
                Some(edge.target.clone()),
                "execution graph edge source node is missing",
            ));
        }
        if !node_ids.contains(&edge.target) {
            diagnostics.push(diagnostic(
                GraphDiagnosticCode::MissingTargetNode,
                Some(edge.target.clone()),
                Some(edge.source.clone()),
                "execution graph edge target node is missing",
            ));
        }
        if edge.source == edge.target {
            diagnostics.push(diagnostic(
                GraphDiagnosticCode::SelfLoop,
                Some(edge.source.clone()),
                None,
                "execution graph edge cannot connect a node to itself",
            ));
        }
        if !edges.insert((edge.source.clone(), edge.target.clone())) {
            diagnostics.push(diagnostic(
                GraphDiagnosticCode::DuplicateEdge,
                Some(edge.source.clone()),
                Some(edge.target.clone()),
                "execution graph edge must be unique",
            ));
        }
    }

    diagnostics
}

fn topological_order_unchecked(graph: &ExecutionGraph) -> Vec<String> {
    let mut incoming = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), EMPTY_DEPENDENCY_COUNT))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = BTreeMap::<String, BTreeSet<String>>::new();

    for edge in &graph.edges {
        if let Some(count) = incoming.get_mut(&edge.target) {
            *count = count.saturating_add(1);
        }
        outgoing
            .entry(edge.source.clone())
            .or_default()
            .insert(edge.target.clone());
    }

    let mut ready = incoming
        .iter()
        .filter_map(|(node_id, count)| {
            (*count == EMPTY_DEPENDENCY_COUNT).then_some(node_id.clone())
        })
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(graph.nodes.len());

    while let Some(node_id) = pop_first(&mut ready) {
        order.push(node_id.clone());
        if let Some(targets) = outgoing.get(&node_id) {
            for target in targets {
                decrement_incoming(&mut incoming, &mut ready, target);
            }
        }
    }

    order
}

fn pop_first(values: &mut BTreeSet<String>) -> Option<String> {
    let value = values.first()?.clone();
    values.remove(&value);
    Some(value)
}

fn decrement_incoming(
    incoming: &mut BTreeMap<String, usize>,
    ready: &mut BTreeSet<String>,
    node_id: &str,
) {
    let Some(count) = incoming.get_mut(node_id) else {
        return;
    };
    if *count == EMPTY_DEPENDENCY_COUNT {
        return;
    }
    *count = count.saturating_sub(1);
    if *count == EMPTY_DEPENDENCY_COUNT {
        ready.insert(node_id.to_string());
    }
}

pub fn direct_dependencies(graph: &ExecutionGraph, node_id: &str) -> Vec<String> {
    graph
        .edges
        .iter()
        .filter(|edge| edge.target == node_id)
        .map(|edge| edge.source.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn direct_dependents(graph: &ExecutionGraph, node_id: &str) -> Vec<String> {
    graph
        .edges
        .iter()
        .filter(|edge| edge.source == node_id)
        .map(|edge| edge.target.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn parallel_groups(plan: &ExecutionPlan) -> Vec<ExecutionGroup> {
    let mut levels = plan
        .graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), ROOT_LEVEL))
        .collect::<BTreeMap<_, _>>();

    for node_id in &plan.topological_order {
        let level = match levels.get(node_id) {
            Some(level) => *level,
            None => ROOT_LEVEL,
        };
        for dependent in plan.dependents(node_id) {
            let candidate = level.saturating_add(NEXT_LEVEL_INCREMENT);
            if let Some(current) = levels.get_mut(&dependent) {
                *current = (*current).max(candidate);
            }
        }
    }

    let mut grouped = BTreeMap::<u32, Vec<String>>::new();
    for node_id in &plan.topological_order {
        let level = match levels.get(node_id) {
            Some(level) => *level,
            None => ROOT_LEVEL,
        };
        grouped.entry(level).or_default().push(node_id.clone());
    }

    grouped
        .into_iter()
        .map(|(level, node_ids)| ExecutionGroup {
            can_parallelize: node_ids.len() >= MIN_PARALLEL_NODE_COUNT,
            node_ids,
            level,
        })
        .collect()
}

fn critical_path(plan: &ExecutionPlan) -> Vec<String> {
    let mut paths = BTreeMap::<String, Vec<String>>::new();

    for node_id in &plan.topological_order {
        let mut best = alloc::vec![node_id.clone()];
        for dependency in plan.dependencies(node_id) {
            let Some(prefix) = paths.get(&dependency) else {
                continue;
            };
            let mut candidate = prefix.clone();
            candidate.push(node_id.clone());
            if path_precedes(&candidate, &best) {
                best = candidate;
            }
        }
        paths.insert(node_id.clone(), best);
    }

    paths.into_values().fold(Vec::new(), |best, candidate| {
        if path_precedes(&candidate, &best) {
            candidate
        } else {
            best
        }
    })
}

fn path_precedes(candidate: &[String], current: &[String]) -> bool {
    match candidate.len().cmp(&current.len()) {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => candidate < current,
    }
}

fn bottlenecks(plan: &ExecutionPlan) -> Vec<Bottleneck> {
    let mut result = plan
        .topological_order
        .iter()
        .map(|node_id| Bottleneck {
            node_id: node_id.clone(),
            dependent_count: transitive_dependent_count(plan, node_id),
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| {
        right
            .dependent_count
            .cmp(&left.dependent_count)
            .then_with(|| left.node_id.cmp(&right.node_id))
    });
    result
}

fn transitive_dependent_count(plan: &ExecutionPlan, node_id: &str) -> u32 {
    let mut visited = BTreeSet::new();
    let mut pending = plan.dependents(node_id);
    let max_visits = plan.graph.nodes.len();
    let mut visits = EMPTY_DEPENDENCY_COUNT;

    while let Some(candidate) = pending.pop() {
        if !visited.insert(candidate.clone()) {
            continue;
        }
        visits = visits.saturating_add(1);
        if visits > max_visits {
            break;
        }
        pending.extend(plan.dependents(&candidate));
    }

    match u32::try_from(visited.len()) {
        Ok(count) => count,
        Err(_) => u32::MAX,
    }
}

pub fn ready_nodes(plan: &ExecutionPlan, states: &BTreeMap<String, StepState>) -> Vec<String> {
    plan.topological_order
        .iter()
        .filter(|node_id| state_for(states, node_id) == StepState::Pending)
        .filter(|node_id| dependencies_completed(plan, states, node_id))
        .cloned()
        .collect()
}

pub fn transition_step_state(
    from: StepState,
    action: StepAction,
) -> Result<StepState, StepTransitionError> {
    if is_terminal_state(from) {
        return Err(StepTransitionError::TerminalState { from, action });
    }
    match (from, action) {
        (StepState::Pending, StepAction::MarkReady) => Ok(StepState::Ready),
        (StepState::Ready, StepAction::Start) => Ok(StepState::Running),
        (StepState::Running, StepAction::Complete) => Ok(StepState::Completed),
        (StepState::Running, StepAction::Fail) => Ok(StepState::Failed),
        (StepState::Pending | StepState::Ready | StepState::Running, StepAction::Cancel) => {
            Ok(StepState::Cancelled)
        }
        _ => Err(StepTransitionError::InvalidTransition { from, action }),
    }
}

pub fn is_terminal_state(state: StepState) -> bool {
    matches!(
        state,
        StepState::Completed | StepState::Failed | StepState::Cancelled
    )
}

pub fn resume_decision(
    plan: &ExecutionPlan,
    states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> ResumeDecision {
    match state_for(states, node_id) {
        StepState::Completed => ResumeDecision::SkipCompleted,
        StepState::Failed | StepState::Cancelled => ResumeDecision::RejectTerminalMutation,
        StepState::Pending | StepState::Ready | StepState::Running => {
            resume_non_terminal(plan, states, node_id)
        }
    }
}

fn resume_non_terminal(
    plan: &ExecutionPlan,
    states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> ResumeDecision {
    if has_failed_dependency(plan, states, node_id) {
        return ResumeDecision::BlockedByFailedDependency;
    }
    if dependencies_completed(plan, states, node_id) {
        ResumeDecision::Execute
    } else {
        ResumeDecision::AwaitDependencies
    }
}

fn state_for(states: &BTreeMap<String, StepState>, node_id: &str) -> StepState {
    match states.get(node_id) {
        Some(state) => *state,
        None => StepState::Pending,
    }
}

fn dependencies_completed(
    plan: &ExecutionPlan,
    states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> bool {
    plan.dependencies(node_id)
        .iter()
        .all(|dependency| state_for(states, dependency) == StepState::Completed)
}

fn has_failed_dependency(
    plan: &ExecutionPlan,
    states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> bool {
    plan.dependencies(node_id).iter().any(|dependency| {
        matches!(
            state_for(states, dependency),
            StepState::Failed | StepState::Cancelled
        )
    })
}

pub fn would_create_cycle(graph: &ExecutionGraph, source: &str, target: &str) -> bool {
    if source == target {
        return true;
    }
    let node_ids = graph
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    if !node_ids.contains(source) || !node_ids.contains(target) {
        return false;
    }

    let mut visited = BTreeSet::new();
    let mut pending = alloc::vec![target.to_string()];
    let max_visits = graph.nodes.len();
    let mut visits = EMPTY_DEPENDENCY_COUNT;

    while let Some(node_id) = pending.pop() {
        if node_id == source {
            return true;
        }
        if !visited.insert(node_id.clone()) {
            continue;
        }
        visits = visits.saturating_add(1);
        if visits > max_visits {
            return true;
        }
        pending.extend(direct_dependents(graph, &node_id));
    }

    false
}

fn diagnostic(
    code: GraphDiagnosticCode,
    node_id: Option<String>,
    related_node_id: Option<String>,
    message: &str,
) -> GraphDiagnostic {
    GraphDiagnostic {
        code,
        node_id,
        related_node_id,
        message: message.to_string(),
    }
}

impl core::fmt::Display for GraphDiagnostic {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let context = match (&self.node_id, &self.related_node_id) {
            (Some(node_id), Some(related)) => format!(" ({node_id} -> {related})"),
            (Some(node_id), None) => format!(" ({node_id})"),
            (None, Some(related)) => format!(" ({related})"),
            (None, None) => String::new(),
        };
        write!(formatter, "{}{context}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NODE_A: &str = "a";
    const NODE_B: &str = "b";
    const NODE_C: &str = "c";
    const NODE_D: &str = "d";
    const MISSING_NODE: &str = "missing";
    const EXPECTED_DIAMOND_NODE_COUNT: usize = 4;
    const EXPECTED_DIAMOND_EDGE_COUNT: usize = 4;
    const EXPECTED_GROUP_COUNT: usize = 3;
    const EXPECTED_FANOUT_WIDTH: usize = 2;
    const EXPECTED_ROOT_DEPENDENT_COUNT: u32 = 3;

    fn diamond_graph() -> ExecutionGraph {
        ExecutionGraph::new(
            alloc::vec![
                ExecutionNode::new(NODE_D),
                ExecutionNode::new(NODE_C),
                ExecutionNode::new(NODE_B),
                ExecutionNode::new(NODE_A),
            ],
            alloc::vec![
                ExecutionEdge::new(NODE_A, NODE_C),
                ExecutionEdge::new(NODE_C, NODE_D),
                ExecutionEdge::new(NODE_A, NODE_B),
                ExecutionEdge::new(NODE_B, NODE_D),
            ],
        )
    }

    #[test]
    fn admitted_graph_has_stable_order_groups_and_analytics() {
        let graph = diamond_graph();
        let plan = admit_graph(&graph).expect("diamond graph should be admitted");

        assert_eq!(plan.graph().nodes.len(), EXPECTED_DIAMOND_NODE_COUNT);
        assert_eq!(plan.graph().edges.len(), EXPECTED_DIAMOND_EDGE_COUNT);
        assert_eq!(
            plan.topological_order,
            alloc::vec![NODE_A, NODE_B, NODE_C, NODE_D]
        );
        assert_eq!(plan.dependencies(NODE_D), alloc::vec![NODE_B, NODE_C]);
        assert_eq!(plan.dependents(NODE_A), alloc::vec![NODE_B, NODE_C]);

        let groups = plan.parallel_groups();
        assert_eq!(groups.len(), EXPECTED_GROUP_COUNT);
        assert_eq!(groups[1].node_ids.len(), EXPECTED_FANOUT_WIDTH);
        assert!(groups[1].can_parallelize);
        assert_eq!(plan.critical_path(), alloc::vec![NODE_A, NODE_B, NODE_D]);

        let bottlenecks = plan.bottlenecks();
        assert_eq!(bottlenecks[0].node_id, NODE_A);
        assert_eq!(
            bottlenecks[0].dependent_count,
            EXPECTED_ROOT_DEPENDENT_COUNT
        );
    }

    #[test]
    fn empty_graph_is_a_valid_no_work_plan() {
        let plan = admit_graph(&ExecutionGraph::default()).expect("empty graph should be valid");
        assert!(plan.topological_order.is_empty());
        assert!(plan.parallel_groups().is_empty());
        assert!(plan.critical_path().is_empty());
        assert!(plan.bottlenecks().is_empty());
    }

    #[test]
    fn static_admission_rejects_malformed_graph_facts() {
        let graph = ExecutionGraph::new(
            alloc::vec![
                ExecutionNode::new(NODE_A),
                ExecutionNode::new(NODE_A),
                ExecutionNode::new(""),
            ],
            alloc::vec![
                ExecutionEdge::new(NODE_A, NODE_A),
                ExecutionEdge::new(NODE_A, MISSING_NODE),
                ExecutionEdge::new(MISSING_NODE, NODE_A),
                ExecutionEdge::new(NODE_A, MISSING_NODE),
            ],
        );

        let diagnostics = admit_graph(&graph).expect_err("malformed graph should be rejected");
        let codes = diagnostics
            .iter()
            .map(|item| item.code)
            .collect::<BTreeSet<_>>();
        assert!(codes.contains(&GraphDiagnosticCode::DuplicateNodeId));
        assert!(codes.contains(&GraphDiagnosticCode::EmptyNodeId));
        assert!(codes.contains(&GraphDiagnosticCode::SelfLoop));
        assert!(codes.contains(&GraphDiagnosticCode::MissingSourceNode));
        assert!(codes.contains(&GraphDiagnosticCode::MissingTargetNode));
        assert!(codes.contains(&GraphDiagnosticCode::DuplicateEdge));
    }

    #[test]
    fn admission_rejects_cycle() {
        let graph = ExecutionGraph::new(
            alloc::vec![ExecutionNode::new(NODE_A), ExecutionNode::new(NODE_B)],
            alloc::vec![
                ExecutionEdge::new(NODE_A, NODE_B),
                ExecutionEdge::new(NODE_B, NODE_A),
            ],
        );

        let diagnostics = admit_graph(&graph).expect_err("cycle should be rejected");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, GraphDiagnosticCode::CycleDetected);
    }

    #[test]
    fn ready_and_resume_decisions_require_completed_dependencies() {
        let plan = admit_graph(&diamond_graph()).expect("diamond graph should be admitted");
        let mut states = BTreeMap::new();
        assert_eq!(ready_nodes(&plan, &states), alloc::vec![NODE_A]);
        assert_eq!(
            resume_decision(&plan, &states, NODE_B),
            ResumeDecision::AwaitDependencies
        );

        states.insert(NODE_A.to_string(), StepState::Completed);
        assert_eq!(ready_nodes(&plan, &states), alloc::vec![NODE_B, NODE_C]);
        assert_eq!(
            resume_decision(&plan, &states, NODE_B),
            ResumeDecision::Execute
        );
        assert_eq!(
            resume_decision(&plan, &states, NODE_A),
            ResumeDecision::SkipCompleted
        );

        states.insert(NODE_A.to_string(), StepState::Failed);
        assert!(ready_nodes(&plan, &states).is_empty());
        assert_eq!(
            resume_decision(&plan, &states, NODE_B),
            ResumeDecision::BlockedByFailedDependency
        );
    }

    #[test]
    fn lifecycle_accepts_valid_paths_and_rejects_invalid_or_terminal_mutation() {
        let ready = transition_step_state(StepState::Pending, StepAction::MarkReady)
            .expect("pending step should become ready");
        let running =
            transition_step_state(ready, StepAction::Start).expect("ready step should start");
        let completed = transition_step_state(running, StepAction::Complete)
            .expect("running step should complete");
        assert_eq!(completed, StepState::Completed);

        assert!(transition_step_state(StepState::Pending, StepAction::Complete).is_err());
        assert!(transition_step_state(StepState::Completed, StepAction::Cancel).is_err());
        assert_eq!(
            transition_step_state(StepState::Running, StepAction::Fail)
                .expect("running step should fail"),
            StepState::Failed
        );
    }

    #[test]
    fn cycle_probe_checks_target_to_source_reachability() {
        let graph = ExecutionGraph::new(
            alloc::vec![ExecutionNode::new(NODE_A), ExecutionNode::new(NODE_B)],
            alloc::vec![ExecutionEdge::new(NODE_A, NODE_B)],
        );
        assert!(would_create_cycle(&graph, NODE_B, NODE_A));
        assert!(!would_create_cycle(&graph, NODE_A, NODE_B));
        assert!(!would_create_cycle(&graph, NODE_A, MISSING_NODE));
    }
}
