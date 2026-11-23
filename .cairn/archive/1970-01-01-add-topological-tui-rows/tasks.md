## Implementation

- [x] [serial] Replace numeric depth-local lanes with deterministic topological frontier connectors and bounded overflow while retaining every node. r[lattice.tui_ux.rows.topology] r[lattice.tui_ux.rows.wide]
- [x] [serial] Preserve graph/rows selection, implement a selection-following viewport and scrolled hit testing, and isolate monitoring gestures from graph mutation. r[lattice.tui_ux.rows.selection]
- [x] [serial] Publish real post-admission per-node results and reduce live state/start/duration/attempts/output/usage without final-event duplication. r[lattice.tui_ux.rows.metadata]
- [x] [serial] Add linear, fork-join, diamond snapshots and rows-specific metadata/viewport/selection regressions; document controls and evidence boundaries. r[lattice.tui_ux.rows.topology] r[lattice.tui_ux.rows.metadata]

## Integration evidence

- [x] [serial] Parent runs rows, reducer, input and producer regressions after shared integration and records observed results. r[lattice.tui_ux.rows.topology] r[lattice.tui_ux.rows.metadata] r[lattice.tui_ux.rows.selection] r[lattice.tui_ux.rows.wide]
- [x] [serial] Parent performs the real-terminal graph/rows toggle and synchronized selected-node surface check without executing workloads. r[lattice.tui_ux.rows.selection]
- [x] [serial] Record coordinated repository and native Cairn lifecycle gates without inferring acceptance from source checks. r[lattice.tui_ux.rows.selection]

Checked implementation items record source and fixtures; checked integration
items record observed runs. Native Cairn proposal, design and tasks gates passed
with no issues, and repository validation passed all 49 specifications.
`scripts/check.rs` and `octet-precommit-check` passed. The x86_64-linux Nix flake
check passed all 32 checks; Nextest ran 1,438 passing tests with 9 skipped.
Octet retained 1,056 root and 25 core accepted IDs with no new or stale IDs.
No archive, other-platform qualification or release acceptance is claimed.
