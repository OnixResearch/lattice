# Design: finish agent-runtime evidence

## Success contract

All 14 reviewed agent-runtime rows MUST be exact and current-byte-bound while coverage remains 304 missing with zero dangling markers.

## Functional core and shell

The pure planning core accepts an explicit program identity contract for drift tests; the production wrapper supplies only built-in constants. Shell tests observe transport-call counts and map non-final runtime reports to node errors. Integration tests observe immutable requests, redaction, typed ordinary output, metadata omission, and preflight zero dispatch.

## Non-claims

The evidence is bounded to the fixed Steel program and maintained adapters. It does not prove arbitrary model truth, provider behavior, external cancellation, orchestration correctness, release readiness, or production readiness.
