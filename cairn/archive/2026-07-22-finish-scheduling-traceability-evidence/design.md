# Design: finish scheduling traceability evidence

## Success contract

Maintained tests MUST directly establish terminal trigger-history timing and all required fields, complete equality between legacy and adapted due plans, and operator-process listener startup from persisted triggers through a real loopback request. All seven reviewed scheduling rows MUST become exact while coverage remains at 324 missing with no markers added.

## Functional core and shell

The schedule compatibility comparison exercises the existing pure due-plan evaluators. Trigger-history field construction remains deterministic; the headless shell owns execution and persistence. The listener test is an imperative integration shell: it allocates an OS loopback port, starts the real binary with an isolated store, sends bounded HTTP bytes, terminates the child through an owned drop guard, and inspects durable outcomes.

## Safety and bounds

The process test uses only loopback, named retry/deadline constants, an isolated temporary data directory, bounded request bytes, and owned child teardown. It does not expose credentials or depend on external network access.

## Validation

Positive and negative scheduling tests remain together. Completion validation checks all seven exact decisions, byte identities, absence of selected markers, unchanged missing coverage, and adversarial drift.

## Non-claims

Loopback observation does not prove remote networking, deployment, exactly-once external effects, release readiness, or production readiness.
