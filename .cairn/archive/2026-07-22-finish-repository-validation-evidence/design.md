# Design: finish repository-validation evidence

## Success contract

All 11 reviewed repository-validation rows MUST be exact and BLAKE3-bound while the 315-item missing set and zero dangling state remain unchanged. The compiled fixture MUST be admitted by Git and the filtered Nix source even though its nested output-like path, formatting, and Clippy policy would expose a broadened check scope.

## Functional core and shell

A pure contract evaluator receives README, flake, Cargo, ignore, fixture, and test-source text and returns bounded issue codes. Thin test shells invoke Git ignore classification and public Cargo/Nix checks. Positive assertions establish the intended contract; mutated inputs establish fail-closed behavior without modifying repository state.

## Evidence model

- Signing evidence combines the maintained check-only command, absence of flake-level signing policy, and a full invocation under a configured missing key.
- Source evidence combines a configured dev path dependency, compile-time use of its nested source, tracked JSON parsing, root-anchored ignore rules, and filtered `tests/` admission.
- Ownership evidence combines an intentionally nonconforming fixture with exact package-scoped formatter and strict `--no-deps` Clippy arguments.

## Non-claims

The checks are bounded to the maintained fixture and current Nix/Cargo configuration. They do not prove arbitrary upstream formatting or lint compatibility, arbitrary Git-ignore behavior, host signing correctness, release readiness, or production readiness.
