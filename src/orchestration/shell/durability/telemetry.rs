// r[impl lattice.agent_runtime.configured_bindings.scenario.usage]
pub(in crate::orchestration::shell) fn provider_reports(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    run_id: crate::engine::RunId,
) -> Result<Vec<crate::orchestration::usage::ProviderReport>, super::super::Error> {
    let Some(workflow) = store.load_workflow(&instance.workflow_revision.workflow_name)? else {
        return Ok(Vec::new());
    };
    // Never interpret outputs against a subsequently edited graph.
    if crate::orchestration::core::workflow_revision(&workflow)? != instance.workflow_revision {
        return Ok(Vec::new());
    }
    let Some(details) = store.load_run_details(run_id)? else {
        return Ok(Vec::new());
    };
    if details.run_id != run_id {
        return Err(super::super::Error::RunIdentityMismatch { run_id });
    }
    reports_from_details(&workflow, &details.nodes)
}

fn reports_from_details(
    workflow: &crate::engine::Workflow,
    details: &[crate::store::RunNodeDetail],
) -> Result<Vec<crate::orchestration::usage::ProviderReport>, super::super::Error> {
    // Admission bounds growth without allocating for absent provider evidence.
    let mut reports = Vec::new();
    for (node_id, config) in &workflow.configs {
        if !matches!(config, crate::engine::NodeConfig::Agent(_)) {
            continue;
        }
        let Some(report) = provider_report(workflow, details, *node_id) else {
            continue;
        };
        reports = retain_report(reports, report).map_err(|(_, error)| error)?;
    }
    Ok(reports)
}

fn retain_report(
    mut reports: Vec<crate::orchestration::usage::ProviderReport>,
    report: crate::orchestration::usage::ProviderReport,
) -> Result<
    Vec<crate::orchestration::usage::ProviderReport>,
    (Vec<crate::orchestration::usage::ProviderReport>, super::super::Error),
> {
    if reports.len() >= crate::orchestration::usage::MAX_PROVIDER_REPORTS {
        return Err((
            reports,
            super::super::Error::InvalidProposal("provider usage report limit exceeded".to_string()),
        ));
    }
    reports.push(report);
    Ok(reports)
}

fn provider_report(
    workflow: &crate::engine::Workflow,
    details: &[crate::store::RunNodeDetail],
    node_id: u64,
) -> Option<crate::orchestration::usage::ProviderReport> {
    let detail = completed_detail(details, node_id)?;
    let port_id = metadata_port(workflow, node_id)?;
    let value = provider_metadata(detail, port_id)?;
    if value.get("provider").and_then(serde_json::Value::as_str) != Some("openai_compatible") {
        return None;
    }
    let model = provider_model(&value);
    let configured_runtime = bound_runtime(workflow, node_id, &value)?;
    Some(crate::orchestration::usage::ProviderReport {
        node_id,
        provider: "openai_compatible".to_string(),
        model,
        configured_runtime,
        tokens: usage_tokens(&value),
        cost_microunits: usage_cost(&value),
    })
}

fn bound_runtime(
    workflow: &crate::engine::Workflow,
    node_id: u64,
    value: &serde_json::Value,
) -> Option<Option<crate::agent_node::ConfiguredBinding>> {
    let binding = configured_runtime(value)?;
    if binding.as_ref().is_some_and(|binding| {
        !matches!(workflow.configs.get(&node_id), Some(crate::engine::NodeConfig::Agent(config)) if config.target == binding.target)
    }) {
        return None;
    }
    Some(binding)
}

fn completed_detail(details: &[crate::store::RunNodeDetail], node_id: u64) -> Option<&crate::store::RunNodeDetail> {
    let mut nodes = details.iter().filter(|detail| detail.node_id.0 == node_id);
    let detail = nodes.next()?;
    // Cached or ambiguous details do not establish a new provider call.
    if nodes.next().is_some() || detail.cached || detail.status != crate::store::RunNodeDetailStatus::Completed {
        return None;
    }
    Some(detail)
}

fn metadata_port(workflow: &crate::engine::Workflow, node_id: u64) -> Option<crate::engine::workflow::PortId> {
    let node = workflow.graph.node(crate::engine::workflow::NodeId(node_id))?;
    let mut ports = node.output_ports.iter().filter(|port| port.label == "metadata");
    let port = ports.next()?;
    if ports.next().is_some() {
        return None;
    }
    Some(port.id)
}

fn provider_model(value: &serde_json::Value) -> Option<String> {
    value
        .get("model")
        .and_then(serde_json::Value::as_str)
        .filter(|model| {
            !model.is_empty()
                && model.len() <= crate::orchestration::usage::MAX_IDENTIFIER_BYTES
                && !model.chars().any(char::is_control)
        })
        .map(str::to_owned)
}

fn configured_runtime(value: &serde_json::Value) -> Option<Option<crate::agent_node::ConfiguredBinding>> {
    let Some(binding) = value.get("configured_runtime") else {
        return Some(None);
    };
    let binding: crate::agent_node::ConfiguredBinding = serde_json::from_value(binding.clone()).ok()?;
    binding.validate().then_some(Some(binding))
}

/// History stores pretty JSON as ordered, port-bound lines, not one output row.
pub(in crate::orchestration::shell) fn provider_metadata(
    detail: &crate::store::RunNodeDetail,
    port_id: crate::engine::workflow::PortId,
) -> Option<serde_json::Value> {
    let rows = || detail.outputs.iter().filter(|row| row.port_id == port_id || row.port_label == "metadata");
    let length_bytes = metadata_length_bytes(detail, port_id)?;
    let mut metadata = String::with_capacity(length_bytes);
    for (index, row) in rows().enumerate() {
        if index != 0 {
            metadata.push('\n');
        }
        metadata.push_str(&row.text);
    }
    serde_json::from_str(&metadata).ok()
}

fn metadata_length_bytes(
    detail: &crate::store::RunNodeDetail,
    port_id: crate::engine::workflow::PortId,
) -> Option<usize> {
    let mut length_bytes = 0usize;
    let mut row_count = 0usize;
    for row in detail.outputs.iter().filter(|row| row.port_id == port_id || row.port_label == "metadata") {
        if row.port_id != port_id || row.port_label != "metadata" {
            return None;
        }
        length_bytes = length_bytes.checked_add(usize::from(row_count != 0))?.checked_add(row.text.len())?;
        if length_bytes > crate::orchestration::usage::MAX_USAGE_OBSERVATION_BYTES {
            return None;
        }
        row_count += 1;
    }
    (row_count != 0).then_some(length_bytes)
}

pub(in crate::orchestration::shell) fn usage_tokens(value: &serde_json::Value) -> Option<u64> {
    let usage = value.get("usage")?;
    let total = match usage.get("total_tokens") {
        Some(value) => Some(value.as_u64()?),
        None => None,
    };
    let input = consistent_alias(usage, ["input_tokens", "prompt_tokens"])?;
    let output = consistent_alias(usage, ["output_tokens", "completion_tokens"])?;
    match (total, input, output) {
        (Some(total), input, output) if input.is_none() || output.is_none() => {
            input.or(output).is_none_or(|component| component <= total).then_some(total)
        }
        (total, Some(input), Some(output)) => {
            let sum = input.checked_add(output)?;
            total.is_none_or(|total| total == sum).then_some(sum)
        }
        _ => None,
    }
}

fn consistent_alias(value: &serde_json::Value, aliases: [&str; 2]) -> Option<Option<u64>> {
    let [first, second] = aliases;
    match (value.get(first), value.get(second)) {
        (None, None) => Some(None),
        (Some(value), None) | (None, Some(value)) => value.as_u64().map(Some),
        (Some(first), Some(second)) => {
            let first = first.as_u64()?;
            (Some(first) == second.as_u64()).then_some(Some(first))
        }
    }
}

pub(in crate::orchestration::shell) fn usage_cost(value: &serde_json::Value) -> Option<u64> {
    consistent_alias(value.get("usage")?, ["total_cost_microunits", "cost_microunits"])?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(node_id: u64) -> crate::orchestration::usage::ProviderReport {
        crate::orchestration::usage::ProviderReport {
            node_id,
            provider: "openai_compatible".to_string(),
            model: None,
            configured_runtime: None,
            tokens: None,
            cost_microunits: None,
        }
    }

    #[test]
    fn report_limit_admits_the_boundary_and_rejects_without_truncation() {
        let mut reports = Vec::with_capacity(crate::orchestration::usage::MAX_PROVIDER_REPORTS);
        for node_id in 0..crate::orchestration::usage::MAX_PROVIDER_REPORTS {
            reports = retain_report(reports, report(u64::try_from(node_id).expect("bounded node id")))
                .expect("report within the limit");
        }
        assert_eq!(reports.len(), crate::orchestration::usage::MAX_PROVIDER_REPORTS);
        let (reports, error) = retain_report(reports, report(u64::MAX)).expect_err("report limit rejects append");
        assert!(matches!(error, crate::orchestration::shell::Error::InvalidProposal(_)));
        assert_eq!(reports.len(), crate::orchestration::usage::MAX_PROVIDER_REPORTS);
        assert_ne!(reports.last().expect("retained report").node_id, u64::MAX);
    }

    #[test]
    fn runtime_bindings_preserve_absence_and_reject_malformed_provenance() {
        assert_eq!(configured_runtime(&serde_json::json!({})), Some(None));
        let binding = crate::agent_node::ConfiguredBinding {
            schema: crate::agent_node::CONFIGURED_RUNTIME_SCHEMA.to_string(),
            target: crate::agent_node::Target::Pi,
            revision: format!("blake3:{}", "a".repeat(64)),
        };
        assert_eq!(
            configured_runtime(&serde_json::json!({"configured_runtime": binding})),
            Some(Some(binding.clone()))
        );
        for value in [
            serde_json::json!(null),
            serde_json::json!({"schema": "unsupported", "target": "Pi", "revision": binding.revision}),
            serde_json::json!({"schema": binding.schema, "target": "Pi", "revision": "blake3:bad"}),
            serde_json::json!({"schema": binding.schema, "target": "Pi", "revision": binding.revision, "lease": "invented"}),
        ] {
            assert_eq!(configured_runtime(&serde_json::json!({"configured_runtime": value})), None);
        }
    }
}
