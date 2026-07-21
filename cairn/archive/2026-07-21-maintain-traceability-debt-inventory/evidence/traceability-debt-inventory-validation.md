# Traceability debt inventory validation

## Scope and non-claims

This evidence covers a deterministic inventory of the exact missing-ID array emitted by Lattice's maintained Cairn traceability profile. It groups every missing identifier by namespace subsystem, preserves explicit review uncertainty, binds reviewed candidate files by BLAKE3, and proves that inventory work did not change existing debt linkage.

Inventory rows do not satisfy requirements or create Tracey evidence. `unassessed` does not mean behavior or evidence is absent. A valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness. The inventory binds the supplied Cairn coverage hashes but does not independently reimplement Cairn's coverage-receipt verifier.

## Portfolio audit

Four mechanisms were considered:

1. Adding markers during inventory was rejected because it changes the measured debt and can manufacture coverage.
2. Lexical inference from names or tokens was rejected because similarity does not establish scenario assertions.
3. Defaulting every unreviewed row to absent was rejected because a bounded search cannot prove behavioral absence.
4. An explicit typed review registry plus an `unassessed` default was selected because it is mechanically complete while preserving uncertainty.

A secondary VibeThinker adversarial review identified that path-only reviewed evidence could drift. The accepted fix adds current file-byte BLAKE3 identities, rejects missing/symlinked/oversized evidence observations, and tests evidence-byte drift. Its suggestion to infer failures for all 361 unassessed rows was rejected because that would contradict the non-claim boundary.

## Baseline

Before adding the inventory requirement and implementation, `cairn traceability coverage --root .` exited unsuccessfully with:

- accepted requirements: 698
- referenced requirements: 331
- missing identifiers: 367
- dangling identifiers: 0
- `valid: false`

The ordered missing array is retained in `target/traceability-inventory-baseline.json` during validation.

## Maintained inventory

Human-authored review policy:

- `cairn-policy/traceability-inventory-contracts.ncl`
- `cairn-policy/traceability-inventory.ncl`

Generated report:

- `cairn-policy/generated/traceability-debt-inventory.json`

Final report facts:

- total rows: 367
- reviewed exact maintained-evidence candidates: 6
- unassessed rows: 361
- missing-set BLAKE3: `4b62e23c7c961ef56cff234e25dd31ab1f0ecfc1fb4278537a981f930e48471d`
- review-policy BLAKE3: `8f9ac601cb7a5922b2911d588a9094b290817df5247d3f73f6afaad7db199921`
- inventory BLAKE3: `816ee64ede6615978f39b389583e93dd0e4967f00f8007eb42a02270e415f875`
- issues: none
- inventory `valid: true`

Subsystem partition:

| Subsystem | Missing rows |
|---|---:|
| agent-orchestration | 39 |
| agent-runtime | 14 |
| credentials | 26 |
| node-templates | 3 |
| operator-diagnostics | 8 |
| repository-validation | 11 |
| scheduling | 7 |
| tui-ux | 98 |
| workflow-examples | 2 |
| workflow-execution | 89 |
| workflow-format | 41 |
| workflow-validation | 29 |

## Reviewed candidate boundary

The review policy records exactly these still-missing identifiers without adding their markers:

- `lattice.workflow_execution.run_store_recovery.scenario.non-claim`
- `lattice.workflow_execution.safe_run_inspection.scenario.missing`
- `lattice.workflow_execution.safe_run_inspection.scenario.not_recorded`
- `lattice.workflow_execution.safe_run_inspection.scenario.owner_binding`
- `lattice.workflow_execution.safe_run_inspection.scenario.replay`
- `lattice.workflow_execution.safe_run_inspection.scenario.safe_output`

Bound maintained files:

- `tests/test_run_inspection.rs`: `f726fbd68d1a478dff3da62b65d72604afb1b533452c9191272367876bb09484`
- `tests/test_run_store_recovery.rs`: `ccd64b7cdc5da4346ced7d937413ec4e75b3f71e4fc33798ba36bd6d16b7c549`

These hashes establish current file identity only. The reviewed classification records that exact assertions were observed; it is still not a Tracey linkage or whole-file correctness proof.

## Final coverage invariant

After synchronizing the new inventory requirement and adding markers only for that new requirement, the maintained coverage command still exits unsuccessfully with:

- accepted requirements: 702
- referenced requirements: 335
- missing identifiers: 367
- dangling identifiers: 0
- `valid: false`
- coverage policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- coverage input BLAKE3: `9da7f2ac27fe944759a13e44cc37c98bcbc5a8f2a5fb16cb6a0666487d1ab598`
- coverage receipt BLAKE3: `3c5d763c6bb439a512735f73ed3421484abc894c6805cff35876ac295abebe8f`

A Steel comparison returned `missing_arrays_equal: true`, `dangling_arrays_equal: true`, `final_dangling: 0`, and `final_valid: false`. Requirements and referenced counts each increased by four solely for the new fully linked inventory requirement and its three scenarios.

## Positive and negative validation

- `test_traceability_inventory`: 6 passed.
- Positive checks cover deterministic repeated generation, sorted complete rows, subsystem/classification summaries, current checked-report self-verification, reviewed evidence identities, and explicit unassessed non-claims.
- Negative checks cover duplicate missing IDs, stale review IDs, unsafe paths, missing evidence observations, changed evidence bytes, changed missing-set identity, changed coverage provenance, tampered rows, summary drift, and inventory-hash drift.
- The valid Nickel policy exports successfully.
- `invalid-traceability-inventory.ncl` fails export for a parent-traversing evidence path.
- `scripts/traceability-inventory.rs generate` and `check` agree byte-for-byte.
- `scripts/check.rs` passes script formatting/smoke, package formatting, the complete test suite, strict project-only Clippy, and Cairn spec checks.
- Cairn policy freshness, proposal/design/tasks gates, synchronization, and repository validation pass.
- `nix flake check -L --option secret-key-files ''` passes all configured `x86_64-linux` checks, including the positive/negative Nickel policy derivation; incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems are omitted.
- Octet retains 913 accepted/current findings with zero new and zero stale IDs under baseline `b3:ec5609b38db008f4145f4c38490b04290d8b52266e7b5a067d2cb29751ac208f`. The underlying Cargo subprocess remains classified as exit code 101 while the reviewed zero-delta wrapper accepts it.
