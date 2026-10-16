# Operation Packs Wasm Runtime Specification

## ADDED Requirements

### Requirement: Pack manifests declare a closed component runtime

r[lattice.operation_packs_wasm.manifest] Lattice MUST add a closed `wasm-component` runtime descriptor that declares the component content role, WIT package, world, method mapping, Mantle bundle role, shared-runtime profile, resource profile, result contract, effect-intent contract, and non-claims in typed Nickel and Rust.

#### Scenario: Complete component descriptor is admitted

- GIVEN a pack declares every component runtime field with exact content roles and supported versions
- WHEN manifest validation runs
- THEN Lattice MAY return deterministic component metadata for lock planning.

#### Scenario: Runtime descriptor is ambiguous

- GIVEN a descriptor omits a required field, duplicates a method, names cross-pack content, or includes an unknown runtime option
- WHEN manifest validation runs
- THEN Lattice MUST reject before lock creation, component loading, or operation dispatch.

### Requirement: Pack locks bind the complete component cohort

r[lattice.operation_packs_wasm.lock] A pack lock MUST bind the component, WIT, world, methods, Mantle bundle, bundle-verifier revision, shared-runtime revision and profile, resources, capabilities, evidence, platform, and pack identities, and any change to those authoritative facts MUST require a new lock.

#### Scenario: Locked component facts match

- GIVEN measured pack and bundle facts match every component lock entry
- WHEN lock admission runs
- THEN Lattice MAY continue to runtime admission.

#### Scenario: Runtime or bundle drifts

- GIVEN the component, world, bundle, verifier, runtime, profile, resource, capability, evidence, or platform identity differs
- WHEN lock admission runs
- THEN Lattice MUST deny and MUST NOT rewrite or weaken the lock.

### Requirement: Component runtime decisions stay pure

r[lattice.operation_packs_wasm.functional_core] Manifest, lock, bundle-observation, world, method, runtime, resource, operation, output, effect-intent, lifecycle, fallback, and receipt decisions MUST be pure deterministic logic over supplied values.

#### Scenario: Equal component facts are planned twice

- GIVEN equal admitted in-memory facts
- WHEN dispatch planning runs twice
- THEN both plans or blocker sets MUST be equal
- AND the core MUST perform no file, process, Wasmtime, network, clock, credential, persistence, or output effect.

### Requirement: Component execution uses a thin shared-runtime shell

r[lattice.operation_packs_wasm.shell] The non-default shell MUST remeasure the admitted Mantle bundle through the published verifier, invoke the exact shared hardened runtime profile under declared bounds, and own deadlines, cancellation, teardown, and normalized observations without building components, contacting registries, or performing operation effects.

#### Scenario: Exact local runtime closure is available

- GIVEN the admitted pack, lock, bundle root, verifier, and runtime closure are available offline
- WHEN the shell executes a component method
- THEN it MAY return one bounded normalized observation to Lattice admission.

#### Scenario: Shell discovers an ambient dependency

- GIVEN the exact verifier, runtime, bundle, or component is absent
- WHEN shell preflight runs
- THEN it MUST fail without searching `PATH`, registries, mirrors, or sibling worktrees.

### Requirement: First pack world has no guest imports

r[lattice.operation_packs_wasm.world] The first operation-pack component world MUST expose no WASI or host imports and MUST return only a bounded method result or typed effect-intent set under the declared WIT and schema identities.

#### Scenario: Import-free method returns valid output

- GIVEN a component implements the exact import-free world and returns schema-valid bounded output
- WHEN output admission runs
- THEN Lattice MAY continue to operation result or effect-intent handling.

#### Scenario: Component requests a host import

- GIVEN a component imports filesystem, network, process, environment, clock, random, credential, device, WASI, or product host functions
- WHEN world or linker admission runs
- THEN Lattice MUST reject before execution.

### Requirement: Effect intents retain Lattice effect authority

r[lattice.operation_packs_wasm.effects] A component-returned effect intent MUST remain an untrusted proposal and MUST pass the existing operation planner, authority adapter, credential policy, durable attempt, and effect shell before any external effect starts.

#### Scenario: Valid effect intent is proposed

- GIVEN a component returns an in-scope schema-valid intent
- WHEN result admission runs
- THEN Lattice MAY submit it to existing operation admission
- AND component success MUST NOT complete the external effect.

#### Scenario: Intent requests undeclared authority

- GIVEN an intent exceeds the pack method, capability, credential, root, or operation contract
- WHEN result admission runs
- THEN Lattice MUST reject before external effects.

### Requirement: Component dispatch preserves durable lifecycle

r[lattice.operation_packs_wasm.lifecycle] Lattice MUST persist `Started` before component dispatch and MUST preserve distinct success, denial, trap, fuel, deadline, cancellation, invalid-output, resource, cleanup, and unknown outcomes without automatic retry.

#### Scenario: Component result is admitted

- GIVEN a durable started attempt and a valid terminal component result
- WHEN terminal handling runs
- THEN Lattice MAY record the admitted method outcome under the exact attempt.

#### Scenario: Dispatch outcome is unknown

- GIVEN component dispatch ends without a complete trusted terminal observation
- WHEN recovery runs
- THEN the attempt MUST remain unknown and MUST NOT run again automatically.

### Requirement: Component runtime never falls back

r[lattice.operation_packs_wasm.fallback] A component runtime failure MUST NOT retry through an executable, shell node, interpreter, shared library, or another runtime, and another route MUST require a new explicit plan and attempt.

#### Scenario: Component runtime is unavailable

- GIVEN the locked shared runtime is unavailable
- WHEN dispatch admission runs
- THEN Lattice MUST fail with a runtime-specific blocker and MUST NOT launch the executable runtime.

### Requirement: Component pack conformance has positive and negative evidence

r[lattice.operation_packs_wasm.verification] The focused rail MUST cover offline exact execution, result admission, effect-intent planning, durable lifecycle, cleanup, and deterministic decisions plus negative manifest, lock, bundle, verifier, runtime, world, import, resource, timeout, cancellation, trap, output, authority, unknown, fallback, receipt-leak, and overclaim cases.

#### Scenario: Component pack is proposed for support

- GIVEN implementation, fixtures, dependency pins, docs, and non-claims are complete
- WHEN focused pack, workspace, Clippy, Octet, Cairn, and Nix validation runs
- THEN every positive and negative case MUST produce its declared result.

### Requirement: Component pack evidence preserves non-claims

r[lattice.operation_packs_wasm.nonclaims] Passing component-pack evidence MUST NOT claim pack trust, component correctness, operation correctness, sandbox completeness, external effect success, production readiness, or release eligibility.

#### Scenario: Passing method is promoted to operation correctness

- GIVEN passing component evidence is labeled as proof of external operation correctness
- WHEN non-claim validation runs
- THEN the evidence MUST fail.
