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

## Non-claims

Checks do not prove human identity, membership-store correctness, consent-policy sufficiency, key truth, global revocation freshness, replay-store durability, UCAN cryptographic correctness, operating-system correctness, external effect success, exactly-once behavior, or whole-workflow correctness.
