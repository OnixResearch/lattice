# Tasks

## Phase 1: Producer adoption and contract

- [x] [serial] 1.1 Re-vendor Animus at `1c370d5029a2667051c46bdfa1919e78e65e2f4b` with `scripts/sync_animus.rs`, adding `animus-frozen-policy` to the required crates, normalized workspace and Lattice path dependencies. r[lattice.typed_policy.invocation] Done: archive BLAKE3 `ff16f0903348db1d5164d055b928d2f1f829ca1dffb6f1624e6d26779b11f2ab`; the vendored tree was compared byte-for-byte (`diff -r`) with the independent serving-adoption clone. The producer revision is a local, unpublished candidate.
- [x] [serial] 1.2 Implement pure profile admission, binding and identity for `lattice.typed-policy-profile.v1`, refusing stateful, unbounded and malformed profiles. r[lattice.typed_policy.profile] r[lattice.typed_policy.stateless] Done in `src/workflow/decision/{admission,checks}.rs`.

## Phase 2: Invocation, validation and routing

- [x] [serial] 2.1 Implement projection, candidate set and invocation identities, independent receipt validation and typed outcomes. r[lattice.typed_policy.validation] Done in `src/workflow/decision/{binding,validation}.rs`.
- [x] [serial] 2.2 Implement fallback resolution with terminal cancellation. r[lattice.typed_policy.fallback]
- [x] [serial] 2.3 Add the Animus frozen-policy adapter shell and the executor Condition hook with deadline, cancellation, routing through existing branch ports and binding-aware cache keys. r[lattice.typed_policy.invocation] r[lattice.typed_policy.route] Done in `src/animus_agent/frozen.rs`, `src/engine/executor/choice.rs`, `src/engine/frontier/runtime.rs`.
- [x] [serial] 2.4 Persist bounded decision records in run details and replay them without adapter work. r[lattice.typed_policy.record] r[lattice.typed_policy.replay] Done for headless completed runs and for runs that suspend at and resume from durable approval gates: `persist_suspension` attaches decisions with no workflow outcome; `resume_with_typed_policy` revalidates stored decisions against the current profile before the claim (stale or unbound refuses with `PolicyDecisionStale`, nothing re-decided or rewritten), carries them with zero adapter calls and persists decisions made after the resume. `tests/test_typed_policy_resume.rs`.

## Phase 3: Fixtures, negative cases and measurement

- [x] [parallel] 3.1 Add positive, disabled-profile byte-identity, malformed/oversized/undeclared/missing-candidate/stale-binding/timeout/cancellation/unavailable fixtures. r[lattice.typed_policy.validation] r[lattice.typed_policy.fallback] `tests/test_typed_policy.rs`.
- [x] [parallel] 3.2 Add authority, frontier, exact-label protocol and uncertain-effect boundary fixtures. r[lattice.typed_policy.route]
- [x] [serial] 3.3 Run the pre-declared comparison and retain the actual result, including unavailable arms and a no-go. r[lattice.typed_policy.measurement] `evidence/measurement.json`; verdict `no-improvement-established`; the reasoning-model arm is unavailable.
- [x] [serial] 3.4 Document the profile, outcomes, replay, accounting boundary and non-claims. r[lattice.typed_policy.profile] r[lattice.typed_policy.measurement] `docs/typed-policy-operations.md`.
- [ ] [serial] 3.5 Run focused tests, Clippy, fmt, Cairn validation and gates, Octet semantic delta and the Nix `nextest` check. r[lattice.typed_policy.measurement] Local results are in `evidence.md`; the parent's integrated run (Octet re-key, full flake check) remains open.

## Evidence Boundary

Checked tasks are complete only at their stated scope. The fixture artifact is hand-authored; no task claims model quality, live reasoning-model comparison, production readiness or default adoption.
