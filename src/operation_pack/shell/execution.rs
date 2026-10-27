#![allow(
    unknown_lints,
    assertion_density,
    reason = "runtime and storage failures return typed outcomes instead of terminating the process"
)]
#![allow(
    function_length,
    reason = "the ordered effect sequence remains visible so durable-start and publication boundaries can be audited"
)]
#![allow(
    path_segment_repetition,
    reason = "operation and pack names remain explicit at effect and data publication boundaries"
)]
#![allow(
    compound_condition,
    reason = "the invocation binding predicate is one exact cross-field authority check"
)]
#![allow(
    ambiguous_params,
    too_many_parameters,
    reason = "private analyzer helpers receive one explicit ordered execution context assembled at a single call site"
)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Completed,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub pack_identity: String,
    pub lock_identity: String,
    pub invocation_identity: String,
    pub run_id: u64,
    pub operation_state: crate::operation::RunState,
    pub status: Status,
    pub operation_data_version: Option<String>,
    pub analyzer_outcomes: Vec<super::super::analyzer::Outcome>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<super::component::Receipt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effect_run_ids: Vec<u64>,
    pub output_content_omitted: bool,
    pub credentials_omitted: bool,
    pub private_paths_omitted: bool,
}

pub struct Request<'a> {
    pub app_config: &'a crate::config::AppConfig,
    pub pack_root: &'a std::path::Path,
    pub admitted: &'a super::super::model::AdmittedPack,
    pub instances_path: &'a std::path::Path,
    pub instance_id: &'a str,
    pub method: &'a str,
    pub arguments_json: &'a str,
    pub sequence: u64,
    pub prohibited_fragments: &'a [String],
    pub cancellation: Option<&'a std::sync::atomic::AtomicBool>,
}

// r[impl lattice.operation_packs.runtime.executable]
// r[impl lattice.operation_packs.registry.offline]
pub fn run(request: Request<'_>) -> Result<Report, super::super::Error> {
    super::super::admission::validate_lock(&request.admitted.candidate, &request.admitted.lock)?;
    let operation = select_operation(&request)?;
    let runtime = select_runtime(&request, operation)?;
    if runtime.kind == super::super::wasm::RUNTIME_KIND {
        return super::component::run(&request, operation, runtime);
    }
    let catalog_path =
        super::content_path(request.pack_root, &request.admitted.candidate, &operation.catalog_content_id)?;
    let executable_path = super::content_path(request.pack_root, &request.admitted.candidate, &runtime.content_id)?;
    validate_external_instances(request.instances_path)?;
    let process = request.app_config.effective_agent_runtime().local_process;
    let admitted_plan = crate::operation::cli::admit_for_pack_with_runtime(
        crate::operation::cli::PlanInput {
            catalog_path: &catalog_path,
            instances_path: request.instances_path,
            instance_id: request.instance_id,
            method: request.method,
            arguments_json: request.arguments_json,
            executable_path: &executable_path,
        },
        &process,
    )?;
    validate_plan_binding(&admitted_plan.plan, operation, runtime, request.method)?;
    let data_root = absolute_data_root(&request.app_config.data_dir)?;
    std::fs::create_dir_all(&data_root).map_err(|_| super::super::Error::Io("operation pack data directory create"))?;
    let store_path = crate::config::store_path(&data_root);
    let store =
        crate::store::Store::new(&store_path).map_err(|error| super::super::Error::Diagnostic(error.to_string()))?;
    let credentials = crate::operation::shell::environment_provider_for_plan(&admitted_plan.plan);
    let authority = crate::operation::shell::LocalReadOnlyAuthorityProvider::new(
        &admitted_plan.catalog,
        &admitted_plan.instance,
        &admitted_plan.plan,
    )?;
    let direct = crate::operation::shell::run_direct(crate::operation::shell::DirectRunRequest {
        store: &store,
        plan: &admitted_plan.plan,
        authority_provider: &authority,
        process: &process,
        credentials: &credentials,
        clock: &crate::operation::shell::SystemClock,
        cancellation: request.cancellation,
    })?;
    drop(store);
    let run_id = direct.record.run_id.0;
    let operation_state = direct.record.state;
    let mut operation_data_version = None;
    let mut references = Vec::new();
    if let Some(output) = direct.output {
        let plane = crate::operation_data::Plane::open(&store_path, &data_root)?;
        let draft = operation_record_draft(
            &request.admitted.candidate.pack_identity,
            &admitted_plan.plan,
            run_id,
            request.sequence,
            output,
        );
        let (metadata, _) = plane.publish(&draft, &crate::operation_data::AdmissionPolicy {
            allow_sensitive: false,
            prohibited_fragments: request.prohibited_fragments,
        })?;
        operation_data_version = Some(metadata.version_identity.clone());
        references.push(super::super::analyzer::DataReference {
            version_identity: metadata.version_identity,
            content_identity: metadata.payload.content_identity,
        });
        drop(plane);
    }
    let analyzer_outcomes =
        run_analyzers(&request, &process, run_id, operation_state, &references, &store_path, &data_root)?;
    let is_operation_completed =
        operation_state == crate::operation::RunState::Completed && operation_data_version.is_some();
    let is_parent_completed = super::super::analyzer::parent_succeeded(is_operation_completed, &analyzer_outcomes);
    let status = if is_parent_completed {
        Status::Completed
    } else if operation_state == crate::operation::RunState::Unknown {
        Status::Unknown
    } else {
        Status::Failed
    };
    Ok(Report {
        pack_identity: request.admitted.candidate.pack_identity.clone(),
        lock_identity: request.admitted.lock.identity.clone(),
        invocation_identity: admitted_plan.plan.identity,
        run_id,
        operation_state,
        status,
        operation_data_version,
        analyzer_outcomes,
        component: None,
        effect_run_ids: Vec::new(),
        output_content_omitted: true,
        credentials_omitted: true,
        private_paths_omitted: true,
    })
}

fn select_operation<'a>(
    request: &'a Request<'_>,
) -> Result<&'a super::super::model::OperationBinding, super::super::Error> {
    let matches = request
        .admitted
        .candidate
        .manifest
        .operations
        .iter()
        .filter(|operation| operation.methods.iter().any(|method| method == request.method))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [operation] => Ok(operation),
        [] => Err(super::super::Error::Invalid("pack operation method")),
        [_, ..] => Err(super::super::Error::Invalid("pack operation method ambiguity")),
    }
}

fn select_runtime<'a>(
    request: &'a Request<'_>,
    operation: &super::super::model::OperationBinding,
) -> Result<&'a super::super::model::RuntimeArtifact, super::super::Error> {
    let runtime = request
        .admitted
        .candidate
        .manifest
        .runtimes
        .iter()
        .find(|runtime| runtime.id == operation.runtime_id)
        .ok_or(super::super::Error::Invalid("pack operation runtime"))?;
    if !runtime.platforms.contains(&request.admitted.lock.platform)
        || !matches!(runtime.kind.as_str(), "bounded_process" | "wasm-component")
    {
        return Err(super::super::Error::Invalid("pack runtime platform or class"));
    }
    Ok(runtime)
}

fn validate_plan_binding(
    plan: &crate::operation::InvocationPlan,
    operation: &super::super::model::OperationBinding,
    runtime: &super::super::model::RuntimeArtifact,
    method: &str,
) -> Result<(), super::super::Error> {
    if plan.type_id != operation.type_id
        || plan.type_revision != operation.type_revision
        || plan.method != method
        || plan.runtime.key != operation.launch_label
        || plan.capabilities.iter().any(|capability| !runtime.capabilities.contains(capability))
        || plan.bounds.timeout_seconds > runtime.timeout_seconds
        || plan.bounds.input_bytes > runtime.input_bytes
        || plan.bounds.output_bytes > runtime.output_bytes
    {
        return Err(super::super::Error::Invalid("pack invocation binding"));
    }
    Ok(())
}

pub(super) fn absolute_data_root(path: &std::path::Path) -> Result<std::path::PathBuf, super::super::Error> {
    if !path.is_absolute() {
        return Err(super::super::Error::Invalid("operation pack data root"));
    }
    Ok(path.to_path_buf())
}

pub(super) fn validate_external_instances(path: &std::path::Path) -> Result<(), super::super::Error> {
    if !path.is_absolute() {
        return Err(super::super::Error::Invalid("operation instances path"));
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|_| super::super::Error::Io("instances observation"))?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > super::super::model::MAXIMUM_MANIFEST_BYTES
    {
        return Err(super::super::Error::Invalid("operation instances file"));
    }
    Ok(())
}

pub(super) fn operation_record_draft(
    pack_identity: &str,
    plan: &crate::operation::InvocationPlan,
    run_id: u64,
    sequence: u64,
    output: serde_json::Value,
) -> crate::operation_data::RecordDraft {
    let selectable_paths = match &output {
        serde_json::Value::Object(object) => object.keys().map(|key| vec![key.clone()]).collect(),
        _ => Vec::new(),
    };
    let mut tags = std::collections::BTreeMap::new();
    tags.insert("pack_identity".to_string(), pack_identity.to_string());
    crate::operation_data::RecordDraft {
        logical_key: format!("pack.{}.{}.{}", plan.type_id, plan.instance_id, plan.method),
        sequence,
        producer: crate::operation_data::Producer {
            operation_type: plan.type_id.clone(),
            type_revision: plan.type_revision.clone(),
            instance: plan.instance_id.clone(),
            method: plan.method.clone(),
            invocation: plan.identity.clone(),
            run_id,
            node: "operation-pack".to_string(),
            workflow_revision: plan.identity.clone(),
        },
        output_name: "result".to_string(),
        output_contract_identity: plan.output_contract_identity.clone(),
        tags,
        sensitivity: crate::operation_data::SensitivityClass::Internal,
        lifetime: crate::operation_data::Lifetime::Retained,
        retention: crate::operation_data::RetentionPolicy {
            identity: super::super::identity::content_identity(format!("retained:{pack_identity}").as_bytes()),
            maximum_versions: None,
        },
        outcome: "completed".to_string(),
        payload: crate::operation_data::PayloadDraft::Resource {
            schema_identity: plan.output_contract_identity.clone(),
            selectable_paths,
            value: output,
        },
        required_non_claims: crate::operation_data::REQUIRED_NON_CLAIMS.map(str::to_string).to_vec(),
    }
}

include!("execution/analyzers.rs");
