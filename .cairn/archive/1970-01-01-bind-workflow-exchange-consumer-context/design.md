# Design: Source-bound workflow exchange execution

## Goal and success contract

A source-bound exchange request must execute one exact saved workflow instantiated with the admitted provider, repository, and revision facts.

Completion requires these results:

- request identity changes when any source fact changes;
- a route explicitly selects `unavailable` or `shell_environment` source binding;
- `unavailable` rejects supplied source context;
- `shell_environment` requires source context and exact mapped shell nodes;
- pre-existing reserved environment keys reject admission;
- the instantiated workflow revision is computed before durable acceptance;
- acceptance and effect authority bind that instantiated revision; and
- restart recovery never reconstructs or redispatches an unstored request.

## Functional core

The contract core validates and frames source context. The application core validates route policy and creates an instantiated workflow value without I/O.

The binding uses fixed keys:

- `LATTICE_SOURCE_PROVIDER`
- `LATTICE_SOURCE_REPOSITORY`
- `LATTICE_SOURCE_REVISION`

A route cannot select arbitrary environment names. A saved workflow cannot predefine these keys.

## Imperative shell

The handler loads the saved workflow, validates its base revision, creates the source-bound snapshot, computes its revision, and commits acceptance before starting a worker.

The worker executes only that committed snapshot. Authority uses the instantiated revision, not the saved template revision.

## Non-claims

The binding does not prove source trust, revision availability, checkout correctness, workflow correctness, host confinement, or external effect success.
