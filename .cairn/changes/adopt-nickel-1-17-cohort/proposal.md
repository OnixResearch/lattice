# Proposal: Adopt the Nickel 1.17 workflow evaluator cohort

## Why

Lattice uses a reviewed `nickel-eval` object from the Nickel `1.16` cohort. It also pins `nickel-lang-core 0.17.0` for development fixtures.

Workflow import must use the same parser, contract, and diagnostic cohort in product and fixture paths.

## What Changes

- Pin the new reviewed `nickel-eval` object for Nickel `1.17.0`.
- Update the direct fixture dependency to `nickel-lang-core 0.18.0`.
- Pin the command-line evaluator to Nickel `1.17.0` at exact upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`.
- Re-run workflow import, Choregraph lowering, diagnostics, examples, and operator checks.
- Add negative fixtures for malformed workflows, import failures, contract failures, and evaluator-cohort substitution.

## Impact

Lattice workflow authoring and import will use one reviewed Nickel cohort. Lattice retains workflow decoding, effects, persistence, retries, diagnostics, and evidence.

## Dependencies

Implementation depends on the archived `nickel-eval` change `adopt-nickel-1-17-cohort` and its immutable Radicle object.

## Non-goals

- Do not move workflow meaning into `nickel-eval`.
- Do not change Choregraph graph semantics.
- Do not add source fallbacks.
- Do not update unrelated dependencies only to obtain Nickel.
