# Proposal: Profile admission core and strategy-parity rail

## Why

`hardened-wasmtime-runtime` is a planned extraction with no implementation. The founding package (`extract-hardened-wasmtime-runtime`) scopes the full hardened component pulled from Aspen; that extraction is a large reviewed effort. This change delivers the first standalone rung that needs no Aspen code: a pure, deterministic core for execution-profile admission and normalized outcome classification, plus a strategy-parity rail that proves the outcome model on a bounded corpus by executing the same WebAssembly modules under two independent Wasmtime compilation strategies — Cranelift (JIT) and Pulley (portable interpreter) — and comparing the admitted observations through `differential-execution-core`.

The rail is evidence machinery, not the product claim: agreement between two Wasmtime strategies establishes parity of the outcome model on the corpus only, and does not prove sandbox effectiveness, correctness, or release eligibility.

## What Changes

- Add a `no_std`-friendly functional core: explicit execution strategies, resource-limit admission that fails closed on zero or absent bounds, and a normalized outcome model (completed, trapped by class, fuel exhausted) with named constants for every bound.
- Add a thin test-side shell that executes a bounded corpus of small WebAssembly modules under both strategies with fuel metering enabled, normalizes results and traps, and admits both observations through the pinned `differential-execution-core` (independent-backend gate, pairwise comparison, oracle classification fixtures).
- Bootstrap the repository lifecycle: pinned Cairn policy (upstream `b6759012…` plus this repo's overlay) and this change package.
- Bootstrap the strict Octet consumer gate: pinned `octet` input, deny-all configuration, toolchain pin, and Nix checks.

## Impact

- **Files**: `Cargo.toml`, `rust-toolchain.toml`, `flake.nix`, `dylint.toml`, `config/octet-deny-all.json`, `src/` (core + test-side rail), `cairn-policy/`, `.cairn/changes/add-profile-admission-core/`
- **Testing**: positive corpus agreement across strategies; negative fixtures for limit admission, forced divergence, duplicate/single-backend independence, and oracle drift; `cargo octet check` deny-all; `nix flake check`
- **Consumers**: future extraction work in this repository; Aspen and later adapters own product policy

## Success Criteria

1. **SC-01 — Limits admission fails closed**: zero or absent fuel/memory bounds are rejected with named errors; admitted limits expose exact values.
2. **SC-02 — Outcome normalization is total over the corpus**: completed, unreachable-trap, and fuel-exhaustion executions map to distinct normalized outcomes with stable classes.
3. **SC-03 — Strategy parity through the harness**: every corpus module executes under both strategies with `Agreement` through `admit_independent_backends` + `compare_pairwise`; case identities are stable and distinct per program.
4. **SC-04 — Gates fail closed**: forced divergence names the mismatch surface; duplicate/single-backend admission and oracle drift are rejected; Octet deny-all and workspace clippy are clean.

## Expected Verification

- Replay: `cargo test` (corpus + fixtures), `nix run .#octet-deny-all -- --workspace -- --all-targets`, `nix flake check -L`, `cairn validate --root .` with the pinned Cairn.
- Raw evidence path: `.cairn/changes/add-profile-admission-core/evidence/`.
- Non-claim: strategy agreement is parity on the bounded corpus only.

## Non-Goals

- No Aspen extraction, restricted linking, component-model support, WASI, or product adapter in this change.
- No claim that Cranelift/Pulley agreement proves WebAssembly conformance.
