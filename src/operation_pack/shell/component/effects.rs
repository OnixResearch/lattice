//! Effect proposals re-enter existing operation authority and durable owners.

use crate::operation_pack::wasm_output::EffectIntent;

struct AdmittedEffect {
    operation: crate::operation::cli::AdmittedPlan,
    authority: crate::operation::shell::LocalReadOnlyAuthorityProvider,
}

pub(super) struct Failure {
    pub class: super::Class,
    pub run_ids: Vec<u64>,
}

// r[impl lattice.operation_packs_wasm.effects]
// r[impl lattice.operation_packs_wasm.fallback]
pub(super) fn execute(
    request: &super::super::execution::Request<'_>,
    parent_plan: &crate::operation::InvocationPlan,
    store: &crate::store::Store,
    intents: &[EffectIntent],
) -> Result<Vec<u64>, Failure> {
    // Admit the complete set before the first external effect. A malformed
    // later intent must not cause a partially admitted set to run.
    let process = request.app_config.effective_agent_runtime().local_process;
    let mut admitted = Vec::with_capacity(intents.len());
    for intent in intents {
        let plan = admit(request, parent_plan, &process, intent).map_err(|_| Failure {
            class: super::Class::Denied,
            run_ids: Vec::new(),
        })?;
        admitted.push(plan);
    }
    let mut run_ids = Vec::with_capacity(admitted.len());
    for effect in admitted {
        let credentials = crate::operation::shell::environment_provider_for_plan(&effect.operation.plan);
        let result = crate::operation::shell::run_direct(crate::operation::shell::DirectRunRequest {
            store,
            plan: &effect.operation.plan,
            authority_provider: &effect.authority,
            process: &process,
            credentials: &credentials,
            clock: &crate::operation::shell::SystemClock,
            cancellation: request.cancellation,
        });
        let result = match result {
            Ok(result) => result,
            Err(_) => {
                return Err(Failure {
                    class: super::Class::Unknown,
                    run_ids,
                });
            }
        };
        run_ids.push(result.record.run_id.0);
        match result.record.state {
            crate::operation::RunState::Completed => {}
            crate::operation::RunState::Started | crate::operation::RunState::Unknown => {
                return Err(Failure {
                    class: super::Class::Unknown,
                    run_ids,
                });
            }
            crate::operation::RunState::Failed => {
                return Err(Failure {
                    class: super::Class::Denied,
                    run_ids,
                });
            }
        }
    }
    Ok(run_ids)
}

fn admit(
    request: &super::super::execution::Request<'_>,
    parent_plan: &crate::operation::InvocationPlan,
    process: &crate::process::core::RuntimeConfig,
    intent: &EffectIntent,
) -> Result<AdmittedEffect, crate::operation_pack::Error> {
    let operation = request
        .admitted
        .candidate
        .manifest
        .operations
        .iter()
        .find(|operation| {
            operation.type_id == intent.type_id
                && operation.type_revision == intent.type_revision
                && operation.methods.contains(&intent.method)
        })
        .ok_or(crate::operation_pack::Error::Invalid("component effect operation"))?;
    let runtime = request
        .admitted
        .candidate
        .manifest
        .runtimes
        .iter()
        .find(|runtime| {
            runtime.id == operation.runtime_id && runtime.kind == "bounded_process" && runtime.wasm.is_none()
        })
        .ok_or(crate::operation_pack::Error::Invalid("component recursive or alternate effect runtime"))?;
    let catalog =
        super::super::content_path(request.pack_root, &request.admitted.candidate, &operation.catalog_content_id)?;
    let executable = super::super::content_path(request.pack_root, &request.admitted.candidate, &runtime.content_id)?;
    let arguments = serde_json::to_string(&intent.arguments).map_err(|_| crate::operation_pack::Error::Encoding)?;
    let admitted = crate::operation::cli::admit_for_pack_with_runtime(
        crate::operation::cli::PlanInput {
            catalog_path: &catalog,
            instances_path: request.instances_path,
            instance_id: &intent.instance_id,
            method: &intent.method,
            arguments_json: &arguments,
            executable_path: &executable,
        },
        process,
    )?;
    if admitted.plan.type_id != intent.type_id
        || admitted.plan.type_revision != intent.type_revision
        || admitted.plan.method != intent.method
        || admitted.plan.runtime.key != operation.launch_label
        || admitted.plan.capabilities.iter().any(|capability| {
            !runtime.capabilities.contains(capability) || !parent_plan.capabilities.contains(capability)
        })
        || admitted.plan.bounds.timeout_seconds > runtime.timeout_seconds
        || admitted.plan.bounds.input_bytes > runtime.input_bytes
        || admitted.plan.bounds.output_bytes > runtime.output_bytes
    {
        return Err(crate::operation_pack::Error::Invalid("component effect scope"));
    }
    let authority = crate::operation::shell::LocalReadOnlyAuthorityProvider::new(
        &admitted.catalog,
        &admitted.instance,
        &admitted.plan,
    )?;
    Ok(AdmittedEffect {
        operation: admitted,
        authority,
    })
}
