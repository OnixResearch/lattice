## Phase 1: Source repair

- [x] [serial] Diagnose the minimal imported-container merge at the actual core invariant and compare the pinned external CLI value. r[lattice.nickel_import_merge.values]
- [x] [serial] Prepare the owned evaluator source repair preserving existing thunk reversion and constant allocation behavior, after lifecycle admission. r[lattice.nickel_import_merge.values]
- [x] [serial] Admit exact immutable patched evaluator source and align Cargo/Nix integration without changing the language cohort or adding fallbacks. Adopted published nickel-eval `f0ea0d2a31d3482ddf4e575482f6ddf7a7e7d4e9`; the consumer snapshot and root patch are removed (evidence/adopted-producer-20261118.json). r[lattice.nickel_import_merge.boundary]

## Phase 2: Consumer evidence

- [x] [parallel] Add complete-value native regressions for both operand orders, nested overlap, and recursive overriding. r[lattice.nickel_import_merge.values]
- [x] [parallel] Add malformed/missing import and contract rejection regressions asserting content-omitting safe categories. r[lattice.nickel_import_merge.boundary]
- [x] [serial] Exercise the admitted native path, compare complete JSON value with the pinned CLI, and run focused regressions. Adapter regressions passed in the lib suite and the pinned 1.17.0 CLI exported identical canonical values for all three cases (evidence/adopted-producer-20261118.json). r[lattice.nickel_import_merge.values]
- [x] [serial] Record exact patched source and evidence, update topic documentation, validate gates, and hand lifecycle closeout to integration. Evidence and `docs/nickel-eval-source.md` record the adopted producer; the Nix `nickel-cohort-exact` cohort guard, `scripts/check.rs` and `nextest` passed on the integrated candidate (`../qualify-native-graph-construction/evidence/final-candidate-observation.json`). r[lattice.nickel_import_merge.boundary]
