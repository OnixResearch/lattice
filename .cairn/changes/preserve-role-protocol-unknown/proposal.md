## Why

https://github.com/OnixResearch/lattice/issues/17 identifies partially dispatched fanout and generic HTTP failures that can erase durable uncertainty. Its receiver-ledger/history extension requires an independent oracle rather than success-shaped traces or agent summaries.

## What Changes

- Select every exact recipient adapter/profile before any adapter starts, retaining current authority and persistence-before-effects.
- Classify generic non-2xx and failure-shaped adapter responses as unknown without an admitted receiver rejection.
- Restrict failed-before-dispatch to Prepared/Authorized; preserve blocked mixed accepted/rejected fanout to prevent blind repetition of accepted recipients.
- Define deterministic independent receiver-ledger, response/history matrix, process-restart, exact-observer and broken-control regressions, with stable cross-attempt logical-operation checking, consumed summary intents and independently persisted per-recipient no-start rejection evidence.
- Document the observer trust boundary and immutable published-profile non-claim.

## Success Criteria

Every #17 criterion is represented by the focused regression target: multi-recipient projection, preflight missing/mismatch, HTTP 500/502 and 202, genuine no-start rejection, exact per-message reconciliation, wrong/cross/stale/substituted observations, persisted restart/replay, summary invariance, duplicate-risk recovery and independent checker controls.

## False Completion

Tests written are not tests passed. A response body or handoff is not receiver evidence. No locally invented factors profile can resolve https://github.com/OnixResearch/lattice/issues/16 while https://github.com/OnixResearch/choregraph/issues/4 has not published an admitted cohort.

## Impact

Only protocol shell/recovery, focused protocol tests, topic documentation and this scoped lifecycle change. No transport backend, scheduler, solver, profile repin, authority expansion or cross-repository edit.

## Affected Specs

- role-protocol-execution: uncertainty and deterministic qualification.
