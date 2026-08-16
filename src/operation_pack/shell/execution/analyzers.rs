fn run_analyzers(
    request: &Request<'_>,
    process: &crate::process::core::RuntimeConfig,
    run_id: u64,
    operation_state: crate::operation::RunState,
    references: &[super::super::analyzer::DataReference],
    store_path: &std::path::Path,
    data_root: &std::path::Path,
) -> Result<Vec<super::super::analyzer::Outcome>, super::super::Error> {
    let selected = super::super::analyzer::select_for_direct(&request.admitted.candidate.manifest.analyzers);
    let mut outcomes = Vec::with_capacity(selected.len());
    for analyzer in selected {
        outcomes.push(run_analyzer(
            request,
            process,
            run_id,
            operation_state,
            references,
            store_path,
            data_root,
            analyzer,
        )?);
    }
    Ok(outcomes)
}

fn run_analyzer(
    request: &Request<'_>,
    process: &crate::process::core::RuntimeConfig,
    run_id: u64,
    operation_state: crate::operation::RunState,
    references: &[super::super::analyzer::DataReference],
    store_path: &std::path::Path,
    data_root: &std::path::Path,
    analyzer: &super::super::model::AnalyzerManifest,
) -> Result<super::super::analyzer::Outcome, super::super::Error> {
    let runtime = request
        .admitted
        .candidate
        .manifest
        .runtimes
        .iter()
        .find(|runtime| runtime.id == analyzer.runtime_id)
        .ok_or(super::super::Error::Invalid("analyzer runtime"))?;
    let executable = super::content_path(request.pack_root, &request.admitted.candidate, &runtime.content_id)?;
    let input = super::super::analyzer::plan_input(
        &request.admitted.candidate.pack_identity,
        analyzer,
        run_id,
        &format!("{operation_state:?}").to_ascii_lowercase(),
        references,
    )?;
    let stdin = serde_json::to_vec(&input).map_err(|_| super::super::Error::Encoding)?;
    if stdin.len() > analyzer.input_bytes {
        return super::super::analyzer::failed_outcome(
            analyzer,
            super::super::analyzer::Status::Invalid,
            "analyzer input exceeded its bound",
        );
    }
    let process_request = crate::agent_node::ProcessRequest {
        program: executable.to_string_lossy().into_owned(),
        args: vec![analyzer.launch_label.clone(), analyzer.id.clone()],
        cwd: Some(request.pack_root.to_string_lossy().into_owned()),
        env: std::collections::BTreeMap::new(),
        stdin,
        timeout_secs: analyzer.timeout_seconds,
        output_limit_bytes: analyzer.output_bytes,
    };
    let output = match crate::process::shell::run_bounded_clear_environment_with_cancellation(
        &process_request,
        process,
        request.cancellation,
    ) {
        Ok(output) if output.status_code == Some(0) => output,
        Ok(_) => {
            return super::super::analyzer::failed_outcome(
                analyzer,
                super::super::analyzer::Status::Failed,
                "analyzer returned failure",
            );
        }
        Err(crate::agent_node::ProcessError::Timeout) => {
            return super::super::analyzer::failed_outcome(
                analyzer,
                super::super::analyzer::Status::TimedOut,
                "analyzer timed out",
            );
        }
        Err(crate::agent_node::ProcessError::OutputLimitExceeded { .. }) => {
            return super::super::analyzer::failed_outcome(
                analyzer,
                super::super::analyzer::Status::Truncated,
                "analyzer output was truncated",
            );
        }
        Err(_) => {
            return super::super::analyzer::failed_outcome(
                analyzer,
                super::super::analyzer::Status::Failed,
                "analyzer outcome unavailable",
            );
        }
    };
    let classified =
        match super::super::analyzer::classify_output(analyzer, &output.stdout, request.prohibited_fragments) {
            Ok(output) => output,
            Err(_) => {
                return super::super::analyzer::failed_outcome(
                    analyzer,
                    super::super::analyzer::Status::Invalid,
                    "analyzer output was invalid",
                );
            }
        };
    let plane = crate::operation_data::Plane::open(store_path, data_root)?;
    let output_value = serde_json::to_value(&classified).map_err(|_| super::super::Error::Encoding)?;
    let mut tags = std::collections::BTreeMap::new();
    tags.insert("pack_identity".to_string(), request.admitted.candidate.pack_identity.clone());
    tags.insert("analyzer".to_string(), analyzer.id.clone());
    let draft = crate::operation_data::RecordDraft {
        logical_key: format!("pack.{}.analyzer.{}", request.admitted.candidate.manifest.pack_id, analyzer.id),
        sequence: request.sequence,
        producer: crate::operation_data::Producer {
            operation_type: request.admitted.candidate.manifest.pack_id.clone(),
            type_revision: request.admitted.candidate.pack_identity.clone(),
            instance: request.instance_id.to_string(),
            method: format!("analyzer-{}", analyzer.id),
            invocation: input.analyzer_identity.clone(),
            run_id,
            node: "operation-pack-analyzer".to_string(),
            workflow_revision: request.admitted.candidate.pack_identity.clone(),
        },
        output_name: "analysis".to_string(),
        output_contract_identity: analyzer.output_schema_identity.clone(),
        tags,
        sensitivity: analyzer.sensitivity,
        lifetime: crate::operation_data::Lifetime::Retained,
        retention: crate::operation_data::RetentionPolicy {
            identity: analyzer.retention_identity.clone(),
            maximum_versions: None,
        },
        outcome: "completed".to_string(),
        payload: crate::operation_data::PayloadDraft::Resource {
            schema_identity: analyzer.output_schema_identity.clone(),
            selectable_paths: vec![vec!["structured".to_string()], vec!["markdown".to_string()]],
            value: output_value,
        },
        required_non_claims: crate::operation_data::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    };
    let (metadata, _) = plane.publish(&draft, &crate::operation_data::AdmissionPolicy {
        allow_sensitive: analyzer.sensitivity == crate::operation_data::SensitivityClass::Sensitive,
        prohibited_fragments: request.prohibited_fragments,
    })?;
    Ok(super::super::analyzer::Outcome {
        analyzer_id: analyzer.id.clone(),
        analyzer_identity: input.analyzer_identity,
        required: analyzer.required,
        status: super::super::analyzer::Status::Completed,
        published_version_identity: Some(metadata.version_identity),
        diagnostic: "analyzer output passed its declared contract".to_string(),
        content_omitted: true,
    })
}
