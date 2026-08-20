# Bounded workflow exchange transport

Lattice publishes a local Linux Unix-domain transport beside `lattice-workflow-contract`.

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

## Disconnects

A failed connection before a complete write is not accepted. A failed or invalid response after a complete write has an unknown outcome.

The client never converts unknown into failure or success. Reconciliation must name the same accepted execution.

## Kiln integration

Kiln can implement `WorkflowExchangePort` by mapping each closed Kiln operation to one producer-owned transport command.

Kiln must preserve its own effect, attempt, candidate, pipeline, policy, idempotency, workflow, and execution identities. It must retain `UnknownAfterWrite` as an unknown provider outcome.

## Non-claims

The transport proves only checked local shape, identity, linkage, peer, and limit facts. It does not prove workflow execution, external effects, host confinement, peer intent, or deployment readiness.
