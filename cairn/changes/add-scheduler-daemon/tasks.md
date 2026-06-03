## Phase 1

- [ ] [serial] r[n8nclone.scheduling.schedule_model] Add persistent schedule definitions for saved workflows.
- [ ] [serial] r[n8nclone.scheduling.due_evaluator] Implement pure due-run evaluation from schedule state and current time.
- [ ] [serial] r[n8nclone.scheduling.daemon.run_due] Add a daemon shell that starts due workflow runs through the headless runner.
- [ ] [parallel] r[n8nclone.scheduling.overlap_policy] Prevent overlapping scheduled runs unless explicitly configured.
- [ ] [parallel] r[n8nclone.scheduling.missed_runs] Add restart and missed-run tests with deterministic clock adapters.
