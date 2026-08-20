# Proposal: Add workflow dispatch recovery

## Problem

A client can finish writing a dispatch request and lose the complete response. The current recovery operation requires `WorkflowAccepted`, which the client does not have in this failure window.

## Outcome

Add a closed dispatch-recovery command keyed by the original admitted workflow request. A handler can look up durable acceptance by request identity and idempotency key without redispatching work.

## Scope

- Add a `RecoverDispatch` exchange command.
- Admit the original profile and workflow request before handler access.
- Accept only the exact linked acceptance as recovery output.
- Add an explicit not-found rejection class.
- Add positive and negative recovery tests and producer fixtures.
- Keep retry and execution authority outside the transport.

## Exclusions

- No automatic retry.
- No in-memory fallback.
- No claim that the handler has durable state.
- No conversion of not-found or disconnect into terminal failure.
