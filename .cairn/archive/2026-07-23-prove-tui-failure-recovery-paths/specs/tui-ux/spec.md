# TUI UX Delta

## ADDED Requirements

### Requirement: Process-boundary TUI failure recovery
r[lattice.tui_ux.process_failure_recovery] Lattice MUST keep workflow failure, validation diagnostics, explicit rerun, active-run cancellation, and terminal resize paths usable through the real PTY process boundary, and MUST preserve durable run evidence without automatic retry or historical mutation.

#### Scenario: Failed run remains inspectable and explicitly rerunnable
r[lattice.tui_ux.process_failure_recovery.scenario.rerun]
- GIVEN a saved workflow's first real TUI run reaches a deterministic node failure
- WHEN the operator opens run history, inspects the failed node, and explicitly requests rerun
- THEN the TUI MUST remain responsive, MUST start one fresh run identity, MUST preserve the failed historical record unchanged, and MUST display and persist the second run's independent outcome

#### Scenario: Fatal validation prevents effects
r[lattice.tui_ux.process_failure_recovery.scenario.diagnostics]
- GIVEN a saved workflow has a fatal validation diagnostic and a potential observable effect
- WHEN the operator requests execution and opens diagnostics in the real TUI
- THEN Lattice MUST display the bounded diagnostic, MUST start no workflow effect or run record, and MUST continue accepting overlay-close and navigation input

#### Scenario: Active cancellation prevents pending work
r[lattice.tui_ux.process_failure_recovery.scenario.cancel]
- GIVEN a real TUI workflow has a bounded child process that reported readiness and has pending work not yet performed
- WHEN the operator requests active-run cancellation
- THEN Lattice MUST stop pending workflow work, MUST render and persist the bounded cancellation outcome, MUST prevent the pending observable effect, and MUST keep history and quit controls usable

#### Scenario: Resize preserves interaction
r[lattice.tui_ux.process_failure_recovery.scenario.resize]
- GIVEN the real TUI has an active workspace or modal surface
- WHEN the PTY is resized below the surface's preferred dimensions and later restored
- THEN Lattice MUST render without panic or overflow, MUST preserve the current workflow and modal/navigation state, and MUST continue handling keyboard input after each resize

#### Scenario: PTY evidence remains bounded
r[lattice.tui_ux.process_failure_recovery.scenario.non-claim]
- GIVEN the process-boundary TUI matrix passes on a supported Unix/Linux host
- WHEN the result is reported as continuous evidence
- THEN it MUST NOT claim byte-identical rendering across terminal emulators, unsupported-platform behavior, arbitrary child cleanup, workflow semantic correctness, or production readiness
