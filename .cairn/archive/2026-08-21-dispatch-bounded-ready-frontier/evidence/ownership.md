# Ready-frontier integration ownership

<!-- r[verify lattice.workflow_execution.ready_frontier.integration] -->

## Choregraph

Choregraph runtime revision `0b2000edaf86d5a7a8601246afbc0d526f4a2a07` owns graph admission, dependency readiness, and lifecycle transitions. Choregraph revision `b3e08e19750f53bdbcae970cdf58a47a791ed20b` owns the accepted readiness-completeness proof evidence.

Lattice does not claim direct proof linkage between those revisions. It compares each product runtime frontier with its compatibility projection and stops on a mismatch.

## Lattice

Lattice owns total capacity, named resource lanes, immutable input preparation, credentials, current authority, backend routes, attempt reservation, conflict aliases, effect dispatch, completion observation, persistence, and recovery.

The pure planner does not perform I/O or authorize effects. The shell executes only selected reservations.

## Memoization

Cache identity remains node, configuration, credential-reference, input, and admitted expression-value identity. Scheduling order is not a cache-key input.

A cache hit becomes an observed completion. It passes through the same Choregraph completion path before readiness is recomputed.

## Backend and attempts

Operation plans retain the admitted backend route. The operation shell retains attempt identity, durable reservation, bounded dispatch, outcome classification, and unknown-outcome recovery.

Ready-frontier capacity does not create an attempt, authorize a backend, or retry an uncertain effect.

## Conflicts

A caller can supply an explicit resolved resource alias before execution. Equal aliases cannot overlap. Missing aliases are unresolved and cannot overlap another effect.

Pure work can overlap an unresolved effect because it has no external mutation authority.

## Ordering

Canonical results remain graph ordered. Observed terminal order is separate and persists in `RunDetailRecord.observed_completion_order`.
