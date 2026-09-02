# Workflow Execution Specification

## ADDED Requirements

### Requirement: Cancellation requests are non-terminal

r[lattice.workflow_execution.cancellation_protocol.request] Lattice MUST distinguish a cancellation request from cancellation completion. A request MUST preserve exact active-run ownership until a terminal runner observation arrives.

#### Scenario: Active local run receives a cancellation request

- GIVEN a local workflow run owns a started worker and has no terminal observation
- WHEN an authorized caller requests cancellation
- THEN Lattice MUST signal that worker and stop new workflow work from starting
- AND it MUST NOT publish `Cancelled`, clear active ownership, or admit another run.

#### Scenario: Cancellation request is repeated

- GIVEN the exact active run is already cancellation-requested
- WHEN the caller repeats the same request
- THEN Lattice MUST treat the request as idempotent
- AND it MUST NOT create another terminal event, run record, or retry authority.

### Requirement: Terminal cancellation requires observed completion

r[lattice.workflow_execution.cancellation_protocol.completion] Lattice MUST publish `Cancelled` only after the exact runner reports cancellation and all started owned work stops. Required local teardown MUST complete before that report.

#### Scenario: Bounded child acknowledges cancellation

- GIVEN the exact active run requested cancellation and Bounded Exec owns a started child
- WHEN Bounded Exec returns `Completion::Cancelled` after process-group teardown and child reap
- THEN Lattice MAY classify the run as `Cancelled`
- AND it MAY release active ownership and admit a later fresh run.

#### Scenario: Normal completion wins the request race

- GIVEN the caller requests cancellation while the exact runner is completing normally
- WHEN Lattice receives a valid non-cancelled terminal observation first
- THEN it MUST preserve that observed terminal class
- AND it MUST NOT rewrite the result as `Cancelled` from request intent alone.

### Requirement: Missing completion remains explicit

r[lattice.workflow_execution.cancellation_protocol.uncertainty] A missing, stale, contradictory, failed, or cleanup-incomplete terminal observation MUST NOT become `Cancelled`. Lattice MUST preserve an explicit non-cancelled failure or unknown outcome without automatic retry.

#### Scenario: Blocking worker has not returned

- GIVEN a started blocking worker received a cancellation request but still owns execution
- WHEN TUI event polling runs before worker completion
- THEN the run MUST remain non-terminal and exclusively active
- AND Lattice MUST NOT persist terminal cancellation or admit another run.

#### Scenario: Completion channel closes without an observation

- GIVEN a cancellation-requested run loses its completion channel without a valid terminal observation
- WHEN Lattice classifies the run
- THEN it MUST record a safe failure or unknown outcome instead of `Cancelled`
- AND it MUST grant no automatic retry or cleanup-success claim.

### Requirement: Cancellation decisions preserve core and shell ownership

r[lattice.workflow_execution.cancellation_protocol.boundary] Request, observation, transition, terminal-classification, persistence-plan, replay-plan, and admission decisions MUST be pure over supplied facts. The shell MUST own task signalling, joining, process teardown, channels, clocks, storage, and terminal output.

#### Scenario: Equal cancellation facts are reduced twice

- GIVEN two equal in-memory cancellation states and observations
- WHEN the lifecycle reducer evaluates both inputs
- THEN both decisions MUST be equal
- AND the reducer MUST perform no task, process, filesystem, clock, persistence, network, or terminal effect.

#### Scenario: Choregraph receives cancellation

- GIVEN Lattice uses Choregraph to record local node lifecycle
- WHEN Lattice supplies the terminal `Cancel` event for started work
- THEN it MUST already hold an admitted local cancellation-completion observation
- AND Choregraph MUST NOT be treated as the owner of cancellation requests or cleanup.

### Requirement: Cancellation protocol evidence is adversarial

r[lattice.workflow_execution.cancellation_protocol.verification] Focused evidence MUST pair successful observed cancellation with negative premature, stale, duplicate, lost-channel, worker-failure, cleanup-failure, persistence-failure, and race cases.

#### Scenario: Cancellation behavior is proposed for acceptance

- GIVEN the reducer, runner shell, persistence path, replay path, docs, and fixtures are complete
- WHEN focused, workspace, Clippy, Octet, Cairn, PTY, and relevant Nix checks run
- THEN every positive case MUST publish its observed result
- AND every invalid or unobserved case MUST fail at its declared boundary.
