## Why

Basalt is the hardened UCAN/Nickel/Steel policy boundary. It owns Nickel-authored contract policy, Rust enforcement, UCAN capabilities, and Steel adapter fixtures. It does not yet express a peer-auth policy contract that a fleet can enforce at the message boundary.

Celld authenticates node-to-node traffic with a versioned, body-bound, HMAC-authenticated, clock-bounded, replay-protected message. That is a clean reference for a hardened peer-auth policy boundary: the policy states the authentication facts a node must hold before it acts, and runtime enforcement checks them without owning transport or key custody.

Basalt should own the peer-auth policy and enforcement contract. Transport, key custody, clock, and durable replay storage remain consumer-owned.

## What Changes

- Add a typed peer-auth policy contract (Nickel) with explicit message, version, freshness, and replay facts.
- Add Rust enforcement that evaluates the policy over supplied authenticated-message verdicts.
- Keep the enforcement boundary separate from transport, key custody, clock, and replay storage.
- Add Steel adapter fixtures with positive and negative peer-auth decisions.
- Reference the reviewed Celld peer-protocol design as a bounded, non-parity input.
- Consume the reviewed `ucan-core` authenticated-message verdict after that change stabilizes.

## Impact

The change gives fleets a typed, reviewable policy for peer authentication decisions. It does not terminate TLS, define a membership protocol, or prove key custody.

Peer-auth policy identities and receipts remain BLAKE3.

## Dependencies

This change consumes the `ucan-core` authenticated-message verdict surface. It has no other lifecycle prerequisite.

## Non-goals

- Do not add a transport, TLS, or an in-process network stack.
- Do not implement a clock or a durable replay store.
- Do not define a membership protocol or a control plane.
- Do not claim parity with, or equivalence to, the Celld peer protocol.
- Do not convert peer authentication into end-user authorization.
