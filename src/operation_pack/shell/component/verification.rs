//! Offline bundle remeasurement and native component preflight; no guest execution.
use hardened_wasmtime_runtime::artifact::shape::Class;
use hardened_wasmtime_runtime::artifact::shape::InterfaceFunction;
use hardened_wasmtime_runtime::artifact::shape::MaterializationFacts;
use hardened_wasmtime_runtime::artifact::shape::WorldIdentity;
use hardened_wasmtime_runtime::epoch::deadline::Budget;
use hardened_wasmtime_shell::Artifact;
use hardened_wasmtime_shell::BoundedRuntime;
use hardened_wasmtime_shell::ByteMethodInvocation;
use hardened_wasmtime_shell::ComponentInvocation;
use hardened_wasmtime_shell::EpochService;
use hardened_wasmtime_shell::PrepareOptions;

use crate::operation_pack::model;
use crate::operation_pack::wasm;

pub(super) struct Prepared {
    pub component: ByteMethodInvocation,
    pub epoch_service: EpochService,
    pub dispatch: wasm::DispatchPlan,
    pub canonical_bundle_identity: String,
}

// r[impl lattice.operation_packs_wasm.shell]
// r[impl lattice.operation_packs_wasm.lock]
pub(super) fn prepare(
    request: &super::super::execution::Request<'_>,
    runtime: &model::RuntimeArtifact,
    operation: &model::OperationBinding,
    plan: &crate::operation::InvocationPlan,
    input_bytes: u64,
) -> Result<Prepared, crate::operation_pack::Error> {
    let descriptor = runtime.wasm.as_ref().ok_or(crate::operation_pack::Error::Invalid("component descriptor"))?;
    let profile = wasm::profile::admit(descriptor)?;
    if profile.identity != descriptor.runtime_profile_identity {
        return Err(crate::operation_pack::Error::Invalid("component runtime profile identity"));
    }
    let bundle = super::bundle::verify(request, runtime, descriptor)?;
    if bundle.component_bytes.len() as u64 > profile.bounds.max_component_bytes
        || wasm::FIRST_WORLD_WIT.len() as u64 > profile.bounds.max_wit_bytes
    {
        return Err(crate::operation_pack::Error::Invalid("component or WIT byte bound"));
    }
    let runtime_owner =
        BoundedRuntime::build(&profile.policy, profile.bounds, hardened_wasmtime_runtime::Strategy::Cranelift)
            .map_err(|_| crate::operation_pack::Error::Invalid("shared component engine"))?;
    let epoch_service = runtime_owner.epoch_service();
    let materialization = MaterializationFacts {
        artifact_identity: blake3::hash(&bundle.component_bytes).to_hex().to_string(),
        class: Class::Component,
        world: WorldIdentity {
            package: descriptor.wit_package.clone(),
            world: descriptor.world.clone(),
        },
        imports: std::collections::BTreeSet::new(),
        exports: [InterfaceFunction {
            qualified_name: hardened_wasmtime_shell::qualified_name(&[
                &descriptor.export_interface,
                &descriptor.invoke,
            ]),
            schema_identity: bundle.wit_identity.clone(),
        }]
        .into_iter()
        .collect(),
    };
    let component = ComponentInvocation::prepare(&runtime_owner, PrepareOptions {
        artifact: Artifact {
            bytes: &bundle.component_bytes,
            facts: &materialization,
        },
        descriptors: &[],
        deadline: Budget { budget_ticks: 1 },
        epoch_service: Some(&epoch_service),
    })
    .and_then(|component| component.into_byte_method(&descriptor.export_interface, &descriptor.invoke))
    .map_err(|_| crate::operation_pack::Error::Invalid("component world, import or method admission"))?;
    // Exact byte/name admission and named signature reflection have now succeeded
    // without instantiation; these are observed facts, not declaration echoes.
    let facts = wasm::DispatchFacts {
        platform: super::super::platform_label(),
        runtime_available: true,
        runtime_revision: wasm::RUNTIME_REVISION.to_string(),
        runtime_profile_identity: profile.identity,
        bundle: wasm::BundleFacts {
            bundle_identity: bundle.bundle_content_identity,
            component_identity: bundle.component_identity,
            wit_identity: bundle.wit_identity,
            verifier_revision: wasm::VERIFIER_REVISION.to_string(),
            valid: true,
            current: true,
            overclaimed: false,
        },
        world: wasm::WorldFacts {
            wit_package: wasm::WIT_PACKAGE.to_string(),
            world: wasm::WORLD.to_string(),
            export_interface: descriptor.export_interface.clone(),
            invoke: descriptor.invoke.clone(),
            imports: Vec::new(),
        },
        resources: descriptor.resources.clone(),
        input_bytes,
        authority: wasm::AuthorityFacts {
            pack_identity: request.admitted.candidate.pack_identity.clone(),
            lock_identity: request.admitted.lock.identity.clone(),
            type_id: plan.type_id.clone(),
            type_revision: plan.type_revision.clone(),
            method: plan.method.clone(),
            capabilities: plan.capabilities.clone(),
            operation_admitted: true,
        },
    };
    let dispatch = wasm::admit_dispatch(request.admitted, &runtime.id, &operation.type_id, request.method, &facts)
        .map_err(|_| crate::operation_pack::Error::Invalid("closed component dispatch admission"))?;
    Ok(Prepared {
        component,
        epoch_service,
        dispatch,
        canonical_bundle_identity: bundle.canonical_bundle_identity,
    })
}

pub(super) fn read_content(
    root: &cap_std::fs::Dir,
    candidate: &model::Candidate,
    content_id: &str,
) -> Result<Vec<u8>, crate::operation_pack::Error> {
    let declared = candidate
        .manifest
        .contents
        .iter()
        .find(|content| content.id == content_id)
        .ok_or(crate::operation_pack::Error::Invalid("component content role"))?;
    let file = crate::capability::open_relative_read(root, std::path::Path::new(&declared.path))
        .map_err(|_| crate::operation_pack::Error::Io("component capability read"))?;
    let metadata = file.metadata().map_err(|_| crate::operation_pack::Error::Io("component file observation"))?;
    if !metadata.is_file() || metadata.len() != declared.byte_length {
        return Err(crate::operation_pack::Error::Invalid("component content file"));
    }
    let maximum = usize::try_from(declared.byte_length)
        .map_err(|_| crate::operation_pack::Error::Invalid("component content length"))?;
    let bytes = crate::capability::read_bounded(file, maximum)
        .map_err(|_| crate::operation_pack::Error::Io("component bounded read"))?;
    if bytes.len() != maximum || crate::operation_pack::identity::content_identity(&bytes) != declared.content_identity
    {
        return Err(crate::operation_pack::Error::Invalid("component content drift"));
    }
    Ok(bytes)
}
