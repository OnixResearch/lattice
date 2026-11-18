//! Operator-authored worker compute and model requirements.
//!
//! `contracts/worker-execution-binding.ncl` is the authoring contract. Its
//! export decodes into the strict Rust shapes below and is revalidated by the
//! pure core, so an unsupported requirement fails before any effect.

// r[impl lattice.execution_binding.profile_facts.scenario.unsupported]

pub const CONFIGURATION_SCHEMA: &str = "lattice.worker-execution-binding.v1";

/// One worker's exported requirements and its per-attempt declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerConfiguration {
    pub schema: String,
    pub worker_compute: lattice_core::execution_binding::request::ComputeRequirement,
    pub worker_model: lattice_core::execution_binding::request::ModelRequirement,
    pub execution_binding: lattice_core::execution_binding::request::Declaration,
}

/// Evaluate one Nickel fixture through the pinned evaluator.
pub fn decode_fixture(path: &std::path::Path) -> Result<serde_json::Value, String> {
    crate::config::load_nickel_as_json(path)
}

/// Evaluate, decode and structurally validate one configuration.
pub fn load_configuration(path: &std::path::Path) -> Result<WorkerConfiguration, String> {
    let configuration: WorkerConfiguration =
        serde_json::from_value(decode_fixture(path)?).map_err(|error| error.to_string())?;
    if configuration.schema != CONFIGURATION_SCHEMA {
        return Err("worker execution-binding configuration schema is unsupported".into());
    }
    let requirement = lattice_core::execution_binding::request::WorkerRequirement {
        compute: configuration.worker_compute.clone(),
        model: configuration.worker_model.clone(),
    };
    lattice_core::execution_binding::request::validate_requirement(&requirement)
        .and_then(|()| lattice_core::execution_binding::request::validate_declaration(&configuration.execution_binding))
        .map_err(|defect| format!("worker execution-binding configuration is unsupported: {defect:?}"))?;
    debug_assert_eq!(configuration.schema, CONFIGURATION_SCHEMA);
    debug_assert_eq!(requirement.compute, configuration.worker_compute);
    let is_host_local_requirement = matches!(
        configuration.worker_compute,
        lattice_core::execution_binding::request::ComputeRequirement::HostLocal {}
    );
    let is_host_local_declaration = matches!(
        configuration.execution_binding.compute,
        lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {}
    );
    if is_host_local_requirement != is_host_local_declaration {
        return Err("worker compute requirement and declared placement differ".into());
    }
    Ok(configuration)
}
