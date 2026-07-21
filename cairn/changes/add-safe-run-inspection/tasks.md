## Phase 1: Safe inspection core

- [x] [serial] Define a serialization-stable run inspection report containing only bounded identifiers, enums, booleans, timestamps, counts, budget dimensions, and diagnostic codes. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Project run details into node status/count evidence without labels, values, or error text. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Validate present run replay records against exact run identity and existing replay invariants while distinguishing absent historical evidence. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Correlate orchestration-owned runs to one exact instance/dispatch/run tuple and derive dispatch, recovery, abandonment, and budget evidence. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Adversarially test malformed detail identity, foreign ownership, invalid run replay, invalid orchestration replay, duplicate dispatch admission, and private-content omission. r[lattice.workflow_execution.safe_run_inspection]

## Phase 2: Read-only shell and CLI

- [x] [serial] Add a thin store shell that loads one persisted run plus optional details/replay and exact orchestration evidence without mutating state. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Add `runs show <run-id> --output text|json` with successful rendering, explicit not-found failure, and unsuccessful exit for invalid present evidence. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Add end-to-end CLI fixtures for unscoped and orchestration-owned runs and assert that prompt, reasoning, credential, output, error, actor, objective, and trigger-key markers never appear. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Document the command, evidence states, and non-claim boundaries. r[lattice.workflow_execution.safe_run_inspection]

## Phase 3: Verification and lifecycle

- [ ] [serial] Run focused post-change tests, repository checks, Octet comparison, Cairn gates/validation, and Nix checks; sync the accepted spec and archive the change with evidence. r[lattice.workflow_execution.safe_run_inspection]

## Baseline evidence

- Before implementation, `nix develop -c cargo test --test test_run_history_details --test test_orchestration_cli --test test_store` passed all selected suites; the final visible suite reported 29 passing store cases with zero failures.
- No active Cairn changes or pre-existing worktree modifications were present.
