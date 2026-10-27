//! Behavioral lifecycle proof over two real, already materialized offline packs.
//! The input JSON contains component/executable fixtures and effect_method.
//! Each fixture names pack_root, lock, admission, instances, instance and method.

use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use lattice::operation::RunState;
use lattice::operation::{self};
use lattice::operation_pack::shell::execution;
use lattice::operation_pack::{self as pack};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    pack_root: PathBuf,
    lock: PathBuf,
    admission: PathBuf,
    instances: PathBuf,
    instance: String,
    method: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    component: Fixture,
    executable: Fixture,
    effect_method: String,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn admit(fixture: &Fixture) -> Result<pack::AdmittedPack, Box<dyn std::error::Error>> {
    let candidate = pack::shell::load(&fixture.pack_root)?;
    let lock = read_json(&fixture.lock)?;
    let facts = read_json(&fixture.admission)?;
    Ok(pack::admit(&candidate, &lock, &facts)?)
}

fn execute(
    fixture: &Fixture,
    admitted: &pack::AdmittedPack,
    data_root: &Path,
    method: &str,
    cancellation: Option<&AtomicBool>,
) -> Result<execution::Report, pack::Error> {
    let app_config = lattice::config::AppConfig {
        data_dir: data_root.to_path_buf(),
        ..lattice::config::AppConfig::default()
    };
    execution::run(execution::Request {
        app_config: &app_config,
        pack_root: &fixture.pack_root,
        admitted,
        instances_path: &fixture.instances,
        instance_id: &fixture.instance,
        method,
        arguments_json: "{}",
        sequence: 1,
        prohibited_fragments: &[],
        cancellation,
    })
}

fn persisted(data_root: &Path, report: &execution::Report) -> Result<operation::RunRecord, Box<dyn std::error::Error>> {
    let store = lattice::store::Store::new(&lattice::config::store_path(data_root))?;
    let record = store.load_operation_run(&report.invocation_identity)?.ok_or("missing durable parent attempt")?;
    assert_eq!(record.run_id.0, report.run_id);
    assert_eq!(record.state, report.operation_state);
    assert!(record.finished_epoch_millis.is_some_and(|finish| finish >= record.started_epoch_millis));
    Ok(record)
}

fn plan(
    fixture: &Fixture,
    admitted: &pack::AdmittedPack,
) -> Result<operation::InvocationPlan, Box<dyn std::error::Error>> {
    let binding = admitted
        .candidate
        .manifest
        .operations
        .iter()
        .find(|binding| binding.methods.contains(&fixture.method))
        .ok_or("missing fixture operation")?;
    let runtime = admitted
        .candidate
        .manifest
        .runtimes
        .iter()
        .find(|runtime| runtime.id == binding.runtime_id)
        .ok_or("missing fixture runtime")?;
    let catalog = pack::shell::content_path(&fixture.pack_root, &admitted.candidate, &binding.catalog_content_id)?;
    let executable = pack::shell::content_path(&fixture.pack_root, &admitted.candidate, &runtime.content_id)?;
    let mut plan = operation::cli::admit_plan(operation::cli::PlanInput {
        catalog_path: &catalog,
        instances_path: &fixture.instances,
        instance_id: &fixture.instance,
        method: &fixture.method,
        arguments_json: "{}",
        executable_path: &executable,
    })?;
    if let Some(component) = &runtime.wasm {
        let resources = &component.resources;
        plan.runtime.maximum_bounds = operation::Bounds {
            timeout_seconds: resources.deadline_millis / 1000,
            input_bytes: usize::try_from(resources.maximum_input_bytes)?,
            output_bytes: usize::try_from(resources.maximum_output_bytes)?,
            artifact_bytes: usize::try_from(resources.maximum_component_bytes)?,
        };
        plan.backend_route = operation::backend::plan_route(&operation::backend::RouteContext {
            requirements: &plan.execution_requirements,
            runtime: &plan.runtime,
            method_identity: &plan.method_identity,
            capabilities: &plan.capabilities,
            credentials: &plan.credentials,
            bounds: plan.bounds,
        })?;
        plan.identity = operation::identity::invocation(&plan)?;
    }
    operation::planning::validate_plan_identity(&plan)?;
    Ok(plan)
}

fn contains_value(value: &serde_json::Value, target: &serde_json::Value) -> bool {
    value == target
        || match value {
            serde_json::Value::Array(values) => values.iter().any(|value| contains_value(value, target)),
            serde_json::Value::Object(values) => values.values().any(|value| contains_value(value, target)),
            _ => false,
        }
}

fn private_report(
    fixture: &Fixture,
    data_root: &Path,
    report: &execution::Report,
) -> Result<(), Box<dyn std::error::Error>> {
    let encoded = serde_json::to_string(report)?;
    for path in [&fixture.pack_root, &fixture.instances] {
        assert!(!encoded.contains(path.to_str().ok_or("fixture path encoding")?));
    }
    let version = report.operation_data_version.as_deref().ok_or("missing admitted operation output")?;
    let plane = lattice::operation_data::Plane::open(&lattice::config::store_path(data_root), data_root)?;
    let metadata = plane.load_metadata(version)?.ok_or("missing published operation data")?;
    let payload: serde_json::Value = serde_json::from_slice(&plane.read_payload(&metadata)?)?;
    let projected = serde_json::to_value(report)?;
    assert!(!contains_value(&projected, &payload), "safe report disclosed the actual operation payload");
    Ok(())
}

fn lifecycle(
    fixture: &Fixture,
    admitted: &pack::AdmittedPack,
    root: &Path,
    component: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let completed_root = root.join("completed");
    let completed = execute(fixture, admitted, &completed_root, &fixture.method, None)?;
    assert_eq!(completed.status, execution::Status::Completed);
    assert_eq!(completed.operation_state, RunState::Completed);
    assert!(completed.operation_data_version.is_some());
    let record = persisted(&completed_root, &completed)?;
    assert_eq!(record.backend_attempt.state, operation::BackendAttemptState::Observed);
    // Compare the actual published payload and private inputs with the safe
    // report, rather than trusting its omission booleans as proof.
    private_report(fixture, &completed_root, &completed)?;
    assert_eq!(completed.component.is_some(), component);
    if let Some(receipt) = &completed.component {
        assert!(receipt.store_dropped && receipt.observer_joined);
        assert_eq!(receipt.post_return, pack::shell::component::PostReturn::Succeeded);
        assert!(!receipt.automatic_retry_permitted);
    }
    // A second request with the exact invocation identity is not a new attempt.
    assert!(execute(fixture, admitted, &completed_root, &fixture.method, None).is_err());
    assert_eq!(persisted(&completed_root, &completed)?, record);

    let cancelled_root = root.join("cancelled");
    let cancelled = execute(fixture, admitted, &cancelled_root, &fixture.method, Some(&AtomicBool::new(true)))?;
    assert_ne!(cancelled.operation_state, RunState::Completed);
    let cancelled_record = persisted(&cancelled_root, &cancelled)?;
    assert!(cancelled_record.output_identity.is_none());
    if let Ok(recovered) = execute(fixture, admitted, &cancelled_root, &fixture.method, None) {
        assert_eq!(recovered.operation_state, RunState::Unknown);
        assert_eq!(recovered.run_id, cancelled.run_id);
        assert!(recovered.component.is_none() && recovered.operation_data_version.is_none());
    }
    assert_eq!(persisted(&cancelled_root, &cancelled)?, cancelled_record);

    // Seed a genuine durable Started attempt using the ordinary store owner,
    // modeling process interruption between its commit and guest dispatch.
    let expected = plan(fixture, admitted)?;
    assert_eq!(expected.identity, completed.invocation_identity);
    let interrupted_root = root.join("interrupted");
    std::fs::create_dir_all(&interrupted_root)?;
    let store = lattice::store::Store::new(&lattice::config::store_path(&interrupted_root))?;
    let started = store.begin_operation_run(&expected, 1)?;
    drop(store);
    let recovered = execute(fixture, admitted, &interrupted_root, &fixture.method, None);
    if component {
        let recovered = recovered?;
        assert_eq!(recovered.operation_state, RunState::Unknown);
        assert_eq!(recovered.run_id, started.run_id.0);
        assert!(recovered.component.is_none() && recovered.operation_data_version.is_none());
    } else {
        assert!(recovered.is_err());
    }
    let store = lattice::store::Store::new(&lattice::config::store_path(&interrupted_root))?;
    assert_eq!(store.load_operation_run(&expected.identity)?, Some(started));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let input = args.next().ok_or("usage: wasm_operation_pack_conformance INPUTS.json NEW-DATA-ROOT")?;
    let root = PathBuf::from(args.next().ok_or("missing create-only data root")?);
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let inputs: Inputs = read_json(Path::new(&input))?;
    std::fs::create_dir(&root)?;
    let component = admit(&inputs.component)?;
    let executable = admit(&inputs.executable)?;
    // r[verify lattice.operation_packs_wasm.lifecycle]
    // r[verify lattice.operation_packs_wasm.fallback]
    lifecycle(&inputs.component, &component, &root.join("component"), true)?;
    lifecycle(&inputs.executable, &executable, &root.join("executable"), false)?;
    let effects_root = root.join("effects");
    let effects = execute(&inputs.component, &component, &effects_root, &inputs.effect_method, None)?;
    assert_eq!(effects.status, execution::Status::Completed);
    persisted(&effects_root, &effects)?;
    private_report(&inputs.component, &effects_root, &effects)?;
    assert!(!effects.effect_run_ids.is_empty());
    assert!(effects.effect_run_ids.iter().all(|effect| *effect > effects.run_id));
    let store = lattice::store::Store::new(&lattice::config::store_path(&effects_root))?;
    for effect in &effects.effect_run_ids {
        let observed =
            store.load_run(lattice::engine::executor::RunId(*effect))?.ok_or("missing durable effect attempt")?;
        assert_eq!(observed.run_id.0, *effect);
        assert!(observed.finished_epoch_ms.is_some());
        assert_eq!(observed.failures, 0);
    }
    println!(
        "real executable/component durable completion, cancellation, duplicate denial, unknown recovery and Started-before-effect ordering passed; outputs need not be equivalent"
    );
    Ok(())
}
