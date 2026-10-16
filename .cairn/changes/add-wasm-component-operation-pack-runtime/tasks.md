# Tasks

## Phase 1: Baseline and dependency admission

- [ ] [serial] Record operation-pack manifest, lock, runtime, durable-attempt, output, recovery, and negative-fixture baselines before core changes. r[lattice.operation_packs_wasm.verification]
- [ ] [serial] Pin completed immutable shared-runtime and Mantle-verifier revisions and record the matching bundle contract without sibling paths. r[lattice.operation_packs_wasm.lock] r[lattice.operation_packs_wasm.shell]
- [ ] [serial] Add typed Nickel and Rust `wasm-component` runtime descriptors with exact content, WIT, bundle, runtime, resource, result, effect-intent, and non-claim fields. r[lattice.operation_packs_wasm.manifest]
- [ ] [serial] Extend pack and lock identity projections with complete component cohort facts and deterministic drift blockers. r[lattice.operation_packs_wasm.lock]

## Phase 2: Core and shell

- [ ] [serial] Implement pure component manifest, lock, bundle-observation, method, runtime, resource, operation, output, effect-intent, lifecycle, fallback, and receipt admission. r[lattice.operation_packs_wasm.functional_core]
- [ ] [serial] Define the import-free first WIT world and closed result and effect-intent schemas. r[lattice.operation_packs_wasm.world]
- [ ] [serial] Add the non-default shell over explicit pack roots, the Mantle verifier, and shared runtime with bounded execution, cancellation, teardown, and no build or registry fallback. r[lattice.operation_packs_wasm.shell]
- [ ] [serial] Route effect intents through existing operation, authority, credential, durable-attempt, and effect owners. r[lattice.operation_packs_wasm.effects]
- [ ] [serial] Preserve `Started`-before-dispatch, normalized terminal outcomes, unknown recovery, and no automatic retry. r[lattice.operation_packs_wasm.lifecycle]
- [ ] [serial] Deny executable, shell-node, interpreter, shared-library, and alternate-runtime fallback. r[lattice.operation_packs_wasm.fallback]

## Phase 3: Fixtures and product evidence

- [ ] [parallel] Add an offline positive component pack with pure result and effect-intent methods, exact lock, durable attempt, cleanup, and safe inspection. r[lattice.operation_packs_wasm.verification]
- [ ] [parallel] Add negative manifest, lock, bundle, verifier, runtime, world, import, resource, timeout, cancellation, trap, output, authority, unknown, fallback, receipt-leak, and overclaim fixtures. r[lattice.operation_packs_wasm.verification] r[lattice.operation_packs_wasm.nonclaims]
- [ ] [parallel] Add shared lifecycle conformance that compares executable and component runtime attempt semantics without requiring result equivalence. r[lattice.operation_packs_wasm.lifecycle] r[lattice.operation_packs_wasm.fallback]
- [ ] [serial] Update operation-pack docs, CLI inspection, doctor output, offline workflow, recovery guidance, and non-claims. r[lattice.operation_packs_wasm.manifest] r[lattice.operation_packs_wasm.nonclaims]
- [ ] [serial] Run focused pack tests before and after changes, workspace tests, Clippy with warnings denied, Octet, Cairn validation and gates, and relevant Nix checks. r[lattice.operation_packs_wasm.verification]
