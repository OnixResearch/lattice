# Observed-failure retry validation

## Scope

This evidence covers one explicit retry path for exact locally observed terminal workflow failures. It does not cover automatic retry loops, backoff, pending-unobserved redispatch, provider retry or cancellation, arbitrary kill safety, provider truth, actual token or billing measurement, remote effect deduplication, remote exactly-once execution, or production readiness.

## Checked boundaries

- `RetryPolicy` is optional, immutable through the definition revision, positive when present, and omitted when disabled.
- A retry ledger retains only the prior numeric run ID and BLAKE3 dispatch ID. It does not retain output, error, prompt, reasoning, credential, objective, actor, or provider content as retry authority.
- The shell creates retry eligibility only after loading an exact instance-and-dispatch-owned terminal `RunRecord` with a nonzero failure count.
- Headless shell errors without that record use a fixed content-omitting terminal summary and remain non-retryable.
- `PendingUnobserved`, abandoned, foreign-owner, and legacy-unscoped evidence is refused without redispatch.
- Pure preflight checks lifecycle, authority, enabled policy, retry count, exact retry evidence, and fresh multidimensional budget before numeric run-ID allocation.
- The accepted `RetryRound` transition atomically reserves fresh budget, installs fresh run and BLAKE3 dispatch identities, increments retry and round counts, clears consumed retry evidence, and enters `Running`.
- Optimistic version checks admit exactly one of two concurrent retry transitions from the same failed version.
- Retry events replay through the ordinary event chain and safe inspection joins them as dispatch starts without rendering private content.

## Focused evidence

The following post-change suites passed through the repository Nix development shell:

```text
cargo test --test test_orchestration_core
cargo test --test test_orchestration_shell
cargo test --test test_orchestration_cli
cargo test --test test_orchestration_format
cargo test --test test_orchestration_store concurrent_retry_transitions_admit_exactly_one_dispatch
cargo test --test test_run_inspection exact_retry_dispatch_remains_safe_and_inspectable
```

The focused cases prove positive retry execution with fresh run/dispatch identity, fresh budget settlement, valid replay, typed Nickel round-trip, CLI execution, exact safe inspection, and single-winner concurrent admission. Negative cases cover disabled and exhausted policy, unauthorized actor, budget exhaustion, pending-unobserved work, abandonment, shell failure without terminal run evidence, foreign ownership, and default-disabled compatibility.

## Adversarial review

A secondary invariant review challenged global run-ID uniqueness, mutable retry policy, automatic-loop risk, pending abandonment, budget ordering, crash handling, foreign ownership, and overclaiming. Repository evidence bounded those concerns as follows:

- `Store::next_run_id` is an atomic global reservation, and run records reject overwrite.
- The retry maximum is bound into the immutable definition revision retained by the instance.
- Retry is a single explicit CLI action; no automatic loop or backoff claim exists.
- Pending dispatches are rejected by retry and remain on recovery/cancel/abandon paths.
- Pure budget preflight occurs before run-ID allocation; the atomic transition repeats reservation checks before dispatch.
- A crash after retry admission leaves an ordinary pending dispatch, which remains non-retriable until exact recovery or explicit authorized abandonment.
- Foreign and legacy-unscoped run owners fail closed.

The reviewer reported no release-blocking issue. Its suggestions for backoff, pending timeout, and provider-wide exactly-once behavior are intentionally outside this bounded change.

## Octet review

The first Octet comparison reported identity drift plus genuine new source-shape findings. The implementation removed the genuine findings by introducing a typed `RunFinishedInput`, predicate-prefixed boolean bindings, pure retry-policy diagnostics, and explicit state assertions. A semantic migration then proved:

```text
old=1166 candidate=1163 matched=1162 stale=4 new=1
```

The four stale findings were resolved assertion-density/parameter-count debt. The sole new finding is `explicit_defaults` on `OrchestrationDefinition.retry`: serde must default the optional external `Option<RetryPolicy>` so schema-v1 definitions that omit retry remain compatible and keep their prior canonical identity. The retained Rust migration tool has positive and negative self-tests and allowlists exactly that lint, file, and message; it refuses every other new semantic finding.

After migration, `octet-precommit-check` retained 913 accepted/current findings, zero new, zero stale, 1,163 total findings, baseline hash `b3:ec5609b38db008f4145f4c38490b04290d8b52266e7b5a067d2cb29751ac208f`, and the known Cargo subprocess exit classification 101.

## Lifecycle and repository gates

Cairn validation plus proposal, design, and tasks gates passed before broad validation. The final `scripts/check.rs` run passed repository formatting, the complete test suite, strict project Clippy, and accepted Cairn spec checks after the source-shape fixes and baseline migration.

`nix flake check -L --option secret-key-files ''` passed all five bounded `x86_64-linux` checks: package build, nextest, strict Clippy, formatting, and the Octet consumer check. Nix reported only the bounded incompatible-system omission for `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin`.

The standalone `cairn tracey coverage --root .` command remains a known repository-wide configuration blocker rather than evidence for this change: `.config/tracey/config.styx` scans `crates/` and `tools/`, while Lattice implementation/test markers live under `src/` and `tests/`. It therefore reported `referenced: 0` and all 692 accepted requirements missing, including every previously accepted requirement. This change does not treat that output as retry-specific failure or as successful trace coverage; the fix is a separate repository-wide Tracey source-root correction.
