## Context and Success Contract

The current Octet run emits 1,141 findings represented by 892 unique stable IDs. Every explicit lint level is `deny`, so rustc exits `101`; Octet correctly refuses to write a baseline after an unsuccessful Cargo child. The checked baseline cannot repair this because baseline evaluation occurs after Cargo exits. A clean-source deterministic comparison reviewed all 1,141 ordered records. Exactly 1,140 retain the same lint, group, file, line, column, message, crate, and identity status. The remaining record is the same `src/config/mod.rs` file-length observation reduced from 676 to 657 lines by already accepted source. The broad ID transition is caused by Octet binding `Cargo.toml` and `dylint.toml` into its config hash.

Completion evidence is a successful warning-level complete Cargo child, a runner-generated 892-ID baseline, zero new/stale IDs under the canonical wrapper, rejection of an injected new finding, exact traceability inventory regeneration, and repository/Cairn validation.

False completion includes hand-authoring accepted IDs, allowing or disabling lints, treating a warning-only direct run as the canonical gate, ignoring non-Octet compiler failure, accepting aggregate-count equality without record equality, or treating inventory refresh as requirement coverage.

## Approach Registry

| Family | Mechanism | State | Evidence / blocker |
|---|---|---|---|
| Replace IDs only | Rewrite the stale baseline while retaining deny levels | Falsified | Cargo still exits `101`; Octet refuses baseline generation and reports integration failure before admission. |
| Upstream deny-baseline inference | Teach Octet to distinguish accepted Dylint errors from all other rustc failures | Blocked | Requires a new upstream semantic contract and unpublished Octet revision; misclassification could hide real compiler errors. |
| Disable or allow findings | Suppress current debt | Rejected | Weakens discovery and violates the no-disabled/no-allow boundary. |
| Warning observation plus ID gate | Run every lint at warning level and make the checked baseline wrapper the admission authority | Selected | Cargo completes, Octet writes canonical identities, all findings remain observable, and a permanent new-ID fixture can prove fail-closed admission. |

Independent advisory review agreed the ID gate can preserve new-finding rejection but requested an explicit injected-finding negative check and documentation of the warning-level boundary. Its suggestion to retain an arbitrary deny lint was rejected because any accepted finding under that lint would recreate the same pre-baseline Cargo failure; the wrapper's deterministic negative fixture is the relevant admission proof.

## Octet Gate Design

`dylint.toml` retains the complete explicit lint inventory and changes each level from `deny` to `warn`. `disabled_lints` and `allow` remain absent. This is an observation configuration, not release acceptance.

`octet-precommit-check` remains the canonical shell. Its existing deterministic status predicate requires a complete Cargo child, a nonempty current set, at least one accepted current identity, and zero new identities. The refreshed baseline is emitted only by `cargo-octet --write-baseline` after the warning-level Cargo child succeeds.

The Nix check first runs the positive wrapper against the immutable checked source. It then creates a writable negative source copy, adds one repository-owned `todo!()` function, runs the same wrapper with the same baseline, and requires nonzero exit plus at least one new identity. The negative path must not replace the baseline or mutate the positive source.

The pure admission core remains Octet's stable-ID set comparison. Lattice's shell only selects the checked baseline, checks structured status, and orchestrates isolated positive/negative runs.

## Traceability Inventory Design

The typed review policy binds `expected_missing_count = 12` and `expected_missing_hash = "c23fc013acdb1e4ae5d093b1f01ddb8c82e7d744ce8f9989f3841ea56e18c457"`. The repository-owned Rust generator consumes the exact failing Cairn coverage receipt, validates sorted uniqueness and reviewed policy membership, reads only safe bounded evidence files, and atomically rewrites the generated report.

All 12 rows remain `unassessed` unless already explicitly reviewed. The inventory is not a Tracey source and does not change the failing coverage result. A stale count/hash, duplicate input, unsafe evidence path, tampered report, or future missing-set transition remains fail closed.

The older repository-validation completion fixture is a separate exact-byte review surface. The accepted Radicle cutover changed `Cargo.toml`, `README.md`, and the historical projection of `tests/test_repository_validation.rs` without changing those 11 completed requirements' classification, rationale, evidence paths, or coverage. Their evidence hashes are refreshed only after the completion test proves every current byte binding and the existing mutation test proves drift rejection.

## Validation Sequence

1. Preserve the old baseline and traceability identities and record the failing statuses.
2. Change only Octet observation levels, generate a runner-owned candidate, and compare complete record sets and unique counts.
3. Install the reviewed candidate and prove positive admission plus injected new-ID rejection.
4. Refresh the exact traceability policy/report and prove repeated check determinism plus stale-input rejection.
5. Refresh the accepted repository-validation completion byte identities without changing decisions or coverage, then run its positive and negative tests.
6. Run formatting, repository checks, Cairn validation/gates, and the bounded full flake check before sync/archive.

## Risks and Non-Claims

A direct `cargo octet check` now reports warnings and can exit successfully; it is not the canonical repository gate. Stable-ID admission can only prove membership in the reviewed static finding set. It does not prove lint correctness, accepted finding safety, source semantics, runtime behavior, complete static analysis, traceability satisfaction, or release readiness.
