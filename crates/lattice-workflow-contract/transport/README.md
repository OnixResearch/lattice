# Lattice workflow transport

`lattice-workflow-transport` provides a bounded Linux exchange for `lattice-workflow-contract` values, over a Unix-domain socket or a guest `AF_VSOCK` socket.

The crate contains two parts:

- a functional core for envelope identity, admission, linkage, failure classification, the reconnect rule table (`next_step`), and observation continuity (`continuity`)
- imperative Unix and vsock shells for finite framed I/O, peer checks, deadlines, clients, and servers

## Handler boundary

Applications implement `WorkflowExchangeHandler`. The transport invokes the handler only after request identity and contract admission pass.

The handler owns workflow runtime behavior. The transport does not select operation catalogs, read credentials, execute processes, or grant authority.

## Server boundary

`serve` binds one explicit socket path, applies owner-only permissions, and handles the profile's finite connection count.

`serve_listener` drives any `Listener` whose peers are Unix credentials. It supports supervisors and local conformance tests that own listener creation.

Both paths require an ordered explicit Unix user-ID allowlist. The server checks peer credentials before it reads a request body.

`serve_vsock` binds `VMADDR_CID_ANY:port` from a `lattice.workflow-vsock-transport-profile.v1` profile and admits only its ordered peer CIDs; `serve_vsock_listener` drives any `Listener` with vsock peers. See `docs/workflow-exchange-guest-gateway.md` in the Lattice repository.

## Client ambiguity

`UnixClient` returns these distinct classes:

- `ConnectFailed`: no request connection was established
- `NotAccepted`: the client did not finish writing the request frame
- `UnknownAfterWrite`: the request frame was complete, but no complete valid admitted response arrived

The client does not retry automatically. `exchange_connected` runs the same client over a stream the caller already connected, such as a hybrid-vsock stream after its `CONNECT` handshake.

After an unknown dispatch, `RecoverDispatch` uses the original admitted request to look up the exact acceptance. Observation-only recovery fails closed.

## Fixture generation

The transport fixture generator is exposed as `emit-transport-fixtures` so its Cargo target name stays unique from the contract crate's generator:

```text
cargo run --manifest-path crates/lattice-workflow-contract/transport/Cargo.toml --example emit-transport-fixtures
```

The reconnect rule table and continuity check ship as `fixtures/exchange-recovery-fixtures-v1.json` and `fixtures/continuity-fixtures-v1.json`, emitted by the `emit-exchange-recovery-fixtures` and `emit-continuity-fixtures` examples.

## Verification

From the Lattice repository root:

```text
nix develop -c cargo test --manifest-path crates/lattice-workflow-contract/Cargo.toml --package lattice-workflow-transport --all-targets --all-features
nix develop -c cargo clippy --manifest-path crates/lattice-workflow-contract/Cargo.toml --package lattice-workflow-transport --all-targets --all-features -- -D warnings
nix run .#workflow-transport-octet-deny-all
nix flake check -L --option secret-key-files ''
```

## Non-claims

Passing checks does not prove peer intent, workflow correctness, operation authority, runtime availability, external effect success, or production deployment.
