## Context

Choregraph already publishes finite role projections, and Lattice already executes them with durable attempts and explicit uncertainty. The proposed Choregraph flow profile adds a different semantic unit: a graph of operators connected by typed asynchronous edges.

Those edges can be live, unordered, duplicate-bearing, or incomplete. Their consumers can be incremental, completion-dependent, order-sensitive, duplicate-sensitive, monotone, or effect-producing.

The executor must preserve these distinctions without becoming a second compiler or proof authority.

## Success Contract

The accepted result is one durable Lattice path for a frozen Choregraph flow cohort and one frozen Trellis law cohort. It admits exact artifacts, creates a bounded session, executes selected local operators, and records all observations and uncertainties.

The first end-to-end pilot reduces unordered duplicate-bearing item sets with set union. It does not claim a general stream processor.

## Approach Registry

| Family | Mechanism | State |
| --- | --- | --- |
| Proof-linked durable flow executor | Exact projected artifacts, pure admission, durable bounded shell | selected |
| Generic async stream loop | Infer semantics from runtime behavior | rejected |
| Marker-trait law admission | Trust consumer implementations of empty law traits | rejected |
| Compile flows into ordinary workflow edges | Erase closure, ordering, repetition, and law meaning | rejected |
| Molten-native direct executor | Move Lattice persistence and effect authority into transport | deferred |

## Decisions

### Decision: consume frozen flow artifacts

**Choice:** Lattice consumes one exact global flow, ordered complete local-flow set, projection receipt, schema cohort, assumption set, and Trellis law-evidence set.

The intake adapter maps canonical producer values into strict Lattice-owned types. Field-name similarity and JSON shape do not establish compatibility.

**Rationale:** Lattice must not recompile, infer, or weaken producer semantics.

### Decision: keep flow decisions in a pure core

**Choice:** Pure code owns artifact admission, edge compatibility, operator readiness, window transitions, closure transitions, duplicate policy, result classification, effect-release decisions, and replay reduction.

The shell owns files, storage, transport, clocks, cancellation, credentials, authorization, retries, and output publication.

**Rationale:** Flow meaning must remain testable without an async runtime or external service.

### Decision: represent runtime state explicitly

**Choice:** A flow session records exact state for each role, local flow, operator, edge, item observation, window, closure marker, nondeterminism assumption, and external attempt.

Live edges retain named bounds for queued items, bytes, observations, windows, and retained evidence. Limit exhaustion returns a typed blocked or failed outcome. It never becomes silent loss.

**Rationale:** A live stream still needs finite runtime resources and bounded evidence.

### Decision: separate logical semantics from transport outcomes

**Choice:** Choregraph edge semantics constrain logical admission. Lattice transport adapters continue to report pre-start rejection, possible dispatch, accepted observation, terminal failure, timeout, cancellation, and uncertainty.

A logical repetition profile does not strengthen transport delivery. Lattice does not expose a transport-level exactly-once state.

**Rationale:** Application effects can remain uncertain after a connection or process failure.

### Decision: require exact law evidence

**Choice:** Unordered input requires order-insensitivity evidence. Duplicate-bearing input requires idempotence or an exact durable deduplication profile. Regrouping requires associativity.

Each reference binds the operation identity, admitted value domain, proof source, proof property, verifier cohort, assumptions, and evidence identity. Empty marker traits and unbound human assertions are rejected.

**Rationale:** Algebraic laws apply to one implementation and domain, not to a display name.

### Decision: distinguish closure from resource bounds

**Choice:** `Closed` means that one admitted closure marker fixes the finite logical input. `Live` means that more input can arrive.

A completion-dependent operator on live input requires an explicit bounded window or checkpoint operator. A runtime resource limit does not logically close an edge.

**Rationale:** Logical completion and bounded memory are different facts.

### Decision: gate protected effect release

**Choice:** An operator result can request a protected effect only after one of these conditions holds:

- its required input is closed;
- its selected window or checkpoint is closed; or
- exact prefix-safety evidence covers the result and effect plan.

Monotonicity without prefix-safety evidence can update internal knowledge. It cannot authorize an external effect.

**Rationale:** A valid partial result can still cause an irreversible action too early.

### Decision: make nondeterminism assumptions visible

**Choice:** A nondeterminism boundary binds its Choregraph assumption identity, affected operators and edges, expected observational relation, selected ChaosControl profile, and allowed claim class.

The runtime records the boundary in inspection and evidence. It does not rename the boundary as Rust memory-safety `unsafe`.

**Rationale:** Assumptions need test and review surfaces without overloading language safety terms.

### Decision: reuse existing placement and adapters

**Choice:** The first profile uses the existing complete role-placement shape and one reviewed local adapter cohort. Later OnixOS output can supply exact placement. Later Molten integration can implement the same narrow transport port.

**Rationale:** Flow semantics do not transfer machine inventory or transport authority to Lattice core.

### Decision: preserve replay as effect-free

**Choice:** Replay reads admitted session records and recomputes pure edge, operator, closure, window, result, and effect-release decisions. It emits no transport, storage mutation, retry, or protected effect.

**Rationale:** Replay must not repeat an external observation or action.

## Functional Core and Imperative Shell

The functional core returns admission decisions, next state, diagnostics, and typed effect plans. It never executes those plans.

The imperative shell loads exact artifacts and state, obtains current authorization and adapter facts, calls the core, persists required records, and executes only admitted plans.

## Validation Strategy

Positive fixtures cover closed ordered input, live incremental input, bounded windows, unordered set union, duplicate set union, admitted deduplication, explicit closure, prefix-safe internal progress, and effect-free replay.

Negative fixtures cover stale cohorts, partial local sets, unordered concatenation, duplicate-sensitive counting, completion-dependent live folds, forged closure, exhausted bounds, unbound law markers, stale proof identity, unsafe effect release, hidden retry, and false delivery claims.

Crash cases cover before persistence, after item admission, after possible dispatch, before receiver observation, after window closure, and before effect outcome persistence.

## Adversarial Audit

The audit must try to make these invalid outcomes pass:

- reorder an order-sensitive operator;
- double-apply one duplicate-bearing item;
- close a live edge from a resource limit;
- release an effect from monotonicity alone;
- turn dispatch uncertainty into retry or success;
- cite a law for another implementation or domain;
- erase a nondeterminism assumption from evidence; or
- dispatch during replay.

Any successful counterexample blocks archive.

## Risks and Trade-offs

- Durable per-item and per-window records increase storage work.
- Exact proof cohorts can become stale when operator implementations change.
- A conservative effect-release rule can delay useful actions.
- The first set-union pilot covers only one algebraic profile.
- Live-flow resource limits can stop progress even when logical semantics remain valid.

## Non-Claims

This design does not prove transport delivery, remote durability, external-role correctness, exactly-once effects, universal determinism, universal convergence, liveness under arbitrary faults, host confinement, production readiness, or release eligibility.
