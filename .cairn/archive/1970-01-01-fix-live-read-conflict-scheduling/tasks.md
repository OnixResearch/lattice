## Implementation

- [x] [serial] Treat live file/image observations as unresolved conflict participants while retaining pure control-flow computation and existing lane/alias behavior. r[lattice.workflow_execution.ready_frontier.live_reads]
- [x] [serial] Add product-candidate, active-reservation, pure-overlap, capacity/lane, and cancellation regressions. r[lattice.workflow_execution.ready_frontier.live_reads]
- [x] [serial] Add explicit-barrier subprocess fixtures using real capability-rooted file/image adapters, admitted same-resource aliases, dependencies, and pending-read cancellation. r[lattice.workflow_execution.ready_frontier.live_reads]
- [x] [serial] Document immutable computation versus live observation/mutation and the within-workflow external-writer boundary. r[lattice.workflow_execution.ready_frontier.live_reads]

## Verification handoff

- [x] [serial] Run focused ready-frontier integration and planner tests, formatting/lint checks, and native Cairn proposal, design and tasks gates after integration. r[lattice.workflow_execution.ready_frontier.live_reads]

Native Cairn proposal, design and tasks gates passed with no issues, and
repository validation passed all 49 specifications. `scripts/check.rs` and
`octet-precommit-check` passed. The x86_64-linux Nix flake check passed all 32
checks; Nextest ran 1,438 passing tests with 9 skipped. Octet retained 1,056 root
and 25 core accepted IDs with no new or stale IDs. These observations are not
an archive, release, other-platform or general concurrency-safety claim.
