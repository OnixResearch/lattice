# Tasks: Profile admission core and strategy-parity rail

> Keep raw evidence under `.cairn/changes/add-profile-admission-core/evidence/` and summarize replay steps in `verification.md` at closeout.

## Phase 1: Repository bootstrap

- [x] [serial] T1.1 Pin the Cairn lifecycle: import upstream policy files from `OnixResearch/cairn` at `b6759012…`, add this repository's overlay, generate `cairn-policy/generated/cairn-policy.json`, and record the pinned-rev usage note. r[hardened_wasmtime.profile_admission]
  - Trace: Proposal (bootstrap)
  - Positive evidence: `policy export` succeeds with the pinned Cairn flake entry; `validate` parses the policy
  - Negative evidence: the ambient installed binary drifts mid-session and is NOT used for lifecycle commands
- [x] [serial] T1.2 Bootstrap the strict Octet gate: pinned `octet` input, reviewed deny-all configuration, toolchain pin, workspace lint catalog, `apps.octet-deny-all`, and `checks.octet` through `mkConsumerCheck`. r[hardened_wasmtime.profile_admission]
  - Trace: Proposal SC-04
  - Positive evidence: `nix run .#octet-deny-all -- --workspace -- --all-targets` runs the deny-all hook
  - Negative evidence: a lint violation fails the gate with a named finding

## Phase 2: Pure core

- [x] [serial] T2.1 Implement strategy enum, resource-limit admission failing closed on zero bounds, and the normalized outcome model (completed / trapped by class / fuel exhausted) with named constants for every bound. r[hardened_wasmtime.profile_admission] r[hardened_wasmtime.outcome_model]
  - Trace: Proposal SC-01, SC-02
  - Positive evidence: positive admission fixtures round-trip exact values; outcome classification maps completed, unreachable trap, and exhaustion to distinct outcomes
  - Negative evidence: zero fuel and zero memory-page bounds fail with named field errors

## Phase 3: Strategy-parity rail through the harness

- [x] [serial] T3.1 Add the test-side execution shell over Wasmtime with fuel metering for both strategies, plus a bounded corpus of small modules (arithmetic completion, unreachable trap, fuel exhaustion). r[hardened_wasmtime.outcome_model] r[hardened_wasmtime.strategy_parity]
  - Trace: Proposal SC-02, SC-03
  - Positive evidence: every corpus module executes under both strategies and normalizes to the expected outcome class
  - Negative evidence: a module run under an insufficient fuel budget normalizes to fuel-exhausted, not a trap
- [x] [serial] T3.2 Admit both strategy observations per case through the pinned `differential-execution-core` (`cad5a7cb…`) and compare with `compare_pairwise`; assert corpus-wide agreement and stable per-program case identities. r[hardened_wasmtime.strategy_parity]
  - Trace: Proposal SC-03
  - Positive evidence: corpus fixture reports `Agreement` for every case with reproducible identities
  - Negative evidence: forced divergence reports `Divergence` naming the mismatch surface
- [x] [serial] T3.3 Add independence-gate and oracle-classification fixtures (duplicate/single backend rejection; exact match versus drifted side). r[hardened_wasmtime.independence] r[hardened_wasmtime.oracle]
  - Trace: Proposal SC-04
  - Evidence: all four negative/positive fixtures green

## Validation

- [x] [serial] V1. Run the crate test suite and record the transcript. r[hardened_wasmtime.strategy_parity]
  - Raw evidence path: `evidence/tests.txt`
- [x] [serial] V2. Run the Octet deny-all hook over the workspace and record the transcript. r[hardened_wasmtime.profile_admission]
  - Raw evidence path: `evidence/octet-deny-all.txt`
- [x] [serial] V3. Run workspace clippy with `-D warnings` and record the transcript. r[hardened_wasmtime.outcome_model]
  - Raw evidence path: `evidence/clippy.txt`
- [x] [serial] V4. Run `cairn validate --root .` with the pinned Cairn and record the receipt. r[hardened_wasmtime.independence]
  - Raw evidence path: `evidence/cairn-validate.txt`

## Lifecycle / Closeout

- [x] [serial] State whether this change is ready to archive or blocked by a named remaining item. r[hardened_wasmtime.oracle]
  - Remaining blocker or archive-ready note: archive-ready — all implementation, fixture, gate, and lifecycle evidence recorded green
  - Evidence: `verification.md`
