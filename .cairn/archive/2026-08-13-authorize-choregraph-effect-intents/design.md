## Context

Choregraph proves only that accepted transitions emit declared effect intents. It does not authorize or execute those intents. Basalt evaluates exact contract, resource, ability, grant, receipt, caveat, replay, and non-claim bindings over caller-supplied verified facts.

Lattice already uses `cap-std` directory capabilities for local file effects. `cap-root` provides reusable pure lexical admission but does not perform filesystem I/O or authorize roots.

## Success Contract

Completion requires exact intent binding, fail-closed Basalt enforcement, configured capability roots, durable attempt ordering, replay suppression, positive and negative fixtures, and explicit external assumptions. Checking only a graph declaration or opening a graph-selected ambient path is false completion.

## Intent and Policy Binding

Each intent carries stable identities for machine, workflow revision, run, node, transition, operation, resource, ability, request, and canonical payload reference.

A scope-protected intent also carries the exact collaboration scope, scope revision, membership epoch, audience-set reference, required consent-decision reference, and scope-snapshot identity. An explicit unscoped value is valid only when reviewed policy permits that resource and ability without a collaboration-scope claim.

The graph declares required resource and ability values. A separate reviewed Basalt policy decides whether the supplied authority can satisfy them. Graph source cannot define or amend the policy that grants its own authority.

Policy, verified UCAN grant references, verification receipts, caveat decisions, revocation observation, replay decision, request binding, and current collaboration-scope facts are explicit inputs. Raw tokens, membership storage, identity resolution, consent policy, and key resolution remain outside the pure decision adapter.

`RunOwner` continues to bind execution ownership. It does not become a collaboration subject, scope, audience, membership, consent, or policy value.

## Dispatch Sequence

1. Lattice receives one Choregraph intent.
2. It validates exact machine, workflow, run, node, transition, and payload bindings.
3. It reserves and persists the intent and local attempt identity under existing durable rules.
4. The shell obtains current external UCAN, caveat, revocation, replay, and required collaboration-scope facts.
5. For a scope-protected intent, the pure adapter verifies the exact scope, revision, membership epoch, audience, consent, resource, ability, request, and snapshot bindings.
6. Basalt evaluates the exact contract, resource, ability, request, grant, and receipt bindings.
7. A denial persists a bounded denial result and causes no product effect.
8. An allowance routes the intent to the selected bounded runtime adapter.
9. Lattice records the observed outcome and feeds one canonical result event back to Choregraph.

A Basalt allowance is scoped to the exact request. It does not grant transport, storage, persistence, retry, or product authority. A collaboration-scope snapshot supplies current bounded facts for one request. It is not a bearer capability or future authority.

## File Capability Path

A file intent contains a configured root identifier and a relative logical path. Lattice maps the root identifier to an operator-configured `cap_std::fs::Dir` opened at startup.

`cap-root` validates the relative path and selected logical mode. Existing Lattice no-follow and capability-relative helpers perform I/O through the directory handle.

Graph input cannot invoke `ambient_authority()` or select a host root. Root lifetime, symlink, file-type, substitution, race, permission, atomic-publication, and rollback policy remain Lattice responsibilities.

## Other Runtime Adapters

Process intents use the existing bounded process adapter. Network and model intents use their selected destination, credential-reference, timeout, and response-limit policies. `cap-std` does not authorize these effects.

## Replay and Crash Boundary

Replaying machine history computes state only. A prior Basalt allowance, scope snapshot, trigger admission, or effect intent is evidence of an observed earlier decision, not fresh dispatch authority.

Lattice records intent and attempt identity before effects. A crash after an external effect but before outcome persistence remains an explicit unknown outcome. This change does not claim exactly-once execution.

## Admitted Source Boundaries

Choregraph revision `85ea5e35caaab23260fcb27bc7c5d995dec546c4` provides the reviewed version-2 Lattice lowerer and concrete intent handoff. Cargo and Nix use this immutable Radicle revision without a sibling or runtime fallback. The source-admission fixture separates that current source revision from the historical publication receipt for `fc47c5f4ecbb7b4341af690fa42199f25d57f54c`; it does not claim that old candidate or bundle hashes attest the current revision.

Basalt revision `bb8fb5f4a763f532df96b6e65407163ac668834e` contains its reviewed UCAN snapshot at revision `277ab768eb532ac0c715a3bbff2a7dc37a702f11`. Lattice vendors that Basalt source because the canonical GitHub repository does not provide anonymous release access. The checked `SOURCE` file records the original Basalt archive identity and both revisions. Lattice then normalizes nested workspace manifests only to use the copied source inside this repository. The resulting vendor tree is not byte-identical to the original archive. Cargo has no runtime, sibling, or network fallback for this dependency.

`cap-root-core` revision `545e77e5a49e4e1d34037aa9a82d924944ac0a1b` comes from the public canonical Radicle source. Cargo and Nix use the same immutable revision.

Scope-protected production dispatch accepts only an explicit versioned snapshot through a Lattice-owned port. A missing producer or snapshot denies dispatch. Test snapshots do not claim Molten schema compatibility or production authority.

## Production Intent Model

Lattice uses one immutable Choregraph version-2 workflow machine. Static declarations bind node, transition, operation, resource, ability, request slot, and payload slot values.

A concrete start event supplies exact request and payload references for those declared slots. Runtime references do not change the workflow-machine identity.

Lattice authorizes only the concrete intent returned by `step_concrete_machine`. It does not synthesize a replacement intent or infer one from `NodeConfig` after the transition.

The request reference identifies the exact Lattice dispatch request. The payload reference identifies a canonical safe execution plan with credential references, not secret values. Credential resolution remains a shell-owned adapter fact.

The checked historical blocker is in `evidence/baseline-and-blocker-2026-07-27.md`. The source and model resolution is in `evidence/blocker-resolution-2026-08-13.md`.

## Validation

Positive fixtures cover exact allowed file, process, selected network, and current scope-protected intents. Negative fixtures cover missing grant, wrong resource, wrong ability, wrong request, stale policy, stale machine, stale scope revision, removed or expired membership, wrong audience, withdrawn consent, revoked or replayed facts, missing root, path escape, symlink policy, unsupported adapter, duplicate attempt, and crash uncertainty.

## Claim Boundary

Passing checks proves selected local binding, scope-currentness, authorization-decision, capability-routing, and observation facts. It does not prove human identity, membership-store correctness, consent-policy sufficiency, UCAN cryptography, key truth, global revocation freshness, replay-store durability, operating-system confinement, external effects, or whole-workflow correctness.
