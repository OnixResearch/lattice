## Why

Choregraph can compile a global role choreography into one exact local protocol artifact per role. Lattice owns runtime selection, effects, persistence, retries, diagnostics, and operator authority, but it has no runtime for those local protocol artifacts.

A protocol executor must bind every message and branch selection to one exact protocol session. It must preserve crash uncertainty and reject stale, duplicated, reordered, wrong-role, or wrong-step input before state advancement.

## What Changes

- Consume one immutable Choregraph global artifact, complete local artifact set, and projection receipt through a frozen adapter.
- Admit one exact role placement and one local artifact per role before session creation.
- Create domain-separated BLAKE3 identities for sessions, role instances, protocol steps, messages, attempts, observations, and outcomes.
- Persist protocol cursor and outbound attempt facts before transport effects.
- Advance a transfer only after the receiving role accepts the exact expected envelope and the outcome is linked to the prepared attempt.
- Preserve explicit unknown outcomes across crashes and deny automatic retry after uncertain dispatch.
- Enforce exact choice labels, peers, value identities, payload tags, local-artifact identities, and expected continuations.
- Support native and external role adapters without treating either role kind as authority.
- Use existing Lattice authorization and bounded runtime adapters for transport and external effects.
- Add safe inspection, positive and negative fixtures, crash cases, and bounded runtime evidence.

## Success Criteria

- A session starts only from a complete current Choregraph projection set and exact role placement.
- Every accepted message matches session, protocol, source role, destination role, step, label, payload tag, value, and attempt identities.
- Duplicate or reordered messages cannot advance a role twice or select another branch.
- A crash after dispatch and before admitted receipt remains an explicit unknown outcome.
- Replay computes state and diagnostics only. It never dispatches transport or external effects.
- Former-owner use is rejected after an admitted transfer commits in the session state.
- External roles require an exact contract and adapter cohort plus current product authorization.

## False Completion

A loop that reads local artifacts and calls network functions is not completion. Transport success without durable protocol-state linkage is not completion.

A retry after uncertain send, a branch selected from an unbound Boolean, or a message accepted by payload shape alone is not completion.

## Impact

- **Core**: pure role-session admission, envelope admission, cursor decisions, outcome classification, replay, and deterministic diagnostics.
- **Shell**: persistence, bounded transport adapters, external role adapters, authorization, clocks, and observations.
- **Storage**: versioned session, role cursor, message attempt, and outcome records.
- **Operator surfaces**: create, inspect, recover, retry, abandon, and replay commands with safe structured output.
- **Testing**: local runtime, adapter, persistence, crash, replay, authorization, and negative protocol fixtures.

## Dependencies

- Choregraph change `add-role-choreography-projection` must publish immutable global, local-set, and projection-receipt schemas.
- Lattice change `authorize-choregraph-effect-intents` must provide the accepted authority boundary used before product transport or external effects.
- Existing bounded process, network, credential, persistence, and unknown-outcome boundaries remain authoritative.
- OnixOS placement integration is a later consumer. The first runtime can use one reviewed local placement fixture.

## Out of Scope

- Defining choreography semantics, endpoint projection, or proof authority.
- Building a new network stack or transport protocol.
- Automatic distributed consensus over protocol cursors.
- Claiming transport confidentiality, peer identity truth, exactly-once delivery, external-role correctness, global deadlock freedom, production readiness, or release eligibility.

## Affected Specs

- `role-protocol-execution`: intake, session identity, durable execution, message and choice admission, uncertainty, external roles, evidence, and non-claims.
