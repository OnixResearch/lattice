## Context

Choregraph proves only that accepted transitions emit declared effect intents. It does not authorize or execute those intents. Basalt evaluates exact contract, resource, ability, grant, receipt, caveat, replay, and non-claim bindings over caller-supplied verified facts.

Lattice already uses `cap-std` directory capabilities for local file effects. `cap-root` provides reusable pure lexical admission but does not perform filesystem I/O or authorize roots.

## Success Contract

Completion requires exact intent binding, fail-closed Basalt enforcement, configured capability roots, durable attempt ordering, replay suppression, positive and negative fixtures, and explicit external assumptions. Checking only a graph declaration or opening a graph-selected ambient path is false completion.

## Intent and Policy Binding

Each intent carries stable identities for machine, workflow revision, run, node, transition, operation, resource, ability, request, and canonical payload reference.

The graph declares required resource and ability values. A separate reviewed Basalt policy decides whether the supplied authority can satisfy them. Graph source cannot define or amend the policy that grants its own authority.

Policy, verified UCAN grant references, verification receipts, caveat decisions, revocation observation, replay decision, and request binding are explicit inputs. Raw tokens and key resolution remain outside the pure decision adapter.

## Dispatch Sequence

1. Lattice receives one Choregraph intent.
2. It validates exact machine, workflow, run, node, transition, and payload bindings.
3. It reserves and persists the intent and local attempt identity under existing durable rules.
4. The shell obtains current external UCAN, caveat, revocation, and replay facts.
5. Basalt evaluates the exact contract, resource, ability, request, grant, and receipt bindings.
6. A denial persists a bounded denial result and causes no product effect.
7. An allowance routes the intent to the selected bounded runtime adapter.
8. Lattice records the observed outcome and feeds one canonical result event back to Choregraph.

A Basalt allowance is scoped to the exact request. It does not grant transport, storage, persistence, retry, or product authority.

## File Capability Path

A file intent contains a configured root identifier and a relative logical path. Lattice maps the root identifier to an operator-configured `cap_std::fs::Dir` opened at startup.

`cap-root` validates the relative path and selected logical mode. Existing Lattice no-follow and capability-relative helpers perform I/O through the directory handle.

Graph input cannot invoke `ambient_authority()` or select a host root. Root lifetime, symlink, file-type, substitution, race, permission, atomic-publication, and rollback policy remain Lattice responsibilities.

## Other Runtime Adapters

Process intents use the existing bounded process adapter. Network and model intents use their selected destination, credential-reference, timeout, and response-limit policies. `cap-std` does not authorize these effects.

## Replay and Crash Boundary

Replaying machine history computes state only. A prior Basalt allowance or effect intent is evidence of an observed earlier decision, not fresh dispatch authority.

Lattice records intent and attempt identity before effects. A crash after an external effect but before outcome persistence remains an explicit unknown outcome. This change does not claim exactly-once execution.

## Dependency Blocker

Basalt currently pins UCAN through an authenticated SSH source. Public or anonymous release adoption remains blocked until Basalt provides an admitted anonymous source or an equivalently reviewed vendored boundary.

## Production Intent Blocker

The published Lattice projection admits graph operations but no effect declarations. The Seaglass adapter builds machines without effects and rejects a non-empty intent set as an authority-boundary violation. The executor dispatches resolved `NodeConfig` plans without a Choregraph `EffectIntent`.

A Lattice-synthesized Basalt request is not a Choregraph-emitted intent. Implementation must not add Basalt, persistence, or adapter wiring until an accepted design defines the production intent producer.

The design must keep one immutable workflow-machine identity or define a separate request-scoped effect-machine identity. It must also define whether Choregraph payload references identify static templates or exact resolved payloads.

The checked baseline and mechanism audit are in `evidence/baseline-and-blocker-2026-07-27.md`.

## Validation

Positive fixtures cover exact allowed file, process, and selected network intents. Negative fixtures cover missing grant, wrong resource, wrong ability, wrong request, stale policy, stale machine, revoked or replayed facts, missing root, path escape, symlink policy, unsupported adapter, replay, duplicate attempt, and crash uncertainty.

## Claim Boundary

Passing checks proves selected local binding, authorization-decision, capability-routing, and observation facts. It does not prove UCAN cryptography, key truth, revocation freshness, replay-store durability, operating-system confinement, external effects, or whole-workflow correctness.
