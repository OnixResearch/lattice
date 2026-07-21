# Octet Baseline Migration

The changed orchestration and store cores shifted accepted finding identities. All new findings in the recovery and run-identity implementation were corrected before migration.

The reviewed migrations compared `octet-baseline.json` with `target/octet-precommit/results.jsonl` using the self-tested evidence tool archived at `cairn/archive/2026-07-20-integrate-animus-agent-runtime/evidence/migrate-octet-baseline.rs`.

First semantic comparison before execution:

```text
old=1172 candidate=1170 matched=1170 stale=2 new=0
STALE: too_many_parameters src/orchestration/core.rs:1254 function has 7 parameters (limit: 5) — consider an options struct
STALE: assertion_density src/orchestration/core.rs:1262 function `admit_trigger_start` has 0 assertion(s) in 25 lines (need 2+)
```

Both findings were resolved by replacing positional trigger-admission parameters with `TriggerStartRequest` and adding post-admission assertions.

After atomic run allocation and owner binding, a second semantic comparison reported:

```text
old=1170 candidate=1169 matched=1169 stale=1 new=0
STALE: assertion_density src/orchestration/shell.rs:222 function `execute_ready_round` has 0 assertion(s) in 47 lines (need 2+)
```

That accepted finding was resolved with assertions at the durable acknowledgment boundary. A final comment-only identity shift compared `old=1169 candidate=1169 matched=1169 stale=0 new=0`. The final migrated baseline passed with 919 accepted/current findings, zero new findings, zero stale findings, 1,169 total findings, and baseline hash `b3:b7b84338e5e0a9ba1d6ff41108e196fcb82345a2940a232b70f0a8d5d9aa034e`.
