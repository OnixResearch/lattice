## Why

Choregraph emits pure declared effect intents. Those intents are not authority and must not directly call Lattice runtimes.

Lattice needs an exact policy and capability boundary that uses Basalt for resource and ability authorization, `cap-root` for lexical path admission, and `cap-std` for local file I/O. This boundary must preserve durable attempt evidence and prevent replay from dispatching effects.

## What Changes

- Bind each Choregraph intent to the exact machine, workflow revision, run, node, transition, operation, resource, ability, request, and payload identities.
- For scope-protected resources, also bind the collaboration scope, scope revision, membership epoch, audience, required consent decision, and current scope-snapshot identity.
- Evaluate the exact resource and ability through Basalt after UCAN verification facts are supplied and before product effects run.
- Keep Basalt policy separate from graph-authored declarations so a graph cannot grant itself authority.
- Route local file effects through configured root identifiers, `cap-root`, and existing `cap-std` directory capabilities.
- Route process and network intents through their existing bounded adapters rather than `cap-std`.
- Persist the bound intent and attempt before effects, then feed observed outcomes back as machine events.
- Deny missing, stale, mismatched, replayed, revoked, unsupported, or unavailable authority without product effects.

## Success Criteria

- No product effect runs without an exact allowed Basalt decision bound to the current intent.
- A scope-protected effect also requires a current matching collaboration-scope snapshot at dispatch time.
- Denied authorization produces no product effect and returns a deterministic outcome event.
- Graph data cannot choose a new ambient filesystem root.
- Replay never reuses an earlier authorization as fresh dispatch authority.
- Existing Lattice crash and uncertainty boundaries remain explicit.

## Non-Goals

- Proving key resolution, revocation freshness, replay-store durability, UCAN cryptography, operating-system behavior, remote effect success, or exactly-once execution.
- Persisting collaboration membership or treating a scope snapshot, trigger receipt, UI audience, or `RunOwner` value as authority.
- Using `cap-std` for process or network authorization.
- Treating Choregraph proof or Basalt allowance as whole-workflow authority.

## Impact

- **Policy**: Nickel-authored Basalt contracts separate from workflow graph data.
- **Runtime**: authorization adapter, effect attempt binding, file capability routing, and outcome events.
- **Testing**: allowed and denied policy, scope-currentness, membership, consent, path, replay, stale, mismatch, crash, and adapter fixtures.
