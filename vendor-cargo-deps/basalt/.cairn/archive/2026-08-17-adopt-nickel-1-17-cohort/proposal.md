# Proposal: Adopt Nickel 1.17 for policy authoring

## Why

Basalt policy checks currently resolve to Nickel `1.16.0`. The stack cohort is Nickel `1.17.0`.

Policy export and contract fixtures must use the same reviewed evaluator as downstream stack consumers.

## What Changes

- Pin Nickel `1.17.0` at upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`.
- Use the pin for policy export, freshness checks, fixtures, and developer commands.
- Add positive and negative compatibility fixtures for contracts, merges, exports, and diagnostics.
- Record the exact evaluator identity in policy release evidence.

## Impact

Basalt policy artifacts will bind the stack Nickel cohort. Basalt retains policy shape, authority decisions, and release authority.

## Dependencies

This change uses the upstream Nickel `1.17.0` release.

## Non-goals

- Do not change Basalt policy meaning or UCAN authority semantics.
- Do not update unrelated dependencies only to obtain Nickel.
- Do not claim policy correctness from evaluator alignment.
