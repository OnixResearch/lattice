# Bounded workflow exchange transport

Lattice publishes a local Linux transport beside `lattice-workflow-contract`. It serves over a Unix-domain socket, or over `AF_VSOCK` inside a microVM guest.

## Ownership

The contract crate owns workflow request and observation meaning. The transport crate owns local envelope identity, framing, bounds, peer admission, and disconnect classification.

The Lattice application owns `WorkflowExchangeHandler`. Only that handler can select operations, load durable state, authorize effects, or invoke the workflow runtime.

Consumers retain their domain state and outcome policy. A transport response cannot replace canonical consumer state.

## Exchange operations

The version-one envelope supports these operations:

1. Dispatch one exact admitted workflow request.
2. Observe one exact accepted execution.
3. Apply one admitted cancel, pause, or resume control.
4. Read one bounded log range linked to an admitted observation.
5. Reconcile one exact accepted execution after uncertainty.
6. Recover a lost dispatch acceptance by the original admitted request without redispatch authority.

Observe, control, log-read, and reconciliation carry the original profile, request, and acceptance. The server can validate the complete linkage without ambient context.

## Wire shape

Each message uses one unsigned big-endian frame length followed by one JSON envelope. Log bytes use canonical standard Base64 inside that envelope.

The selected profile bounds the complete frame before allocation.

Request and response identities use tagged, length-prefixed BLAKE3 framing. The receiver recomputes each identity before the handler or consumer receives data.

## Unix service

The typed operator profile is `contracts/workflow-transport-profile.ncl`. Nickel rejects zero limits and empty peer sets before runtime decoding.

The server requires:

- one explicit socket path
- one ordered Unix user-ID allowlist
- positive frame and log limits
- positive read and write deadlines
- one positive finite connection limit

The server checks peer credentials before request decoding. The socket path and allowed users remain operator authority inputs.

`serve_listener` drives any `Listener` whose peers are Unix credentials, so supervisors and conformance tests can supply already-connected streams. `exchange_connected` is the matching client half for a stream the caller already connected.

## Vsock guest service

`lattice.workflow-vsock-transport-profile.v1` carries the same limits and non-claims as the Unix profile. It replaces the user-ID allowlist with a port and an ordered peer-CID allowlist. `serve_vsock` binds `VMADDR_CID_ANY:port` and admits only allowed peer CIDs. It then runs the same serve loop, frames and failure classes as the Unix service.

See [Guest workflow exchange over vsock](workflow-exchange-guest-gateway.md) for the guest profile, the host gateway boundary, and evidence limits.

## Disconnects

A failed connection before a complete write is not accepted. A failed or invalid response after a complete write has an unknown outcome.

The client never converts unknown into failure or success. Reconciliation must name the same accepted execution.

If dispatch acceptance is missing, `RecoverDispatch` carries the original admitted request. It returns only the exact linked acceptance or a closed rejection.

## Reconnect and continuity rules

The transport publishes two pure decisions for callers that reconnect:

- `next_step` and `control_after_observe` (`lattice.workflow-exchange-recovery.v1`) name the only admissible step after each client outcome. An unknown dispatch is recovered, never resent blindly. A missing acceptance is redispatched only on the same endpoint incarnation. A control whose expected sequence is behind is never resent.
- `continuity` (`lattice.workflow-observation-continuity.v1`) rejects an observation that changes identity, lowers the sequence, changes a terminal state or moves a log cursor backwards. It names every log stream with a gap.

Both ship as checked-in fixtures (`exchange-recovery-fixtures-v1.json` and `continuity-fixtures-v1.json`) that consumers implement independently.

## Lattice application handler

The `lattice workflow-exchange serve` command composes this transport with the normal Lattice store and headless runtime.

Its typed handler profile maps exact consumer graphs to exact saved workflow revisions. Durable admission stores acceptance before execution. Acceptance recovery never redispatches.

See [Durable workflow exchange application](workflow-exchange-application.md) for route authority, lifecycle, logs, setup, and non-claims.

## Kiln integration

Kiln can implement `WorkflowExchangePort` by mapping each closed Kiln operation to one producer-owned transport command.

Kiln must preserve its own effect, attempt, candidate, pipeline, policy, idempotency, workflow, and execution identities. It must retain `UnknownAfterWrite` as an unknown provider outcome.

## Non-claims

The transport proves only checked local shape, identity, linkage, peer, and limit facts. It does not prove workflow execution, external effects, host confinement, peer intent, or deployment readiness.
