//! Closed operation contract for the import-free component-pack adapter.
//!
//! Product effects are proposals returned to the existing operation owners;
//! this runtime grants no guest imports, target roots, or credentials.

pub const RUNTIME_KIND: &str = "wasm-component";
pub const RUNTIME_KEY: &str = "lattice-wasm-component-v1";

pub fn runtime_is_component(runtime: &super::super::model::RuntimeRequirement) -> bool {
    runtime.kind == RUNTIME_KIND && runtime.key == RUNTIME_KEY
}

pub(crate) fn type_is_supported(definition: &super::super::model::OperationType) -> bool {
    use super::super::backend::ExecutionFormat;
    use super::super::backend::FilesystemSemantics;
    use super::super::backend::IsolationClaim;
    use super::super::backend::NetworkPosture;
    use super::super::backend::PeripheralNeed;
    use super::super::backend::PersistenceNeed;
    use super::super::backend::ProcessModel;
    use super::super::backend::TerminalNeed;
    let requirements = &definition.execution_requirements;
    runtime_is_component(&definition.runtime)
        && requirements.format == ExecutionFormat::WasmComponent
        && requirements.filesystem == FilesystemSemantics::NoFilesystem
        && requirements.network == NetworkPosture::NetworkDenied
        && requirements.process == ProcessModel::InProcess
        && requirements.terminal == TerminalNeed::NoTerminal
        && requirements.peripheral == PeripheralNeed::NoPeripheral
        && requirements.persistence == PersistenceNeed::Ephemeral
        && requirements.isolation == IsolationClaim::MediatedIsolation
        && definition.credential_bindings.is_empty()
        && definition.methods.iter().all(|method| {
            method.side_effect == super::super::model::SideEffectClass::ReadOnly
                && method.preflight == [super::super::model::PreflightCheck::RuntimeAvailable]
        })
}
