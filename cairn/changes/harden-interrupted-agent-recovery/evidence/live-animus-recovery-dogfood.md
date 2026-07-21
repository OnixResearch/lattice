# Live Animus Recovery Dogfood

## Scope

- **Date:** 2026-07-21 local time
- **Published baseline:** `c7779f4`
- **Endpoint:** reachable local OpenAI-compatible VibeThinker service, model `VibeThinker-3B`
- **Credentials:** none supplied
- **Store:** isolated `/tmp/lattice-animus-dogfood/data/lattice.redb`
- **Workflow revision:** `d6236edd0281eef69e340620f542cb1181fddb0af344e9e52b09597afd54b0e1`
- **Definition revision:** `751b4f6c4b6554e089c321593be3aa774204f762e09b9fcb6011a3c7350d8e8d`

## Observations

1. Two direct headless executions reached the real endpoint and succeeded as runs `0` and `1`, each with one completed node and no failures. Both reported zero cache hits; this does not establish remote idempotency.
2. Orchestration instance `555d173a837a54f7795dc139a64e0395b6c0932046f59fc2f841b807392bd475` was terminated with process status `124` eight seconds after start, after dispatch acknowledgment and during the provider call for run `2`.
3. Two recovery attempts both returned `PendingUnobserved { run_id: 2 }`. No automatic retry or replacement run occurred.
4. Replay remained valid while the instance was running with one acknowledged pending dispatch and one held reservation. Consumed budget remained zero because no terminal local run observation existed.
5. Authorized cancellation set `cancellation_requested` but correctly did not claim that the remote effect was cancelled. The instance and reservation remained pending, exposing the closure gap addressed by this change.
6. A separate uninterrupted instance `1c6a63e5e36dc2bb3980a88ed44148a5778a3bc83aa76cce4c2c1108c940f50d` completed run `2`, cleared its reservation, consumed the configured upper bound, replayed validly, and returned `NothingPending` from recovery.

## Portfolio of mechanisms

| Family | Mechanism | Result |
|---|---|---|
| Explicit abandonment | Authorized two-phase cancel then abandon, no retry, terminal local failure, conservative acknowledged settlement | Selected and implemented |
| Automatic retry | Reinvoke the same or a replacement dispatch after missing local observation | Rejected: can duplicate a completed remote effect |
| Timeout inference | Mark remote failure/cancellation after elapsed time | Rejected: elapsed time cannot establish remote outcome |
| Cancellation as proof | Clear state immediately when cancellation is requested | Rejected: local intent is not remote observation |
| Provider idempotency | Retry under a provider-enforced idempotency key | Blocked: absent from the generic provider contract and not portable |

## Adversarial audit

The surviving mechanism was audited against an active-executor race, an observed `RunRecord`, unauthorized actors, missing cancellation, dispatch/run mismatch, repeated facts, terminal mutation, zero acknowledged settlement, fabricated completion, and replay drift. The command requires the operator to stop the original executor first; Lattice cannot prove distributed process liveness or provider cancellation. The shell refuses abandonment if a matching run is already persisted, while the core preserves authority, identity, settlement, and replay invariants.

## Additional friction outside this change

- `--init` overwrites an existing config path; the first isolated setup attempt therefore fell back to the default user store. The imported workflow and failed run contained no credentials, but no-clobber initialization remains a separate operator-safety follow-up.
- Headless execution persists summaries and replay records but not the TUI-style run-detail record, so inspecting typed node outputs required a temporary read-only probe. Headless run inspection remains a separate observability follow-up.
- The local provider returned visible reasoning markup in normal response content despite thinking being disabled in configuration. Lattice did not fall back to a hidden reasoning field; provider-content policy remains a separate concern.

## Non-claims

This dogfood does not prove provider cancellation, provider failure, workflow success for the interrupted call, actual token or billing usage, remote exactly-once execution, external event anchoring, live endpoint quality, or production readiness.
