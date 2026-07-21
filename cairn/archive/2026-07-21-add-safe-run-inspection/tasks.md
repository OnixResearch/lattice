## Phase 1: Safe inspection core

- [x] [serial] Define a serialization-stable run inspection report containing only bounded identifiers, enums, booleans, timestamps, counts, budget dimensions, and diagnostic codes. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Project run details into node status/count evidence without labels, values, or error text. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Validate present run replay records against exact run identity and existing replay invariants while distinguishing absent historical evidence. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Correlate orchestration-owned runs to one exact instance/dispatch/run tuple and derive dispatch, recovery, abandonment, and budget evidence. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Adversarially test malformed detail identity, foreign ownership, invalid run replay, invalid orchestration replay, duplicate dispatch admission, and private-content omission. r[lattice.workflow_execution.safe_run_inspection]

## Phase 2: Read-only shell and CLI

- [x] [serial] Add a thin store shell that loads one persisted run plus optional details/replay and exact orchestration evidence without mutating state. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Add `runs show <run-id> --output text|json` with successful rendering, explicit not-found failure, and unsuccessful exit for invalid present evidence. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Add end-to-end CLI fixtures for unscoped and orchestration-owned runs and assert that prompt, reasoning, credential, output, error, actor, objective, and trigger-key markers never appear. r[lattice.workflow_execution.safe_run_inspection]
- [x] [serial] Document the command, evidence states, and non-claim boundaries. r[lattice.workflow_execution.safe_run_inspection]

## Phase 3: Verification and lifecycle

- [x] [serial] Run focused post-change tests, repository checks, Octet comparison, Cairn gates/validation, and Nix checks; sync the accepted spec and archive the change with evidence. r[lattice.workflow_execution.safe_run_inspection]

## Verification evidence

- The focused safe-inspection suite passed 11 positive and negative cases covering unscoped and orchestration-owned CLI output, settled and conservatively abandoned budgets, exact and foreign owner tuples, duplicate dispatch admission, malformed owner identity omission, replay/detail identity mismatch, legacy absence, not-found behavior, invalid-evidence exits, and missing-store non-creation.
- The selected post-change run-history, orchestration CLI, store, and safe-inspection suites passed. `scripts/check.rs` passed the repository test suite, strict project Clippy, and accepted-spec references.
- `octet-precommit-check` passed with 916 accepted/current findings, zero new, zero stale, 1,166 total findings, and baseline hash `b3:1aeb6f42e5bbb4bf92fbb1b16fbc2a05fe03e997ac30a3b7ba2b00b63ffab8f0`; no rebaseline was needed.
- `nix flake check -L --option secret-key-files ''` passed all bounded `x86_64-linux` checks. Cairn validation plus proposal, design, and tasks gates passed through the canonical sibling Cairn CLI.
- Adversarial review rejected direct store serialization, TUI renderer reuse, and loose orchestration joining. It additionally verified that malformed owner strings are replaced with `[invalid]`, missing stores do not create data directories or databases, and every serialized string is either a validated BLAKE3 identity or a fixed enum/marker.
- The broad host-style `cargo clippy -p lattice --all-targets -- -D warnings` remains unsuitable because it traverses vendored Rats wildcard-import debt; the repository-owned strict Clippy rail passed through `scripts/check.rs` and Nix.
- No raw output/error recovery, unobserved-dispatch run synthesis, provider truth, billing/token truth, remote exactly-once effect, externally anchored completeness, or production-readiness claim is made.

## Baseline evidence

- Before implementation, `nix develop -c cargo test --test test_run_history_details --test test_orchestration_cli --test test_store` passed all selected suites; the final visible suite reported 29 passing store cases with zero failures.
- No active Cairn changes or pre-existing worktree modifications were present.
