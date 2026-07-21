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
2. Orchestration instance `555d173a837a54f7795dc139a64e0395b6c0932046f59fc2f841b807392bd475` allocated run `2`. A wrapper returned process status `124` eight seconds after start, after dispatch acknowledgment and during the provider call.
3. Two immediate recovery attempts both returned `PendingUnobserved { run_id: 2 }`. Replay remained valid, no automatic retry occurred, and authorized cancellation left the uncertain dispatch and reservation pending.
4. After the first wrapper exited without a completion record, a separate orchestration process also allocated run `2`, completed another provider call, and persisted that numeric record. This proved that `next_run_id` was a read-only `max(persisted) + 1` query rather than an allocation and that a restart could reuse an interrupted dispatch's identity.
5. The first implementation of `orchestration abandon` correctly refused once a record existed, but the pre-owner recovery model could not prove which dispatch created that record. Any apparent reconciliation of the first instance is therefore invalid evidence and MUST NOT be treated as verified completion or cancellation.
6. This counterexample expanded the fix: reserve run IDs atomically in redb, reject duplicate record writes, persist exact orchestration instance/dispatch ownership, and reject foreign or legacy-unscoped records during recovery and abandonment.
7. A separate uninterrupted instance `1c6a63e5e36dc2bb3980a88ed44148a5778a3bc83aa76cce4c2c1108c940f50d` did complete and replay validly, but its collided numeric run identity is retained only as evidence of the allocator defect—not as proof that the interrupted instance completed.
8. A deliberate simultaneous second process against the same redb file failed with `Database already open`; the observed identity bug is a crash/restart reservation failure, while in-process allocators still require transactional serialization.

## Post-fix live validation

A fresh store at `/tmp/lattice-animus-dogfood-v3/data/lattice.redb` exercised the corrected binary:

1. The first orchestration atomically reserved run `0`, persisted its acknowledged dispatch, and was wrapper-terminated after eight seconds. Immediate recovery returned `PendingUnobserved { run_id: 0 }`.
2. A restarted orchestration used run `1`—not run `0`—and completed one live provider call. Its terminal summary recorded `run:1`.
3. The original instance was explicitly cancelled and abandoned. It terminated as `failed`, cleared active and pending identities, conservatively consumed the complete acknowledged reservation, and recorded `local dispatch abandoned after acknowledgement; remote outcome unknown`.
4. Replay returned `valid: true`, reconstructed the same terminal state and empty reservation set, and subsequent recovery returned `NothingPending`.

## Portfolio of mechanisms

| Family | Mechanism | Result |
|---|---|---|
| Atomic counter + owner binding | Reserve numeric ID in a write transaction, reject duplicate writes, bind record to instance/dispatch | Selected and implemented |
| Completion-record scan | Infer next ID from persisted records | Falsified by live concurrent run-2 allocation |
| Random/BLAKE3 run IDs | Replace `u64` run identity throughout the runtime | Deferred: broader compatibility change than required |
| Placeholder run record | Persist a pending record before dispatch | Rejected for this slice: conflates allocation with observed completion unless a larger run-state schema is introduced |
| Explicit abandonment | Authorized two-phase cancel then abandon, no retry, terminal local failure, conservative acknowledged settlement | Selected after exact owner absence is established |
| Automatic retry | Reinvoke the same or replacement dispatch after missing local observation | Rejected: can duplicate a completed remote effect |
| Timeout inference | Mark remote failure/cancellation after elapsed time | Rejected: elapsed time cannot establish remote outcome |
| Provider idempotency | Retry under a provider-enforced idempotency key | Blocked: absent from the generic provider contract and not portable |

## Adversarial audit

The final mechanism is checked against concurrent allocation before persistence, explicit high-ID records, duplicate writes, legacy-unscoped records, wrong instance/dispatch ownership, active-executor races, matching observed records, unauthorized actors, missing cancellation, dispatch/run mismatch, repeated facts, terminal mutation, zero acknowledged settlement, fabricated completion, and replay drift.

An operator still must stop the original executor before abandonment. Lattice does not prove distributed process liveness or provider cancellation. If any record occupies the numeric run identity but owner correlation fails, both recovery and abandonment fail closed for investigation.

## Additional friction outside this change

- `--init` overwrites an existing config path; the first isolated setup attempt therefore fell back to the default user store. The imported workflow and failed run contained no credentials, but no-clobber initialization remains a separate operator-safety follow-up.
- Headless execution persists summaries and replay records but not the TUI-style run-detail record, so inspecting typed node outputs required a temporary read-only probe. Headless run inspection remains a separate observability follow-up.
- The local provider returned visible reasoning markup in normal response content despite thinking being disabled in configuration. Lattice did not fall back to a hidden reasoning field; provider-content policy remains a separate concern.

## Non-claims

This dogfood does not prove provider cancellation, provider failure, workflow success for the interrupted call, actual token or billing usage, remote exactly-once execution, external event anchoring, live endpoint quality, or production readiness.
