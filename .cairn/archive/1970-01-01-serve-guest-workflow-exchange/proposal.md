## Why

Lattice #11 needs the durable workflow exchange to run inside a no-NIC microVM guest and to survive reconnects across a host gateway. Today the exchange serves only a Unix socket. The reconnect behaviour after a lost or ambiguous exchange exists only as prose. A consumer cannot tell a continuous observation history from a fork. Kiln owns the host gateway (Kiln #54) and must implement the same reconnect and continuity rules without linking new AGPL Lattice code, so it needs those rules as a specification and fixtures.

## What Changes

- Make the transport serve loop generic over a small `Listener` seam (accept, peer observation, deadlines), with profile-owned peer admission. The Unix path keeps its wire behaviour, failure classes and peer rule. Add `exchange_connected` for streams the caller already connected.
- Add the guest vsock listener and `lattice.workflow-vsock-transport-profile.v1` (C5). It binds `VMADDR_CID_ANY:port` and admits only an ordered allow-list of peer CIDs, using nix's existing `VsockAddr` and no new dependency.
- Add the pure reconnect rule table `lattice.workflow-exchange-recovery.v1` (C6) and the observation continuity check `lattice.workflow-observation-continuity.v1` (C7). They ship as the Kiln-facing fixtures `exchange-recovery-fixtures-v1.json` and `continuity-fixtures-v1.json`.
- Add handler profile `lattice.workflow-exchange-handler-profile.v2`, with an explicit Unix or vsock listener and its own identity domain. Version one stays accepted and byte-compatible. Workflow pause and resume stay refused.
- Add Q1 tests that drive the real `ApplicationHandler` through a fault-injecting `Listener` over Unix socketpairs. Document the guest gateway boundary.

## Owner decisions applied

- **OD-1:** Kiln consumes this work only through the `lattice` process boundary and the published fixtures. Nothing here assumes Kiln links new Lattice code.
- **OD-2:** no placement or fencing provider is touched. Production placement waits for Aspen; no local-lock fallback is introduced.
- **OD-3:** no live model is involved. This change has no model surface.
- **OD-5:** the #12 addenda stay with their producers. This change makes no #12 claim.

## Impact

- **Files:** transport `unix.rs`, new `vsock.rs` (module `guest`) and `reconnect/{rules,continuity}.rs` (modules `rules` and `history`; Octet's module-size and naming lints rule out three new top-level files), `model.rs` (C5 types), `core.rs` (shared profile checks), `lib.rs`, fixtures, examples and tests; application `model.rs`, `admission.rs`, `application/{service,shell,mod}.rs` and `workflow_exchange/mod.rs`; Nickel contracts and fixtures; `tests/workflow_exchange_reconnect.rs`; documentation.
- **Non-goals:** real `AF_VSOCK` qualification (`vsock_loopback` is not loaded here; Kiln's Cloud Hypervisor test qualifies it); the Iroh host gateway, principal and placement binding (Kiln); new execution states; cross-host, live-model or production claims.
