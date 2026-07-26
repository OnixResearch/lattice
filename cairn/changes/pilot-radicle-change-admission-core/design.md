## Context

`RevisionProposal` binds instance, definition revision, epoch, base workflow revision, successor, and summary. `ApprovalDecision` binds instance, epoch, base, proposal, authority-policy hash, actor, and decision. Lattice computes product-domain BLAKE3 identities and applies transitions through its orchestration event model.

A shared lifecycle must consume these facts without owning their construction or persisted representation.

## Success Contract

Completion is either a simplifying production adapter over an accepted Radicle source or a documented rejection with the exact blocker. Re-exporting proposal DTOs, moving authority policy, or leaving all decisions in callbacks is false completion.

## Decisions

### Baseline identities and persistence first

Run focused proposal, approval, stale-decision, replay, shell, store, and inspection tests before core changes. Record proposal/approval identity vectors, serialized facts/events, safe summaries, and CLI JSON.

### Keep shared identifiers opaque and role-typed

Lattice continues to compute proposal, workflow revision, definition revision, authority-policy, actor, and event identities. The shared core compares already validated role-specific values and never rehashes or interprets them.

### Map validation to non-mutating preview facts

Lattice successor validation can supply an admissibility observation for the exact proposal and candidate. It does not approve an actor, reserve a budget, update the workflow revision, or persist an event.

### Supply authorization from Lattice

Lattice checks actor membership, authority-policy hash, current instance epoch, and other policy. The shared core receives only the resulting authorization observation and exact bound identities. It does not define or broaden authority.

### Keep mutation and events local

A shared `Admit`, `Reject`, `Discard`, `Stale`, or `Deny` decision is input to the current Lattice transition. Lattice owns terminal state mutation, workflow revision updates, round progression, event hashing, replay, store transactions, and CLI output.

### Measure simplification

Acceptance compares removed stale/exact-decision branches, adapter size, callbacks, assertions, and fixture parity. If epoch, authority, and event logic leave no material shared decision, adoption is rejected.

### Require exact Radicle source conditionally

No dependency is added unless the producer selected extraction and published explicit licensing, provenance, canonical RID, authority policy, exact object, endpoint, and no-fallback evidence. Cargo and Nix then pin one exact revision. `flake.lock` is generated, never manually edited.

### Retain VCS rollback

After acceptance, production uses one lifecycle decision path. Rollback reverts dependencies, generated locks, adapters, and removed local branches together. There is no sibling, vendor, GitHub, or runtime fallback.

## Validation

Positive cases cover exact proposal submission, successful successor validation, authorized approval, rejection, and replay. Negative cases cover malformed identities, proposal mismatch, stale epoch/base, changed candidate, failed successor validation, authority hash mismatch, unauthorized actor, terminal mutation, repeated decisions, source mismatch, and fallback attempts.

## Claim Boundary

Passing checks establish only selected local lifecycle parity over supplied facts. They do not prove semantic diff correctness, workflow safety, actor authority truth, mutation success, model truth, persistence correctness, or release readiness.

## Risks

- The shared lifecycle may be too small after Lattice retains epoch and authority logic.
- Identity or event changes can break persisted replay.
- Generic authorization observations can hide policy context. Lattice evidence must retain exact policy and actor facts.
- Producer licensing or publication may remain blocked.
