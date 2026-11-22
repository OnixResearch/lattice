## Context

The exchange transport (`crates/lattice-workflow-contract/transport`) serves one bounded Unix listener and classifies client outcomes as `ConnectFailed`, `NotAccepted` or `UnknownAfterWrite`. The application handler stores acceptance before execution. It returns the stored acceptance for an identical dispatch, finds it again through `RecoverDispatch`, and marks a non-terminal execution `Unknown` when no live worker owns it. The relocation plan (D1) puts the guest endpoint and the reconnect semantics in Lattice, and the Iroh host gateway in Kiln. Kiln vendors an MIT snapshot of the transport crate, so new behaviour must reach Kiln as a specification and fixtures.

## Decisions

### Decision: One serve loop behind a `Listener` seam

**Choice:** `Listener` exposes `accept`, `observe_peer` and `set_deadlines`, with associated `Stream: Read + Write` and `Peer` types. A crate-private `serve_connections` owns the bounded loop: it counts connections, checks the peer before reading, applies deadlines, enforces frame limits and replaces over-limit responses. `serve_listener` keeps its existing call shape and Unix user-ID rule; `serve_vsock_listener` admits CIDs. The client is split into `prepare_request_frame` and `exchange_frame`, so `exchange_connected` reuses the exact classification for pre-connected streams.

**Rationale:** The plan requires one wire behaviour across transports, with tests that inject faults at the real boundary. A peer type per listener keeps admission policy with the profile rather than the socket.

### Decision: Vsock without a new dependency or `unsafe`

**Choice:** Use nix 0.31's `socket`, `bind`, `listen`, `accept4`, `getpeername::<VsockAddr>`, `recv` and `send`. nix returns accepted descriptors as raw descriptors, and the crate forbids `unsafe`, so the stream owns the raw descriptor and closes it on drop. Deadlines are non-blocking `recv`/`send` polls at a 2 ms interval, bounded by the profile deadline.

**Rationale:** It keeps `#![forbid(unsafe_code)]` and adds no dependency. A blocking socket with kernel timeouts needs an owned descriptor, which nix's raw `accept4` result cannot become without `unsafe`. A deadline is honoured to within one poll interval.

### Decision: Rule table and continuity as pure, fixture-published functions

**Choice:** `next_step(CommandKind, ExchangeOutcome, same_incarnation) -> RecoveryStep` and `control_after_observe(expected, observed) -> RecoveryStep` implement C6. `continuity(previous, next) -> Continuous | Gap{nodes, streams} | Reject{reason}` implements C7. The generators emit every command × outcome × incarnation row, refused (forbidden) steps, and continuity cases over sealed producer observations. Tests compare the checked-in fixtures with the functions and with an independent row-by-row oracle of the planned table.

**Rationale:** Kiln implements the same rules independently, so the fixtures must be exhaustive, keep positive and negative cases together, and be tied to the code by tests.

**Interpretations recorded:**

- **`RecoverDispatch` failures:** C6 leaves the `RecoverDispatch` × `NotAccepted`/`ConnectFailed`/`UnknownAfterWrite` cell as "—". Recovery is a read-only lookup (`shell.rs:181-191`), so it retries within the caller's bound.
- **Control comparison:** an equal observed sequence licenses a resend; a higher one supersedes the control and forbids a resend; a lower one stops as unknown.
- **`Gap{nodes, streams}`:** `nodes` lists the affected node labels and `streams` the affected (node, stream) pairs, both sorted and free of duplicates.
- **Terminal changes:** "terminal state reverted" rejects any change after a terminal state, including a change to another terminal state or to `Unknown`.

### Decision: Handler profile v2 beside an unchanged v1

**Choice:** `HandlerProfileV2` carries `listener: {kind: unix, socket_path, transport} | {kind: vsock, transport: C5}` with identity domain `lattice.workflow-exchange-handler-profile.v2`. `VersionedHandlerProfile` holds either version for validation, identity and route selection. `load_profile` selects by the evaluated `schema` field, and `serve` selects the listener. Shared body validation keeps the version-one rule order and refuses workflow pause and resume.

**Rationale:** D6 keeps every v1 schema byte-compatible, and C5 requires a separate identity domain. Accepting both versions through `impl Into<VersionedHandlerProfile>` keeps the CLI and the existing callers unchanged.

### Decision: Q1 through a fault-injecting listener

**Choice:** `tests/workflow_exchange_reconnect.rs` runs the real `ApplicationHandler` with a v2 vsock profile under `serve_vsock_listener`. A test `Listener` hands the guest one end of a Unix socketpair, reports a configured peer CID, and injects one fault per connection: close before read, dropped, corrupted, truncated or delayed response. The guest's attempted response bytes are captured so the test can show what a lost response contained.

**Rationale:** `vsock_loopback` is not loaded on the qualification host. Socketpairs keep the real handler, frames, deadlines and classification, and change only the address family.

## Risks / Trade-offs

- **Vsock shell coverage:** `VsockListener` and real CID observation are not exercised here. Kiln's Cloud Hypervisor test (Q2) must qualify them.
- **Wire observations:** observations carry no `cancellation_requested` field. The cancel race asserts it on the durable record and shows that the observation stays `Running` until the worker is observed `Cancelled`.
- **Execution identities:** executions admitted under a v2 profile have different execution identities from the same routes under v1, because the handler profile identity is an input.
- **Fixture name:** the rule-table fixture is `exchange-recovery-fixtures-v1.json`, because `recovery-fixtures-v1.json` already holds the vendored `RecoverDispatch` producer fixtures.
