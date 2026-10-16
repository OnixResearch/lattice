# Choregraph effect authority

Lattice authorizes only concrete intents returned by the Choregraph V2 handoff.

## Boundary

Choregraph owns machine admission and transition decisions. It returns node, transition, operation, resource, ability, request, and payload facts. These facts are not authority.

Lattice adds machine, workflow revision, run, policy, authority, and optional collaboration-scope identities. It validates the complete binding before product dispatch.

A reviewed Nickel policy defines the Basalt contracts. Workflow graph data cannot change this policy.

## Dispatch order

1. Load current authority facts.
2. Validate exact request and optional scope facts.
3. Run Basalt with verified grant, receipt, caveat, revocation, and replay dispositions.
4. Persist the bound intent and attempt.
5. Invoke only the selected bounded adapter.
6. Persist an observed success, failure, or unknown-after-dispatch outcome.

A duplicate attempt or persistence error stops before the adapter. Replay always denies fresh dispatch.

## File effects

The operator opens `cap_std::fs::Dir` roots at startup and assigns root identifiers. Graph data can name only one configured identifier and a relative path.

`cap-root` performs lexical admission. Existing capability-relative open behavior enforces the Lattice no-follow boundary. Authority code does not call `ambient_authority()`.

## Process and network effects

Process effects use the existing bounded process runner. Network effects use the selected HTTP client after method and URL validation. `cap-std` does not authorize either effect type.

## Scope currentness

Protected effects require an explicit current snapshot from a Lattice-owned port. The snapshot binds scope, revision, membership epoch, audience, consent, resource, ability, request, and snapshot identity.

A missing or stale snapshot denies dispatch. `RunOwner`, trigger history, UI audience, and prior authorization are not scope authority.

## Adoption baseline and source admission

r[impl lattice.choregraph_effect_authority.baseline]

The [pre-integration baseline](../.cairn/archive/2026-08-13-authorize-choregraph-effect-intents/evidence/baseline-and-blocker-2026-07-27.md) records the effect adapters, operator-configured roots, outer admission ordering, absent per-node attempts, replay behavior, Basalt/UCAN sources, supplied-fact boundary, and crash fixtures before integration. The [resolution record](../.cairn/archive/2026-08-13-authorize-choregraph-effect-intents/evidence/blocker-resolution-2026-08-13.md) retains the later pre-change checks and selected immutable V2 intent model.

r[impl lattice.choregraph_effect_authority.baseline.scenario.unknown]

The original baseline left implementation blocked while production intent and machine-identity ordering were unresolved; it did not substitute an external allow boolean or a task checkbox for that prerequisite. These are bounded historical adoption records, not fresh execution results or a new automatic readiness gate.

r[impl lattice.choregraph_effect_authority.source]

The admitted build uses the checked-in Basalt snapshot at `bb8fb5f4a763f532df96b6e65407163ac668834e` and its UCAN snapshot at `277ab768eb532ac0c715a3bbff2a7dc37a702f11`. `vendor-cargo-deps/basalt/SOURCE` binds their revisions and the Basalt archive identity. Cargo selects that exact local snapshot; it has no authenticated upstream fetch fallback.

r[impl lattice.choregraph_effect_authority.source.scenario.ssh]

The original baseline explicitly rejected the direct Basalt dependency because its SSH-only UCAN closure could not be reproduced without source authority. It recorded a release blocker rather than skipping Cargo-dependent checks. The resolution selected the reviewed vendored closure instead; it did not infer source access from a sandbox-safe check that deferred Cargo.

## Non-claims

r[impl lattice.choregraph_effect_authority.non_claims.scenario.scope]

`AuthorizationOutcomeEvent` records the bound local attempt, its local outcome state, and any authorization denial. An observed success is an observation of the selected local adapter result, not evidence of external truth or a global guarantee.

Checks do not prove human identity, membership-store correctness, consent-policy sufficiency, key truth, global revocation freshness, replay-store durability, UCAN cryptographic correctness, operating-system correctness, external effect success, exactly-once behavior, or whole-workflow correctness.
