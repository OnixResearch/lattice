# TUI UX Specification

## ADDED Requirements

### Requirement: TUI cancellation has a visible non-terminal phase

r[lattice.tui_ux.cancellation_lifecycle] The TUI MUST show `Cancelling` after an active-run cancellation request. It MUST keep the run active until terminal classification.

#### Scenario: Operator requests active-run cancellation

- GIVEN the TUI owns one active workflow run
- WHEN the operator requests cancellation
- THEN the TUI MUST remain responsive and display a non-terminal cancelling state
- AND it MUST retain the exact handle, receiver, workflow, and run identity.

#### Scenario: Operator starts another run while cancellation is pending

- GIVEN the current run is cancellation-requested or cancelling
- WHEN the operator requests another workflow run
- THEN the TUI MUST reject the new run without replacing current ownership.

### Requirement: TUI persists only classified terminal cancellation

r[lattice.tui_ux.cancellation_persistence] The TUI MUST persist one cancelled run, detail record, and replay terminal only after it admits the exact runner completion. Request intent alone MUST create no terminal record.

#### Scenario: Runner returns observed cancellation

- GIVEN the exact cancelling run returns a valid cancellation observation after required teardown
- WHEN the TUI applies the terminal decision
- THEN it MUST persist exactly one cancelled summary, detail record, and replay terminal
- AND it MUST release active ownership only after that decision.

#### Scenario: Runner completion is unavailable

- GIVEN the exact cancelling run has no valid terminal observation
- WHEN the TUI handles channel loss, task failure, or cleanup uncertainty
- THEN it MUST NOT persist a cancelled summary or replay terminal
- AND any persisted result MUST identify a non-cancelled failure or unknown outcome.

### Requirement: Cancelling runs preserve operator usability

r[lattice.tui_ux.cancellation_usability] A cancelling run MUST preserve rendering, history inspection, diagnostics, resize, and explicit quit controls. These controls MUST NOT weaken active-run exclusivity.

#### Scenario: Operator inspects history during cancellation

- GIVEN one run remains in the cancelling phase
- WHEN the operator opens history or diagnostics and resizes the terminal
- THEN the TUI MUST remain usable and preserve current run ownership
- AND it MUST admit no second workflow run.

### Requirement: TUI cancellation evidence covers task and process boundaries

r[lattice.tui_ux.cancellation_verification] TUI tests MUST cover a controlled started blocking worker and a real bounded child process. They MUST pair terminal success cases with premature, uncertain, and failure cases.

#### Scenario: Process-boundary cancellation rail runs

- GIVEN a bounded child reports readiness and pending observable work remains
- WHEN the PTY requests cancellation
- THEN pending work MUST NOT start
- AND terminal cancellation MUST appear only after the child is reaped and the runner reports completion.

#### Scenario: Blocking worker delays completion

- GIVEN a controlled started blocking worker cannot complete yet
- WHEN the TUI requests cancellation and polls events
- THEN the test MUST observe `Cancelling`, no terminal record, and no fresh-run admission
- AND only a later terminal observation MAY close the run.
