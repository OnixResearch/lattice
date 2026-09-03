## Phase 1: Dependencies and baseline

- [ ] [depends:choregraph-add-asynchronous-flow-profile] Pin the immutable Choregraph global, complete local-set, projection-receipt, schema, assumption, fixture, and source identities. r[lattice.async_flow.intake]
- [ ] [depends:trellis-prove-asynchronous-flow-algebra] Pin the immutable Trellis law, proof, verifier, assumption, manifest, and source identities. r[lattice.async_flow.law_evidence]
- [ ] [serial] Record current role-protocol, persistence, attempt, recovery, replay, adapter, authority, Cairn, Octet, and Nix baselines. r[lattice.async_flow.validation]

## Phase 2: Pure intake and planning

- [ ] [serial] Define strict Lattice-owned DTOs for global flows, local flows, projection receipts, edge semantics, operator obligations, law references, and nondeterminism assumptions. r[lattice.async_flow.intake]
- [ ] [serial] Implement pure intake that rejects stale, partial, duplicate, reordered, unknown, or incompatible artifacts. r[lattice.async_flow.intake]
- [ ] [serial] Implement complete role placement, named runtime-limit admission, and deterministic flow-session planning. r[lattice.async_flow.session]
- [ ] [parallel] Add positive intake fixtures and negative stale-cohort, missing-role, duplicate-local, wrong-schema, wrong-proof-domain, and unsupported-version fixtures. r[lattice.async_flow.intake] r[lattice.async_flow.validation]

## Phase 3: Durable flow core

- [ ] [serial] Define pure flow session, edge, item observation, operator, window, closure, assumption, outcome, and terminal state types. r[lattice.async_flow.session] r[lattice.async_flow.closure]
- [ ] [serial] Implement ordered and unordered item admission with exact edge, item, source, destination, position, repetition, and observation identities. r[lattice.async_flow.edge_semantics]
- [ ] [serial] Implement law-evidence admission for associativity, commutativity, idempotence, monotonicity, and exact durable deduplication profiles. r[lattice.async_flow.law_evidence]
- [ ] [serial] Implement explicit closed and live edge transitions plus bounded window and checkpoint closure. r[lattice.async_flow.closure]
- [ ] [serial] Implement pure operator readiness, reduction, result classification, and limit-exhaustion decisions. r[lattice.async_flow.session] r[lattice.async_flow.edge_semantics]
- [ ] [parallel] Add negative unordered-concatenation, duplicate-count, live-final-fold, forged-closure, unbound-marker, stale-proof, overflow, and exhausted-limit tests. r[lattice.async_flow.edge_semantics] r[lattice.async_flow.law_evidence] r[lattice.async_flow.closure] r[lattice.async_flow.validation]

## Phase 4: Persistence, adapters, and effects

- [ ] [serial] Add versioned persistence for flow sessions, edge positions, item observations, windows, closure markers, assumptions, attempts, outcomes, and terminal records. r[lattice.async_flow.persistence]
- [ ] [serial] Persist exact item and attempt facts before any transport or external effect. r[lattice.async_flow.persistence] r[lattice.async_flow.uncertainty]
- [ ] [serial] Route admitted plans through existing bounded adapters and preserve pre-start, possible-dispatch, observed, terminal, timeout, cancellation, and uncertain outcomes. r[lattice.async_flow.uncertainty]
- [ ] [serial] Deny automatic retry after possible dispatch and require a fresh explicit attempt with unchanged artifacts and duplicate-risk policy. r[lattice.async_flow.uncertainty]
- [ ] [serial] Add the protected effect-release gate for closed input, closed windows, or exact prefix-safety evidence. r[lattice.async_flow.effect_release]
- [ ] [serial] Record nondeterminism assumptions and selected ChaosControl profiles in state, diagnostics, and evidence. r[lattice.async_flow.assumptions]
- [ ] [parallel] Add crash and denial tests before persistence, after observation, after possible dispatch, at closure, before effect release, and before outcome commit. r[lattice.async_flow.persistence] r[lattice.async_flow.uncertainty] r[lattice.async_flow.effect_release] r[lattice.async_flow.validation]

## Phase 5: Pilot, replay, and evidence

- [ ] [serial] Implement the bounded unordered duplicate-bearing set-union pilot with exact Trellis ACI evidence. r[lattice.async_flow.pilot]
- [ ] [parallel] Add negative order-sensitive concatenation and duplicate-sensitive counting pilot fixtures. r[lattice.async_flow.pilot] r[lattice.async_flow.validation]
- [ ] [serial] Extend replay to recompute flow state and effect-release decisions without transport, retries, persistence mutation, or protected effects. r[lattice.async_flow.replay]
- [ ] [serial] Add domain-separated BLAKE3 identities and bounded evidence for producer, law, runtime, state, adapter, observations, uncertainty, assumptions, replay, and non-claims. r[lattice.async_flow.evidence]
- [ ] [parallel] Add safe operator inspection for edge state, closure, windows, limits, assumptions, blockers, uncertainty, and evidence references. r[lattice.async_flow.evidence]

## Phase 6: Documentation and closeout

- [ ] [parallel] Document intake, edge semantics, law evidence, live versus closed input, limits, uncertainty, effect release, assumptions, replay, adapters, and non-claims. r[lattice.async_flow.boundary]
- [ ] [serial] Run focused core, persistence, crash, replay, adapter, authority, set-union, formatting, Clippy, Octet, Cairn, and bounded Nix checks. r[lattice.async_flow.validation]
- [ ] [serial] Run the adversarial audit and block archive for semantic inference, law mismatch, forged closure, duplicate application, hidden retry, unsafe effect release, erased assumptions, or replay dispatch. r[lattice.async_flow.validation] r[lattice.async_flow.boundary]
- [ ] [serial] Review sync and archive plans and archive only with immutable producer and proof cohorts plus current positive and negative evidence. r[lattice.async_flow.validation]
