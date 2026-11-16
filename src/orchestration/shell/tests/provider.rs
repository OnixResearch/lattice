use super::super::usage_cost;
use super::super::usage_tokens;
use super::known_status;
use super::pending_usage_fixture_for_workflow;

// r[verify lattice.agent_orchestration.usage.scenario.uncertainty]
#[test]
fn production_multiline_metadata_reaches_charge_and_visible_overrun() {
    use crate::engine::executor::NodeStatus;
    use crate::engine::executor::StepResult;
    use crate::engine::value::ContentHash;
    use crate::engine::value::Value;
    use crate::orchestration::*;
    let mut workflow = crate::engine::Workflow::new("usage-agent", crate::engine::workflow::WorkflowTimestamp::epoch());
    let node_id =
        workflow.add_agent_node("agent", crate::agent_node::Target::Pi, "work", crate::agent_node::OutputMode::Text);
    let port_id = workflow
        .graph
        .node(node_id)
        .expect("agent node")
        .output_ports
        .iter()
        .find(|port| port.label == "metadata")
        .expect("metadata port")
        .id;
    let binding = crate::agent_node::ConfiguredBinding {
        schema: crate::agent_node::CONFIGURED_RUNTIME_SCHEMA.to_string(),
        target: crate::agent_node::Target::Pi,
        revision: format!("blake3:{}", "a".repeat(64)),
    };
    let metadata = Value::from_json(serde_json::json!({
        "provider": "openai_compatible",
        "model": "reported-model",
        "configured_runtime": binding,
        "usage": {"total_tokens": 250, "prompt_tokens": 150}
    }));
    let displayed = metadata.as_display_string();
    let steps = [StepResult {
        node_id,
        status: NodeStatus::Completed {
            outputs: std::collections::HashMap::from([(port_id, metadata)]),
            cached: false,
        },
        input_hash: ContentHash::from_bytes(b"accounting-metadata"),
    }];
    let (_directory, store, started) = pending_usage_fixture_for_workflow(Some(52), workflow.clone());
    let dispatch = started.pending_dispatch.as_ref().expect("pending dispatch");
    // Use the exact projection called by headless completion, not hand-built rows.
    let details = crate::run::history::detail_record_from_results_with_completion_order(
        dispatch.run_id,
        &workflow,
        &steps,
        vec![node_id],
    );
    let rows = &details.nodes[0].outputs;
    assert!(rows.len() > 1, "Agent JSON is pretty-printed into ordered history rows");
    assert!(rows.iter().all(|row| row.port_id == port_id && row.port_label == "metadata"));
    assert_eq!(rows.iter().map(|row| row.text.as_str()).collect::<Vec<_>>().join("\n"), displayed);
    store.save_run_details(&details).expect("production history persists");
    let reports = super::super::provider_reports(&store, &started, dispatch.run_id).expect("reports");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].tokens, Some(250));
    assert_eq!(reports[0].configured_runtime, Some(binding));
    let action = super::super::finish_action(&store, &started, super::super::FinishRequest {
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
        status: known_status(true),
        is_retryable_observed_failure: false,
        safe_summary: "done".to_string(),
    })
    .expect("multiline provider usage is retained");
    let finished =
        super::super::durability::persistence::apply_and_commit(&store, &started, action).expect("overrun settles");
    assert_eq!(finished.budgets.consumed.tokens, 250);
    assert_eq!(finished.status, Status::Exhausted);
    assert!(finished.budgets.reservations.is_empty());
    let receipt = store.load_command_worker_usage_for_run(dispatch.run_id).expect("receipt loads").expect("receipt");
    assert_eq!(receipt.accounting.expect("accounting").provider_reports, reports);
    assert!(super::super::replay(&store, started.id.as_str()).expect("overrun replays").valid);

    let mut malformed = details.nodes[0].clone();
    malformed.outputs.pop();
    assert!(super::super::provider_metadata(&malformed, port_id).is_none(), "truncation is not a report");
    let mut duplicate = details.nodes[0].clone();
    duplicate.outputs.extend(details.nodes[0].outputs.clone());
    assert!(
        super::super::provider_metadata(&duplicate, port_id).is_none(),
        "two JSON documents are not one report"
    );
    let mut crossed = details.nodes[0].clone();
    crossed.outputs[0].port_id = crate::engine::workflow::PortId(u64::MAX);
    assert!(super::super::provider_metadata(&crossed, port_id).is_none(), "labels alone do not bind an output");
    let mut oversized = details.nodes[0].clone();
    oversized.outputs[0].text = " ".repeat(crate::orchestration::usage::MAX_USAGE_OBSERVATION_BYTES);
    assert!(
        super::super::provider_metadata(&oversized, port_id).is_none(),
        "bounded reassembly rejects rather than truncates"
    );
    let mut cached = details.clone();
    cached.nodes[0].cached = true;
    store.save_run_details(&cached).expect("cached history persists");
    assert!(
        super::super::provider_reports(&store, &started, dispatch.run_id)
            .expect("cached reports")
            .is_empty()
    );
    let mut ambiguous = details.clone();
    ambiguous.nodes.push(details.nodes[0].clone());
    store.save_run_details(&ambiguous).expect("duplicate node history persists");
    assert!(
        super::super::provider_reports(&store, &started, dispatch.run_id)
            .expect("ambiguous reports")
            .is_empty()
    );
    let mut crossed_target = details;
    for output in &mut crossed_target.nodes[0].outputs {
        output.text = output.text.replace("\"target\": \"Pi\"", "\"target\": \"Animus\"");
    }
    store
        .save_run_details(&crossed_target)
        .expect("crossed runtime target persists as untrusted history");
    assert!(
        super::super::provider_reports(&store, &started, dispatch.run_id)
            .expect("crossed target rejected")
            .is_empty()
    );
}

#[test]
fn trustworthy_partial_totals_survive_but_invalid_or_conflicting_tokens_do_not() {
    for usage in [
        serde_json::json!({"total_tokens": 200, "prompt_tokens": 150}),
        serde_json::json!({"total_tokens": 200, "completion_tokens": 50}),
        serde_json::json!({"total_tokens": 200, "input_tokens": 200, "prompt_tokens": 200}),
        serde_json::json!({"total_tokens": 0, "output_tokens": 0}),
    ] {
        let total = usage["total_tokens"].as_u64();
        assert_eq!(usage_tokens(&serde_json::json!({"usage": usage})), total);
    }
    for usage in [
        serde_json::json!({"total_tokens": 200, "prompt_tokens": 201}),
        serde_json::json!({"total_tokens": 200, "completion_tokens": 201}),
        serde_json::json!({"total_tokens": 200, "prompt_tokens": 150, "completion_tokens": 49}),
        serde_json::json!({"total_tokens": 200, "prompt_tokens": 150, "input_tokens": 149}),
        serde_json::json!({"total_tokens": 200, "prompt_tokens": -1}),
        serde_json::json!({"total_tokens": -1, "prompt_tokens": 1}),
        serde_json::json!({"total_tokens": 200, "completion_tokens": "50"}),
        serde_json::json!({"total_tokens": 200, "completion_tokens": null}),
        serde_json::json!({"total_tokens": 200.5, "prompt_tokens": 150}),
        serde_json::json!({"total_tokens": u64::MAX, "prompt_tokens": u64::MAX, "completion_tokens": 1}),
        serde_json::json!({"prompt_tokens": 150}),
    ] {
        assert_eq!(usage_tokens(&serde_json::json!({"usage": usage})), None);
    }
    let overflow = serde_json::from_str::<serde_json::Value>(
        "{\"usage\":{\"total_tokens\":18446744073709551616,\"prompt_tokens\":150}}",
    )
    .expect("JSON number");
    assert_eq!(usage_tokens(&overflow), None);
}

#[test]
fn usage_cost_supports_total_and_short_shapes() {
    assert_eq!(usage_cost(&serde_json::json!({"usage": {"total_cost_microunits": 42}})), Some(42));
    assert_eq!(usage_cost(&serde_json::json!({"usage": {"cost_microunits": 42}})), Some(42));
    assert_eq!(usage_cost(&serde_json::json!({"usage": {}})), None);
    assert_eq!(usage_cost(&serde_json::json!({"usage": {"total_cost_microunits": "42"}})), None);
    assert_eq!(
        usage_cost(&serde_json::json!({"usage": {"total_cost_microunits": 42, "cost_microunits": 43}})),
        None
    );
}

#[test]
fn usage_tokens_supports_total_and_component_shapes() {
    assert_eq!(usage_tokens(&serde_json::json!({"usage": {"total_tokens": 42}})), Some(42));
    assert_eq!(
        usage_tokens(&serde_json::json!({"usage": {"prompt_tokens": 30, "completion_tokens": 12}})),
        Some(42)
    );
    assert_eq!(usage_tokens(&serde_json::json!({"usage": {"input_tokens": 30, "output_tokens": 12}})), Some(42));
    assert_eq!(usage_tokens(&serde_json::json!({"usage": {}})), None);
    assert_eq!(usage_tokens(&serde_json::json!({"usage": {"total_tokens": "42"}})), None);
    assert_eq!(
        usage_tokens(&serde_json::json!({"usage": {"total_tokens": 42, "input_tokens": 30, "output_tokens": 13}})),
        None
    );
    assert_eq!(
        usage_tokens(&serde_json::json!({"usage": {"input_tokens": 30, "prompt_tokens": 31, "output_tokens": 12}})),
        None
    );
    assert_eq!(usage_tokens(&serde_json::json!({"usage": {"input_tokens": u64::MAX, "output_tokens": 1}})), None);
}
