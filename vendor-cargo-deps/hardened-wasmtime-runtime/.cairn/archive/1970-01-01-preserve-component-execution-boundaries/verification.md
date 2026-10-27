# Verification — component execution boundaries — 2026-10-26

## Outcome and ownership

The producer now exposes `hwr.shell.v4`: typed initialization yields, a single successful instantiation checkpoint before export execution, and independent positive per-memory and cumulative memory ceilings. Baseline was immutable producer `6381322a68407da71325c4c33113d11ada9af67f`; Wasmtime remains 45.0.0.

Immediate outcome: the previously ambiguous initialization boundary and conflated memory inputs are explicit mechanisms. Durable contribution: actual allocation/start-work/cancellation regressions plus a versioned consumer contract. Hardened-runtime maintainers own this mechanism; the current consumer is Molten's frozen adoption parity corpus at `ce5873f2`. Molten owns product profile interpretation, callback ledgers, canonical receipts, effects, authority and adoption. Its immutable dependency update and independent rerun remain consumer-owned publication handoff actions, not claims made by this archive.

## Executed checks

All commands ran from the producer implementation worktree using its pinned Nix development tools, executable task-owned `/dev/shm` scratch and separate Cargo/Octet targets.

| Command / surface | Observed result |
|---|---|
| `cargo test --workspace --all-targets` | 8 core, 30 admission and 44 actual module/component execution cases passed; no failures or ignored cases. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed after the final allocation-module split. |
| `cargo fmt --all --check` | Passed. |
| Pinned `octet-deny-all -- --workspace -- --all-targets` | Clean; zero findings, warnings or errors. |
| Native Cairn proposal, design and tasks gates | PASS, with known requirement references. |
| Native `cairn validate --root . --strict` | Valid, no findings or issues. |
| `nix flake check path:$PWD -L --no-update-lock-file --builders '' --max-jobs 2` | All three host checks passed: build, tests and deny-all Octet. Other architectures were omitted, not verified. |

Nix evaluated an existing app-metadata warning. The default remote build hook first failed because its gRPC URI exceeded the lock-file name limit; disabling remote builders allowed the same host rails to execute locally. Kache reported a pre-existing cache-index I/O problem and compiled without cache hits/stores. Neither condition was suppressed in product code.

Octet's initial private-module name repetition, resource-test local quantity name and oversized resource-test file were repaired by a private module rename, clearer admission input name and a responsibility-based allocation-test split. No lint allowance or policy relaxation was added. The temporary smoke initially exposed a local helper shadow and missing crate documentation; both were fixed before the successful run and the executable was then removed.

## Public API execution, outside the test runner

A temporary Cargo example constructed the admitted engine, ran a real component with finite start work, polled the public retained invocation and compared its complete outcome with synchronous execution. It then executed a real module whose memory grows by one page under three independently supplied cap pairs. Exact observed output:

```text
checkpoint remaining=9517 start-yields=7
schema=hwr.shell.v4 checkpoints=1 class=Completed output=[79, 75] remaining=9501 sync-parity=true
per-memory=2 aggregate=1 class=MemoryLimitExceeded
per-memory=1 aggregate=2 class=MemoryLimitExceeded
per-memory=2 aggregate=2 class=Completed
scope=public-api-smoke product-adoption=false authority=false
```

The owned smoke source was removed. Permanent tests additionally observe a host marker to prove no selected export executes before the checkpoint; retain the only permit at the checkpoint; cancel and successfully reacquire it; reject trapping/fuel-exhausted initialization without a checkpoint; and compare successful same-store output/fuel through cleanup. The fixed profile reader's original cumulative ceiling is exercised by actual two-memory allocation/growth, not a copied-field assertion.

## Handoff and non-claims

Publish the completed correction to canonical Radicle main, then let Molten select the immutable revision in Cargo, Nix, release evidence, metadata-free source hashes and both regenerated plans. Rerun the original start-work and two-129-page-memory cases with complete product receipt/ledger equality and the paired resource denials. No source-path override is publication evidence.

This producer result does not establish production Molten adoption, real actor/extension installation, matching Mantle bundles, product effect-world authorization, operated product deadlines, remote seed replication, arbitrary guest correctness, host isolation or release readiness. The older failed-run synchronous/sliced fuel caveat remains a consumer compatibility fact; no failed-run readback is relabelled as successful fuel.
