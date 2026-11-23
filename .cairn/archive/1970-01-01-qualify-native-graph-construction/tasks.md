## Implementation

- [x] [serial] Map every #20 and remaining #21 acceptance criterion to its production route and tests in `evidence/acceptance-matrix.json`. r[lattice.graph_construction.profile]
- [x] [serial] Verify independent instance identities, shared-handle fanout, nested and cross-instance resolution, attached configuration/effect/implementation facts and copy/drop rules. r[lattice.graph_construction.instances]
- [x] [serial] Compare the generated graph with an independent reference by ordered values and role-mapped required dependencies. r[lattice.graph_construction.reference]
- [x] [serial] Run the same constructed graph with no, foreign and exact local authority, and keep authoring, navigation, export and inspection inert. r[lattice.graph_construction.authority]
- [x] [serial] Reject generation-cap and native expansion overflow beside a fitting control. r[lattice.graph_construction.rejection]
- [x] [serial] Invalidate parameter, imported-member, evaluator and elaborator drift. r[lattice.graph_construction.drift]
- [x] [serial] Exercise snapshot cancellation, shell-result validation and orchestration budgets on a constructed zero-data sequence. r[lattice.graph_construction.runtime_controls]
- [x] [serial] Keep an in-flight run on its exact admitted snapshot while a newer source revision is admitted. r[lattice.graph_construction.snapshot]
- [x] [serial] Show template, instance and verified location through TUI and `lattice authoring origin`, and run the imported example through `lattice run` with an exact grant. r[lattice.graph_construction.origins]

## Qualification

- [x] [serial] Replace the unqualifiable sandboxed `construction-runtime` check with the `construction-runtime-tests` package and the host-run `construction-runtime-qualification` app, and document it. r[lattice.graph_construction.qualification]
- [x] [serial] Run the host app into a new output directory and retain its observations in `evidence/host-run-observation.json`. r[lattice.graph_construction.qualification]
- [x] [serial] Partition every captured-worker or v3 test behind `#[ignore]`, keep the sandboxed `nextest` check on the non-ignored partition without a worker, and prove captured evaluation fails closed with no worker. r[lattice.graph_construction.profile]
- [x] [serial] Record focused dev-shell observations, Clippy, formatting, native Cairn validation and the Octet comparison in `evidence/focused-observation.json`. r[lattice.graph_construction.compatibility]

The final `nix run .#construction-runtime-qualification` passed with zero
unexpected outcomes. An earlier script run over the same artifacts failed only
its derived run-status expectation (wrong case), since corrected.
