## Phase 1: Source repair

- [x] [serial] Diagnose the minimal imported-container merge at the actual core invariant and compare the pinned external CLI value. r[lattice.nickel_import_merge.values]
- [x] [serial] Prepare the owned evaluator source repair preserving existing thunk reversion and constant allocation behavior, after lifecycle admission. r[lattice.nickel_import_merge.values]
- [ ] [serial] Admit exact immutable patched evaluator source and align Cargo/Nix integration without changing the language cohort or adding fallbacks. r[lattice.nickel_import_merge.boundary]

## Phase 2: Consumer evidence

- [x] [parallel] Add complete-value native regressions for both operand orders, nested overlap, and recursive overriding. r[lattice.nickel_import_merge.values]
- [x] [parallel] Add malformed/missing import and contract rejection regressions asserting content-omitting safe categories. r[lattice.nickel_import_merge.boundary]
- [ ] [serial] Exercise the admitted native path, compare complete JSON value with the pinned CLI, and run focused regressions. r[lattice.nickel_import_merge.values]
- [ ] [serial] Record exact patched source and evidence, update topic documentation, validate gates, and hand lifecycle closeout to integration. r[lattice.nickel_import_merge.boundary]
