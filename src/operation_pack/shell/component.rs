//! Thin offline component shell. Guest output never grants product authority.

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use crate::operation::shell::Clock as _;

mod bundle;
mod control;
mod effects;
mod observation;
mod planning;
pub mod verification;

pub use observation::Class;
pub use observation::PostReturn;
pub use observation::Receipt;

static ACTIVE_COMPONENT: AtomicBool = AtomicBool::new(false);

struct ActiveComponent;
impl Drop for ActiveComponent {
    fn drop(&mut self) {
        ACTIVE_COMPONENT.store(false, Ordering::Release);
    }
}

// r[impl lattice.operation_packs_wasm.shell]
// r[impl lattice.operation_packs_wasm.lifecycle]
// r[impl lattice.operation_packs_wasm.fallback]
pub(super) fn run(
    request: &super::execution::Request<'_>,
    operation: &crate::operation_pack::model::OperationBinding,
    runtime: &crate::operation_pack::model::RuntimeArtifact,
) -> Result<super::execution::Report, crate::operation_pack::Error> {
    // The first Lattice profile admits one active component pack in this
    // process. Producers additionally enforce each runtime object's limit.
    ACTIVE_COMPONENT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| crate::operation_pack::Error::Invalid("component concurrency bound"))?;
    let _active = ActiveComponent;
    let admitted_plan = planning::admit(request, operation, runtime)?;
    let input =
        serde_json::to_vec(&admitted_plan.plan.arguments).map_err(|_| crate::operation_pack::Error::Encoding)?;
    let descriptor = runtime.wasm.as_ref().ok_or(crate::operation_pack::Error::Invalid("component descriptor"))?;
    let mapping = descriptor
        .methods
        .iter()
        .find(|mapping| mapping.type_id == operation.type_id && mapping.method == request.method)
        .ok_or(crate::operation_pack::Error::Invalid("component method mapping"))?;
    let input_bytes = mapping
        .guest_method
        .len()
        .checked_add(input.len())
        .and_then(|size| u64::try_from(size).ok())
        .ok_or(crate::operation_pack::Error::Invalid("component input bound"))?;
    let prepared = verification::prepare(request, runtime, operation, &admitted_plan.plan, input_bytes)?;
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(descriptor.resources.deadline_millis))
        .ok_or(crate::operation_pack::Error::Invalid("component deadline"))?;
    let data_root = super::execution::absolute_data_root(&request.app_config.data_dir)?;
    std::fs::create_dir_all(&data_root)
        .map_err(|_| crate::operation_pack::Error::Io("component operation data directory"))?;
    let store_path = crate::config::store_path(&data_root);
    let store = crate::store::Store::new(&store_path)
        .map_err(|error| crate::operation_pack::Error::Diagnostic(error.to_string()))?;
    if let Some(previous) = store
        .load_operation_run(&admitted_plan.plan.identity)
        .map_err(|error| crate::operation_pack::Error::Diagnostic(error.to_string()))?
    {
        if previous.state == crate::operation::RunState::Started
            || previous.state == crate::operation::RunState::Unknown
        {
            // An observed Started record may belong to a still-live invocation.
            // Report uncertainty without changing evidence or dispatching again.
            return Ok(super::execution::Report {
                pack_identity: request.admitted.candidate.pack_identity.clone(),
                lock_identity: request.admitted.lock.identity.clone(),
                invocation_identity: admitted_plan.plan.identity,
                run_id: previous.run_id.0,
                operation_state: crate::operation::RunState::Unknown,
                status: super::execution::Status::Unknown,
                operation_data_version: None,
                analyzer_outcomes: Vec::new(),
                component: None,
                effect_run_ids: Vec::new(),
                output_content_omitted: true,
                credentials_omitted: true,
                private_paths_omitted: true,
            });
        }
        return Err(crate::operation_pack::Error::Invalid(
            "component invocation already has durable terminal evidence",
        ));
    }
    let started = store
        .begin_operation_run(&admitted_plan.plan, crate::operation::shell::SystemClock.observe_epoch_millis())
        .map_err(|error| crate::operation_pack::Error::Diagnostic(error.to_string()))?;
    // No component instantiation, initialization, invocation or post-return
    // is reachable until begin_operation_run has durably returned Started.
    let (receipt, observation, effect_run_ids) =
        if crate::operation::backend::admit_dispatch(&admitted_plan.plan, &started.backend_attempt).is_err() {
            (
                None,
                crate::operation::runtime::backend_denied("component backend route changed after durable start"),
                Vec::new(),
            )
        } else {
            let (receipt, observation, effects) =
                execute(request, &admitted_plan.plan, &prepared, &store, &input, deadline);
            (Some(receipt), observation, effects)
        };
    let terminal = started.finished(&observation, crate::operation::shell::SystemClock.observe_epoch_millis());
    store
        .finish_operation_run(&terminal)
        .map_err(|error| crate::operation_pack::Error::Diagnostic(error.to_string()))?;
    drop(store);
    let run_id = terminal.run_id.0;
    let operation_state = terminal.state;
    let mut operation_data_version = None;
    let mut references = Vec::new();
    if let Some(output) = observation.output {
        let plane = crate::operation_data::Plane::open(&store_path, &data_root)?;
        let draft = super::execution::operation_record_draft(
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
        references.push(crate::operation_pack::analyzer::DataReference {
            version_identity: metadata.version_identity,
            content_identity: metadata.payload.content_identity,
        });
    }
    let process = request.app_config.effective_agent_runtime().local_process;
    let analyzer_outcomes = super::execution::run_analyzers(
        request,
        &process,
        run_id,
        operation_state,
        &references,
        &store_path,
        &data_root,
    )?;
    let is_completed = operation_state == crate::operation::RunState::Completed && operation_data_version.is_some();
    let status = if crate::operation_pack::analyzer::parent_succeeded(is_completed, &analyzer_outcomes) {
        super::execution::Status::Completed
    } else if operation_state == crate::operation::RunState::Unknown {
        super::execution::Status::Unknown
    } else {
        super::execution::Status::Failed
    };
    Ok(super::execution::Report {
        pack_identity: request.admitted.candidate.pack_identity.clone(),
        lock_identity: request.admitted.lock.identity.clone(),
        invocation_identity: admitted_plan.plan.identity,
        run_id,
        operation_state,
        status,
        operation_data_version,
        analyzer_outcomes,
        component: receipt,
        effect_run_ids,
        output_content_omitted: true,
        credentials_omitted: true,
        private_paths_omitted: true,
    })
}

fn execute(
    request: &super::execution::Request<'_>,
    plan: &crate::operation::InvocationPlan,
    prepared: &verification::Prepared,
    store: &crate::store::Store,
    input: &[u8],
    deadline: Instant,
) -> (Receipt, crate::operation::runtime::Observation, Vec<u64>) {
    let raw = control::invoke(prepared, input, deadline, request.cancellation);
    let mut receipt = Receipt::from_observation(prepared, &raw);
    if receipt.outcome != Class::Completed {
        let observation = observation::failed(receipt.outcome);
        return (receipt, observation, Vec::new());
    }
    let Some(bytes) = raw.invocation.map(|invocation| invocation.output) else {
        receipt.outcome = Class::Unknown;
        return (receipt, observation::failed(Class::Unknown), Vec::new());
    };
    let output = match crate::operation_pack::wasm_output::admit(
        &bytes,
        request.admitted,
        &prepared.dispatch.descriptor,
        plan,
        request.prohibited_fragments,
    ) {
        Ok(output) => output,
        Err(blocker) => {
            receipt.outcome = match blocker {
                crate::operation_pack::wasm_output::Blocker::Plan
                | crate::operation_pack::wasm_output::Blocker::IntentTarget => Class::Denied,
                _ => Class::InvalidOutput,
            };
            let observation = observation::failed(receipt.outcome);
            return (receipt, observation, Vec::new());
        }
    };
    let effect_run_ids = match effects::execute(request, plan, store, output.intents()) {
        Ok(run_ids) => run_ids,
        Err(error) => {
            receipt.outcome = error.class;
            return (receipt, observation::failed(error.class), error.run_ids);
        }
    };
    let observation = match observation::completed(output.into_result()) {
        Ok(observation) => observation,
        Err(_) => {
            receipt.outcome = Class::Unknown;
            observation::failed(Class::Unknown)
        }
    };
    (receipt, observation, effect_run_ids)
}
