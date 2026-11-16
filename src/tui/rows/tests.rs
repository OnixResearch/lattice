use super::*;
use crate::engine::executor::RunId;
use crate::engine::workflow::WorkflowTimestamp;

fn workflow_with_edges(edges: &[(usize, usize)], nodes: usize) -> Workflow {
    let mut workflow = Workflow::new("rows", WorkflowTimestamp::epoch());
    let ids = (0..nodes).map(|index| workflow.add_shell_node(format!("node-{index}"), "true")).collect::<Vec<_>>();
    for &(source, target) in edges {
        let source_port = workflow.graph.node(ids[source]).expect("source").output_ports[0].id;
        let target_port = workflow.graph.node(ids[target]).expect("target").input_ports[0].id;
        workflow.graph.add_edge(source_port, target_port).expect("edge");
    }
    workflow
}

fn render(workflow: &Workflow, layout: &Layout, width: u16, height: u16, scroll: usize) -> Buffer {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    super::render(
        layout,
        workflow,
        RenderData {
            statuses: &HashMap::new(),
            started: &HashMap::new(),
            start_times: &HashMap::new(),
            durations_millis: &HashMap::new(),
            tokens: &HashMap::new(),
            costs_microunits: &HashMap::new(),
            attempts: &HashMap::new(),
            outputs: &[],
            selected: &BTreeSet::new(),
            focused: None,
            scroll,
        },
        buffer.area,
        &mut buffer,
    );
    buffer
}

fn buffer_lines(buffer: &Buffer) -> Vec<String> {
    buffer
        .content()
        .chunks(usize::from(buffer.area.width))
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>().trim_end().to_string())
        .collect()
}

/// Snapshot the actual rendered topology/name columns, keeping metadata tests
/// separate so elapsed wall time never makes the workflow snapshots unstable.
fn topology_snapshot(workflow: &Workflow) -> String {
    let layout = layout(workflow, DEFAULT_MAX_LANES).expect("fixture layout");
    let buffer = render(workflow, &layout, 22, u16::try_from(layout.rows.len() + 2).expect("height"), 0);
    let mut lines = buffer_lines(&buffer);
    lines.pop(); // aggregate footer is covered by metadata assertions
    format!("{}\n", lines.join("\n"))
}

// r[verify lattice.tui_ux.rows.topology]
#[test]
fn linear_workflow_snapshot() {
    let workflow = workflow_with_edges(&[(0, 1), (1, 2)], 3);
    assert_eq!(topology_snapshot(&workflow), include_str!("../../../tests/fixtures/tui-rows/linear.snap"));
    let layout = layout(&workflow, DEFAULT_MAX_LANES).expect("linear layout");
    assert_eq!(layout.lane_count, 1);
    assert_eq!(layout.rows.iter().map(|row| row.depth).collect::<Vec<_>>(), [0, 1, 2]);
}

// r[verify lattice.tui_ux.rows.topology]
#[test]
fn fork_join_workflow_snapshot() {
    let workflow = workflow_with_edges(&[(0, 1), (0, 2), (1, 3), (2, 4), (3, 5), (4, 5)], 6);
    assert_eq!(topology_snapshot(&workflow), include_str!("../../../tests/fixtures/tui-rows/fork-join.snap"));
}

// r[verify lattice.tui_ux.rows.topology]
#[test]
fn diamond_workflow_snapshot() {
    let workflow = workflow_with_edges(&[(0, 1), (0, 2), (1, 3), (2, 3)], 4);
    assert_eq!(topology_snapshot(&workflow), include_str!("../../../tests/fixtures/tui-rows/diamond.snap"));
}

#[test]
fn unequal_branches_keep_pending_lane_and_every_dependency_precedes_child() {
    let workflow = workflow_with_edges(&[(0, 1), (0, 2), (1, 3), (3, 4), (2, 4)], 5);
    let layout = layout(&workflow, DEFAULT_MAX_LANES).expect("branch layout");
    assert_eq!(layout.rows[1].lane, layout.rows[3].lane);
    assert_eq!(layout.rows[2].lane, layout.rows[4].lane);
    for edge in workflow.graph.edges() {
        let source = workflow.graph.port_owner(edge.source).expect("source");
        let target = workflow.graph.port_owner(edge.target).expect("target");
        let source_index = layout.rows.iter().position(|row| row.node_id == source).expect("row");
        let target_index = layout.rows.iter().position(|row| row.node_id == target).expect("row");
        assert!(source_index < target_index);
    }
}

#[test]
fn reverse_identity_order_and_disconnected_nodes_are_deterministic() {
    let workflow = workflow_with_edges(&[(2, 0), (0, 1)], 4);
    let layout = layout(&workflow, DEFAULT_MAX_LANES).expect("disconnected layout");
    assert_eq!(layout.rows.iter().map(|row| row.label.as_str()).collect::<Vec<_>>(), [
        "node-2", "node-3", "node-0", "node-1"
    ]);
    assert_eq!(layout, super::layout(&workflow, DEFAULT_MAX_LANES).expect("repeated layout"));
    assert_eq!(layout.rows.len(), 4);
}

// r[verify lattice.tui_ux.rows.wide]
#[test]
fn wide_fan_out_bounds_connectors_without_hiding_nodes() {
    let edges = (1..=10).map(|target| (0, target)).collect::<Vec<_>>();
    let workflow = workflow_with_edges(&edges, 11);
    let layout = layout(&workflow, 4).expect("wide layout");
    assert_eq!(layout.lane_count, 4);
    assert_eq!(layout.collapsed_lanes, 6);
    assert_eq!(layout.rows.len(), 11);
    assert!(layout.rows.iter().all(|row| row.lane < 4 && row.connectors.chars().count() <= 7));
    let rendered = buffer_lines(&render(&workflow, &layout, 100, 14, 0)).join("\n");
    assert!(rendered.contains("+6"));
    assert!(rendered.contains("◆"));
    assert!(rendered.contains("node-10"));
    assert_eq!(super::layout(&workflow, 0).expect("zero budget normalizes").lane_count, 1);
    assert_eq!(max_lanes_for_width(0), 1);
    assert_eq!(max_lanes_for_width(80), DEFAULT_MAX_LANES);
}

#[test]
fn focused_row_scrolls_into_view_and_resizing_clamps_offset() {
    let workflow = workflow_with_edges(&[], 20);
    let layout = layout(&workflow, DEFAULT_MAX_LANES).expect("viewport layout");
    let last = layout.rows.last().expect("last").node_id;
    let scroll = scroll_for_focus(&layout, Some(last), 0, 6);
    assert_eq!(scroll, 16);
    let rendered = buffer_lines(&render(&workflow, &layout, 90, 6, scroll)).join("\n");
    assert!(rendered.contains("node-19"));
    assert!(!rendered.contains("node-0 "));
    assert_eq!(scroll_for_focus(&layout, None, scroll, 40), 0);
    assert_eq!(scroll_for_focus(&layout, Some(layout.rows[0].node_id), scroll, 6), 0);
}

// r[verify lattice.tui_ux.rows.metadata]
#[test]
fn rows_render_observed_metadata_and_prefer_stdout_over_exit() {
    let workflow = workflow_with_edges(&[(0, 1)], 2);
    let layout = layout(&workflow, DEFAULT_MAX_LANES).expect("metadata layout");
    let node = workflow.graph.node(layout.rows[0].node_id).expect("node");
    let outputs = vec![
        RunOutputLine {
            run_id: RunId(1),
            node_id: node.id,
            port_id: node.output_ports[0].id,
            port_label: "stdout".to_string(),
            text: "bounded output".to_string(),
        },
        RunOutputLine {
            run_id: RunId(1),
            node_id: node.id,
            port_id: node.output_ports[2].id,
            port_label: "exit".to_string(),
            text: "exit code: 0".to_string(),
        },
    ];
    let mut buffer = Buffer::empty(Rect::new(0, 0, 180, 5));
    super::render(
        &layout,
        &workflow,
        RenderData {
            statuses: &HashMap::from([
                (node.id.0, NodeRunStatus::Completed),
                (layout.rows[1].node_id.0, NodeRunStatus::Failed),
            ]),
            started: &HashMap::new(),
            start_times: &HashMap::from([(node.id.0, "12:34:56".to_string())]),
            durations_millis: &HashMap::from([(node.id.0, 42)]),
            tokens: &HashMap::from([(node.id.0, 1234)]),
            costs_microunits: &HashMap::from([(node.id.0, 567)]),
            attempts: &HashMap::from([(node.id.0, 2)]),
            outputs: &outputs,
            selected: &BTreeSet::from([node.id]),
            focused: Some(node.id),
            scroll: 0,
        },
        buffer.area,
        &mut buffer,
    );
    let rendered = buffer_lines(&buffer).join("\n");
    for value in [
        "NODE",
        "completed",
        "failed",
        "12:34:56",
        "42ms",
        "1234",
        "567µ",
        "ATT",
        "TOTAL",
        "stdout: bounded output",
        "▶",
    ] {
        assert!(rendered.contains(value), "missing {value}: {rendered}");
    }
    assert!(!rendered.contains("exit code: 0"));
}

#[test]
fn live_duration_grows_final_duration_is_fixed_and_text_stays_one_line() {
    let started = std::time::Instant::now() - std::time::Duration::from_secs(2);
    assert!(duration_text(None, Some(&started)).ends_with('s'));
    assert_eq!(duration_text(Some(&42), Some(&started)), "42ms");
    assert_eq!(status_text(Some(NodeRunStatus::Skipped)), "skipped");
    assert_eq!(bounded_chars("a\nb\tc\u{1b}d", 20), "a b c d");
    assert_eq!(bounded_chars("abcdef", 4), "abc…");
    let workflow = workflow_with_edges(&[], 1);
    let layout = layout(&workflow, 1).expect("narrow layout");
    let rendered = buffer_lines(&render(&workflow, &layout, 32, 3, 0)).join("\n");
    assert!(rendered.contains("node-0"));
}

#[test]
fn oversized_lane_budget_preserves_topology_without_oversized_geometry() {
    let workflow = workflow_with_edges(&[(0, 1), (0, 2), (1, 3), (2, 3)], 4);
    assert_eq!(
        layout(&workflow, usize::MAX).expect("oversized budget"),
        layout(&workflow, 4).expect("node-bounded budget"),
    );
    assert_eq!(layout(&workflow_with_edges(&[], 0), usize::MAX).expect("empty layout").lane_count, 1);
}

#[test]
fn aggregate_metadata_retains_totals_beyond_individual_record_ranges() {
    let workflow = workflow_with_edges(&[], 2);
    let layout = layout(&workflow, 2).expect("aggregate layout");
    let totals = workflow.graph.nodes().map(|node| (node.id.0, u64::MAX)).collect::<HashMap<_, _>>();
    let attempts = workflow.graph.nodes().map(|node| (node.id.0, u32::MAX)).collect::<HashMap<_, _>>();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 240, 4));
    super::render(
        &layout,
        &workflow,
        RenderData {
            statuses: &HashMap::new(),
            started: &HashMap::new(),
            start_times: &HashMap::new(),
            durations_millis: &totals,
            tokens: &totals,
            costs_microunits: &totals,
            attempts: &attempts,
            outputs: &[],
            selected: &BTreeSet::new(),
            focused: None,
            scroll: 0,
        },
        buffer.area,
        &mut buffer,
    );
    let text = buffer_lines(&buffer).join("\n");
    assert!(text.contains("tokens 36893488147419103230"));
    assert!(text.contains("cost 36893488147419103230µ"));
    assert!(text.contains("attempts 8589934590"));
    assert_eq!(duration_millis_text(Some(18_446_744_073_709_551_616)), "18446744073709552.0s");
}
