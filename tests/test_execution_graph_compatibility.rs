use std::collections::BTreeMap;

use seaglass_execution_graph::ExecutionEdge;
use seaglass_execution_graph::ExecutionGraph;
use seaglass_execution_graph::ExecutionNode;
use seaglass_execution_graph::GraphDiagnosticCode;
use seaglass_execution_graph::ResumeDecision;
use seaglass_execution_graph::StepAction;
use seaglass_execution_graph::StepState;
use seaglass_execution_graph::admit_graph;
use seaglass_execution_graph::ready_nodes;
use seaglass_execution_graph::resume_decision;
use seaglass_execution_graph::transition_step_state;

const SOURCE: &str = "source";
const LEFT: &str = "left";
const RIGHT: &str = "right";
const SINK: &str = "sink";
const MISSING: &str = "missing";
const ROOT_DEPENDENT_COUNT: u32 = 3;

fn diamond_graph() -> ExecutionGraph {
    ExecutionGraph::new(
        vec![
            ExecutionNode::new(SOURCE),
            ExecutionNode::new(LEFT),
            ExecutionNode::new(RIGHT),
            ExecutionNode::new(SINK),
        ],
        vec![
            ExecutionEdge::new(SOURCE, LEFT),
            ExecutionEdge::new(SOURCE, RIGHT),
            ExecutionEdge::new(LEFT, SINK),
            ExecutionEdge::new(RIGHT, SINK),
        ],
    )
}

// r[verify lattice.execution_graph_external_adoption.compatibility]
// r[verify lattice.execution_graph_external_adoption.compatibility.scenario.agree]
#[test]
fn external_core_preserves_selected_graph_and_lifecycle_decisions() {
    let plan = admit_graph(&diamond_graph()).expect("diamond graph must be admitted");
    assert_eq!(plan.topological_order, [SOURCE, LEFT, RIGHT, SINK]);
    assert_eq!(plan.dependencies(SINK), [LEFT, RIGHT]);
    assert_eq!(plan.dependents(SOURCE), [LEFT, RIGHT]);

    let groups = plan.parallel_groups();
    assert_eq!(groups.len(), ROOT_DEPENDENT_COUNT as usize);
    assert_eq!(groups[1].node_ids, [LEFT, RIGHT]);
    assert!(groups[1].can_parallelize);
    assert_eq!(plan.critical_path(), [SOURCE, LEFT, SINK]);
    assert_eq!(plan.bottlenecks()[0].node_id, SOURCE);
    assert_eq!(plan.bottlenecks()[0].dependent_count, ROOT_DEPENDENT_COUNT);

    let mut states = BTreeMap::new();
    assert_eq!(ready_nodes(&plan, &states), [SOURCE]);
    assert_eq!(resume_decision(&plan, &states, SINK), ResumeDecision::AwaitDependencies);
    states.insert(SOURCE.to_owned(), StepState::Completed);
    assert_eq!(ready_nodes(&plan, &states), [LEFT, RIGHT]);
    assert_eq!(transition_step_state(StepState::Pending, StepAction::MarkReady), Ok(StepState::Ready));
}

// r[verify lattice.execution_graph_external_adoption.compatibility.scenario.differ]
#[test]
fn external_core_rejects_malformed_cycles_and_terminal_mutation() {
    let malformed = ExecutionGraph::new(vec![ExecutionNode::new(SOURCE)], vec![ExecutionEdge::new(SOURCE, MISSING)]);
    let malformed_diagnostics = admit_graph(&malformed).expect_err("missing endpoint must fail");
    assert!(
        malformed_diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == GraphDiagnosticCode::MissingTargetNode })
    );

    let cycle = ExecutionGraph::new(vec![ExecutionNode::new(SOURCE), ExecutionNode::new(SINK)], vec![
        ExecutionEdge::new(SOURCE, SINK),
        ExecutionEdge::new(SINK, SOURCE),
    ]);
    let cycle_diagnostics = admit_graph(&cycle).expect_err("cycle must fail");
    assert!(cycle_diagnostics.iter().any(|diagnostic| diagnostic.code == GraphDiagnosticCode::CycleDetected));

    assert!(transition_step_state(StepState::Completed, StepAction::Cancel).is_err());
}
