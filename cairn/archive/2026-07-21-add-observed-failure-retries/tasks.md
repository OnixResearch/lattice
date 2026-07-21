# Tasks: Add bounded observed-failure retries

- [x] [serial] Add backward-compatible optional retry policy and retry ledger types with pure validation and canonical omission of defaults. r[lattice.agent_orchestration.observed_failure_retry]
- [x] [serial] Record retry eligibility only from exact persisted failed run evidence; keep shell errors, abandonment, and foreign records non-retryable. r[lattice.agent_orchestration.observed_failure_retry]
- [x] [serial] Implement pure preflight and atomic `RetryRound` transition with authority, policy, identity, count, workflow-revision, and fresh-budget checks. r[lattice.agent_orchestration.retry_round]
- [x] [serial] Add the thin `orchestration retry` shell and CLI path without automatic loops or pending-dispatch fallback. r[lattice.agent_orchestration.retry_round]
- [x] [serial] Extend safe inspection joins, typed Nickel rendering, examples, and operator documentation without exposing private content. r[lattice.agent_orchestration.retry_round]
- [x] [serial] Add positive core/shell/CLI/format/replay tests and negative disabled, exhausted, unauthorized, budget, pending-unobserved, abandoned, shell-error, foreign-owner, and compatibility tests. r[lattice.agent_orchestration.retry_round]
- [x] [serial] Run focused post-change suites, repository checks, Octet comparison, Cairn gates/validation, and Nix checks; record bounded evidence. r[lattice.agent_orchestration.retry_round]
- [x] [serial] Sync accepted requirements, archive the completed change, commit incrementally, and push the current branch. r[lattice.agent_orchestration.retry_round]

## Baseline evidence

- `main` was clean and synchronized with `origin/main` at `063db71` after publishing post-failure run-store recovery.
- Before retry implementation, `nix develop -c cargo test --test test_orchestration_core --test test_orchestration_shell --test test_orchestration_cli --test test_orchestration_format` passed all selected integration suites, including 11 orchestration-shell tests.
- Existing accepted behavior explicitly forbids retry of `PendingUnobserved` and foreign/legacy run evidence; this change preserves those boundaries.
