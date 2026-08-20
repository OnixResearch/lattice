# Design: Publish the Kiln workflow contract

## Context

Lattice owns workflow runtime selection, effects, persistence, retries, triggers, diagnostics, and operator interfaces. Kiln owns CI state and outcome policy.

The integration boundary needs stable producer-owned values. It must not expose the Lattice application crate or accept ambient provider authority.

## Success Contract

Completion requires an independently consumable contract with bounded values, deterministic identities, strict admission, positive and negative fixtures, and explicit non-claims.

False completion includes JSON examples without Rust validation, a profile-only contract, hidden credential fields, unbounded collections, or observations that can report paused work as terminal success.

## Decisions

### Decision: Publish one narrow contract crate

**Choice:** Add `lattice-workflow-contract` as a separate `no_std + alloc` crate.

The crate owns only interchange values, identity framing, and pure admission. It has no dependency on the Lattice application crate.

**Reason:** Consumers can pin and test the boundary without importing runtime or shell authority.

### Decision: Bind consumer identities without owning them

**Choice:** The request preserves exact effect, attempt, candidate, pipeline revision, workflow policy, graph, profile, and idempotency identities.

Lattice validates shape and linkage. It does not reinterpret canonical Kiln identity meaning.

**Reason:** Native Lattice identities must not replace the consumer's domain identities.

### Decision: Use deterministic framed BLAKE3 identities

**Choice:** Profile, graph, request, and observation identities use tagged, length-prefixed BLAKE3 framing.

Identity does not depend on JSON map ordering, host time, random values, or provider identifiers.

**Reason:** Equal admitted inputs need equal identities across independent consumers.

### Decision: Keep graph payloads operation-addressed

**Choice:** Each graph node carries a bounded node kind, effect class, adapter kind, and immutable operation identity.

The contract carries no command, credential, environment, path, or provider payload.

**Reason:** Lattice operation catalogs and shells retain executable configuration and authority.

### Decision: Make ambiguity explicit

**Choice:** Execution observations include exact request, workflow, execution, idempotency, sequence, node, receipt, and log identities.

`Unknown` remains non-terminal. Paused observations cannot claim terminal success. Reconciliation accepts only exact linked execution identities.

**Reason:** Transport uncertainty must not become a false CI outcome.

### Decision: Mirror the wire shape in Nickel

**Choice:** A typed Nickel contract validates human-reviewed fixtures and exported wire values.

Rust remains the executable admission implementation. Fixtures test agreement between both boundaries.

**Reason:** Reviewable configuration needs contract checks before runtime decoding.

## Required Flow

```text
Kiln-owned CI effect facts
  -> Kiln pure adapter lowering
  -> lattice-workflow-contract request admission
  -> Lattice application shell
  -> provider-owned workflow observation
  -> lattice-workflow-contract observation admission
  -> Kiln outcome admission
```

## Test Design

Positive tests cover profile admission, deterministic request identity, DAG admission, terminal completion, controls, and bounded log reads.

Negative tests cover unsupported adapters, unsupported controls, graph cycles, changed identities, stale executions, false paused success, missing nodes, malformed identities, and oversized logs.

The crate must compile for `wasm32-unknown-unknown` without the standard library.

## Claim Boundary

The contract validates bounded shape, identity, graph, lifecycle, and log facts. It does not execute or authorize effects and does not establish correctness or readiness.
