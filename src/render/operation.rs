//! The opt-in closed operation profile; the normal graph/effect executor remains
//! the sole scheduler and authority path. There is no automatic local grant.
use anyhow::Context;

mod profile;

pub const TYPE_ID: &str = "molten.render";
pub const RUNTIME_KIND: &str = "process";
pub const RUNTIME_KEY: &str = "lattice-render-v1";
pub const METHODS: &[&str] = &["advance", "cancel", "observe", "supersede"];
pub const FILE_TARGET: &str = "render_binding_file";
pub const ID_TARGET: &str = "render_binding_identity";

const PREFLIGHT_CHECKS: [crate::operation::model::PreflightCheck; 4] = [
    crate::operation::model::PreflightCheck::RuntimeAvailable,
    crate::operation::model::PreflightCheck::ExecutableAvailable,
    crate::operation::model::PreflightCheck::RepositoryRootAdmitted,
    crate::operation::model::PreflightCheck::CredentialReferencesValid,
];

pub fn is_runtime(runtime: &crate::operation::model::RuntimeRequirement) -> bool {
    runtime.kind == RUNTIME_KIND && runtime.key == RUNTIME_KEY
}

pub fn uses_runtime(plan: &crate::operation::InvocationPlan) -> bool {
    plan.runtime.kind == RUNTIME_KIND && plan.runtime.key == RUNTIME_KEY
}

pub fn requirements() -> crate::operation::ExecutionRequirements {
    crate::operation::backend::ExecutionRequirements {
        format: crate::operation::backend::ExecutionFormat::NativeBinary,
        filesystem: crate::operation::backend::FilesystemSemantics::WritableWorkspace,
        network: crate::operation::backend::NetworkPosture::NetworkAllowlisted,
        process: crate::operation::backend::ProcessModel::ProcessTree,
        terminal: crate::operation::backend::TerminalNeed::NoTerminal,
        peripheral: crate::operation::backend::PeripheralNeed::NoPeripheral,
        persistence: crate::operation::backend::PersistenceNeed::DurableWorkspace,
        isolation: crate::operation::backend::IsolationClaim::GuardedProcessIsolation,
        resources: crate::operation::backend::ResourceRequirements {
            memory_bytes: 536_870_912,
            storage_bytes: 1_073_741_824,
            cpu_milliseconds: 600_000,
            wall_milliseconds: 600_000,
        },
    }
}

pub fn type_is_supported(definition: &crate::operation::OperationType) -> bool {
    definition.id == TYPE_ID
        && is_runtime(&definition.runtime)
        && definition.execution_requirements == requirements()
        && definition.credential_bindings.is_empty()
        && definition.capabilities == [crate::operation::model::CapabilityClass::ProcessRead]
        && definition.methods.iter().map(|m| m.name.as_str()).eq(METHODS.iter().copied())
        && definition.methods.iter().all(method_is_supported)
}

fn output_contract() -> crate::operation::model::RecordContract {
    crate::operation::model::RecordContract {
        fields: vec![crate::operation::model::FieldContract {
            name: "outcome".into(),
            value_type: crate::operation::model::ValueType::String,
            required: true,
            max_bytes: 64,
        }],
        deny_unknown_fields: false,
    }
}

pub fn catalog(executable: &std::path::Path) -> anyhow::Result<crate::operation::model::Catalog> {
    profile::catalog(executable)
}

pub fn instance(
    catalog: &crate::operation::model::Catalog,
    repository: &std::path::Path,
    binding_file: &std::path::Path,
    binding: molten_batch_core::Id,
) -> crate::operation::model::InstanceFile {
    profile::instance(catalog, repository, binding_file, binding)
}

pub fn target_is_supported(target: &std::collections::BTreeMap<String, serde_json::Value>) -> bool {
    target.len() == 3
        && [crate::operation::runtime::REPOSITORY_ROOT_TARGET, FILE_TARGET].iter().all(|key| {
            target
                .get(*key)
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| std::path::Path::new(value).is_absolute())
        })
        && target
            .get(ID_TARGET)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| molten_batch_core::Id::parse(value).is_ok())
}

pub fn planned_request(plan: &crate::operation::InvocationPlan) -> anyhow::Result<super::Request> {
    anyhow::ensure!(
        plan.type_id == TYPE_ID
            && uses_runtime(plan)
            && METHODS.contains(&plan.method.as_str())
            && plan.arguments.is_empty()
            && target_is_supported(&plan.target),
        "render operation binding is not exact"
    );
    let file = plan.target[FILE_TARGET].as_str().context("missing binding file")?;
    let identity = molten_batch_core::Id::parse(plan.target[ID_TARGET].as_str().context("missing binding identity")?)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    super::shell::load_request(std::path::Path::new(file), identity)
}

pub fn process_request(plan: &crate::operation::InvocationPlan) -> anyhow::Result<crate::agent_node::ProcessRequest> {
    planned_request(plan)?; // remeasure canonical binding immediately before dispatch
    process_settings(plan)
}

pub(crate) fn bind_process_context(
    plan: &crate::operation::InvocationPlan,
    execution: &super::shell::HostExecution,
    process: &mut crate::agent_node::ProcessRequest,
) -> anyhow::Result<()> {
    anyhow::ensure!(execution.invocation_identity == plan.identity, "crossed host invocation");
    let request = planned_request(plan)?;
    let (file, identity) = super::shell::bind_host_execution(&request, execution, &plan.method)?;
    process.args.extend([
        "--permit".into(),
        file.to_str().context("dispatch file")?.into(),
        "--permit-identity".into(),
        identity.to_string(),
    ]);
    Ok(())
}

/// A zero exit, JSON shape or producer Succeeded flag alone cannot complete the
/// graph node. Re-read authority and independently verify the committed bytes.
pub fn classify(
    plan: &crate::operation::InvocationPlan,
    value: &serde_json::Value,
    execution: Option<&super::shell::HostExecution>,
) -> anyhow::Result<crate::operation::runtime::ObservationClass> {
    let request = planned_request(plan)?;
    let receipt: super::shell::Receipt = serde_json::from_value(value.clone())?;
    let execution = execution.context("actual host execution context is required")?;
    anyhow::ensure!(
        receipt.observed_host_execution.as_ref() == Some(execution) && receipt.host_execution.is_some(),
        "crossed host-issued observation context"
    );
    anyhow::ensure!(
        receipt.binding == request.id()?
            && receipt.execution == request.execution
            && receipt.attempt == request.attempt
            && receipt.budget_identity == request.budget.identity,
        "crossed render receipt"
    );
    classify_receipt(&request, &receipt)
}

fn method_is_supported(m: &crate::operation::model::Method) -> bool {
    m.arguments.fields.is_empty()
        && m.arguments.deny_unknown_fields
        && m.preflight == PREFLIGHT_CHECKS
        && m.capabilities == [crate::operation::model::CapabilityClass::ProcessRead]
        && m.side_effect == crate::operation::model::SideEffectClass::Mutation
        && m.output == output_contract()
}

fn process_settings(plan: &crate::operation::InvocationPlan) -> anyhow::Result<crate::agent_node::ProcessRequest> {
    Ok(crate::agent_node::ProcessRequest {
        program: plan.runtime.executable_path.clone(),
        args: process_arguments(plan)?,
        cwd: Some(
            plan.target[crate::operation::runtime::REPOSITORY_ROOT_TARGET]
                .as_str()
                .context("repository")?
                .into(),
        ),
        env: std::collections::BTreeMap::new(),
        stdin: vec![],
        timeout_secs: plan.bounds.timeout_seconds,
        output_limit_bytes: plan.bounds.output_bytes,
    })
}

fn classify_receipt(
    request: &super::Request,
    receipt: &super::shell::Receipt,
) -> anyhow::Result<crate::operation::runtime::ObservationClass> {
    Ok(match receipt.outcome {
        super::Outcome::Completed => {
            super::shell::verify_committed(request, receipt)?;
            crate::operation::runtime::ObservationClass::Completed
        }
        super::Outcome::Failed | super::Outcome::Superseded => {
            crate::operation::runtime::ObservationClass::DeterministicFailure
        }
        _ => crate::operation::runtime::ObservationClass::Unknown,
    })
}

fn process_arguments(plan: &crate::operation::InvocationPlan) -> anyhow::Result<Vec<String>> {
    Ok(vec![
        "render".into(),
        "step".into(),
        "--binding".into(),
        plan.target[FILE_TARGET].as_str().context("binding")?.into(),
        "--identity".into(),
        plan.target[ID_TARGET].as_str().context("identity")?.into(),
        "--method".into(),
        plan.method.clone(),
    ])
}
