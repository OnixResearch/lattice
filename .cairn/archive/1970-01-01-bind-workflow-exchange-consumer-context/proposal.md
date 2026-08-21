# Bind workflow exchange consumer context

## Why

The durable workflow exchange executes an exact saved workflow, but consumer source facts are not available to that workflow. A CI workflow can therefore admit a request without receiving the exact repository and revision that caused it.

## What Changes

- Add an optional typed source context to the published workflow request contract.
- Bind source context into request identity and contract validation.
- Add a route policy that either rejects source context or injects it into exact mapped shell nodes.
- Derive and persist the instantiated workflow revision before any effect.
- Bind effect authority, acceptance, recovery, and observations to that instantiated revision.
- Add positive and negative contract, route, persistence, and execution tests.

## Impact

- **Contract**: request schema and identity framing advance to version 2; source context remains optional in version 2.
- **Application**: source-bound routes require exact source facts and reject unsafe targets or environment collisions.
- **Consumers**: source-aware consumers must pin the new immutable Lattice revision.
