# Design: Add a Wasm component operation-pack runtime

## Context

Operation packs already bind exact content, locks, evidence, capabilities, runtimes, and product effects. The new runtime class must fit that model. It must not make component validity, WIT compatibility, or Mantle evidence authoritative for an operation.

## Success Contract

An admitted local pack can run one import-free component method offline through the shared runtime. Lattice persists the attempt before dispatch, validates the result or effect intents, and preserves every existing authority and recovery rule.

## Decisions

### Extend the typed runtime vocabulary

The Nickel manifest and Rust DTOs gain a closed `wasm-component` runtime kind. It declares one component content role, WIT package, world, method map, Mantle bundle role, shared-runtime profile, resource profile, allowed result class, effect-intent schema, and non-claims.

Missing, unknown, duplicate, cross-pack, or undeclared fields fail before lock creation.

### Bind the complete component path in the lock

The pack lock binds component and WIT identities, bundle identity, bundle verifier revision, shared runtime revision and profile, method mapping, resource profile, capability profile, evidence refs, and target platform.

Display paths, registry locations, and channels remain locator metadata. A changed runtime or bundle requires a new lock.

### Keep admission pure

The pure core validates manifest, lock, pack bytes, bundle observations, world and method facts, runtime compatibility, resources, operation authority inputs, and output or effect-intent contracts. It returns one dispatch plan or ordered blockers.

The core performs no file, process, Wasmtime, clock, network, credential, persistence, or output effect.

### Use a thin shared-runtime shell

The shell remeasures the admitted bundle through the Mantle verifier, invokes the shared hardened runtime, applies deadlines and cancellation, and records normalized observations. It provides no WASI or guest imports in the first profile.

The shell cannot build components, select another runtime, resolve registries, or perform operation effects.

### Return results or effect intents

A component may return a pure method result or a bounded typed effect-intent set. Lattice revalidates every intent through the existing operation planner, authority adapter, credential policy, durable attempt, and effect shell.

Component execution never means that an external effect ran. Guest output cannot mark an attempt complete.

### Preserve durable lifecycle and uncertainty

Lattice persists `Started` before component dispatch. It records success, deterministic denial, trap, fuel exhaustion, deadline, cancellation, invalid output, resource failure, cleanup failure, and unknown outcomes separately.

Unknown outcomes do not retry automatically. A component failure cannot fall back to an executable or shell node.

### Keep offline operation

An exact pack directory, lock, component bundle, verifier, and runtime closure are sufficient for ordinary execution. Registry access and update checks remain absent.

## Functional Core and Imperative Shell

- **Core**: manifest, lock, bundle facts, method admission, operation binding, output and effect-intent validation, lifecycle classification, and receipts.
- **Shell**: capability-root reads, bundle remeasurement, component execution, cancellation, cleanup, durable persistence, and approved product effects after separate admission.
- **Shared components**: Mantle verifies materialized bytes. The hardened runtime executes bounded components. Neither owns Lattice semantics.

## Risks and Controls

- Pack authors can hide effects in output. Closed result and effect-intent schemas reject unknown forms.
- Runtime drift can invalidate locks. Exact revision and profile identities are lock inputs.
- Parallel runtime classes can diverge. Shared operation lifecycle conformance runs across executable and component paths.

## Non-Claims

Passing evidence does not prove pack trust, component correctness, operation correctness, sandbox completeness, external effect success, production readiness, or release eligibility.
