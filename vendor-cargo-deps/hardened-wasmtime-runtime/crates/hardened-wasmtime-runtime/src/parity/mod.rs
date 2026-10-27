//! Strategy-parity differential rail: Cranelift versus Winch.
//!
//! Executes a bounded corpus of small WebAssembly modules under both
//! strategies with fuel metering, normalizes the outcomes through the pure
//! core, and compares the admitted observations through the pinned
//! `differential-execution-core` boundary. Agreement is parity on this corpus
//! only; it does not prove correctness, conformance, or sandbox
//! effectiveness.
//!
//! Foreign and own-crate items are referenced by fully qualified paths at the
//! call site rather than imported, per the Octet import policy.
#![allow(
    clippy::missing_panics_doc,
    reason = "the single deliberate panic path documents unreachable corpus states"
)]

const DIFF_DOMAIN: &[u8] = b"hardened-wasmtime.strategy-parity.v1";
const DIFF_VALUE_SCHEMA: &[u8] = b"hardened-wasmtime-outcome-v1";
const DIFF_ADAPTER: &[u8] = b"strategy-parity-rail-v1";
const DIFF_BACKEND_COHORT: &[u8] = b"wasmtime-strategies";
const DIFF_RUNTIME_COHORT: &[u8] = b"wasmtime-41-v1";
const DIFF_RESOURCES: &[u8] = b"bounded-host";
const DIFF_EFFECTS: &[u8] = b"bounded-wasm-exec";
const DIFF_NORMALIZATION: &[u8] = b"normalized-outcome-v1";
const RAIL_FUEL_BUDGET: u64 = 10_000;
const RAIL_MEMORY_PAGE_LIMIT: u32 = 1;
/// Lowercase hex length of one BLAKE3 identity.
const IDENTITY_HEX_LENGTH: usize = 64;
/// Corpus modules export `run`; arity and results are fixed for the rail.
const RUN_EXPORT: &str = "run";

/// One bounded corpus case: a WebAssembly text module plus its expected
/// normalized outcome class under any admitted profile.
pub struct CorpusCase {
    /// Stable case label bound into the harness identity.
    pub label: &'static str,
    /// WebAssembly text source for the case module.
    pub wat: &'static str,
    /// Outcome class both strategies must produce.
    pub expected: crate::NormalizedOutcome,
}

const ADD_MODULE: &str = r#"(module (func (export "run") i32.const 40 i32.const 2 i32.add drop))"#;

const UNREACHABLE_MODULE: &str = r#"(module (func (export "run") unreachable))"#;

const INFINITE_LOOP_MODULE: &str = r#"(module (func (export "run") (loop br 0)))"#;

/// The bounded strategy-parity corpus.
pub fn corpus() -> Vec<CorpusCase> {
    vec![
        CorpusCase {
            label: "arithmetic-completion",
            wat: ADD_MODULE,
            expected: crate::NormalizedOutcome::Completed,
        },
        CorpusCase {
            label: "unreachable-trap",
            wat: UNREACHABLE_MODULE,
            expected: crate::NormalizedOutcome::Trapped(crate::TrapClass::Unreachable),
        },
        CorpusCase {
            label: "fuel-exhaustion",
            wat: INFINITE_LOOP_MODULE,
            expected: crate::NormalizedOutcome::FuelExhausted,
        },
    ]
}

/// Executes one module under one strategy with metered fuel and normalizes
/// the result through the pure core.
///
/// # Errors
/// Returns engine, compilation, or instantiation diagnostics verbatim.
pub fn execute(
    strategy: crate::Strategy,
    limits: crate::ResourceLimits,
    wat: &str,
) -> Result<crate::NormalizedOutcome, String> {
    assert!(
        limits.fuel >= crate::MINIMUM_FUEL,
        "rail fuel budget must satisfy the admitted minimum"
    );
    assert!(!wat.is_empty(), "corpus modules must carry WAT source");
    let mut config = wasmtime::Config::new();
    config.strategy(match strategy {
        crate::Strategy::Cranelift => wasmtime::Strategy::Cranelift,
        crate::Strategy::Winch => wasmtime::Strategy::Winch,
    });
    config
        .consume_fuel(true)
        .wasm_gc(false)
        .collector(wasmtime::Collector::Null)
        .wasm_function_references(false)
        .wasm_exceptions(false);

    let engine = wasmtime::Engine::new(&config).map_err(|error| format!("engine: {error}"))?;
    let module = wasmtime::Module::new(&engine, wat).map_err(|error| format!("module: {error}"))?;
    let mut store = wasmtime::Store::new(&engine, ());
    store
        .set_fuel(limits.fuel)
        .map_err(|error| format!("set fuel: {error}"))?;

    let instance = wasmtime::Instance::new(&mut store, &module, &[])
        .map_err(|error| format!("instantiate: {error}"))?;
    let run = instance
        .get_typed_func::<(), ()>(&mut store, RUN_EXPORT)
        .map_err(|error| format!("resolve export: {error}"))?;

    match run.call(&mut store, ()) {
        Ok(()) => Ok(crate::NormalizedOutcome::Completed),
        Err(error) => {
            let fuel_remaining = store.get_fuel().unwrap_or(0);
            if fuel_remaining == 0 {
                return Ok(crate::NormalizedOutcome::FuelExhausted);
            }
            if error.downcast_ref::<wasmtime::Trap>()
                == Some(&wasmtime::Trap::UnreachableCodeReached)
            {
                return Ok(crate::NormalizedOutcome::Trapped(
                    crate::TrapClass::Unreachable,
                ));
            }
            Ok(crate::NormalizedOutcome::Trapped(crate::TrapClass::Other))
        }
    }
}

fn derive_identity(
    label: &[u8],
    parts: &[&[u8]],
) -> Result<differential_execution_core::Identity, String> {
    differential_execution_core::Identity::derive(DIFF_DOMAIN, parts)
        .map_err(|error| format!("derive {label:?}: {error:?}"))
}

/// Admits one differential case from the corpus label.
const MAX_CASE_LABEL_BYTES: usize = 128;

fn admit_case(label: &str) -> Result<differential_execution_core::ExecutionCase, String> {
    assert!(!label.is_empty(), "case labels must be non-empty");
    assert!(
        label.len() <= MAX_CASE_LABEL_BYTES,
        "case labels must stay within the bounded identity budget"
    );
    let label_bytes = label.as_bytes();
    let draft = differential_execution_core::CaseDraft {
        program: Some(derive_identity(b"program", &[label_bytes])?),
        arguments: Some(derive_identity(b"arguments", &[b"none"])?),
        initial_state: Some(derive_identity(b"initial-state", &[b"empty"])?),
        effects: Some(derive_identity(b"effects", &[DIFF_EFFECTS])?),
        backend_cohort: Some(derive_identity(b"backend-cohort", &[DIFF_BACKEND_COHORT])?),
        runtime_cohort: Some(derive_identity(b"runtime-cohort", &[DIFF_RUNTIME_COHORT])?),
        resources: Some(derive_identity(b"resources", &[DIFF_RESOURCES])?),
        adapter_contract: Some(derive_identity(b"adapter-contract", &[DIFF_ADAPTER])?),
        normalization: Some(derive_identity(b"normalization", &[DIFF_NORMALIZATION])?),
    };
    differential_execution_core::admit_case(draft).map_err(|error| format!("admit case: {error:?}"))
}

/// Observation bounds for the rail; outcomes are one class token each.
const OUTCOME_MAX_EVENTS: u32 = 64;
const OUTCOME_MAX_VALUE_BYTES: u32 = 65_536;
const OUTCOME_MAX_SET_METADATA: u32 = 16;

/// Admits one strategy's normalized outcome for an already-admitted case.
fn admit_observation(
    case: &differential_execution_core::ExecutionCase,
    strategy: crate::Strategy,
    outcome: &crate::NormalizedOutcome,
) -> Result<differential_execution_core::AdmittedObservation, String> {
    let class_name = outcome.class_name();
    assert!(
        !class_name.is_empty(),
        "outcome classes must be non-empty for admission"
    );
    assert_eq!(
        case.identity().to_hex().len(),
        IDENTITY_HEX_LENGTH,
        "admitted cases must carry full-length identities"
    );
    let value_bytes = class_name.as_bytes().to_vec();
    let profiles = differential_execution_core::ObservationProfiles::from(case.profiles());
    let raw = differential_execution_core::NormalizedObservation {
        case: case.identity(),
        profiles,
        backend_implementation: derive_identity(
            b"backend",
            &[strategy.implementation_name().as_bytes()],
        )?,
        adapter_implementation: derive_identity(b"adapter", &[DIFF_ADAPTER])?,
        value: differential_execution_core::CanonicalValue {
            schema: derive_identity(b"value-schema", &[DIFF_VALUE_SCHEMA])?,
            bytes: value_bytes.clone(),
        },
        events: Vec::new(),
        trap: match outcome {
            crate::NormalizedOutcome::Trapped(_) => {
                differential_execution_core::OptionalOutcome::Present(derive_identity(
                    b"trap",
                    &[class_name.as_bytes()],
                )?)
            }
            _ => differential_execution_core::OptionalOutcome::Absent,
        },
        interrupt: differential_execution_core::OptionalOutcome::Absent,
        resource: differential_execution_core::ResourceOutcome {
            class: derive_identity(
                b"resource-class",
                &[match outcome {
                    crate::NormalizedOutcome::FuelExhausted => b"fuel".as_slice(),
                    _ => b"none".as_slice(),
                }],
            )?,
            detail: derive_identity(b"resource-detail", &[DIFF_RESOURCES])?,
        },
        final_state: derive_identity(b"final-state", &[&value_bytes])?,
        set_metadata: Vec::new(),
    };
    let bounds = differential_execution_core::ObservationBounds::new(
        OUTCOME_MAX_EVENTS,
        OUTCOME_MAX_VALUE_BYTES,
        OUTCOME_MAX_SET_METADATA,
    )
    .map_err(|error| format!("observation bounds rejected: {error:?}"))?;
    differential_execution_core::admit_observation(case, raw, bounds)
        .map_err(|error| format!("admit observation: {error:?}"))
}

/// One reviewed harness verdict for a strategy-parity case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessVerdict {
    /// Lowercase harness case identity.
    pub case_identity_hex: String,
    /// Pairwise agreement or divergence from the reviewed core.
    pub verdict: differential_execution_core::ComparisonVerdict,
    /// First bounded mismatch surface, when divergent.
    pub mismatch: Option<differential_execution_core::MismatchPath>,
}

fn compare_outcomes(
    case: &differential_execution_core::ExecutionCase,
    left: &differential_execution_core::AdmittedObservation,
    right: &differential_execution_core::AdmittedObservation,
) -> HarnessVerdict {
    differential_execution_core::admit_independent_backends(Some(left), Some(right))
        .expect("rail admits two distinct strategies");
    let pairwise_verdict = differential_execution_core::compare_pairwise(left, right);
    assert_eq!(
        case.identity().to_hex().len(),
        IDENTITY_HEX_LENGTH,
        "admitted cases must carry full-length identities"
    );
    HarnessVerdict {
        case_identity_hex: case.identity().to_hex(),
        verdict: pairwise_verdict.verdict,
        mismatch: pairwise_verdict.mismatch,
    }
}

/// Runs one corpus case under both strategies and returns the reviewed
/// pairwise verdict over their admitted observations.
///
/// # Errors
/// Fails when execution or harness admission rejects the case.
pub fn run_differential(case: &CorpusCase) -> Result<HarnessVerdict, String> {
    let resource_caps = crate::admit_limits(crate::LimitsDraft {
        fuel: RAIL_FUEL_BUDGET,
        maximum_memory_pages: RAIL_MEMORY_PAGE_LIMIT,
    })
    .map_err(|error| format!("limits: {error:?}"))?;
    let cranelift_outcome = execute(crate::Strategy::Cranelift, resource_caps, case.wat)?;
    let winch_outcome = execute(crate::Strategy::Winch, resource_caps, case.wat)?;

    assert_eq!(
        cranelift_outcome.class_name(),
        case.expected.class_name(),
        "{}: cranelift outcome drifted from the pinned expectation",
        case.label
    );
    assert_eq!(
        winch_outcome.class_name(),
        case.expected.class_name(),
        "{}: winch outcome drifted from the pinned expectation",
        case.label
    );

    let admitted_case = admit_case(case.label)?;
    let left = admit_observation(
        &admitted_case,
        crate::Strategy::Cranelift,
        &cranelift_outcome,
    )?;
    let right = admit_observation(&admitted_case, crate::Strategy::Winch, &winch_outcome)?;
    Ok(compare_outcomes(&admitted_case, &left, &right))
}

#[cfg(test)]
mod tests;
