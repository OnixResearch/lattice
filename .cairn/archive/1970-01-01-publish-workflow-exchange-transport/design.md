# Design: Bounded workflow exchange transport

## Success contract

Completion requires an independently consumable transport crate, exact wire identities, finite I/O bounds, positive and negative tests, and a strict Octet error gate.

False completion includes a raw JSON socket, unbounded reads, ambient peer trust, a fixture-only protocol, swallowed disconnects, or transport code that executes workflow operations.

## Architecture

```text
Kiln client shell
  -> typed exchange request
  -> length-prefixed Unix stream
  -> Lattice transport server shell
  -> pure request admission
  -> application-owned WorkflowExchangeHandler
  -> pure response admission
  -> typed exchange response
```

The functional core owns envelope shape, identity framing, operation linkage, limit admission, and response classification.

The imperative shell owns socket I/O, peer observation, read and write deadlines, frame buffers, connection limits, and handler invocation.

## Decisions

### Separate transport crate

`lattice-workflow-transport` will depend on `lattice-workflow-contract`. The contract crate remains `no_std + alloc` and free of transport effects.

The new crate shares a nested workflow workspace with the contract crate. It receives its own pinned strict Octet gate without changing the root application's accepted baseline.

### Producer-owned exchange envelope

The crate will define one versioned request and response envelope. Dispatch, observe, control, log-read, and reconciliation use closed enum variants.

Observe, control, log-read, and reconciliation requests carry the exact profile, workflow request, and accepted execution. The server can therefore run contract admission without hidden durable context.

### Exact framed identities

Every request and successful response uses tagged, length-prefixed BLAKE3 framing over the typed fields. The receiver recomputes the identity before it invokes the handler or returns data.

### Finite Unix socket service

The server accepts an explicit socket path, peer policy, byte limits, deadlines, and maximum connection count. One invocation handles at most the configured count and then exits.

The server observes Unix peer credentials where the platform supports them. Admission compares the observed user identity with an explicit policy.

### Explicit handler port

`WorkflowExchangeHandler` is the only path from transport to runtime behavior. The trait receives admitted commands and returns typed outcomes.

The transport crate cannot read Lattice stores, select operation catalogs, access credentials, or execute effects.

### Disconnect ambiguity

A client that finishes a request write but cannot read a complete valid response returns `UnknownAfterWrite`. It does not retry automatically.

A failure before the complete request write remains `NotAccepted`.

### Safe diagnostics

Wire rejections use a closed code and bounded field path. They do not include raw payloads, credentials, socket paths, handler errors, or host diagnostics.

## Test design

Positive tests cover deterministic identities, dispatch, observe, control, bounded logs, reconciliation, local peer admission, and multi-frame service limits.

Negative tests cover malformed lengths, oversized frames, wrong identities, wrong schemas, stale controls, oversized logs, denied peers, read timeouts, truncated responses, and handler failure.

## Claim boundary

The transport checks bounded local protocol facts. It does not prove peer intent, operation authority, runtime correctness, workflow completion, external effect success, or production deployment.
