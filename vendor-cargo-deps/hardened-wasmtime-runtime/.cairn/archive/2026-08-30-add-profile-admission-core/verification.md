# Verification: Profile admission core and strategy-parity rail

## What was implemented

- Pure functional core (`src/lib.rs`, zero dependencies): execution
  strategies (Cranelift / Winch), resource-limit admission failing closed on
  sub-minimum bounds, and the normalized outcome model (completed /
  trapped-by-class / fuel-exhausted) with named constants throughout.
- Test-side strategy-parity rail (`src/parity/`): executes the bounded corpus
  under both strategies with fuel metering and compares the admitted
  observations through `differential-execution-core` pinned at `cad5a7cb…`.
- Strict Octet consumer gate: pinned `octet` input (`374bd16b…`), reviewed
  deny-all configuration, toolchain pin 1.97.1, `apps.octet-deny-all`, and
  `checks.octet` via `mkConsumerCheck`.
- Pinned Cairn lifecycle: upstream policy from `OnixResearch/cairn` at
  `b6759012…`, repository overlay, generated projection committed.

## Evidence

- `evidence/tests.txt` — 8/8 green: limits admission (positive + both
  zero-bound negatives), outcome-class distinctness, corpus agreement across
  strategies with stable per-program identities, forced divergence naming the
  mismatch surface, duplicate/single-backend rejection, oracle exact/drift.
- `evidence/octet-deny-all.txt` — `Status: clean`, `Findings: 0` over
  workspace / all-targets with the full reviewed lint catalog.
- `evidence/clippy.txt` — workspace clippy clean (exit 0).
- `evidence/cairn-validate.txt` — `cairn validate` valid, 2 specs validated.

## Environment notes

- The ambient installed `cairn` binary drifted mid-session and rejects
  previously valid policies; all lifecycle commands in this repository run
  through the pinned flake input (`b6759012…`), matching this repository's
  AGENTS guidance to avoid ambient host binaries.
- Wasmtime 41 removed the Pulley *strategy* (Pulley is now a compilation
  target); the second independent backend for the rail is therefore Winch,
  Wasmtime's baseline compiler — still a distinct codegen pipeline from
  Cranelift.

## Non-claims

- Strategy agreement is parity on the bounded corpus only. It does not prove
  correctness, WebAssembly conformance, sandbox effectiveness, or release
  eligibility, and both strategies share Wasmtime validation and store
  semantics.
- This rung does not implement restricted linking, component-model support,
  WASI, or the Aspen extraction; those belong to
  `extract-hardened-wasmtime-runtime`.
