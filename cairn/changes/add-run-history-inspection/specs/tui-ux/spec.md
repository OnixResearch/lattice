## ADDED Requirements

### Requirement: Run history inspection
r[n8nclone.tui_ux.run_history.persist] The system SHOULD persist workflow run records and expose recent runs for inspection in the TUI.

#### Scenario: Persist run record
r[n8nclone.tui_ux.run_history.persist_record]
- GIVEN a workflow run reaches a terminal state
- WHEN the run finishes, fails, or is canceled
- THEN the system stores a run record with summary status for later inspection

#### Scenario: List recent runs
r[n8nclone.tui_ux.run_history.list]
- GIVEN run records exist in the store
- WHEN the user opens run history
- THEN the system lists recent runs with enough summary information to choose one

#### Scenario: Inspect selected run
r[n8nclone.tui_ux.run_history.inspect]
- GIVEN the run history list is open
- WHEN the user selects a run
- THEN the system displays that run's summary without changing the current workflow

#### Scenario: Empty or unavailable history
r[n8nclone.tui_ux.run_history.empty_error]
- GIVEN no run history is available or storage returns an error
- WHEN the user opens run history
- THEN the system displays a clear message and keeps the TUI usable
