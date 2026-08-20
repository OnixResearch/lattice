# Durable workflow exchange application

Lattice can serve the published workflow exchange transport through its normal store and headless runtime.

## Command

```console
lattice --config ./config.ncl workflow-exchange serve \
  --profile ./workflow-exchange-handler.ncl
```

The command is available on Linux. It serves the finite connection limit in the handler profile and then waits for admitted local workers.

## Handler profile

`contracts/workflow-exchange-handler-profile.ncl` defines the typed profile.

Each route binds these values:

- one workflow contract profile identity;
- one consumer graph identity;
- one saved Lattice workflow name;
- one exact saved workflow revision identity;
- one ordered consumer-node to Lattice-node map;
- one effect-authority mode.

The profile also contains the Unix socket path, peer user IDs, transport limits, and active execution limit.

Routes must be sorted by profile identity and graph identity. Route nodes must be sorted by consumer node ID.

The command rejects a changed saved workflow revision, changed node operation identity, changed dependency graph, invalid profile, missing workflow, and active execution overflow before effects.

## Effect authority

`unavailable` keeps the normal fail-closed authority source. Effectful nodes cannot run.

`route_exact` grants only effect bindings created by the exact saved workflow revision, request, reserved run, and handler profile.

It does not grant path roots. File effects still need an approved configured-root adapter.

The route profile contains no command, credential, environment value, or provider payload.

## Durable admission

One redb transaction stores the request and acceptance before the worker starts.

An exact duplicate dispatch returns the stored acceptance. A changed request cannot reuse the same idempotency key.

`RecoverDispatch` returns the exact stored acceptance. It does not start a worker.

## Lifecycle

A worker stores `Running` before it enters the headless runtime. Terminal results retain node and execution receipt identities.

Cancellation records the request and signals the bounded executor cancellation flag. A cancellation request does not by itself prove cancellation.

Pause and resume fail closed. The current headless runtime cannot pause an active external effect safely.

A non-terminal record without a live worker in the current service process becomes `Unknown`. Observation and reconciliation do not infer success or failure, and they do not redispatch.

## Logs

The handler retains only fixed terminal diagnostic chunks:

- `completed`
- `failed`
- `cancelled`
- `skipped`

It does not retain workflow outputs, commands, credentials, environment values, or provider payloads.

A log read must name the exact node, diagnostic stream, cursor, execution, and complete byte bound.

## Setup sequence

1. Import and review the saved Lattice workflow.
2. Get its exact workflow revision identity.
3. Build the consumer request graph and operation identities.
4. Write the sorted handler route with the exact workflow node IDs.
5. Type-check and export the Nickel profile.
6. Start the service under an operator-selected process manager.
7. Configure the consumer with the same socket path and transport profile.

A changed workflow or consumer graph requires a new route.

## Non-claims

This service proves checked local route, durability, lifecycle, and bounded observation facts only.

It does not prove workflow correctness, host confinement, external effect success, credential freshness, peer intent, or release readiness.
