# Storage commit uncertainty validation

## Shared core

- RID: `rad:z4Tky6zvC8w4Y6c4YBzNxVbq5n752`
- Revision: `eb2bd3441753af97bfcb247cef7cc22d72675b62`
- Cargo, Nix, and the source-agreement check select the same revision.

## Focused results

- Baseline: 67 passed, zero failed, one expected ignored helper.
- Shared-core adapter tests: exact publication, uncertainty quarantine, exact reopen, and missing reservation passed.
- Scheduler uncertainty test: post-commit observation loss quarantined, blocked retry, exact successor cleared, and missing row stayed blocked.
- Scheduler occurrence suite: 15 tests passed.
- Scheduler daemon and operator suite: 14 tests passed.
- Repository-owned Clippy with `--no-deps` and `-D warnings`: passed.

## Selected fault classes

- Definite pre-commit drop.
- Commit outcome unknown before observed publication.
- Commit succeeds and its observation is lost.
- Exact successor read-back.
- Missing-row read-back.
- Retry while quarantined.

## Completion blocker

The final Octet gate reports 846 new and 846 stale stable identities, with 232 unmatched semantic records. No baseline migration was applied.

The full repository check observed one operation-pack truncation mismatch. Its immediate focused rerun passed without a source change.

## Boundaries

The quarantine token is process-local. `runs repair` does not clear it or establish workflow truth.

The evidence does not prove arbitrary Redb failures, power-loss behavior, external effect truth, exactly-once execution, or production readiness.
