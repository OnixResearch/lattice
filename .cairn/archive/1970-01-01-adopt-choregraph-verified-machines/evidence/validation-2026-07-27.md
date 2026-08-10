# Choregraph machine-cutover validation — 2026-07-27

## Passed

- `nix develop -c cargo test --locked --test test_runner_core`: 21 passed.
- `nix develop -c cargo test --locked --test test_choregraph_migration`: 3 passed.
- `scripts/check.rs`: passed after one transient loopback webhook reset; the exact failed test passed alone and the complete script then passed.
- `nix develop -c cargo clippy --locked --package lattice --all-targets --no-deps -- -D warnings`: passed.
- `nix build .#checks.x86_64-linux.choregraph-source --no-link -L --option secret-key-files ''`: passed, including the expected failing floating-source fixture.
- `nix flake check -L --option secret-key-files ''`: exited successfully.
- Cairn validation, proposal gate, and design gate: passed.
- isolated rollback worktree at `c8d56ad83a57287064edb85f58808f53e4bf4536`: 21 runner tests passed.

The repository-validation evidence hashes were updated for the changed `Cargo.toml` and `README.md` bytes. The intentional nonconforming nested vendor fixtures remain unchanged.

## Open Octet gate

`cargo octet check` exits with status 2 before lint execution:

```text
error: rustc 1.96.0-nightly is not supported by the following packages:
  choregraph-core@0.1.0 requires rustc 1.97.1
  choregraph-seaglass@0.1.0 requires rustc 1.97.1
```

The selected Octet package owns `nightly-2026-03-21`. Lattice's normal toolchain is `nightly-2026-07-17` and compiles the exact Choregraph source. The pre-commit wrapper and Nix derivation report this as `integration-failure` with zero findings, but they currently return success under the existing accepted-baseline policy. This report does not treat that result as a passing Octet gate.

Lattice does not override Octet's rustc-private toolchain or claim lint coverage from a run that never compiled the consumer. The maintained-adoption-validation task remains open until an accepted Octet source supports Rust 1.97.1 or newer.

## Open lowering gate

Direct `choregraph-nickel` integration remains blocked by the exact BLAKE3 dependency conflict recorded in `nickel-lowering-blocker-2026-07-27.md`. The pure-lowering task remains open.

## Resolution — 2026-07-28

The compiler and dependency blockers are resolved by exact published Choregraph revision `81b260da2256147ec82cb5dd44c0a0c1dcb7500d`.

Octet now analyzes the full configured library scope. Its current run reports 1,141 finding occurrences. The migration check matched all 1,141 old semantic findings and found zero additions or removals. The maintained wrapper accepts 892 current stable identities with zero new and zero stale identities.

The raw Octet child still uses deny-level lints and therefore records `integration-failure` after emitting the complete accepted finding set. The maintained wrapper passes by exact baseline policy. This evidence does not call the raw result clean.

The pure lowering implementation and current validation are recorded in `lowering-validation-2026-07-28.md`.

## Claim boundary

These results support the checked machine cutover only. They do not prove whole-Lattice correctness, effect safety, verifier soundness, successful Choregraph Nickel lowering, complete Octet coverage, or release eligibility.
