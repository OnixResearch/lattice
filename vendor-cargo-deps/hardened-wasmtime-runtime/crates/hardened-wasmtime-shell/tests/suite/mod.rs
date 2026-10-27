//! Real module and component executions, with positive and negative admission.
#[path = "../cases/abi.rs"]
mod abi;
#[path = "../cases/allocations.rs"]
mod allocations;
#[path = "../cases/checkpoints.rs"]
mod checkpoints;
#[path = "../cases/componentimports.rs"]
mod componentimports;
#[path = "../cases/imports.rs"]
mod imports;
#[path = "../cases/phases.rs"]
mod phases;
#[path = "../cases/resources.rs"]
mod resources;
#[path = "../cases/slicebounds.rs"]
mod slicebounds;
#[path = "../cases/slicelimits.rs"]
mod slicelimits;
#[path = "../cases/slicesupport.rs"]
mod slicesupport;
#[path = "../cases/slicing.rs"]
mod slicing;
#[path = "../cases/supervision.rs"]
mod supervision;
#[path = "../support/mod.rs"]
mod support;

#[test]
// r[verify onix.wasmtime_runtime.artifact_shape]
fn oversized_artifact_is_denied_before_compilation() {
    let mut bounds = support::profile().limits;
    bounds.max_component_bytes = 1;
    let engine = support::engine_with_bounds(bounds);
    let service = engine.epoch_service();
    let facts = support::module_facts(support::MODULE);
    assert!(matches!(
        hardened_wasmtime_shell::BoundedInvocation::prepare(
            &engine,
            support::options(support::MODULE, &facts, &service)
        ),
        Err(hardened_wasmtime_shell::PrepareError::ArtifactTooLarge)
    ));
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.valid]
fn source_product_module_returns_exact_output_and_remaining_fuel() {
    let engine = support::engine();
    let service = engine.epoch_service();
    let facts = support::module_facts(support::MODULE);
    let prepared = hardened_wasmtime_shell::BoundedInvocation::prepare(
        &engine,
        support::options(support::MODULE, &facts, &service),
    )
    .expect("module");
    let result = prepared.run(b"input", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    });
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(result.output, b"OK");
    assert!(result.fuel_remaining < support::profile().limits.fuel);
}

#[test]
// r[verify onix.wasmtime_runtime.artifact_shape.valid]
fn actual_component_returns_exact_output() {
    let bytes = support::component(
        "invoke",
        support::Body {
            action: "",
            cleanup: "",
        },
    );
    let engine = support::engine();
    let service = engine.epoch_service();
    let facts = support::facts(
        &bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "actor",
        &["invoke"],
    );
    let prepared = hardened_wasmtime_shell::ComponentInvocation::prepare(
        &engine,
        support::options(&bytes, &facts, &service),
    )
    .expect("actual component");
    let result = prepared.run("invoke", b"input", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    });
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(result.output, b"OK");
    assert!(result.fuel_remaining < support::profile().limits.fuel);
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.output]
fn component_output_is_refused_before_host_copy() {
    let profile = support::profile();
    let mut bounds = profile.limits;
    bounds.max_output_bytes = 1;
    let engine = hardened_wasmtime_shell::BoundedRuntime::build(
        &profile.policy,
        bounds,
        hardened_wasmtime_runtime::Strategy::Cranelift,
    )
    .expect("engine");
    let service = engine.epoch_service();
    let bytes = support::component(
        "render",
        support::Body {
            action: "",
            cleanup: "",
        },
    );
    let facts = support::facts(
        &bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "renderer",
        &["render"],
    );
    let prepared = hardened_wasmtime_shell::ComponentInvocation::prepare(
        &engine,
        support::options(&bytes, &facts, &service),
    )
    .expect("component");
    let outcome = prepared.run("render", b"", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    });
    assert_eq!(
        outcome.class,
        hardened_wasmtime_runtime::outcomes::Class::OutputLimitExceeded
    );
    assert!(outcome.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.epoch_deadline.service]
fn absent_or_crossed_epoch_service_is_denied() {
    let engine = support::engine();
    let service = engine.epoch_service();
    let other = support::engine().epoch_service();
    let facts = support::module_facts(support::MODULE);
    for supplied in [None, Some(&other)] {
        let mut options = support::options(support::MODULE, &facts, &service);
        options.epoch_service = supplied;
        let failure = hardened_wasmtime_shell::BoundedInvocation::prepare(&engine, options).err();
        assert_eq!(
            failure,
            Some(hardened_wasmtime_shell::PrepareError::DeadlineDenied)
        );
    }
}

#[test]
// r[verify onix.wasmtime_runtime.epoch_deadline.service]
fn zero_or_unbounded_epoch_budget_is_denied() {
    let engine = support::engine();
    let service = engine.epoch_service();
    let facts = support::module_facts(support::MODULE);
    for ticks in [0, u64::MAX] {
        let mut options = support::options(support::MODULE, &facts, &service);
        options.deadline.budget_ticks = ticks;
        let failure = hardened_wasmtime_shell::BoundedInvocation::prepare(&engine, options).err();
        assert_eq!(
            failure,
            Some(hardened_wasmtime_shell::PrepareError::DeadlineDenied)
        );
    }
}

#[test]
// r[verify onix.wasmtime_runtime.artifact_shape.import]
fn identity_class_and_export_drift_deny_before_guest_execution() {
    let engine = support::engine();
    let service = engine.epoch_service();
    let base = support::module_facts(support::MODULE);
    let mut identity = base.clone();
    identity.artifact_identity = String::from("wrong");
    let mut class = base.clone();
    class.class = hardened_wasmtime_runtime::artifact::shape::Class::Component;
    let mut omitted = base;
    omitted.exports.remove(&support::function(&["run"]));
    for facts in [identity, class, omitted] {
        let options = support::options(support::MODULE, &facts, &service);
        let failure = hardened_wasmtime_shell::BoundedInvocation::prepare(&engine, options).err();
        assert_eq!(
            failure,
            Some(hardened_wasmtime_shell::PrepareError::ArtifactMismatch)
        );
    }
}

#[test]
// r[verify onix.wasmtime_runtime.feature_policy.unsupported]
fn forged_policy_and_invalid_bounds_cannot_construct_an_engine() {
    let profile = support::profile();
    let mut policy = profile.policy;
    policy.features.threads = true;
    assert!(matches!(
        hardened_wasmtime_shell::BoundedRuntime::build(
            &policy,
            profile.limits,
            hardened_wasmtime_runtime::Strategy::Cranelift
        ),
        Err(hardened_wasmtime_shell::BuildFailure::ProfileDenied)
    ));
    let mut bounds = profile.limits;
    bounds.fuel = 0;
    assert!(matches!(
        hardened_wasmtime_shell::BoundedRuntime::build(
            &profile.policy,
            bounds,
            hardened_wasmtime_runtime::Strategy::Cranelift
        ),
        Err(hardened_wasmtime_shell::BuildFailure::BoundsDenied)
    ));
}
