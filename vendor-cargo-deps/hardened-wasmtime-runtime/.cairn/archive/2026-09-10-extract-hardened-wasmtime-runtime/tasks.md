## Phase 1: Source and profile baseline

- [x] [serial] r[onix.wasmtime_runtime.source_inventory] Record Aspen source modules, Wasmtime revision, feature settings, limits, worlds, and fixtures.
- [x] [serial] r[onix.wasmtime_runtime.fixture_compatibility] Freeze compile, invocation, limit, and trap fixtures before extraction.
- [x] [serial] r[onix.wasmtime_runtime.source_revision] Create the core and shell packages with immutable Cargo and Nix source identities.

## Phase 2: Runtime profile core

- [x] [serial] r[onix.wasmtime_runtime.feature_policy] Implement complete explicit feature-policy admission.
- [x] [serial] r[onix.wasmtime_runtime.artifact_shape] Implement exact artifact, world, import, and export checks.
- [x] [serial] r[onix.wasmtime_runtime.resource_limits] Implement fuel, stack, memory, table, instance, input, and output limit admission.
- [x] [serial] r[onix.wasmtime_runtime.epoch_deadline] Implement injected epoch-deadline plans and observations.
- [x] [serial] r[onix.wasmtime_runtime.restricted_imports] Implement deny-by-default exact import descriptors.
- [x] [serial] r[onix.wasmtime_runtime.normalized_outcomes] Implement stable normalized failure and success classes.
- [x] [serial] r[onix.wasmtime_runtime.functional_core] Keep Wasmtime, clocks, linking, artifacts, and host effects in the shell and adapters.

## Phase 3: Shell and boundary tests

- [x] [serial] r[onix.wasmtime_runtime.resource_limits.output] Add bounded component and core-Wasm input and output adapters.
- [x] [parallel] r[onix.wasmtime_runtime.fixture_compatibility.negative] Add unknown-feature, import, pointer, overflow, fuel, deadline, memory, output, and trap fixtures.
- [x] [parallel] r[onix.wasmtime_runtime.restricted_imports.capability] Add tests that product imports require exact operation and capability bindings.

## Phase 4: Pilots and release

- [x] [serial] r[onix.wasmtime_runtime.consumer_pilots] Cut Aspen over to the extraction and add one independent product adapter.
  - Pilot A (source product): `molten-wasm-pilot` in the Aspen repository (`pilot/hwr-consumer-20260909`), pinned to this Radicle revision, admits the Molten profile and runs a bounded guest.
  - Pilot B (second consumer): `hwr-consumer-pilot` in the Site repository (`pilots/hwr-consumer`), Site-owned world `site:pilot@1.0.0`, no Aspen types, pinned to the same revision.
- [x] [serial] r[onix.wasmtime_runtime.source_revision] Run repository checks, consumer pilots, Cairn validation, and all change gates.
