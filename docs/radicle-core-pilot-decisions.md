# Radicle core pilot decisions

## Decision

Lattice rejects the proposed `change-admission-core` and `effect-attempt-core` production adoptions.

No reviewed producer extraction is available. The workspace catalog, workspace guides, sibling repositories, and public OnixResearch repository inventory contain no canonical producer entry for either core.

Neither pilot has the required license, provenance, canonical Radicle RID, immutable object, source endpoint, authority policy, or two-consumer simplification evidence. Lattice therefore adds no dependency, adapter, fallback, or lockfile entry.

## Change-admission mapping

| Lattice fact | Shared target | Lattice owner retained |
| --- | --- | --- |
| Proposal, base, candidate, epoch, and revisions | No admitted target | Orchestration core |
| Successor validation | No admitted target | Workflow validation |
| Actor and authority-policy decision | No admitted target | Lattice authority policy |
| Terminal disposition and mutation | No admitted target | Orchestration transitions |
| Events, replay, storage, and CLI output | Out of shared scope | Lattice shell and store |

The proposed producer cannot remove a branch or assertion because no reviewed API or behavior exists. Adapter size, callback count, and removed branch count are all zero.

## Effect-attempt mapping

| Lattice fact | Shared target | Lattice owner retained |
| --- | --- | --- |
| Attempt identity and parent | No admitted target | Effect authority core |
| Dispatch phase and observation | No admitted target | Scheduler and orchestration cores |
| Retry and abandonment acknowledgement | No admitted target | Product policy |
| Budget reservation and settlement | Out of shared scope | Lattice budget owner |
| Durable closure, redb state, and CLI output | Out of shared scope | Lattice shell and store |

No shared producer exists to compare against the scheduler and orchestration paths. An adapter would only copy existing product decisions into unowned DTOs. That result would not simplify Lattice.

## Adversarial audit

The rejection prevents these risks:

- preview presented as approval;
- actor or authority facts hidden behind a generic authorization flag;
- stale proposal or candidate substitution admitted;
- mutation success invented from a pure decision;
- missing or foreign effect observations converted to terminal outcomes;
- exactly-once claims inferred from local deduplication;
- retry, abandonment, cancellation, or budget policy moved outside Lattice;
- persisted schemas changed without a versioned migration;
- runtime, sibling-path, vendored, or GitHub fallback dependencies.

## Baseline evidence

Before the decision, these tests passed:

- `cargo test --test test_orchestration_core`: 24 passed;
- `cargo test --test test_scheduler_occurrences`: 15 passed;
- `cargo test --test test_orchestration_shell`: 13 parent tests passed, including expected rejected child-process phases.

The existing proposal, approval, stale-decision, replay, attempt, recovery, retry, abandonment, budget, and persistence behavior remains unchanged.

## Reconsideration condition

A later change can reconsider either pilot only after a producer publishes all required source and simplification evidence. That later change must rerun disposable parity adapters before adding a production dependency.

## Non-claims

This rejection does not prove that no useful shared core can ever exist. It proves only that the required producer evidence is absent from the reviewed publication surfaces.

Passing baseline tests does not prove external effect truth, exactly-once behavior, actor authority, persistence correctness, or release readiness.

<!--
r[impl lattice.change_admission_adoption.source]
r[impl lattice.change_admission_adoption.boundary]
r[impl lattice.change_admission_adoption.cutover]
r[verify lattice.change_admission_adoption.compatibility]
r[impl lattice.effect_attempt_adoption.source]
r[impl lattice.effect_attempt_adoption.boundary]
r[impl lattice.effect_attempt_adoption.cutover]
r[verify lattice.effect_attempt_adoption.compatibility]
-->
