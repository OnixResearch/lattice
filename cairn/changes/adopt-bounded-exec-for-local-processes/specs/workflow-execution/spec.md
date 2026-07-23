## ADDED Requirements

### Requirement: Explicit bounded process policy
r[lattice.workflow_execution.bounded_process_policy] Lattice MUST construct every local process request from explicit executable, argument, environment, working-directory, stdin, stdout, stderr, deadline, polling, teardown, termination-scope, accepted-exit, and truncation policy values and MUST reject malformed or unsupported bounds before spawn.

#### Scenario: Valid process policy is admitted
r[lattice.workflow_execution.bounded_process_policy.scenario.valid]
- GIVEN a local process route has a resolvable executable, absolute working directory, bounded input, unique environment entries, and positive supported limits
- WHEN Lattice builds the process request
- THEN it MUST produce one deterministic admitted plan containing every required process and outcome-policy field

#### Scenario: Malformed process policy fails before spawn
r[lattice.workflow_execution.bounded_process_policy.scenario.invalid]
- GIVEN a local process route has a missing executable, invalid working directory, duplicate environment name, oversized stdin, zero deadline, zero stream limit, zero polling interval, or zero teardown limit
- WHEN Lattice evaluates process admission
- THEN it MUST reject the request before spawn with a bounded product diagnostic

### Requirement: Shared bounded process mechanics
r[lattice.workflow_execution.bounded_process_mechanics] Lattice MUST execute shell nodes and other production local-process adapters through one reviewed bounded mechanism that drains stdout and stderr concurrently, retains only configured prefixes, enforces deadlines and cancellation, and tears down owned processes before returning.

#### Scenario: Bounded process completes normally
r[lattice.workflow_execution.bounded_process_mechanics.scenario.success]
- GIVEN an admitted process writes output within both stream bounds and exits with an accepted status before its deadline
- WHEN the shared process shell runs it
- THEN Lattice MUST return the complete bounded observation for consumer-owned parsing and typing

#### Scenario: Output flood fails without unbounded retention
r[lattice.workflow_execution.bounded_process_mechanics.scenario.output-flood]
- GIVEN an admitted process writes stdout or stderr beyond the configured stream bound
- WHEN the shared process shell drains that stream
- THEN it MUST retain no more than the configured prefix, report truncation, fail the affected consumer before parsing, and MUST NOT allocate storage proportional to the complete output

#### Scenario: Timeout or cancellation tears down owned work
r[lattice.workflow_execution.bounded_process_mechanics.scenario.teardown]
- GIVEN an admitted process or its descendant remains active past the deadline or receives cancellation
- WHEN bounded execution stops the request
- THEN Lattice MUST tear down the owned process scope and bounded capture workers before reporting timeout or cancellation

#### Scenario: Runner infrastructure failure is node scoped
r[lattice.workflow_execution.bounded_process_mechanics.scenario.infrastructure]
- GIVEN spawn, pipe, wait, kill, reap, capture, or worker teardown fails
- WHEN the shared shell translates the runner error
- THEN Lattice MUST fail only the affected node or provider with a bounded diagnostic and MUST NOT expose raw output or credential values

### Requirement: Existing process-route compatibility
r[lattice.workflow_execution.bounded_process_compatibility] Lattice MUST preserve existing shell interpretation, explicit environment overrides, stdin routing, working-directory behavior, redaction, typed outputs, cache identity, graph lifecycle, and product diagnostics when process execution remains within admitted bounds.

#### Scenario: Existing shell node remains compatible
r[lattice.workflow_execution.bounded_process_compatibility.scenario.shell]
- GIVEN an existing shell node uses the configured shell, environment, working directory, stdin connection, timeout, and typed output ports within the new bounds
- WHEN it executes through the shared process mechanism
- THEN it MUST produce the same admitted stdout, stderr, exit-code, redaction, cache, and graph behavior

#### Scenario: Process bounds do not imply sandbox authority
r[lattice.workflow_execution.bounded_process_compatibility.scenario.non-claim]
- GIVEN a local process completes under every configured bound
- WHEN Lattice reports its result
- THEN it MUST NOT claim sandboxing, hermeticity, executable authorization, filesystem confinement, network isolation, child correctness, remote exactly-once execution, or release readiness
