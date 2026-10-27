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
- one effect-authority mode;
- one source-context binding mode.

The profile also contains the Unix socket path, peer user IDs, transport limits, and active execution limit.

Routes must be sorted by profile identity and graph identity. Route nodes must be sorted by consumer node ID.

The command rejects a changed saved workflow revision, changed node operation identity, changed dependency graph, invalid profile, missing workflow, and active execution overflow before effects.

## Consumer source context

A request can carry one typed provider, repository, and revision source context. Request identity binds all three fields.

`unavailable` routes reject source context. `shell_environment` routes require it and inject these fixed keys into every exact mapped shell node:

- `LATTICE_SOURCE_PROVIDER`
- `LATTICE_SOURCE_REPOSITORY`
- `LATTICE_SOURCE_REVISION`

The saved workflow must not predefine those keys. A non-shell target or key collision rejects before durable acceptance.

Lattice derives the instantiated workflow revision after injection. Acceptance, durable state, and effect authority bind that revision.

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

A worker stores `Running` before it enters the headless runtime. Executed terminal nodes and the terminal execution retain receipt identities. Skipped nodes have no execution receipt, as required by the published observation contract.

Immutable exchange snapshots require every mapped shell command to exit zero
before completing that node successfully. A nonzero or unavailable successful
exit status retains a failed node, skips dependent effects, and produces a failed
terminal observation with `successful = false`. Ordinary graph execution outside
this exchange snapshot path retains its existing exit-as-data behavior.

Cancellation records the request and signals the bounded executor cancellation
flag. A cancellation request does not by itself prove cancellation. Canonical
`Cancelled` requires the owned local process runner's observed cancellation
disposition, retained separately from redacted adapter errors. Once that owned
execution stops, the exchange retains terminal `Cancelled`, `successful = false`,
and cancellation receipts; pending dependent effects do not execute. The ordinary
run-detail adapter error can remain `Failed`; it is not the canonical exchange
outcome and is never parsed to infer cancellation.

If local success or failure completes before cancellation is observed, its
ordinary completed/failed outcome wins even if cancel intent was accepted before
the exchange completion was persisted. A late request flag cannot rewrite it as
`Cancelled`. Unknown/lost workers, persistence failures, timeouts and remote
effects retain their existing recovery semantics; no remote cancellation or
automatic redispatch is implied. Consumers need a corrected producer executable,
not a contract-import refresh or consumer-side outcome relabeling.

The compiler prerequisites are repaired without disabling Tile, component
execution or asynchronous CI. Tile remains an exact Git dependency; the shared
runtime, Mantle verifier/build contract and Basalt use native imported source
cohorts with repository-owned `SOURCE` provenance. The component shell calls the
actual named byte-method carrier and typed file/source-tree verifier rather
than inventing the former `component::{Profile, Runtime, Prepared}` API.
Valence validation remains the actual separately pinned producer validator.

The complete application binary builds with all those integrations enabled.
The started-child cancellation, dependent-effect and completion-before-cancel
regressions pass. Kiln's unchanged strict native web-service exercise also
passes for the measured immutable executable: ordinary success/failure,
started-command cancellation, restart without redispatch, logs and real artifact
transfer. `fix-observed-workflow-exchange-cancellation` retains the exact
cohort and consumer receipt reference in `evidence/integrated-verification.json`.
The producer's broader repository checks are not all green: nine library tests,
three TUI PTY cases and Octet baseline findings remain recorded separately.
Historical compiler failures remain in `evidence/verification.json`; no consumer
relabeling, contract-import refresh, publication, lifecycle acceptance or
production activation follows from this local qualification.

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
4. Write the sorted handler route with the exact workflow node IDs and source-binding mode.
5. For `shell_environment`, make each mapped node a shell node that consumes the fixed source keys.
6. Type-check and export the Nickel profile.
7. Start the service under an operator-selected process manager.
8. Configure the consumer with the same socket path and transport profile.

A changed workflow or consumer graph requires a new route.

## Non-claims

This service proves checked local route, durability, lifecycle, and bounded observation facts only.

It does not prove workflow correctness, host confinement, external effect success, credential freshness, peer intent, or release readiness.
