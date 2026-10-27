## Phase 1: Producer correction

- [x] [serial] Require successful shell exits in immutable exchange snapshots and persist failure before dependent dispatch. r[lattice.workflow_interchange.shell_success]
- [x] [serial] Add paired successful and nonzero-exit application regressions, including durable status and dependent-effect suppression; preserve ordinary shell exit-as-data behavior. r[lattice.workflow_interchange.shell_success.scenario.zero] r[lattice.workflow_interchange.shell_success.scenario.nonzero]

## Phase 2: Consumer evidence

- [x] [serial] Exercise signed GitHub push and PR events through actual Kiln and Lattice execution, with successful and failing exact-source commands and bounded result publication. r[lattice.workflow_interchange.shell_success]
- [x] [serial] Run required producer checks, update the application contract and documentation, and record bounded evidence. r[lattice.workflow_interchange.shell_success]
