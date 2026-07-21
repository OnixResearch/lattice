## Phase 1: Baseline and classification

- [x] [serial] Run focused safe-inspection and restart-idempotency tests before changing run-store handling. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Add a pure redb open-error classifier for active-owner, recovery-required, and unavailable states with fixed content-omitting diagnostics. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Keep `runs show` read-only and prove a failed recovery-required inspection does not change the store's BLAKE3 identity. r[lattice.workflow_execution.safe_run_inspection]

## Phase 2: Explicit recovery

- [x] [serial] Add existing-store-only `runs repair` using redb-owned write-mode recovery without opening or creating Lattice application records. r[lattice.workflow_execution.run_store_recovery]
- [x] [serial] Add positive clean/panic/abort recovery fixtures and negative active-owner, missing-store, unknown-child-phase, and private-error omission cases. r[lattice.workflow_execution.run_store_recovery]
- [x] [serial] Document diagnosis, explicit mutation, retry flow, and non-claim boundaries. r[lattice.workflow_execution.run_store_recovery]

## Phase 3: Validation and closure

- [x] [serial] Run focused post-change tests, repository checks, Octet comparison, Cairn gates/validation, and Nix checks; sync accepted requirements and archive the change with bounded evidence. r[lattice.workflow_execution.run_store_recovery]

## Verification evidence

- The safe-inspection suite passed 11 tests and the new process-recovery suite passed 5 positive/negative tests covering clean exit, panic unwind, abrupt abort, active ownership, missing store, malformed store, unknown child phase, BLAKE3 non-mutation, explicit repair, already-clean receipt, exact record retention, and fixed private-content-omitting diagnostics.
- `scripts/check.rs` passed the repository test suite, strict project Clippy, and accepted/change requirement references.
- `octet-precommit-check` retained 916 accepted/current findings, zero new, zero stale, 1,166 total findings, and baseline hash `b3:1aeb6f42e5bbb4bf92fbb1b16fbc2a05fe03e997ac30a3b7ba2b00b63ffab8f0`; no rebaseline was needed.
- `nix flake check -L --option secret-key-files ''` passed all bounded `x86_64-linux` checks. Cairn validation plus proposal, design, and tasks gates passed.
- Secondary adversarial review identified that a generic success receipt could overstate work on an already-clean store. A redb repair callback now distinguishes fixed `recovery completed` and `already clean` outcomes; both are covered by process tests.
- Broad host-style all-target Clippy remains blocked by pre-existing vendored Rats wildcard-import and Animus lint-expectation findings; the repository-owned strict Clippy rail passed through `scripts/check.rs` and Nix.
- No application semantic correctness, complete-history, arbitrary-corruption, external-anchoring, provider-truth, remote-exactly-once, or production-readiness claim is made.

## Baseline evidence

- The existing restart-idempotency filter passed before changes.
- `nix develop -c cargo test --test test_run_inspection` passed all 11 safe-inspection tests before implementation.
