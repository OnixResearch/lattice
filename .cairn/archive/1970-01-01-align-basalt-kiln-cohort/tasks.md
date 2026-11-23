## Phase 1: Cohort import

- [x] [serial] Identify the cohort divergence: `b34d6ac2` = `4c5c9e9` plus three unpublished holder commits that Lattice calls. Export them as a patch series for Basalt. r[lattice.repository_validation.basalt_cohort.scenario.provenance]
- [x] [serial] Sync candidate `6ab48bf` (`5e0f1f4c` plus the revised holder series) through `scripts/sync-vendored-sibling.rs`. Verify that the imported bytes equal `git archive 6ab48bf` for the synced paths, then update `vendor-cargo-deps/README.md`. r[lattice.repository_validation.basalt_cohort]
- [x] [serial] Update `Cargo.lock` only as the vendored manifests require, and record every crate version change. r[lattice.repository_validation.basalt_cohort]

## Phase 2: Holder exchange adaptation

- [x] [serial] Pass the typed native Mantle request and the endpoint's `BasaltPolicyRef` to Basalt's holder delivery. Keep Lattice's pre-effect subject, service, project, operation and resource checks, and its verification of the issued grant. r[lattice.repository_validation.basalt_cohort.holder_exchange]
- [x] [serial] Add a real-exchange consumer test for the issued exact holder grant, and for denial of a second exchange with the same OIDC token. r[lattice.repository_validation.basalt_cohort.holder_exchange.scenario.issued]
- [x] [serial] In the same test, refuse a mismatched subject, project, service and operation without any producer effect. r[lattice.repository_validation.basalt_cohort.holder_exchange.scenario.rejected]

## Phase 3: Verification

- [x] [serial] Run `cargo check -p lattice --all-targets`, the focused library tests (authority, OIDC, GitHub CI, effect authority, operation shell, workflow exchange) and the integration tests (Choregraph effect authority, GitHub CI Mantle, headless, role-protocol authority). r[lattice.repository_validation.basalt_cohort]
- [x] [serial] Run `cargo clippy -p lattice --no-deps --all-targets -- -D warnings`. r[lattice.repository_validation.basalt_cohort]
- [x] [serial] Re-key Octet once in the relocation integration. r[lattice.repository_validation.basalt_cohort]
- [x] [serial] Re-sync at published Basalt `main` `fac0d7f1`. Confirm that its Rust code equals the candidate's (only doc-comment wording differs), update `SOURCE` and the README pin, and rerun `cargo check -p lattice --all-targets` and the exchange tests. r[lattice.repository_validation.basalt_cohort.scenario.provenance]

## Closeout verification

- Octet baselines were admitted for the relocation cohort in `ab38ab89` (`octet-baseline.json` and `crates/lattice-core/octet-baseline.json`).
- Re-verified at `08f07749`: `octet-precommit-check` exited 0 (0 new / 0 stale on `lattice` and `lattice-core`), `scripts/check.rs` exited 0, and `cairn validate --strict` reported 0 findings.
