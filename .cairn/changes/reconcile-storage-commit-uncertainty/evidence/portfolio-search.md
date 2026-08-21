# Bounded architecture search

## Success contract

Goal: preserve storage commit uncertainty as a distinct state and block effects until exact durable reservation read-back proves publication.

Completion evidence requires all of these results:

- The product-neutral core is published at one immutable revision.
- Cargo and Nix select that same revision without an ambient sibling path.
- Lattice retains Redb schema, transactions, workflow policy, authority, effects, operator actions, and receipts.
- Positive tests admit an exact published reservation.
- Negative tests block uncertain, repaired, corrupt, missing, stale, conflicting, and foreign observations.
- Process-death and commit-fault fixtures show that no protected effect starts before reconciliation.
- Cairn gates, focused tests, Clippy, Octet, and Nix checks pass.

False completion includes these cases:

- A commit error becomes definite non-publication.
- Process-local quarantine disappears after restart.
- Redb repair clears workflow uncertainty.
- Read-back omits the plan, operation, revision, attempt, reservation, or predecessor identity.
- Tests cover only pure classification and do not cover the shell effect gate.
- A consumer uses an ambient sibling checkout.
- Evidence claims exactly-once external effects.

Audit risks include Redb commit semantics, reopen behavior, decode failure, stale cache, conflicting rows, process death, operator retry authority, and effect-dispatch ordering.

The search uses local source and repository validators. It uses three serial correlated lenses because subagent consent is absent. The allowed outcomes are validated, blocked, exhausted, or user-decision-required.

## Approach registry

### Shared product-neutral state machine

- Family: shared core contract
- Mechanism: classify explicit persistence observations and emit typed reconciliation decisions.
- Claim: product-neutral deterministic classification stays outside Lattice storage and effect authority.
- Artifact: `transactional-reconciliation-core` published revision and consumer tests.
- State: blocked
- Blocker: the configured GitHub repository does not exist or is inaccessible, and `gh` authentication is invalid.
- Next check: restore GitHub authentication and establish `origin/main` for the producer repository.

### Lattice Redb quarantine shell

- Family: consumer persistence adapter
- Mechanism: map uncertain commit, repair, corrupt, and decode observations into durable quarantine before any effect gate.
- Claim: protected Lattice effects cannot start from an unobserved reservation.
- Artifact: Redb adapter, exact read-back path, and restart tests.
- State: active
- Next check: design the durable quarantine record after the shared contract is fixed.

### Adversarial fault cohort

- Family: storage fault injection
- Mechanism: inject commit outcomes and process death at reservation boundaries, then reopen and inspect exact rows.
- Claim: the shell preserves uncertainty across selected commit and restart boundaries.
- Artifact: positive and negative commit-fault tests plus operator evidence.
- State: active
- Next check: introduce a narrow commit boundary that tests can control without weakening production Redb behavior.

## Correlation note

These lenses come from one assistant pass. Agreement between them is not independent evidence. Repository tests and lifecycle gates remain authoritative.
