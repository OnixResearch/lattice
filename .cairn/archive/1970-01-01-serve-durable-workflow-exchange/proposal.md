# Proposal: Serve the durable workflow exchange

## Why

Lattice publishes the bounded workflow exchange transport, but the application has no handler or service command.

Kiln can send valid requests, but no supported Lattice process can admit them, bind them to saved workflows, execute them, or return durable observations.

## What changes

- Add a typed Nickel handler profile that maps exact consumer graph identities to exact saved workflow revisions.
- Add durable request, idempotency, acceptance, execution, node, and bounded log records.
- Add an application `WorkflowExchangeHandler` that supports dispatch, acceptance recovery, observation, cancellation, log reads, and reconciliation.
- Execute only the reviewed saved workflow revision through the existing headless runtime.
- Add `lattice workflow-exchange serve --profile <file.ncl>` on Linux.
- Preserve unknown outcomes after process loss. Do not redispatch automatically.

## Impact

The change affects the root Lattice shell, redb storage, headless cancellation composition, the CLI, Nickel fixtures, tests, and workflow exchange documentation.

It does not change the independent transport wire contract.

## Non-claims

This change does not prove workflow correctness, host confinement, external effect success, credential freshness, peer intent, or release readiness.
