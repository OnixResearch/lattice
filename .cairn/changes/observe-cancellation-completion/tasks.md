# Tasks

## Phase 1: Baseline and lifecycle core

- [ ] [serial] Record focused TUI, runner, bounded-process, persistence, replay, and PTY results before core changes. r[lattice.workflow_execution.cancellation_protocol.verification]
- [ ] [serial] Define cancellation request, cancelling, observed terminal, unknown, and cleanup-failed facts in a narrow pure module. r[lattice.workflow_execution.cancellation_protocol.boundary]
- [ ] [serial] Implement deterministic transition, terminal-classification, persistence-plan, replay-plan, and new-run admission decisions. r[lattice.workflow_execution.cancellation_protocol.request] r[lattice.workflow_execution.cancellation_protocol.completion] r[lattice.workflow_execution.cancellation_protocol.uncertainty]
- [ ] [parallel] Add positive reducer tests and negative stale, duplicate, impossible-order, terminal-mutation, race, unknown, and cleanup-failure tests. r[lattice.workflow_execution.cancellation_protocol.verification]

## Phase 2: Runner and TUI shell

- [ ] [serial] Change active-run cancellation into a request that retains the handle, receiver, workflow, and exact run identity. r[lattice.tui_ux.cancellation_lifecycle]
- [ ] [serial] Remove outer-task abort as the completion mechanism for an already-started blocking worker. r[lattice.workflow_execution.cancellation_protocol.request]
- [ ] [serial] Send one typed terminal observation after executor return and required local teardown. r[lattice.workflow_execution.cancellation_protocol.completion]
- [ ] [serial] Keep the TUI responsive, show `Cancelling`, and deny another run while cancellation remains non-terminal. r[lattice.tui_ux.cancellation_lifecycle] r[lattice.tui_ux.cancellation_usability]
- [ ] [serial] Persist run, detail, and replay records exactly once after terminal classification. r[lattice.tui_ux.cancellation_persistence]
- [ ] [serial] Map channel loss, wrapper failure, missing teardown, and persistence failure to explicit non-cancelled outcomes without automatic retry. r[lattice.workflow_execution.cancellation_protocol.uncertainty]

## Phase 3: Integration evidence

- [ ] [parallel] Add a controlled `spawn_blocking` fixture that rejects terminal cancellation and fresh-run admission before worker completion. r[lattice.workflow_execution.cancellation_protocol.verification]
- [ ] [parallel] Add a bounded child-process fixture that observes cancellation only after process-group teardown and child reap. r[lattice.workflow_execution.cancellation_protocol.verification]
- [ ] [parallel] Add completion-race, duplicate-request, stale-event, channel-loss, cleanup-failure, and persistence-failure fixtures. r[lattice.workflow_execution.cancellation_protocol.verification] r[lattice.tui_ux.cancellation_verification]
- [ ] [serial] Extend the PTY matrix with visible cancelling, blocked fresh-run admission, pending-effect prevention, child reap, terminal persistence, and usable history controls. r[lattice.tui_ux.cancellation_verification] r[lattice.tui_ux.cancellation_usability]
- [ ] [serial] Audit local, component, agent, credential, orchestration, and interchange adapters for request-versus-completion wording. r[lattice.workflow_execution.cancellation_protocol.boundary]
- [ ] [serial] Document the lifecycle, recovery path, Choregraph observation boundary, and non-claims. r[lattice.workflow_execution.cancellation_protocol.boundary] r[lattice.tui_ux.cancellation_persistence]
- [ ] [serial] Run formatting, focused tests, workspace tests, Clippy, Octet, Cairn gates, the evidence guard, and relevant Nix checks. r[lattice.workflow_execution.cancellation_protocol.verification] r[lattice.tui_ux.cancellation_verification]
