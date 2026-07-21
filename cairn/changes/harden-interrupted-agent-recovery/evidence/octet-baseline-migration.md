# Octet Baseline Migration

The changed orchestration core initially shifted accepted finding identities. All new findings in the recovery implementation were corrected before migration.

The reviewed migration compared `octet-baseline.json` with `target/octet-precommit/results.jsonl` using the self-tested evidence tool archived at `cairn/archive/2026-07-20-integrate-animus-agent-runtime/evidence/migrate-octet-baseline.rs`.

Final semantic comparison before execution:

```text
old=1172 candidate=1170 matched=1170 stale=2 new=0
STALE: too_many_parameters src/orchestration/core.rs:1254 function has 7 parameters (limit: 5) — consider an options struct
STALE: assertion_density src/orchestration/core.rs:1262 function `admit_trigger_start` has 0 assertion(s) in 25 lines (need 2+)
```

Both stale findings were resolved by replacing positional trigger-admission parameters with `TriggerStartRequest` and adding post-admission assertions. The migrated baseline then passed with 920 accepted/current findings, zero new findings, zero stale findings, 1,170 total findings, and baseline hash `b3:62c44137c89f1a8ba85a087e8e0a61ebe47ae57431bf9fdc19623c62cf1765a2`.
