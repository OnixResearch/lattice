## Phase 1: Baseline and process harness

- [x] [serial] Run the existing focused duplicate-trigger shell test before changing the suite and record whether it passes. r[lattice.agent_orchestration.triggers]
- [x] [serial] Add a self-spawned two-process fixture that performs fresh admission, process exit, store reopen, and exact duplicate submission. r[lattice.agent_orchestration.triggers]
- [x] [serial] Prove the final store has one admission, one instance, one run, one settled round charge, no pending reservation, and valid replay. r[lattice.agent_orchestration.triggers]
- [x] [serial] Add a negative child-phase case that exits unsuccessfully without creating the durable store. r[lattice.agent_orchestration.triggers]

## Phase 2: Evidence and closure

- [x] [serial] Record bounded process identities, returned instance equality, durable cardinalities, budget settlement, replay validity, and explicit non-claims without retaining private values or raw store dumps. r[lattice.agent_orchestration.triggers]
- [x] [serial] Run focused post-change tests, repository checks, Octet comparison, Cairn gates/validation, and Nix checks; sync the accepted scenario and archive the change. r[lattice.agent_orchestration.triggers]

## Verification evidence

- The focused restart filter passed its positive two-process fixture and negative invalid-phase fixture. The complete orchestration-shell and orchestration-store suites passed 11 and 5 tests respectively.
- `scripts/check.rs` passed the repository test suite, strict project Clippy, and accepted/change requirement references.
- `octet-precommit-check` retained 916 accepted/current findings, zero new, zero stale, 1,166 total findings, and baseline hash `b3:1aeb6f42e5bbb4bf92fbb1b16fbc2a05fe03e997ac30a3b7ba2b00b63ffab8f0`; no rebaseline was needed.
- `nix flake check -L --option secret-key-files ''` passed all bounded `x86_64-linux` checks. Cairn validation plus proposal, design, and tasks gates passed.
- The live Animus/VibeThinker restart drill returned the exact original failed-terminal instance from a separately launched duplicate process with one admission, one instance, one run, one consumed round/invocation, and valid replay. The safe report passed private-marker scans, and all temporary probe/store artifacts were deleted after bounded evidence was recorded.
- Secondary adversarial review led to direct assertions for distinct process IDs, the exact scoped admission key, exact run ownership, every consumed budget dimension, closed dispatch state, and invalid-phase non-creation.
- No provider-success, provider-cancellation, remote exactly-once, arbitrary kill-point, contiguous-sequence, externally anchored ledger, billing/token-truth, or production-readiness claim is made.

## Baseline evidence

- `nix develop -c cargo test --test test_orchestration_shell duplicate_trigger_returns_the_original_instance_without_another_run` passed before the new process-boundary fixture was added.
