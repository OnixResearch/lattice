## Phase 1: Producer implementation

- [x] [serial] Restore the exact dependency/runtime integration needed to compile the existing Tile and component shells without deleting functionality or adding sibling-path overrides. r[lattice.workflow_interchange.observed_cancellation]
- [x] [serial] Preserve typed owned cancellation through runtime facts into pure completion and retain completion winners, r[lattice.workflow_interchange.observed_cancellation].
- [x] [serial] Add real started-child/dependent-effect and completion-race regressions, r[lattice.workflow_interchange.observed_cancellation.scenario.active] and r[lattice.workflow_interchange.observed_cancellation.scenario.race].
- [x] [serial] Document operator semantics and run integrated focused/producer checks against the measured immutable cohort, r[lattice.workflow_interchange.observed_cancellation].

## Verification status

The complete Lattice application binary now builds with its Tile, native shared
runtime, typed Mantle verifier/private Build API, Basalt and actual Valence
integrations enabled. Native snapshots are imported through the owned sync
script with exact source revisions; no sibling dependency override, fabricated
runtime identity, file-hash substitute or disabled integration is used.

The real started-child cancellation regression and completion-before-cancel
regression pass. One initial downstream-success assertion failed without a node
diagnostic; the diagnostic-enhanced run and 32 bounded repetitions pass. That
earlier observation is retained rather than erased or relabeled.

The all-target check, all 12 workflow-exchange library tests, 33 focused native
component/async-CI integration tests and the real component CLI exercise pass.
The CLI retains successful post-return/cleanup facts and rejects source-tree
drift. Kiln's unchanged strict native exercise passes for the frozen complete
executable, including started-command `Cancelled`, ordinary success/failure,
restart without redispatch, real logs and artifact transfer.

Broader gates are not green: the complete library run has 253 passes and nine
failures; the repository script stops at three TUI PTY failures; Octet reports
1,335 new and 849 stale baseline findings. The immutable Nix check reaches the
build but fails Octet's external-vendor diagnostic-path admission (`..`).
`evidence/integrated-verification.json` binds these outcomes and the exact
consumer receipt. No baseline refresh, acceptance, archive, publication,
production activation or live GitHub/OIDC deployment is claimed.
`evidence/verification.json` retains the historical 55-error and later separate
40/42-build-unit compiler failures and prior Tile exercise; it is not a current
failure verdict for the corrected application.
