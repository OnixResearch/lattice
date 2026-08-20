# Design: Workflow dispatch recovery

## Success contract

Completion requires a producer-owned recovery command, exact request linkage, linked recovery outputs, negative fixtures, and all existing transport gates.

False completion includes automatic redispatch, recovery by a provider-native execution alone, an invented acceptance, or a fixture that bypasses normal admission.

## Flow

```text
complete dispatch write
  -> incomplete or invalid response
  -> client retains original admitted request
  -> RecoverDispatch(profile, request)
  -> application handler performs durable lookup
  -> exact linked WorkflowAccepted
```

## Decisions

The original `WorkflowRequest` is the recovery key. It already binds the request identity and idempotency key.

The transport validates the complete request before handler invocation. The handler receives no retry instruction.

An observation cannot replace the missing acceptance because the client cannot verify its workflow and execution linkage. Observation-only recovery fails closed.

Not-found is a closed rejection. It does not prove the dispatch was never accepted.

## Tests

Positive tests cover recovered acceptance. Negative tests cover changed requests, observation-only output, stale observations, not-found, and another incomplete recovery response.

## Non-claims

Recovery admission does not prove durable storage, non-execution, exactly-once behavior, provider availability, or terminal workflow outcome.
