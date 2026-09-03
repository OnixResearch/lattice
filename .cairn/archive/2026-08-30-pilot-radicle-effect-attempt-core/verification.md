# Verification: pilot-radicle-effect-attempt-core

r[verify lattice.effect_attempt_adoption.source]
r[verify lattice.effect_attempt_adoption.boundary]
r[verify lattice.effect_attempt_adoption.compatibility]
r[verify lattice.effect_attempt_adoption.cutover]

## Verdict

Rejected. No reviewed `effect-attempt-core` producer extraction or immutable Radicle publication exists in the workspace catalog, workspace guides, sibling repositories, or reviewed public organization inventory.

Required license, provenance, RID, object, endpoint, authority, and two-consumer simplification evidence is absent. Source admission therefore stops before dual-adapter or dependency work.

## Evidence

- `nix develop -c cargo test --test test_scheduler_occurrences`: 15 passed before the decision.
- `nix develop -c cargo test --test test_orchestration_shell`: 13 parent tests passed before the decision; expected rejected child-process phases remained fail-closed.
- Existing attempt, observation, recovery, retry, abandonment, authority, budget, persistence, and crash fixtures remain unchanged.
- `docs/radicle-core-pilot-decisions.md` records the mapping, measurements, audit, rollback, and non-claims.
- Cargo, Nix, `Cargo.lock`, and `flake.lock` have no producer dependency or fallback.
- `scripts/check.rs`: passed after refreshing the existing README evidence hash.
- `nix develop -c octet-precommit-check`: passed with no new or stale findings.
- `nix flake check --no-build` and the Nickel cohort Nix check passed.
- Cairn validation and proposal, design, and tasks gates passed.

## Non-claims

The source review is bounded to reviewed publication surfaces. It does not prove that no future shared core can exist.

The baseline tests do not prove external effect truth, exactly-once behavior, cancellation, budget truth, crash safety, persistence correctness, or release readiness.
