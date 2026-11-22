# Guest workflow exchange over vsock

Lattice can serve its durable workflow exchange inside a no-NIC microVM guest. The guest listens on `AF_VSOCK`. The host gateway forwards unchanged exchange frames to it. This page covers the Lattice side of that arrangement: the guest listener, its profiles, and the reconnect and continuity rules a host gateway must follow.

The host gateway itself, with Iroh, placement and principal binding, belongs to Kiln.

## Topology

```text
remote consumer ──Iroh──▶ Kiln host gateway ──CH hybrid vsock "CONNECT <port>\n"──▶ guest AF_VSOCK listener
                                                                                     │
                                                                  lattice workflow-exchange serve
                                                                  (WorkflowExchangeService + ApplicationHandler)
```

- The guest runs `lattice workflow-exchange serve --profile <handler profile v2>`, using a `vsock` listener.
- The listener binds `VMADDR_CID_ANY:port` and admits a connection only when the kernel-observed peer CID (`getpeername`) is in the profile's allow-list. On Cloud Hypervisor the host is CID 2.
- Frames, envelopes, identities and limits are exactly those of the Unix transport ([workflow-exchange-transport.md](workflow-exchange-transport.md)). A host gateway forwards bytes without decoding them, so it needs no Lattice code.
- On the host side, the stream returned after Cloud Hypervisor's `CONNECT <port>` handshake is an ordinary Unix stream. `lattice_workflow_transport::exchange_connected` runs one exchange over such a stream with the normal deadlines and failure classes.

## Profiles

### `lattice.workflow-vsock-transport-profile.v1`

| Field | Rule |
| --- | --- |
| `schema` | exactly `lattice.workflow-vsock-transport-profile.v1` |
| `port` | `1 ..= 2^32-2`; `VMADDR_PORT_ANY` is refused |
| `limits` | the Unix `TransportLimits` record, with the same positive bounds |
| `peers.allowed_cids` | strictly ascending, non-empty, at most 64; `VMADDR_CID_ANY` is refused |
| `non_claims` | must include every Unix transport non-claim |

Every record is `deny_unknown_fields`. The Nickel contract is `VsockTransportProfile` in `contracts/workflow-transport-profile.ncl`.

### `lattice.workflow-exchange-handler-profile.v2`

Version two is version one with `socket_path` and `transport` replaced by one tagged `listener`:

```nickel
listener = { kind = "unix", socket_path = "/run/lattice/workflow-exchange.sock", transport = { ... } }
# or
listener = { kind = "vsock", transport = { schema = "lattice.workflow-vsock-transport-profile.v1", port = 10300, ... } }
```

- The identity domain is `lattice.workflow-exchange-handler-profile.v2`. Execution identities derive from the handler profile identity, so a profile served under v2 yields different execution identities from the same routes under v1.
- Version one stays accepted. Its schema, wire shape and identity are unchanged, and it always selects the Unix listener.
- Workflow `pause` and `resume` stay refused in both versions. A VMM that can pause a guest does not make workflow pause an advertised control.
- `load_profile` picks the version from the evaluated `schema` field. The Nickel contract is `HandlerProfileV2` in `contracts/workflow-exchange-handler-profile.ncl`.
- `tests/fixtures/workflow-exchange-handler/profile-v2-vsock.ncl` is a complete guest example.

## Listener boundary

`lattice_workflow_transport::Listener` is the seam shared by the Unix and vsock endpoints. It has three operations: accept one stream, observe that stream's peer, and apply the profile deadlines. Admission policy stays with the serving profile. `serve_listener` admits Unix user IDs. `serve_vsock_listener` admits vsock CIDs. Both drive the same bounded loop, which handles a finite connection count, checks the peer before reading, refuses oversized frames before allocating, and replaces an over-limit response with a closed rejection.

The vsock shell enforces read and write deadlines with non-blocking `recv` and `send` calls, retried at 2 ms intervals until the profile deadline. A deadline is therefore honoured to within one poll interval. The polling exists because the transport crate forbids `unsafe`, and nix's `accept4` returns a raw descriptor that cannot safely become an owned socket with kernel timeouts.

## Reconnect rules: `lattice.workflow-exchange-recovery.v1`

`next_step(command, outcome, same_incarnation)` is a pure rule table. After a failed or ambiguous exchange it names the only admissible next step:

| Command | `ConnectFailed` / `NotAccepted` | `UnknownAfterWrite` | Response |
| --- | --- | --- | --- |
| `Dispatch` | resend identical bytes | `RecoverDispatch` with the identical request | success → complete; rejection → stop |
| `RecoverDispatch` | retry, bounded | retry, bounded | `NotFound`: resend `Dispatch` only if `same_incarnation`, else stop unknown |
| `Control` | resend identical bytes | observe, then `control_after_observe` | success → complete; rejection → stop |
| `Observe` / `ReadLog` / `Reconcile` | retry, bounded | retry, bounded | success → complete; rejection → stop |

- `InvalidProfile` and `InvalidRequest` mean nothing was sent, so the step is to stop as invalid.
- `control_after_observe(expected_sequence, observed_sequence)` resends a control only when the observed sequence still equals its expected sequence. An applied control advances the sequence. A higher sequence means the control is superseded and must never be resent. A lower sequence means the observation predates the control's basis, so the step is to stop as unknown.
- `same_incarnation` is gateway evidence, never wire data. It means the guest's environment and attempt are unchanged and no restore from an older snapshot happened after the original dispatch was written.
- The table never retries by itself. Retry bounds and backoff belong to the caller.

## Observation continuity: `lattice.workflow-observation-continuity.v1`

`continuity(previous, next)` compares the last retained observation with a new one, for example after a reconnect, an endpoint restart or a vsock reset:

- **Reject:** the request, execution or workflow identity changed; the sequence decreased; a terminal state changed (including terminal → `Unknown`); or a log cursor moved backwards for any (node, stream) pair.
- **Gap:** at least one log reference in `next` has `gap_before`, is not `retained`, or is `truncated`. `Gap.nodes` lists the affected nodes and `Gap.streams` the affected (node, stream) pairs, both sorted.
- **Continuous:** otherwise. A restart that reports `Unknown` after `Running` is continuous: it is a reported transition, not a fork.

## Kiln-facing fixtures

The checked-in fixtures are the specification Kiln implements independently. Kiln consumes them across the process and fixture boundary and does not link Lattice code.

| File | Content |
| --- | --- |
| `crates/lattice-workflow-contract/transport/fixtures/exchange-recovery-fixtures-v1.json` | every command × outcome × incarnation row (192), `control_after_observe` cases, and refused (forbidden) steps |
| `crates/lattice-workflow-contract/transport/fixtures/continuity-fixtures-v1.json` | continuous, gap and every rejection case over sealed producer observations |

The rule-table fixture is not named `recovery-fixtures-v1.json`, because that file already holds the `RecoverDispatch` producer fixtures that Kiln vendors. To regenerate:

```text
cargo run -p lattice-workflow-transport --example emit-exchange-recovery-fixtures > crates/lattice-workflow-contract/transport/fixtures/exchange-recovery-fixtures-v1.json
cargo run -p lattice-workflow-transport --example emit-continuity-fixtures > crates/lattice-workflow-contract/transport/fixtures/continuity-fixtures-v1.json
```

## Evidence

- **Q0:** the pure profile, rule-table and continuity tests, plus the fixture tests (`transport/tests/exchange-recovery-fixtures.rs`, `transport/tests/continuity-fixtures.rs`).
- **Q1:** `tests/workflow_exchange_reconnect.rs` drives the real `ApplicationHandler`, a v2 vsock handler profile, the real vsock serve loop with CID admission, and the real client classification. A fault-injecting `Listener` hands the guest one end of a Unix socketpair and reports the configured peer CID. The scenarios cover a lost dispatch response, duplicate and changed requests, an incomplete write, malformed and truncated responses, an oversized frame, read deadlines, a refused peer CID, a handler restart reporting `Unknown`, cancellation racing a reconnect, and a log cursor resuming after a lost read. Server write deadlines are covered at the transport level in `transport/src/tests/listener.rs`.
- **Not exercised here:** real `AF_VSOCK`. `vsock_loopback` is not loaded on the qualification host, so `VsockListener::bind` and peer-CID observation against a real hypervisor are qualified by Kiln's Cloud Hypervisor test (Q2).

## Non-claims

- A peer CID is a hypervisor-asserted address. It does not identify a principal, an execution or a placement; the host gateway binds those.
- The rule table and continuity check do not prove which endpoint incarnation answered, that an observation is current, or that any workflow effect happened.
- Nothing here is cross-host, live-model or production evidence.
