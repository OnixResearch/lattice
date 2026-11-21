# Bound same-attempt repair and consume Kiln's budget lineage

Tracking: [Lattice #6](https://github.com/OnixResearch/lattice/issues/6), section "Canonical deterministic workflow around bounded agent steps": the same-attempt repair, bypass, disposition and usage criteria, and the [Kiln #61](https://github.com/OnixResearch/kiln/issues/61) lineage integration listed as open in the 2026-11-21 status comment. Producer contract (read-only): Kiln [`339116f6`](https://github.com/OnixResearch/kiln/tree/339116f68e762e4caf0d05a9ac73a6e97a74b67d/crates/kiln-core/src/fair_admission) `fair_admission::{lineage, serving}`. Owner-approved on 2026-11-21.

## Why

An admitted implementation attempt runs exactly one bounded worker step. Issue #6 requires a deterministic workflow around bounded agent steps: when a required check fails with an admitted failure class, the same attempt may repair its candidate a bounded number of times, but only under an admitted policy, with every candidate recorded and rechecked, and with no path for exit status, worker claims, skipped or reconfigured checks or self-approval to pass a candidate. Correct, repaired, exhausted, cancelled and uncertain outcomes must stay distinct, and changed work or a new semantic attempt stays with Kiln.

Kiln #61 now admits one recursive budget lineage per root and blocks requests on revocation, deadlines, parent cancellation and serving backpressure. Lattice receives none of it: an attempt cannot refuse a spawn its lineage cannot cover, and every step would look like a fresh budget.

## What Changes

- An optional `repair` field (`lattice.implementation-attempt-repair-policy.v1`) in the v3 request names admitted failure classes, repair iterations, an elapsed bound, aggregate model and tool budgets, the admitted check executor and the required checks with their configuration identities. It is validated against the envelope budget and broker ceiling and framed into the profile revision when present. r[lattice.implementation_attempt.repair] r[lattice.implementation_attempt.repair.scenario.policy]
- A pure loop (`implementation_attempt::repair::state`) decides every spawn, charges each step and check conservatively and yields one of: `correct_candidate`, `repaired_candidate`, `budget_exhausted`, `cancelled`, `uncertain`, `not_repairable`, `checks_rejected`, `lineage_blocked`, `kiln_owned`, `fence_superseded` or `authority_withdrawn`. A pass is issued only while the clock is readable, the elapsed bound is not reached and the charged totals fit every budget and the lineage remainder. r[lattice.implementation_attempt.repair.scenario.dispositions]
- A pure check classifier (`implementation_attempt::repair::checks`) passes a candidate only on complete evidence from the admitted executor, with the admitted configuration, against the exact host-derived revision, each check run against its own read-only copy of it. Skipped, missing, duplicated, undeclared, reconfigured, crossed, self-approved and drifted observations reject the evidence. r[lattice.implementation_attempt.repair.scenario.bypass]
- Each step binds the frozen work, version, task, attempt, fence and envelope; another binding is refused as `changed_work` or `new_semantic_attempt`, and a worker's proposal of either ends the loop as `kiln_owned`. r[lattice.implementation_attempt.repair.scenario.kiln_owned]
- The shell (`implementation_attempt::cli::repair`) runs the loop through embedder-supplied `WorkerPort`, `CheckPort` and clock, captures every candidate from the workspace, retains write-once step intents, step records and the disposition, persists the existing `Scope::Attempt` usage observation and recovers only from the retained disposition. The standalone CLI supplies no ports and refuses a repair request as `repair_ports_unavailable`. Without a policy, requests decode, frame and run exactly as before. r[lattice.implementation_attempt.repair.scenario.absent]
- An optional `lineage` field (`lattice.kiln-budget-lineage.v1`) maps Kiln's lineage explanation, account remainder and serving gate. A blocker, backpressure or a remainder that cannot cover the first step is refused before any write (`lineage_blocked`, `lineage_exhausted`); every later step draws from the same account; a child is admitted only as a subtractive part of the remainder. r[lattice.implementation_attempt.lineage] r[lattice.implementation_attempt.lineage.scenario.exhausted] r[lattice.implementation_attempt.lineage.scenario.shared]
- Preparation, edit, check and repair usage is attributed per phase with the existing `UsageProvenance`; unknown counts stay unknown and are charged at the reservation. r[lattice.implementation_attempt.lineage.scenario.usage]

## Impact

- **Pure core:** new `src/implementation_attempt/lineage.rs` and `lineage/wire.rs`, `src/implementation_attempt/repair.rs` and `repair/{checks,revision,state,step}.rs`; two module lines in `src/implementation_attempt.rs`.
- **Shell:** new `src/implementation_attempt/cli/repair/{mod,records,steps,materialized}.rs`; `cli.rs` (two optional request fields, `Ports::repair`), `cli/execution.rs` (gate, directory creation after the pre-write gates, branch, recovery), `cli/recovery.rs` (single-shot lineage draw), `cli/refusal.rs` (three reasons), `cli/session/profile.rs` (revision framing), `cli/binding/mod.rs` and `cli/session/revision.rs` (non-writing `recheck` halves shared by `revalidate` and `permit`).
- **Tests:** `src/implementation_attempt/lineage/tests.rs`, `src/implementation_attempt/repair/tests.rs`, `src/implementation_attempt/cli/tests/repair.rs`; struct literals in existing tests gain the two absent fields.
- **Docs:** `docs/candidate-worker.md`, `docs/worker-inventory.md` (wait-reason note), changelog.
- **Compatibility:** additive. A request without `repair` and `lineage` keeps its bytes, identity and profile revision. `Ports` gains a field, so embedders constructing it add `repair: None`.

## Dependencies

- Local: `execute-admitted-implementation-attempt` (the attempt shell, intent, fencing and refusals), `manage-agent-revisions` (profile revision framing and the session seam).
- Producer: Kiln #61 at `339116f6` (mapped, not vendored). Kiln #46 keeps independent acceptance.

## Out of Scope

- An OMP-backed `WorkerPort`; the standalone CLI and the pinned OMP route stay single-shot.
- `export-candidate` and `inspect` projections of repaired candidates.
- Kiln-side emission of the lineage DTO, Cairn SDLC stages (#26) and dialog-style revalidation.
- Live model, check executor or lineage qualification.
