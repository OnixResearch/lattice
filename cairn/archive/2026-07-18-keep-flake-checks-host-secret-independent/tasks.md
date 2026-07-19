## Phase 1: Source and check boundaries

- [x] [serial] r[n8nclone.repository_validation.vendor_snapshot.scenario.nested-output-name] Anchor repository output ignores and admit required nested Rats source files.
- [x] [serial] r[n8nclone.repository_validation.test_source.scenario.json-fixture] Include tracked non-Rust test fixtures in the filtered Nix source closure.
- [x] [serial] r[n8nclone.repository_validation.format_scope.scenario.vendor-format] Restrict the Crane format check to the repository-owned package and validate the focused derivation.
- [x] [serial] r[n8nclone.repository_validation.clippy_scope.scenario.vendor-lints] Restrict the Crane Clippy check to repository-owned targets and validate the focused derivation.
- [x] [serial] r[n8nclone.repository_validation.flake_check.scenario.missing-signing-key] Document and run the full check-scoped flake validation command without the host signing secret. Evidence: `nix flake check -L --option secret-key-files ''` completed with `all checks passed!`; the baseline plain invocation failed at the configured missing `/run/secrets/vars/nix-signing-key/key`.

## Phase 2: Validation and lifecycle

- [x] [serial] r[n8nclone.repository_validation.flake_check] Run repository checks, validate positive and negative ignore behavior, sync accepted requirements, archive the change, and commit locally without pushing. Evidence: root outputs remain ignored, required nested vendor sources are tracked and not ignored, no ignored vendor files remain, `scripts/check.rs` passes, and Octet remains exact at 831/831.
