# Storage commit uncertainty baseline

## Candidate

- Repository: `OnixResearch/lattice`
- Revision: `c513d94`
- Worktree branch: `cairn/reconcile-storage-commit-uncertainty-20260821`

## Command

```text
nix develop -c cargo test --test test_scheduler_occurrences --test test_scheduler --test test_workflow_approval --test test_run_store_recovery --test test_choregraph_effect_authority
```

## Result

- Scheduler occurrence core and shell: 14 passed.
- Scheduler daemon and operator paths: 14 passed.
- Workflow effect authority and reservation paths: 20 passed.
- Run-store recovery and repair paths: 5 passed.
- Approval suspension, crash, and resume paths: 14 passed and one helper remained ignored by design.
- Total: 67 passed, zero failed, and one expected ignored helper.

## Current boundary

Redb commit errors currently become ordinary `StoreError::Db` or `StoreError::Unavailable` values. The focused suites do not inject an uncertain commit result before effect dispatch.

This baseline proves only that the selected existing tests pass at revision `c513d94`. It does not prove safe handling of an uncertain Redb commit.
