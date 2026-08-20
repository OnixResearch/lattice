# Design: Serve the durable workflow exchange

## Context

The transport crate validates wire shape, identity, peer credentials, limits, and response linkage. The Lattice application must still select executable workflow state and own all effects.

A workflow request contains immutable operation identities. It does not contain commands, paths, credentials, or provider payloads.

## Success contract

Completion requires one supported command that serves the existing Unix transport with durable application state. A complete implementation must recover exact acceptance after a lost reply, preserve uncertainty after process loss, and never use a fallback process path.

False completion includes an echo handler, an in-memory acceptance map, automatic redispatch, a route that ignores saved workflow revision drift, or a command that executes consumer payloads directly.

## Architecture

### Functional core

The workflow exchange core owns:

- typed handler profiles and route admission;
- deterministic BLAKE3 workflow, execution, adapter, receipt, log, and profile identities;
- exact route selection;
- durable lifecycle transitions;
- terminal node and bounded diagnostic-log projection;
- observation construction and validation.

The core performs no filesystem, network, process, database, clock, or environment access.

### Imperative shell

The shell owns:

- Nickel evaluation;
- saved workflow and revision loading;
- redb transactions;
- Unix socket serving;
- background runtime threads;
- cancellation flags;
- headless workflow execution;
- current process and peer observations.

The existing transport remains the Unix adapter. The existing headless runner remains the execution adapter.

## Exact route binding

Each operator-authored route binds:

- one contract profile identity;
- one request graph identity;
- one saved workflow name;
- one saved workflow revision identity;
- an ordered mapping from request node IDs and operation identities to Lattice workflow node IDs;
- one explicit effect-authority mode.

Dispatch rejects missing routes, stale workflow revisions, changed node mappings, changed graph dependencies, and active-execution overflow before effects.

The `route_exact` authority mode grants only bindings created by the exact admitted workflow revision, reserved run, request, and route profile. The unavailable mode keeps effectful nodes denied.

## Durable admission

One redb transaction checks the request and idempotency indexes, reserves a global run ID, derives the acceptance identities, and stores the complete bounded execution record.

An exact duplicate request returns the stored acceptance. A reused idempotency key with different request material is rejected.

The worker marks the record running before workflow effects. A terminal update stores node states, receipt identities, and bounded fixed diagnostic logs.

## Process loss and recovery

An acceptance record survives a lost transport response. `RecoverDispatch` returns only that exact acceptance and never starts work.

A non-terminal record without a live worker in the current service process becomes `Unknown`. Observation and reconciliation preserve that non-terminal state. They do not infer failure or success and do not redispatch.

## Controls

Cancellation requires the exact current sequence. It records the request and signals the existing bounded executor cancellation flag.

Pause and resume return a closed rejection because the current headless runner cannot pause an active external effect safely.

## Logs

Terminal projection stores short fixed diagnostic bytes such as `completed` or `failed`. It never stores workflow outputs, commands, credentials, environment values, or provider payloads.

A log read returns one complete linked retained chunk. Missing, stale, or undersized requests fail closed.

## Command

`lattice workflow-exchange serve --profile <file.ncl>` loads the normal app config, opens the normal store, validates the handler profile, binds the Unix socket, and serves the finite transport connection limit.

## Test strategy

Positive tests cover exact route admission, dispatch, duplicate dispatch, acceptance recovery, terminal observation, cancellation, logs, reconciliation, and the Unix composition.

Negative tests cover malformed profiles, route drift, idempotency conflict, missing execution, stale sequence, unsupported pause, active limits, missing logs, and restart uncertainty.

## Non-claims

The handler proves only the checked local route, durable state, lifecycle, and bounded observation facts. It does not prove workflow correctness, sandboxing, external success, credential freshness, or production deployment.
