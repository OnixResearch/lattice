# Verification: adopt-nickel-1-17-cohort

r[verify lattice.nickel_toolchain.cohort]
r[verify lattice.nickel_toolchain.fixture_cohort]
r[verify lattice.nickel_toolchain.boundary]
r[verify lattice.nickel_toolchain.compatibility]
r[verify lattice.nickel_toolchain.validation]

## Implemented boundary

- Lattice pins Nickel CLI `1.17.0` to upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`.
- Cargo and Nix pin the archived `nickel-eval` Radicle object `f23be09e41337de8fa729274f50023f861ae0e8a`.
- Direct fixtures use `nickel-lang-core 0.18.0`; the shared evaluator uses `nickel-lang 2.2.0`.
- The existing Lattice adapter still evaluates and decodes before workflow or Choregraph admission.
- Source guards reject the prior object, prior direct core, floating branches, GitHub fallback, and sibling paths.

## Positive evidence

- `nix build .#checks.x86_64-linux.nickel-cohort-exact --no-link -L --option builders ''`: passed.
- `nix develop -c cargo test nickel::tests --lib`: 5 passed.
- `nix develop -c cargo test --test test_nickel_eval_adoption`: 4 passed.
- `nix develop -c cargo clippy -p lattice --lib --test test_nickel_eval_adoption --no-deps -- -D warnings`: passed.
- `nix develop -c octet-precommit-check`: passed after guarded semantic identity migration. The migration matched all 846 Lattice and 23 lattice-core accepted semantic findings with no unmatched findings.
- `scripts/check.rs`: passed, including operator E2E checks, scoped formatting, focused consumer compilation, all tests, Clippy, and Cairn specification checks.

## Negative evidence

- Malformed source, missing import, contract rejection, unknown-field policy, excessive-bound policy, floating source, and GitHub fallback cases are rejected.
- The Nix cohort check initially rejected its own old-version search literal. The guard was narrowed to product manifests and documentation, then passed.
- The direct Nickel cohort fixture initially failed to compile against the removed `Program::new_from_file` API. It now uses the `ProgramBuilder` API from `nickel-lang-core 0.18.0` and passes.

## Repository-wide observations

- The repository intentionally keeps `tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs` unformatted. The repository-scoped format check excludes it and verifies that exclusion.
- The repository Clippy rail checks owned Lattice targets with `--no-deps`; vendored source retains separate ownership and evidence.

## Non-claims

These checks establish the selected cohort identity and bounded fixture compatibility. They do not prove Nickel correctness, arbitrary import completeness, policy correctness, Choregraph correctness, effect success, or release readiness beyond the recorded gates.
