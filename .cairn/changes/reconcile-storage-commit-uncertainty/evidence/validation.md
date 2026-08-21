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

## Octet evidence

A parent-revision control reproduced the repository-wide stable-identity drift. The reviewed migration preserved 846 root findings and 23 pure-core findings with zero baseline growth.

`nix develop -c octet-precommit-check` passes with zero new and zero stale findings in both scopes. See `evidence/octet-migration.md`.

## Full validation

- `./scripts/check.rs`: all tests, scoped Clippy, and Cairn gates passed after two evidence corrections:
  - restored the intentionally nonconforming vendor fixture bytes and refreshed the byte-bound traceability hashes for `Cargo.toml`, `README.md`, and `src/store/mod.rs`;
  - made the analyzer truncation fixture deterministic by ignoring a child write error that raced the bounded reader.
- `nix flake check -L --option secret-key-files ''`: all checks passed, including 1,066 nextest tests.

The earlier full repository check observed one operation-pack truncation mismatch. Its immediate focused reruns passed; the deterministic fixture fix removed the race.

## Boundaries

The quarantine token is process-local. `runs repair` does not clear it or establish workflow truth.

The evidence does not prove arbitrary Redb failures, power-loss behavior, external effect truth, exactly-once execution, or production readiness.
