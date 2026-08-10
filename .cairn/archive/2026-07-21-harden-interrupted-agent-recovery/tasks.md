## Phase 1: Lifecycle and core

- [x] [serial] Record the live interrupted/successful dogfood evidence, exact claim boundary, alternatives, and the explicit abandonment contract. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Replace scan-based run allocation with an atomic persisted reservation counter and reject duplicate run-record writes without overwriting history. r[lattice.agent_orchestration.run_identity]
- [x] [serial] Bind orchestration run records to exact instance/dispatch owners with legacy-unscoped decoding and route the orchestration headless shell through that owner. r[lattice.agent_orchestration.run_identity]
- [x] [serial] Add the pure abandonment action, event, authority/cancellation/identity validation, acknowledgment-sensitive budget settlement, and fixed terminal failure state. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Add positive and negative core tests for acknowledged and planned abandonment, missing cancellation, mismatched dispatch, unauthorized actor, duplicate/terminal facts, and deterministic replay. r[lattice.agent_orchestration.unobserved_abandon]

## Phase 2: Shell and operator controls

- [x] [serial] Add positive and negative tests for concurrent pre-persistence allocations, duplicate-record rejection, exact owner recovery, and foreign/legacy owner rejection. r[lattice.agent_orchestration.run_identity]
- [x] [serial] Add the shell boundary that refuses abandonment when a matching persisted run exists and commits only the unobserved core fact. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Add `orchestration abandon` with actor and text/JSON output while preserving existing recovery and cancellation commands. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Add shell and CLI tests for the two-phase recover/cancel/abandon flow, observed-run rejection, unauthorized use, repeat invocation, and safe output. r[lattice.agent_orchestration.unobserved_abandon]
- [x] [serial] Document the operator sequence, conservative budget semantics, and explicit remote-effect non-claims. r[lattice.agent_orchestration.unobserved_abandon]

## Phase 3: Validation and closure

- [x] [serial] Run focused tests, `scripts/check.rs`, Octet, Cairn gates/validation, and bounded Nix validation; sync the accepted spec and archive with current evidence. r[lattice.agent_orchestration.unobserved_abandon]

## Verification evidence

- Pre-change focused orchestration baselines passed 13 core tests, 6 shell tests, and the existing CLI suite.
- Post-change focused store, headless, core, shell, and CLI suites passed, including concurrent pre-persistence allocation, duplicate write rejection, legacy-unscoped decoding, exact owner recovery, foreign owner rejection, acknowledged/planned abandonment, authority, mismatch, duplicate, terminal, and replay paths.
- Production-shaped live dogfood reserved run `0`, terminated the wrapper during dispatch, restarted with distinct run `1`, completed that run, then cancelled and abandoned run `0` without retry; abandonment failed terminally, consumed the full reservation, replayed validly, and returned `NothingPending` from later recovery.
- `scripts/check.rs` passed the full default-member test suite and strict Clippy; `nix flake check -L --option secret-key-files ''` passed 483 nextest cases and all bounded checks on `x86_64-linux`.
- `octet-precommit-check` passed with 919 accepted/current findings, zero new, zero stale, 1,169 total findings, and baseline hash `b3:b7b84338e5e0a9ba1d6ff41108e196fcb82345a2940a232b70f0a8d5d9aa034e`.
- Cairn validation plus proposal, design, and tasks gates passed through the canonical sibling Cairn CLI.
- No provider cancellation, provider failure, interrupted-workflow success, actual billing/token usage, remote exactly-once execution, external anchoring, or production-readiness claim is made.
