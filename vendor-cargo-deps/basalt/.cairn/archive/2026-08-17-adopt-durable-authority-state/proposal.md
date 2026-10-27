## Why

Basalt now carries UCAN's published Durable Authority State adapters, but callers must still assemble replay and revocation ports by hand.
A second consumer pilot must prove that Basalt policy composes with the same exact state mechanisms without moving persistence, freshness, or authority into Basalt.

## What Changes

- Pin the two separate cores directly to the canonical Radicle adapter and exact reviewed object.
- Add a Basalt composition helper around UCAN's published durable replay and revocation adapters.
- Keep replay and revocation ports generic and caller-owned.
- Keep protocol profile, policy, token, signature, caveat, action, and final replay ordering in their current owners.
- Add positive and negative end-to-end compact-profile tests with in-memory ports.
- Record duplicate, incomplete, unauthenticated, unavailable, and no-fallback cases.
- Promote only the bounded two-consumer compatibility claim after validation.

## Impact

- **Source**: exact Radicle dependencies for `replay-ledger-core` and `revocation-view-core`
- **Rust**: one thin composition module and reusable test adapters
- **Authority**: Basalt retains application policy; UCAN retains protocol authority; callers retain storage, time, freshness, recovery, and effects
- **Testing**: focused end-to-end tests, source checks, release verification, Cairn gates, and Nix
